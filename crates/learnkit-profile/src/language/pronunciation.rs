use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A pronunciation-contrast pair (e.g. "sheets" /ʃiːts/ vs "shits" /ʃɪts/) —
/// `odd/tasks/language-study-pack.md` T3. Persisted per-session, same shape
/// as `Dialogue`: pronunciation drills are tied to the session/class they
/// were raised in, not to project-wide vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinimalPair {
    pub id: String,
    pub session_id: String,
    pub word_a: String,
    pub ipa_a: String,
    pub word_b: String,
    pub ipa_b: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn pronunciation_dir(session_root: &Path) -> PathBuf {
    session_root.join("language").join("pronunciation")
}

fn pair_path(session_root: &Path, id: &str) -> PathBuf {
    pronunciation_dir(session_root).join(format!("{id}.yaml"))
}

/// Creates or replaces the `MinimalPair` with this `id` (upsert) — `learn
/// pronunciation set`.
#[allow(clippy::too_many_arguments)]
pub fn set(
    session_root: &Path,
    id: &str,
    session_id: &str,
    word_a: &str,
    ipa_a: &str,
    word_b: &str,
    ipa_b: &str,
    note: Option<&str>,
) -> std::io::Result<MinimalPair> {
    let pair = MinimalPair {
        id: id.to_string(),
        session_id: session_id.to_string(),
        word_a: word_a.to_string(),
        ipa_a: ipa_a.to_string(),
        word_b: word_b.to_string(),
        ipa_b: ipa_b.to_string(),
        note: note.map(|s| s.to_string()),
    };

    let yaml = serde_yaml::to_string(&pair)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&pair_path(session_root, id), yaml.as_bytes())?;

    Ok(pair)
}

/// Loads every persisted `MinimalPair` for this session.
pub fn load_all(session_root: &Path) -> std::io::Result<Vec<MinimalPair>> {
    let dir = pronunciation_dir(session_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut pairs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let pair: MinimalPair = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        pairs.push(pair);
    }
    Ok(pairs)
}

/// Deletes the persisted `MinimalPair` with this `id`, if any. Returns
/// `false` (not an error) when it didn't exist.
pub fn remove(session_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = pair_path(session_root, id);
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
    fn set_creates_a_new_minimal_pair() {
        let dir = tempfile::tempdir().unwrap();
        let pair = set(
            dir.path(),
            "mp-1",
            "session-1",
            "sheets",
            "ʃiːts",
            "shits",
            "ʃɪts",
            Some("long vs short /i/"),
        )
        .unwrap();

        assert_eq!(pair.word_a, "sheets");
        assert_eq!(pair.ipa_a, "ʃiːts");
        assert_eq!(pair.word_b, "shits");
        assert_eq!(pair.note.as_deref(), Some("long vs short /i/"));
    }

    #[test]
    fn set_is_an_upsert_keyed_by_id() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "mp-1",
            "session-1",
            "ship",
            "ʃɪp",
            "sheep",
            "ʃiːp",
            None,
        )
        .unwrap();

        set(
            dir.path(),
            "mp-1",
            "session-1",
            "live",
            "lɪv",
            "leave",
            "liːv",
            None,
        )
        .unwrap();

        let all = load_all(dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].word_a, "live");
    }

    #[test]
    fn remove_deletes_an_existing_pair_and_reports_missing_as_false() {
        let dir = tempfile::tempdir().unwrap();
        set(
            dir.path(),
            "mp-1",
            "session-1",
            "ship",
            "ʃɪp",
            "sheep",
            "ʃiːp",
            None,
        )
        .unwrap();

        assert!(remove(dir.path(), "mp-1").unwrap());
        assert!(load_all(dir.path()).unwrap().is_empty());
        assert!(!remove(dir.path(), "mp-1").unwrap());
    }

    #[test]
    fn load_all_returns_empty_when_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_all(dir.path()).unwrap().is_empty());
    }
}
