use learnkit_core::error::LearnKitError;
use std::path::Path;

/// Resolves which session a command should operate on: the explicit
/// `--session` value if given, otherwise the project's active session
/// (`learnkit-store::active_session`, FR-003b). Mirrors how Spec Kit's own
/// `.specify/feature.json` resolves the active feature for this repository.
pub fn resolve_session(root: &Path, explicit: Option<String>) -> Result<String, LearnKitError> {
    if let Some(session_id) = explicit {
        return Ok(session_id);
    }

    match learnkit_store::active_session::get_active(root) {
        Ok(Some(session_id)) => Ok(session_id),
        Ok(None) => Err(LearnKitError::NoActiveSession),
        Err(source) => Err(LearnKitError::Filesystem {
            path: root.display().to_string(),
            source,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_session_wins_over_active() {
        let dir = tempfile::tempdir().unwrap();
        learnkit_store::active_session::set_active(dir.path(), "active-one").unwrap();

        let resolved = resolve_session(dir.path(), Some("explicit-one".to_string())).unwrap();
        assert_eq!(resolved, "explicit-one");
    }

    #[test]
    fn falls_back_to_active_session() {
        let dir = tempfile::tempdir().unwrap();
        learnkit_store::active_session::set_active(dir.path(), "active-one").unwrap();

        let resolved = resolve_session(dir.path(), None).unwrap();
        assert_eq!(resolved, "active-one");
    }

    #[test]
    fn errors_when_neither_is_available() {
        let dir = tempfile::tempdir().unwrap();
        let err = resolve_session(dir.path(), None).unwrap_err();
        assert!(matches!(err, LearnKitError::NoActiveSession));
    }
}
