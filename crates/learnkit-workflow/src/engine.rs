use learnkit_core::atomic::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    /// Aggregate state for a checklist-bearing phase (`PhaseManifest.checklist`)
    /// where at least one item is `ChecklistItemState::PendingUserDecision` and
    /// none is failed — see `data-model.md` → PhaseManifest/ChecklistItemManifest
    /// and `research.md` §1/§3.
    NeedsUserInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseResult {
    Valid,
    Failed,
}

/// State of a single element inside a checklist-bearing phase's manifest
/// (e.g. `summary`/`mindmap`/`page:<id>` within `analyse`), per
/// `data-model.md` → ChecklistItemManifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistItemState {
    Pending,
    Done,
    Blocked,
    PendingUserDecision,
}

/// How an item that went through `PendingUserDecision` was resolved, per
/// `research.md` §3 (symmetric to the existing hard-guard override pattern
/// in `docs/04-workflow-guards.md §10`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChecklistItemResolution {
    Confirmed,
    Skipped { reason: String },
}

/// A single checklist element's cached diagnostic, mirroring `PhaseManifest`
/// at item granularity — same "never trusted as sole authority" rule applies
/// (see `recompute_checklist_item_state`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItemManifest {
    pub item_id: String,
    pub state: ChecklistItemState,
    pub input_fingerprint: String,
    pub pending_reason: Option<String>,
    pub resolution: Option<ChecklistItemResolution>,
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
    /// `None` for phases without a checklist (`inventory`, `vocabulary`,
    /// `cards`, `anki`, `assessment`, `consolidate`) — behaves exactly as
    /// before this field existed. `Some(items)` for a checklist-bearing
    /// phase (`analyse`); its aggregate `PhaseState` is then derived from
    /// the items instead of `result` alone (see `recompute_state`).
    /// `#[serde(default)]` keeps old manifests written before this field
    /// existed readable as `None`.
    #[serde(default)]
    pub checklist: Option<Vec<ChecklistItemManifest>>,
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

    if let Some(items) = manifest.and_then(|m| m.checklist.as_ref()) {
        return recompute_checklist_aggregate_state(items);
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

/// Derives a checklist-bearing phase's aggregate `PhaseState` from its
/// items, per `data-model.md` → PhaseManifest and `research.md` §1: `Valid`
/// only if every item is `Done`; `NeedsUserInput` if at least one item is
/// `PendingUserDecision`; otherwise the phase is still in progress (`Dirty`)
/// — the phase's own dependency-blocking is handled by the caller
/// (`recompute_state`) before this is ever reached.
fn recompute_checklist_aggregate_state(items: &[ChecklistItemManifest]) -> PhaseState {
    if items
        .iter()
        .all(|item| item.state == ChecklistItemState::Done)
    {
        return PhaseState::Valid;
    }
    if items
        .iter()
        .any(|item| item.state == ChecklistItemState::PendingUserDecision)
    {
        return PhaseState::NeedsUserInput;
    }
    PhaseState::Dirty
}

/// Recomputes a single checklist item's live state against the current
/// fingerprint of the sources it depends on — same fingerprint-comparison
/// pattern `recompute_state` already uses for a whole phase (FR-004
/// cascading invalidation, `data-model.md` → ChecklistItemManifest): an item
/// stuck as `Done` whose dependency changed underneath it is reported as
/// `Pending`, never trusted as still `Done` just because it was written that
/// way. Any other stored state (`Blocked`/`PendingUserDecision`) is returned
/// unchanged — cascading invalidation only ever demotes a completed item.
pub fn recompute_checklist_item_state(
    item: &ChecklistItemManifest,
    current_input_fingerprint: &str,
) -> ChecklistItemState {
    if item.state == ChecklistItemState::Done && item.input_fingerprint != current_input_fingerprint
    {
        return ChecklistItemState::Pending;
    }
    item.state
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

/// Same relative layout as `SessionPaths::phase_manifest` (`validation/<phase>.json`
/// under the session root), reproduced here instead of taking a
/// `learnkit-store` `SessionPaths` so these helpers only need the session's
/// own root directory, not a separate `project_root`/`session_id` pair.
fn checklist_manifest_path(session_root: &Path, phase: &str) -> PathBuf {
    session_root
        .join("validation")
        .join(format!("{phase}.json"))
}

/// Reads a checklist-bearing phase's items as currently persisted. Returns
/// `None` when there is no manifest yet, or when the manifest exists but has
/// no checklist (a phase that never adopted one) — same "absence means not
/// started / not applicable" contract as `read_manifest`. Callers that need
/// live (cascade-invalidated) state should pass each item through
/// `recompute_checklist_item_state` rather than trusting `item.state` alone.
pub fn read_checklist(
    session_root: &Path,
    phase: &str,
) -> std::io::Result<Option<Vec<ChecklistItemManifest>>> {
    let path = checklist_manifest_path(session_root, phase);
    let manifest = read_manifest(&path)?;
    Ok(manifest.and_then(|m| m.checklist))
}

/// Creates or updates a single checklist item inside the phase's manifest,
/// leaving every other item untouched. Creates the manifest itself (with an
/// empty `result`/fingerprint scaffold and an empty checklist to insert
/// into) when the phase has not been written to yet.
pub fn write_checklist_item(
    session_root: &Path,
    phase: &str,
    item: ChecklistItemManifest,
) -> std::io::Result<()> {
    let path = checklist_manifest_path(session_root, phase);
    let mut manifest = read_manifest(&path)?.unwrap_or_else(|| PhaseManifest {
        phase: phase.to_string(),
        input_fingerprint: String::new(),
        output_fingerprint: String::new(),
        validated_at: String::new(),
        result: PhaseResult::Failed,
        checklist: Some(Vec::new()),
    });

    let items = manifest.checklist.get_or_insert_with(Vec::new);
    match items
        .iter_mut()
        .find(|existing| existing.item_id == item.item_id)
    {
        Some(existing) => *existing = item,
        None => items.push(item),
    }

    write_manifest(&path, &manifest)
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
            checklist: None,
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
            checklist: None,
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
            checklist: None,
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
            checklist: None,
        };

        write_manifest(&path, &manifest).unwrap();
        let read_back = read_manifest(&path).unwrap().unwrap();

        assert_eq!(read_back.phase, "inventory");
        assert_eq!(read_back.input_fingerprint, "fp1");
    }

    #[test]
    fn old_manifest_without_checklist_field_deserializes_as_none() {
        // Regression guard for FR-003 / backward compatibility: a manifest
        // written before this feature existed has no `checklist` key at all.
        let raw = r#"{
            "phase": "inventory",
            "input_fingerprint": "fp1",
            "output_fingerprint": "out1",
            "validated_at": "now",
            "result": "valid"
        }"#;
        let manifest: PhaseManifest = serde_json::from_str(raw).unwrap();
        assert_eq!(manifest.checklist, None);
    }

    fn checklist_item(
        item_id: &str,
        state: ChecklistItemState,
        input_fingerprint: &str,
    ) -> ChecklistItemManifest {
        ChecklistItemManifest {
            item_id: item_id.to_string(),
            state,
            input_fingerprint: input_fingerprint.to_string(),
            pending_reason: None,
            resolution: None,
        }
    }

    fn manifest_with_checklist(items: Vec<ChecklistItemManifest>) -> PhaseManifest {
        PhaseManifest {
            phase: "analyse".to_string(),
            input_fingerprint: "fp1".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Failed,
            checklist: Some(items),
        }
    }

    #[test]
    fn checklist_phase_is_valid_only_when_every_item_is_done() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Valid);

        let all_done = manifest_with_checklist(vec![
            checklist_item("summary", ChecklistItemState::Done, "fp1"),
            checklist_item("mindmap", ChecklistItemState::Done, "fp1"),
        ]);
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&all_done), "fp1");
        assert_eq!(state, PhaseState::Valid);

        let one_pending = manifest_with_checklist(vec![
            checklist_item("summary", ChecklistItemState::Done, "fp1"),
            checklist_item("mindmap", ChecklistItemState::Pending, "fp1"),
        ]);
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&one_pending), "fp1");
        assert_ne!(state, PhaseState::Valid);
    }

    #[test]
    fn checklist_phase_needs_user_input_when_an_item_is_pending_user_decision() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Valid);

        let manifest = manifest_with_checklist(vec![
            checklist_item("summary", ChecklistItemState::Done, "fp1"),
            checklist_item("mindmap", ChecklistItemState::PendingUserDecision, "fp1"),
        ]);
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&manifest), "fp1");
        assert_eq!(state, PhaseState::NeedsUserInput);
    }

    #[test]
    fn checklist_phase_is_blocked_when_its_dependency_is_not_valid() {
        let mut deps = HashMap::new();
        deps.insert("inventory".to_string(), PhaseState::Dirty);

        let manifest = manifest_with_checklist(vec![checklist_item(
            "summary",
            ChecklistItemState::Done,
            "fp1",
        )]);
        let state = recompute_state(&["inventory".to_string()], &deps, Some(&manifest), "fp1");
        assert_eq!(state, PhaseState::Blocked);
    }

    #[test]
    fn checklist_item_with_stale_fingerprint_recomputes_as_pending() {
        let item = checklist_item("summary", ChecklistItemState::Done, "fp-old");
        let state = recompute_checklist_item_state(&item, "fp-new");
        assert_eq!(state, ChecklistItemState::Pending);
    }

    #[test]
    fn checklist_item_with_matching_fingerprint_stays_done() {
        let item = checklist_item("summary", ChecklistItemState::Done, "fp1");
        let state = recompute_checklist_item_state(&item, "fp1");
        assert_eq!(state, ChecklistItemState::Done);
    }

    #[test]
    fn checklist_item_pending_user_decision_is_untouched_by_fingerprint_change() {
        let item = checklist_item("mindmap", ChecklistItemState::PendingUserDecision, "fp-old");
        let state = recompute_checklist_item_state(&item, "fp-new");
        assert_eq!(state, ChecklistItemState::PendingUserDecision);
    }

    #[test]
    fn write_checklist_item_creates_manifest_and_read_checklist_round_trips() {
        let dir = tempfile::tempdir().unwrap();

        write_checklist_item(
            dir.path(),
            "analyse",
            checklist_item("summary", ChecklistItemState::Done, "fp1"),
        )
        .unwrap();

        let items = read_checklist(dir.path(), "analyse").unwrap().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_id, "summary");
        assert_eq!(items[0].state, ChecklistItemState::Done);
    }

    #[test]
    fn write_checklist_item_updates_existing_item_without_touching_others() {
        let dir = tempfile::tempdir().unwrap();

        write_checklist_item(
            dir.path(),
            "analyse",
            checklist_item("summary", ChecklistItemState::Pending, "fp1"),
        )
        .unwrap();
        write_checklist_item(
            dir.path(),
            "analyse",
            checklist_item("mindmap", ChecklistItemState::Pending, "fp1"),
        )
        .unwrap();
        write_checklist_item(
            dir.path(),
            "analyse",
            checklist_item("summary", ChecklistItemState::Done, "fp1"),
        )
        .unwrap();

        let items = read_checklist(dir.path(), "analyse").unwrap().unwrap();
        assert_eq!(items.len(), 2);
        let summary = items.iter().find(|i| i.item_id == "summary").unwrap();
        let mindmap = items.iter().find(|i| i.item_id == "mindmap").unwrap();
        assert_eq!(summary.state, ChecklistItemState::Done);
        assert_eq!(mindmap.state, ChecklistItemState::Pending);
    }

    #[test]
    fn read_checklist_is_none_for_a_phase_without_one() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = PhaseManifest {
            phase: "inventory".to_string(),
            input_fingerprint: "fp1".to_string(),
            output_fingerprint: "out1".to_string(),
            validated_at: "now".to_string(),
            result: PhaseResult::Valid,
            checklist: None,
        };
        write_manifest(&checklist_manifest_path(dir.path(), "inventory"), &manifest).unwrap();

        let items = read_checklist(dir.path(), "inventory").unwrap();
        assert_eq!(items, None);
    }

    #[test]
    fn read_checklist_is_none_when_no_manifest_exists_yet() {
        let dir = tempfile::tempdir().unwrap();
        let items = read_checklist(dir.path(), "analyse").unwrap();
        assert_eq!(items, None);
    }
}
