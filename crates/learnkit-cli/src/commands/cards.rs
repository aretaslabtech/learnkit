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

#[derive(Debug, Serialize, Default)]
struct CardsData {
    #[serde(skip_serializing_if = "Option::is_none")]
    cards: Option<Vec<CardState>>,
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
                asset_id: resolved_id(resolve_image(
                    &assets_dir,
                    &item.title,
                    None,
                    &image_provider,
                    |url| download_via_provider(&image_provider, url),
                )),
            });
        }
        if template.front_audio != MediaPolicy::Disabled {
            front_blocks.push(Block::Audio {
                asset_id: resolved_id(resolve_audio(
                    &assets_dir,
                    &item.title,
                    "en-GB",
                    "project-default",
                    None,
                    &voice_provider,
                )),
            });
        }

        let mut back_blocks = vec![Block::Text {
            value: item.summary.clone(),
        }];
        if template.back_audio != MediaPolicy::Disabled {
            back_blocks.push(Block::Audio {
                asset_id: resolved_id(resolve_audio(
                    &assets_dir,
                    &item.summary,
                    "en-GB",
                    "project-default",
                    None,
                    &voice_provider,
                )),
            });
        }
        if template.back_image != MediaPolicy::Disabled {
            back_blocks.push(Block::Image {
                asset_id: resolved_id(resolve_image(
                    &assets_dir,
                    &item.title,
                    None,
                    &image_provider,
                    |url| download_via_provider(&image_provider, url),
                )),
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

    if args.json {
        Envelope::ok(
            "CARDS_BUILT",
            CardsData {
                cards: Some(states),
            },
        )
        .print_json();
    } else {
        for c in &states {
            println!("{}\t{}", c.id, c.state);
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
                },
            )
            .print_json();
        } else {
            Envelope::err(
                "CARDS_INCOMPLETE",
                CardsData {
                    cards: Some(states),
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

fn resolved_id(resolved: ResolvedMedia) -> Option<String> {
    match resolved {
        ResolvedMedia::Asset(a) => Some(a.id),
        ResolvedMedia::Pending { .. } => None,
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
