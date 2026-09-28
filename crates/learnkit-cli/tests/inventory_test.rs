//! Integration tests for `learnkit ingest`/`learnkit inventory`,
//! User Story 1 (specs/002-english-eoi-flow/tasks.md T010-T012).

use assert_cmd::Command;
use std::fs;

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

fn new_session(project_root: &std::path::Path) -> String {
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
fn ingest_and_inventory_register_all_source_types_with_hash() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let fixtures = fixtures_dir();

    learnkit()
        .arg("ingest")
        .arg(fixtures.join("notes.md"))
        .arg(fixtures.join("class-audio.wav"))
        .arg(fixtures.join("whiteboard.png"))
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("inventory")
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
    let sources = json["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 3);
    for s in sources {
        assert_eq!(s["change"], "added");
        assert!(!s["sha256"].as_str().unwrap().is_empty());
    }
}

#[test]
fn reinventorying_without_changes_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let fixtures = fixtures_dir();

    learnkit()
        .arg("ingest")
        .arg(fixtures.join("notes.md"))
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("inventory")
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
    let sources = json["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["change"], "unchanged");
}

#[test]
fn changing_source_content_marks_it_updated_with_new_hash() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let fixtures = fixtures_dir();

    learnkit()
        .arg("ingest")
        .arg(fixtures.join("notes.md"))
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    let first = learnkit()
        .arg("inventory")
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
    let first_hash = serde_json::from_slice::<serde_json::Value>(&first).unwrap()["sources"][0]
        ["sha256"]
        .as_str()
        .unwrap()
        .to_string();

    let notes_path = dir
        .path()
        .join("sessions")
        .join(&session_id)
        .join("input")
        .join("notes.md");
    fs::write(&notes_path, "contenido cambiado a mano\n").unwrap();

    let second = learnkit()
        .arg("inventory")
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
    let second_json: serde_json::Value = serde_json::from_slice(&second).unwrap();
    assert_eq!(second_json["sources"][0]["change"], "updated");
    assert_ne!(
        second_json["sources"][0]["sha256"].as_str().unwrap(),
        first_hash
    );
}
