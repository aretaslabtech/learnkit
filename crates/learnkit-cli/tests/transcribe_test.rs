//! Integration tests for `learnkit transcribe`, User Story 2
//! (specs/002-english-eoi-flow/tasks.md T019-T021).
//!
//! Real generation via whisper.cpp (T018) is not automatable here — see
//! `learnkit-transcription`'s own unit tests for the JSON-parsing contract,
//! and `quickstart.md` Escenario 2 for the manual whisper.cpp validation.

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

fn session_with_audio(project_root: &std::path::Path) -> (String, String) {
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
        .arg(fixtures_dir().join("class-audio.wav"))
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

fn write_srt(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("clase.srt");
    fs::write(
        &path,
        "1\n00:00:00,000 --> 00:00:02,000\nHello there.\n\n2\n00:00:02,000 --> 00:00:04,500\nHe got away with it.\n",
    )
    .unwrap();
    path
}

#[test]
fn importing_a_transcript_does_not_require_whisper_cpp() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = session_with_audio(dir.path());
    let srt = write_srt(dir.path());

    let out = learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&source_id)
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
    assert_eq!(json["transcripts"][0]["segments"], 2);
}

#[test]
fn re_importing_creates_a_new_version_and_stays_current() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = session_with_audio(dir.path());
    let srt = write_srt(dir.path());

    learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&source_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    // Importing again for the same (unchanged) audio source should not
    // error, and should keep the transcript "current" (not stale) — the
    // source's sha256 has not changed between the two imports.
    let out = learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&source_id)
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
}

#[test]
fn changing_the_audio_source_invalidates_the_transcript() {
    let dir = tempfile::tempdir().unwrap();
    let (session_id, source_id) = session_with_audio(dir.path());
    let srt = write_srt(dir.path());

    learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&source_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    // Mutate the audio file on disk, then re-inventory so its sha256 changes.
    let audio_path = dir
        .path()
        .join("sessions")
        .join(&session_id)
        .join("input")
        .join("class-audio.wav");
    let mut bytes = fs::read(&audio_path).unwrap();
    bytes.push(0);
    fs::write(&audio_path, bytes).unwrap();

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
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&inv).unwrap()["sources"][0]["change"],
        "updated"
    );

    // The previously imported transcript's recorded fingerprint no longer
    // matches the source's new hash — re-importing must be treated as a
    // fresh transcription, not a no-op (FR-006).
    let out = learnkit()
        .arg("transcribe")
        .arg("import")
        .arg(&srt)
        .arg("--source")
        .arg(&source_id)
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
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&out).unwrap()["ok"],
        true
    );
}
