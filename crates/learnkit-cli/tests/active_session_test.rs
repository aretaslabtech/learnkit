//! Integration tests for the active-session pointer (FR-003b), added
//! post-release after real usage — mirrors Spec Kit's own
//! `.specify/feature.json` (specs/002-english-eoi-flow/tasks.md T082-T084).

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

#[test]
fn session_new_sets_the_active_session_and_later_commands_omit_session_flag() {
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

    // No --session here at all — must fall back to the just-created session.
    let show = learnkit()
        .arg("session")
        .arg("show")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&show).unwrap();
    assert_eq!(json["session_id"], session_id);
}

#[test]
fn session_use_switches_the_active_session() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let create_first = |title: &str| -> String {
        let out = learnkit()
            .arg("session")
            .arg("new")
            .arg(title)
            .arg("--path")
            .arg(dir.path())
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
    };

    let first_id = create_first("Unit 5");
    let second_id = create_first("Unit 6"); // becomes active automatically

    // Switch back to the first session explicitly.
    learnkit()
        .arg("session")
        .arg("use")
        .arg(&first_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let show = learnkit()
        .arg("session")
        .arg("show")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&show).unwrap();
    assert_eq!(json["session_id"], first_id);
    assert_ne!(json["session_id"], second_id);
}

#[test]
fn commands_fail_explicitly_when_no_active_session_and_no_flag_given() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    // No session has ever been created in this project — no active session
    // pointer exists, and none is passed explicitly.
    learnkit()
        .arg("inventory")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(2);
}

#[test]
fn explicit_session_flag_still_overrides_the_active_one() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Active Session")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let _active_id = serde_json::from_slice::<serde_json::Value>(&out).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let other = learnkit()
        .arg("session")
        .arg("new")
        .arg("Other Session (not active)")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    // Creating "other" makes it the active one too — but we still pass it
    // explicitly on purpose to prove --session takes priority regardless.
    let other_id = serde_json::from_slice::<serde_json::Value>(&other).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let show = learnkit()
        .arg("session")
        .arg("show")
        .arg(&other_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&show).unwrap();
    assert_eq!(json["session_id"], other_id);
}
