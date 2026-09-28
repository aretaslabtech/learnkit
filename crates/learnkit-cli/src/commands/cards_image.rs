//! `learnkit cards image-batch` / `image-grid crop` / `image-grid assign` —
//! Modo 2 (generación por rejilla) of `odd/tasks/ai-image-grid-generation.md`.
//!
//! Everything here is mechanical (list what's missing, crop a grid image,
//! persist an already-approved crop as a card's image asset). Rust never
//! judges whether an image is coherent with its concept — that's always the
//! agent, via a Skill built on top of these commands (T7, out of scope
//! here).

use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_cards::card::{completeness, load_all, save, Block, CardDefinition, Completeness};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_media::asset::{self, AssetOrigin, AssetType};
use learnkit_media::grid;
use learnkit_profile::language::learning_item::{load_all_vocabulary_items, LearningItem};
use learnkit_store::session_paths::SessionPaths;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Provider tag recorded on every asset assigned via `image-grid assign` —
/// distinct from Wikimedia's `fetched` and audio's `generated`, per T4.
const GRID_PROVIDER: &str = "chatgpt-grid-manual";

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

#[derive(Debug, Serialize)]
struct ImageBatchItem {
    learning_item_id: String,
    title: String,
    card_id: String,
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
        if !matches!(completeness(card), Completeness::PendingImage { .. }) {
            continue;
        }
        let Some(item_id) = card.learning_item_ids.first() else {
            continue;
        };
        let Some(item) = items_by_id.get(item_id) else {
            continue;
        };
        batch.push(ImageBatchItem {
            learning_item_id: item.id.clone(),
            title: item.title.clone(),
            card_id: card.id.clone(),
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
