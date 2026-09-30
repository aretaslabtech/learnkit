//! `learnkit export study-pack`: renders a session's learning material into a
//! single `.xlsx` workbook with the same tab structure as the reference
//! Excel that motivated this feature (`odd/tasks/language-study-pack.md`,
//! Fase C / T8): `Lo aprendido, Vocabulario, Expresiones, Pronunciacion,
//! Alfabeto, Dialogos, Repaso, Soluciones, Flashcards, Fuente`.
//!
//! Reuses the same loading functions as `export study-guide`/`export anki`
//! (never re-derives business logic): `learnkit_workflow::analysis` for
//! "Lo aprendido", `learnkit_cards::card` for "Flashcards",
//! `learnkit_assessment::item` for "Repaso"/"Soluciones".
//!
//! ## Design decisions (documented per
//! `odd/tasks/language-study-pack.md` T8, since the data model doesn't
//! specify all of these exactly):
//!
//! - **Vocabulario is project-wide, not session-filtered.** `VocabularyEntry`
//!   survives across sessions (see `vocabulary.rs` doc comments) and carries
//!   no `session_id`; its only traceability is `sources[].source_id`, which
//!   points at an *inventory source*, not a session. Reliably mapping an
//!   inventory source back to "the session this export targets" would
//!   require crossing session inventories, which is out of scope here.
//!   `export study-pack` therefore exports **all** project vocabulary,
//!   mirroring how `assessment build`/`cards build` already treat vocabulary
//!   as project-level. The "Sesión" column is filled with the *target*
//!   session id for every row (the session the export was run for) rather
//!   than a per-entry value, since no per-entry session exists to report.
//! - **"Traducción ejemplo"** has no dedicated field in `Example` today (only
//!   `text`, the English example sentence) — left empty, never fabricated.
//! - **Expresiones "Uso"/"Origen"**: `VocabularyEntry` has no literal
//!   "origin" field distinguishing material worked in class from material
//!   added later (unlike `Dialogue`/`ConceptPage`, which do). `Uso` is left
//!   empty; `Origen` is fixed to "Trabajado en clase" for every expression,
//!   since expressions come from `learn vocabulary add`, which the Skill
//!   documents as capturing lemmas actually encountered — this is a known
//!   simplification, not a real trace of origin.
//! - **Fuente "Fecha"**: `Session::created_at` is `format!("{:?}",
//!   SystemTime::now())` (Rust `Debug` of `SystemTime`, not a clean
//!   human date) — used as-is since it's the only creation timestamp the
//!   model exposes; not reformatted (no date/time crate in the workspace,
//!   same constraint documented in `level.rs`).

use crate::session_context::resolve_session;
use clap::Args;
use learnkit_assessment::item::{load_all_items, load_assessment, AssessmentItem};
use learnkit_cards::card::{load_all as load_all_cards, Block, CardDefinition};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_profile::language::dialogue::{self, Dialogue, DialogueOrigin};
use learnkit_profile::language::level;
use learnkit_profile::language::pronunciation::{self, MinimalPair};
use learnkit_profile::language::vocabulary::{self, VocabularyEntry};
use learnkit_workflow::analysis::{list_concept_pages, read_study_map, read_summary};
use learnkit_workflow::session::read_session;
use rust_xlsxwriter::{Workbook, Worksheet};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportStudyPackArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct ExportData {
    #[serde(skip_serializing_if = "Option::is_none")]
    out: Option<String>,
}

/// Static reference table (British IPA), documented in
/// `odd/tasks/language-study-pack.md` decision 5: English-only for now.
const ENGLISH_ALPHABET: [(&str, &str); 26] = [
    ("A", "eɪ"),
    ("B", "biː"),
    ("C", "siː"),
    ("D", "diː"),
    ("E", "iː"),
    ("F", "ef"),
    ("G", "dʒiː"),
    ("H", "eɪtʃ"),
    ("I", "aɪ"),
    ("J", "dʒeɪ"),
    ("K", "keɪ"),
    ("L", "el"),
    ("M", "em"),
    ("N", "en"),
    ("O", "əʊ"),
    ("P", "piː"),
    ("Q", "kjuː"),
    ("R", "ɑːr"),
    ("S", "es"),
    ("T", "tiː"),
    ("U", "juː"),
    ("V", "viː"),
    ("W", "ˈdʌbəljuː"),
    ("X", "eks"),
    ("Y", "waɪ"),
    ("Z", "zed"),
];

pub fn run(args: ExportStudyPackArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    let session = match read_session(&root, &session_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ExporterConstraint {
                    message: format!("session '{session_id}' not found"),
                },
            )
        }
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let session_root = learnkit_store::session_paths::SessionPaths::new(&root, &session_id);

    macro_rules! try_io {
        ($expr:expr) => {
            match $expr {
                Ok(v) => v,
                Err(source) => {
                    return emit_error(
                        args.json,
                        LearnKitError::Filesystem {
                            path: root.display().to_string(),
                            source,
                        },
                    )
                }
            }
        };
    }

    let summary = try_io!(read_summary(&root, &session_id));
    let study_map = try_io!(read_study_map(&root, &session_id));
    let pages = try_io!(list_concept_pages(&root, &session_id));

    let vocabulary = try_io!(vocabulary::list_all(&root));

    let dialogues = try_io!(dialogue::load_all(session_root.root()));
    let minimal_pairs = try_io!(pronunciation::load_all(session_root.root()));

    let cards = try_io!(load_all_cards(session_root.root()));

    let assessment_items: Vec<AssessmentItem> = match load_assessment(
        session_root.root(),
        &format!("exam-{session_id}"),
    ) {
        Ok(Some(assessment)) => {
            let all_items = try_io!(load_all_items(session_root.root()));
            all_items
                .into_iter()
                .filter(|i| assessment.item_ids.contains(&i.id))
                .collect()
        }
        Ok(None) => Vec::new(),
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let language_level = try_io!(level::show(&root));

    // --- fail if there is nothing at all to export ---
    let has_learned_content = summary.is_some() || study_map.is_some() || !pages.is_empty();
    let has_anything = has_learned_content
        || !vocabulary.is_empty()
        || !dialogues.is_empty()
        || !minimal_pairs.is_empty()
        || !cards.is_empty()
        || !assessment_items.is_empty();

    if !has_anything {
        return emit_error(
            args.json,
            LearnKitError::ExporterConstraint {
                message: "cannot export — nothing to export for this session (no summary/mindmap/pages, no vocabulary, no dialogues, no minimal pairs, no cards, no assessment)".to_string(),
            },
        );
    }

    let mut workbook = Workbook::new();

    if let Err(err) = write_lo_aprendido(&mut workbook, &summary, &study_map, &pages) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_vocabulario(&mut workbook, &vocabulary, &session_id) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_expresiones(&mut workbook, &vocabulary) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_pronunciacion(&mut workbook, &minimal_pairs) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_alfabeto(&mut workbook) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_dialogos(&mut workbook, &dialogues) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_repaso(&mut workbook, &assessment_items) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_soluciones(&mut workbook, &assessment_items) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_flashcards(&mut workbook, &cards) {
        return emit_write_error(args.json, err);
    }
    if let Err(err) = write_fuente(&mut workbook, &session_id, &session.created_at, &language_level)
    {
        return emit_write_error(args.json, err);
    }

    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(source) = std::fs::create_dir_all(parent) {
                return emit_error(
                    args.json,
                    LearnKitError::Filesystem {
                        path: args.out.display().to_string(),
                        source,
                    },
                );
            }
        }
    }

    if let Err(err) = workbook.save(&args.out) {
        return emit_write_error(args.json, err.to_string());
    }

    if args.json {
        Envelope::ok(
            "STUDY_PACK_EXPORTED",
            ExportData {
                out: Some(args.out.display().to_string()),
            },
        )
        .print_json();
    } else {
        println!("Material didáctico exportado a {}", args.out.display());
    }
    0
}

fn write_header(sheet: &mut Worksheet, headers: &[&str]) -> Result<(), rust_xlsxwriter::XlsxError> {
    for (col, header) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, *header)?;
    }
    Ok(())
}

fn write_lo_aprendido(
    workbook: &mut Workbook,
    summary: &Option<learnkit_workflow::analysis::ClassSummary>,
    study_map: &Option<learnkit_workflow::analysis::StudyMap>,
    pages: &[learnkit_workflow::analysis::ConceptPage],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Lo aprendido")?;
    write_header(sheet, &["Tipo", "Título", "Contenido", "Huecos rellenados"])?;

    let mut row = 1u32;

    if let Some(summary) = summary {
        let gaps = format_filled_gaps(&summary.filled_gaps);
        sheet.write_string(row, 0, "Resumen")?;
        sheet.write_string(row, 1, "Resumen")?;
        sheet.write_string(row, 2, &summary.content)?;
        sheet.write_string(row, 3, &gaps)?;
        row += 1;
    }

    for page in pages {
        let gaps = format_filled_gaps(&page.filled_gaps);
        sheet.write_string(row, 0, "Concepto")?;
        sheet.write_string(row, 1, &page.concept)?;
        sheet.write_string(row, 2, &page.content)?;
        sheet.write_string(row, 3, &gaps)?;
        row += 1;
    }

    if let Some(map) = study_map {
        let gaps = format_filled_gaps(&map.filled_gaps);
        sheet.write_string(row, 0, "Mapa de estudio")?;
        sheet.write_string(row, 1, "Mapa de estudio")?;
        sheet.write_string(row, 2, &map.content)?;
        sheet.write_string(row, 3, &gaps)?;
    }

    Ok(())
}

fn format_filled_gaps(gaps: &[learnkit_workflow::analysis::FilledGap]) -> String {
    gaps.iter()
        .map(|g| format!("{}: {}", g.concept, g.note))
        .collect::<Vec<_>>()
        .join("; ")
}

fn write_vocabulario(
    workbook: &mut Workbook,
    vocabulary: &[VocabularyEntry],
    session_id: &str,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Vocabulario")?;
    write_header(
        sheet,
        &[
            "English",
            "Español",
            "Tipo",
            "Pronunciación",
            "Ejemplo",
            "Traducción ejemplo",
            "Tema",
            "Sesión",
            "Observaciones",
        ],
    )?;

    for (i, entry) in vocabulary.iter().enumerate() {
        let row = (i + 1) as u32;
        let gloss = entry
            .senses
            .first()
            .map(|s| s.gloss.as_str())
            .unwrap_or("");
        let example = entry
            .examples
            .first()
            .map(|e| e.text.as_str())
            .unwrap_or("");

        sheet.write_string(row, 0, &entry.lemma)?;
        sheet.write_string(row, 1, gloss)?;
        sheet.write_string(row, 2, entry.part_of_speech.as_deref().unwrap_or(""))?;
        sheet.write_string(row, 3, entry.ipa.as_deref().unwrap_or(""))?;
        sheet.write_string(row, 4, example)?;
        // "Traducción ejemplo": no dedicated field exists — left empty.
        sheet.write_string(row, 5, "")?;
        sheet.write_string(row, 6, entry.topic.as_deref().unwrap_or(""))?;
        sheet.write_string(row, 7, session_id)?;
        sheet.write_string(row, 8, entry.notes.as_deref().unwrap_or(""))?;
    }

    Ok(())
}

fn write_expresiones(
    workbook: &mut Workbook,
    vocabulary: &[VocabularyEntry],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Expresiones")?;
    write_header(sheet, &["English", "Español", "Uso", "Origen"])?;

    let expressions: Vec<&VocabularyEntry> = vocabulary
        .iter()
        .filter(|e| {
            e.part_of_speech
                .as_deref()
                .map(|p| p.eq_ignore_ascii_case("expression"))
                .unwrap_or(false)
        })
        .collect();

    for (i, entry) in expressions.iter().enumerate() {
        let row = (i + 1) as u32;
        let gloss = entry
            .senses
            .first()
            .map(|s| s.gloss.as_str())
            .unwrap_or("");
        sheet.write_string(row, 0, &entry.lemma)?;
        sheet.write_string(row, 1, gloss)?;
        // "Uso": no dedicated field exists — left empty.
        sheet.write_string(row, 2, "")?;
        // "Origen": VocabularyEntry has no literal-vs-added marker (unlike
        // Dialogue/ConceptPage) — fixed text, documented as a simplification.
        sheet.write_string(row, 3, "Trabajado en clase")?;
    }

    Ok(())
}

fn write_pronunciacion(
    workbook: &mut Workbook,
    pairs: &[MinimalPair],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Pronunciacion")?;
    write_header(sheet, &["Palabra A", "IPA A", "Palabra B", "IPA B", "Nota"])?;

    for (i, pair) in pairs.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet.write_string(row, 0, &pair.word_a)?;
        sheet.write_string(row, 1, &pair.ipa_a)?;
        sheet.write_string(row, 2, &pair.word_b)?;
        sheet.write_string(row, 3, &pair.ipa_b)?;
        sheet.write_string(row, 4, pair.note.as_deref().unwrap_or(""))?;
    }

    Ok(())
}

fn write_alfabeto(workbook: &mut Workbook) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Alfabeto")?;
    write_header(sheet, &["Letra", "IPA"])?;

    for (i, (letter, ipa)) in ENGLISH_ALPHABET.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet.write_string(row, 0, *letter)?;
        sheet.write_string(row, 1, format!("/{ipa}/"))?;
    }

    Ok(())
}

fn write_dialogos(
    workbook: &mut Workbook,
    dialogues: &[Dialogue],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Dialogos")?;
    write_header(sheet, &["Origen", "Diálogo", "Nota"])?;

    for (i, dialogue) in dialogues.iter().enumerate() {
        let row = (i + 1) as u32;
        let origin = match dialogue.origin {
            DialogueOrigin::FromClass => "Trabajado en clase",
            DialogueOrigin::AddedForConsolidation => "Añadido para consolidar",
        };
        let text = dialogue
            .lines
            .iter()
            .map(|l| format!("{}: {}", l.speaker, l.text))
            .collect::<Vec<_>>()
            .join("\n");

        sheet.write_string(row, 0, origin)?;
        sheet.write_string(row, 1, &text)?;
        sheet.write_string(row, 2, dialogue.note.as_deref().unwrap_or(""))?;
    }

    Ok(())
}

fn write_repaso(
    workbook: &mut Workbook,
    items: &[AssessmentItem],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Repaso")?;

    let max_options = items.iter().map(|i| i.options.len()).max().unwrap_or(0);
    let mut headers = vec!["Pregunta".to_string()];
    for i in 0..max_options {
        headers.push(format!("Opción {}", option_letter(i)));
    }
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    write_header(sheet, &header_refs)?;

    for (row_idx, item) in items.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        sheet.write_string(row, 0, &item.prompt)?;
        for (col_idx, option) in item.options.iter().enumerate() {
            sheet.write_string(row, (col_idx + 1) as u16, &option.text)?;
        }
        // Never write correct_option_ids here — that's "Soluciones" only.
    }

    Ok(())
}

fn write_soluciones(
    workbook: &mut Workbook,
    items: &[AssessmentItem],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Soluciones")?;
    write_header(sheet, &["Pregunta", "Respuesta correcta"])?;

    for (row_idx, item) in items.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        let correct_text = item
            .options
            .iter()
            .filter(|o| item.correct_option_ids.contains(&o.id))
            .map(|o| o.text.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        sheet.write_string(row, 0, &item.prompt)?;
        sheet.write_string(row, 1, &correct_text)?;
    }

    Ok(())
}

fn option_letter(index: usize) -> char {
    (b'A' + (index as u8)) as char
}

fn write_flashcards(
    workbook: &mut Workbook,
    cards: &[CardDefinition],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Flashcards")?;
    write_header(sheet, &["Id", "Frente", "Dorso"])?;

    for (i, card) in cards.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet.write_string(row, 0, &card.id)?;
        sheet.write_string(row, 1, side_text(&card.front))?;
        sheet.write_string(row, 2, side_text(&card.back))?;
    }

    Ok(())
}

fn side_text(side: &learnkit_cards::card::Side) -> String {
    side.blocks
        .iter()
        .filter_map(|b| match b {
            Block::Text { value } => Some(value.as_str()),
            Block::Image { .. } | Block::Audio { .. } => None,
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn write_fuente(
    workbook: &mut Workbook,
    session_id: &str,
    created_at: &str,
    language_level: &Option<level::LanguageLevel>,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet().set_name("Fuente")?;
    write_header(sheet, &["Campo", "Valor"])?;

    let nivel = match language_level {
        Some(l) => format!("{} {} — {}", l.language, l.variety, l.level),
        None => "Sin nivel registrado".to_string(),
    };

    let rows: [(&str, &str); 5] = [
        ("Sesión", session_id),
        ("Fecha", created_at),
        ("Nivel", &nivel),
        (
            "Criterio",
            "Lo trabajado en clase se distingue del material añadido para completar el aprendizaje (ver columna Huecos rellenados / Origen en las pestañas correspondientes).",
        ),
        ("Herramienta", "Generado por LearnKit"),
    ];

    for (i, (field, value)) in rows.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet.write_string(row, 0, *field)?;
        sheet.write_string(row, 1, *value)?;
    }

    Ok(())
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), ExportData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}

fn emit_write_error(json: bool, err: impl std::fmt::Display) -> i32 {
    let err = LearnKitError::ExporterConstraint {
        message: format!("failed to write .xlsx: {err}"),
    };
    emit_error(json, err)
}
