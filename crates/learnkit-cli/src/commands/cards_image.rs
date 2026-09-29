//! `learnkit cards image-batch` / `image-grid crop` / `image-grid assign` /
//! `image-review` / `image-reject` — Modo 1 (revisión de Wikimedia) and Modo
//! 2 (generación por rejilla) of `odd/tasks/ai-image-grid-generation.md`.
//!
//! Everything here is mechanical (list what's missing, crop a grid image,
//! persist an already-approved crop as a card's image asset, list/clear an
//! already-resolved Wikimedia image). Rust never judges whether an image is
//! coherent with its concept — that's always the agent, via the
//! `learnkit-image-prompts` Skill built on top of these commands (T7).

use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_cards::card::{load_all, save, Block, CardDefinition};
use learnkit_cards::template::{find as find_template, MediaPolicy};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_media::asset::{self, AssetOrigin, AssetType};
use learnkit_media::grid;
use learnkit_profile::language::learning_item::{load_all_vocabulary_items, LearningItem};
use learnkit_profile::language::vocabulary::find_by_id as find_vocabulary_by_id;
use learnkit_store::session_paths::SessionPaths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Provider tag recorded on every asset assigned via `image-grid assign` —
/// distinct from Wikimedia's `fetched` and audio's `generated`, per T4.
/// Provider-independent by design (v2 refinement, §8 of David's spec):
/// LearnKit never calls any specific image-generation API — the grid is
/// produced externally by whatever tool the agent/human chooses (ChatGPT or
/// otherwise) — so this label must not bake in a vendor name.
const GRID_PROVIDER: &str = "external-grid-manual";

#[derive(Args)]
pub struct ImageBatchArgs {
    #[arg(long)]
    session: Option<String>,
    /// Maximum number of pending-image cards to list (a ChatGPT grid prompt
    /// covers at most 16 concepts at once).
    #[arg(long, default_value_t = 16)]
    limit: usize,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct ImageGridArgs {
    #[command(subcommand)]
    action: ImageGridAction,
}

#[derive(Subcommand)]
enum ImageGridAction {
    Crop(ImageGridCropArgs),
    Assign(ImageGridAssignArgs),
}

#[derive(Args)]
struct ImageGridCropArgs {
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    rows: u32,
    #[arg(long)]
    cols: u32,
    #[arg(long)]
    out_dir: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ImageGridAssignArgs {
    #[arg(long)]
    session: Option<String>,
    /// The learning item id this crop illustrates.
    #[arg(long)]
    item: String,
    /// Path to the already-approved crop file.
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct ImageReviewArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct ImageRejectArgs {
    #[arg(long)]
    session: Option<String>,
    /// The learning item id whose card's front image should be rejected.
    #[arg(long)]
    item: String,
    /// Why the image doesn't coherently represent the concept — recorded
    /// durably alongside the rejected asset id so it isn't silently
    /// re-proposed without context.
    #[arg(long)]
    reason: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct ImageBatchItem {
    learning_item_id: String,
    title: String,
    card_id: String,
    /// The item's gloss/meaning (`LearningItem::summary`), when non-empty —
    /// v2 refinement: disambiguates concepts like "bank" (riverbank vs
    /// financial) for whoever is generating/searching the image, without
    /// Rust ever judging image coherence itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    sense: Option<String>,
    /// Usage examples from the underlying `VocabularyEntry`, when any exist
    /// — same disambiguation rationale as `sense`. Best-effort: a missing or
    /// unreadable vocabulary entry just omits this field, it never fails the
    /// whole batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    examples: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Default)]
struct ImageBatchData {
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<ImageBatchItem>>,
}

#[derive(Debug, Serialize, Default)]
struct ImageGridCropData {
    #[serde(skip_serializing_if = "Option::is_none")]
    crops: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Default)]
struct ImageGridAssignData {
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card_id: Option<String>,
}

/// One Wikimedia-sourced image already resolved onto a card's front, listed
/// for the agent to open and judge for coherence — never judged here.
#[derive(Debug, Serialize)]
struct ImageReviewItem {
    card_id: String,
    learning_item_id: String,
    title: String,
    asset_id: String,
    /// Filesystem path to the actual image bytes, so the agent (or a human)
    /// can open and look at it — the whole point of this listing.
    path: String,
    license_name: String,
    license_url: String,
    author: String,
    source_url: String,
    /// Same semantic-context fields as `ImageBatchItem`, for the same
    /// reason: an agent judging whether a Wikimedia image coherently
    /// represents the concept needs the gloss/examples to disambiguate,
    /// just like one generating a grid prompt does.
    #[serde(skip_serializing_if = "Option::is_none")]
    sense: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    examples: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Default)]
struct ImageReviewData {
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<ImageReviewItem>>,
}

/// Durable record of a rejected Wikimedia image — T6. Persisted one file per
/// rejection under the session's `validation/rejected-images/` dir (same
/// read/write-one-file-per-entity JSON convention already used for asset
/// metadata in `learnkit-media::asset`), so a later `image-batch`/`cards
/// build` run — or a human reading the session — can see exactly which
/// candidate was rejected and why, instead of it being silently retried.
#[derive(Debug, Serialize, Deserialize)]
struct RejectedImageRecord {
    learning_item_id: String,
    card_id: String,
    asset_id: String,
    reason: String,
}

#[derive(Debug, Serialize, Default)]
struct ImageRejectData {
    #[serde(skip_serializing_if = "Option::is_none")]
    learning_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rejected_asset_id: Option<String>,
}

pub fn run_image_batch(args: ImageBatchArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error::<ImageBatchData>(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let mut cards = match load_all(session_paths.root()) {
        Ok(c) => c,
        Err(source) => {
            return emit_error::<ImageBatchData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    // `load_all` reads a directory, whose order isn't guaranteed — sort by
    // id first so the batch (and which items land in a size-limited batch)
    // is deterministic across runs/platforms.
    cards.sort_by(|a, b| a.id.cmp(&b.id));

    let items = match load_all_vocabulary_items(&root) {
        Ok(items) => items,
        Err(source) => {
            return emit_error::<ImageBatchData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    let items_by_id: HashMap<String, LearningItem> =
        items.into_iter().map(|item| (item.id.clone(), item)).collect();

    let mut batch = Vec::new();
    for card in &cards {
        if batch.len() >= args.limit {
            break;
        }
        // Needs a front image whenever the template calls for one (Required
        // *or* Optional — Disabled never does) and none is assigned yet.
        // Deliberately NOT `completeness(card) == PendingImage`: that only
        // fires for `Required`, so a card whose `Optional` front image was
        // cleared by `image-reject` (`clear_front_image_asset` always clears
        // it regardless of policy) would otherwise never be offered a
        // replacement — `completeness` correctly still calls that card
        // "complete" (an Optional slot never blocks it), but nothing should
        // ever leave a rejected image unreplaced forever.
        let wants_front_image = find_template(&card.template)
            .map(|t| t.front_image != MediaPolicy::Disabled)
            .unwrap_or(false);
        if !wants_front_image || front_image_asset_id(card).is_some() {
            continue;
        }
        let Some(item_id) = card.learning_item_ids.first() else {
            continue;
        };
        let Some(item) = items_by_id.get(item_id) else {
            continue;
        };
        let (sense, examples) = semantic_context(&root, item);
        batch.push(ImageBatchItem {
            learning_item_id: item.id.clone(),
            title: item.title.clone(),
            card_id: card.id.clone(),
            sense,
            examples,
        });
    }

    if args.json {
        Envelope::ok("IMAGE_BATCH_LISTED", ImageBatchData { items: Some(batch) }).print_json();
    } else {
        for item in &batch {
            println!("{}\t{}\t{}", item.learning_item_id, item.card_id, item.title);
        }
    }
    0
}

pub fn run_image_grid(args: ImageGridArgs) -> i32 {
    match args.action {
        ImageGridAction::Crop(a) => run_image_grid_crop(a),
        ImageGridAction::Assign(a) => run_image_grid_assign(a),
    }
}

fn run_image_grid_crop(args: ImageGridCropArgs) -> i32 {
    match grid::crop_file_to_dir(&args.file, args.rows, args.cols, &args.out_dir) {
        Ok(paths) => {
            let crops: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            if args.json {
                Envelope::ok(
                    "IMAGE_GRID_CROPPED",
                    ImageGridCropData {
                        crops: Some(crops.clone()),
                    },
                )
                .print_json();
            } else {
                for c in &crops {
                    println!("{c}");
                }
            }
            0
        }
        Err(source) => emit_error::<ImageGridCropData>(
            args.json,
            LearnKitError::Filesystem {
                path: args.file.display().to_string(),
                source,
            },
        ),
    }
}

fn run_image_grid_assign(args: ImageGridAssignArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error::<ImageGridAssignData>(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);
    let assets_dir = session_paths.assets();

    let content = match fs::read(&args.file) {
        Ok(c) => c,
        Err(source) => {
            return emit_error::<ImageGridAssignData>(
                args.json,
                LearnKitError::Filesystem {
                    path: args.file.display().to_string(),
                    source,
                },
            )
        }
    };

    let fingerprint = asset::content_hash(&content);
    let (extension, mime) = guess_image_mime(&args.file);

    let registered = match asset::register_or_reuse(
        &assets_dir,
        &fingerprint,
        AssetType::Image,
        &content,
        extension,
        mime,
        AssetOrigin::GeneratedGrid {
            provider: GRID_PROVIDER.to_string(),
        },
    ) {
        Ok(a) => a,
        Err(source) => {
            return emit_error::<ImageGridAssignData>(
                args.json,
                LearnKitError::Filesystem {
                    path: assets_dir.display().to_string(),
                    source,
                },
            )
        }
    };

    let cards = match load_all(session_paths.root()) {
        Ok(c) => c,
        Err(source) => {
            return emit_error::<ImageGridAssignData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let Some(mut card) = cards
        .into_iter()
        .find(|c| c.learning_item_ids.iter().any(|id| id == &args.item))
    else {
        return emit_error::<ImageGridAssignData>(
            args.json,
            LearnKitError::ExporterConstraint {
                message: format!(
                    "no card found for learning item '{}' in session '{session_id}'",
                    args.item
                ),
            },
        );
    };

    set_front_image_asset(&mut card, registered.id.clone());

    if let Err(source) = save(session_paths.root(), &card) {
        return emit_error::<ImageGridAssignData>(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

    let data = ImageGridAssignData {
        asset_id: Some(registered.id.clone()),
        card_id: Some(card.id.clone()),
    };
    if args.json {
        Envelope::ok("IMAGE_GRID_ASSIGNED", data).print_json();
    } else {
        println!("{}\t{}", registered.id, card.id);
    }
    0
}

pub fn run_image_review(args: ImageReviewArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error::<ImageReviewData>(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);
    let assets_dir = session_paths.assets();

    let mut cards = match load_all(session_paths.root()) {
        Ok(c) => c,
        Err(source) => {
            return emit_error::<ImageReviewData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    // Same determinism reasoning as `image-batch`: `load_all` reads a
    // directory, whose order isn't guaranteed.
    cards.sort_by(|a, b| a.id.cmp(&b.id));

    let items = match load_all_vocabulary_items(&root) {
        Ok(items) => items,
        Err(source) => {
            return emit_error::<ImageReviewData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    let items_by_id: HashMap<String, LearningItem> =
        items.into_iter().map(|item| (item.id.clone(), item)).collect();

    let mut review_items = Vec::new();
    for card in &cards {
        let Some(asset_id) = front_image_asset_id(card) else {
            continue;
        };
        let Ok(Some(asset)) = asset::load(&assets_dir, &asset_id) else {
            continue;
        };
        // Only Wikimedia-sourced images belong here — images `Supplied` by a
        // human, or already reviewed/generated via the grid, never need
        // this review (they were never picked automatically, or were
        // already coherence-checked by the agent when assigned).
        let AssetOrigin::Fetched {
            license_name,
            license_url,
            author,
            source_url,
            ..
        } = &asset.origin
        else {
            continue;
        };
        let Some(item_id) = card.learning_item_ids.first() else {
            continue;
        };
        let Some(item) = items_by_id.get(item_id) else {
            continue;
        };

        let (sense, examples) = semantic_context(&root, item);
        review_items.push(ImageReviewItem {
            card_id: card.id.clone(),
            learning_item_id: item.id.clone(),
            title: item.title.clone(),
            asset_id: asset.id.clone(),
            path: asset.path.display().to_string(),
            license_name: license_name.clone(),
            license_url: license_url.clone(),
            author: author.clone(),
            source_url: source_url.clone(),
            sense,
            examples,
        });
    }

    if args.json {
        Envelope::ok(
            "IMAGE_REVIEW_LISTED",
            ImageReviewData {
                items: Some(review_items),
            },
        )
        .print_json();
    } else {
        for item in &review_items {
            println!(
                "{}\t{}\t{}\t{}\t{} ({})",
                item.learning_item_id,
                item.card_id,
                item.title,
                item.path,
                item.license_name,
                item.author
            );
        }
    }
    0
}

pub fn run_image_reject(args: ImageRejectArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error::<ImageRejectData>(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let cards = match load_all(session_paths.root()) {
        Ok(c) => c,
        Err(source) => {
            return emit_error::<ImageRejectData>(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let Some(mut card) = cards
        .into_iter()
        .find(|c| c.learning_item_ids.iter().any(|id| id == &args.item))
    else {
        return emit_error::<ImageRejectData>(
            args.json,
            LearnKitError::ExporterConstraint {
                message: format!(
                    "no card found for learning item '{}' in session '{session_id}'",
                    args.item
                ),
            },
        );
    };

    let Some(rejected_asset_id) = clear_front_image_asset(&mut card) else {
        return emit_error::<ImageRejectData>(
            args.json,
            LearnKitError::ExporterConstraint {
                message: format!(
                    "card '{}' has no front image asset to reject",
                    card.id
                ),
            },
        );
    };

    if let Err(source) = save(session_paths.root(), &card) {
        return emit_error::<ImageRejectData>(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

    let record = RejectedImageRecord {
        learning_item_id: args.item.clone(),
        card_id: card.id.clone(),
        asset_id: rejected_asset_id.clone(),
        reason: args.reason.clone(),
    };
    if let Err(source) = record_rejection(&session_paths, &record) {
        return emit_error::<ImageRejectData>(
            args.json,
            LearnKitError::Filesystem {
                path: session_paths.validation().display().to_string(),
                source,
            },
        );
    }

    let data = ImageRejectData {
        learning_item_id: Some(args.item.clone()),
        card_id: Some(card.id.clone()),
        rejected_asset_id: Some(rejected_asset_id.clone()),
    };
    if args.json {
        Envelope::ok("IMAGE_REJECTED", data).print_json();
    } else {
        println!("{}\t{}\t{}", args.item, card.id, rejected_asset_id);
    }
    0
}

/// Reads the front image block's asset id, if any, without mutating it —
/// used by `image-review` to check what's currently assigned.
fn front_image_asset_id(card: &CardDefinition) -> Option<String> {
    card.front.blocks.iter().find_map(|b| match b {
        Block::Image { asset_id } => asset_id.clone(),
        _ => None,
    })
}

/// Clears the card's front image block's `asset_id` back to `None` (T6:
/// reverts the card to `pending_image`), without removing the block itself
/// and without deleting the underlying `Asset` file — another card may still
/// reference it via `register_or_reuse`'s dedup. Returns the asset id that
/// was cleared, or `None` if the front had no image asset to reject.
fn clear_front_image_asset(card: &mut CardDefinition) -> Option<String> {
    for block in card.front.blocks.iter_mut() {
        if let Block::Image { asset_id } = block {
            return asset_id.take();
        }
    }
    None
}

fn rejected_images_dir(session_paths: &SessionPaths) -> PathBuf {
    session_paths.validation().join("rejected-images")
}

/// Appends a new rejection record as its own file — never overwrites a
/// previous rejection, so the session keeps a full history of every
/// candidate an agent has already turned down for a given concept
/// (append-only, unrelated to `analyse`'s own stable `page:<id>` identity
/// scheme — this is just a sequential rejection log).
fn record_rejection(
    session_paths: &SessionPaths,
    record: &RejectedImageRecord,
) -> std::io::Result<PathBuf> {
    let dir = rejected_images_dir(session_paths);
    let existing_count = if dir.exists() {
        fs::read_dir(&dir)?
            .filter(|entry| {
                entry
                    .as_ref()
                    .ok()
                    .map(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                    .unwrap_or(false)
            })
            .count()
    } else {
        0
    };
    let path = dir.join(format!("rejection-{:03}.json", existing_count + 1));
    let json = serde_json::to_string_pretty(record)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&path, json.as_bytes())?;
    Ok(path)
}

/// Sets the card's front image block to `asset_id`, replacing an existing
/// front `Block::Image` if there is one (the usual case: `cards build` left
/// it `None` because Wikimedia resolution failed) or appending a new one
/// when the front has none at all.
fn set_front_image_asset(card: &mut CardDefinition, asset_id: String) {
    for block in card.front.blocks.iter_mut() {
        if let Block::Image { asset_id: existing } = block {
            *existing = Some(asset_id);
            return;
        }
    }
    card.front.blocks.push(Block::Image {
        asset_id: Some(asset_id),
    });
}

/// Best-effort (extension, mime) pair for a supplied crop file. Defaults to
/// PNG — `image-grid crop` always writes PNGs, and PNG is a safe assumption
/// for a manually-supplied file whose real extension is missing or unknown.
fn guess_image_mime(path: &Path) -> (&'static str, &'static str) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => ("jpg", "image/jpeg"),
        _ => ("png", "image/png"),
    }
}

/// Best-effort `(sense, examples)` pair for a `LearningItem`, shared by
/// `image-batch` and `image-review` (v2 refinement). `sense` comes straight
/// from `item.summary` (already loaded, no extra lookup); `examples` needs
/// the full `VocabularyEntry`, looked up the same way
/// `cards.rs::build_image_to_production_blocks` does it. A missing or
/// unreadable vocabulary entry never fails the caller — it just yields no
/// examples for that item.
fn semantic_context(root: &Path, item: &LearningItem) -> (Option<String>, Option<Vec<String>>) {
    let sense = if item.summary.trim().is_empty() {
        None
    } else {
        Some(item.summary.clone())
    };

    let examples = find_vocabulary_by_id(root, &item.vocabulary_entry_id)
        .ok()
        .flatten()
        .map(|entry| {
            entry
                .examples
                .into_iter()
                .map(|example| example.text)
                .collect::<Vec<String>>()
        })
        .filter(|examples| !examples.is_empty());

    (sense, examples)
}

fn emit_error<T: Serialize + Default>(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), T::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card_with(id: &str, learning_item_id: &str, front_image_asset: Option<&str>) -> CardDefinition {
        use learnkit_cards::card::Side;
        CardDefinition {
            id: id.to_string(),
            learning_item_ids: vec![learning_item_id.to_string()],
            template: "image-to-production-v1".to_string(),
            front: Side {
                blocks: vec![
                    Block::Image {
                        asset_id: front_image_asset.map(|s| s.to_string()),
                    },
                    Block::Audio {
                        asset_id: Some("audio-1".to_string()),
                    },
                ],
            },
            back: Side { blocks: vec![] },
        }
    }

    #[test]
    fn set_front_image_asset_replaces_an_existing_block() {
        let mut card = card_with("card-1", "li-1", None);
        set_front_image_asset(&mut card, "asset-new".to_string());

        let image = card.front.blocks.iter().find_map(|b| match b {
            Block::Image { asset_id } => Some(asset_id.clone()),
            _ => None,
        });
        assert_eq!(image, Some(Some("asset-new".to_string())));
        // Still exactly one image block, not a second one appended.
        let image_blocks = card
            .front
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::Image { .. }))
            .count();
        assert_eq!(image_blocks, 1);
    }

    #[test]
    fn set_front_image_asset_appends_when_front_has_no_image_block() {
        use learnkit_cards::card::Side;
        let mut card = CardDefinition {
            id: "card-2".to_string(),
            learning_item_ids: vec!["li-2".to_string()],
            template: "word-to-meaning-v1".to_string(),
            front: Side { blocks: vec![] },
            back: Side { blocks: vec![] },
        };

        set_front_image_asset(&mut card, "asset-appended".to_string());

        let image = card.front.blocks.iter().find_map(|b| match b {
            Block::Image { asset_id } => Some(asset_id.clone()),
            _ => None,
        });
        assert_eq!(image, Some(Some("asset-appended".to_string())));
    }

    #[test]
    fn guess_image_mime_defaults_to_png() {
        assert_eq!(
            guess_image_mime(Path::new("crop-01.png")),
            ("png", "image/png")
        );
        assert_eq!(
            guess_image_mime(Path::new("no-extension")),
            ("png", "image/png")
        );
    }

    #[test]
    fn guess_image_mime_detects_jpeg() {
        assert_eq!(
            guess_image_mime(Path::new("photo.jpg")),
            ("jpg", "image/jpeg")
        );
        assert_eq!(
            guess_image_mime(Path::new("photo.JPEG")),
            ("jpg", "image/jpeg")
        );
    }
}
