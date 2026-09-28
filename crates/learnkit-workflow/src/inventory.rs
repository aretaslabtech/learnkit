use learnkit_store::session_paths::SessionPaths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Text,
    Pdf,
    Image,
    Audio,
    Other,
}

impl SourceKind {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "txt" => SourceKind::Text,
            "pdf" => SourceKind::Pdf,
            "png" | "jpg" | "jpeg" | "gif" | "webp" => SourceKind::Image,
            "wav" | "mp3" | "m4a" | "ogg" | "flac" => SourceKind::Audio,
            _ => SourceKind::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    /// Path relative to the session root (e.g. `input/notes.md`).
    pub path: String,
    pub sha256: String,
}

/// Result of a single source after running `inventory` — the transient
/// signal used to detect a content change (FR-003), never persisted as its
/// own field: the persisted truth is only the current `sha256`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryChange {
    Added,
    Unchanged,
    Updated,
}

#[derive(Debug, Clone)]
pub struct InventoriedSource {
    pub source: Source,
    pub change: InventoryChange,
}

fn inventory_index_path(paths: &SessionPaths) -> std::path::PathBuf {
    paths.inventory().join("sources.json")
}

fn load_recorded(paths: &SessionPaths) -> std::io::Result<HashMap<String, Source>> {
    let path = inventory_index_path(paths);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(path)?;
    let sources: Vec<Source> = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(sources.into_iter().map(|s| (s.path.clone(), s)).collect())
}

fn save_recorded(paths: &SessionPaths, sources: &[Source]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(sources)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&inventory_index_path(paths), json.as_bytes())
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Fingerprint of the whole inventoried set, used as the `inventory` phase's
/// output fingerprint (and as the `analyse` phase's input fingerprint in a
/// later story) — see `data-model.md` → WorkflowPhase/PhaseManifest.
pub fn sources_fingerprint<'a>(ids_and_hashes: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let mut pairs: Vec<String> = ids_and_hashes
        .map(|(id, sha256)| format!("{id}:{sha256}"))
        .collect();
    pairs.sort();
    content_hash_of_str(&pairs.join(","))
}

fn source_id_for(relative_path: &str) -> String {
    format!("src-{}", &content_hash_of_str(relative_path)[..8])
}

fn content_hash_of_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Copies `files` into the session's `input/` directory (FR-002 ingest step).
pub fn ingest(session_root: &Path, files: &[std::path::PathBuf]) -> std::io::Result<()> {
    let input_dir = session_root.join("input");
    fs::create_dir_all(&input_dir)?;
    for file in files {
        let file_name = file.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing file name")
        })?;
        fs::copy(file, input_dir.join(file_name))?;
    }
    Ok(())
}

/// Scans `session_root/input/` and (re)registers every file as a `Source`,
/// per FR-002/FR-003. Idempotent: re-running with no changes reports every
/// source as `Unchanged` and never duplicates an entry; a changed file is
/// reported as `Updated` with its new hash.
pub fn inventory(project_root: &Path, session_id: &str) -> std::io::Result<Vec<InventoriedSource>> {
    let paths = SessionPaths::new(project_root, session_id);
    let recorded = load_recorded(&paths)?;

    let mut results = Vec::new();
    let input_dir = paths.input();
    if input_dir.exists() {
        for entry in walkdir::WalkDir::new(&input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let absolute = entry.path();
            let relative = absolute
                .strip_prefix(paths.root())
                .unwrap_or(absolute)
                .to_string_lossy()
                .replace('\\', "/");

            let sha256 = hash_file(absolute)?;
            let extension = absolute.extension().and_then(|e| e.to_str()).unwrap_or("");
            let kind = SourceKind::from_extension(extension);

            let change = match recorded.get(&relative) {
                None => InventoryChange::Added,
                Some(existing) if existing.sha256 == sha256 => InventoryChange::Unchanged,
                Some(_) => InventoryChange::Updated,
            };

            let id = recorded
                .get(&relative)
                .map(|s| s.id.clone())
                .unwrap_or_else(|| source_id_for(&relative));

            results.push(InventoriedSource {
                source: Source {
                    id,
                    kind,
                    path: relative,
                    sha256,
                },
                change,
            });
        }
    }

    let to_persist: Vec<Source> = results.iter().map(|r| r.source.clone()).collect();
    save_recorded(&paths, &to_persist)?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::create_session;

    #[test]
    fn inventories_new_sources() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();
        let paths = SessionPaths::new(dir.path(), &session.id);
        fs::write(paths.input().join("notes.md"), "hello").unwrap();

        let result = inventory(dir.path(), &session.id).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].change, InventoryChange::Added);
        assert_eq!(result[0].source.kind, SourceKind::Text);
    }

    #[test]
    fn reinventorying_unchanged_source_reports_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();
        let paths = SessionPaths::new(dir.path(), &session.id);
        fs::write(paths.input().join("notes.md"), "hello").unwrap();

        inventory(dir.path(), &session.id).unwrap();
        let second = inventory(dir.path(), &session.id).unwrap();

        assert_eq!(second.len(), 1);
        assert_eq!(second[0].change, InventoryChange::Unchanged);
    }

    #[test]
    fn changed_content_is_reported_as_updated_with_new_hash() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();
        let paths = SessionPaths::new(dir.path(), &session.id);
        let note_path = paths.input().join("notes.md");
        fs::write(&note_path, "hello").unwrap();

        let first = inventory(dir.path(), &session.id).unwrap();
        let original_hash = first[0].source.sha256.clone();

        fs::write(&note_path, "hello world, changed!").unwrap();
        let second = inventory(dir.path(), &session.id).unwrap();

        assert_eq!(second[0].change, InventoryChange::Updated);
        assert_ne!(second[0].source.sha256, original_hash);
    }
}
