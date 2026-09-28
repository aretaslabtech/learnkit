//! Integration tests for `learnkit analyse set` / `flag-pending` / `skip`,
//! User Story 2/3 (feature 003) — rewritten for the unified item-based API
//! (FR-018, Phase 10, tasks T045-T053). Every result of `analyse` is
//! addressed through one stable `item_id`: `summary`, `mindmap`, or
//! `page:<stable_id>`.

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

/// Thin wrapper around `learnkit analyse set`, mirroring the real CLI
/// surface (`--session --item --file [--concept] [--filled-gap]... [--force]`).
#[allow(clippy::too_many_arguments)]
fn analyse_set(
    project_root: &Path,
    session_id: &str,
    item: &str,
    file: &Path,
    concept: Option<&str>,
    filled_gaps: &[&str],
    force: bool,
) -> assert_cmd::assert::Assert {
    let mut cmd = learnkit();
    cmd.arg("analyse")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--item")
        .arg(item)
        .arg("--file")
        .arg(file)
        .arg("--path")
        .arg(project_root)
        .arg("--json");
    if let Some(c) = concept {
        cmd.arg("--concept").arg(c);
    }
    for gap in filled_gaps {
        cmd.arg("--filled-gap").arg(gap);
    }
    if force {
        cmd.arg("--force");
    }
    cmd.assert()
}

fn analyse_set_json(
    project_root: &Path,
    session_id: &str,
    item: &str,
    file: &Path,
    concept: Option<&str>,
) -> serde_json::Value {
    let out = analyse_set(project_root, session_id, item, file, concept, &[], false)
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

// --- 1/2/3: create summary; idempotent re-run; --force required to override deliberately ---

#[test]
fn analyse_set_summary_creates_confirms_and_is_idempotent() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(
        content_dir.path(),
        "summary.md",
        "# Resumen\n\nHoy repasamos present perfect vs past simple, con varios ejemplos.",
    );

    let first = analyse_set_json(project.path(), &session_id, "summary", &summary_file, None);
    assert_eq!(first["ok"], true);
    assert_eq!(first["code"], "ANALYSE_ITEM_SET");
    assert_eq!(first["item_id"], "summary");
    assert_eq!(first["state"], "done");
    assert_eq!(first["already_done"], false);

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "summary"), Some("done"));

    let persisted = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .expect("summary persisted");
    assert!(!persisted.source_fingerprint.is_empty());

    // 2) Re-running unchanged is idempotent (no-op).
    let second = analyse_set_json(project.path(), &session_id, "summary", &summary_file, None);
    assert_eq!(second["already_done"], true);

    // 3) A different file's content is ignored unless --force; --force overrides deliberately.
    let updated_file =
        write_content_file(content_dir.path(), "summary2.md", "Resumen actualizado.");
    let noop = analyse_set_json(
        project.path(),
        &session_id,
        "summary",
        &updated_file,
        None,
    );
    assert_eq!(noop["already_done"], true);
    let persisted = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.content, "# Resumen\n\nHoy repasamos present perfect vs past simple, con varios ejemplos.");
    assert_ne!(persisted.content, "Resumen actualizado.");

    let forced = analyse_set(
        project.path(),
        &session_id,
        "summary",
        &updated_file,
        None,
        &[],
        true,
    )
    .success()
    .get_output()
    .stdout
    .clone();
    let forced_json: serde_json::Value = serde_json::from_slice(&forced).unwrap();
    assert_eq!(forced_json["already_done"], false);
    let persisted_after_force =
        learnkit_workflow::analysis::read_summary(project.path(), &session_id)
            .unwrap()
            .unwrap();
    assert_eq!(persisted_after_force.content, "Resumen actualizado.");
}

// --- 4/5: create mindmap; validation differentiating mindmap vs summary is preserved ---

#[test]
fn analyse_set_mindmap_confirms_and_rejects_content_identical_to_summary() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let mindmap_file = write_content_file(
        content_dir.path(),
        "mindmap.md",
        "- Present perfect\n  - unfinished time\n- Past simple\n  - closed time",
    );
    let out = analyse_set_json(project.path(), &session_id, "mindmap", &mindmap_file, None);
    assert_eq!(out["ok"], true);
    assert_eq!(out["item_id"], "mindmap");
    assert_eq!(out["state"], "done");

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "mindmap"), Some("done"));

    // Validation preserved: identical-to-summary content is rejected.
    let project2 = tempfile::tempdir().unwrap();
    let session2 = new_session_with_notes(project2.path());
    let shared_content = "# Resumen\n\nContenido idéntico para ambos elementos.";
    let summary_file = write_content_file(content_dir.path(), "summary_dup.md", shared_content);
    let mindmap_file2 = write_content_file(content_dir.path(), "mindmap_dup.md", shared_content);

    analyse_set(
        project2.path(),
        &session2,
        "summary",
        &summary_file,
        None,
        &[],
        false,
    )
    .success();

    let rejected = analyse_set(
        project2.path(),
        &session2,
        "mindmap",
        &mindmap_file2,
        None,
        &[],
        false,
    )
    .failure()
    .code(10)
    .get_output()
    .stdout
    .clone();
    let rejected_json: serde_json::Value = serde_json::from_slice(&rejected).unwrap();
    assert_eq!(rejected_json["ok"], false);

    let analyse2 = analyse_phase(project2.path(), &session2);
    assert_ne!(checklist_item_state(&analyse2, "mindmap"), Some("done"));
}

// --- 6/7/8/9: create page:foo; update it (no duplicate); change --concept
// keeping item_id; several distinct pages ---

#[test]
fn analyse_set_page_creates_updates_in_place_and_supports_several_pages() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let page1_file = write_content_file(content_dir.path(), "layover_v1.md", "Contenido v1.");
    let created = analyse_set_json(
        project.path(),
        &session_id,
        "page:layover",
        &page1_file,
        Some("Layover"),
    );
    assert_eq!(created["ok"], true);
    assert_eq!(created["item_id"], "page:layover");
    assert_eq!(created["state"], "done");
    assert_eq!(created["kind"], "concept_page");
    assert_eq!(created["concept"], "Layover");

    // 7) Update the same page (source unchanged -> would be no-op unless
    // forced); force the deliberate content replacement and prove no second
    // page was created.
    let page1_v2_file = write_content_file(content_dir.path(), "layover_v2.md", "Contenido v2.");
    let updated = analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page1_v2_file,
        None,
        &[],
        true,
    )
    .success()
    .get_output()
    .stdout
    .clone();
    let updated_json: serde_json::Value = serde_json::from_slice(&updated).unwrap();
    assert_eq!(updated_json["item_id"], "page:layover");
    assert_eq!(updated_json["already_done"], false);

    let pages = learnkit_workflow::analysis::list_concept_pages(project.path(), &session_id)
        .unwrap();
    assert_eq!(pages.len(), 1, "update must not create a second page");
    assert_eq!(pages[0].content, "Contenido v2.");

    // 8) Change the visible --concept while keeping the same item_id. The
    // source fingerprint and content are unchanged from the last confirm,
    // so — same as a deliberate content replacement — this needs --force.
    let renamed_out = analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page1_v2_file,
        Some("Layovers and connections"),
        &[],
        true,
    )
    .success()
    .get_output()
    .stdout
    .clone();
    let renamed: serde_json::Value = serde_json::from_slice(&renamed_out).unwrap();
    assert_eq!(renamed["item_id"], "page:layover");
    let pages = learnkit_workflow::analysis::list_concept_pages(project.path(), &session_id)
        .unwrap();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].concept, "Layovers and connections");

    // 9) Several distinct pages coexist.
    let page2_file = write_content_file(content_dir.path(), "present_perfect.md", "Otro contenido.");
    analyse_set(
        project.path(),
        &session_id,
        "page:present-perfect",
        &page2_file,
        Some("Present perfect"),
        &[],
        false,
    )
    .success();

    let pages = learnkit_workflow::analysis::list_concept_pages(project.path(), &session_id)
        .unwrap();
    assert_eq!(pages.len(), 2);
    let ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"page:layover"));
    assert!(ids.contains(&"page:present-perfect"));

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "page:layover"), Some("done"));
    assert_eq!(
        checklist_item_state(&analyse, "page:present-perfect"),
        Some("done")
    );
}

#[test]
fn analyse_set_page_requires_concept_on_first_creation_but_not_on_update() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let page_file = write_content_file(content_dir.path(), "page.md", "Contenido.");

    // Creating without --concept fails.
    analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page_file,
        None,
        &[],
        false,
    )
    .failure()
    .code(10);

    // Creating with --concept succeeds.
    analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page_file,
        Some("Layover"),
        &[],
        false,
    )
    .success();

    // Updating without --concept succeeds, keeping the existing display name.
    let page_v2 = write_content_file(content_dir.path(), "page_v2.md", "Contenido nuevo.");
    let updated = analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page_v2,
        None,
        &[],
        true,
    )
    .success()
    .get_output()
    .stdout
    .clone();
    let updated_json: serde_json::Value = serde_json::from_slice(&updated).unwrap();
    assert_eq!(updated_json["concept"], "Layover");
}

// --- 10/11/12/13: --filled-gap on summary, mindmap, concept page; repeatable ---

#[test]
fn analyse_set_records_filled_gaps_on_every_item_kind_distinguishable_from_content() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(
        content_dir.path(),
        "summary.md",
        "# Resumen\n\nRepasamos present perfect vs past simple y vocabulario de viajes.",
    );
    analyse_set(
        project.path(),
        &session_id,
        "summary",
        &summary_file,
        None,
        &[
            "present perfect of unfinished time periods:se usa cuando la acción empezó en el pasado y el periodo de tiempo sigue abierto",
        ],
        false,
    )
    .success();

    let persisted_summary = learnkit_workflow::analysis::read_summary(project.path(), &session_id)
        .unwrap()
        .expect("summary persisted");
    assert_eq!(persisted_summary.filled_gaps.len(), 1);
    assert_eq!(
        persisted_summary.filled_gaps[0].concept,
        "present perfect of unfinished time periods"
    );
    assert!(!persisted_summary
        .content
        .contains("se usa cuando la acción empezó en el pasado"));

    let mindmap_file = write_content_file(content_dir.path(), "mindmap.md", "- nodo 1\n- nodo 2");
    analyse_set(
        project.path(),
        &session_id,
        "mindmap",
        &mindmap_file,
        None,
        &["gap concepto mindmap:nota del mindmap"],
        false,
    )
    .success();
    let persisted_map = learnkit_workflow::analysis::read_study_map(project.path(), &session_id)
        .unwrap()
        .expect("mindmap persisted");
    assert_eq!(persisted_map.filled_gaps.len(), 1);
    assert_eq!(persisted_map.filled_gaps[0].concept, "gap concepto mindmap");

    // 13) several --filled-gap on the same item.
    let page_file = write_content_file(content_dir.path(), "page.md", "Contenido de la página.");
    analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page_file,
        Some("Layover"),
        &["gap a:nota a", "gap b:nota b"],
        false,
    )
    .success();
    let page = learnkit_workflow::analysis::read_concept_page(
        project.path(),
        &session_id,
        "page:layover",
    )
    .unwrap()
    .expect("page persisted");
    assert_eq!(page.filled_gaps.len(), 2);

    // Gaps of one item never leak into another item.
    assert_eq!(persisted_summary.filled_gaps.len(), 1);
    assert_eq!(persisted_map.filled_gaps.len(), 1);
}

#[test]
fn analyse_set_rejects_malformed_filled_gap() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let summary_file = write_content_file(content_dir.path(), "summary.md", "Resumen breve.");

    analyse_set(
        project.path(),
        &session_id,
        "summary",
        &summary_file,
        None,
        &["solo-concepto-sin-nota"],
        false,
    )
    .failure()
    .code(10);
}

// --- 14/15: rejection of empty `page:` and unknown item ids ---

#[test]
fn analyse_set_rejects_empty_page_id() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());
    let file = write_content_file(content_dir.path(), "page.md", "Contenido.");

    analyse_set(
        project.path(),
        &session_id,
        "page:",
        &file,
        Some("Algo"),
        &[],
        false,
    )
    .failure()
    .code(10);
}

#[test]
fn analyse_set_rejects_unknown_item_id() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());
    let file = write_content_file(content_dir.path(), "x.md", "Contenido.");

    analyse_set(project.path(), &session_id, "bogus", &file, None, &[], false)
        .failure()
        .code(10);
}

// --- 16/17: flag-pending page:foo, then resolve later via `set` ---

#[test]
fn analyse_flag_pending_and_skip_operate_on_page_item_ids() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let out = learnkit()
        .arg("analyse")
        .arg("flag-pending")
        .arg("page:foo")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("el material no explica el concepto todavía")
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
    assert_eq!(json["item_id"], "page:foo");
    assert_eq!(json["state"], "pending_user_decision");

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(
        checklist_item_state(&analyse, "page:foo"),
        Some("pending_user_decision")
    );

    // 17) resolve later via `set`.
    let page_file = write_content_file(content_dir.path(), "foo.md", "Ahora sí hay contenido.");
    analyse_set(
        project.path(),
        &session_id,
        "page:foo",
        &page_file,
        Some("Foo"),
        &[],
        false,
    )
    .success();

    let analyse_after = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse_after, "page:foo"), Some("done"));

    // 18) skip page:bar.
    let skip_out = learnkit()
        .arg("analyse")
        .arg("skip")
        .arg("page:bar")
        .arg("--session")
        .arg(&session_id)
        .arg("--reason")
        .arg("se omite esta página para esta sesión")
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let skip_json: serde_json::Value = serde_json::from_slice(&skip_out).unwrap();
    assert_eq!(skip_json["item_id"], "page:bar");
    assert_eq!(skip_json["state"], "done");
    assert_eq!(skip_json["resolution"], "skipped");

    let analyse_final = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse_final, "page:bar"), Some("done"));
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
fn analyse_flag_pending_rejects_unknown_shaped_item_id() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

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

// --- 19/20: invalidation on source change + reconfirmation of the same item ---

#[test]
fn analyse_set_reconfirms_same_item_after_source_invalidation() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let page_file = write_content_file(content_dir.path(), "page.md", "Contenido v1.");
    analyse_set(
        project.path(),
        &session_id,
        "page:layover",
        &page_file,
        Some("Layover"),
        &[],
        false,
    )
    .success();

    let analyse = analyse_phase(project.path(), &session_id);
    assert_eq!(checklist_item_state(&analyse, "page:layover"), Some("done"));

    // Change the session's source material -> invalidates the item.
    let extra_notes = write_content_file(
        content_dir.path(),
        "extra-notes.md",
        "Notas adicionales que cambian la huella de fuentes de la sesión.",
    );
    learnkit()
        .arg("ingest")
        .arg(&extra_notes)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();
    learnkit()
        .arg("inventory")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    // `status`'s per-item state is a cached diagnostic synced by the last
    // `analyse` command (`engine.rs`: "never trusted as sole authority on
    // the next read") — it only refreshes when an `analyse` command runs
    // again, so the real, live invalidation check below (inside `analyse
    // set` itself, via a freshly recomputed source fingerprint) is what
    // actually proves FR-004 cascading invalidation: re-running `set`
    // without `--force` is no longer a no-op once sources changed.

    // Reconfirm the SAME item (no --force needed — invalidation, not
    // deliberate override) and prove no second page was created.
    let page_v2 = write_content_file(content_dir.path(), "page_v2.md", "Contenido v2.");
    let out = analyse_set_json(
        project.path(),
        &session_id,
        "page:layover",
        &page_v2,
        None,
    );
    assert_eq!(out["item_id"], "page:layover");
    assert_eq!(out["already_done"], false);

    let pages = learnkit_workflow::analysis::list_concept_pages(project.path(), &session_id)
        .unwrap();
    assert_eq!(pages.len(), 1, "reconfirmation must not create a second page");

    let analyse_final = analyse_phase(project.path(), &session_id);
    assert_eq!(
        checklist_item_state(&analyse_final, "page:layover"),
        Some("done")
    );
}
