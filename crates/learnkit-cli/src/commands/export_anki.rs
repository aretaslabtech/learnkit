use crate::session_context::resolve_session;
use clap::Args;
use learnkit_cards::card::{completeness, load_all, Completeness};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_media::asset;
use learnkit_store::session_paths::SessionPaths;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportAnkiArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Export the complete cards anyway, excluding incomplete ones instead
    /// of failing the whole export (FR-019b). The default (this flag
    /// omitted) is still FR-019: fail explicitly if any card is incomplete.
    /// Excluded cards are always listed in the result, never dropped
    /// silently.
    #[arg(long)]
    skip_incomplete: bool,
}

#[derive(Debug, Clone, Serialize)]
struct IncompleteCard {
    id: String,
    reason: String,
}

#[derive(Debug, Serialize, Default)]
struct ExportData {
    #[serde(skip_serializing_if = "Option::is_none")]
    out: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incomplete_cards: Option<Vec<IncompleteCard>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excluded_cards: Option<Vec<IncompleteCard>>,
}

pub fn run(args: ExportAnkiArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let cards = match load_all(session_paths.root()) {
        Ok(c) => c,
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

    let incomplete_ids: std::collections::HashSet<String>;
    let incomplete: Vec<IncompleteCard> = cards
        .iter()
        .filter_map(|c| match completeness(c) {
            Completeness::Complete => None,
            Completeness::PendingImage { reason, .. }
            | Completeness::PendingAudio { reason, .. } => Some(IncompleteCard {
                id: c.id.clone(),
                reason,
            }),
        })
        .collect();

    if !incomplete.is_empty() && !args.skip_incomplete {
        let message = incomplete
            .iter()
            .map(|c| format!("{}: {}", c.id, c.reason))
            .collect::<Vec<_>>()
            .join("; ");
        let err = LearnKitError::ExporterConstraint {
            message: format!("cannot export — incomplete cards: {message}"),
        };
        if args.json {
            Envelope::err(
                err.code(),
                ExportData {
                    incomplete_cards: Some(incomplete),
                    ..Default::default()
                },
            )
            .print_json();
        } else {
            eprintln!("Error: {err}");
        }
        return err.exit_code();
    }

    incomplete_ids = incomplete.iter().map(|c| c.id.clone()).collect();
    let cards: Vec<_> = cards
        .into_iter()
        .filter(|c| !incomplete_ids.contains(&c.id))
        .collect();

    let assets_dir = session_paths.assets();
    let mut assets_map = HashMap::new();
    for card in &cards {
        for block in card.front.blocks.iter().chain(card.back.blocks.iter()) {
            let asset_id = match block {
                learnkit_cards::card::Block::Image { asset_id } => asset_id,
                learnkit_cards::card::Block::Audio { asset_id } => asset_id,
                learnkit_cards::card::Block::Text { .. } => &None,
            };
            if let Some(id) = asset_id {
                if !assets_map.contains_key(id) {
                    if let Ok(Some(a)) = asset::load(&assets_dir, id) {
                        assets_map.insert(id.clone(), a);
                    }
                }
            }
        }
    }

    let excluded_cards = if incomplete.is_empty() {
        None
    } else {
        Some(incomplete)
    };

    match learnkit_anki::package::build_apkg(&cards, &assets_map, &args.out) {
        Ok(result) => {
            if args.json {
                Envelope::ok(
                    "ANKI_EXPORTED",
                    ExportData {
                        out: Some(args.out.display().to_string()),
                        card_count: Some(result.card_count),
                        incomplete_cards: None,
                        excluded_cards: excluded_cards.clone(),
                    },
                )
                .print_json();
            } else {
                println!(
                    "Mazo exportado a {} ({} tarjetas)",
                    args.out.display(),
                    result.card_count
                );
                if let Some(excluded) = &excluded_cards {
                    println!(
                        "Excluidas {} tarjeta(s) incompleta(s) (--skip-incomplete):",
                        excluded.len()
                    );
                    for c in excluded {
                        println!("  - {}: {}", c.id, c.reason);
                    }
                }
            }
            0
        }
        Err(source) => emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: args.out.display().to_string(),
                source,
            },
        ),
    }
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), ExportData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
