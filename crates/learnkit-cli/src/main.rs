mod commands;
mod interactive;
mod session_context;

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
    /// Manage sessions (a session groups a class's sources and workflow).
    Session(commands::session::SessionArgs),
    /// Copy files into a session's `input/` directory.
    Ingest(commands::inventory::IngestArgs),
    /// Inventory a session's sources (type, size, integrity hash).
    Inventory(commands::inventory::InventoryArgs),
    /// Transcribe (or import a transcription for) a session's audio sources.
    Transcribe(commands::transcribe::TranscribeArgs),
    /// Persist confirmed learning content (vocabulary, ...).
    Learn(commands::learn::LearnArgs),
    /// Build or validate study cards from vocabulary learning items.
    Cards(commands::cards::CardsArgs),
    /// Export session artifacts to external formats (Anki, exam HTML, ...).
    Export(commands::export::ExportArgs),
    /// Generate the assessment question bank for a session.
    Assessment(commands::assessment::AssessmentArgs),
    /// Import a completed exam's results.
    Attempt(commands::attempt::AttemptArgs),
    /// View progress aggregated by learning item and skill.
    Progress(commands::progress::ProgressArgs),
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
        Command::Session(args) => commands::session::run(args),
        Command::Ingest(args) => commands::inventory::run_ingest(args),
        Command::Inventory(args) => commands::inventory::run_inventory(args),
        Command::Transcribe(args) => commands::transcribe::run(args),
        Command::Learn(args) => commands::learn::run(args),
        Command::Cards(args) => commands::cards::run(args),
        Command::Export(args) => commands::export::run(args),
        Command::Assessment(args) => commands::assessment::run(args),
        Command::Attempt(args) => commands::attempt::run(args),
        Command::Progress(args) => commands::progress::run(args),
    };

    std::process::exit(exit_code);
}
