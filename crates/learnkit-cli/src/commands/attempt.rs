use crate::commands::assessment::load_items_for_session;
use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_assessment::attempt::import_results;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct AttemptArgs {
    #[command(subcommand)]
    action: AttemptAction,
}

#[derive(Subcommand)]
enum AttemptAction {
    /// Import a completed exam's downloaded results.json.
    Import(ImportArgs),
}

#[derive(Args)]
struct ImportArgs {
    file: PathBuf,
    #[arg(long)]
    assessment: String,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct AttemptData {
    #[serde(skip_serializing_if = "Option::is_none")]
    attempts_imported: Option<usize>,
}

pub fn run(args: AttemptArgs) -> i32 {
    match args.action {
        AttemptAction::Import(a) => run_import(a),
    }
}

fn run_import(args: ImportArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    let items = match load_items_for_session(&root, &session_id) {
        Ok(i) => i,
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    let item_learning_items: HashMap<String, Vec<String>> = items
        .into_iter()
        .map(|i| (i.id, i.learning_item_ids))
        .collect();

    let raw = match std::fs::read_to_string(&args.file) {
        Ok(r) => r,
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: args.file.display().to_string(),
                    source,
                },
            )
        }
    };

    match import_results(&root, &raw, &item_learning_items) {
        Ok(count) => {
            if args.json {
                Envelope::ok(
                    "ATTEMPTS_IMPORTED",
                    AttemptData {
                        attempts_imported: Some(count),
                    },
                )
                .print_json();
            } else {
                println!("{count} intento(s) importado(s).");
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
        Envelope::err(err.code(), AttemptData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
