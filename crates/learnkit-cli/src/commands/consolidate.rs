//! `learnkit consolidate` / `learnkit consolidate list` — US4, feature 003.
//!
//! Derives vocabulary candidates from a session's already-confirmed
//! `analyse` output (`ClassSummary`/`ConceptPage`), checks each one against
//! the persisted `VocabularyEntry` store (dedup by `lemma`, reused as-is
//! from feature 002) and persists the result via
//! `learnkit-workflow::consolidate`. This is the one place allowed to depend
//! on both `learnkit-workflow` and `learnkit-profile` — the dedup check
//! itself never moves into `learnkit-workflow` (`research.md` §4).
//!
//! Candidate derivation is deliberately mechanical, not linguistically
//! sophisticated (`tasks.md` T033): split the confirmed text on
//! non-alphanumeric boundaries, keep tokens of 4+ characters, lowercase and
//! deduplicate within the session, and drop a small stoplist of trivial
//! function words. The actual vocabulary *suggestion* step already happens
//! conversationally via the `learnkit-language` Skill (feature 002); this is
//! only the candidate-generation input to it.

use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_profile::language::vocabulary::find_by_lemma;
use learnkit_store::session_paths::SessionPaths;
use learnkit_workflow::analysis::{self, ClassSummary, ConceptPage};
use learnkit_workflow::consolidate::{
    self, blocking_pending_items, CandidateSourceRef, CandidateType, ConsolidatedCandidate,
};
use learnkit_workflow::engine::{
    read_checklist, write_manifest, ChecklistItemManifest, PhaseManifest, PhaseResult,
};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

/// Common English and Spanish words trivial enough to never be useful
/// vocabulary candidates on their own, plus "meta" words that describe the
/// class/session itself rather than anything taught in it (`resumen`,
/// `clase`, `turno`...) — these show up constantly in a LearnKit-generated
/// summary precisely *because* it's a summary of a class, not because they
/// were vocabulary worked on in it.
///
/// Not a linguistically complete stopword list for either language — this
/// is a mechanical, non-NLP derivation (`tasks.md` T033/T040), the actual
/// vocabulary *suggestion* step happens conversationally via the
/// `learnkit-language` Skill (feature 002). The original ~35-word list
/// (FR-016, found post-release) let a single real summary through almost
/// entirely as "candidates" — 831 of them, nearly all noise — so this list
/// is deliberately much larger, even though it still can't replace real
/// frequency data or an agent's judgment.
const STOPWORDS: &[&str] = &[
    // English: pronouns, be/have/do, modals, determiners, conjunctions,
    // prepositions.
    "the", "and", "that", "with", "from", "this", "have", "were", "they", "what", "when", "where",
    "which", "been", "being", "would", "could", "should", "about", "there", "then", "than", "also",
    "your", "their", "them", "some", "such", "only", "just", "very", "more", "most", "other",
    "into", "over", "after", "before", "because", "while", "during", "these", "those", "here",
    "who", "whom", "whose", "does", "doing", "done", "will", "shall", "must", "might", "cannot",
    "each", "every", "both", "either", "neither", "any", "few", "many", "much", "several", "all",
    "same", "different", "own", "another", "again", "once", "still", "even", "though", "although",
    "until", "unless", "since", "upon", "within", "without", "under", "above", "below", "between",
    "among", "through", "against", "toward", "towards", "across", "along", "around", "behind",
    "beyond", "beside", "besides", "despite", "except", "plus", "minus", "per", "via",
    // English: very common verbs/adjectives/adverbs/nouns that are almost
    // never new vocabulary in a language-class summary (they're the
    // narration's own scaffolding, not target words).
    "make", "made", "said", "says", "like", "time", "people", "know", "knew", "known", "think",
    "thought", "want", "wanted", "come", "came", "look", "looked", "give", "gave", "given", "work",
    "worked", "call", "called", "need", "needed", "feel", "felt", "seem", "seemed", "leave", "left",
    "keep", "kept", "let", "ask", "asked", "show", "showed", "shown", "try", "tried", "point",
    "back", "good", "well", "way", "ways", "day", "days", "life", "world", "hand", "part", "parts",
    "place", "case", "week", "weeks", "month", "months", "year", "years", "student", "students",
    "teacher", "teachers", "class", "classes", "lesson", "lessons", "example", "examples", "word",
    "words", "review", "mistake", "mistakes", "rule", "rules", "concept", "concepts", "material",
    "session", "sessions", "content", "topic", "topics", "practice", "practiced", "exercise",
    "exercises", "activity", "activities", "level", "levels", "group", "groups", "note", "notes",
    // Spanish: artículos, pronombres, preposiciones, conjunciones,
    // conectores, verbos muy comunes.
    "para", "pero", "como", "hemos", "esta", "este", "esto", "estas", "estos", "muy", "queda",
    "quedan", "desde", "hasta", "sobre", "entre", "durante", "cuando", "donde", "porque", "aunque",
    "mientras", "también", "tampoco", "además", "sino", "según", "mediante", "hacia", "cada",
    "otros", "otras", "otro", "otra", "algunos", "algunas", "todos", "todas", "mismo", "misma",
    "propio", "propia", "cual", "cuales", "quien", "quienes", "cuyo", "cuya",
    // Spanish: palabras "meta" que describen la propia clase/sesión, no
    // vocabulario objetivo — encontrado tras uso real (FR-016).
    "resumen", "clase", "clases", "turno", "turnos", "profesor", "profesora", "alumno", "alumna",
    "alumnos", "estudiante", "estudiantes", "sesión", "sesiones", "apuntes", "página", "páginas",
    "tabla", "tablas", "ejemplo", "ejemplos", "palabra", "palabras", "repaso", "error", "errores",
    "regla", "reglas", "concepto", "conceptos", "material", "contenido", "tema", "temas",
    "práctica", "practicamos", "trabajamos", "trabajó", "vimos", "hicimos", "vocabulario",
    "ejercicio", "ejercicios", "pizarra",
];

#[derive(Args)]
pub struct ConsolidateArgs {
    #[command(subcommand)]
    action: Option<ConsolidateAction>,

    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum ConsolidateAction {
    /// List the `ConsolidatedCandidate`s already persisted for a session.
    List(ListArgs),
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct SourceRefReport {
    origin: String,
    locator: String,
}

#[derive(Debug, Serialize)]
struct CandidateReport {
    id: String,
    text: String,
    already_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    existing_vocabulary_id: Option<String>,
    source_ref: SourceRefReport,
}

#[derive(Debug, Serialize)]
struct PendingItemReport {
    item_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize, Default)]
struct ConsolidateData {
    #[serde(skip_serializing_if = "Option::is_none")]
    candidates: Option<Vec<CandidateReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_items: Option<Vec<PendingItemReport>>,
}

pub fn run(args: ConsolidateArgs) -> i32 {
    if let Some(ConsolidateAction::List(list_args)) = args.action {
        return run_list(list_args);
    }
    run_consolidate(args.session, args.path, args.json)
}

fn run_consolidate(session: Option<String>, path: Option<PathBuf>, json: bool) -> i32 {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, session) {
        Ok(id) => id,
        Err(err) => return emit_error(json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    // T032 guard: any `analyse` item still `pending_user_decision` blocks
    // the whole run — candidates are derived from the combination of
    // summary + pages, and there is no clean way to consolidate only the
    // part of the session that does not depend on the still-missing
    // content (FR-013, Edge Case of spec.md).
    let checklist = match read_checklist(session_paths.root(), "analyse") {
        Ok(items) => items.unwrap_or_default(),
        Err(source) => {
            return emit_error(
                json,
                LearnKitError::Filesystem {
                    path: session_paths.root().display().to_string(),
                    source,
                },
            )
        }
    };
    let pending = blocking_pending_items(&checklist);
    if !pending.is_empty() {
        return emit_blocked(json, &pending);
    }

    let summary = match analysis::read_summary(&root, &session_id) {
        Ok(s) => s,
        Err(source) => {
            return emit_error(
                json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };
    let pages = match analysis::list_concept_pages(&root, &session_id) {
        Ok(p) => p,
        Err(source) => {
            return emit_error(
                json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let extracted = extract_candidates(summary.as_ref(), &pages);

    let mut candidates = Vec::with_capacity(extracted.len());
    for (index, item) in extracted.into_iter().enumerate() {
        let (already_exists, existing_vocabulary_id) = match find_by_lemma(&root, &item.text) {
            Ok(Some(entry)) => (true, Some(entry.id)),
            Ok(None) => (false, None),
            Err(source) => {
                return emit_error(
                    json,
                    LearnKitError::Filesystem {
                        path: root.display().to_string(),
                        source,
                    },
                )
            }
        };
        candidates.push(ConsolidatedCandidate {
            id: format!("cand-{}", index + 1),
            session_id: session_id.clone(),
            candidate_type: CandidateType::Vocabulary,
            text: item.text,
            source_ref: item.source_ref,
            already_exists,
            existing_vocabulary_id,
        });
    }

    if let Err(source) = consolidate::write_candidates(&root, &session_id, &candidates) {
        return emit_error(
            json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

    if let Err(err) = mark_consolidate_valid(&session_paths) {
        return emit_error(json, err);
    }

    emit_candidates(json, "CONSOLIDATE_DONE", &candidates)
}

fn run_list(args: ListArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    let candidates = match consolidate::list_candidates(&root, &session_id) {
        Ok(c) => c,
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

    emit_candidates(args.json, "CONSOLIDATE_LIST", &candidates)
}

struct ExtractedCandidate {
    text: String,
    source_ref: CandidateSourceRef,
}

/// Mechanical, deterministic candidate derivation (see module doc): scans
/// the confirmed `summary` first, then each `ConceptPage` in id order,
/// keeping the first origin a given word appears in for traceability and
/// deduplicating across both (FR-012, SC-005 — dedup against the session's
/// own material; dedup against previously-persisted vocabulary happens
/// separately, against `VocabularyEntry`, by the caller).
fn extract_candidates(
    summary: Option<&ClassSummary>,
    pages: &[ConceptPage],
) -> Vec<ExtractedCandidate> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    if let Some(summary) = summary {
        for word in candidate_words(&summary.content) {
            if seen.insert(word.clone()) {
                out.push(ExtractedCandidate {
                    source_ref: CandidateSourceRef {
                        origin: "summary".to_string(),
                        locator: word.clone(),
                    },
                    text: word,
                });
            }
        }
    }

    for page in pages {
        for word in candidate_words(&page.content) {
            if seen.insert(word.clone()) {
                out.push(ExtractedCandidate {
                    source_ref: CandidateSourceRef {
                        origin: page.id.clone(),
                        locator: word.clone(),
                    },
                    text: word,
                });
            }
        }
    }

    out
}

/// Splits `text` on non-alphanumeric boundaries, lowercases, drops tokens
/// shorter than 4 characters, purely numeric tokens, and stopwords, and
/// deduplicates within `text` itself — preserving first-appearance order.
fn candidate_words(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in text.split(|c: char| !c.is_alphanumeric()) {
        if raw.is_empty() || raw.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let word = raw.to_lowercase();
        if word.chars().count() < 4 {
            continue;
        }
        if STOPWORDS.contains(&word.as_str()) {
            continue;
        }
        if seen.insert(word.clone()) {
            out.push(word);
        }
    }
    out
}

/// Same non-checklist single-state manifest pattern already used by
/// `commands/inventory.rs` for a phase without a checklist of its own.
fn mark_consolidate_valid(session_paths: &SessionPaths) -> Result<(), LearnKitError> {
    let path = session_paths.phase_manifest("consolidate");
    let manifest = PhaseManifest {
        phase: "consolidate".to_string(),
        input_fingerprint: String::new(),
        output_fingerprint: String::new(),
        validated_at: format!("{:?}", SystemTime::now()),
        result: PhaseResult::Valid,
        checklist: None,
    };
    write_manifest(&path, &manifest).map_err(|source| LearnKitError::Filesystem {
        path: path.display().to_string(),
        source,
    })
}

fn to_report(candidate: &ConsolidatedCandidate) -> CandidateReport {
    CandidateReport {
        id: candidate.id.clone(),
        text: candidate.text.clone(),
        already_exists: candidate.already_exists,
        existing_vocabulary_id: candidate.existing_vocabulary_id.clone(),
        source_ref: SourceRefReport {
            origin: candidate.source_ref.origin.clone(),
            locator: candidate.source_ref.locator.clone(),
        },
    }
}

fn emit_candidates(json: bool, code: &str, candidates: &[ConsolidatedCandidate]) -> i32 {
    let reports: Vec<CandidateReport> = candidates.iter().map(to_report).collect();
    if json {
        Envelope::ok(
            code,
            ConsolidateData {
                candidates: Some(reports),
                pending_items: None,
            },
        )
        .print_json();
    } else if reports.is_empty() {
        println!("(sin candidatos consolidados)");
    } else {
        for c in &reports {
            match (c.already_exists, &c.existing_vocabulary_id) {
                (true, Some(existing_id)) => println!(
                    "  - {} \"{}\" [origen: {}] — ya existe en vocabulario ({})",
                    c.id, c.text, c.source_ref.origin, existing_id
                ),
                _ => println!(
                    "  - {} \"{}\" [origen: {}]",
                    c.id, c.text, c.source_ref.origin
                ),
            }
        }
        println!("{} candidato(s) consolidado(s).", reports.len());
    }
    0
}

/// FR-013: exit `20`, `code: "BLOCKED"` per `contracts/cli-commands.md` —
/// reuses `LearnKitError::GuardBlocked`'s exit-code tier (the existing
/// guard-failure tier, `docs/04-workflow-guards.md`) while keeping this
/// command's own JSON `code` literally `"BLOCKED"` as the contract
/// specifies, naming every still-pending item.
fn emit_blocked(json: bool, pending: &[ChecklistItemManifest]) -> i32 {
    let items: Vec<PendingItemReport> = pending
        .iter()
        .map(|item| PendingItemReport {
            item_id: item.item_id.clone(),
            reason: item.pending_reason.clone(),
        })
        .collect();

    if json {
        Envelope::err(
            "BLOCKED",
            ConsolidateData {
                candidates: None,
                pending_items: Some(items),
            },
        )
        .print_json();
    } else {
        eprintln!(
            "Error: consolidate bloqueado — elemento(s) de 'analyse' pendientes de decisión:"
        );
        for item in pending {
            match &item.pending_reason {
                Some(reason) => eprintln!("  - {}: {}", item.item_id, reason),
                None => eprintln!("  - {}", item.item_id),
            }
        }
    }

    LearnKitError::GuardBlocked {
        phase: "analyse".to_string(),
        reason: "uno o más elementos de 'analyse' siguen pending_user_decision".to_string(),
    }
    .exit_code()
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), ConsolidateData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_words_filters_short_numeric_and_stopwords() {
        let words = candidate_words("The itinerary and the layover, 1234, has 5 legs.");
        assert!(words.contains(&"itinerary".to_string()));
        assert!(words.contains(&"layover".to_string()));
        assert!(!words.contains(&"the".to_string()));
        assert!(!words.contains(&"has".to_string()));
        assert!(!words.contains(&"1234".to_string()));
    }

    #[test]
    fn candidate_words_deduplicates_case_insensitively() {
        let words = candidate_words("Itinerary itinerary ITINERARY");
        assert_eq!(words, vec!["itinerary".to_string()]);
    }
}
