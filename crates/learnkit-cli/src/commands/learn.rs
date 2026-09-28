use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_profile::language::learning_item::{ensure_for_vocabulary, find_by_vocabulary_entry};
use learnkit_profile::language::vocabulary::{add_or_reuse, find_by_lemma};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct LearnArgs {
    #[command(subcommand)]
    action: LearnAction,
}

#[derive(Subcommand)]
enum LearnAction {
    Vocabulary(VocabularyArgs),
}

#[derive(Args)]
struct VocabularyArgs {
    #[command(subcommand)]
    action: VocabularyAction,
}

#[derive(Subcommand)]
enum VocabularyAction {
    /// Persist a confirmed vocabulary entry (manual, or confirmed from an
    /// agent suggestion) — never auto-created without confirmation (FR-012b).
    Add(AddArgs),
    /// Delete a vocabulary entry, its learning item, and any card that
    /// depends on it (in any session) — FR-012c. The only supported way to
    /// remove vocabulary; the files under `knowledge/` must never be edited
    /// by hand.
    Remove(RemoveArgs),
}

#[derive(Args)]
struct RemoveArgs {
    #[arg(long)]
    lemma: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AddArgs {
    /// Session this vocabulary was found in (used for traceability only —
    /// the entry itself is stored at the project level, FR-011). Defaults
    /// to the active session (FR-003b) if omitted.
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    lemma: String,
    #[arg(long)]
    sense: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    locator: Option<String>,
    /// `agent` if confirmed from a Skill suggestion, omitted for manual entries.
    #[arg(long = "suggested-by")]
    suggested_by: Option<String>,
    #[arg(long)]
    variety: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct LearnData {
    #[serde(skip_serializing_if = "Option::is_none")]
    vocabulary_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    learning_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed_card_ids: Option<Vec<String>>,
}

pub fn run(args: LearnArgs) -> i32 {
    match args.action {
        LearnAction::Vocabulary(v) => match v.action {
            VocabularyAction::Add(a) => run_add(a),
            VocabularyAction::Remove(a) => run_remove(a),
        },
    }
}

fn run_add(args: AddArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let variety = args.variety.unwrap_or_else(|| "en-GB".to_string());

    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    // Validate `--source` actually belongs to the session's inventory —
    // catches a typo'd source id before it silently becomes an entry's only
    // traceability link.
    match learnkit_workflow::inventory::inventory(&root, &session_id) {
        Ok(sources) => {
            if !sources.iter().any(|s| s.source.id == args.source) {
                return emit_error(
                    args.json,
                    LearnKitError::ProjectNotInitialized {
                        path: format!(
                            "source '{}' not found in session '{session_id}'",
                            args.source
                        ),
                    },
                );
            }
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
    }

    let vocab = match add_or_reuse(
        &root,
        &args.lemma,
        &args.sense,
        &args.source,
        args.locator.as_deref(),
        args.suggested_by.as_deref(),
        &variety,
    ) {
        Ok(v) => v,
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

    let item = match ensure_for_vocabulary(&root, &vocab) {
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

    if args.json {
        Envelope::ok(
            "VOCABULARY_ADDED",
            LearnData {
                vocabulary_id: Some(vocab.id),
                learning_item_id: Some(item.id),
                removed_card_ids: None,
            },
        )
        .print_json();
    } else {
        println!("Vocabulario añadido: {} ({})", vocab.lemma, vocab.id);
    }
    0
}

/// FR-012c: deletes the vocabulary entry, its learning item, and any card
/// (in any session) that depends on that learning item — the only
/// supported way to remove vocabulary state; `knowledge/` must never be
/// edited by hand.
fn run_remove(args: RemoveArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    let vocab = match find_by_lemma(&root, &args.lemma) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ExporterConstraint {
                    message: format!("no vocabulary entry found for lemma '{}'", args.lemma),
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

    let item = match find_by_vocabulary_entry(&root, &vocab.id) {
        Ok(item) => item,
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

    let mut removed_card_ids = Vec::new();
    if let Some(item) = &item {
        let sessions_dir = root.join("sessions");
        if sessions_dir.exists() {
            let entries = match std::fs::read_dir(&sessions_dir) {
                Ok(e) => e,
                Err(source) => {
                    return emit_error(
                        args.json,
                        LearnKitError::Filesystem {
                            path: sessions_dir.display().to_string(),
                            source,
                        },
                    )
                }
            };
            for entry in entries {
                let Ok(entry) = entry else { continue };
                if !entry.path().is_dir() {
                    continue;
                }
                let session_root = entry.path();
                let cards = match learnkit_cards::card::load_all(&session_root) {
                    Ok(c) => c,
                    Err(source) => {
                        return emit_error(
                            args.json,
                            LearnKitError::Filesystem {
                                path: session_root.display().to_string(),
                                source,
                            },
                        )
                    }
                };
                for card in cards {
                    if card.learning_item_ids.iter().any(|id| id == &item.id) {
                        if let Err(source) = learnkit_cards::card::remove(&session_root, &card.id)
                        {
                            return emit_error(
                                args.json,
                                LearnKitError::Filesystem {
                                    path: session_root.display().to_string(),
                                    source,
                                },
                            );
                        }
                        removed_card_ids.push(card.id);
                    }
                }
            }
        }

        if let Err(source) = learnkit_profile::language::learning_item::remove(&root, &item.id) {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            );
        }
    }

    if let Err(source) = learnkit_profile::language::vocabulary::remove(&root, &vocab.id) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

    if args.json {
        Envelope::ok(
            "VOCABULARY_REMOVED",
            LearnData {
                vocabulary_id: Some(vocab.id),
                learning_item_id: item.map(|i| i.id),
                removed_card_ids: Some(removed_card_ids),
            },
        )
        .print_json();
    } else {
        println!(
            "Vocabulario eliminado: {} ({}), {} tarjeta(s) eliminada(s)",
            vocab.lemma,
            vocab.id,
            removed_card_ids.len()
        );
    }
    0
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), LearnData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
