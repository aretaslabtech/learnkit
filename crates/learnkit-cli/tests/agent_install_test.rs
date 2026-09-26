//! Integration tests for `learnkit agent install`, covering User Story 2
//! (see specs/001-cli-init-agent-hooks/tasks.md T024-T027).

use assert_cmd::Command;
use std::fs;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn init_project_without_agents(dir: &std::path::Path) {
    learnkit()
        .arg("init")
        .arg(dir)
        .arg("--profile")
        .arg("generic")
        .arg("--agents")
        .arg("codex")
        .assert()
        .success();
    // Remove the codex integration the fallback installed, to start this
    // suite from "no agents installed" as the story describes.
    let _ = fs::remove_file(dir.join("AGENTS.md"));
    let _ = fs::remove_dir_all(dir.join(".agents"));
}

#[test]
fn agent_install_on_existing_project_installs_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    init_project_without_agents(dir.path());

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("CLAUDE.md").exists());
    assert!(dir
        .path()
        .join(".claude/skills/learnkit-session/SKILL.md")
        .exists());
}

#[test]
fn agent_install_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    init_project_without_agents(dir.path());

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    let first = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
    let second = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();

    assert_eq!(first, second);
}

#[test]
fn agent_install_never_overwrites_manually_modified_files() {
    let dir = tempfile::tempdir().unwrap();
    init_project_without_agents(dir.path());

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    fs::write(dir.path().join("CLAUDE.md"), "notas personales\n").unwrap();

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(20);

    assert_eq!(
        fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap(),
        "notas personales\n"
    );
}

#[test]
fn agent_install_rejects_unsupported_agent() {
    let dir = tempfile::tempdir().unwrap();
    init_project_without_agents(dir.path());

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("agente-inventado")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(2);
}

#[test]
fn agent_install_rejects_uninitialized_project() {
    let dir = tempfile::tempdir().unwrap();

    learnkit()
        .arg("agent")
        .arg("install")
        .arg("claude")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(2);

    assert!(!dir.path().join("CLAUDE.md").exists());
}
