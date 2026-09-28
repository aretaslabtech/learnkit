use clap::Args;
use learnkit_core::error::LearnKitError;
use learnkit_store::session_paths::SessionPaths;
use learnkit_store::status::build_status;
use learnkit_workflow::engine::{read_manifest, recompute_state, PhaseState};
use learnkit_workflow::phases::phase_definitions;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct StatusArgs {
    /// Project folder to inspect (defaults to the current directory).
    #[arg(long)]
    path: Option<PathBuf>,

    /// If given, also report the workflow phase states for this session.
    #[arg(long)]
    session: Option<String>,

    /// Emit machine-readable JSON instead of human-readable output.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct PhaseStatus {
    phase: String,
    state: String,
}

pub fn run(args: StatusArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    if !root.exists() {
        eprintln!("Error: la ruta '{}' no existe", root.display());
        return 2;
    }

    match build_status(&root) {
        Ok(report) => {
            let phase_statuses = args
                .session
                .as_deref()
                .map(|session_id| session_phase_statuses(&root, session_id));

            if args.json {
                print_json(&report, phase_statuses.as_deref());
            } else if report.ok {
                println!(
                    "Proyecto LearnKit válido (perfil: {}, agentes: {}, shell: {}).",
                    report.profile_id.as_deref().unwrap_or("?"),
                    if report.installed_agents.is_empty() {
                        "ninguno".to_string()
                    } else {
                        report.installed_agents.join(", ")
                    },
                    report.shell_preference.as_deref().unwrap_or("?")
                );
                if let Some(phases) = &phase_statuses {
                    for p in phases {
                        println!("  fase {}: {}", p.phase, p.state);
                    }
                }
            } else {
                println!(
                    "No hay un proyecto LearnKit válido en esta carpeta ({}). Ejecuta 'learnkit init'.",
                    report.code
                );
            }
            0
        }
        Err(source) => {
            let err = LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            };
            eprintln!("Error: {err}");
            err.exit_code()
        }
    }
}

fn session_phase_statuses(project_root: &Path, session_id: &str) -> Vec<PhaseStatus> {
    let paths = SessionPaths::new(project_root, session_id);
    let defs = phase_definitions();
    let mut states: HashMap<String, PhaseState> = HashMap::new();
    let mut ordered = Vec::new();

    // Phases are declared in dependency order already (phase_definitions()
    // lists them so that a phase never appears before what it requires);
    // evaluating in that order lets each phase see its dependencies' freshly
    // computed states, never a stale cached one (Hard Guards).
    for def in &defs {
        let manifest_path = paths.phase_manifest(&def.id);
        let manifest = read_manifest(&manifest_path).ok().flatten();
        let current_input_fingerprint = manifest
            .as_ref()
            .map(|m| m.output_fingerprint.clone())
            .unwrap_or_default();
        let state = recompute_state(
            &def.requires,
            &states,
            manifest.as_ref(),
            &current_input_fingerprint,
        );
        states.insert(def.id.clone(), state);
        ordered.push(PhaseStatus {
            phase: def.id.clone(),
            state: format!("{state:?}").to_lowercase(),
        });
    }

    ordered
}

fn print_json(report: &learnkit_store::status::StatusReport, phases: Option<&[PhaseStatus]>) {
    let mut value = match serde_json::to_value(report) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("failed to serialize --json output: {err}");
            return;
        }
    };
    if let (Some(phases), Some(obj)) = (phases, value.as_object_mut()) {
        obj.insert(
            "phases".to_string(),
            serde_json::to_value(phases).unwrap_or_default(),
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_default()
    );
}
