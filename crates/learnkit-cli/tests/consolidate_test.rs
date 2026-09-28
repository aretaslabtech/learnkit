//! Integration tests for `learnkit consolidate` / `learnkit consolidate
//! list`, User Story 4 (specs/003-analyse-consolidate-checklist/tasks.md
//! T027-T030).

use assert_cmd::Command;
use std::path::Path;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("analyse-sample")
}

/// Creates a project + session and ingests/inventories the T001 fixture
/// (`analyse-sample/notes.md`) — same setup `analyse_test.rs` uses, needed
/// here so `consolidate` has a real `analyse`-dependent session to run on.
fn new_session_with_notes(project_root: &Path) -> String {
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
    learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();

    session_id
}

fn write_content_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn first_source_id(project_root: &Path, session_id: &str) -> String {
    let out = learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(session_id)
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    json["sources"][0]["id"].as_str().unwrap().to_string()
}

fn confirm_summary(project_root: &Path, content_dir: &Path, session_id: &str, content: &str) {
    let summary_file = write_content_file(content_dir, "summary.md", content);
    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn add_concept_page(
    project_root: &Path,
    content_dir: &Path,
    session_id: &str,
    concept: &str,
    content: &str,
) {
    let page_file = write_content_file(content_dir, "page.md", content);
    learnkit()
        .arg("analyse")
        .arg("page")
        .arg("add")
        .arg("--session")
        .arg(session_id)
        .arg("--concept")
        .arg(concept)
        .arg("--file")
        .arg(&page_file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn consolidate_json(project_root: &Path, session_id: &str) -> (i32, serde_json::Value) {
    let output = learnkit()
        .arg("consolidate")
        .arg("--session")
        .arg(session_id)
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    (code, json)
}

fn consolidate_list_json(project_root: &Path, session_id: &str) -> serde_json::Value {
    let out = learnkit()
        .arg("consolidate")
        .arg("list")
        .arg("--session")
        .arg(session_id)
        .arg("--path")
        .arg(project_root)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

// --- T027: `consolidate` produces a non-empty candidate list, traceable to
// its origin ---

#[test]
fn consolidate_produces_traceable_candidates_from_summary_and_pages() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    confirm_summary(
        project.path(),
        content_dir.path(),
        &session_id,
        "Resumen: repasamos vocabulario de viajes como itinerary, layover y boarding.",
    );
    add_concept_page(
        project.path(),
        content_dir.path(),
        &session_id,
        "Present perfect",
        "Explicación concentrada del concepto checkpoint para el viaje.",
    );

    let (code, json) = consolidate_json(project.path(), &session_id);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let candidates = json["candidates"].as_array().unwrap();
    assert!(!candidates.is_empty(), "expected at least one candidate");
    for candidate in candidates {
        assert!(!candidate["text"].as_str().unwrap().is_empty());
        assert!(!candidate["source_ref"]["origin"]
            .as_str()
            .unwrap()
            .is_empty());
    }
    // At least one candidate traces back to the summary, and one to the page.
    assert!(candidates
        .iter()
        .any(|c| c["source_ref"]["origin"] == "summary"));
    assert!(candidates
        .iter()
        .any(|c| c["source_ref"]["origin"] == "page-1"));
}

// --- T028: a candidate whose lemma already exists as a `VocabularyEntry`
// (from a previous session) is reported with `already_exists: true`, not
// dropped ---

#[test]
fn consolidate_marks_already_existing_vocabulary_without_dropping_it() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();

    // Session A: simulate "itinerary" already confirmed as real vocabulary
    // in a previous session, via the existing feature-002 command.
    let session_a = new_session_with_notes(project.path());
    let source_id = first_source_id(project.path(), &session_a);
    let add_out = learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_a)
        .arg("--lemma")
        .arg("itinerary")
        .arg("--sense")
        .arg("plan de viaje")
        .arg("--source")
        .arg(&source_id)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let add_json: serde_json::Value = serde_json::from_slice(&add_out).unwrap();
    let vocabulary_id = add_json["vocabulary_id"].as_str().unwrap().to_string();

    // Session B: a different session whose summary mentions the same word.
    let session_b = new_session_with_notes(project.path());
    confirm_summary(
        project.path(),
        content_dir.path(),
        &session_b,
        "Resumen: volvimos a ver la palabra itinerary en varios ejemplos.",
    );

    let (code, json) = consolidate_json(project.path(), &session_b);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);

    let candidates = json["candidates"].as_array().unwrap();
    let itinerary = candidates
        .iter()
        .find(|c| c["text"] == "itinerary")
        .expect("'itinerary' candidate still reported, not silently dropped");
    assert_eq!(itinerary["already_exists"], true);
    assert_eq!(itinerary["existing_vocabulary_id"], vocabulary_id);
}

// --- T029: a session with an `analyse` item still `pending_user_decision`
// blocks `consolidate` (exit 20, BLOCKED), producing no candidates ---

#[test]
fn consolidate_blocks_when_an_analyse_item_is_pending_user_decision() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    confirm_summary(
        project.path(),
        content_dir.path(),
        &session_id,
        "Resumen con vocabulario de viajes como itinerary.",
    );
    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("no hay material suficiente para el mapa mental")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let (code, json) = consolidate_json(project.path(), &session_id);
    assert_eq!(code, 20);
    assert_eq!(json["ok"], false);
    assert_eq!(json["code"], "BLOCKED");
    let pending_items = json["pending_items"].as_array().unwrap();
    assert!(pending_items.iter().any(|i| i["item_id"] == "mindmap"));

    // No candidates were persisted for this blocked run.
    let listed = consolidate_list_json(project.path(), &session_id);
    assert_eq!(listed["candidates"].as_array().unwrap().len(), 0);
}

// --- T030: `consolidate list` shows the candidates already consolidated,
// with origin and duplicate status ---

#[test]
fn consolidate_list_shows_persisted_candidates_with_origin_and_duplicate_status() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    confirm_summary(
        project.path(),
        content_dir.path(),
        &session_id,
        "Resumen: repasamos vocabulario de viajes como itinerary y layover.",
    );

    let (code, run_json) = consolidate_json(project.path(), &session_id);
    assert_eq!(code, 0);
    let run_candidates = run_json["candidates"].as_array().unwrap().clone();
    assert!(!run_candidates.is_empty());

    let listed = consolidate_list_json(project.path(), &session_id);
    assert_eq!(listed["ok"], true);
    let listed_candidates = listed["candidates"].as_array().unwrap();
    assert_eq!(listed_candidates.len(), run_candidates.len());

    for candidate in listed_candidates {
        assert!(!candidate["source_ref"]["origin"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(candidate.get("already_exists").is_some());
    }
}
