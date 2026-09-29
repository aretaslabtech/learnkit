//! Integration tests for `learnkit learn pronunciation set/remove` —
//! `odd/tasks/language-study-pack.md` T3.

use assert_cmd::Command;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

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

#[test]
fn learn_pronunciation_set_persists_a_minimal_pair() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    let out = learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--word-a")
        .arg("sheets")
        .arg("--ipa-a")
        .arg("ʃiːts")
        .arg("--word-b")
        .arg("shits")
        .arg("--ipa-b")
        .arg("ʃɪts")
        .arg("--note")
        .arg("long vs short i")
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
    assert_eq!(json["minimal_pair_id"], "mp-1");
    assert_eq!(json["session_id"], session_id);

    let session_root = dir.path().join("sessions").join(&session_id);
    let pairs = learnkit_profile::language::pronunciation::load_all(&session_root).unwrap();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].word_a, "sheets");
    assert_eq!(pairs[0].ipa_a, "ʃiːts");
    assert_eq!(pairs[0].word_b, "shits");
    assert_eq!(pairs[0].ipa_b, "ʃɪts");
    assert_eq!(pairs[0].note.as_deref(), Some("long vs short i"));
}

#[test]
fn learn_pronunciation_set_is_an_upsert_keyed_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--word-a")
        .arg("ship")
        .arg("--ipa-a")
        .arg("ʃɪp")
        .arg("--word-b")
        .arg("sheep")
        .arg("--ipa-b")
        .arg("ʃiːp")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--word-a")
        .arg("live")
        .arg("--ipa-a")
        .arg("lɪv")
        .arg("--word-b")
        .arg("leave")
        .arg("--ipa-b")
        .arg("liːv")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let session_root = dir.path().join("sessions").join(&session_id);
    let pairs = learnkit_profile::language::pronunciation::load_all(&session_root).unwrap();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].word_a, "live");
}

#[test]
fn learn_pronunciation_remove_deletes_and_reports_missing_as_false() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--word-a")
        .arg("ship")
        .arg("--ipa-a")
        .arg("ʃɪp")
        .arg("--word-b")
        .arg("sheep")
        .arg("--ipa-b")
        .arg("ʃiːp")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();

    let out = learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("remove")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["removed"], true);

    let session_root = dir.path().join("sessions").join(&session_id);
    assert!(
        learnkit_profile::language::pronunciation::load_all(&session_root)
            .unwrap()
            .is_empty()
    );

    let second_out = learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("remove")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--path")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second_json: serde_json::Value = serde_json::from_slice(&second_out).unwrap();
    assert_eq!(second_json["removed"], false);
}

/// Encoding guard: mojibake in `--word-a`/`--word-b`/`--note` is rejected,
/// but real IPA symbols in `--ipa-a`/`--ipa-b` must never be flagged —
/// they're legitimate non-ASCII text, not mojibake.
#[test]
fn learn_pronunciation_set_rejects_mojibake_word_but_allows_real_ipa() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = new_session(dir.path());

    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-1")
        .arg("--word-a")
        .arg("clasificaciÃ³n")
        .arg("--ipa-a")
        .arg("ʃɪp")
        .arg("--word-b")
        .arg("sheep")
        .arg("--ipa-b")
        .arg("ʃiːp")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure()
        .code(10);

    // Clean words + real IPA symbols must succeed.
    learnkit()
        .arg("learn")
        .arg("pronunciation")
        .arg("set")
        .arg("--session")
        .arg(&session_id)
        .arg("--id")
        .arg("mp-2")
        .arg("--word-a")
        .arg("ship")
        .arg("--ipa-a")
        .arg("ʃɪp")
        .arg("--word-b")
        .arg("sheep")
        .arg("--ipa-b")
        .arg("ʃiːp")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success();
}
