//! `learnkit analyse set` — confirms the `analyse` phase's checklist
//! elements (US2, feature 003; unified onto a single item-based API by
//! FR-018, Phase 10). Content is always redacted by an agent via the
//! `learnkit-analyse` Skill; this module only validates
//! structure/traceability and persists — never prose quality
//! (`contracts/agent-skill.md`, Principio IV).
//!
//! Every result of `analyse` — the summary, the mind map, and any number of
//! concept pages — is addressed through one stable `item_id`: `summary`,
//! `mindmap`, or `page:<stable_id>`. `summary`/`mindmap` are reserved
//! singleton ids; any `page:<id>` (non-empty `<id>`) identifies a
//! `ConceptPage` with a stable identity chosen by the caller, never
//! positional. The underlying domain types (`ClassSummary`/`StudyMap`/
//! `ConceptPage`) and their own validation rules are unchanged — only the
//! CLI surface is unified (FR-018).
//!
//! `learnkit analyse flag-pending/skip` (US3, feature 003) let a checklist
//! item be marked `pending_user_decision` when the source material isn't
//! enough to redact it, and later resolved explicitly (either by confirming
//! real content via `analyse set` once it exists, or by `skip`ping it) — see
//! `contracts/cli-commands.md`.

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
    /// Upsert one analysis item (`summary`, `mindmap`, or `page:<id>`).
    Set(SetArgs),
    /// Mark a checklist item as `pending_user_decision` with a reason
    /// (US3, FR-007) — item is `summary`, `mindmap`, or `page:<id>`.
    FlagPending(ItemActionArgs),
    /// Resolve a checklist item explicitly without confirming real content
    /// (US3, FR-008) — item is `summary`, `mindmap`, or `page:<id>`.
    Skip(ItemActionArgs),
}

#[derive(Args)]
struct SetArgs {
    #[arg(long)]
    session: Option<String>,
    /// The analysis item to confirm/update: `summary`, `mindmap`, or
    /// `page:<stable_id>`.
    #[arg(long)]
    item: String,
    #[arg(long)]
    file: PathBuf,
    /// Display name for a `page:<id>` item — required the first time that
    /// page is created; optional on a later update (keeps the existing
    /// display name when omitted). Ignored for `summary`/`mindmap`.
    #[arg(long)]
    concept: Option<String>,
    /// A concept the source material left mentioned but unexplained, filled
    /// in by the redacting agent (FR-006, generalized to every item kind by
    /// FR-018) — `concepto:nota`, both non-empty.
    #[arg(long = "filled-gap")]
    filled_gap: Vec<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Re-confirms even if the item is already `done` with a matching
    /// source fingerprint (FR-009 idempotency, same pattern as
    /// `cards build --force`).
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct ItemActionArgs {
    /// The checklist item to act on: `summary`, `mindmap`, or `page:<id>`.
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

/// What kind of analysis item an `item_id` string names (FR-018): the
/// public API is unified, but each kind still routes to its own domain
/// type/validation (`ClassSummary`/`StudyMap`/`ConceptPage`) internally.
enum ItemKind {
    Summary,
    Mindmap,
    Page(String),
}

impl ItemKind {
    fn label(&self) -> &'static str {
        match self {
            ItemKind::Summary => "class_summary",
            ItemKind::Mindmap => "study_map",
            ItemKind::Page(_) => "concept_page",
        }
    }
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
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    concept: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

pub fn run(args: AnalyseArgs) -> i32 {
    match args.action {
        AnalyseAction::Set(a) => run_set(a),
        AnalyseAction::FlagPending(a) => run_flag_pending(a),
        AnalyseAction::Skip(a) => run_skip(a),
    }
}

/// Parses and validates an `item_id`'s shape (FR-018): `summary`/`mindmap`
/// are reserved singletons; any other id must be `page:<non-empty-id>`.
/// Rejects everything else with a structured, useful error — never silently
/// falls back to treating an unknown id as a page.
fn parse_item_id(item_id: &str) -> Result<ItemKind, LearnKitError> {
    match item_id {
        "summary" => Ok(ItemKind::Summary),
        "mindmap" => Ok(ItemKind::Mindmap),
        _ => match item_id.strip_prefix("page:") {
            Some(rest) if !rest.is_empty() => Ok(ItemKind::Page(rest.to_string())),
            Some(_) => Err(LearnKitError::ValidationFailed {
                message: "'page:' necesita un id no vacío después de los dos puntos (p. ej. 'page:layover')".to_string(),
            }),
            None => Err(LearnKitError::ValidationFailed {
                message: format!(
                    "'{item_id}' no es un item_id válido de 'analyse' (se esperaba 'summary', 'mindmap' o 'page:<id>')"
                ),
            }),
        },
    }
}

fn run_set(args: SetArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let kind = match parse_item_id(&args.item) {
        Ok(k) => k,
        Err(err) => return emit_error(args.json, err),
    };

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

    // Type-specific structural validation (kept exactly as before FR-018 —
    // unifying the CLI surface does not unify domain validation):
    // Acceptance Scenario 2 of US2: a mindmap byte-for-byte identical to the
    // session's already-confirmed summary is rejected explicitly — purely
    // structural detection of "not a repetition", never a prose-quality
    // judgment (`contracts/agent-skill.md`).
    if let ItemKind::Mindmap = kind {
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
    }

    if !args.force {
        match item_already_done(session_paths.root(), &args.item, &fingerprint) {
            Ok(true) => {
                let concept = existing_concept(&root, &session_id, &kind);
                return emit_ok(
                    args.json,
                    &args.item,
                    "done",
                    true,
                    kind.label(),
                    concept.as_deref(),
                );
            }
            Ok(false) => {}
            Err(err) => return emit_error(args.json, err),
        }
    }

    let concept_out = match kind {
        ItemKind::Summary => {
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
            None
        }
        ItemKind::Mindmap => {
            let map = StudyMap {
                session_id: session_id.clone(),
                content,
                source_fingerprint: fingerprint.clone(),
                filled_gaps,
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
            None
        }
        ItemKind::Page(ref page_id) => {
            let full_id = format!("page:{page_id}");
            let existing = match analysis::read_concept_page(&root, &session_id, &full_id) {
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
            let concept = match (&args.concept, &existing) {
                (Some(c), _) if !c.trim().is_empty() => c.trim().to_string(),
                (Some(_), _) => {
                    return emit_error(
                        args.json,
                        LearnKitError::ValidationFailed {
                            message: "--concept no puede estar vacío".to_string(),
                        },
                    )
                }
                (None, Some(p)) => p.concept.clone(),
                (None, None) => {
                    return emit_error(
                        args.json,
                        LearnKitError::ValidationFailed {
                            message: "--concept es obligatorio al crear una página nueva (item_id no existía todavía)".to_string(),
                        },
                    )
                }
            };
            match analysis::upsert_concept_page(
                &root,
                &session_id,
                &full_id,
                &concept,
                &content,
                &fingerprint,
                filled_gaps,
            ) {
                Ok(page) => Some(page.concept),
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
        }
    };

    if let Err(err) = confirm_item(session_paths.root(), &args.item, &fingerprint) {
        return emit_error(args.json, err);
    }
    if let Err(err) = sync_output_fingerprint(&session_paths, &fingerprint) {
        return emit_error(args.json, err);
    }

    emit_ok(
        args.json,
        &args.item,
        "done",
        false,
        kind.label(),
        concept_out.as_deref(),
    )
}

/// Reads the current display name for an already-`done` item, for reporting
/// purposes only (never authoritative for validation — that already
/// happened before this is called).
fn existing_concept(root: &Path, session_id: &str, kind: &ItemKind) -> Option<String> {
    match kind {
        ItemKind::Summary | ItemKind::Mindmap => None,
        ItemKind::Page(page_id) => {
            let full_id = format!("page:{page_id}");
            analysis::read_concept_page(root, session_id, &full_id)
                .ok()
                .flatten()
                .map(|p| p.concept)
        }
    }
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
    if let Err(err) = validate_item_id(&args.item_id) {
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
    if let Err(err) = validate_item_id(&args.item_id) {
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

/// Validates that `item_id` has a legal shape for the `analyse` phase
/// (US3/FR-018): `summary`, `mindmap`, or `page:<non-empty-id>`. Since
/// Phase 10, page ids are arbitrary agent-chosen stable slugs rather than a
/// positional sequence, so — unlike the old `page-<n>` scheme — there is no
/// "next expected id" to check against: `flag-pending`/`skip` may
/// legitimately be the very first thing ever recorded for a brand new
/// `page:<id>`, same as `analyse set` creating it for the first time.
fn validate_item_id(item_id: &str) -> Result<(), LearnKitError> {
    parse_item_id(item_id).map(|_| ())
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
/// means the confirmation is a no-op. Generalized to every item kind by
/// FR-018 — `page:<id>` now shares the exact same idempotency semantics as
/// `summary`/`mindmap`, instead of always creating a new page.
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

fn confirm_item(
    session_root: &Path,
    item_id: &str,
    fingerprint: &str,
) -> Result<(), LearnKitError> {
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

fn emit_ok(
    json: bool,
    item_id: &str,
    state: &str,
    already_done: bool,
    kind: &str,
    concept: Option<&str>,
) -> i32 {
    if json {
        Envelope::ok(
            "ANALYSE_ITEM_SET",
            AnalyseData {
                item_id: Some(item_id.to_string()),
                state: Some(state.to_string()),
                already_done: Some(already_done),
                kind: Some(kind.to_string()),
                concept: concept.map(|c| c.to_string()),
                ..Default::default()
            },
        )
        .print_json();
    } else if already_done {
        println!("✓ {item_id} already current (sin cambios en las fuentes)");
    } else {
        println!("✓ {item_id} confirmed");
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
