mod commands;
mod interactive;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "learnkit", version, about = "LearnKit CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize (or re-confirm) a LearnKit project in a folder.
    Init(commands::init::InitArgs),
    /// Manage agent integrations (Codex/Claude/...).
    Agent(commands::agent::AgentArgs),
    /// Report whether a folder contains a valid LearnKit project.
    Status(commands::status::StatusArgs),
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Agent(args) => commands::agent::run(args),
        Command::Status(args) => commands::status::run(args),
    };

    std::process::exit(exit_code);
}
