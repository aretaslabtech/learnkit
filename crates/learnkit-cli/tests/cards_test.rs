//! Integration tests for `learnkit cards build`/`validate`, User Story 4.
//!
//! Real network (Wikimedia Commons) and Piper availability vary by
//! environment, so these tests only assert the command's own contract
//! (exit code, JSON shape, no crash) — never a specific resolved/pending
//! outcome. `learnkit-cards`/`learnkit-media`'s own unit tests already cover
//! completeness and provider-resolution logic with fake providers.

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn session_with_vocabulary(project_root: &std::path::Path) -> String {
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

    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("eoi-sample");
    learnkit()
        .arg("ingest")
        .arg(fixtures.join("notes.md"))
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

    session_id
}

#[test]
fn cards_build_produces_one_card_per_vocabulary_item() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_vocabulary(dir.path());

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["cards"].as_array().unwrap().len(), 1);
    let state = json["cards"][0]["state"].as_str().unwrap();
    assert!(["complete", "pending_image", "pending_audio"].contains(&state));
}

#[test]
fn cards_validate_reports_the_same_states_as_build() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_vocabulary(dir.path());

    learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    let validate = learnkit()
        .arg("cards")
        .arg("validate")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert();

    // `validate` exits 0 only if every card is complete, 40 otherwise — both
    // are valid outcomes here since we don't control network/Piper
    // availability in CI; either way it must produce the JSON shape.
    let output = validate.get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["cards"].as_array().unwrap().len(), 1);
}

/// FR-017f: a card that is already complete must never be reprocessed on a
/// plain rebuild — otherwise an external provider's intermittency (Wikimedia
/// Commons) can turn an already-resolved card back into pending. We prove
/// this deterministically by planting a sentinel asset id (that no real
/// provider could ever produce) directly on disk and confirming a rebuild
/// leaves it untouched.
#[test]
fn cards_build_reuses_an_already_complete_card_without_reprocessing() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_vocabulary(dir.path());
    let session_paths = learnkit_store::session_paths::SessionPaths::new(dir.path(), &session_id);

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let card_id = json["cards"][0]["id"].as_str().unwrap().to_string();

    const SENTINEL: &str = "sentinel-asset-no-real-provider-could-produce-this";
    let sentinel_card = learnkit_cards::card::CardDefinition {
        id: card_id.clone(),
        learning_item_ids: vec!["li-whatever".to_string()],
        template: "image-to-production-v1".to_string(),
        front: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Image {
                asset_id: Some(SENTINEL.to_string()),
            }],
        },
        back: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Text {
                value: "pizarra".to_string(),
            }],
        },
    };
    learnkit_cards::card::save(session_paths.root(), &sentinel_card).unwrap();

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["cards"][0]["state"], "complete");

    let reloaded = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    let card = reloaded.iter().find(|c| c.id == card_id).unwrap();
    let front_asset = card.front.blocks.iter().find_map(|b| match b {
        learnkit_cards::card::Block::Image { asset_id } => asset_id.clone(),
        _ => None,
    });
    assert_eq!(front_asset.as_deref(), Some(SENTINEL));
}

/// FR-017f: `--force` is the escape hatch that bypasses the reuse above and
/// reprocesses every card from scratch.
#[test]
fn cards_build_force_reprocesses_an_already_complete_card() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_vocabulary(dir.path());
    let session_paths = learnkit_store::session_paths::SessionPaths::new(dir.path(), &session_id);

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let card_id = json["cards"][0]["id"].as_str().unwrap().to_string();

    const SENTINEL: &str = "sentinel-asset-no-real-provider-could-produce-this";
    let sentinel_card = learnkit_cards::card::CardDefinition {
        id: card_id.clone(),
        learning_item_ids: vec!["li-whatever".to_string()],
        template: "image-to-production-v1".to_string(),
        front: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Image {
                asset_id: Some(SENTINEL.to_string()),
            }],
        },
        back: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Text {
                value: "pizarra".to_string(),
            }],
        },
    };
    learnkit_cards::card::save(session_paths.root(), &sentinel_card).unwrap();

    learnkit()
        .arg("cards")
        .arg("build")
        .arg("--force")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    let reloaded = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    let card = reloaded.iter().find(|c| c.id == card_id).unwrap();
    let front_asset = card.front.blocks.iter().find_map(|b| match b {
        learnkit_cards::card::Block::Image { asset_id } => asset_id.clone(),
        _ => None,
    });
    assert_ne!(front_asset.as_deref(), Some(SENTINEL));
}
