use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_cards::card::{
    completeness, load_all, save, Block, CardDefinition, Completeness, Side,
};
use learnkit_cards::template::{find as find_template, MediaPolicy};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_media::image::WikimediaCommonsProvider;
use learnkit_media::resolve::{resolve_audio, resolve_image, ResolvedMedia};
use learnkit_media::voice::RestVoiceProvider;
use learnkit_store::session_paths::SessionPaths;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct CardsArgs {
    #[command(subcommand)]
    action: CardsAction,
}

#[derive(Subcommand)]
enum CardsAction {
    Build(BuildArgs),
    Validate(ValidateArgs),
}

#[derive(Args)]
struct BuildArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long, default_value = "image-to-production-v1")]
    template: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Regenerate every card, including ones that are already complete
    /// (FR-017f default behaviour skips those to avoid an external
    /// provider's intermittency turning a resolved card back into pending).
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct ValidateArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct CardState {
    id: String,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// An *optional* media slot (e.g. `back_audio: Optional`) that failed to
/// resolve. `completeness()` never surfaces this — an optional slot missing
/// its asset doesn't stop a card being `complete` — so without this, a
/// systemic failure (e.g. `TTS_API_KEY` unset) resolves to `None` silently
/// on every single card, with no diagnostic anywhere. Found post-release:
/// 127/127 cards reported `complete` with zero audio, and the only way to
/// find out why was to read the source.
#[derive(Debug, Serialize)]
struct MediaWarning {
    card_id: String,
    side: &'static str,
    kind: &'static str,
    reason: String,
}

#[derive(Debug, Serialize, Default)]
struct CardsData {
    #[serde(skip_serializing_if = "Option::is_none")]
    cards: Option<Vec<CardState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_warnings: Option<Vec<MediaWarning>>,
}

pub fn run(args: CardsArgs) -> i32 {
    match args.action {
        CardsAction::Build(a) => run_build(a),
        CardsAction::Validate(a) => run_validate(a),
    }
}

fn run_build(args: BuildArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);
    let assets_dir = session_paths.assets();

    let items = match learnkit_profile::language::learning_item::load_all_vocabulary_items(&root) {
        Ok(items) => items,
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

    let Some(template) = find_template(&args.template) else {
        return emit_error(
            args.json,
            LearnKitError::ExporterConstraint {
                message: format!("unknown card template '{}'", args.template),
            },
        );
    };

    let voice_provider = RestVoiceProvider::default();
    let image_provider = WikimediaCommonsProvider::default();

    let existing: std::collections::HashMap<String, CardDefinition> = if args.force {
        std::collections::HashMap::new()
    } else {
        match load_all(session_paths.root()) {
            Ok(cards) => cards.into_iter().map(|c| (c.id.clone(), c)).collect(),
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
    };

    // Building a card can make real network calls (Wikimedia Commons) and
    // spawn a subprocess (Piper) per item — with dozens/hundreds of
    // vocabulary items this can legitimately take minutes. Without this,
    // the command prints nothing until the very end and looks hung; each
    // line goes to stderr so stdout stays clean JSON when --json is used.
    let total = items.len();
    let mut states = Vec::with_capacity(total);
    let mut media_warnings: Vec<MediaWarning> = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        eprintln!("[{}/{total}] {} ...", index + 1, item.title);
        let card_id = format!("card-{}", item.id);

        if let Some(card) = existing.get(&card_id) {
            if card.template == args.template && completeness(card) == Completeness::Complete {
                let state = to_card_state(card);
                eprintln!("  -> {} (reused, already complete)", state.state);
                states.push(state);
                continue;
            }
        }

        let mut front_blocks = Vec::new();
        if template.front_image != MediaPolicy::Disabled {
            front_blocks.push(Block::Image {
                asset_id: resolved_id(
                    resolve_image(
                        &assets_dir,
                        &item.title,
                        None,
                        &image_provider,
                        |url| download_via_provider(&image_provider, url),
                    ),
                    template.front_image,
                    &card_id,
                    "front",
                    "image",
                    &mut media_warnings,
                ),
            });
        }
        if template.front_audio != MediaPolicy::Disabled {
            front_blocks.push(Block::Audio {
                asset_id: resolved_id(
                    resolve_audio(
                        &assets_dir,
                        &item.title,
                        "en-GB",
                        "project-default",
                        None,
                        &voice_provider,
                    ),
                    template.front_audio,
                    &card_id,
                    "front",
                    "audio",
                    &mut media_warnings,
                ),
            });
        }

        let mut back_blocks = vec![Block::Text {
            value: item.summary.clone(),
        }];
        if template.back_audio != MediaPolicy::Disabled {
            back_blocks.push(Block::Audio {
                asset_id: resolved_id(
                    resolve_audio(
                        &assets_dir,
                        &item.summary,
                        "en-GB",
                        "project-default",
                        None,
                        &voice_provider,
                    ),
                    template.back_audio,
                    &card_id,
                    "back",
                    "audio",
                    &mut media_warnings,
                ),
            });
        }
        if template.back_image != MediaPolicy::Disabled {
            back_blocks.push(Block::Image {
                asset_id: resolved_id(
                    resolve_image(
                        &assets_dir,
                        &item.title,
                        None,
                        &image_provider,
                        |url| download_via_provider(&image_provider, url),
                    ),
                    template.back_image,
                    &card_id,
                    "back",
                    "image",
                    &mut media_warnings,
                ),
            });
        }

        let card = CardDefinition {
            id: card_id.clone(),
            learning_item_ids: vec![item.id.clone()],
            template: args.template.clone(),
            front: Side {
                blocks: front_blocks,
            },
            back: Side {
                blocks: back_blocks,
            },
        };

        if let Err(source) = save(session_paths.root(), &card) {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            );
        }

        let state = to_card_state(&card);
        eprintln!("  -> {}", state.state);
        states.push(state);
    }

    let media_warnings = if media_warnings.is_empty() {
        None
    } else {
        Some(media_warnings)
    };

    if args.json {
        Envelope::ok(
            "CARDS_BUILT",
            CardsData {
                cards: Some(states),
                media_warnings,
            },
        )
        .print_json();
    } else {
        for c in &states {
            println!("{}\t{}", c.id, c.state);
        }
        if let Some(warnings) = &media_warnings {
            eprintln!(
                "{} recurso(s) opcional(es) no se pudieron generar (no bloquea la tarjeta, pero probablemente quieras corregirlo):",
                warnings.len()
            );
            for w in warnings {
                eprintln!("  - {} ({} {}): {}", w.card_id, w.side, w.kind, w.reason);
            }
        }
    }
    0
}

fn run_validate(args: ValidateArgs) -> i32 {
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

    let states: Vec<CardState> = cards.iter().map(to_card_state).collect();
    let ok = states.iter().all(|c| c.state == "complete");

    if args.json {
        if ok {
            Envelope::ok(
                "CARDS_VALID",
                CardsData {
                    cards: Some(states),
                    media_warnings: None,
                },
            )
            .print_json();
        } else {
            Envelope::err(
                "CARDS_INCOMPLETE",
                CardsData {
                    cards: Some(states),
                    media_warnings: None,
                },
            )
            .print_json();
        }
    } else {
        for c in &states {
            println!(
                "{}\t{}\t{}",
                c.id,
                c.state,
                c.reason.as_deref().unwrap_or("")
            );
        }
    }
    if ok {
        0
    } else {
        40
    }
}

/// Extracts the resolved asset id, if any. When the slot is `Optional` and
/// resolution failed, records why in `warnings` instead of discarding the
/// reason — an `Optional` slot missing its asset never blocks completeness,
/// so this is the only place that reason is ever seen (post-release fix,
/// found after `TTS_API_KEY` being unset silently produced zero audio on
/// every card with no diagnostic anywhere).
fn resolved_id(
    resolved: ResolvedMedia,
    policy: MediaPolicy,
    card_id: &str,
    side: &'static str,
    kind: &'static str,
    warnings: &mut Vec<MediaWarning>,
) -> Option<String> {
    match resolved {
        ResolvedMedia::Asset(a) => Some(a.id),
        ResolvedMedia::Pending { reason } => {
            if policy == MediaPolicy::Optional {
                warnings.push(MediaWarning {
                    card_id: card_id.to_string(),
                    side,
                    kind,
                    reason,
                });
            }
            None
        }
    }
}

/// Downloads a Wikimedia Commons candidate's bytes using the provider's own
/// client (`User-Agent`/timeout), rejecting non-image responses — a bare
/// `reqwest::blocking::get` here was the original bug (research.md §4):
/// Wikimedia's media servers reject requests without a `User-Agent` and
/// return an HTML/text error page, which we were previously accepting as
/// if it were the image itself.
fn download_via_provider(
    provider: &WikimediaCommonsProvider,
    url: &str,
) -> std::io::Result<Vec<u8>> {
    provider.download(url).map_err(std::io::Error::other)
}

fn to_card_state(card: &CardDefinition) -> CardState {
    match completeness(card) {
        Completeness::Complete => CardState {
            id: card.id.clone(),
            state: "complete".to_string(),
            reason: None,
        },
        Completeness::PendingImage { reason, .. } => CardState {
            id: card.id.clone(),
            state: "pending_image".to_string(),
            reason: Some(reason),
        },
        Completeness::PendingAudio { reason, .. } => CardState {
            id: card.id.clone(),
            state: "pending_audio".to_string(),
            reason: Some(reason),
        },
    }
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), CardsData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(reason: &str) -> ResolvedMedia {
        ResolvedMedia::Pending {
            reason: reason.to_string(),
        }
    }

    #[test]
    fn optional_slot_failing_to_resolve_records_a_media_warning() {
        let mut warnings = Vec::new();
        let id = resolved_id(
            pending("missing TTS_API_KEY environment variable"),
            MediaPolicy::Optional,
            "card-1",
            "back",
            "audio",
            &mut warnings,
        );
        assert_eq!(id, None);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].card_id, "card-1");
        assert_eq!(warnings[0].side, "back");
        assert_eq!(warnings[0].kind, "audio");
        assert_eq!(warnings[0].reason, "missing TTS_API_KEY environment variable");
    }

    #[test]
    fn required_slot_failing_to_resolve_does_not_record_a_warning() {
        // A Required slot's failure already surfaces via `completeness()`
        // (pending_image/pending_audio) — recording it here too would be a
        // duplicate, noisier diagnostic for the same fact.
        let mut warnings = Vec::new();
        resolved_id(
            pending("no license-compatible image found"),
            MediaPolicy::Required,
            "card-2",
            "front",
            "image",
            &mut warnings,
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn a_resolved_asset_never_records_a_warning() {
        let mut warnings = Vec::new();
        let id = resolved_id(
            ResolvedMedia::Asset(Box::new(learnkit_media::asset::Asset {
                id: "asset-1".to_string(),
                asset_type: learnkit_media::asset::AssetType::Audio,
                path: PathBuf::from("assets/asset-1.mp3"),
                sha256: "deadbeef".to_string(),
                mime: "audio/mpeg".to_string(),
                origin: learnkit_media::asset::AssetOrigin::Supplied,
            })),
            MediaPolicy::Optional,
            "card-3",
            "back",
            "audio",
            &mut warnings,
        );
        assert_eq!(id.as_deref(), Some("asset-1"));
        assert!(warnings.is_empty());
    }
}
