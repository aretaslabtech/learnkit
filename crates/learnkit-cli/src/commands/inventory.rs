use crate::session_context::resolve_session;
use clap::Args;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_store::session_paths::SessionPaths;
use learnkit_workflow::engine::{write_manifest, PhaseManifest, PhaseResult};
use learnkit_workflow::inventory::{ingest, inventory, sources_fingerprint, InventoryChange};
use serde::Serialize;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Args)]
pub struct IngestArgs {
    files: Vec<PathBuf>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct InventoryArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct SourceReport {
    id: String,
    kind: String,
    path: String,
    sha256: String,
    change: String,
}

#[derive(Debug, Serialize, Default)]
struct InventoryData {
    #[serde(skip_serializing_if = "Option::is_none")]
    sources: Option<Vec<SourceReport>>,
}

pub fn run_ingest(args: IngestArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_root = root.join("sessions").join(&session_id);

    match ingest(&session_root, &args.files) {
        Ok(()) => {
            if args.json {
                Envelope::ok("SOURCES_INGESTED", InventoryData::default()).print_json();
            } else {
                println!(
                    "{} fichero(s) ingerido(s) en la sesión {}.",
                    args.files.len(),
                    session_id
                );
            }
            0
        }
        Err(source) => emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: session_root.display().to_string(),
                source,
            },
        ),
    }
}

pub fn run_inventory(args: InventoryArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    match inventory(&root, &session_id) {
        Ok(results) => {
            let reports: Vec<SourceReport> = results
                .into_iter()
                .map(|r| SourceReport {
                    id: r.source.id,
                    kind: format!("{:?}", r.source.kind).to_lowercase(),
                    path: r.source.path,
                    sha256: r.source.sha256,
                    change: match r.change {
                        InventoryChange::Added => "added".to_string(),
                        InventoryChange::Unchanged => "unchanged".to_string(),
                        InventoryChange::Updated => "updated".to_string(),
                    },
                })
                .collect();

            let fingerprint =
                sources_fingerprint(reports.iter().map(|r| (r.id.as_str(), r.sha256.as_str())));
            let paths = SessionPaths::new(&root, &session_id);
            let manifest = PhaseManifest {
                phase: "inventory".to_string(),
                input_fingerprint: String::new(),
                output_fingerprint: fingerprint,
                validated_at: format!("{:?}", SystemTime::now()),
                result: PhaseResult::Valid,
                checklist: None,
            };
            let _ = write_manifest(&paths.phase_manifest("inventory"), &manifest);

            if args.json {
                Envelope::ok(
                    "INVENTORY_COMPLETE",
                    InventoryData {
                        sources: Some(reports),
                    },
                )
                .print_json();
            } else {
                for r in &reports {
                    println!("{}\t{}\t{}", r.change, r.kind, r.path);
                }
            }
            0
        }
        Err(source) => emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        ),
    }
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), InventoryData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
