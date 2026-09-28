use super::vocabulary::VocabularyEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Generic `LearningItem`, per `data-model.md` → LearningItem. `kind` is
/// always `"vocabulary"` for entries created from this module — the rest of
/// the system (cards, assessment, progress) only ever sees this generic
/// shape, never `VocabularyEntry` directly (Principle VI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub mastery_dimensions: Vec<String>,
    pub vocabulary_entry_id: String,
}

fn learning_items_dir(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("learning-items")
}

fn item_path(project_root: &Path, id: &str) -> PathBuf {
    learning_items_dir(project_root).join(format!("{id}.yaml"))
}

/// Loads every persisted `LearningItem` (all created via this module are
/// `kind = "vocabulary"` — see `ensure_for_vocabulary`).
pub fn load_all_vocabulary_items(project_root: &Path) -> std::io::Result<Vec<LearningItem>> {
    let dir = learning_items_dir(project_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let item: LearningItem = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        items.push(item);
    }
    Ok(items)
}

pub fn find_by_vocabulary_entry(
    project_root: &Path,
    vocabulary_entry_id: &str,
) -> std::io::Result<Option<LearningItem>> {
    let dir = learning_items_dir(project_root);
    if !dir.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let item: LearningItem = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        if item.vocabulary_entry_id == vocabulary_entry_id {
            return Ok(Some(item));
        }
    }
    Ok(None)
}

/// Creates (or reuses, if one already exists) the `LearningItem` linked to
/// `vocab` — FR-010.
pub fn ensure_for_vocabulary(
    project_root: &Path,
    vocab: &VocabularyEntry,
) -> std::io::Result<LearningItem> {
    if let Some(existing) = find_by_vocabulary_entry(project_root, &vocab.id)? {
        return Ok(existing);
    }

    let item = LearningItem {
        id: format!("li-{}", vocab.id),
        kind: "vocabulary".to_string(),
        title: vocab.lemma.clone(),
        summary: vocab
            .senses
            .first()
            .map(|s| s.gloss.clone())
            .unwrap_or_default(),
        tags: vec!["vocabulary".to_string(), vocab.language.clone()],
        mastery_dimensions: vec![
            "recognition".to_string(),
            "production".to_string(),
            "listening".to_string(),
        ],
        vocabulary_entry_id: vocab.id.clone(),
    };

    let yaml = serde_yaml::to_string(&item)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&item_path(project_root, &item.id), yaml.as_bytes())?;

    Ok(item)
}

/// Deletes the persisted `LearningItem` with this `id`, if any — FR-012c.
/// Returns `false` (not an error) when it didn't exist.
pub fn remove(project_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = item_path(project_root, id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::vocabulary::add_or_reuse;

    #[test]
    fn creates_a_learning_item_linked_to_the_vocabulary_entry() {
        let dir = tempfile::tempdir().unwrap();
        let vocab = add_or_reuse(
            dir.path(),
            "get away with",
            "gloss",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();

        let item = ensure_for_vocabulary(dir.path(), &vocab).unwrap();

        assert_eq!(item.vocabulary_entry_id, vocab.id);
        assert_eq!(item.kind, "vocabulary");
        assert_eq!(item.mastery_dimensions.len(), 3);
    }

    #[test]
    fn calling_twice_reuses_the_same_learning_item() {
        let dir = tempfile::tempdir().unwrap();
        let vocab = add_or_reuse(
            dir.path(),
            "whiteboard",
            "pizarra",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();

        let first = ensure_for_vocabulary(dir.path(), &vocab).unwrap();
        let second = ensure_for_vocabulary(dir.path(), &vocab).unwrap();

        assert_eq!(first.id, second.id);
    }
}
