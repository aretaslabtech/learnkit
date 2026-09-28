use learnkit_core::atomic::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Observable phase states, per `docs/04-workflow-guards.md §4`. Never
/// persisted as an authority value by itself — always recomputed against
/// live fingerprints and dependency states (Hard Guards principle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseState {
    NotStarted,
    Dirty,
    Valid,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseResult {
    Valid,
    Failed,
}

/// Declares a phase and the phases it depends on. Read from
/// `.learnkit/workflow.toml` by each phase-owning crate (session/inventory,
/// transcription, cards, ...); the engine itself is domain-agnostic.
#[derive(Debug, Clone)]
pub struct PhaseDefinition {
    pub id: String,
    pub requires: Vec<String>,
}

/// Cached diagnostic written after a phase runs — never trusted as sole
/// authority on the next read (`docs/04-workflow-guards.md §7`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseManifest {
    pub phase: String,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub validated_at: String,
    pub result: PhaseResult,
}

/// Returns phase ids in dependency order (a phase always comes after
/// everything it `requires`). Errors on an unknown dependency or a cycle.
pub fn topological_order(defs: &[PhaseDefinition]) -> Result<Vec<String>, String> {
    let by_id: HashMap<&str, &PhaseDefinition> = defs.iter().map(|d| (d.id.as_str(), d)).collect();

    let mut order = Vec::with_capacity(defs.len());
    let mut visited: HashSet<&str> = HashSet::new();
    let mut visiting: HashSet<&str> = HashSet::new();

    fn visit<'a>(
        id: &'a str,
        by_id: &HashMap<&'a str, &'a PhaseDefinition>,
        visited: &mut HashSet<&'a str>,
        visiting: &mut HashSet<&'a str>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(id) {
            return Ok(());
        }
        if visiting.contains(id) {
            return Err(format!("cycle detected at phase '{id}'"));
        }
        let def = by_id
            .get(id)
            .ok_or_else(|| format!("unknown phase dependency '{id}'"))?;

        visiting.insert(id);
        for dep in &def.requires {
            visit(dep, by_id, visited, visiting, order)?;
        }
        visiting.remove(id);
        visited.insert(id);
        order.push(id.to_string());
        Ok(())
    }

    for def in defs {
        visit(
            def.id.as_str(),
            &by_id,
            &mut visited,
            &mut visiting,
            &mut order,
        )?;
    }

    Ok(order)
}

/// Recomputes the observable state of a phase from scratch. Never reads
/// `manifest.result` as truth without also comparing dependency states and
/// the current input fingerprint — this is the concrete implementation of
/// the Hard Guards principle for this feature.
pub fn recompute_state(
    requires: &[String],
    dependency_states: &HashMap<String, PhaseState>,
    manifest: Option<&PhaseManifest>,
    current_input_fingerprint: &str,
) -> PhaseState {
    let all_deps_valid = requires
        .iter()
        .all(|dep| dependency_states.get(dep) == Some(&PhaseState::Valid));

    if !all_deps_valid {
        return PhaseState::Blocked;
    }

    match manifest {
        None => PhaseState::NotStarted,
        Some(m) if m.input_fingerprint != current_input_fingerprint => PhaseState::Dirty,
        Some(m) => match m.result {
            PhaseResult::Valid => PhaseState::Valid,
            PhaseResult::Failed => PhaseState::Failed,
        },
    }
}

pub fn read_manifest(path: &Path) -> std::io::Result<Option<PhaseManifest>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)?;
    let manifest: PhaseManifest = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(manifest))
}

pub fn write_manifest(path: &Path, manifest: &PhaseManifest) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    write_atomic(path, json.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(id: &str, requires: &[&str]) -> PhaseDefinition {
        PhaseDefinition {
            id: id.to_string(),
            requires: requires.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let defs = vec![
            def("cards", &["vocabulary"]),
            def("vocabulary", &["analyse"]),
            def("analyse", &["inventory"]),
            def("inventory", &[]),
        ];

        let order = topological_order(&defs).unwrap();
        let pos = |id: &str| order.iter().position(|x| x == id).unwrap();

        assert!(pos("inventory") < pos("analyse"));
        assert!(pos("analyse") < pos("vocabulary"));
        assert!(pos("vocabulary") < pos("cards"));
    }

    #[test]
    fn topological_order_detects_cycles() {
        let defs = vec![def("a", &["b"]), def("b", &["a"])];
        assert!(topological_order(&defs).is_err());
    }

    #[test]
    fn missing_manifest_is_not_started_when_deps_are_valid() {
        let deps = HashMap::new();
        let state = recompute_state(&[], &deps, None, "fp1");
        assert_eq!(state, PhaseState::NotStarted);
    }

    #[test]
    fn unmet_dependency_blocks_regardless_of_manifest() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Dirty);
        let manifest = PhaseManifest {
            phase: "analyse".to_string(),
            input_fingerprint: "fp1".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Valid,
        };
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&manifest), "fp1");
        assert_eq!(state, PhaseState::Blocked);
    }

    #[test]
    fn changed_input_fingerprint_is_dirty() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Valid);
        let manifest = PhaseManifest {
            phase: "analyse".to_string(),
            input_fingerprint: "fp-old".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Valid,
        };
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&manifest), "fp-new");
        assert_eq!(state, PhaseState::Dirty);
    }

    #[test]
    fn matching_fingerprint_and_valid_deps_is_valid() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Valid);
        let manifest = PhaseManifest {
            phase: "analyse".to_string(),
            input_fingerprint: "fp1".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Valid,
        };
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&manifest), "fp1");
        assert_eq!(state, PhaseState::Valid);
    }

    #[test]
    fn manifest_roundtrips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("inventory.json");
        let manifest = PhaseManifest {
            phase: "inventory".to_string(),
            input_fingerprint: "fp1".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Valid,
        };

        write_manifest(&path, &manifest).unwrap();
        let read_back = read_manifest(&path).unwrap().unwrap();

        assert_eq!(read_back.phase, "inventory");
        assert_eq!(read_back.input_fingerprint, "fp1");
    }
}
