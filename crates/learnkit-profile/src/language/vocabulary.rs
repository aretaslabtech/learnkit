use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sense {
    pub gloss: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabularyEntry {
    pub id: String,
    pub language: String,
    pub variety: String,
    pub lemma: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of_speech: Option<String>,
    pub senses: Vec<Sense>,
    pub sources: Vec<SourceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_by: Option<String>,
}

fn vocabulary_dir(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("vocabulary")
}

fn normalize_lemma(lemma: &str) -> String {
    lemma.trim().to_lowercase()
}

fn slug_id(lemma: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for ch in normalize_lemma(lemma).chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(lemma.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    format!("vocab-en-{slug}-{}", &hash[..6])
}

fn load_all(project_root: &Path) -> std::io::Result<Vec<VocabularyEntry>> {
    let dir = vocabulary_dir(project_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let vocab: VocabularyEntry = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        entries.push(vocab);
    }
    Ok(entries)
}

fn save(project_root: &Path, entry: &VocabularyEntry) -> std::io::Result<()> {
    let path = vocabulary_dir(project_root).join(format!("{}.yaml", entry.id));
    let yaml = serde_yaml::to_string(entry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())
}

/// Finds an existing entry with the same normalized `lemma`, project-wide
/// (vocabulary survives across sessions — FR-011).
pub fn find_by_lemma(project_root: &Path, lemma: &str) -> std::io::Result<Option<VocabularyEntry>> {
    let target = normalize_lemma(lemma);
    Ok(load_all(project_root)?
        .into_iter()
        .find(|e| normalize_lemma(&e.lemma) == target))
}

/// Creates a new `VocabularyEntry`, or reuses (and extends the source
/// traceability of) an existing one with the same `lemma` — FR-008/FR-009.
pub fn add_or_reuse(
    project_root: &Path,
    lemma: &str,
    gloss: &str,
    source_id: &str,
    locator: Option<&str>,
    suggested_by: Option<&str>,
    variety: &str,
) -> std::io::Result<VocabularyEntry> {
    let new_source = SourceRef {
        source_id: source_id.to_string(),
        locator: locator.map(|s| s.to_string()),
    };

    if let Some(mut existing) = find_by_lemma(project_root, lemma)? {
        if !existing.sources.contains(&new_source) {
            existing.sources.push(new_source);
            save(project_root, &existing)?;
        }
        return Ok(existing);
    }

    let entry = VocabularyEntry {
        id: slug_id(lemma),
        language: "en".to_string(),
        variety: variety.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: None,
        senses: vec![Sense {
            gloss: gloss.to_string(),
        }],
        sources: vec![new_source],
        suggested_by: suggested_by.map(|s| s.to_string()),
    };
    save(project_root, &entry)?;
    Ok(entry)
}

/// Deletes the persisted `VocabularyEntry` with this `id`, if any —
/// FR-012c. Returns `false` (not an error) when it didn't exist, so a
/// caller can distinguish "already gone" from a real filesystem failure.
pub fn remove(project_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = vocabulary_dir(project_root).join(format!("{id}.yaml"));
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_new_entry_with_traceability() {
        let dir = tempfile::tempdir().unwrap();
        let entry = add_or_reuse(
            dir.path(),
            "get away with",
            "hacer algo malo sin castigo",
            "src-transcript-1",
            Some("segment:00:12:34"),
            Some("agent"),
            "en-GB",
        )
        .unwrap();

        assert_eq!(entry.lemma, "get away with");
        assert_eq!(entry.sources.len(), 1);
        assert_eq!(entry.suggested_by, Some("agent".to_string()));
    }

    #[test]
    fn reuses_entry_with_same_lemma_and_adds_new_source() {
        let dir = tempfile::tempdir().unwrap();
        let first = add_or_reuse(
            dir.path(),
            "get away with",
            "gloss",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();
        let second = add_or_reuse(
            dir.path(),
            "Get Away With",
            "gloss",
            "src-2",
            None,
            None,
            "en-GB",
        )
        .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.sources.len(), 2);
    }

    #[test]
    fn manual_entry_without_any_suggestion_works() {
        let dir = tempfile::tempdir().unwrap();
        let entry = add_or_reuse(
            dir.path(),
            "whiteboard",
            "pizarra",
            "src-notes",
            None,
            None,
            "en-GB",
        )
        .unwrap();
        assert_eq!(entry.suggested_by, None);
    }
}
