//! Integration tests for `learnkit export study-pack` — Fase C, T9
//! (`odd/tasks/language-study-pack.md`). Follows the setup pattern of
//! `export_study_guide_test.rs` (CLI-driven `analyse set`) and
//! `export_anki_test.rs` (direct `learnkit_cards::card::save` for
//! deterministic card fixtures), plus direct calls into
//! `learnkit_profile::language::{dialogue,pronunciation,vocabulary}` and
//! `learnkit_assessment::item` for the entities that have no simple CLI
//! shortcut in this test's scope.

use assert_cmd::Command;
use calamine::{open_workbook, Reader, Xlsx};
use learnkit_assessment::item::{save_assessment, save_item, Assessment, AssessmentItem, Option_, Skill};
use learnkit_cards::card::{save as save_card, Block, CardDefinition, Side};
use learnkit_profile::language::dialogue::{self, DialogueLine, DialogueOrigin};
use learnkit_profile::language::pronunciation;
use learnkit_profile::language::vocabulary;
use learnkit_store::session_paths::SessionPaths;
use std::path::Path;

fn learnkit() -> Command {
    Command::cargo_bin("learnkit").unwrap()
}

fn new_session(project_root: &Path) -> String {
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

fn set_summary(project_root: &Path, session_id: &str, content_dir: &Path) {
    let file = content_dir.join("summary.md");
    std::fs::write(&file, "Contenido del resumen de la clase.").unwrap();
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

/// Builds a session with content in several of the 10 tabs: summary,
/// vocabulary (one regular entry + one expression), a dialogue, a minimal
/// pair, a flashcard, and an assessment item.
fn full_session(project_root: &Path) -> String {
    let content_dir = tempfile::tempdir().unwrap();
    let session_id = new_session(project_root);
    let session_paths = SessionPaths::new(project_root, &session_id);

    set_summary(project_root, &session_id, content_dir.path());

    let mut vocab = vocabulary::add_or_reuse(
        project_root,
        "whiteboard",
        "pizarra",
        "src-1",
        None,
        None,
        "en-GB",
    )
    .unwrap();
    vocabulary::set_details(
        project_root,
        &mut vocab,
        Some("ˈwaɪtbɔːd"),
        &["Write it on the whiteboard.".to_string()],
        Some("Unit 5 — Classroom"),
        Some("common classroom object"),
    )
    .unwrap();

    let mut expression = vocabulary::add_or_reuse(
        project_root,
        "get away with",
        "hacer algo malo sin castigo",
        "src-1",
        None,
        None,
        "en-GB",
    )
    .unwrap();
    vocabulary::set_fields(
        project_root,
        &mut expression,
        None,
        Some("expression"),
        None,
        None,
        None,
        None,
    )
    .unwrap();

    dialogue::set(
        session_paths.root(),
        "dlg-1",
        &session_id,
        DialogueOrigin::FromClass,
        vec![
            DialogueLine {
                speaker: "A".to_string(),
                text: "Hello".to_string(),
            },
            DialogueLine {
                speaker: "B".to_string(),
                text: "Hi there".to_string(),
            },
        ],
        Some("greeting exchange"),
    )
    .unwrap();

    pronunciation::set(
        session_paths.root(),
        "mp-1",
        &session_id,
        "sheets",
        "ʃiːts",
        "shits",
        "ʃɪts",
        Some("long vs short /i/"),
    )
    .unwrap();

    save_card(
        session_paths.root(),
        &CardDefinition {
            id: "card-1".to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            template: "word-to-meaning-v1".to_string(),
            front: Side {
                blocks: vec![Block::Text {
                    value: "whiteboard".to_string(),
                }],
            },
            back: Side {
                blocks: vec![Block::Text {
                    value: "pizarra".to_string(),
                }],
            },
        },
    )
    .unwrap();

    let item = AssessmentItem {
        id: "q-001".to_string(),
        learning_item_ids: vec!["li-1".to_string()],
        item_type: "multiple_choice".to_string(),
        prompt: "What does 'whiteboard' mean?".to_string(),
        prompt_audio_asset_id: None,
        options: vec![
            Option_ {
                id: "opt-a".to_string(),
                text: "pizarra".to_string(),
            },
            Option_ {
                id: "opt-b".to_string(),
                text: "ventana".to_string(),
            },
        ],
        correct_option_ids: vec!["opt-a".to_string()],
        skill: Skill::Recognition,
    };
    save_item(session_paths.root(), &item).unwrap();
    save_assessment(
        session_paths.root(),
        &Assessment {
            id: format!("exam-{session_id}"),
            title: "Unit 5 exam".to_string(),
            item_ids: vec!["q-001".to_string()],
        },
    )
    .unwrap();

    session_id
}

fn sheet_names(path: &Path) -> Vec<String> {
    let workbook: Xlsx<_> = open_workbook(path).unwrap();
    workbook.sheet_names().to_vec()
}

fn read_sheet(path: &Path, name: &str) -> Vec<Vec<String>> {
    let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let range = workbook.worksheet_range(name).unwrap();
    range
        .rows()
        .map(|row| row.iter().map(|c| c.to_string()).collect())
        .collect()
}

#[test]
fn export_study_pack_writes_all_ten_tabs_with_expected_content() {
    let project = tempfile::tempdir().unwrap();
    let session_id = full_session(project.path());

    let out_file = project.path().join("pack.xlsx");
    learnkit()
        .arg("export")
        .arg("study-pack")
        .arg("--session")
        .arg(&session_id)
        .arg("--out")
        .arg(&out_file)
        .arg("--path")
        .arg(project.path())
        .arg("--json")
        .assert()
        .success();

    assert!(out_file.exists());
    assert!(std::fs::metadata(&out_file).unwrap().len() > 0);

    let names = sheet_names(&out_file);
    assert_eq!(
        names,
        vec![
            "Lo aprendido",
            "Vocabulario",
            "Expresiones",
            "Pronunciacion",
            "Alfabeto",
            "Dialogos",
            "Repaso",
            "Soluciones",
            "Flashcards",
            "Fuente",
        ]
    );

    let lo_aprendido = read_sheet(&out_file, "Lo aprendido");
    assert!(lo_aprendido
        .iter()
        .any(|r| r.contains(&"Contenido del resumen de la clase.".to_string())));

    let vocabulario = read_sheet(&out_file, "Vocabulario");
    assert!(vocabulario.iter().any(|r| r[0] == "whiteboard"));
    let whiteboard_row = vocabulario
        .iter()
        .find(|r| r[0] == "whiteboard")
        .unwrap()
        .clone();
    assert_eq!(whiteboard_row[1], "pizarra");
    assert_eq!(whiteboard_row[3], "ˈwaɪtbɔːd");
    assert_eq!(whiteboard_row[6], "Unit 5 — Classroom");
    assert_eq!(whiteboard_row[7], session_id);

    let expresiones = read_sheet(&out_file, "Expresiones");
    assert!(expresiones.iter().any(|r| r[0] == "get away with"));
    assert!(!expresiones.iter().any(|r| r[0] == "whiteboard"));

    let pronunciacion = read_sheet(&out_file, "Pronunciacion");
    assert!(pronunciacion.iter().any(|r| r[0] == "sheets"));

    let alfabeto = read_sheet(&out_file, "Alfabeto");
    assert_eq!(alfabeto.len(), 27); // header + 26 letters
    assert!(alfabeto.iter().any(|r| r[0] == "A" && r[1] == "/eɪ/"));

    let dialogos = read_sheet(&out_file, "Dialogos");
    assert!(dialogos
        .iter()
        .any(|r| r[0] == "Trabajado en clase" && r[1].contains("A: Hello")));

    let repaso = read_sheet(&out_file, "Repaso");
    assert!(repaso
        .iter()
        .any(|r| r[0] == "What does 'whiteboard' mean?"));
    // Never leak the correct answer in "Repaso".
    for row in &repaso {
        assert!(!row.iter().any(|c| c == "opt-a"));
    }

    let soluciones = read_sheet(&out_file, "Soluciones");
    assert!(soluciones
        .iter()
        .any(|r| r[0] == "What does 'whiteboard' mean?" && r[1] == "pizarra"));

    let flashcards = read_sheet(&out_file, "Flashcards");
    assert!(flashcards
        .iter()
        .any(|r| r[0] == "card-1" && r[1] == "whiteboard" && r[2] == "pizarra"));

    let fuente = read_sheet(&out_file, "Fuente");
    assert!(fuente.iter().any(|r| r[0] == "Sesión" && r[1] == session_id));
    assert!(fuente
        .iter()
        .any(|r| r[0] == "Nivel" && r[1] == "Sin nivel registrado"));
}

#[test]
fn export_study_pack_fails_on_a_completely_empty_session_and_writes_no_file() {
    let project = tempfile::tempdir().unwrap();
    let session_id = new_session(project.path());

    let out_file = project.path().join("pack.xlsx");
    learnkit()
        .arg("export")
        .arg("study-pack")
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
