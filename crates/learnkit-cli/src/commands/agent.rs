use clap::{Args, Subcommand};
use learnkit_agent::catalog::{is_supported, supported_agents};
use learnkit_agent::install::install_agent;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_store::project::read_project;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    action: AgentAction,
}

#[derive(Subcommand)]
enum AgentAction {
    /// Install (or reinstall) a single agent's integration files.
    Install(InstallArgs),
}

#[derive(Args)]
struct InstallArgs {
    /// Agent to install (e.g. `codex`, `claude`).
    agent_id: String,

    /// Project folder (defaults to the current directory).
    #[arg(long)]
    path: Option<PathBuf>,

    /// Emit machine-readable JSON instead of human-readable output.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct AgentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installed_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_agents: Option<Vec<String>>,
}

pub fn run(args: AgentArgs) -> i32 {
    match args.action {
        AgentAction::Install(install_args) => run_install(install_args),
    }
}

fn run_install(args: InstallArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let json = args.json;

    match execute(&root, &args.agent_id) {
        Ok(installed_files) => {
            info!(agent_id = %args.agent_id, "agent install succeeded");
            if json {
                Envelope::ok(
                    "AGENT_INTEGRATION_INSTALLED",
                    AgentData {
                        agent_id: Some(args.agent_id.clone()),
                        installed_files: Some(installed_files),
                        ..Default::default()
                    },
                )
                .print_json();
            } else {
                println!("Integración de '{}' instalada.", args.agent_id);
            }
            0
        }
        Err(err) => {
            warn!(agent_id = %args.agent_id, code = err.code(), "agent install failed");
            let data = match &err {
                LearnKitError::AgentUnsupported { supported, .. } => AgentData {
                    supported_agents: Some(supported.clone()),
                    ..Default::default()
                },
                LearnKitError::AgentFilesModified { files } => AgentData {
                    agent_id: Some(args.agent_id.clone()),
                    modified_files: Some(files.clone()),
                    ..Default::default()
                },
                _ => AgentData::default(),
            };
            if json {
                Envelope::err(err.code(), data).print_json();
            } else {
                eprintln!("Error: {err}");
            }
            err.exit_code()
        }
    }
}

fn execute(root: &Path, agent_id: &str) -> Result<Vec<String>, LearnKitError> {
    if !is_supported(agent_id) {
        return Err(LearnKitError::AgentUnsupported {
            requested: agent_id.to_string(),
            supported: supported_agents(),
        });
    }

    let project = read_project(root).map_err(|source| LearnKitError::Filesystem {
        path: root.display().to_string(),
        source,
    })?;
    if project.is_none() {
        return Err(LearnKitError::ProjectNotInitialized {
            path: root.display().to_string(),
        });
    }

    let outcome = install_agent(root, agent_id).map_err(|source| LearnKitError::Filesystem {
        path: root.display().to_string(),
        source,
    })?;

    if outcome.is_blocked() {
        return Err(LearnKitError::AgentFilesModified {
            files: outcome.modified_files,
        });
    }

    Ok(outcome.installed_files)
}
