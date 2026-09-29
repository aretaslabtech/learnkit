use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_profile::language::learning_item::{ensure_for_vocabulary, find_by_vocabulary_entry};
use learnkit_profile::language::vocabulary::{add_or_reuse, find_by_lemma, set_fields};
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
    /// Edit fields of an already-persisted vocabulary entry — FR-012d. The
    /// only supported way to modify vocabulary; the files under
    /// `knowledge/` must never be edited by hand. Omitting a flag leaves
    /// that field exactly as persisted; only `--example` and
    /// `--clear-examples` affect `examples`, and never affects `id` or
    /// `sources`.
    Edit(EditArgs),
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
struct EditArgs {
    #[arg(long)]
    lemma: String,
    #[arg(long)]
    sense: Option<String>,
    #[arg(long = "part-of-speech")]
    part_of_speech: Option<String>,
    #[arg(long)]
    ipa: Option<String>,
    /// Repeatable. When supplied at least once, REPLACES the existing
    /// examples list (rather than appending) — FR-012d/T113. Omit both this
    /// and `--clear-examples` to leave the existing examples untouched.
    #[arg(long = "example")]
    example: Vec<String>,
    /// Empties the examples list explicitly. Without `--example`, this is
    /// the only way to clear examples — a bare `edit` with neither flag
    /// must never lose existing data.
    #[arg(long)]
    clear_examples: bool,
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
    /// IPA transcription — FR-013b. Optional.
    #[arg(long)]
    ipa: Option<String>,
    /// A usage example — FR-013b. Repeatable; only the first is currently
    /// shown on the card back, but every one supplied is persisted.
    #[arg(long = "example")]
    example: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    lemma: Option<String>,
}

pub fn run(args: LearnArgs) -> i32 {
    match args.action {
        LearnAction::Vocabulary(v) => match v.action {
            VocabularyAction::Add(a) => run_add(a),
            VocabularyAction::Remove(a) => run_remove(a),
            VocabularyAction::Edit(a) => run_edit(a),
        },
    }
}

fn run_add(args: AddArgs) -> i32 {
    if let Some(code) = check_encoding(args.json, "lemma", &args.lemma) {
        return code;
    }
    if let Some(code) = check_encoding(args.json, "sense", &args.sense) {
        return code;
    }
    if let Some(ipa) = &args.ipa {
        if let Some(code) = check_encoding(args.json, "ipa", ipa) {
            return code;
        }
    }
    for example in &args.example {
        if let Some(code) = check_encoding(args.json, "example", example) {
            return code;
        }
    }

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

    let mut vocab = match add_or_reuse(
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

    if let Err(source) = learnkit_profile::language::vocabulary::set_details(
        &root,
        &mut vocab,
        args.ipa.as_deref(),
        &args.example,
    ) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

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
                lemma: None,
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
                lemma: None,
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

/// FR-012d/T113: edits fields of an already-persisted vocabulary entry —
/// the only supported way to modify vocabulary state; `knowledge/` must
/// never be edited by hand. Same "lemma not found" failure pattern as
/// `run_remove` (exit 40).
fn run_edit(args: EditArgs) -> i32 {
    if let Some(sense) = &args.sense {
        if let Some(code) = check_encoding(args.json, "sense", sense) {
            return code;
        }
    }
    if let Some(ipa) = &args.ipa {
        if let Some(code) = check_encoding(args.json, "ipa", ipa) {
            return code;
        }
    }
    for example in &args.example {
        if let Some(code) = check_encoding(args.json, "example", example) {
            return code;
        }
    }

    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    let mut vocab = match find_by_lemma(&root, &args.lemma) {
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

    // `--example` supplied at least once replaces the list; otherwise only
    // `--clear-examples` (with none supplied) empties it explicitly. Neither
    // flag present means "leave examples untouched" (`None`).
    let examples = if !args.example.is_empty() || args.clear_examples {
        Some(args.example.as_slice())
    } else {
        None
    };

    if let Err(source) = set_fields(
        &root,
        &mut vocab,
        args.sense.as_deref(),
        args.part_of_speech.as_deref(),
        args.ipa.as_deref(),
        examples,
    ) {
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
            "VOCABULARY_EDITED",
            LearnData {
                vocabulary_id: Some(vocab.id.clone()),
                learning_item_id: None,
                removed_card_ids: None,
                lemma: Some(vocab.lemma.clone()),
            },
        )
        .print_json();
    } else {
        println!("Vocabulario editado: {} ({})", vocab.lemma, vocab.id);
    }
    0
}

/// Rejects a free-text CLI argument that shows the classic
/// "UTF-8 misread as a legacy codepage" mojibake pattern (e.g. a BOM-less
/// `.ps1` script read by Windows PowerShell 5.1) rather than silently
/// persisting corrupted vocabulary data. Returns the exit code to return
/// immediately when the field is suspect, or `None` when it is clean.
fn check_encoding(json: bool, field: &str, value: &str) -> Option<i32> {
    learnkit_core::encoding::detect_mojibake(value).map(|suggested| {
        emit_error(
            json,
            LearnKitError::EncodingSuspicious {
                field: field.to_string(),
                value: value.to_string(),
                suggested,
            },
        )
    })
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), LearnData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
