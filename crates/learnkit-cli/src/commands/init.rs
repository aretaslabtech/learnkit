use crate::interactive::{is_interactive_context, prompt_agents, prompt_profile, prompt_shell};
use clap::Args;
use learnkit_agent::catalog::{
    is_supported as agent_is_supported, supported_agents, DEFAULT_AGENT,
};
use learnkit_agent::install::install_agent;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_core::shell::{
    detect_default as detect_default_shell, is_supported as shell_is_supported, supported_shells,
};
use learnkit_profile::catalog::DEFAULT_PROFILE;
use learnkit_profile::profile::resolve as resolve_profile;
use learnkit_store::init::scaffold_project;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Args)]
pub struct InitArgs {
    /// Target folder (defaults to the current directory).
    path: Option<PathBuf>,

    /// Domain profile to activate. If omitted, prompted interactively (TTY)
    /// or defaulted to `generic` (non-interactive) — FR-003.
    #[arg(long)]
    profile: Option<String>,

    /// Comma-separated agent ids to install (e.g. `codex,claude`). If
    /// omitted, prompted interactively (TTY) or defaulted to `claude`
    /// (non-interactive) — FR-004. An explicit empty value is rejected.
    #[arg(long)]
    agents: Option<String>,

    /// Shell preference to save in `learnkit.toml` (purely informational —
    /// FR-015). If omitted, prompted interactively (TTY) or auto-detected
    /// from the OS (non-interactive).
    #[arg(long)]
    shell: Option<String>,

    /// Emit machine-readable JSON instead of human-readable output. Always
    /// treated as non-interactive (FR-013).
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct InitData {
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installed_agents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shell_preference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_profiles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_agents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_shells: Option<Vec<String>>,
}

struct InitResult {
    profile_id: String,
    installed_agents: Vec<String>,
    shell_preference: String,
}

pub fn run(args: InitArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let json = args.json;

    match execute(&root, &args) {
        Ok(result) => {
            info!(
                profile_id = %result.profile_id,
                installed_agents = ?result.installed_agents,
                shell_preference = %result.shell_preference,
                "learnkit init succeeded"
            );
            if json {
                Envelope::ok(
                    "PROJECT_INITIALIZED",
                    InitData {
                        profile_id: Some(result.profile_id),
                        installed_agents: Some(result.installed_agents),
                        shell_preference: Some(result.shell_preference),
                        ..Default::default()
                    },
                )
                .print_json();
            } else {
                println!(
                    "Proyecto LearnKit inicializado (perfil: {}, agentes: {}, shell: {}).",
                    result.profile_id,
                    result.installed_agents.join(", "),
                    result.shell_preference
                );
            }
            0
        }
        Err(err) => {
            warn!(code = err.code(), "learnkit init failed");
            let data = match &err {
                LearnKitError::ProfileUnsupported { supported, .. } => InitData {
                    supported_profiles: Some(supported.clone()),
                    ..Default::default()
                },
                LearnKitError::AgentUnsupported { supported, .. } => InitData {
                    supported_agents: Some(supported.clone()),
                    ..Default::default()
                },
                LearnKitError::ShellUnsupported { supported, .. } => InitData {
                    supported_shells: Some(supported.clone()),
                    ..Default::default()
                },
                _ => InitData::default(),
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

fn execute(root: &Path, args: &InitArgs) -> Result<InitResult, LearnKitError> {
    let interactive = is_interactive_context(args.json);

    let profile_id = match &args.profile {
        Some(p) => resolve_profile(p)?,
        None if interactive => prompt_profile()?,
        None => DEFAULT_PROFILE.to_string(),
    };

    let agent_ids = match &args.agents {
        Some(raw) => parse_agents(raw)?,
        None if interactive => prompt_agents()?,
        None => vec![DEFAULT_AGENT.to_string()],
    };

    let shell_preference = match &args.shell {
        Some(s) => resolve_shell(s)?,
        None if interactive => prompt_shell()?,
        None => detect_default_shell().to_string(),
    };

    scaffold_project(root, &profile_id, &shell_preference).map_err(|source| {
        LearnKitError::Filesystem {
            path: root.display().to_string(),
            source,
        }
    })?;

    let mut installed_agents = Vec::with_capacity(agent_ids.len());
    for agent_id in &agent_ids {
        install_agent(root, agent_id).map_err(|source| LearnKitError::Filesystem {
            path: root.display().to_string(),
            source,
        })?;
        installed_agents.push(agent_id.clone());
    }

    Ok(InitResult {
        profile_id,
        installed_agents,
        shell_preference,
    })
}

fn parse_agents(raw: &str) -> Result<Vec<String>, LearnKitError> {
    let ids: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if ids.is_empty() {
        return Err(LearnKitError::NoAgentSelected);
    }

    for id in &ids {
        if !agent_is_supported(id) {
            return Err(LearnKitError::AgentUnsupported {
                requested: id.clone(),
                supported: supported_agents(),
            });
        }
    }

    Ok(ids)
}

fn resolve_shell(shell: &str) -> Result<String, LearnKitError> {
    if shell_is_supported(shell) {
        Ok(shell.to_string())
    } else {
        Err(LearnKitError::ShellUnsupported {
            requested: shell.to_string(),
            supported: supported_shells(),
        })
    }
}
