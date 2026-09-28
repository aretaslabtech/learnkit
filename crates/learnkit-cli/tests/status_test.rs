//! Integration tests for `learnkit status`, covering User Story 3
//! (see specs/001-cli-init-agent-hooks/tasks.md T030-T031) and, below, User
//! Story 1 of feature 003 (per-item checklist visibility,
//! specs/003-analyse-consolidate-checklist/tasks.md T007-T010).

use assert_cmd::Command;
use learnkit_workflow::engine::{
    read_manifest, write_checklist_item, write_manifest, ChecklistItemManifest,
    ChecklistItemState,
};

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

// --- User Story 1 (feature 003): per-item checklist visibility ---

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

fn session_root(project_root: &std::path::Path, session_id: &str) -> std::path::PathBuf {
    project_root.join("sessions").join(session_id)
}

fn checklist_item(
    item_id: &str,
    state: ChecklistItemState,
    input_fingerprint: &str,
) -> ChecklistItemManifest {
    ChecklistItemManifest {
        item_id: item_id.to_string(),
        state,
        input_fingerprint: input_fingerprint.to_string(),
        pending_reason: None,
        resolution: None,
    }
}

/// Runs `learnkit status --session <id> --json` and returns the `analyse`
/// entry of the `phases` array.
fn analyse_phase_json(
    project_root: &std::path::Path,
    session_id: &str,
) -> serde_json::Value {
    let out = learnkit()
        .arg("status")
        .arg("--path")
        .arg(project_root)
        .arg("--session")
        .arg(session_id)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    json["phases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["phase"] == "analyse")
        .cloned()
        .expect("analyse phase present in status output")
}

#[test]
fn status_reports_checklist_items_separately_not_as_a_single_phase_state() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let root = session_root(dir.path(), &session_id);

    // Both items' input_fingerprint matches the phase manifest's own
    // output_fingerprint ("" by default from write_checklist_item's
    // scaffold), so `summary` recomputes as still current (`done`) while
    // `mindmap` was simply never confirmed (`pending`).
    write_checklist_item(
        &root,
        "analyse",
        checklist_item("summary", ChecklistItemState::Done, ""),
    )
    .unwrap();
    write_checklist_item(
        &root,
        "analyse",
        checklist_item("mindmap", ChecklistItemState::Pending, ""),
    )
    .unwrap();

    let analyse = analyse_phase_json(dir.path(), &session_id);
    let items = analyse["checklist"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    let summary = items.iter().find(|i| i["item_id"] == "summary").unwrap();
    let mindmap = items.iter().find(|i| i["item_id"] == "mindmap").unwrap();
    assert_eq!(summary["state"], "done");
    assert_eq!(mindmap["state"], "pending");
}

#[test]
fn status_reports_phase_and_every_item_done_when_all_items_are_done() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let root = session_root(dir.path(), &session_id);

    // input_fingerprint matches the phase manifest's own output_fingerprint
    // ("" by default from write_checklist_item's scaffold), so every item
    // recomputes as still current.
    write_checklist_item(
        &root,
        "analyse",
        checklist_item("summary", ChecklistItemState::Done, ""),
    )
    .unwrap();
    write_checklist_item(
        &root,
        "analyse",
        checklist_item("mindmap", ChecklistItemState::Done, ""),
    )
    .unwrap();

    let analyse = analyse_phase_json(dir.path(), &session_id);
    let items = analyse["checklist"].as_array().unwrap();
    assert!(items.iter().all(|i| i["state"] == "done"));
}

#[test]
fn status_reports_stale_fingerprint_item_as_no_longer_done() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());
    let root = session_root(dir.path(), &session_id);

    write_checklist_item(
        &root,
        "analyse",
        checklist_item("summary", ChecklistItemState::Done, "fp-old"),
    )
    .unwrap();

    // Simulate the source material changing after `summary` was already
    // confirmed: the phase manifest's own output_fingerprint (used by
    // `status` as the "current" fingerprint, per FR-004) now differs from
    // what the item was confirmed against.
    let manifest_path = root.join("validation").join("analyse.json");
    let mut manifest = read_manifest(&manifest_path).unwrap().unwrap();
    manifest.output_fingerprint = "fp-new".to_string();
    write_manifest(&manifest_path, &manifest).unwrap();

    let analyse = analyse_phase_json(dir.path(), &session_id);
    let items = analyse["checklist"].as_array().unwrap();
    let summary = items.iter().find(|i| i["item_id"] == "summary").unwrap();
    assert_ne!(summary["state"], "done");
}

#[test]
fn status_leaves_phases_without_a_checklist_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    let out = learnkit()
        .arg("status")
        .arg("--path")
        .arg(dir.path())
        .arg("--session")
        .arg(&session_id)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let phases = json["phases"].as_array().unwrap();

    let inventory = phases
        .iter()
        .find(|p| p["phase"] == "inventory")
        .expect("inventory phase present");
    // Regression (FR-003): a phase without a checklist keeps reporting a
    // single aggregate `state` string and has no `checklist` key at all —
    // not `null`, absent entirely, matching the feature 002 output shape.
    assert!(inventory.get("checklist").is_none());
    assert!(inventory["state"].is_string());
}
