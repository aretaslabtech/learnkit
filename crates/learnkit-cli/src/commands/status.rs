use clap::Args;
use learnkit_core::error::LearnKitError;
use learnkit_store::session_paths::SessionPaths;
use learnkit_store::status::build_status;
use learnkit_workflow::engine::{
    read_manifest, recompute_checklist_item_state, recompute_state, PhaseState,
};
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
    /// Per-item live state for a checklist-bearing phase (`analyse`), per
    /// `contracts/cli-commands.md` → `learnkit status`. `None` for a phase
    /// without a checklist (`inventory`, `vocabulary`, ...) — no `checklist`
    /// key is emitted in JSON for those, preserving the feature 002 output
    /// shape exactly (FR-003).
    #[serde(skip_serializing_if = "Option::is_none")]
    checklist: Option<Vec<ChecklistItemStatus>>,
}

#[derive(Debug, Serialize)]
struct ChecklistItemStatus {
    item_id: String,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_reason: Option<String>,
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
                        if let Some(items) = &p.checklist {
                            for item in items {
                                match &item.pending_reason {
                                    Some(reason) => println!(
                                        "    - {}: {} ({})",
                                        item.item_id, item.state, reason
                                    ),
                                    None => {
                                        println!("    - {}: {}", item.item_id, item.state)
                                    }
                                }
                            }
                        }
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

/// Renders a `PhaseState`/`ChecklistItemState` using its own
/// `#[serde(rename_all = "snake_case")]` code (e.g. `PendingUserDecision` ->
/// `pending_user_decision`), instead of hand-rolling `Debug`-then-lowercase
/// — which silently produced `pendinguserdecision`/`needsuserinput` for
/// multi-word variants (found while implementing US3's `flag-pending`/`skip`,
/// feature 003: `contracts/cli-commands.md` documents `pending_user_decision`
/// as the exact wire value).
fn state_code<T: serde::Serialize>(state: &T) -> String {
    match serde_json::to_value(state) {
        Ok(serde_json::Value::String(s)) => s,
        _ => format!("{:?}", std::any::type_name::<T>()),
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
        // Per-item live state (US1, FR-002/FR-004): each item is recomputed
        // against the phase's current fingerprint rather than trusting its
        // stored `state`, same cascading-invalidation rule `recompute_state`
        // already applies at phase granularity. No better per-item source
        // fingerprint exists yet (that lands with real per-item fingerprints
        // in a later task), so this reuses the phase-level
        // `current_input_fingerprint` already computed above.
        let checklist = manifest.as_ref().and_then(|m| m.checklist.as_ref()).map(|items| {
            items
                .iter()
                .map(|item| {
                    let live_state =
                        recompute_checklist_item_state(item, &current_input_fingerprint);
                    ChecklistItemStatus {
                        item_id: item.item_id.clone(),
                        state: state_code(&live_state),
                        pending_reason: item.pending_reason.clone(),
                    }
                })
                .collect()
        });
        states.insert(def.id.clone(), state);
        ordered.push(PhaseStatus {
            phase: def.id.clone(),
            state: state_code(&state),
            checklist,
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
