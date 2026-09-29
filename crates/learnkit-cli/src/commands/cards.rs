use crate::commands::cards_image::{self, ImageBatchArgs, ImageGridArgs, ImageRejectArgs, ImageReviewArgs};
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
use learnkit_profile::card_spec::{self, CardSpec};
use learnkit_profile::language::learning_item::{self, LearningItem};
use learnkit_profile::language::vocabulary::{find_by_id as find_vocabulary_by_id, VocabularyEntry};
use learnkit_store::session_paths::SessionPaths;
use serde::Serialize;
use std::path::PathBuf;

const IMAGE_TO_PRODUCTION_V1: &str = "image-to-production-v1";
const VOCABULARY_KIND: &str = "vocabulary";

#[derive(Args)]
pub struct CardsArgs {
    #[command(subcommand)]
    action: CardsAction,
}

#[derive(Subcommand)]
enum CardsAction {
    Build(BuildArgs),
    Validate(ValidateArgs),
    Set(SetArgs),
    ImageBatch(ImageBatchArgs),
    ImageGrid(ImageGridArgs),
    ImageReview(ImageReviewArgs),
    ImageReject(ImageRejectArgs),
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

/// `learnkit cards set` — generic (non-vocabulary) `CardSpec` upsert
/// (`odd/tasks/cardspec-generalization.md` T3). Vocabulary items keep using
/// `learn vocabulary add`/`edit`; this is for every other `LearningItem`
/// `kind` (grammar, dialogue, a C# enum, a Clean Code rubric, geography...).
#[derive(Args)]
struct SetArgs {
    /// The `LearningItem.id` this `CardSpec` belongs to. Must already exist.
    #[arg(long = "item")]
    item: String,
    #[arg(long)]
    activity: String,
    #[arg(long)]
    stimulus: String,
    #[arg(long)]
    response: String,
    #[arg(long)]
    feedback: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct SetData {
    #[serde(skip_serializing_if = "Option::is_none")]
    card_spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    learning_item_id: Option<String>,
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
        CardsAction::Set(a) => run_set(a),
        CardsAction::ImageBatch(a) => cards_image::run_image_batch(a),
        CardsAction::ImageGrid(a) => cards_image::run_image_grid(a),
        CardsAction::ImageReview(a) => cards_image::run_image_review(a),
        CardsAction::ImageReject(a) => cards_image::run_image_reject(a),
    }
}

/// `learnkit cards set --item <learning_item_id> --activity <str> --stimulus
/// <str> --response <str> [--feedback <str>]` — upserts the generic
/// `CardSpec` a non-vocabulary `LearningItem` needs before `cards build` can
/// generate a card for it (T3/T4/T5).
fn run_set(args: SetArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    let item = match learning_item::find_by_id(&root, &args.item) {
        Ok(Some(item)) => item,
        Ok(None) => {
            return emit_set_error(
                args.json,
                LearnKitError::ExporterConstraint {
                    message: format!("learning item '{}' does not exist", args.item),
                },
            )
        }
        Err(source) => {
            return emit_set_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    for (field, value) in [
        ("activity", args.activity.as_str()),
        ("stimulus", args.stimulus.as_str()),
        ("response", args.response.as_str()),
    ] {
        if let Some(suggested) = learnkit_core::encoding::detect_mojibake(value) {
            return emit_set_error(
                args.json,
                LearnKitError::EncodingSuspicious {
                    field: field.to_string(),
                    value: value.to_string(),
                    suggested,
                },
            );
        }
    }
    if let Some(feedback) = args.feedback.as_deref() {
        if let Some(suggested) = learnkit_core::encoding::detect_mojibake(feedback) {
            return emit_set_error(
                args.json,
                LearnKitError::EncodingSuspicious {
                    field: "feedback".to_string(),
                    value: feedback.to_string(),
                    suggested,
                },
            );
        }
    }

    let spec = match card_spec::set(
        &root,
        &item.id,
        &args.activity,
        &args.stimulus,
        &args.response,
        args.feedback.as_deref(),
    ) {
        Ok(spec) => spec,
        Err(source) => {
            return emit_set_error(
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
            "CARD_SPEC_SET",
            SetData {
                card_spec_id: Some(spec.id),
                learning_item_id: Some(spec.learning_item_id),
            },
        )
        .print_json();
    } else {
        println!("{}\t{}", spec.id, spec.learning_item_id);
    }
    0
}

fn emit_set_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), SetData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
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

    // Hard Guards entry check (odd/tasks/hard-guards-entry-checks.md): a live
    // check against real data, never against a cached phase manifest —
    // `vocabulary` never writes one, and this must not retroactively block a
    // project whose vocabulary/cards already existed before this guard did.
    if items.is_empty() {
        return emit_blocked(
            args.json,
            "vocabulary",
            "no hay ningún elemento de aprendizaje de vocabulario — ejecuta 'learn vocabulary add' primero",
        );
    }

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

        // Generalization (`cardspec-generalization` T4/T5): a `vocabulary`
        // item keeps sourcing text/media exactly as before (untouched
        // below). Any other `kind` needs its `CardSpec` — normal, non-BLOCKED
        // error (not a Hard Guards guard: a concrete item without content,
        // not "zero vocabulary project-wide") when it hasn't been set yet via
        // `cards set`.
        let generic_spec: Option<CardSpec> = if item.kind == VOCABULARY_KIND {
            None
        } else {
            match card_spec::find_by_learning_item(&root, &item.id) {
                Ok(Some(spec)) => Some(spec),
                Ok(None) => {
                    return emit_error(
                        args.json,
                        LearnKitError::CardSpecMissing {
                            learning_item_id: item.id.clone(),
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
            }
        };

        let (front_blocks, back_blocks) = if args.template == IMAGE_TO_PRODUCTION_V1 {
            if let Some(spec) = &generic_spec {
                let image_asset_id = resolved_id(
                    resolve_image(&assets_dir, &spec.stimulus, None, &image_provider, |url| {
                        download_via_provider(&image_provider, url)
                    }),
                    template.front_image,
                    &card_id,
                    "front",
                    "image",
                    &mut media_warnings,
                );
                let audio_asset_id = resolved_id(
                    resolve_audio(
                        &assets_dir,
                        &spec.stimulus,
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
                );

                build_image_to_production_blocks_generic(spec, image_asset_id, audio_asset_id)
            } else {
                // FR-013b: front = image + pronunciation audio (both required);
                // back = word + translation + IPA/example (when present) + the
                // *same* pronunciation audio, resolved exactly once.
                let image_asset_id = resolved_id(
                    resolve_image(&assets_dir, &item.title, None, &image_provider, |url| {
                        download_via_provider(&image_provider, url)
                    }),
                    template.front_image,
                    &card_id,
                    "front",
                    "image",
                    &mut media_warnings,
                );
                let audio_asset_id = resolved_id(
                    resolve_audio(
                        &assets_dir,
                        pronunciation_source_text(&item),
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
                );

                let vocabulary_entry =
                    match find_vocabulary_by_id(&root, &item.vocabulary_entry_id) {
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

                build_image_to_production_blocks(
                    &item,
                    vocabulary_entry.as_ref(),
                    image_asset_id,
                    audio_asset_id,
                )
            }
        } else if let Some(spec) = &generic_spec {
            let mut front_blocks = Vec::new();
            if template.front_image != MediaPolicy::Disabled {
                front_blocks.push(Block::Image {
                    asset_id: resolved_id(
                        resolve_image(
                            &assets_dir,
                            &spec.stimulus,
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
                            &spec.stimulus,
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
                value: spec.response.clone(),
            }];
            if let Some(feedback) = &spec.feedback {
                back_blocks.push(Block::Text {
                    value: feedback.clone(),
                });
            }
            if template.back_audio != MediaPolicy::Disabled {
                back_blocks.push(Block::Audio {
                    asset_id: resolved_id(
                        resolve_audio(
                            &assets_dir,
                            &spec.response,
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
                            &spec.stimulus,
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

            (front_blocks, back_blocks)
        } else {
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

            (front_blocks, back_blocks)
        };

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

/// The text pronunciation audio must always be generated from — the English
/// word (`item.title`), never the Spanish translation (`item.summary`).
/// Extracted as its own pure function so the bug this fixes (FR-013b: audio
/// was generated from `item.summary`, Spanish text spoken in an `en-GB`
/// voice) has a direct, dependency-free regression test (T111) without
/// needing to inject a fake `VoiceProvider` into `run_build`'s hardcoded
/// `RestVoiceProvider::default()`.
fn pronunciation_source_text(item: &LearningItem) -> &str {
    &item.title
}

/// Builds the front/back blocks for an `image-to-production-v1` card —
/// FR-013b. Pure (no I/O, no provider calls): takes already-resolved asset
/// ids and the (optional) `VocabularyEntry` for `ipa`/`examples`, so it's
/// directly testable (T110, T112) without a real image/voice provider.
///
/// - Front: image, then audio (both from the ids given).
/// - Back: the English word, the translation, the IPA (if present), the
///   first usage example (if present), then the *same* audio id as the
///   front — never a second, independently-resolved asset.
fn build_image_to_production_blocks(
    item: &LearningItem,
    vocabulary_entry: Option<&VocabularyEntry>,
    image_asset_id: Option<String>,
    audio_asset_id: Option<String>,
) -> (Vec<Block>, Vec<Block>) {
    let front_blocks = vec![
        Block::Image {
            asset_id: image_asset_id,
        },
        Block::Audio {
            asset_id: audio_asset_id.clone(),
        },
    ];

    let mut back_blocks = vec![
        Block::Text {
            value: item.title.clone(),
        },
        Block::Text {
            value: item.summary.clone(),
        },
    ];

    if let Some(vocab) = vocabulary_entry {
        if let Some(ipa) = &vocab.ipa {
            back_blocks.push(Block::Text { value: ipa.clone() });
        }
        if let Some(example) = vocab.examples.first() {
            back_blocks.push(Block::Text {
                value: example.text.clone(),
            });
        }
    }

    back_blocks.push(Block::Audio {
        asset_id: audio_asset_id,
    });

    (front_blocks, back_blocks)
}

/// Generic counterpart of `build_image_to_production_blocks` for a
/// non-vocabulary `LearningItem` (`cardspec-generalization` T4): same
/// front/back shape (image+audio front; text(s) + the same audio back), but
/// sourced from `CardSpec.stimulus`/`response`/`feedback` instead of
/// `item.title`/`item.summary`/`VocabularyEntry`.
fn build_image_to_production_blocks_generic(
    spec: &CardSpec,
    image_asset_id: Option<String>,
    audio_asset_id: Option<String>,
) -> (Vec<Block>, Vec<Block>) {
    let front_blocks = vec![
        Block::Image {
            asset_id: image_asset_id,
        },
        Block::Audio {
            asset_id: audio_asset_id.clone(),
        },
    ];

    let mut back_blocks = vec![
        Block::Text {
            value: spec.stimulus.clone(),
        },
        Block::Text {
            value: spec.response.clone(),
        },
    ];

    if let Some(feedback) = &spec.feedback {
        back_blocks.push(Block::Text {
            value: feedback.clone(),
        });
    }

    back_blocks.push(Block::Audio {
        asset_id: audio_asset_id,
    });

    (front_blocks, back_blocks)
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

/// Hard Guards entry check failure (odd/tasks/hard-guards-entry-checks.md):
/// same `code: "BLOCKED"` / exit `20` contract `consolidate`'s own guard
/// already uses (`GuardBlocked`'s exit-code tier), for consistency across
/// every guarded command in this project.
fn emit_blocked(json: bool, phase: &str, reason: &str) -> i32 {
    if json {
        Envelope::err("BLOCKED", CardsData::default()).print_json();
    } else {
        eprintln!("Error: bloqueado — {reason}");
    }
    LearnKitError::GuardBlocked {
        phase: phase.to_string(),
        reason: reason.to_string(),
    }
    .exit_code()
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

    fn item_with(title: &str, summary: &str) -> LearningItem {
        LearningItem {
            id: "li-1".to_string(),
            kind: "vocabulary".to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            tags: vec![],
            mastery_dimensions: vec![],
            vocabulary_entry_id: "vocab-1".to_string(),
        }
    }

    fn vocab_with(
        ipa: Option<&str>,
        examples: &[&str],
    ) -> learnkit_profile::language::vocabulary::VocabularyEntry {
        use learnkit_profile::language::vocabulary::{Example, Sense, SourceRef, VocabularyEntry};
        VocabularyEntry {
            id: "vocab-1".to_string(),
            language: "en".to_string(),
            variety: "en-GB".to_string(),
            lemma: "whiteboard".to_string(),
            part_of_speech: None,
            senses: vec![Sense {
                gloss: "pizarra".to_string(),
            }],
            sources: vec![SourceRef {
                source_id: "src-1".to_string(),
                locator: None,
            }],
            suggested_by: None,
            ipa: ipa.map(|s| s.to_string()),
            examples: examples
                .iter()
                .map(|t| Example {
                    text: t.to_string(),
                })
                .collect(),
        }
    }

    // --- T111 (regression, FR-013b): pronunciation audio must always be
    // generated from item.title (English), never item.summary (Spanish). ---
    #[test]
    fn pronunciation_source_text_is_always_the_english_title_not_the_translation() {
        let item = item_with("whiteboard", "pizarra");
        assert_eq!(pronunciation_source_text(&item), "whiteboard");
        assert_ne!(pronunciation_source_text(&item), item.summary);
    }

    // --- T110: ipa/example present -> separate back text blocks; absent ->
    // no such blocks, card still complete (no missing-required-field gap). ---
    #[test]
    fn back_includes_ipa_and_example_blocks_when_present_on_the_vocabulary_entry() {
        let item = item_with("whiteboard", "pizarra");
        let vocab = vocab_with(Some("ˈwaɪtbɔːd"), &["Write it on the whiteboard."]);

        let (_front, back) = build_image_to_production_blocks(
            &item,
            Some(&vocab),
            Some("img-1".to_string()),
            Some("audio-1".to_string()),
        );

        let texts: Vec<&str> = back
            .iter()
            .filter_map(|b| match b {
                Block::Text { value } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(
            texts,
            vec!["whiteboard", "pizarra", "ˈwaɪtbɔːd", "Write it on the whiteboard."]
        );
    }

    #[test]
    fn back_omits_ipa_and_example_blocks_when_absent() {
        let item = item_with("whiteboard", "pizarra");
        let vocab = vocab_with(None, &[]);

        let (_front, back) = build_image_to_production_blocks(
            &item,
            Some(&vocab),
            Some("img-1".to_string()),
            Some("audio-1".to_string()),
        );

        let texts: Vec<&str> = back
            .iter()
            .filter_map(|b| match b {
                Block::Text { value } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(texts, vec!["whiteboard", "pizarra"]);
    }

    #[test]
    fn back_omits_ipa_and_example_blocks_when_no_vocabulary_entry_found() {
        let item = item_with("whiteboard", "pizarra");

        let (_front, back) = build_image_to_production_blocks(
            &item,
            None,
            Some("img-1".to_string()),
            Some("audio-1".to_string()),
        );

        let texts: Vec<&str> = back
            .iter()
            .filter_map(|b| match b {
                Block::Text { value } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(texts, vec!["whiteboard", "pizarra"]);
    }

    // --- T112: front and back audio blocks on the same card share the
    // identical asset_id (the resource is reused, never re-resolved). ---
    #[test]
    fn front_and_back_audio_blocks_share_the_identical_asset_id() {
        let item = item_with("whiteboard", "pizarra");
        let vocab = vocab_with(None, &[]);

        let (front, back) = build_image_to_production_blocks(
            &item,
            Some(&vocab),
            Some("img-1".to_string()),
            Some("audio-shared".to_string()),
        );

        let front_audio = front.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });
        let back_audio = back.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });

        assert_eq!(front_audio, Some(Some("audio-shared".to_string())));
        assert_eq!(front_audio, back_audio);
    }

    #[test]
    fn front_and_back_audio_blocks_are_both_none_when_resolution_failed() {
        let item = item_with("whiteboard", "pizarra");

        let (front, back) = build_image_to_production_blocks(&item, None, None, None);

        let front_audio = front.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });
        let back_audio = back.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });

        assert_eq!(front_audio, Some(None));
        assert_eq!(front_audio, back_audio);
    }

    // --- cardspec-generalization T4/T6: the generic (non-vocabulary)
    // counterpart of `build_image_to_production_blocks`, sourced from
    // `CardSpec` instead of `item.title`/`item.summary`/`VocabularyEntry`. ---

    fn spec_with(stimulus: &str, response: &str, feedback: Option<&str>) -> CardSpec {
        CardSpec {
            id: "cardspec-li-1".to_string(),
            learning_item_id: "li-1".to_string(),
            activity: "activity".to_string(),
            stimulus: stimulus.to_string(),
            response: response.to_string(),
            feedback: feedback.map(|s| s.to_string()),
        }
    }

    #[test]
    fn generic_back_includes_stimulus_response_and_feedback_when_present() {
        let spec = spec_with(
            "She ___ to school every day.",
            "goes",
            Some("Third person singular present takes -s."),
        );

        let (_front, back) = build_image_to_production_blocks_generic(
            &spec,
            Some("img-1".to_string()),
            Some("audio-1".to_string()),
        );

        let texts: Vec<&str> = back
            .iter()
            .filter_map(|b| match b {
                Block::Text { value } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(
            texts,
            vec![
                "She ___ to school every day.",
                "goes",
                "Third person singular present takes -s."
            ]
        );
    }

    #[test]
    fn generic_back_omits_feedback_block_when_absent() {
        let spec = spec_with("stimulus", "response", None);

        let (_front, back) = build_image_to_production_blocks_generic(
            &spec,
            Some("img-1".to_string()),
            Some("audio-1".to_string()),
        );

        let texts: Vec<&str> = back
            .iter()
            .filter_map(|b| match b {
                Block::Text { value } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(texts, vec!["stimulus", "response"]);
    }

    #[test]
    fn generic_front_and_back_audio_blocks_share_the_identical_asset_id() {
        let spec = spec_with("stimulus", "response", None);

        let (front, back) = build_image_to_production_blocks_generic(
            &spec,
            Some("img-1".to_string()),
            Some("audio-shared".to_string()),
        );

        let front_audio = front.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });
        let back_audio = back.iter().find_map(|b| match b {
            Block::Audio { asset_id } => Some(asset_id.clone()),
            _ => None,
        });

        assert_eq!(front_audio, Some(Some("audio-shared".to_string())));
        assert_eq!(front_audio, back_audio);
    }
}
