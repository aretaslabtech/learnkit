//! Integration tests for `learnkit analyse summary/mindmap/page`, User
//! Story 2 (specs/003-analyse-consolidate-checklist/tasks.md T012-T016).

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
/// (`analyse-sample/notes.md`, deliberate gap for FR-006).
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

/// Returns the `analyse` entry of `learnkit status --session <id> --json`.
fn analyse_phase(project_root: &Path, session_id: &str) -> serde_json::Value {
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

fn checklist_item_state<'a>(analyse: &'a serde_json::Value, item_id: &str) -> Option<&'a str> {
    analyse["checklist"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item_id"] == item_id)
        .map(|i| i["state"].as_str().unwrap())
}

// --- T012: `analyse summary set` confirms `summary` with a fingerprint ---

#[test]
fn analyse_summary_set_confirms_summary_item() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(
        content_dir.path(),
        "summary.md",
        "# Resumen\n\nHoy repasamos present perfect vs past simple, con varios ejemplos.",
    );

    let out = learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["item_id"], "summary");
    assert_eq!(json["state"], "done");

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "summary"), Some("done"));

    let persisted = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .expect("summary persisted");
    assert!(!persisted.source_fingerprint.is_empty());
}

// --- T013: `analyse mindmap set` confirms `mindmap`; identical-to-summary content fails ---

#[test]
fn analyse_mindmap_set_confirms_mindmap_item() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let mindmap_file = write_content_file(
        content_dir.path(),
        "mindmap.md",
        "- Present perfect\n  - unfinished time\n- Past simple\n  - closed time",
    );

    learnkit()
        .arg("analyse")
        .arg("mindmap")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&mindmap_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "mindmap"), Some("done"));
}

#[test]
fn analyse_mindmap_set_rejects_content_identical_to_confirmed_summary() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let shared_content = "# Resumen\n\nContenido idéntico para ambos elementos.";
    let summary_file = write_content_file(content_dir.path(), "summary.md", shared_content);
    let mindmap_file = write_content_file(content_dir.path(), "mindmap.md", shared_content);

    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("analyse")
        .arg("mindmap")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&mindmap_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .failure()
        .code(10)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], false);

    let analyse = analyse_phase(project.path(), &session_id);
    assert_ne!(checklist_item_state(&analyse, "mindmap"), Some("done"));
}

// --- T014: `analyse page add` creates independent items, any order ---

#[test]
fn analyse_page_add_creates_independent_items_in_any_order() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let page1_file = write_content_file(content_dir.path(), "page1.md", "Explicación del concepto A.");
    let page2_file = write_content_file(content_dir.path(), "page2.md", "Explicación del concepto B.");
    let mindmap_file = write_content_file(content_dir.path(), "mindmap.md", "- nodo 1\n- nodo 2");

    let out1 = learnkit()
        .arg("analyse")
        .arg("page")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--concept")
        .arg("Concepto A")
        .arg("--file")
        .arg(&page1_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json1: serde_json::Value = serde_json::from_slice(&out1).unwrap();
    assert_eq!(json1["item_id"], "page-1");

    // Confirm mindmap in between the two page additions — order between
    // independent checklist elements must not matter.
    learnkit()
        .arg("analyse")
        .arg("mindmap")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&mindmap_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let out2 = learnkit()
        .arg("analyse")
        .arg("page")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--concept")
        .arg("Concepto B")
        .arg("--file")
        .arg(&page2_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json2: serde_json::Value = serde_json::from_slice(&out2).unwrap();
    assert_eq!(json2["item_id"], "page-2");

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "page-1"), Some("done"));
    assert_eq!(checklist_item_state(&analyse, "page-2"), Some("done"));
    assert_eq!(checklist_item_state(&analyse, "mindmap"), Some("done"));
}

// --- T015: `--filled-gap` is recorded distinguishably on `ClassSummary` ---

#[test]
fn analyse_summary_set_records_filled_gap_distinguishable_from_content() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(
        content_dir.path(),
        "summary.md",
        "# Resumen\n\nRepasamos present perfect vs past simple y vocabulario de viajes.",
    );

    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .arg("--filled-gap")
        .arg("present perfect of unfinished time periods:se usa cuando la acción empezó en el pasado y el periodo de tiempo sigue abierto")
        .assert()
        .success();

    let persisted = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .expect("summary persisted");
    assert_eq!(persisted.filled_gaps.len(), 1);
    assert_eq!(
        persisted.filled_gaps[0].concept,
        "present perfect of unfinished time periods"
    );
    // Distinguishable from the regular content: not silently merged in.
    assert!(!persisted
        .content
        .contains("se usa cuando la acción empezó en el pasado"));
}

#[test]
fn analyse_summary_set_rejects_malformed_filled_gap() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(content_dir.path(), "summary.md", "Resumen breve.");

    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .arg("--filled-gap")
        .arg("solo-concepto-sin-nota")
        .assert()
        .failure()
        .code(10);
}

// --- T016: repeating `analyse summary set` unchanged is a no-op; `--force` re-runs it ---

#[test]
fn analyse_summary_set_is_idempotent_and_force_reruns_it() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(content_dir.path(), "summary.md", "Resumen original.");

    let first = learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let first_json: serde_json::Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(first_json["already_done"], false);

    // Re-run with unchanged source material and an unchanged file: no-op.
    let second = learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_json: serde_json::Value = serde_json::from_slice(&second).unwrap();
    assert_eq!(second_json["already_done"], true);

    // A different file's content is ignored by the no-op path unless forced.
    let updated_file = write_content_file(content_dir.path(), "summary2.md", "Resumen actualizado.");
    let noop_with_new_file = learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&updated_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let noop_json: serde_json::Value = serde_json::from_slice(&noop_with_new_file).unwrap();
    assert_eq!(noop_json["already_done"], true);
    let persisted = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.content, "Resumen original.");

    // `--force` re-runs it, picking up the new content.
    let forced = learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&updated_file)
        .arg("--path")
        .arg(project.path())
        .arg("--force")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let forced_json: serde_json::Value = serde_json::from_slice(&forced).unwrap();
    assert_eq!(forced_json["already_done"], false);
    let persisted_after_force = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted_after_force.content, "Resumen actualizado.");
}

// --- T022: `analyse flag-pending mindmap --reason "..."` marks the item as
// `pending_user_decision`, leaving other items unaffected ---

#[test]
fn analyse_flag_pending_marks_item_as_pending_user_decision_with_reason() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    // Confirm `summary` independently — must be unaffected by flagging `mindmap`.
    let summary_file = write_content_file(
        content_dir.path(),
        "summary.md",
        "# Resumen\n\nRepaso de present perfect.",
    );
    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("no hay material suficiente para redactar el mapa mental")
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["item_id"], "mindmap");
    assert_eq!(json["state"], "pending_user_decision");
    assert_eq!(
        json["reason"],
        "no hay material suficiente para redactar el mapa mental"
    );

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(
        checklist_item_state(&analyse, "mindmap"),
        Some("pending_user_decision")
    );
    let mindmap_item = analyse["checklist"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item_id"] == "mindmap")
        .unwrap();
    assert_eq!(
        mindmap_item["pending_reason"],
        "no hay material suficiente para redactar el mapa mental"
    );

    // `summary` followed its own normal course, unaffected.
    assert_eq!(checklist_item_state(&analyse, "summary"), Some("done"));
}

#[test]
fn analyse_flag_pending_rejects_empty_reason() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("   ")
        .arg("--path")
        .arg(project.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn analyse_flag_pending_rejects_unknown_page_item() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    // Only `page-1` exists so far; `page-3` must be rejected.
    let page1_file = write_content_file(content_dir.path(), "page1.md", "Explicación del concepto A.");
    learnkit()
        .arg("analyse")
        .arg("page")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--concept")
        .arg("Concepto A")
        .arg("--file")
        .arg(&page1_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("page-3")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("falta material")
        .arg("--path")
        .arg(project.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn analyse_flag_pending_allows_first_ever_page_item() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    // No `analyse` items exist yet at all — `page-1` is a legitimate first item.
    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("page-1")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("el material no explica el concepto de la primera página")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(
        checklist_item_state(&analyse, "page-1"),
        Some("pending_user_decision")
    );
}

// --- T023: providing the missing material and re-calling `analyse mindmap
// set` clears the pending state without repeating other items' work ---

#[test]
fn analyse_set_after_flag_pending_clears_pending_state() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    // Confirm `summary` first — must survive untouched throughout.
    let summary_file = write_content_file(content_dir.path(), "summary.md", "Resumen ya confirmado.");
    learnkit()
        .arg("analyse")
        .arg("summary")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&summary_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("falta material para el mapa mental")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let analyse_before = analyse_phase(project.path(), &session_id);
    assert_eq!(
        checklist_item_state(&analyse_before, "mindmap"),
        Some("pending_user_decision")
    );

    // The missing material is now provided.
    let mindmap_file = write_content_file(
        content_dir.path(),
        "mindmap.md",
        "- Present perfect\n  - unfinished time\n- Past simple\n  - closed time",
    );
    learnkit()
        .arg("analyse")
        .arg("mindmap")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--file")
        .arg(&mindmap_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let analyse_after = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse_after, "mindmap"), Some("done"));
    let mindmap_item = analyse_after["checklist"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item_id"] == "mindmap")
        .unwrap();
    assert!(mindmap_item["pending_reason"].is_null());

    // `summary`'s already-done work was not repeated/disturbed.
    assert_eq!(checklist_item_state(&analyse_after, "summary"), Some("done"));
    let persisted_summary = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted_summary.content, "Resumen ya confirmado.");
}

// --- T024: `analyse skip mindmap --reason "..."` resolves a pending item
// auditably; a later `status` query does not show it as pending again ---

#[test]
fn analyse_skip_resolves_pending_item_and_status_does_not_re_show_pending() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("falta material para el mapa mental")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("analyse")
        .arg("skip")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("se decide omitir el mapa mental para esta sesión")
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["item_id"], "mindmap");
    assert_eq!(json["state"], "done");
    assert_eq!(json["resolution"], "skipped");
    assert_eq!(
        json["reason"],
        "se decide omitir el mapa mental para esta sesión"
    );

    // Immediately after skip.
    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "mindmap"), Some("done"));

    // A later, independent `status` query must not show it as
    // `pending_user_decision` again.
    let analyse_again = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse_again, "mindmap"), Some("done"));
    let mindmap_item = analyse_again["checklist"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item_id"] == "mindmap")
        .unwrap();
    assert!(mindmap_item["pending_reason"].is_null());
}

#[test]
fn analyse_skip_rejects_empty_reason() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("falta material")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    learnkit()
        .arg("analyse")
        .arg("skip")
        .arg("mindmap")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("")
        .arg("--path")
        .arg(project.path())
        .assert()
        .failure()
        .code(10);
}

#[test]
fn analyse_skip_can_resolve_item_never_flagged_first() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    // `summary` was never confirmed nor flagged — skip is still a valid,
    // explicit decision per spec.md (not gated on a prior flag-pending).
    learnkit()
        .arg("analyse")
        .arg("skip")
        .arg("summary")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("se omite el resumen para esta sesión")
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "summary"), Some("done"));
}
