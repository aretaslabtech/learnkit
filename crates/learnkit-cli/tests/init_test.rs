//! Integration tests for `learnkit init`, covering User Story 1
//! (see specs/001-cli-init-agent-hooks/tasks.md T014-T018b).
//!
//! `assert_cmd` runs the binary with piped (non-TTY) stdio, so every case
//! here exercises the non-interactive fallback path (generic + claude by
//! default) rather than the interactive menu — see quickstart.md Escenario 1.

use assert_cmd::Command;
use std::fs;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

#[test]
fn init_on_empty_folder_creates_scaffold_with_default_agent() {
    let dir = tempfile::tempdir().unwrap();

    learnkit().arg("init").arg(dir.path()).assert().success();

    assert!(dir.path().join("learnkit.toml").exists());
    assert!(dir.path().join(".learnkit/workflow.toml").exists());
    assert!(dir
        .path()
        .join(".learnkit/profiles/generic/profile.toml")
        .exists());
    // Default agent per FR-004 fallback is `claude`.
    assert!(dir.path().join("CLAUDE.md").exists());
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();

    learnkit().arg("init").arg(dir.path()).assert().success();
    let first_config = fs::read_to_string(dir.path().join("learnkit.toml")).unwrap();

    learnkit().arg("init").arg(dir.path()).assert().success();
    let second_config = fs::read_to_string(dir.path().join("learnkit.toml")).unwrap();

    assert_eq!(first_config, second_config);
}

#[test]
fn init_rejects_unsupported_profile_without_partial_structure() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--profile")
        .arg("no-existe")
        .assert()
        .failure()
        .code(2);

    assert!(!dir.path().join("learnkit.toml").exists());
    assert!(!dir.path().join(".learnkit").exists());
}

#[test]
fn init_does_not_touch_unrelated_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "otro proyecto\n").unwrap();

    learnkit().arg("init").arg(dir.path()).assert().success();

    assert_eq!(
        fs::read_to_string(dir.path().join("README.md")).unwrap(),
        "otro proyecto\n"
    );
}

#[test]
fn init_with_explicit_profile_and_agents_installs_both_agents() {
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

    assert!(dir
        .path()
        .join(".learnkit/profiles/geography/profile.toml")
        .exists());
    assert!(dir.path().join("AGENTS.md").exists());
    assert!(dir.path().join("CLAUDE.md").exists());
}

#[test]
fn init_rejects_explicit_empty_agent_list() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--agents")
        .arg("")
        .assert()
        .failure()
        .code(2);

    assert!(!dir.path().join("learnkit.toml").exists());
}

#[test]
fn init_rejects_unsupported_agent_without_installing_valid_ones() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--agents")
        .arg("claude,agente-inventado")
        .assert()
        .failure()
        .code(2);

    assert!(!dir.path().join("learnkit.toml").exists());
    assert!(!dir.path().join("CLAUDE.md").exists());
}

#[test]
fn init_saves_auto_detected_shell_preference_when_not_interactive() {
    let dir = tempfile::tempdir().unwrap();

    learnkit().arg("init").arg(dir.path()).assert().success();

    let config = fs::read_to_string(dir.path().join("learnkit.toml")).unwrap();
    assert!(
        config.contains("shell_preference = \"sh\"")
            || config.contains("shell_preference = \"ps\""),
        "expected a detected shell_preference, got: {config}"
    );
}

#[test]
fn init_saves_explicit_shell_preference() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--shell")
        .arg("ps")
        .assert()
        .success();

    let config = fs::read_to_string(dir.path().join("learnkit.toml")).unwrap();
    assert!(config.contains("shell_preference = \"ps\""));
}

#[test]
fn init_rejects_unsupported_shell() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("init")
        .arg(dir.path())
        .arg("--shell")
        .arg("fish")
        .assert()
        .failure()
        .code(2);

    assert!(!dir.path().join("learnkit.toml").exists());
}
