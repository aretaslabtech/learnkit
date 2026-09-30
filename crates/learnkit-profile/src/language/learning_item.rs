use super::vocabulary::VocabularyEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Generic `LearningItem`, per `data-model.md` → LearningItem. Created either
/// from a `VocabularyEntry` (`ensure_for_vocabulary`, `kind = "vocabulary"`,
/// `vocabulary_entry_id: Some(..)`) or from any other source
/// (`ensure_generic` — grammar/dialogue/pronunciation/etc.,
/// `source_ref: Some(..)`) — the rest of the system (cards, assessment,
/// progress) only ever sees this generic shape, never `VocabularyEntry` or
/// the original source directly (Principle VI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub mastery_dimensions: Vec<String>,
    /// Only set for `kind = "vocabulary"` items (`ensure_for_vocabulary`).
    /// `Option` rather than `String` so a generic item (created via
    /// `ensure_generic`) can omit it — a legacy YAML file with this field as
    /// a plain string still deserializes fine into `Some(that string)`
    /// (aditivo, no migration needed).
    pub vocabulary_entry_id: Option<String>,
    /// Only set for a generic (non-vocabulary) item — `"{kind}:{session_id}:{source_id}"`,
    /// globally unique, used by `find_by_source_ref` for idempotent upserts.
    /// `#[serde(default)]` because it's absent from every YAML persisted
    /// before this field existed.
    #[serde(default)]
    pub source_ref: Option<String>,
}

fn learning_items_dir(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("learning-items")
}

fn item_path(project_root: &Path, id: &str) -> PathBuf {
    learning_items_dir(project_root).join(format!("{id}.yaml"))
}

/// Loads every persisted `LearningItem`, of any `kind` — vocabulary
/// (`ensure_for_vocabulary`) and generic (`ensure_generic`) items live side
/// by side in the same directory.
pub fn load_all(project_root: &Path) -> std::io::Result<Vec<LearningItem>> {
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
        if item.vocabulary_entry_id.as_deref() == Some(vocabulary_entry_id) {
            return Ok(Some(item));
        }
    }
    Ok(None)
}

/// Finds the persisted `LearningItem` whose `source_ref` matches exactly —
/// same traversal pattern as `find_by_vocabulary_entry`, used by
/// `ensure_generic` for idempotent upserts of non-vocabulary items.
pub fn find_by_source_ref(
    project_root: &Path,
    source_ref: &str,
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
        if item.source_ref.as_deref() == Some(source_ref) {
            return Ok(Some(item));
        }
    }
    Ok(None)
}

/// Finds the persisted `LearningItem` with this exact `id`, if any — used by
/// `learnkit cards set` to validate `--item` before upserting its
/// `CardSpec`.
pub fn find_by_id(project_root: &Path, id: &str) -> std::io::Result<Option<LearningItem>> {
    let path = item_path(project_root, id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let item: LearningItem = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(item))
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
        vocabulary_entry_id: Some(vocab.id.clone()),
        source_ref: None,
    };

    let yaml = serde_yaml::to_string(&item)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&item_path(project_root, &item.id), yaml.as_bytes())?;

    Ok(item)
}

/// Creates (or reuses, if one with this exact `source_ref` already exists)
/// a generic `LearningItem` for a non-vocabulary source — grammar
/// (`ConceptPage`), dialogue, or pronunciation minimal pair, per `learnkit
/// learn item promote`. Idempotent by `source_ref`, same pattern as
/// `ensure_for_vocabulary`/`find_by_vocabulary_entry`.
///
/// `id` is derived deterministically from `source_ref` (`li-<slug>`, same
/// `li-` prefix convention as vocabulary items) so calling this twice with
/// the same `source_ref` always targets the same file even before the
/// existence check runs.
pub fn ensure_generic(
    project_root: &Path,
    kind: &str,
    source_ref: &str,
    title: &str,
    summary: &str,
    tags: Vec<String>,
) -> std::io::Result<LearningItem> {
    if let Some(existing) = find_by_source_ref(project_root, source_ref)? {
        return Ok(existing);
    }

    let item = LearningItem {
        id: format!("li-{}", slugify(source_ref)),
        kind: kind.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        tags,
        mastery_dimensions: vec![
            "recognition".to_string(),
            "production".to_string(),
            "listening".to_string(),
        ],
        vocabulary_entry_id: None,
        source_ref: Some(source_ref.to_string()),
    };

    let yaml = serde_yaml::to_string(&item)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&item_path(project_root, &item.id), yaml.as_bytes())?;

    Ok(item)
}

/// Turns an arbitrary `source_ref` (e.g. `"grammar:sess-1:page:layover"`)
/// into a filesystem/id-safe slug: lowercase ASCII alphanumerics kept as-is,
/// every other byte collapsed to a single `-` (runs merged, no leading/
/// trailing `-`). Deterministic and stable across calls, which is all
/// `ensure_generic` needs — it does not need to be reversible.
fn slugify(source_ref: &str) -> String {
    let mut slug = String::with_capacity(source_ref.len());
    let mut last_was_dash = false;
    for ch in source_ref.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

/// Deletes the persisted `LearningItem` with this `id`, if any — FR-012c.
/// Also deletes its `CardSpec`, if any, so a generic (non-vocabulary) item
/// never leaves an orphaned `CardSpec` behind (same cascade shape as
/// `card.rs::remove` on vocabulary removal — `cardspec-generalization` T2).
/// Returns `false` (not an error) when the `LearningItem` itself didn't
/// exist; the `CardSpec` cascade runs regardless (there is nothing to
/// distinguish there — a vocabulary item simply never has one).
pub fn remove(project_root: &Path, id: &str) -> std::io::Result<bool> {
    crate::card_spec::remove(project_root, id)?;

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

        assert_eq!(item.vocabulary_entry_id.as_deref(), Some(vocab.id.as_str()));
        assert_eq!(item.source_ref, None);
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

    /// T2 (cardspec-generalization): removing a `LearningItem` also removes
    /// its `CardSpec`, if any — same cascade shape as `card.rs::remove` on
    /// vocabulary removal.
    #[test]
    fn removing_a_learning_item_cascades_to_its_card_spec() {
        let dir = tempfile::tempdir().unwrap();
        crate::card_spec::set(
            dir.path(),
            "li-generic-1",
            "activity",
            "stimulus",
            "response",
            None,
        )
        .unwrap();

        let removed = remove(dir.path(), "li-generic-1").unwrap();

        // The LearningItem YAML itself was never written in this test, so
        // `remove` correctly reports it wasn't there — but the cascade must
        // still have run.
        assert!(!removed);
        assert_eq!(
            crate::card_spec::find_by_learning_item(dir.path(), "li-generic-1").unwrap(),
            None
        );
    }

    #[test]
    fn find_by_id_returns_none_for_unknown_id() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(find_by_id(dir.path(), "does-not-exist").unwrap(), None);
    }

    #[test]
    fn ensure_generic_creates_a_new_non_vocabulary_item() {
        let dir = tempfile::tempdir().unwrap();

        let item = ensure_generic(
            dir.path(),
            "grammar",
            "grammar:sess-1:page:layover",
            "Layover (present perfect)",
            "Use the present perfect for...",
            vec!["grammar".to_string()],
        )
        .unwrap();

        assert_eq!(item.kind, "grammar");
        assert_eq!(item.title, "Layover (present perfect)");
        assert_eq!(item.vocabulary_entry_id, None);
        assert_eq!(
            item.source_ref.as_deref(),
            Some("grammar:sess-1:page:layover")
        );
        assert_eq!(item.mastery_dimensions.len(), 3);
        assert_eq!(item.tags, vec!["grammar".to_string()]);
    }

    #[test]
    fn ensure_generic_is_idempotent_by_source_ref() {
        let dir = tempfile::tempdir().unwrap();

        let first = ensure_generic(
            dir.path(),
            "dialogue",
            "dialogue:sess-1:dlg-1",
            "title one",
            "summary one",
            vec!["dialogue".to_string()],
        )
        .unwrap();

        // Same source_ref, different content supplied — must reuse the
        // existing item unchanged, never create a second one or overwrite it.
        let second = ensure_generic(
            dir.path(),
            "dialogue",
            "dialogue:sess-1:dlg-1",
            "title two",
            "summary two",
            vec!["dialogue".to_string()],
        )
        .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.title, "title one");

        let dir_entries: Vec<_> = fs::read_dir(learning_items_dir(dir.path()))
            .unwrap()
            .collect();
        assert_eq!(dir_entries.len(), 1);
    }

    #[test]
    fn find_by_source_ref_finds_the_matching_item() {
        let dir = tempfile::tempdir().unwrap();
        let created = ensure_generic(
            dir.path(),
            "pronunciation",
            "pronunciation:sess-1:mp-1",
            "sheets vs shits",
            "long vs short /i/",
            vec!["pronunciation".to_string()],
        )
        .unwrap();

        let found = find_by_source_ref(dir.path(), "pronunciation:sess-1:mp-1")
            .unwrap()
            .unwrap();
        assert_eq!(found.id, created.id);

        assert_eq!(
            find_by_source_ref(dir.path(), "pronunciation:sess-1:does-not-exist").unwrap(),
            None
        );
    }

    /// Regression: a `LearningItem` YAML persisted before `source_ref`
    /// existed, and before `vocabulary_entry_id` became `Option<String>`,
    /// has `vocabulary_entry_id` as a plain scalar string and no
    /// `source_ref` key at all — exactly the shape of David's ~127 real
    /// vocabulary cards. It must still deserialize, with
    /// `vocabulary_entry_id: Some(<that string>)` and `source_ref: None`.
    #[test]
    fn deserializes_legacy_yaml_with_plain_string_vocabulary_entry_id() {
        let legacy_yaml = "\
id: li-vocab-1
kind: vocabulary
title: whiteboard
summary: pizarra
tags:
  - vocabulary
  - en-GB
mastery_dimensions:
  - recognition
  - production
  - listening
vocabulary_entry_id: vocab-1
";

        let item: LearningItem = serde_yaml::from_str(legacy_yaml).unwrap();

        assert_eq!(item.vocabulary_entry_id.as_deref(), Some("vocab-1"));
        assert_eq!(item.source_ref, None);
    }
}
