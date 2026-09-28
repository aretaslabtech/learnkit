//! `consolidate` phase entities (US4, feature 003): `ConsolidatedCandidate`,
//! per `data-model.md` → ConsolidatedCandidate. Scoped to vocabulary
//! candidates in this feature (`candidate_type` has a single variant,
//! `Vocabulary`, kept as an enum so it can be extended later without
//! breaking this schema — see `spec.md` Clarifications and Assumptions).
//!
//! Critical architectural constraint (`research.md` §4): this module — and
//! this crate — NEVER depends on `learnkit-profile` and never knows what a
//! "lemma" is. It only persists `ConsolidatedCandidate`s it is handed
//! already-computed, including `already_exists`/`existing_vocabulary_id`,
//! which the CLI handler (`crates/learnkit-cli/src/commands/consolidate.rs`)
//! resolves by calling `learnkit-profile::language::vocabulary` itself.
//! `crates/learnkit-workflow/Cargo.toml` must never gain that dependency.

use crate::engine::{ChecklistItemManifest, ChecklistItemState};
use learnkit_store::session_paths::SessionPaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Reserved for extension with other candidate types (e.g. grammar/general
/// knowledge questions) in a later feature — this feature only ever
/// produces `Vocabulary` (`spec.md` Clarifications).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateType {
    Vocabulary,
}

/// Traceability back to the part of a session's `analyse` output a candidate
/// was derived from (FR-012) — `origin` is `"summary"`, `"mindmap"`, or
/// `"page-<n>"`; `locator` is a free-form pointer within that origin (e.g. an
/// excerpt or offset), left to the caller.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateSourceRef {
    pub origin: String,
    pub locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidatedCandidate {
    pub id: String,
    pub session_id: String,
    pub candidate_type: CandidateType,
    pub text: String,
    pub source_ref: CandidateSourceRef,
    /// Whether a `VocabularyEntry` equivalent to `text` already exists.
    /// Always computed by the CLI handler, never by this crate (see module
    /// doc) — when `true`, the candidate is still persisted/reported (FR-012,
    /// spec.md Acceptance Scenario 2) but not proposed as new (FR-011).
    pub already_exists: bool,
    pub existing_vocabulary_id: Option<String>,
}

fn candidates_path(project_root: &Path, session_id: &str) -> PathBuf {
    SessionPaths::new(project_root, session_id)
        .consolidated()
        .join("candidates.yaml")
}

fn to_io_err(err: serde_yaml::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
}

/// Replaces the full set of consolidated candidates for a session —
/// `consolidate` is meant to be re-run (same idempotent-by-replacement
/// pattern as re-running other phases), so this is not additive.
pub fn write_candidates(
    project_root: &Path,
    session_id: &str,
    candidates: &[ConsolidatedCandidate],
) -> std::io::Result<()> {
    let path = candidates_path(project_root, session_id);
    let yaml = serde_yaml::to_string(candidates).map_err(to_io_err)?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())
}

/// Lists the `ConsolidatedCandidate`s currently persisted for a session, in
/// the order they were written. Returns an empty list (not an error) when
/// `consolidate` has never run for this session.
pub fn list_candidates(
    project_root: &Path,
    session_id: &str,
) -> std::io::Result<Vec<ConsolidatedCandidate>> {
    let path = candidates_path(project_root, session_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    let candidates: Vec<ConsolidatedCandidate> = serde_yaml::from_str(&raw).map_err(to_io_err)?;
    Ok(candidates)
}

/// Entry guard (US4, FR-013, Edge Case of `spec.md`): returns every `analyse`
/// checklist item still `PendingUserDecision`. `consolidate` derives its
/// candidates from the combination of the whole session's confirmed
/// `analyse` output (summary + pages), and there is no clean way to
/// consolidate "only part of the session" without knowing what the
/// still-missing content would have contained — so the caller blocks the
/// entire `consolidate` run whenever this returns anything non-empty,
/// naming each pending item.
pub fn blocking_pending_items(
    analyse_checklist: &[ChecklistItemManifest],
) -> Vec<ChecklistItemManifest> {
    analyse_checklist
        .iter()
        .filter(|item| item.state == ChecklistItemState::PendingUserDecision)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::create_session;

    fn candidate(id: &str, session_id: &str, text: &str) -> ConsolidatedCandidate {
        ConsolidatedCandidate {
            id: id.to_string(),
            session_id: session_id.to_string(),
            candidate_type: CandidateType::Vocabulary,
            text: text.to_string(),
            source_ref: CandidateSourceRef {
                origin: "summary".to_string(),
                locator: "excerpt".to_string(),
            },
            already_exists: false,
            existing_vocabulary_id: None,
        }
    }

    fn checklist_item(item_id: &str, state: ChecklistItemState) -> ChecklistItemManifest {
        ChecklistItemManifest {
            item_id: item_id.to_string(),
            state,
            input_fingerprint: "fp1".to_string(),
            pending_reason: None,
            resolution: None,
        }
    }

    #[test]
    fn candidates_round_trip_and_replace_the_full_set() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        write_candidates(
            dir.path(),
            &session.id,
            &[candidate("cand-1", &session.id, "itinerary")],
        )
        .unwrap();
        let first = list_candidates(dir.path(), &session.id).unwrap();
        assert_eq!(first.len(), 1);

        write_candidates(
            dir.path(),
            &session.id,
            &[
                candidate("cand-1", &session.id, "itinerary"),
                candidate("cand-2", &session.id, "layover"),
            ],
        )
        .unwrap();
        let second = list_candidates(dir.path(), &session.id).unwrap();
        assert_eq!(second.len(), 2);
    }

    #[test]
    fn list_candidates_is_empty_when_never_consolidated() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let candidates = list_candidates(dir.path(), &session.id).unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn blocking_pending_items_returns_only_pending_user_decision_items() {
        let items = vec![
            checklist_item("summary", ChecklistItemState::Done),
            checklist_item("mindmap", ChecklistItemState::PendingUserDecision),
        ];
        let blocking = blocking_pending_items(&items);
        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].item_id, "mindmap");
    }

    #[test]
    fn blocking_pending_items_is_empty_when_nothing_pending() {
        let items = vec![checklist_item("summary", ChecklistItemState::Done)];
        assert!(blocking_pending_items(&items).is_empty());
    }
}
