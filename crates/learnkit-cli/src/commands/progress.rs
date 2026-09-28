use clap::Args;
use learnkit_assessment::progress::compute;
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct ProgressArgs {
    #[arg(long)]
    skill: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct ProgressData {
    #[serde(skip_serializing_if = "Option::is_none")]
    by_learning_item: Option<Vec<learnkit_assessment::progress::GroupProgress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_skill: Option<Vec<learnkit_assessment::progress::GroupProgress>>,
}

pub fn run(args: ProgressArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    match compute(&root, args.skill.as_deref()) {
        Ok(report) => {
            if args.json {
                Envelope::ok(
                    "PROGRESS_COMPUTED",
                    ProgressData {
                        by_learning_item: Some(report.by_learning_item),
                        by_skill: Some(report.by_skill),
                    },
                )
                .print_json();
            } else {
                println!("Por elemento de aprendizaje:");
                for g in &report.by_learning_item {
                    println!(
                        "  {}\t{:.0}% ({} intentos)",
                        g.key,
                        g.accuracy_all_time * 100.0,
                        g.attempt_count
                    );
                }
                println!("Por destreza:");
                for g in &report.by_skill {
                    println!(
                        "  {}\t{:.0}% ({} intentos)",
                        g.key,
                        g.accuracy_all_time * 100.0,
                        g.attempt_count
                    );
                }
            }
            0
        }
        Err(source) => {
            let err = LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            };
            if args.json {
                Envelope::err(err.code(), ProgressData::default()).print_json();
            } else {
                eprintln!("Error: {err}");
            }
            err.exit_code()
        }
    }
}
