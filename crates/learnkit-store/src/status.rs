use crate::project::read_project;
use learnkit_agent::catalog::SUPPORTED_AGENTS;
use learnkit_agent::state::{derive_state, AgentFileState};
use learnkit_agent::templates::templates_for;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Issue {
    pub check: String,
    pub message: String,
}

/// `StatusReport` per `data-model.md` — always recomputed from the
/// filesystem, never cached (Hard Guards principle).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct StatusReport {
    pub ok: bool,
    pub code: String,
    pub profile_id: Option<String>,
    pub installed_agents: Vec<String>,
    pub shell_preference: Option<String>,
    pub issues: Vec<Issue>,
}

/// Builds a `StatusReport` by inspecting `root` right now.
pub fn build_status(root: &Path) -> std::io::Result<StatusReport> {
    let project = match read_project(root)? {
        None => {
            return Ok(StatusReport {
                ok: false,
                code: "PROJECT_NOT_INITIALIZED".to_string(),
                profile_id: None,
                installed_agents: Vec::new(),
                shell_preference: None,
                issues: Vec::new(),
            })
        }
        Some(project) => project,
    };

    let mut issues = Vec::new();
    if !learnkit_profile::catalog::is_supported(&project.profile_id) {
        issues.push(Issue {
            check: "profile_supported".to_string(),
            message: format!(
                "profile '{}' is not a supported profile",
                project.profile_id
            ),
        });
    }

    let installed_agents = SUPPORTED_AGENTS
        .iter()
        .filter(|agent_id| agent_is_installed(root, agent_id))
        .map(|s| s.to_string())
        .collect();

    let ok = issues.is_empty();
    let code = if ok {
        "PROJECT_READY".to_string()
    } else {
        "PROFILE_INVALID".to_string()
    };

    Ok(StatusReport {
        ok,
        code,
        profile_id: Some(project.profile_id),
        installed_agents,
        shell_preference: Some(project.shell_preference),
        issues,
    })
}

/// An agent is considered installed when every one of its integration files
/// is present on disk, regardless of whether the content still matches the
/// template (a manually edited file is still an installed integration).
fn agent_is_installed(root: &Path, agent_id: &str) -> bool {
    templates_for(agent_id).into_iter().all(|file| {
        let path = root.join(file.relative_path);
        !matches!(
            derive_state(&path, file.content),
            Ok(AgentFileState::Missing) | Err(_)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init::scaffold_project;
    use learnkit_agent::install::install_agent;

    #[test]
    fn uninitialized_project_reports_not_initialized() {
        let dir = tempfile::tempdir().unwrap();
        let status = build_status(dir.path()).unwrap();

        assert!(!status.ok);
        assert_eq!(status.code, "PROJECT_NOT_INITIALIZED");
        assert_eq!(status.installed_agents, Vec::<String>::new());
    }

    #[test]
    fn initialized_project_with_agent_reports_ready() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_project(dir.path(), "geography", "sh").unwrap();
        install_agent(dir.path(), "claude").unwrap();

        let status = build_status(dir.path()).unwrap();

        assert!(status.ok);
        assert_eq!(status.code, "PROJECT_READY");
        assert_eq!(status.profile_id, Some("geography".to_string()));
        assert_eq!(status.installed_agents, vec!["claude".to_string()]);
    }
}
