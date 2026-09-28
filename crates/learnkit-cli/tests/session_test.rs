//! Integration tests for `learnkit session`, User Story 1
//! (specs/002-english-eoi-flow/tasks.md T009, T031).

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

#[test]
fn session_new_creates_session_with_stable_id_and_directory() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let output = learnkit()
        .arg("session")
        .arg("new")
        .arg("EOI — Unit 5")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    let session_id = json["session_id"].as_str().unwrap().to_string();
    assert!(!session_id.is_empty());
    assert!(dir
        .path()
        .join("sessions")
        .join(&session_id)
        .join("session.yaml")
        .exists());
}

#[test]
fn session_show_reports_sources_after_ingest_and_inventory() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let create = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit 5")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let session_id = serde_json::from_slice::<serde_json::Value>(&create).unwrap()["session_id"]
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

    let show = learnkit()
        .arg("session")
        .arg("show")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&show).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["sources"].as_array().unwrap().len(), 1);
}

#[test]
fn session_show_exposes_notes_text_and_transcript_segments_for_agent_use() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let create = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit 5")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let session_id = serde_json::from_slice::<serde_json::Value>(&create).unwrap()["session_id"]
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
        .arg(fixtures.join("class-audio.wav"))
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    let inv = learnkit()
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
    let inv_json: serde_json::Value = serde_json::from_slice(&inv).unwrap();
    let audio_source_id = inv_json["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["kind"] == "audio")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let srt = dir.path().join("clase.srt");
    std::fs::write(&srt, "1\n00:00:00,000 --> 00:00:01,000\nHi there.\n").unwrap();
    learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&audio_source_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let show = learnkit()
        .arg("session")
        .arg("show")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&show).unwrap();

    let material = json["text_material"].as_array().unwrap();
    let notes_entry = material.iter().find(|m| m["kind"] == "text").unwrap();
    assert!(notes_entry["notes_text"]
        .as_str()
        .unwrap()
        .contains("phrasal verbs"));

    let audio_entry = material.iter().find(|m| m["kind"] == "audio").unwrap();
    assert_eq!(audio_entry["transcript_segments"][0]["text"], "Hi there.");
}

#[test]
fn missing_session_reads_as_none() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    learnkit()
        .arg("session")
        .arg("show")
        .arg("does-not-exist")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure();
}
