//! Integration tests for `learnkit learn level set/show` —
//! `odd/tasks/language-study-pack.md` T4.

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

#[test]
fn learn_level_show_before_any_set_reports_none_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let out = learnkit()
        .arg("learn")
        .arg("level")
        .arg("show")
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
    assert_eq!(json["level_set"], false);
    assert!(json["level"].is_null());
}

#[test]
fn learn_level_set_persists_and_show_reads_it_back() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    let set_out = learnkit()
        .arg("learn")
        .arg("level")
        .arg("set")
        .arg("--language")
        .arg("en")
        .arg("--variety")
        .arg("en-GB")
        .arg("--level")
        .arg("B1")
        .arg("--notes")
        .arg("solid on past tenses")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let set_json: serde_json::Value = serde_json::from_slice(&set_out).unwrap();
    assert_eq!(set_json["ok"], true);
    assert_eq!(set_json["level"], "B1");
    assert!(set_json["updated_at"].as_str().unwrap().ends_with('Z'));

    let show_out = learnkit()
        .arg("learn")
        .arg("level")
        .arg("show")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_json: serde_json::Value = serde_json::from_slice(&show_out).unwrap();
    assert_eq!(show_json["level_set"], true);
    assert_eq!(show_json["level"], "B1");
    assert_eq!(show_json["variety"], "en-GB");
    assert_eq!(show_json["language"], "en");
    assert_eq!(show_json["level_notes"], "solid on past tenses");

    assert_eq!(
        learnkit_profile::language::level::show(dir.path())
            .unwrap()
            .unwrap()
            .level,
        "B1"
    );
}

#[test]
fn learn_level_set_overwrites_the_previous_level_wholesale() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    learnkit()
        .arg("learn")
        .arg("level")
        .arg("set")
        .arg("--language")
        .arg("en")
        .arg("--variety")
        .arg("en-GB")
        .arg("--level")
        .arg("A2")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    learnkit()
        .arg("learn")
        .arg("level")
        .arg("set")
        .arg("--language")
        .arg("en")
        .arg("--variety")
        .arg("en-GB")
        .arg("--level")
        .arg("B1")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let level = learnkit_profile::language::level::show(dir.path())
        .unwrap()
        .unwrap();
    assert_eq!(level.level, "B1");
}

/// Encoding guard: mojibake in `--notes` is rejected.
#[test]
fn learn_level_set_rejects_mojibake_notes() {
    let dir = tempfile::tempdir().unwrap();
    learnkit().arg("init").arg(dir.path()).assert().success();

    learnkit()
        .arg("learn")
        .arg("level")
        .arg("set")
        .arg("--language")
        .arg("en")
        .arg("--variety")
        .arg("en-GB")
        .arg("--level")
        .arg("B1")
        .arg("--notes")
        .arg("clasificaciÃ³n")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);
}
