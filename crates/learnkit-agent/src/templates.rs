/// A single templated file that belongs to an agent's integration.
#[derive(Debug, Clone, Copy)]
pub struct AgentFileTemplate {
    /// Path relative to the project root where this file is written.
    pub relative_path: &'static str,
    /// Embedded template content (source of truth: `templates/` in this crate).
    pub content: &'static str,
}

const CODEX_AGENTS_MD: &str = include_str!("../templates/AGENTS.md");
const CLAUDE_MD: &str = include_str!("../templates/CLAUDE.md");
const SESSION_SKILL_MD: &str = include_str!("../templates/skills/learnkit-session/SKILL.md");

/// Returns the file templates that make up `agent_id`'s integration.
///
/// Callers MUST validate `agent_id` against `catalog::is_supported` first;
/// this function panics on an unknown agent since it is only ever called
/// after that validation (see `research.md` §4/§7).
pub fn templates_for(agent_id: &str) -> Vec<AgentFileTemplate> {
    match agent_id {
        "codex" => vec![
            AgentFileTemplate {
                relative_path: "AGENTS.md",
                content: CODEX_AGENTS_MD,
            },
            AgentFileTemplate {
                relative_path: ".agents/skills/learnkit-session/SKILL.md",
                content: SESSION_SKILL_MD,
            },
        ],
        "claude" => vec![
            AgentFileTemplate {
                relative_path: "CLAUDE.md",
                content: CLAUDE_MD,
            },
            AgentFileTemplate {
                relative_path: ".claude/skills/learnkit-session/SKILL.md",
                content: SESSION_SKILL_MD,
            },
        ],
        other => panic!("templates_for called with unsupported agent_id: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn every_supported_agent_has_templates() {
        for agent_id in catalog::SUPPORTED_AGENTS {
            let files = templates_for(agent_id);
            assert!(!files.is_empty(), "agent {agent_id} has no templates");
        }
    }
}
