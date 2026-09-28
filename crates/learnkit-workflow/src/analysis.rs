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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptPage {
    pub id: String,
    pub session_id: String,
    pub concept: String,
    pub content: String,
    pub source_fingerprint: String,
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

fn page_path(project_root: &Path, session_id: &str, id: &str) -> PathBuf {
    pages_dir(project_root, session_id).join(format!("{id}.yaml"))
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
/// `id` (`page-1`, `page-2`, ...).
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
    pages.sort_by(|a, b| page_number(&a.id).cmp(&page_number(&b.id)));
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

fn page_number(id: &str) -> u32 {
    id.strip_prefix("page-")
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Assigns the next sequential `page-<n>` id for this session and persists a
/// brand new `ConceptPage` — never overwrites an existing one. Each call
/// creates an independent checklist element (Edge Case of `spec.md`: pages
/// can be confirmed in any order, relative to each other and to
/// `summary`/`mindmap`).
pub fn add_concept_page(
    project_root: &Path,
    session_id: &str,
    concept: &str,
    content: &str,
    source_fingerprint: &str,
) -> std::io::Result<ConceptPage> {
    let existing = list_concept_pages(project_root, session_id)?;
    let next_n = existing.iter().map(|p| page_number(&p.id)).max().unwrap_or(0) + 1;

    let page = ConceptPage {
        id: format!("page-{next_n}"),
        session_id: session_id.to_string(),
        concept: concept.to_string(),
        content: content.to_string(),
        source_fingerprint: source_fingerprint.to_string(),
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

    #[test]
    fn concept_pages_get_sequential_stable_ids() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let page1 =
            add_concept_page(dir.path(), &session.id, "concepto A", "contenido A", "fp1").unwrap();
        let page2 =
            add_concept_page(dir.path(), &session.id, "concepto B", "contenido B", "fp1").unwrap();

        assert_eq!(page1.id, "page-1");
        assert_eq!(page2.id, "page-2");

        let pages = list_concept_pages(dir.path(), &session.id).unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].id, "page-1");
        assert_eq!(pages[1].id, "page-2");
    }
}
