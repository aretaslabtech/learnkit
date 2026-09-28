//! Integration tests for `learnkit export study-guide` (ODD, tracked at
//! `odd/tasks/export-study-guide.md`, T3-T4). Bootstrap follows
//! `analyse_test.rs`'s pattern: init, session new, ingest, inventory, then
//! confirm `analyse` content via the real CLI with `--file`.

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

fn set_summary(project_root: &Path, session_id: &str, content_dir: &Path, content: &str) {
    let file = write_content_file(content_dir, "summary.md", content);
    learnkit()
        .arg("analyse")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--item")
        .arg("summary")
        .arg("--file")
        .arg(&file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn set_mindmap(project_root: &Path, session_id: &str, content_dir: &Path, content: &str) {
    let file = write_content_file(content_dir, "mindmap.md", content);
    learnkit()
        .arg("analyse")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--item")
        .arg("mindmap")
        .arg("--file")
        .arg(&file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

fn add_page(
    project_root: &Path,
    session_id: &str,
    content_dir: &Path,
    item_id: &str,
    name: &str,
    concept: &str,
    content: &str,
) {
    let file = write_content_file(content_dir, name, content);
    learnkit()
        .arg("analyse")
        .arg("set")
        .arg("--session")
        .arg(session_id)
        .arg("--item")
        .arg(item_id)
        .arg("--concept")
        .arg(concept)
        .arg("--file")
        .arg(&file)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();
}

// --- T3: summary + mindmap + 2 pages exports a .md with all four pieces, in order ---

#[test]
fn export_study_guide_includes_all_content_in_order() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    set_summary(
        project.path(),
        &session_id,
        content_dir.path(),
        "Contenido del resumen de la clase.",
    );
    set_mindmap(
        project.path(),
        &session_id,
        content_dir.path(),
        "- nodo del mapa mental",
    );
    add_page(
        project.path(),
        &session_id,
        content_dir.path(),
        "page:concept-a",
        "page1.md",
        "Concepto A",
        "Contenido de la página del concepto A.",
    );
    add_page(
        project.path(),
        &session_id,
        content_dir.path(),
        "page:concept-b",
        "page2.md",
        "Concepto B",
        "Contenido de la página del concepto B.",
    );

    let out_file = project.path().join("guide.md");
    learnkit()
        .arg("export")
        .arg("study-guide")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let doc = std::fs::read_to_string(&out_file).unwrap();

    assert!(doc.contains("# Unit 5"));
    assert!(doc.contains("## Resumen"));
    assert!(doc.contains("Contenido del resumen de la clase."));
    assert!(doc.contains("## Mapa mental"));
    assert!(doc.contains("- nodo del mapa mental"));
    assert!(doc.contains("## Concepto A"));
    assert!(doc.contains("Contenido de la página del concepto A."));
    assert!(doc.contains("## Concepto B"));
    assert!(doc.contains("Contenido de la página del concepto B."));

    let title_idx = doc.find("# Unit 5").unwrap();
    let summary_idx = doc.find("## Resumen").unwrap();
    let mindmap_idx = doc.find("## Mapa mental").unwrap();
    let page_a_idx = doc.find("## Concepto A").unwrap();
    let page_b_idx = doc.find("## Concepto B").unwrap();

    assert!(title_idx < summary_idx);
    assert!(summary_idx < mindmap_idx);
    assert!(mindmap_idx < page_a_idx);
    assert!(page_a_idx < page_b_idx);
}

// --- T4: no confirmed summary fails explicitly, no file written ---

#[test]
fn export_study_guide_fails_without_summary_and_writes_no_file() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    let out_file = project.path().join("guide.md");
    learnkit()
        .arg("export")
        .arg("study-guide")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .failure();

    assert!(!out_file.exists());
}

// --- T4: summary only (no mindmap/pages) exports successfully, omitting those sections ---

#[test]
fn export_study_guide_omits_missing_optional_sections() {
    let project = tempfile::tempdir().unwrap();
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session_with_notes(project.path());

    set_summary(
        project.path(),
        &session_id,
        content_dir.path(),
        "Solo hay resumen para esta sesión.",
    );

    let out_file = project.path().join("guide.md");
    learnkit()
        .arg("export")
        .arg("study-guide")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_file)
        .arg("--path")
        .arg(project.path())
        .assert()
        .success();

    let doc = std::fs::read_to_string(&out_file).unwrap();
    assert!(doc.contains("## Resumen"));
    assert!(doc.contains("Solo hay resumen para esta sesión."));
    assert!(!doc.contains("Mapa mental"));
    assert!(!doc.contains("## Concepto"));
}
