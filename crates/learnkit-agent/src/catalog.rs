/// Agents supported at this point in the implementation, per
/// `learnkit-implementation-spec/docs/09-agent-integration.md`.
///
/// Order matters: it is the order shown in the interactive menu.
pub const SUPPORTED_AGENTS: &[&str] = &["codex", "claude"];

/// Default agent installed on non-interactive `init` runs when no `--agents`
/// was given, per `spec.md` → Assumptions (matches this repo's own
/// `.specify/init-options.json` → `"ai": "claude"`).
pub const DEFAULT_AGENT: &str = "claude";

pub fn is_supported(agent_id: &str) -> bool {
    SUPPORTED_AGENTS.contains(&agent_id)
}

pub fn supported_agents() -> Vec<String> {
    SUPPORTED_AGENTS.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_is_supported() {
        assert!(is_supported(DEFAULT_AGENT));
    }

    #[test]
    fn unknown_agent_is_not_supported() {
        assert!(!is_supported("agente-inventado"));
    }
}
