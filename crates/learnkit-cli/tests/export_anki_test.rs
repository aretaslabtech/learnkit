//! Integration tests for `learnkit export anki`, User Story 5
//! (specs/002-english-eoi-flow/tasks.md T051-T053).
//!
//! Card completeness is set up directly via `learnkit-cards` (rather than
//! going through `cards build`'s real Wikimedia/Piper providers) so these
//! tests are deterministic regardless of network/Piper availability.

use assert_cmd::Command;
use learnkit_cards::card::{save, Block, CardDefinition, Side};
use learnkit_store::session_paths::SessionPaths;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn init_session(project_root: &std::path::Path) -> String {
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
    serde_json::from_slice::<serde_json::Value>(&out).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn exporting_complete_cards_produces_a_valid_apkg() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = init_session(dir.path());
    let paths = SessionPaths::new(dir.path(), &session_id);

    let card = CardDefinition {
        id: "card-complete".to_string(),
        learning_item_ids: vec!["li-1".to_string()],
        template: "word-to-meaning-v1".to_string(),
        front: Side {
            blocks: vec![Block::Text {
                value: "whiteboard".to_string(),
            }],
        },
        back: Side {
            blocks: vec![Block::Text {
                value: "pizarra".to_string(),
            }],
        },
    };
    save(paths.root(), &card).unwrap();

    let out_path = dir.path().join("dist").join("deck.apkg");
    learnkit()
        .arg("export")
        .arg("anki")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_path)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn exporting_with_an_incomplete_card_fails_explicitly_without_a_partial_file() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = init_session(dir.path());
    let paths = SessionPaths::new(dir.path(), &session_id);

    let card = CardDefinition {
        id: "card-incomplete".to_string(),
        learning_item_ids: vec!["li-2".to_string()],
        template: "image-to-production-v1".to_string(),
        front: Side {
            blocks: vec![Block::Image { asset_id: None }],
        },
        back: Side {
            blocks: vec![Block::Text {
                value: "sin imagen".to_string(),
            }],
        },
    };
    save(paths.root(), &card).unwrap();

    let out_path = dir.path().join("dist").join("deck.apkg");
    let assert = learnkit()
        .arg("export")
        .arg("anki")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_path)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .failure()
        .code(40);

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(output["incomplete_cards"][0]["id"], "card-incomplete");
    assert!(!out_path.exists());
}

// --- T101-T103 (FR-019b, post-release): --skip-incomplete excludes
// incomplete cards instead of blocking the whole export ---

#[test]
fn export_with_skip_incomplete_excludes_the_incomplete_card_and_reports_it() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = init_session(dir.path());
    let paths = SessionPaths::new(dir.path(), &session_id);

    let complete = CardDefinition {
        id: "card-complete".to_string(),
        learning_item_ids: vec!["li-1".to_string()],
        template: "word-to-meaning-v1".to_string(),
        front: Side {
            blocks: vec![Block::Text {
                value: "whiteboard".to_string(),
            }],
        },
        back: Side {
            blocks: vec![Block::Text {
                value: "pizarra".to_string(),
            }],
        },
    };
    save(paths.root(), &complete).unwrap();

    let incomplete = CardDefinition {
        id: "card-ty-numbers".to_string(),
        learning_item_ids: vec!["li-2".to_string()],
        template: "image-to-production-v1".to_string(),
        front: Side {
            blocks: vec![Block::Image { asset_id: None }],
        },
        back: Side {
            blocks: vec![Block::Text {
                value: "sin imagen".to_string(),
            }],
        },
    };
    save(paths.root(), &incomplete).unwrap();

    let out_path = dir.path().join("dist").join("deck.apkg");
    let assert = learnkit()
        .arg("export")
        .arg("anki")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_path)
        .arg("--path")
        .arg(dir.path())
        .arg("--skip-incomplete")
        .arg("--json")
        .assert()
        .success();

    // Exports the valid .apkg with only the complete card...
    assert!(out_path.exists());
    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(output["ok"], true);
    assert_eq!(output["card_count"], 1);
    // ...and lists exactly which card was excluded and why, never silently.
    assert_eq!(output["excluded_cards"][0]["id"], "card-ty-numbers");
    assert!(!output["excluded_cards"][0]["reason"]
        .as_str()
        .unwrap()
        .is_empty());
}

#[test]
fn reexporting_the_same_cards_keeps_stable_note_identity() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = init_session(dir.path());
    let paths = SessionPaths::new(dir.path(), &session_id);

    let card = CardDefinition {
        id: "card-stable".to_string(),
        learning_item_ids: vec!["li-3".to_string()],
        template: "word-to-meaning-v1".to_string(),
        front: Side {
            blocks: vec![Block::Text {
                value: "get away with".to_string(),
            }],
        },
        back: Side { blocks: vec![] },
    };
    save(paths.root(), &card).unwrap();

    let out_path = dir.path().join("deck.apkg");
    learnkit()
        .arg("export")
        .arg("anki")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_path)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    assert!(out_path.exists());

    // Re-running the export over the same cards must succeed again (not
    // error out because the file already exists) — the underlying note/card
    // id stability across exports of the same `card_id` is unit-tested
    // directly in `learnkit-anki`'s `package` module.
    learnkit()
        .arg("export")
        .arg("anki")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_path)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    assert!(out_path.exists());
}
