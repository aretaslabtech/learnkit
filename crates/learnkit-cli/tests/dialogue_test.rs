//! Integration tests for `learnkit learn dialogue set/remove` —
//! `odd/tasks/language-study-pack.md` T2.

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
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
fn learn_dialogue_set_persists_a_dialogue_with_at_least_two_lines() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    let out = learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: Hello")
        .arg("--line")
        .arg("B: Hi")
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
    assert_eq!(json["dialogue_id"], "dlg-1");
    assert_eq!(json["session_id"], session_id);

    let session_root = dir.path().join("sessions").join(&session_id);
    let dialogues =
        learnkit_profile::language::dialogue::load_all(&session_root).unwrap();
    assert_eq!(dialogues.len(), 1);
    assert_eq!(dialogues[0].lines.len(), 2);
    assert_eq!(dialogues[0].lines[0].speaker, "A");
    assert_eq!(dialogues[0].lines[0].text, "Hello");
    assert_eq!(
        dialogues[0].origin,
        learnkit_profile::language::dialogue::DialogueOrigin::FromClass
    );
}

#[test]
fn learn_dialogue_set_is_an_upsert_keyed_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: Hello")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("added")
        .arg("--line")
        .arg("A: Good morning")
        .arg("--line")
        .arg("B: Good morning to you too")
        .arg("--note")
        .arg("drills greetings")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let session_root = dir.path().join("sessions").join(&session_id);
    let dialogues =
        learnkit_profile::language::dialogue::load_all(&session_root).unwrap();
    assert_eq!(dialogues.len(), 1);
    assert_eq!(
        dialogues[0].origin,
        learnkit_profile::language::dialogue::DialogueOrigin::AddedForConsolidation
    );
    assert_eq!(dialogues[0].lines[0].text, "Good morning");
    assert_eq!(dialogues[0].note.as_deref(), Some("drills greetings"));
}

#[test]
fn learn_dialogue_set_rejects_a_single_line() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: Hello")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn learn_dialogue_set_rejects_a_line_without_the_speaker_separator() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("Hello there")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn learn_dialogue_set_rejects_an_unknown_origin() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("bogus")
        .arg("--line")
        .arg("A: Hello")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn learn_dialogue_remove_deletes_and_reports_missing_as_false() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: Hello")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("remove")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["removed"], true);

    let session_root = dir.path().join("sessions").join(&session_id);
    assert!(learnkit_profile::language::dialogue::load_all(&session_root)
        .unwrap()
        .is_empty());

    let second_out = learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("remove")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_json: serde_json::Value = serde_json::from_slice(&second_out).unwrap();
    assert_eq!(second_json["removed"], false);
}

/// Encoding guard: mojibake in a dialogue line's text must be rejected.
#[test]
fn learn_dialogue_set_rejects_mojibake_in_a_line() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("dialogue")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("dlg-1")
        .arg("--origin")
        .arg("class")
        .arg("--line")
        .arg("A: clasificaciÃ³n")
        .arg("--line")
        .arg("B: Hi")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);
}
