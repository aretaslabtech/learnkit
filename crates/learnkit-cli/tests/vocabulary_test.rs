//! Integration tests for `learnkit learn vocabulary add`, User Story 3
//! (specs/002-english-eoi-flow/tasks.md T028-T030).

use assert_cmd::Command;

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

fn new_session_with_notes(project_root: &std::path::Path) -> (String, String) {
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

    (session_id, source_id)
}

#[test]
fn learn_vocabulary_add_creates_entry_with_traceability() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = new_session_with_notes(dir.path());

    let out = learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("get away with")
        .arg("--sense")
        .arg("hacer algo malo sin castigo")
        .arg("--source")
        .arg(&source_id)
        .arg("--locator")
        .arg("notes:line-4")
        .arg("--suggested-by")
        .arg("agent")
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
    assert!(json["vocabulary_id"]
        .as_str()
        .unwrap()
        .starts_with("vocab-en-"));
    assert!(json["learning_item_id"]
        .as_str()
        .unwrap()
        .starts_with("li-"));
}

#[test]
fn same_lemma_across_sessions_reuses_the_entry() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = new_session_with_notes(dir.path());

    let first = learnkit()
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
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let first_id = serde_json::from_slice::<serde_json::Value>(&first).unwrap()["vocabulary_id"]
        .as_str()
        .unwrap()
        .to_string();

    // A second session referencing the same lemma reuses the entry.
    let second_session_out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit 6")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_session_id = serde_json::from_slice::<serde_json::Value>(&second_session_out)
        .unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();
    learnkit()
        .arg("ingest")
        .arg(fixtures_dir().join("notes.md"))
        .arg("--session")
        .arg(&second_session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    let inv2 = learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(&second_session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let source_id_2 = serde_json::from_slice::<serde_json::Value>(&inv2).unwrap()["sources"][0]
        ["id"]
        .as_str()
        .unwrap()
        .to_string();

    let second = learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&second_session_id)
        .arg("--lemma")
        .arg("Whiteboard")
        .arg("--sense")
        .arg("pizarra")
        .arg("--source")
        .arg(&source_id_2)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_id = serde_json::from_slice::<serde_json::Value>(&second).unwrap()["vocabulary_id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(first_id, second_id);
}

#[test]
fn manual_entry_without_any_prior_suggestion_works() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = new_session_with_notes(dir.path());

    learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("put up with")
        .arg("--sense")
        .arg("tolerar")
        .arg("--source")
        .arg(&source_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn rejects_a_source_id_that_does_not_belong_to_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, _source_id) = new_session_with_notes(dir.path());

    learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("bogus")
        .arg("--sense")
        .arg("bogus")
        .arg("--source")
        .arg("src-does-not-exist")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure();
}

/// FR-012c: `learn vocabulary remove` is the only supported way to delete
/// vocabulary state — it must cascade to the learning item and to any card
/// (in any session) that depends on it, and a repeat removal of the same
/// lemma must fail explicitly rather than silently succeed.
#[test]
fn learn_vocabulary_remove_deletes_entry_learning_item_and_dependent_cards() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = new_session_with_notes(dir.path());

    let add_out = learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("walkie-talkie")
        .arg("--sense")
        .arg("walkie-talkie")
        .arg("--source")
        .arg(&source_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let add_json: serde_json::Value = serde_json::from_slice(&add_out).unwrap();
    let vocabulary_id = add_json["vocabulary_id"].as_str().unwrap().to_string();
    let learning_item_id = add_json["learning_item_id"].as_str().unwrap().to_string();

    let session_root = dir.path().join("sessions").join(&session_id);
    let card = learnkit_cards::card::CardDefinition {
        id: format!("card-{learning_item_id}"),
        learning_item_ids: vec![learning_item_id.clone()],
        template: "image-to-production-v1".to_string(),
        front: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Image {
                asset_id: Some("asset-1".to_string()),
            }],
        },
        back: learnkit_cards::card::Side {
            blocks: vec![learnkit_cards::card::Block::Text {
                value: "walkie-talkie".to_string(),
            }],
        },
    };
    learnkit_cards::card::save(&session_root, &card).unwrap();

    let remove_out = learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("remove")
        .arg("--lemma")
        .arg("walkie-talkie")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let remove_json: serde_json::Value = serde_json::from_slice(&remove_out).unwrap();
    assert_eq!(remove_json["ok"], true);
    assert_eq!(remove_json["vocabulary_id"], vocabulary_id);
    assert_eq!(remove_json["learning_item_id"], learning_item_id);
    assert_eq!(
        remove_json["removed_card_ids"].as_array().unwrap(),
        &[serde_json::Value::String(card.id.clone())]
    );

    assert!(learnkit_cards::card::load_all(&session_root)
        .unwrap()
        .is_empty());
    assert!(!dir
        .path()
        .join("knowledge")
        .join("vocabulary")
        .join(format!("{vocabulary_id}.yaml"))
        .exists());
    assert!(!dir
        .path()
        .join("knowledge")
        .join("learning-items")
        .join(format!("{learning_item_id}.yaml"))
        .exists());

    // Removing the same lemma again must fail explicitly, not succeed silently.
    learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("remove")
        .arg("--lemma")
        .arg("walkie-talkie")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure();
}
