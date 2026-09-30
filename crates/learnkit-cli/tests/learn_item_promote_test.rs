//! Integration tests for `learnkit learn item promote` —
//! `odd/tasks/study-pipeline-completion.md` T3. This is the only creator of
//! a non-vocabulary `LearningItem` — before this command, gramática/diálogos/
//! pronunciación could never reach `cards set`/`cards build`/`export anki`.

use assert_cmd::Command;
use std::path::Path;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn new_session(project_root: &Path) -> String {
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

fn write_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Creates a `ConceptPage` via the already-existing `analyse set`, same
/// pattern `consolidate_test.rs` uses.
fn add_concept_page(
    project_root: &Path,
    content_dir: &Path,
    session_id: &str,
    page_id: &str,
    concept: &str,
    content: &str,
) {
    let page_file = write_file(content_dir, "page.md", content);
    learnkit()
        .arg("analyse")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--item")
        .arg(page_id)
        .arg("--concept")
        .arg(concept)
        .arg("--file")
        .arg(&page_file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn add_dialogue(project_root: &Path, session_id: &str, dialogue_id: &str) {
    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--id")
        .arg(dialogue_id)
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: Hello")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn add_minimal_pair(project_root: &Path, session_id: &str, pair_id: &str) {
    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--id")
        .arg(pair_id)
        .arg("--word-a")
        .arg("sheets")
        .arg("--ipa-a")
        .arg("ʃiːts")
        .arg("--word-b")
        .arg("shits")
        .arg("--ipa-b")
        .arg("ʃɪts")
        .arg("--note")
        .arg("long vs short /i/")
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn promote_json(
    project_root: &Path,
    session_id: &str,
    kind: &str,
    source_id: &str,
) -> (i32, serde_json::Value) {
    let output = learnkit()
        .arg("learn")
        .arg("item")
        .arg("promote")
        .arg("--session")
        .arg(session_id)
        .arg("--kind")
        .arg(kind)
        .arg("--source-id")
        .arg(source_id)
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    (code, json)
}

// --- Promoting each of the 3 `--kind`s from a real, already-persisted source ---

#[test]
fn promotes_a_grammar_concept_page() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());

    add_concept_page(
        project.path(),
        content_dir.path(),
        &session_id,
        "page:layover",
        "Layover (present perfect)",
        "Use the present perfect for an experience up to now, e.g. 'I have had a layover before.'",
    );

    let (code, json) = promote_json(project.path(), &session_id, "grammar", "page:layover");
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["code"], "LEARNING_ITEM_PROMOTED");
    let learning_item_id = json["learning_item_id"].as_str().unwrap();
    assert!(!learning_item_id.is_empty());

    let item =
        learnkit_profile::language::learning_item::find_by_id(project.path(), learning_item_id)
            .unwrap()
            .unwrap();
    assert_eq!(item.kind, "grammar");
    assert_eq!(item.title, "Layover (present perfect)");
    assert!(item.summary.contains("present perfect"));
    assert_eq!(item.tags, vec!["grammar".to_string()]);
    assert_eq!(item.vocabulary_entry_id, None);
    assert_eq!(
        item.source_ref.as_deref(),
        Some(format!("grammar:{session_id}:page:layover").as_str())
    );
}

#[test]
fn promotes_a_dialogue() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());
    add_dialogue(project.path(), &session_id, "dlg-1");

    let (code, json) = promote_json(project.path(), &session_id, "dialogue", "dlg-1");
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    let learning_item_id = json["learning_item_id"].as_str().unwrap();

    let item =
        learnkit_profile::language::learning_item::find_by_id(project.path(), learning_item_id)
            .unwrap()
            .unwrap();
    assert_eq!(item.kind, "dialogue");
    assert!(item.title.contains("Hello"));
    assert!(item.summary.contains("A: Hello"));
    assert!(item.summary.contains("B: Hi"));
    assert_eq!(item.tags, vec!["dialogue".to_string()]);
}

#[test]
fn promotes_a_minimal_pair() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());
    add_minimal_pair(project.path(), &session_id, "mp-1");

    let (code, json) = promote_json(project.path(), &session_id, "pronunciation", "mp-1");
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    let learning_item_id = json["learning_item_id"].as_str().unwrap();

    let item =
        learnkit_profile::language::learning_item::find_by_id(project.path(), learning_item_id)
            .unwrap()
            .unwrap();
    assert_eq!(item.kind, "pronunciation");
    assert_eq!(item.title, "sheets vs shits");
    assert_eq!(item.summary, "long vs short /i/");
    assert_eq!(item.tags, vec!["pronunciation".to_string()]);
}

// --- Idempotency: promoting the same source twice returns the same id and
// never creates a second `LearningItem` ---

#[test]
fn promoting_the_same_source_twice_is_idempotent() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());
    add_dialogue(project.path(), &session_id, "dlg-1");

    let (_, first) = promote_json(project.path(), &session_id, "dialogue", "dlg-1");
    let (_, second) = promote_json(project.path(), &session_id, "dialogue", "dlg-1");

    assert_eq!(first["learning_item_id"], second["learning_item_id"]);

    let items_dir = project.path().join("knowledge").join("learning-items");
    let count = std::fs::read_dir(&items_dir).unwrap().count();
    assert_eq!(count, 1);
}

// --- Error: `--source-id` not found for the given `--kind` ---

#[test]
fn promoting_an_unknown_source_id_fails_clearly() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());

    let (code, json) = promote_json(
        project.path(),
        &session_id,
        "dialogue",
        "does-not-exist",
    );
    assert_ne!(code, 0);
    assert_eq!(json["ok"], false);
}

// --- The end-to-end test: promote -> `cards set` -> `cards build` produces
// a real card. This is the test that proves the audited gap (grammar/dialogue
// never reaching Anki) is actually closed, not just closed on paper. ---

#[test]
fn promoted_grammar_item_reaches_a_real_card_via_cards_set_and_build() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());

    add_concept_page(
        project.path(),
        content_dir.path(),
        &session_id,
        "page:layover",
        "Layover",
        "Explanation of the present perfect for layovers.",
    );

    let (_, promoted) = promote_json(project.path(), &session_id, "grammar", "page:layover");
    let learning_item_id = promoted["learning_item_id"].as_str().unwrap().to_string();

    learnkit()
        .arg("cards")
        .arg("set")
        .arg("--item")
        .arg(&learning_item_id)
        .arg("--activity")
        .arg("fill-in-the-blank")
        .arg("--stimulus")
        .arg("She ___ to school every day.")
        .arg("--response")
        .arg("goes")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project.path())
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

    let session_paths = learnkit_store::session_paths::SessionPaths::new(project.path(), &session_id);
    let cards = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    assert_eq!(cards.len(), 1);
    let card = &cards[0];
    assert!(card.learning_item_ids.contains(&learning_item_id));

    let back_texts: Vec<&str> = card
        .back
        .blocks
        .iter()
        .filter_map(|b| match b {
            learnkit_cards::card::Block::Text { value } => Some(value.as_str()),
            _ => None,
        })
        .collect();
    assert!(back_texts.contains(&"goes"));
}

#[test]
fn promoted_dialogue_and_pronunciation_items_also_reach_real_cards() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());
    add_dialogue(project.path(), &session_id, "dlg-1");
    add_minimal_pair(project.path(), &session_id, "mp-1");

    let (_, dlg_promoted) = promote_json(project.path(), &session_id, "dialogue", "dlg-1");
    let dlg_item_id = dlg_promoted["learning_item_id"].as_str().unwrap().to_string();
    let (_, mp_promoted) = promote_json(project.path(), &session_id, "pronunciation", "mp-1");
    let mp_item_id = mp_promoted["learning_item_id"].as_str().unwrap().to_string();

    for (item_id, stimulus, response) in [
        (dlg_item_id.as_str(), "A: Hello", "B: Hi"),
        (mp_item_id.as_str(), "sheets /\u{283}i\u{2d0}ts/", "shits /\u{283}\u{26a}ts/"),
    ] {
        learnkit()
            .arg("cards")
            .arg("set")
            .arg("--item")
            .arg(item_id)
            .arg("--activity")
            .arg("recall")
            .arg("--stimulus")
            .arg(stimulus)
            .arg("--response")
            .arg(response)
            .arg("--path")
            .arg(project.path())
            .assert()
            .success();
    }

    let out = learnkit()
        .arg("cards")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["cards"].as_array().unwrap().len(), 2);

    let session_paths = learnkit_store::session_paths::SessionPaths::new(project.path(), &session_id);
    let cards = learnkit_cards::card::load_all(session_paths.root()).unwrap();
    assert_eq!(cards.len(), 2);
}
