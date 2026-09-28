//! Integration tests for `learnkit assessment build`, `export exam`, and
//! `learnkit attempt import`, User Story 6 (tasks.md T059-T061).

use assert_cmd::Command;

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

fn session_with_two_vocabulary_items(project_root: &std::path::Path) -> String {
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

    for (lemma, sense) in [
        ("get away with", "hacer algo malo sin castigo"),
        ("whiteboard", "pizarra"),
    ] {
        learnkit()
            .arg("learn")
            .arg("vocabulary")
            .arg("add")
            .arg("--session")
            .arg(&session_id)
            .arg("--lemma")
            .arg(lemma)
            .arg("--sense")
            .arg(sense)
            .arg("--source")
            .arg(&source_id)
            .arg("--path")
            .arg(project_root)
            .assert()
            .success();
    }

    session_id
}

#[test]
fn assessment_build_covers_all_three_skills() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_two_vocabulary_items(dir.path());

    let out = learnkit()
        .arg("assessment")
        .arg("build")
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
    assert_eq!(json["items"], 6); // 2 vocabulary items x 3 skills
    let mut skills: Vec<String> = json["skills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    skills.sort();
    assert_eq!(skills, vec!["listening", "production", "recognition"]);
}

#[test]
fn export_exam_produces_self_contained_html_and_attempt_import_is_append_only() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_two_vocabulary_items(dir.path());

    let build_out = learnkit()
        .arg("assessment")
        .arg("build")
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
    let assessment_id = serde_json::from_slice::<serde_json::Value>(&build_out).unwrap()
        ["assessment_id"]
        .as_str()
        .unwrap()
        .to_string();

    let exam_path = dir.path().join("dist").join("exam.html");
    learnkit()
        .arg("export")
        .arg("exam")
        .arg("--assessment")
        .arg(&assessment_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&exam_path)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let html = std::fs::read_to_string(&exam_path).unwrap();
    assert!(html.contains("<html"));
    assert!(html.contains("results.json"));
    assert!(!html.contains("http://"));

    // Extract a real item id embedded in the exam's `const ITEMS = [...]`
    // payload, so the simulated results below reference an item that
    // actually exists in this assessment's question bank.
    let items_marker = "const ITEMS = ";
    let items_start = html.find(items_marker).unwrap() + items_marker.len();
    let items_end = html[items_start..].find(";\n").unwrap() + items_start;
    let items_json: serde_json::Value =
        serde_json::from_str(&html[items_start..items_end]).unwrap();
    let real_item_id = items_json[0]["id"].as_str().unwrap().to_string();

    // Simulate the browser's "download results" output.
    let results_path = dir.path().join("results.json");
    let results_json = format!(
        r#"{{"assessment_id": "{assessment_id}", "attempts": [
            {{"item_id": "{real_item_id}", "skill": "recognition", "selected_option_ids": [], "correct": true, "score": 1.0}}
        ]}}"#
    );
    std::fs::write(&results_path, &results_json).unwrap();

    let import1 = learnkit()
        .arg("attempt")
        .arg("import")
        .arg(&results_path)
        .arg("--assessment")
        .arg(&assessment_id)
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
        serde_json::from_slice::<serde_json::Value>(&import1).unwrap()["attempts_imported"],
        1
    );

    // Re-importing the same file must add, never overwrite/replace.
    learnkit()
        .arg("attempt")
        .arg("import")
        .arg(&results_path)
        .arg("--assessment")
        .arg(&assessment_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let attempts_log =
        std::fs::read_to_string(dir.path().join("attempts").join("attempts.jsonl")).unwrap();
    assert_eq!(attempts_log.lines().count(), 2);
}

// --- Hard Guards entry check (odd/tasks/hard-guards-entry-checks.md) ---

#[test]
fn assessment_build_is_blocked_when_there_is_no_vocabulary() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();
    let out = learnkit()
        .arg("session")
        .arg("new")
        .arg("Unit sin vocabulario")
        .arg("--path")
        .arg(dir.path())
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

    let assert = learnkit()
        .arg("assessment")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .failure()
        .code(20);

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(output["ok"], false);
    assert_eq!(output["code"], "BLOCKED");
}

#[test]
fn assessment_build_works_for_preexisting_vocabulary_with_no_phase_manifest_history() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = session_with_two_vocabulary_items(dir.path());

    learnkit()
        .arg("assessment")
        .arg("build")
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
}
