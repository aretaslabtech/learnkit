use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Generic card content for a `LearningItem` whose `kind` is not
/// `"vocabulary"` (grammar, dialogue, a C# enum, a Clean Code rubric,
/// geography, etc.) — `odd/tasks/cardspec-generalization.md`.
///
/// Vocabulary items keep sourcing their card text from `VocabularyEntry` via
/// `item.vocabulary_entry_id` (untouched); every other kind sources it from
/// here instead: `stimulus` is the front text (and the media search hint),
/// `response` is the back text, `feedback` is optional extra back text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardSpec {
    pub id: String,
    pub learning_item_id: String,
    pub activity: String,
    pub stimulus: String,
    pub response: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

fn card_specs_dir(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("card-specs")
}

fn card_spec_path(project_root: &Path, learning_item_id: &str) -> PathBuf {
    card_specs_dir(project_root).join(format!("{learning_item_id}.yaml"))
}

/// Finds the persisted `CardSpec` for this `learning_item_id`, if any.
pub fn find_by_learning_item(
    project_root: &Path,
    learning_item_id: &str,
) -> std::io::Result<Option<CardSpec>> {
    let path = card_spec_path(project_root, learning_item_id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let spec: CardSpec = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(spec))
}

/// Creates or replaces the `CardSpec` for `learning_item_id` — `learnkit
/// cards set` (upsert, one `CardSpec` per `LearningItem`, keyed by the
/// learning item's own id so `find_by_learning_item`/`remove` are O(1)).
#[allow(clippy::too_many_arguments)]
pub fn set(
    project_root: &Path,
    learning_item_id: &str,
    activity: &str,
    stimulus: &str,
    response: &str,
    feedback: Option<&str>,
) -> std::io::Result<CardSpec> {
    let spec = CardSpec {
        id: format!("cardspec-{learning_item_id}"),
        learning_item_id: learning_item_id.to_string(),
        activity: activity.to_string(),
        stimulus: stimulus.to_string(),
        response: response.to_string(),
        feedback: feedback.map(|s| s.to_string()),
    };

    let yaml = serde_yaml::to_string(&spec)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(
        &card_spec_path(project_root, learning_item_id),
        yaml.as_bytes(),
    )?;

    Ok(spec)
}

/// Deletes the persisted `CardSpec` for this `learning_item_id`, if any —
/// cascade for `learning_item::remove` (FR-012c pattern). Returns `false`
/// (not an error) when it didn't exist.
pub fn remove(project_root: &Path, learning_item_id: &str) -> std::io::Result<bool> {
    let path = card_spec_path(project_root, learning_item_id);
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
    fn set_creates_a_new_card_spec() {
        let dir = tempfile::tempdir().unwrap();
        let spec = set(
            dir.path(),
            "li-grammar-1",
            "fill-in-the-blank",
            "She ___ to school every day.",
            "goes",
            Some("Third person singular present takes -s."),
        )
        .unwrap();

        assert_eq!(spec.learning_item_id, "li-grammar-1");
        assert_eq!(spec.stimulus, "She ___ to school every day.");
        assert_eq!(spec.response, "goes");
        assert_eq!(
            spec.feedback.as_deref(),
            Some("Third person singular present takes -s.")
        );
    }

    #[test]
    fn set_is_an_upsert_keyed_by_learning_item_id() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "li-grammar-1",
            "fill-in-the-blank",
            "first stimulus",
            "first response",
            None,
        )
        .unwrap();

        let second = set(
            dir.path(),
            "li-grammar-1",
            "fill-in-the-blank",
            "second stimulus",
            "second response",
            None,
        )
        .unwrap();

        let found = find_by_learning_item(dir.path(), "li-grammar-1")
            .unwrap()
            .unwrap();
        assert_eq!(found.stimulus, "second stimulus");
        assert_eq!(found.id, second.id);
    }

    #[test]
    fn find_by_learning_item_returns_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            find_by_learning_item(dir.path(), "does-not-exist").unwrap(),
            None
        );
    }

    #[test]
    fn remove_deletes_an_existing_spec_and_reports_missing_as_false() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "li-grammar-1",
            "fill-in-the-blank",
            "stimulus",
            "response",
            None,
        )
        .unwrap();

        assert!(remove(dir.path(), "li-grammar-1").unwrap());
        assert_eq!(
            find_by_learning_item(dir.path(), "li-grammar-1").unwrap(),
            None
        );
        assert!(!remove(dir.path(), "li-grammar-1").unwrap());
    }
}
