use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Derived state of a single agent-integration file, recomputed from the
/// filesystem on every run (never cached) — see `data-model.md` → AgentFile
/// and the Hard Guards principle in `constitution.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFileState {
    /// The file does not exist yet: safe to install.
    Missing,
    /// The file matches the current template exactly: nothing to do.
    UpToDate,
    /// The file exists and its content does not match the current template:
    /// treated as user-modified and MUST NOT be overwritten (FR-008).
    ModifiedByUser,
}

pub fn hash_of(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Compares the file at `path` against `template_content` and returns its
/// derived state per `research.md` §5.
///
/// This crate does not yet track "stale but not user-modified" template
/// versions (no previous template hash history exists before this feature);
/// any on-disk content that does not match the current template hash is
/// conservatively treated as `ModifiedByUser`, which is the safe default —
/// it never overwrites unexpected content.
pub fn derive_state(path: &Path, template_content: &str) -> std::io::Result<AgentFileState> {
    if !path.exists() {
        return Ok(AgentFileState::Missing);
    }

    let on_disk = fs::read_to_string(path)?;
    if hash_of(&on_disk) == hash_of(template_content) {
        Ok(AgentFileState::UpToDate)
    } else {
        Ok(AgentFileState::ModifiedByUser)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");

        assert_eq!(
            derive_state(&path, "template").unwrap(),
            AgentFileState::Missing
        );
    }

    #[test]
    fn matching_content_is_up_to_date() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "template").unwrap();

        assert_eq!(
            derive_state(&path, "template").unwrap(),
            AgentFileState::UpToDate
        );
    }

    #[test]
    fn differing_content_is_modified_by_user() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "template\n<!-- nota personal -->").unwrap();

        assert_eq!(
            derive_state(&path, "template").unwrap(),
            AgentFileState::ModifiedByUser
        );
    }
}
