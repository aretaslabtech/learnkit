use clap::Args;
use learnkit_core::error::LearnKitError;
use learnkit_store::status::build_status;
use std::path::PathBuf;

#[derive(Args)]
pub struct StatusArgs {
    /// Project folder to inspect (defaults to the current directory).
    #[arg(long)]
    path: Option<PathBuf>,

    /// Emit machine-readable JSON instead of human-readable output.
    #[arg(long)]
    json: bool,
}

pub fn run(args: StatusArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    if !root.exists() {
        eprintln!("Error: la ruta '{}' no existe", root.display());
        return 2;
    }

    match build_status(&root) {
        Ok(report) => {
            if args.json {
                match serde_json::to_string_pretty(&report) {
                    Ok(json) => println!("{json}"),
                    Err(err) => eprintln!("failed to serialize --json output: {err}"),
                }
            } else if report.ok {
                println!(
                    "Proyecto LearnKit válido (perfil: {}, agentes: {}, shell: {}).",
                    report.profile_id.as_deref().unwrap_or("?"),
                    if report.installed_agents.is_empty() {
                        "ninguno".to_string()
                    } else {
                        report.installed_agents.join(", ")
                    },
                    report.shell_preference.as_deref().unwrap_or("?")
                );
            } else {
                println!(
                    "No hay un proyecto LearnKit válido en esta carpeta ({}). Ejecuta 'learnkit init'.",
                    report.code
                );
            }
            0
        }
        Err(source) => {
            let err = LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            };
            eprintln!("Error: {err}");
            err.exit_code()
        }
    }
}
