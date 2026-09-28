//! `analyse` phase content entities (US2, feature 003):
//! `ClassSummary`/`StudyMap`/`ConceptPage`, per `data-model.md`. Generic
//! (no language/subject concepts — Principio VI): the CLI validates
//! structure/traceability, never prose quality; the actual redaction always
//! comes from an agent via the `learnkit-analyse` Skill (`research.md` §2).
//!
//! Persisted as YAML under `SessionPaths::analysis()`, same
//! read/write-one-file-per-entity pattern already used by
//! `learnkit-profile::language::vocabulary` for `VocabularyEntry`.

use learnkit_store::session_paths::SessionPaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A concept the material mentioned but never explained, filled in by the
/// redacting agent from its own knowledge (FR-006) — kept as a distinct,
/// traceable field rather than mixed into `content`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilledGap {
    pub concept: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassSummary {
    pub session_id: String,
    pub content: String,
    #[serde(default)]
    pub filled_gaps: Vec<FilledGap>,
    pub source_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StudyMap {
    pub session_id: String,
    pub content: String,
    pub source_fingerprint: String,
    /// Generalized from `ClassSummary` (FR-018, feature 003 Phase 10):
    /// `analyse set --item mindmap` accepts `--filled-gap` too now.
    #[serde(default)]
    pub filled_gaps: Vec<FilledGap>,
}

/// A concept page's stable, agent-facing identity is `id` (e.g.
/// `"page:layover"`) — never positional, never derived from `concept` (the
/// editable display name) or from filesystem/persistence order (FR-018).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptPage {
    pub id: String,
    pub session_id: String,
    pub concept: String,
    pub content: String,
    pub source_fingerprint: String,
    /// Generalized from `ClassSummary` (FR-018).
    #[serde(default)]
    pub filled_gaps: Vec<FilledGap>,
    /// Explicit creation/persistence order (FR-018) — consumers that need a
    /// stable rendering order (e.g. `export study-guide`) sort by this field
    /// instead of parsing `id`, which is now an arbitrary agent-chosen slug,
    /// not a sequence number. Assigned once, at creation, and never changed
    /// by a later `analyse set` update to the same page.
    #[serde(default)]
    pub order: u64,
}

fn analysis_dir(project_root: &Path, session_id: &str) -> PathBuf {
    SessionPaths::new(project_root, session_id).analysis()
}

fn summary_path(project_root: &Path, session_id: &str) -> PathBuf {
    analysis_dir(project_root, session_id).join("summary.yaml")
}

fn study_map_path(project_root: &Path, session_id: &str) -> PathBuf {
    analysis_dir(project_root, session_id).join("mindmap.yaml")
}

fn pages_dir(project_root: &Path, session_id: &str) -> PathBuf {
    analysis_dir(project_root, session_id).join("pages")
}

/// A `ConceptPage`'s logical `id` (e.g. `"page:layover"`) can contain `:`,
/// which is not a legal character in a Windows filename. The on-disk
/// filename is a separate, sanitized encoding — the real `id` string is
/// always the one stored inside the YAML (and used everywhere in the
/// CLI/checklist), never reconstructed from the filename.
fn sanitize_filename_component(id: &str) -> String {
    id.replace(':', "_")
}

fn page_path(project_root: &Path, session_id: &str, id: &str) -> PathBuf {
    pages_dir(project_root, session_id).join(format!("{}.yaml", sanitize_filename_component(id)))
}

fn to_io_err(err: serde_yaml::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
}

pub fn write_summary(project_root: &Path, summary: &ClassSummary) -> std::io::Result<()> {
    let path = summary_path(project_root, &summary.session_id);
    let yaml = serde_yaml::to_string(summary).map_err(to_io_err)?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())
}

pub fn read_summary(
    project_root: &Path,
    session_id: &str,
) -> std::io::Result<Option<ClassSummary>> {
    let path = summary_path(project_root, session_id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let summary: ClassSummary = serde_yaml::from_str(&raw).map_err(to_io_err)?;
    Ok(Some(summary))
}

pub fn write_study_map(project_root: &Path, map: &StudyMap) -> std::io::Result<()> {
    let path = study_map_path(project_root, &map.session_id);
    let yaml = serde_yaml::to_string(map).map_err(to_io_err)?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())
}

pub fn read_study_map(project_root: &Path, session_id: &str) -> std::io::Result<Option<StudyMap>> {
    let path = study_map_path(project_root, session_id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let map: StudyMap = serde_yaml::from_str(&raw).map_err(to_io_err)?;
    Ok(Some(map))
}

/// Lists every `ConceptPage` already persisted for this session, sorted by
/// its explicit creation/persistence `order` (FR-018) — never by parsing
/// `id`, which since Phase 10 is an arbitrary agent-chosen stable slug
/// (`page:layover`), not a sequence number.
pub fn list_concept_pages(
    project_root: &Path,
    session_id: &str,
) -> std::io::Result<Vec<ConceptPage>> {
    let dir = pages_dir(project_root, session_id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut pages = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let page: ConceptPage = serde_yaml::from_str(&raw).map_err(to_io_err)?;
        pages.push(page);
    }
    pages.sort_by_key(|p| p.order);
    Ok(pages)
}

pub fn read_concept_page(
    project_root: &Path,
    session_id: &str,
    id: &str,
) -> std::io::Result<Option<ConceptPage>> {
    let path = page_path(project_root, session_id, id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let page: ConceptPage = serde_yaml::from_str(&raw).map_err(to_io_err)?;
    Ok(Some(page))
}

/// Creates or updates the `ConceptPage` identified by `id` (e.g.
/// `"page:layover"`, stable across updates — FR-018): assigns the next
/// `order` value (see `ConceptPage::order`) on first creation, and reuses
/// the existing `order` on every later update to the same `id` — the same
/// page is updated in place, never a new `page:<id>-2`. `concept` must
/// already be resolved by the caller (`analyse set` requires `--concept` on
/// first creation and falls back to the existing display name otherwise —
/// `data-model.md`/FR-018 leave that decision to the CLI layer, not this
/// storage function).
pub fn upsert_concept_page(
    project_root: &Path,
    session_id: &str,
    id: &str,
    concept: &str,
    content: &str,
    source_fingerprint: &str,
    filled_gaps: Vec<FilledGap>,
) -> std::io::Result<ConceptPage> {
    let existing = read_concept_page(project_root, session_id, id)?;
    let order = match &existing {
        Some(p) => p.order,
        None => {
            let pages = list_concept_pages(project_root, session_id)?;
            pages.iter().map(|p| p.order).max().unwrap_or(0) + 1
        }
    };

    let page = ConceptPage {
        id: id.to_string(),
        session_id: session_id.to_string(),
        concept: concept.to_string(),
        content: content.to_string(),
        source_fingerprint: source_fingerprint.to_string(),
        filled_gaps,
        order,
    };

    let path = page_path(project_root, session_id, &page.id);
    let yaml = serde_yaml::to_string(&page).map_err(to_io_err)?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())?;
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::create_session;

    #[test]
    fn summary_round_trips_with_filled_gaps() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let summary = ClassSummary {
            session_id: session.id.clone(),
            content: "Resumen de la clase.".to_string(),
            filled_gaps: vec![FilledGap {
                concept: "present perfect of unfinished time periods".to_string(),
                note: "regla añadida por el agente".to_string(),
            }],
            source_fingerprint: "fp1".to_string(),
        };
        write_summary(dir.path(), &summary).unwrap();

        let read_back = read_summary(dir.path(), &session.id).unwrap().unwrap();
        assert_eq!(read_back, summary);
    }

    #[test]
    fn study_map_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let map = StudyMap {
            session_id: session.id.clone(),
            content: "- nodo 1\n  - nodo 1.1".to_string(),
            source_fingerprint: "fp1".to_string(),
            filled_gaps: Vec::new(),
        };
        write_study_map(dir.path(), &map).unwrap();

        let read_back = read_study_map(dir.path(), &session.id).unwrap().unwrap();
        assert_eq!(read_back, map);
    }

    #[test]
    fn missing_entities_read_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        assert_eq!(read_summary(dir.path(), &session.id).unwrap(), None);
        assert_eq!(read_study_map(dir.path(), &session.id).unwrap(), None);
    }

    // --- FR-018 (Phase 10): stable `page:<id>` identity, not positional ---

    #[test]
    fn concept_pages_get_stable_agent_chosen_ids_in_creation_order() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let page1 = upsert_concept_page(
            dir.path(),
            &session.id,
            "page:layover",
            "Layover",
            "contenido A",
            "fp1",
            Vec::new(),
        )
        .unwrap();
        let page2 = upsert_concept_page(
            dir.path(),
            &session.id,
            "page:present-perfect",
            "Present perfect",
            "contenido B",
            "fp1",
            Vec::new(),
        )
        .unwrap();

        assert_eq!(page1.id, "page:layover");
        assert_eq!(page2.id, "page:present-perfect");
        assert!(page1.order < page2.order);

        let pages = list_concept_pages(dir.path(), &session.id).unwrap();
        assert_eq!(pages.len(), 2);
        // list_concept_pages sorts by creation order, not by id.
        assert_eq!(pages[0].id, "page:layover");
        assert_eq!(pages[1].id, "page:present-perfect");
    }

    #[test]
    fn upserting_an_existing_page_id_updates_it_in_place_without_creating_another() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let created = upsert_concept_page(
            dir.path(),
            &session.id,
            "page:layover",
            "Layover",
            "contenido v1",
            "fp1",
            Vec::new(),
        )
        .unwrap();
        let updated = upsert_concept_page(
            dir.path(),
            &session.id,
            "page:layover",
            "Layovers and connections",
            "contenido v2",
            "fp2",
            Vec::new(),
        )
        .unwrap();

        // Same identity, same creation order — content/concept/fingerprint change.
        assert_eq!(created.id, updated.id);
        assert_eq!(created.order, updated.order);
        assert_eq!(updated.concept, "Layovers and connections");
        assert_eq!(updated.content, "contenido v2");

        let pages = list_concept_pages(dir.path(), &session.id).unwrap();
        assert_eq!(pages.len(), 1, "update must not create a second page");
    }

    #[test]
    fn page_id_containing_colon_round_trips_despite_windows_filename_restriction() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        upsert_concept_page(
            dir.path(),
            &session.id,
            "page:layover",
            "Layover",
            "contenido",
            "fp1",
            Vec::new(),
        )
        .unwrap();

        let read_back = read_concept_page(dir.path(), &session.id, "page:layover")
            .unwrap()
            .expect("page readable by its real, colon-containing id");
        assert_eq!(read_back.id, "page:layover");
    }
}
