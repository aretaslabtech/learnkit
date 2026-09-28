use crate::engine::PhaseDefinition;

/// The fixed V1 pipeline for this feature, per
/// `docs/04-workflow-guards.md §2`. Declared here as the single source of
/// truth for the DAG (a "DAG, not a rigid list" per §3, but V1's pipeline is
/// small and fixed, so a plain list is enough — no need for a
/// `workflow.toml` parser yet).
///
/// Extended incrementally as each user story adds its phase.
pub fn phase_definitions() -> Vec<PhaseDefinition> {
    vec![
        PhaseDefinition {
            id: "inventory".to_string(),
            requires: vec![],
        },
        PhaseDefinition {
            id: "analyse".to_string(),
            requires: vec!["inventory".to_string()],
        },
        PhaseDefinition {
            id: "vocabulary".to_string(),
            requires: vec!["analyse".to_string()],
        },
        PhaseDefinition {
            id: "cards".to_string(),
            requires: vec!["vocabulary".to_string()],
        },
        PhaseDefinition {
            id: "anki".to_string(),
            requires: vec!["cards".to_string()],
        },
        PhaseDefinition {
            id: "assessment".to_string(),
            requires: vec!["vocabulary".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::topological_order;

    #[test]
    fn declared_phases_have_a_valid_topological_order() {
        let defs = phase_definitions();
        assert!(topological_order(&defs).is_ok());
    }
}
