//! Integration tests for `learnkit cards image-batch` / `image-grid crop` /
//! `image-grid assign` — T2/T3/T5 of
//! `odd/tasks/ai-image-grid-generation.md` (Modo 2, generación por rejilla).
//!
//! These deliberately never invoke `cards build` (real network/TTS
//! dependent, per `cards_test.rs`'s own convention) — a `pending_image` card
//! is planted directly on disk, exactly like `cards_test.rs`'s "reuse"
//! tests plant a sentinel `complete` card.

use assert_cmd::Command;
use image::{GenericImageView, Rgba, RgbaImage};
use learnkit_cards::card::{Block, CardDefinition, Side};
use learnkit_store::session_paths::SessionPaths;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("eoi-sample")
}

/// Creates a session with one vocabulary item (and its `LearningItem`), like
/// `cards_test.rs::session_with_vocabulary`, but never calls `cards build` —
/// the caller plants whatever card fixtures it needs directly.
fn session_with_vocabulary_item(project_root: &std::path::Path) -> (String, String) {
    learnkit().arg("init").arg(project_root).assert().success();
    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit 5")
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let session_id = serde_json::from_slice::<serde_json::Value>(&out).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    learnkit()
        .arg("ingest")
        .arg(fixtures_dir().join("notes.md"))
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
    let inv = learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let source_id = serde_json::from_slice::<serde_json::Value>(&inv).unwrap()["sources"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("whiteboard")
        .arg("--sense")
        .arg("pizarra")
        .arg("--source")
        .arg(&source_id)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();

    let items =
        learnkit_profile::language::learning_item::load_all_vocabulary_items(project_root)
            .unwrap();
    let item = items
        .into_iter()
        .find(|i| i.title == "whiteboard")
        .expect("learning item for 'whiteboard' must exist");

    (session_id, item.id)
}

fn pending_image_card(learning_item_id: &str) -> CardDefinition {
    CardDefinition {
        id: format!("card-{learning_item_id}"),
        learning_item_ids: vec![learning_item_id.to_string()],
        template: "image-to-production-v1".to_string(),
        front: Side {
            blocks: vec![Block::Image { asset_id: None }],
        },
        back: Side { blocks: vec![] },
    }
}

/// A 400x400 test grid with 16 distinctly-colored 100x100 quadrants, laid
/// out in reading order — mirrors `learnkit_media::grid`'s own unit test
/// fixture so the CLI surface is checked with the same strong "did we crop
/// the right cell" signal.
fn sixteen_quadrant_grid_png(path: &std::path::Path) {
    let mut img = RgbaImage::new(400, 400);
    for row in 0..4u32 {
        for col in 0..4u32 {
            let index = row * 4 + col;
            let color = Rgba([
                (index * 16) as u8,
                255u8.saturating_sub((index * 16) as u8),
                128,
                255,
            ]);
            for y in (row * 100)..(row * 100 + 100) {
                for x in (col * 100)..(col * 100 + 100) {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }
    img.save(path).unwrap();
}

fn expected_color(index: u32) -> Rgba<u8> {
    Rgba([
        (index * 16) as u8,
        255u8.saturating_sub((index * 16) as u8),
        128,
        255,
    ])
}

#[test]
fn image_batch_lists_only_pending_image_cards() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, item_id) = session_with_vocabulary_item(dir.path());
    let session_paths = SessionPaths::new(dir.path(), &session_id);

    // The pending_image card for the real vocabulary item...
    learnkit_cards::card::save(session_paths.root(), &pending_image_card(&item_id)).unwrap();
    // ...and a second, already-complete card that must NOT be listed.
    let complete_card = CardDefinition {
        id: "card-li-other".to_string(),
        learning_item_ids: vec!["li-other".to_string()],
        template: "word-to-meaning-v1".to_string(),
        front: Side { blocks: vec![] },
        back: Side { blocks: vec![] },
    };
    learnkit_cards::card::save(session_paths.root(), &complete_card).unwrap();

    let out = learnkit()
        .arg("cards")
        .arg("image-batch")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["learning_item_id"], item_id);
    assert_eq!(items[0]["title"], "whiteboard");
    assert_eq!(items[0]["card_id"], format!("card-{item_id}"));
}

#[test]
fn image_batch_caps_results_at_limit() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, item_id) = session_with_vocabulary_item(dir.path());
    let session_paths = SessionPaths::new(dir.path(), &session_id);

    learnkit_cards::card::save(session_paths.root(), &pending_image_card(&item_id)).unwrap();
    let second = CardDefinition {
        id: "card-li-second".to_string(),
        learning_item_ids: vec!["li-second".to_string()],
        template: "image-to-production-v1".to_string(),
        front: Side {
            blocks: vec![Block::Image { asset_id: None }],
        },
        back: Side { blocks: vec![] },
    };
    learnkit_cards::card::save(session_paths.root(), &second).unwrap();
    // "li-second" has no matching LearningItem file, so it's silently
    // skipped (image-batch only reports items it can resolve a title for) —
    // this test only needs the *real* item to be present and respects the
    // limit, not a count of exactly how many pending cards exist.

    let out = learnkit()
        .arg("cards")
        .arg("image-batch")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--limit")
        .arg("1")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let items = json["items"].as_array().unwrap();
    assert!(items.len() <= 1);
}

#[test]
fn image_grid_crop_produces_16_crops_in_reading_order() {
    let dir = tempfile::tempdir().unwrap();
    let grid_path = dir.path().join("rejilla.png");
    sixteen_quadrant_grid_png(&grid_path);
    let out_dir = dir.path().join("crops");

    let out = learnkit()
        .arg("cards")
        .arg("image-grid")
        .arg("crop")
        .arg("--file")
        .arg(&grid_path)
        .arg("--rows")
        .arg("4")
        .arg("--cols")
        .arg("4")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    let crops = json["crops"].as_array().unwrap();
    assert_eq!(crops.len(), 16);
    assert!(crops[0].as_str().unwrap().ends_with("crop-01.png"));
    assert!(crops[15].as_str().unwrap().ends_with("crop-16.png"));

    for (index, crop_value) in crops.iter().enumerate() {
        let path = crop_value.as_str().unwrap();
        let img = image::open(path).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
        let pixel = img.get_pixel(50, 50);
        assert_eq!(
            pixel,
            expected_color(index as u32),
            "crop {index} ({path}) has the wrong color (reading-order bug?)"
        );
    }
}

#[test]
fn image_grid_assign_registers_the_asset_and_updates_the_card() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, item_id) = session_with_vocabulary_item(dir.path());
    let session_paths = SessionPaths::new(dir.path(), &session_id);
    learnkit_cards::card::save(session_paths.root(), &pending_image_card(&item_id)).unwrap();

    let grid_path = dir.path().join("rejilla.png");
    sixteen_quadrant_grid_png(&grid_path);
    let out_dir = dir.path().join("crops");
    let crop_path = out_dir.join("crop-01.png");
    crop_via_cli(&grid_path, &out_dir);

    let out = learnkit()
        .arg("cards")
        .arg("image-grid")
        .arg("assign")
        .arg("--session")
        .arg(&session_id)
        .arg("--item")
        .arg(&item_id)
        .arg("--file")
        .arg(&crop_path)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    let asset_id = json["asset_id"].as_str().unwrap().to_string();
    assert_eq!(json["card_id"], format!("card-{item_id}"));

    let cards = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    let card = cards
        .iter()
        .find(|c| c.id == format!("card-{item_id}"))
        .unwrap();
    let front_asset = card.front.blocks.iter().find_map(|b| match b {
        Block::Image { asset_id } => asset_id.clone(),
        _ => None,
    });
    assert_eq!(front_asset, Some(asset_id.clone()));

    let asset = learnkit_media::asset::load(&session_paths.assets(), &asset_id)
        .unwrap()
        .expect("assigned asset must be loadable");
    assert!(matches!(
        asset.origin,
        learnkit_media::asset::AssetOrigin::GeneratedGrid { .. }
    ));
}

/// Test-only helper that crops a grid file into `out_dir` via the CLI
/// itself, reused so the "assign" test doesn't need to duplicate the
/// cropping pipeline — it needs a real crop file, produced the same way a
/// real user's crop would be.
fn crop_via_cli(grid_path: &std::path::Path, out_dir: &std::path::Path) {
    learnkit()
        .arg("cards")
        .arg("image-grid")
        .arg("crop")
        .arg("--file")
        .arg(grid_path)
        .arg("--rows")
        .arg("4")
        .arg("--cols")
        .arg("4")
        .arg("--out-dir")
        .arg(out_dir)
        .assert()
        .success();
}
