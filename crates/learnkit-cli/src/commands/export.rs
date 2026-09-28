use crate::commands::{assessment, export_anki};
use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_assessment::exam_html;
use learnkit_assessment::item::load_assessment;
use learnkit_core::error::LearnKitError;
use learnkit_store::session_paths::SessionPaths;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportArgs {
    #[command(subcommand)]
    action: ExportAction,
}

#[derive(Subcommand)]
enum ExportAction {
    /// Export a session's completed cards to an Anki-importable .apkg.
    Anki(export_anki::ExportAnkiArgs),
    /// Export a self-contained HTML exam for an assessment.
    Exam(ExamArgs),
}

#[derive(Args)]
pub struct ExamArgs {
    #[arg(long)]
    assessment: String,
    #[arg(long)]
    session: Option<String>,
    #[arg(long = "format", default_value = "html")]
    format: String,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    path: Option<PathBuf>,
}

pub fn run(args: ExportArgs) -> i32 {
    match args.action {
        ExportAction::Anki(a) => export_anki::run(a),
        ExportAction::Exam(a) => run_exam(a),
    }
}

fn run_exam(args: ExamArgs) -> i32 {
    if args.format != "html" {
        eprintln!("Error: solo se soporta --format html");
        return 2;
    }

    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("Error: {err}");
            return err.exit_code();
        }
    };
    let session_paths = SessionPaths::new(&root, &session_id);

    let assessment = match load_assessment(session_paths.root(), &args.assessment) {
        Ok(Some(a)) => a,
        Ok(None) => {
            eprintln!("Error: assessment '{}' not found", args.assessment);
            return 2;
        }
        Err(source) => {
            let err = LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            };
            eprintln!("Error: {err}");
            return err.exit_code();
        }
    };

    let items = match assessment::load_items_for_session(&root, &session_id) {
        Ok(i) => i,
        Err(source) => {
            let err = LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            };
            eprintln!("Error: {err}");
            return err.exit_code();
        }
    };
    let relevant_items: Vec<_> = items
        .into_iter()
        .filter(|i| assessment.item_ids.contains(&i.id))
        .collect();

    let html = exam_html::render(&assessment, &relevant_items);
    if let Some(parent) = args.out.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!("Error: {err}");
            return 50;
        }
    }
    if let Err(err) = std::fs::write(&args.out, html) {
        eprintln!("Error: {err}");
        return 50;
    }

    println!("Examen exportado a {}", args.out.display());
    0
}
