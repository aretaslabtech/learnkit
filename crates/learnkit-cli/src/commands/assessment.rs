use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_assessment::build::build_items;
use learnkit_assessment::item::{load_all_items, save_assessment, save_item, Assessment};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_profile::language::learning_item::load_all;
use learnkit_store::session_paths::SessionPaths;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Args)]
pub struct AssessmentArgs {
    #[command(subcommand)]
    action: AssessmentAction,
}

#[derive(Subcommand)]
enum AssessmentAction {
    /// Build the question bank covering recognition/production/listening.
    Build(BuildArgs),
}

#[derive(Args)]
pub struct BuildArgs {
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

pub fn run(args: AssessmentArgs) -> i32 {
    match args.action {
        AssessmentAction::Build(a) => run_build(a),
    }
}

#[derive(Debug, Serialize, Default)]
struct AssessmentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    assessment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skills: Option<Vec<String>>,
}

fn run_build(args: BuildArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let learning_items = match load_all(&root) {
        Ok(items) => items,
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

    // Hard Guards entry check (odd/tasks/hard-guards-entry-checks.md): a live
    // check against real data, never against a cached phase manifest — same
    // rationale/pattern as cards build's guard.
    if learning_items.is_empty() {
        return emit_blocked(
            args.json,
            "vocabulary",
            "no hay ningún elemento de aprendizaje de vocabulario — ejecuta 'learn vocabulary add' primero",
        );
    }

    let items = build_items(&learning_items);
    for item in &items {
        if let Err(source) = save_item(session_paths.root(), item) {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            );
        }
    }

    let assessment = Assessment {
        id: format!("exam-{session_id}"),
        title: format!("Assessment for session {session_id}"),
        item_ids: items.iter().map(|i| i.id.clone()).collect(),
    };
    if let Err(source) = save_assessment(session_paths.root(), &assessment) {
        return emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        );
    }

    let skills: HashSet<String> = items.iter().map(|i| i.skill.as_str().to_string()).collect();
    let mut skills: Vec<String> = skills.into_iter().collect();
    skills.sort();

    if args.json {
        Envelope::ok(
            "ASSESSMENT_BUILT",
            AssessmentData {
                assessment_id: Some(assessment.id),
                items: Some(items.len()),
                skills: Some(skills),
            },
        )
        .print_json();
    } else {
        println!(
            "Assessment {} creado con {} preguntas ({}).",
            assessment.id,
            items.len(),
            skills.join(", ")
        );
    }
    0
}

/// Reads every item persisted for `assessment_id` within `session`, used by
/// both `export exam` and `attempt import` to avoid re-deriving the
/// question bank.
pub fn load_items_for_session(
    root: &std::path::Path,
    session: &str,
) -> std::io::Result<Vec<learnkit_assessment::item::AssessmentItem>> {
    let paths = SessionPaths::new(root, session);
    load_all_items(paths.root())
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), AssessmentData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}

/// Hard Guards entry check failure (odd/tasks/hard-guards-entry-checks.md):
/// same `code: "BLOCKED"` / exit `20` contract used by `cards build`'s and
/// `consolidate`'s own guards.
fn emit_blocked(json: bool, phase: &str, reason: &str) -> i32 {
    if json {
        Envelope::err("BLOCKED", AssessmentData::default()).print_json();
    } else {
        eprintln!("Error: bloqueado — {reason}");
    }
    LearnKitError::GuardBlocked {
        phase: phase.to_string(),
        reason: reason.to_string(),
    }
    .exit_code()
}
