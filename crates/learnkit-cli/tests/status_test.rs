//! Integration tests for `learnkit status`, covering User Story 3
//! (see specs/001-cli-init-agent-hooks/tasks.md T030-T031).

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

#[test]
fn status_on_uninitialized_folder_reports_not_initialized() {
    let dir = tempfile::tempdir().unwrap();

    let output = learnkit()
        .arg("status")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["code"], "PROJECT_NOT_INITIALIZED");
}

#[test]
fn status_on_valid_project_reports_profile_and_agents_from_disk() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--profile")
        .arg("geography")
        .arg("--agents")
        .arg("codex,claude")
        .assert()
        .success();

    let output = learnkit()
        .arg("status")
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
    assert_eq!(json["code"], "PROJECT_READY");
    assert_eq!(json["profile_id"], "geography");
    assert!(json["shell_preference"] == "sh" || json["shell_preference"] == "ps");

    let mut agents: Vec<String> = json["installed_agents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    agents.sort();
    assert_eq!(agents, vec!["claude".to_string(), "codex".to_string()]);
}
