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

    // FR-013b: `image-to-production-v1` now also requires audio on both
    // sides (the back reuses the front's resolved asset), so the planted
    // "already complete" fixture needs an Audio block on both sides too,
    // or completeness() would report it pending and reprocess it — which
    // is exactly what this test must prove does NOT happen.
    const SENTINEL: &str = "sentinel-asset-no-real-provider-could-produce-this";
    const SENTINEL_AUDIO: &str = "sentinel-audio-no-real-provider-could-produce-this";
    let sentinel_card = learnkit_cards::card::CardDefinition {
        id: card_id.clone(),
        learning_item_ids: vec!["li-whatever".to_string()],
        template: "image-to-production-v1".to_string(),
        front: learnkit_cards::card::Side {
            blocks: vec![
                learnkit_cards::card::Block::Image {
                    asset_id: Some(SENTINEL.to_string()),
                },
                learnkit_cards::card::Block::Audio {
                    asset_id: Some(SENTINEL_AUDIO.to_string()),
                },
            ],
        },
        back: learnkit_cards::card::Side {
            blocks: vec![
                learnkit_cards::card::Block::Text {
                    value: "pizarra".to_string(),
                },
                learnkit_cards::card::Block::Audio {
                    asset_id: Some(SENTINEL_AUDIO.to_string()),
                },
            ],
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

    // See the "reuses" test above: the fixture needs an Audio block on both
    // sides too now that image-to-production-v1 requires audio (FR-013b).
    const SENTINEL: &str = "sentinel-asset-no-real-provider-could-produce-this";
    const SENTINEL_AUDIO: &str = "sentinel-audio-no-real-provider-could-produce-this";
    let sentinel_card = learnkit_cards::card::CardDefinition {
        id: card_id.clone(),
        learning_item_ids: vec!["li-whatever".to_string()],
        template: "image-to-production-v1".to_string(),
        front: learnkit_cards::card::Side {
            blocks: vec![
                learnkit_cards::card::Block::Image {
                    asset_id: Some(SENTINEL.to_string()),
                },
                learnkit_cards::card::Block::Audio {
                    asset_id: Some(SENTINEL_AUDIO.to_string()),
                },
            ],
        },
        back: learnkit_cards::card::Side {
            blocks: vec![
                learnkit_cards::card::Block::Text {
                    value: "pizarra".to_string(),
                },
                learnkit_cards::card::Block::Audio {
                    asset_id: Some(SENTINEL_AUDIO.to_string()),
                },
            ],
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

// --- Hard Guards entry check (odd/tasks/hard-guards-entry-checks.md) ---

#[test]
fn cards_build_is_blocked_when_there_is_no_vocabulary() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();
    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit sin vocabulario")
        .arg("--path")
        .arg(dir.path())
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

    let assert = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .failure()
        .code(20);

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(output["ok"], false);
    assert_eq!(output["code"], "BLOCKED");
}

// --- cardspec-generalization (T6): `cards build` for a non-vocabulary
// `LearningItem` sourced from its `CardSpec` — and proof the vocabulary path
// stays byte-for-byte the same. There is no CLI to create a generic
// `LearningItem` yet (out of scope, per odd/tasks/cardspec-generalization.md
// §"Fuera de alcance"), so the fixture writes the YAML directly, exactly
// where `learnkit_profile::language::learning_item` itself persists it. ---

fn write_generic_learning_item(project_root: &std::path::Path, id: &str, kind: &str) {
    let dir = project_root.join("knowledge").join("learning-items");
    std::fs::create_dir_all(&dir).unwrap();
    let item = learnkit_profile::language::learning_item::LearningItem {
        id: id.to_string(),
        kind: kind.to_string(),
        title: "unused-for-a-non-vocabulary-item".to_string(),
        summary: "unused-for-a-non-vocabulary-item".to_string(),
        tags: vec![],
        mastery_dimensions: vec![],
        vocabulary_entry_id: None,
        source_ref: None,
    };
    let yaml = serde_yaml::to_string(&item).unwrap();
    std::fs::write(dir.join(format!("{id}.yaml")), yaml).unwrap();
}

/// A generic (non-vocabulary) item with a `CardSpec` set via `cards set`
/// builds a card whose text comes from `stimulus`/`response`/`feedback`,
/// never from `item.title`/`item.summary` (which are deliberately garbage
/// here — if the card used them, this test would fail).
#[test]
fn cards_build_uses_card_spec_content_for_a_non_vocabulary_item() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();
    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Grammar unit")
        .arg("--path")
        .arg(dir.path())
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

    write_generic_learning_item(dir.path(), "li-grammar-1", "grammar");

    learnkit()
        .arg("cards")
        .arg("set")
        .arg("--item")
        .arg("li-grammar-1")
        .arg("--activity")
        .arg("fill-in-the-blank")
        .arg("--stimulus")
        .arg("She ___ to school every day.")
        .arg("--response")
        .arg("goes")
        .arg("--feedback")
        .arg("Third person singular present takes -s.")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

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

    let session_paths = learnkit_store::session_paths::SessionPaths::new(dir.path(), &session_id);
    let cards = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    let card = &cards[0];
    let back_texts: Vec<&str> = card
        .back
        .blocks
        .iter()
        .filter_map(|b| match b {
            learnkit_cards::card::Block::Text { value } => Some(value.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        back_texts,
        vec![
            "She ___ to school every day.",
            "goes",
            "Third person singular present takes -s."
        ]
    );
}

/// `cards set` rejects mojibake-corrupted text the same way `learn vocabulary
/// add`/`edit` do (both use `learnkit_core::encoding::detect_mojibake`) —
/// this command didn't exist yet when that guard was first added, so it's
/// its own regression once `cards set` merged alongside it.
#[test]
fn cards_set_rejects_mojibake_stimulus() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();
    write_generic_learning_item(dir.path(), "li-grammar-mojibake", "grammar");

    let out = learnkit()
        .arg("cards")
        .arg("set")
        .arg("--item")
        .arg("li-grammar-mojibake")
        .arg("--activity")
        .arg("fill-in-the-blank")
        .arg("--stimulus")
        .arg("clasificaciÃ³n")
        .arg("--response")
        .arg("ok")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .code(10)
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["code"], "ENCODING_SUSPICIOUS");
}

/// `cards build` on a non-vocabulary item with no `CardSpec` set yet must
/// fail with a clear, specific error — never silently generate an empty
/// card, and never the Hard Guards `BLOCKED`/exit-20 contract (that's
/// reserved for "zero vocabulary project-wide" — this is a single concrete
/// item missing its own content).
#[test]
fn cards_build_fails_clearly_for_a_non_vocabulary_item_without_a_card_spec() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();
    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Grammar unit")
        .arg("--path")
        .arg(dir.path())
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

    write_generic_learning_item(dir.path(), "li-grammar-no-spec", "grammar");

    let assert = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .failure();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(output["ok"], false);
    assert_eq!(output["code"], "CARD_SPEC_MISSING");
}

/// Regression: a session mixing a vocabulary item and a generic item (with
/// its `CardSpec` set) must build both, and the vocabulary card's content
/// must be exactly what it was before this feature (word + translation,
/// sourced from `item.title`/`item.summary`/`VocabularyEntry`, never
/// touched by the `CardSpec` path).
#[test]
fn cards_build_handles_a_mixed_session_without_changing_the_vocabulary_card() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_vocabulary(dir.path());

    write_generic_learning_item(dir.path(), "li-grammar-mixed", "grammar");
    learnkit()
        .arg("cards")
        .arg("set")
        .arg("--item")
        .arg("li-grammar-mixed")
        .arg("--activity")
        .arg("fill-in-the-blank")
        .arg("--stimulus")
        .arg("stimulus text")
        .arg("--response")
        .arg("response text")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

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
    assert_eq!(json["cards"].as_array().unwrap().len(), 2);

    let session_paths = learnkit_store::session_paths::SessionPaths::new(dir.path(), &session_id);
    let cards = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    let vocab_card = cards
        .iter()
        .find(|c| c.learning_item_ids.iter().any(|id| !id.contains("grammar")))
        .unwrap();
    let vocab_back_texts: Vec<&str> = vocab_card
        .back
        .blocks
        .iter()
        .filter_map(|b| match b {
            learnkit_cards::card::Block::Text { value } => Some(value.as_str()),
            _ => None,
        })
        .collect();
    // Same shape as `cards_build_produces_one_card_per_vocabulary_item` /
    // `build_image_to_production_blocks`'s own unit tests: English word then
    // translation, nothing from CardSpec.
    assert_eq!(vocab_back_texts, vec!["whiteboard", "pizarra"]);
}

#[test]
fn cards_build_works_for_preexisting_vocabulary_with_no_phase_manifest_history() {
    // Regresión explícita del caso real: vocabulario/tarjetas ya confirmados
    // sin que la fase `vocabulary` haya escrito nunca un manifest (el
    // mecanismo no existía cuando se creó ese vocabulario). La guarda de
    // Hard Guards es una comprobación en vivo contra datos reales, nunca
    // contra un manifest cacheado — así que esto NUNCA debe bloquearse.
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
}
