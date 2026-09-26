use dialoguer::{MultiSelect, Select};
use is_terminal::IsTerminal;
use learnkit_agent::catalog as agent_catalog;
use learnkit_core::error::LearnKitError;
use learnkit_core::shell::{detect_default as detect_default_shell, supported_shells};
use learnkit_profile::catalog as profile_catalog;
use std::io;

/// Whether this process can show an interactive menu: both stdin and stdout
/// must be a real terminal. `--json` callers must short-circuit this
/// themselves (see `is_interactive_context`) — FR-013.
pub fn is_tty() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Combines the TTY check with the `--json` flag: `--json` is always treated
/// as non-interactive even if it happens to run in a terminal (FR-013).
pub fn is_interactive_context(json_requested: bool) -> bool {
    !json_requested && is_tty()
}

/// Step 1/3: pick a profile (generic preselected). FR-003/FR-014.
pub fn prompt_profile() -> Result<String, LearnKitError> {
    let profiles = profile_catalog::supported_profiles();
    let default_index = profiles
        .iter()
        .position(|p| p == profile_catalog::DEFAULT_PROFILE)
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt("Selecciona un perfil para el proyecto")
        .items(&profiles)
        .default(default_index)
        .interact_opt()
        .map_err(|_| LearnKitError::SelectionCancelled)?
        .ok_or(LearnKitError::SelectionCancelled)?;

    Ok(profiles[selection].clone())
}

/// Step 2/3: pick one or more agents (minimum one). FR-004/FR-014.
pub fn prompt_agents() -> Result<Vec<String>, LearnKitError> {
    let agents = agent_catalog::supported_agents();

    let selection = MultiSelect::new()
        .with_prompt("Selecciona uno o más agentes (mínimo uno)")
        .items(&agents)
        .interact_opt()
        .map_err(|_| LearnKitError::SelectionCancelled)?
        .ok_or(LearnKitError::SelectionCancelled)?;

    if selection.is_empty() {
        return Err(LearnKitError::NoAgentSelected);
    }

    Ok(selection.into_iter().map(|i| agents[i].clone()).collect())
}

/// Step 3/3: pick a shell preference (auto-detected value preselected).
/// Purely informational — FR-015/FR-014.
pub fn prompt_shell() -> Result<String, LearnKitError> {
    let shells = supported_shells();
    let default_index = shells
        .iter()
        .position(|s| s == detect_default_shell())
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt("Selecciona tu shell preferida (informativo)")
        .items(&shells)
        .default(default_index)
        .interact_opt()
        .map_err(|_| LearnKitError::SelectionCancelled)?
        .ok_or(LearnKitError::SelectionCancelled)?;

    Ok(shells[selection].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_flag_forces_non_interactive_even_if_tty() {
        assert!(!is_interactive_context(true));
    }

    #[test]
    fn prompt_profile_without_a_real_terminal_is_treated_as_cancelled() {
        // `cargo test` runs with no interactive TTY attached to stdin, so
        // dialoguer cannot render the menu; this must surface as a
        // cancellation, never as a panic or as a silent default (FR-014).
        assert!(matches!(
            prompt_profile(),
            Err(LearnKitError::SelectionCancelled)
        ));
    }

    #[test]
    fn prompt_agents_without_a_real_terminal_is_treated_as_cancelled() {
        assert!(matches!(
            prompt_agents(),
            Err(LearnKitError::SelectionCancelled) | Err(LearnKitError::NoAgentSelected)
        ));
    }

    #[test]
    fn prompt_shell_without_a_real_terminal_is_treated_as_cancelled() {
        assert!(matches!(
            prompt_shell(),
            Err(LearnKitError::SelectionCancelled)
        ));
    }
}
