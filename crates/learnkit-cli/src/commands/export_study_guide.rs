//! `learnkit export study-guide`: renders a session's already-confirmed
//! `analyse` content (resumen, mapa mental, páginas de concepto) into a
//! single, printable Markdown document — no phase/checklist/fingerprint
//! metadata, just the study content itself (ODD `odd/tasks/export-study-guide.md`).

use crate::session_context::resolve_session;
use clap::Args;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_workflow::analysis::{list_concept_pages, read_study_map, read_summary};
use learnkit_workflow::session::read_session;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportStudyGuideArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct ExportData {
    #[serde(skip_serializing_if = "Option::is_none")]
    out: Option<String>,
}

pub fn run(args: ExportStudyGuideArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    let session = match read_session(&root, &session_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ExporterConstraint {
                    message: format!("session '{session_id}' not found"),
                },
            )
        }
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

    let summary = match read_summary(&root, &session_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ExporterConstraint {
                    message: "cannot export — no confirmed summary for this session (run `analyse set --item summary` first)".to_string(),
                },
            )
        }
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

    let study_map = match read_study_map(&root, &session_id) {
        Ok(m) => m,
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

    let pages = match list_concept_pages(&root, &session_id) {
        Ok(p) => p,
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

    let mut doc = String::new();
    doc.push_str(&format!("# {}\n\n", session.title));
    doc.push_str("## Resumen\n\n");
    doc.push_str(summary.content.trim_end());
    doc.push('\n');

    if let Some(map) = &study_map {
        doc.push_str("\n## Mapa mental\n\n");
        doc.push_str(map.content.trim_end());
        doc.push('\n');
    }

    for page in &pages {
        doc.push_str(&format!("\n## {}\n\n", page.concept));
        doc.push_str(page.content.trim_end());
        doc.push('\n');
    }

    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(source) = std::fs::create_dir_all(parent) {
                return emit_error(
                    args.json,
                    LearnKitError::Filesystem {
                        path: args.out.display().to_string(),
                        source,
                    },
                );
            }
        }
    }

    if let Err(source) = std::fs::write(&args.out, doc) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: args.out.display().to_string(),
                source,
            },
        );
    }

    if args.json {
        Envelope::ok(
            "STUDY_GUIDE_EXPORTED",
            ExportData {
                out: Some(args.out.display().to_string()),
            },
        )
        .print_json();
    } else {
        println!("Guía de estudio exportada a {}", args.out.display());
    }
    0
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), ExportData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
