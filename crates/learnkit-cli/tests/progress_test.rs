//! Integration tests for `learnkit progress`, User Story 7
//! (specs/002-english-eoi-flow/tasks.md T068-T070).

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

fn session_with_one_attempt(project_root: &std::path::Path) -> String {
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

    learnkit()
        .arg("learn")
        .arg("vocabulary")
        .arg("add")
        .arg("--session")
        .arg(&session_id)
        .arg("--lemma")
        .arg("whiteboard")
        .arg("--sense")
        .arg("pizarra")
        .arg("--source")
        .arg(&source_id)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();

    let build_out = learnkit()
        .arg("assessment")
        .arg("build")
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
    let assessment_id = serde_json::from_slice::<serde_json::Value>(&build_out).unwrap()
        ["assessment_id"]
        .as_str()
        .unwrap()
        .to_string();

    let exam_path = project_root.join("exam.html");
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
        .arg(project_root)
        .assert()
        .success();
    let html = std::fs::read_to_string(&exam_path).unwrap();
    let items_marker = "const ITEMS = ";
    let items_start = html.find(items_marker).unwrap() + items_marker.len();
    let items_end = html[items_start..].find(";\n").unwrap() + items_start;
    let items_json: serde_json::Value =
        serde_json::from_str(&html[items_start..items_end]).unwrap();
    let listening_item = items_json
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["skill"] == "listening")
        .unwrap();
    let item_id = listening_item["id"].as_str().unwrap();

    let results_path = project_root.join("results.json");
    std::fs::write(
        &results_path,
        format!(
            r#"{{"assessment_id": "{assessment_id}", "attempts": [{{"item_id": "{item_id}", "skill": "listening", "selected_option_ids": [], "correct": true, "score": 1.0}}]}}"#
        ),
    )
    .unwrap();
    learnkit()
        .arg("attempt")
        .arg("import")
        .arg(&results_path)
        .arg("--assessment")
        .arg(&assessment_id)
        .arg("--session")
        .arg(&session_id)
        .arg("--path")
        .arg(project_root)
        .assert()
        .success();

    session_id
}

#[test]
fn progress_aggregates_by_learning_item_and_skill() {
    let dir = tempfile::tempdir().unwrap();
    let _session_id = session_with_one_attempt(dir.path());

    let out = learnkit()
        .arg("progress")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(json["by_learning_item"].as_array().unwrap().len(), 1);
    let skills = json["by_skill"].as_array().unwrap();
    assert!(skills.iter().any(|s| s["key"] == "listening"));
}

#[test]
fn progress_recomputes_on_each_call_after_new_attempts() {
    let dir = tempfile::tempdir().unwrap();
    let _session_id = session_with_one_attempt(dir.path());

    let first = learnkit()
        .arg("progress")
        .arg("--skill")
        .arg("listening")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let first_json: serde_json::Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(first_json["by_skill"][0]["attempt_count"], 1);

    // Append a second attempt directly and confirm progress reflects it
    // without any caching.
    let attempts_path = dir.path().join("attempts").join("attempts.jsonl");
    let existing = std::fs::read_to_string(&attempts_path).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_str(existing.lines().next().unwrap()).unwrap();
    value["attempt_id"] = serde_json::Value::String("attempt-extra".to_string());
    value["correct"] = serde_json::Value::Bool(false);
    std::fs::write(&attempts_path, format!("{existing}{}\n", value)).unwrap();

    let second = learnkit()
        .arg("progress")
        .arg("--skill")
        .arg("listening")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_json: serde_json::Value = serde_json::from_slice(&second).unwrap();
    assert_eq!(second_json["by_skill"][0]["attempt_count"], 2);
}

#[test]
fn skill_filter_narrows_the_view() {
    let dir = tempfile::tempdir().unwrap();
    session_with_one_attempt(dir.path());

    let out = learnkit()
        .arg("progress")
        .arg("--skill")
        .arg("listening")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["by_skill"].as_array().unwrap().len(), 1);
    assert_eq!(json["by_skill"][0]["key"], "listening");
}
