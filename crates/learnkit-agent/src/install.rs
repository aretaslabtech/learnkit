use crate::state::{derive_state, AgentFileState};
use crate::templates::templates_for;
use learnkit_core::atomic::write_atomic;
use std::path::Path;

/// Result of installing (or attempting to install) one agent's integration.
#[derive(Debug)]
pub struct InstallOutcome {
    pub agent_id: String,
    /// Files written or confirmed up-to-date.
    pub installed_files: Vec<String>,
    /// Files that exist with unexpected content and were left untouched.
    pub modified_files: Vec<String>,
}

impl InstallOutcome {
    pub fn is_blocked(&self) -> bool {
        !self.modified_files.is_empty()
    }
}

/// Installs `agent_id`'s integration files under `project_root`.
///
/// - `Missing`/`UpToDate` files are written (or left as-is) — see FR-002 (idempotent).
/// - `ModifiedByUser` files are never overwritten — see FR-008. They are
///   reported in `modified_files` so the caller can turn this into an
///   `AGENT_FILES_MODIFIED` error instead of silently succeeding.
///
/// Callers MUST validate `agent_id` beforehand (see `templates_for`).
pub fn install_agent(project_root: &Path, agent_id: &str) -> std::io::Result<InstallOutcome> {
    let mut outcome = InstallOutcome {
        agent_id: agent_id.to_string(),
        installed_files: Vec::new(),
        modified_files: Vec::new(),
    };

    for file in templates_for(agent_id) {
        let path = project_root.join(file.relative_path);

        match derive_state(&path, file.content)? {
            AgentFileState::ModifiedByUser => {
                outcome.modified_files.push(file.relative_path.to_string());
            }
            AgentFileState::Missing | AgentFileState::UpToDate => {
                write_atomic(&path, file.content.as_bytes())?;
                outcome.installed_files.push(file.relative_path.to_string());
            }
        }
    }

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn installs_clean_agent_files() {
        let dir = tempfile::tempdir().unwrap();

        let outcome = install_agent(dir.path(), "claude").unwrap();

        assert!(!outcome.is_blocked());
        assert!(dir.path().join("CLAUDE.md").exists());
        assert!(dir
            .path()
            .join(".claude/skills/learnkit-session/SKILL.md")
            .exists());
    }

    #[test]
    fn reinstalling_unchanged_files_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();

        install_agent(dir.path(), "claude").unwrap();
        let second = install_agent(dir.path(), "claude").unwrap();

        assert!(!second.is_blocked());
        // CLAUDE.md + learnkit-session + learnkit-language + learnkit-analyse
        // + learnkit-image-prompts + learnkit-cards (see `templates.rs`).
        assert_eq!(second.installed_files.len(), 6);
    }

    #[test]
    fn manually_modified_file_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        install_agent(dir.path(), "claude").unwrap();

        let claude_md = dir.path().join("CLAUDE.md");
        fs::write(&claude_md, "personal notes\n").unwrap();

        let outcome = install_agent(dir.path(), "claude").unwrap();

        assert!(outcome.is_blocked());
        assert_eq!(outcome.modified_files, vec!["CLAUDE.md".to_string()]);
        assert_eq!(fs::read_to_string(&claude_md).unwrap(), "personal notes\n");
    }
}
