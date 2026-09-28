//! `learnkit analyse summary/mindmap/page` — confirms the `analyse` phase's
//! checklist elements (US2, feature 003). Content is always redacted by an
//! agent via the `learnkit-analyse` Skill; this module only validates
//! structure/traceability and persists — never prose quality
//! (`contracts/agent-skill.md`, Principio IV).
//!
//! `learnkit analyse flag-pending/skip` (US3, feature 003) let a checklist
//! item be marked `pending_user_decision` when the source material isn't
//! enough to redact it, and later resolved explicitly (either by confirming
//! real content via `summary`/`mindmap`/`page` once it exists, or by
//! `skip`ping it) — see `contracts/cli-commands.md`.

use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_store::session_paths::SessionPaths;
use learnkit_workflow::analysis::{self, ClassSummary, FilledGap, StudyMap};
use learnkit_workflow::engine::{
    read_checklist, read_manifest, recompute_checklist_item_state, write_checklist_item,
    write_manifest, ChecklistItemManifest, ChecklistItemResolution, ChecklistItemState,
    PhaseManifest, PhaseResult,
};
use learnkit_workflow::inventory as wf_inventory;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct AnalyseArgs {
    #[command(subcommand)]
    action: AnalyseAction,
}

#[derive(Subcommand)]
enum AnalyseAction {
    /// Confirm the `summary` checklist element.
    Summary(SummaryArgs),
    /// Confirm the `mindmap` checklist element.
    Mindmap(MindmapArgs),
    /// Add and confirm a `page-<n>` checklist element.
    Page(PageArgs),
    /// Mark a checklist item as `pending_user_decision` with a reason
    /// (US3, FR-007) — item is `summary`, `mindmap`, or `page-<n>`.
    FlagPending(ItemActionArgs),
    /// Resolve a checklist item explicitly without confirming real content
    /// (US3, FR-008) — item is `summary`, `mindmap`, or `page-<n>`.
    Skip(ItemActionArgs),
}

#[derive(Args)]
struct ItemActionArgs {
    /// The checklist item to act on: `summary`, `mindmap`, or `page-<n>`.
    item_id: String,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    reason: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct SummaryArgs {
    #[command(subcommand)]
    action: SummaryAction,
}

#[derive(Subcommand)]
enum SummaryAction {
    Set(SummarySetArgs),
}

#[derive(Args)]
struct SummarySetArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    file: PathBuf,
    /// A concept the source material left mentioned but unexplained, filled
    /// in by the redacting agent (FR-006) — `concepto:nota`, both non-empty.
    #[arg(long = "filled-gap")]
    filled_gap: Vec<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Re-confirms even if `summary` is already `done` with a matching
    /// source fingerprint (FR-009 idempotency, same pattern as
    /// `cards build --force`).
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct MindmapArgs {
    #[command(subcommand)]
    action: MindmapAction,
}

#[derive(Subcommand)]
enum MindmapAction {
    Set(MindmapSetArgs),
}

#[derive(Args)]
struct MindmapSetArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct PageArgs {
    #[command(subcommand)]
    action: PageAction,
}

#[derive(Subcommand)]
enum PageAction {
    Add(PageAddArgs),
}

#[derive(Args)]
struct PageAddArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    concept: String,
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct AnalyseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    already_done: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

pub fn run(args: AnalyseArgs) -> i32 {
    match args.action {
        AnalyseAction::Summary(a) => match a.action {
            SummaryAction::Set(a) => run_summary_set(a),
        },
        AnalyseAction::Mindmap(a) => match a.action {
            MindmapAction::Set(a) => run_mindmap_set(a),
        },
        AnalyseAction::Page(a) => match a.action {
            PageAction::Add(a) => run_page_add(a),
        },
        AnalyseAction::FlagPending(a) => run_flag_pending(a),
        AnalyseAction::Skip(a) => run_skip(a),
    }
}

fn run_summary_set(args: SummarySetArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let content = match read_content_file(&args.file) {
        Ok(c) => c,
        Err(err) => return emit_error(args.json, err),
    };
    let filled_gaps = match parse_filled_gaps(&args.filled_gap) {
        Ok(g) => g,
        Err(err) => return emit_error(args.json, err),
    };
    let fingerprint = match current_source_fingerprint(&root, &session_id) {
        Ok(f) => f,
        Err(err) => return emit_error(args.json, err),
    };

    if !args.force {
        match item_already_done(session_paths.root(), "summary", &fingerprint) {
            Ok(true) => {
                return emit_ok(args.json, "ANALYSE_SUMMARY_SET", "summary", "done", true)
            }
            Ok(false) => {}
            Err(err) => return emit_error(args.json, err),
        }
    }

    let summary = ClassSummary {
        session_id: session_id.clone(),
        content,
        filled_gaps,
        source_fingerprint: fingerprint.clone(),
    };
    if let Err(source) = analysis::write_summary(&root, &summary) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }
    if let Err(err) = confirm_item(session_paths.root(), "summary", &fingerprint) {
        return emit_error(args.json, err);
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_ok(args.json, "ANALYSE_SUMMARY_SET", "summary", "done", false)
}

fn run_mindmap_set(args: MindmapSetArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let content = match read_content_file(&args.file) {
        Ok(c) => c,
        Err(err) => return emit_error(args.json, err),
    };

    // Acceptance Scenario 2 of US2: a mindmap byte-for-byte identical to the
    // session's already-confirmed summary is rejected explicitly — purely
    // structural detection of "not a repetition", never a prose-quality
    // judgment (`contracts/agent-skill.md`).
    match analysis::read_summary(&root, &session_id) {
        Ok(Some(summary)) if summary.content == content => {
            return emit_error(
                args.json,
                LearnKitError::ValidationFailed {
                    message: "el mapa mental no puede ser idéntico, byte a byte, al resumen ya confirmado de esta sesión".to_string(),
                },
            );
        }
        Ok(_) => {}
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    }

    let fingerprint = match current_source_fingerprint(&root, &session_id) {
        Ok(f) => f,
        Err(err) => return emit_error(args.json, err),
    };

    if !args.force {
        match item_already_done(session_paths.root(), "mindmap", &fingerprint) {
            Ok(true) => {
                return emit_ok(args.json, "ANALYSE_MINDMAP_SET", "mindmap", "done", true)
            }
            Ok(false) => {}
            Err(err) => return emit_error(args.json, err),
        }
    }

    let map = StudyMap {
        session_id: session_id.clone(),
        content,
        source_fingerprint: fingerprint.clone(),
    };
    if let Err(source) = analysis::write_study_map(&root, &map) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }
    if let Err(err) = confirm_item(session_paths.root(), "mindmap", &fingerprint) {
        return emit_error(args.json, err);
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_ok(args.json, "ANALYSE_MINDMAP_SET", "mindmap", "done", false)
}

fn run_page_add(args: PageAddArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    if args.concept.trim().is_empty() {
        return emit_error(
            args.json,
            LearnKitError::ValidationFailed {
                message: "--concept no puede estar vacío".to_string(),
            },
        );
    }

    let content = match read_content_file(&args.file) {
        Ok(c) => c,
        Err(err) => return emit_error(args.json, err),
    };
    let fingerprint = match current_source_fingerprint(&root, &session_id) {
        Ok(f) => f,
        Err(err) => return emit_error(args.json, err),
    };

    // Each call creates a brand new, independent checklist element (Edge
    // Case of spec.md) — no idempotency/`--force` here, unlike
    // `summary`/`mindmap`.
    let page = match analysis::add_concept_page(&root, &session_id, &args.concept, &content, &fingerprint)
    {
        Ok(p) => p,
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    if let Err(err) = confirm_item(session_paths.root(), &page.id, &fingerprint) {
        return emit_error(args.json, err);
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_ok(args.json, "ANALYSE_PAGE_ADDED", &page.id, "done", false)
}

fn run_flag_pending(args: ItemActionArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let reason = args.reason.trim().to_string();
    if reason.is_empty() {
        return emit_error(
            args.json,
            LearnKitError::ValidationFailed {
                message: "--reason no puede estar vacío".to_string(),
            },
        );
    }
    if let Err(err) = validate_item_id(session_paths.root(), &args.item_id) {
        return emit_error(args.json, err);
    }

    let fingerprint = match current_source_fingerprint(&root, &session_id) {
        Ok(f) => f,
        Err(err) => return emit_error(args.json, err),
    };

    if let Err(source) = write_checklist_item(
        session_paths.root(),
        "analyse",
        ChecklistItemManifest {
            item_id: args.item_id.clone(),
            state: ChecklistItemState::PendingUserDecision,
            input_fingerprint: fingerprint.clone(),
            pending_reason: Some(reason.clone()),
            resolution: None,
        },
    ) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: session_paths.root().display().to_string(),
                source,
            },
        );
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_pending(args.json, &args.item_id, &reason)
}

fn run_skip(args: ItemActionArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let reason = args.reason.trim().to_string();
    if reason.is_empty() {
        return emit_error(
            args.json,
            LearnKitError::ValidationFailed {
                message: "--reason no puede estar vacío".to_string(),
            },
        );
    }
    if let Err(err) = validate_item_id(session_paths.root(), &args.item_id) {
        return emit_error(args.json, err);
    }

    let fingerprint = match current_source_fingerprint(&root, &session_id) {
        Ok(f) => f,
        Err(err) => return emit_error(args.json, err),
    };

    // Resolved for pipeline purposes (`Done`, won't block dependent
    // phases/checks — FR-008), but `resolution: Skipped{reason}` keeps it
    // distinguishable from a real `Confirmed` item for anyone inspecting it
    // later (e.g. `consolidate`'s guard, or a human). `pending_reason` is
    // cleared: per `data-model.md`, it's only populated while
    // `state = PendingUserDecision`.
    if let Err(source) = write_checklist_item(
        session_paths.root(),
        "analyse",
        ChecklistItemManifest {
            item_id: args.item_id.clone(),
            state: ChecklistItemState::Done,
            input_fingerprint: fingerprint.clone(),
            pending_reason: None,
            resolution: Some(ChecklistItemResolution::Skipped {
                reason: reason.clone(),
            }),
        },
    ) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: session_paths.root().display().to_string(),
                source,
            },
        );
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_skipped(args.json, &args.item_id, &reason)
}

/// Validates that `item_id` refers to a real checklist item of the `analyse`
/// phase (US3): either it already exists in the checklist (any state), or —
/// since `flag-pending`/`skip` may be the very first thing ever recorded for
/// it — it is a known singleton item (`summary`/`mindmap`) or the next
/// sequential `page-<n>` id (matching how `analyse page add` assigns ids).
/// Rejects, e.g., `page-3` when only `page-1`/`page-2` exist.
fn validate_item_id(session_root: &Path, item_id: &str) -> Result<(), LearnKitError> {
    let checklist = read_checklist(session_root, "analyse")
        .map_err(|source| LearnKitError::Filesystem {
            path: session_root.display().to_string(),
            source,
        })?
        .unwrap_or_default();

    if checklist.iter().any(|item| item.item_id == item_id) {
        return Ok(());
    }

    if item_id == "summary" || item_id == "mindmap" {
        return Ok(());
    }

    if let Some(rest) = item_id.strip_prefix("page-") {
        let requested: Option<u32> = rest.parse().ok().filter(|n| *n > 0);
        let next_expected = checklist
            .iter()
            .filter_map(|item| item.item_id.strip_prefix("page-").and_then(|n| n.parse::<u32>().ok()))
            .max()
            .unwrap_or(0)
            + 1;
        return match requested {
            Some(n) if n == next_expected => Ok(()),
            _ => Err(LearnKitError::ValidationFailed {
                message: format!(
                    "'{item_id}' no existe en el checklist de 'analyse' de esta sesión (el siguiente elemento de página esperado es 'page-{next_expected}')"
                ),
            }),
        };
    }

    Err(LearnKitError::ValidationFailed {
        message: format!(
            "'{item_id}' no es un elemento de 'analyse' válido (se esperaba 'summary', 'mindmap' o 'page-<n>')"
        ),
    })
}

fn read_content_file(path: &Path) -> Result<String, LearnKitError> {
    let raw = std::fs::read_to_string(path).map_err(|source| LearnKitError::Filesystem {
        path: path.display().to_string(),
        source,
    })?;
    if raw.trim().is_empty() {
        return Err(LearnKitError::ValidationFailed {
            message: format!("el fichero '{}' está vacío", path.display()),
        });
    }
    Ok(raw)
}

fn parse_filled_gaps(raw: &[String]) -> Result<Vec<FilledGap>, LearnKitError> {
    raw.iter()
        .map(|entry| {
            let mut parts = entry.splitn(2, ':');
            let concept = parts.next().unwrap_or("").trim().to_string();
            let note = parts.next().unwrap_or("").trim().to_string();
            if concept.is_empty() || note.is_empty() {
                return Err(LearnKitError::ValidationFailed {
                    message: format!(
                        "--filled-gap inválido: '{entry}' (formato esperado 'concepto:nota', ambos no vacíos)"
                    ),
                });
            }
            Ok(FilledGap { concept, note })
        })
        .collect()
}

/// Refreshes the session's inventory and computes the whole-session sources
/// fingerprint — used as every `analyse` checklist item's
/// `input_fingerprint` (a per-item subset is allowed by `data-model.md` but
/// not needed yet; every element depends on the same notes/transcript).
fn current_source_fingerprint(root: &Path, session_id: &str) -> Result<String, LearnKitError> {
    match wf_inventory::inventory(root, session_id) {
        Ok(sources) => Ok(wf_inventory::sources_fingerprint(
            sources
                .iter()
                .map(|r| (r.source.id.as_str(), r.source.sha256.as_str())),
        )),
        Err(source) => Err(LearnKitError::Filesystem {
            path: root.display().to_string(),
            source,
        }),
    }
}

/// FR-009 idempotency (same pattern as `cards build --force`): an item
/// already `done` whose live state (fingerprint-checked) is still `done`
/// means the confirmation is a no-op.
fn item_already_done(
    session_root: &Path,
    item_id: &str,
    current_fingerprint: &str,
) -> Result<bool, LearnKitError> {
    let checklist =
        read_checklist(session_root, "analyse").map_err(|source| LearnKitError::Filesystem {
            path: session_root.display().to_string(),
            source,
        })?;
    Ok(checklist
        .unwrap_or_default()
        .iter()
        .find(|item| item.item_id == item_id)
        .map(|item| {
            recompute_checklist_item_state(item, current_fingerprint) == ChecklistItemState::Done
        })
        .unwrap_or(false))
}

fn confirm_item(session_root: &Path, item_id: &str, fingerprint: &str) -> Result<(), LearnKitError> {
    write_checklist_item(
        session_root,
        "analyse",
        ChecklistItemManifest {
            item_id: item_id.to_string(),
            state: ChecklistItemState::Done,
            input_fingerprint: fingerprint.to_string(),
            pending_reason: None,
            resolution: Some(ChecklistItemResolution::Confirmed),
        },
    )
    .map_err(|source| LearnKitError::Filesystem {
        path: session_root.display().to_string(),
        source,
    })
}

/// Keeps the `analyse` phase manifest's own `output_fingerprint` in step
/// with the sources every checklist item was just confirmed against —
/// `learnkit status` compares each item's stored `input_fingerprint` against
/// this value to detect cascading invalidation (FR-004), same mechanism
/// `inventory` already writes for its own phase.
fn sync_output_fingerprint(
    session_paths: &SessionPaths,
    fingerprint: &str,
) -> Result<(), LearnKitError> {
    let path = session_paths.phase_manifest("analyse");
    let mut manifest = read_manifest(&path)
        .map_err(|source| LearnKitError::Filesystem {
            path: path.display().to_string(),
            source,
        })?
        .unwrap_or_else(|| PhaseManifest {
            phase: "analyse".to_string(),
            input_fingerprint: String::new(),
            output_fingerprint: String::new(),
            validated_at: String::new(),
            result: PhaseResult::Failed,
            checklist: Some(Vec::new()),
        });
    manifest.output_fingerprint = fingerprint.to_string();
    write_manifest(&path, &manifest).map_err(|source| LearnKitError::Filesystem {
        path: path.display().to_string(),
        source,
    })
}

fn emit_ok(json: bool, code: &str, item_id: &str, state: &str, already_done: bool) -> i32 {
    if json {
        Envelope::ok(
            code,
            AnalyseData {
                item_id: Some(item_id.to_string()),
                state: Some(state.to_string()),
                already_done: Some(already_done),
                ..Default::default()
            },
        )
        .print_json();
    } else if already_done {
        println!("{item_id}: ya estaba {state} (sin cambios en las fuentes)");
    } else {
        println!("{item_id}: {state}");
    }
    0
}

fn emit_pending(json: bool, item_id: &str, reason: &str) -> i32 {
    if json {
        Envelope::ok(
            "ANALYSE_ITEM_FLAG_PENDING",
            AnalyseData {
                item_id: Some(item_id.to_string()),
                state: Some("pending_user_decision".to_string()),
                reason: Some(reason.to_string()),
                ..Default::default()
            },
        )
        .print_json();
    } else {
        println!("{item_id}: pending_user_decision ({reason})");
    }
    0
}

fn emit_skipped(json: bool, item_id: &str, reason: &str) -> i32 {
    if json {
        Envelope::ok(
            "ANALYSE_ITEM_SKIP",
            AnalyseData {
                item_id: Some(item_id.to_string()),
                state: Some("done".to_string()),
                resolution: Some("skipped".to_string()),
                reason: Some(reason.to_string()),
                ..Default::default()
            },
        )
        .print_json();
    } else {
        println!("{item_id}: done (omitido — {reason})");
    }
    0
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), AnalyseData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
