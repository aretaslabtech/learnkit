use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Mirrors Spec Kit's own `.specify/feature.json` pointer: which session a
/// project-scoped command should operate on when `--session` is omitted.
/// Machine-local state, not meant to be the source of truth for anything —
/// re-derivable at any time by running `learnkit session use <id>` again.
#[derive(Debug, Serialize, Deserialize)]
struct ActiveSession {
    session_id: String,
}

fn active_session_path(project_root: &Path) -> PathBuf {
    project_root.join(".learnkit").join("active-session.json")
}

pub fn set_active(project_root: &Path, session_id: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(&ActiveSession {
        session_id: session_id.to_string(),
    })
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&active_session_path(project_root), json.as_bytes())
}

pub fn get_active(project_root: &Path) -> std::io::Result<Option<String>> {
    let path = active_session_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)?;
    let active: ActiveSession = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(active.session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_active_session_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(get_active(dir.path()).unwrap(), None);
    }

    #[test]
    fn set_then_get_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        set_active(dir.path(), "session-1").unwrap();
        assert_eq!(
            get_active(dir.path()).unwrap(),
            Some("session-1".to_string())
        );
    }

    #[test]
    fn setting_again_overwrites_the_previous_active_session() {
        let dir = tempfile::tempdir().unwrap();
        set_active(dir.path(), "session-1").unwrap();
        set_active(dir.path(), "session-2").unwrap();
        assert_eq!(
            get_active(dir.path()).unwrap(),
            Some("session-2".to_string())
        );
    }
}
