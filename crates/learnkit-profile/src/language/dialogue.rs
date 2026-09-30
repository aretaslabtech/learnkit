use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Where a `Dialogue`'s lines came from — `odd/tasks/language-study-pack.md`
/// T2. `FromClass` means every line is a verbatim (or lightly cleaned)
/// transcript of something the teacher/class actually said; `AddedForConsolidation`
/// means the agent (or the user) authored it afterwards to drill a pattern —
/// never presented as something the teacher said. Serialized in snake_case.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DialogueOrigin {
    FromClass,
    AddedForConsolidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
}

/// A short spoken exchange, persisted per-session (dialogues are tied to the
/// class/session they came from or were written to reinforce, unlike
/// `VocabularyEntry` which survives across sessions).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dialogue {
    pub id: String,
    pub session_id: String,
    pub origin: DialogueOrigin,
    pub lines: Vec<DialogueLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn dialogues_dir(session_root: &Path) -> PathBuf {
    session_root.join("language").join("dialogues")
}

fn dialogue_path(session_root: &Path, id: &str) -> PathBuf {
    dialogues_dir(session_root).join(format!("{id}.yaml"))
}

/// Creates or replaces the `Dialogue` with this `id` (upsert) — `learn
/// dialogue set`. Rejects fewer than 2 lines: a single-line "dialogue" isn't
/// one.
pub fn set(
    session_root: &Path,
    id: &str,
    session_id: &str,
    origin: DialogueOrigin,
    lines: Vec<DialogueLine>,
    note: Option<&str>,
) -> std::io::Result<Dialogue> {
    if lines.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "a dialogue needs at least 2 lines, got {} — pass --line at least twice",
                lines.len()
            ),
        ));
    }

    let dialogue = Dialogue {
        id: id.to_string(),
        session_id: session_id.to_string(),
        origin,
        lines,
        note: note.map(|s| s.to_string()),
    };

    let yaml = serde_yaml::to_string(&dialogue)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&dialogue_path(session_root, id), yaml.as_bytes())?;

    Ok(dialogue)
}

/// Loads every persisted `Dialogue` for this session.
pub fn load_all(session_root: &Path) -> std::io::Result<Vec<Dialogue>> {
    let dir = dialogues_dir(session_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut dialogues = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let dialogue: Dialogue = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        dialogues.push(dialogue);
    }
    Ok(dialogues)
}

/// Deletes the persisted `Dialogue` with this `id`, if any. Returns `false`
/// (not an error) when it didn't exist.
pub fn remove(session_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = dialogue_path(session_root, id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lines() -> Vec<DialogueLine> {
        vec![
            DialogueLine {
                speaker: "A".to_string(),
                text: "Hello".to_string(),
            },
            DialogueLine {
                speaker: "B".to_string(),
                text: "Hi".to_string(),
            },
        ]
    }

    #[test]
    fn set_creates_a_new_dialogue() {
        let dir = tempfile::tempdir().unwrap();
        let dialogue = set(
            dir.path(),
            "dlg-1",
            "session-1",
            DialogueOrigin::FromClass,
            sample_lines(),
            Some("greeting exchange"),
        )
        .unwrap();

        assert_eq!(dialogue.id, "dlg-1");
        assert_eq!(dialogue.lines.len(), 2);
        assert_eq!(dialogue.origin, DialogueOrigin::FromClass);
        assert_eq!(dialogue.note.as_deref(), Some("greeting exchange"));
    }

    #[test]
    fn set_is_an_upsert_keyed_by_id() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "dlg-1",
            "session-1",
            DialogueOrigin::FromClass,
            sample_lines(),
            None,
        )
        .unwrap();

        let updated = set(
            dir.path(),
            "dlg-1",
            "session-1",
            DialogueOrigin::AddedForConsolidation,
            vec![
                DialogueLine {
                    speaker: "A".to_string(),
                    text: "Good morning".to_string(),
                },
                DialogueLine {
                    speaker: "B".to_string(),
                    text: "Good morning to you too".to_string(),
                },
            ],
            None,
        )
        .unwrap();

        let all = load_all(dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].origin, DialogueOrigin::AddedForConsolidation);
        assert_eq!(updated.lines[0].text, "Good morning");
    }

    #[test]
    fn set_rejects_fewer_than_two_lines() {
        let dir = tempfile::tempdir().unwrap();
        let err = set(
            dir.path(),
            "dlg-1",
            "session-1",
            DialogueOrigin::FromClass,
            vec![DialogueLine {
                speaker: "A".to_string(),
                text: "Hello".to_string(),
            }],
            None,
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn remove_deletes_an_existing_dialogue_and_reports_missing_as_false() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "dlg-1",
            "session-1",
            DialogueOrigin::FromClass,
            sample_lines(),
            None,
        )
        .unwrap();

        assert!(remove(dir.path(), "dlg-1").unwrap());
        assert!(load_all(dir.path()).unwrap().is_empty());
        assert!(!remove(dir.path(), "dlg-1").unwrap());
    }

    #[test]
    fn load_all_returns_empty_when_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_all(dir.path()).unwrap().is_empty());
    }
}
