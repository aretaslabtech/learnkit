use crate::provider_error::ProviderError;
use thiserror::Error;

/// Domain-level errors shared by every LearnKit command.
///
/// Each variant maps to a stable `code` string (see `output::Envelope`) and to
/// one of the exit codes fixed in `docs/05-cli-spec.md` §14 / `contracts/cli-commands.md`.
#[derive(Debug, Error)]
pub enum LearnKitError {
    #[error("profile '{requested}' is not supported (supported: {supported:?})")]
    ProfileUnsupported {
        requested: String,
        supported: Vec<String>,
    },

    #[error("agent '{requested}' is not supported (supported: {supported:?})")]
    AgentUnsupported {
        requested: String,
        supported: Vec<String>,
    },

    #[error("at least one agent must be selected")]
    NoAgentSelected,

    #[error("shell '{requested}' is not supported (supported: {supported:?})")]
    ShellUnsupported {
        requested: String,
        supported: Vec<String>,
    },

    #[error("project not initialized at {path}")]
    ProjectNotInitialized { path: String },

    #[error("no active session — pass --session <id> or run 'learnkit session use <id>' first")]
    NoActiveSession,

    #[error("agent integration files were modified manually: {files:?}")]
    AgentFilesModified { files: Vec<String> },

    #[error("interactive selection was cancelled")]
    SelectionCancelled,

    #[error("filesystem error at {path}: {source}")]
    Filesystem {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("guard blocked: {phase} — {reason}")]
    GuardBlocked { phase: String, reason: String },

    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error("exporter constraint failed: {message}")]
    ExporterConstraint { message: String },

    /// Structural/traceability validation failure at the CLI boundary — e.g.
    /// an empty confirmation file, a malformed `--filled-gap`, or a mindmap
    /// byte-for-byte identical to its session's summary (feature 003, US2).
    /// Never a judgment on prose quality (Principio IV) — purely structural.
    #[error("validation failed: {message}")]
    ValidationFailed { message: String },
}

impl LearnKitError {
    /// Stable machine-readable code, shared across human and `--json` output.
    pub fn code(&self) -> &'static str {
        match self {
            LearnKitError::ProfileUnsupported { .. } => "PROFILE_UNSUPPORTED",
            LearnKitError::AgentUnsupported { .. } => "AGENT_UNSUPPORTED",
            LearnKitError::NoAgentSelected => "NO_AGENT_SELECTED",
            LearnKitError::ShellUnsupported { .. } => "SHELL_UNSUPPORTED",
            LearnKitError::ProjectNotInitialized { .. } => "PROJECT_NOT_INITIALIZED",
            LearnKitError::NoActiveSession => "NO_ACTIVE_SESSION",
            LearnKitError::AgentFilesModified { .. } => "AGENT_FILES_MODIFIED",
            LearnKitError::SelectionCancelled => "SELECTION_CANCELLED",
            LearnKitError::Filesystem { .. } => "FILESYSTEM_ERROR",
            LearnKitError::GuardBlocked { .. } => "PHASE_BLOCKED",
            LearnKitError::Provider(_) => "PROVIDER_FAILED",
            LearnKitError::ExporterConstraint { .. } => "EXPORT_FAILED",
            LearnKitError::ValidationFailed { .. } => "VALIDATION_FAILED",
        }
    }

    /// Exit code per `contracts/cli-commands.md`.
    pub fn exit_code(&self) -> i32 {
        match self {
            LearnKitError::ProfileUnsupported { .. }
            | LearnKitError::AgentUnsupported { .. }
            | LearnKitError::NoAgentSelected
            | LearnKitError::ShellUnsupported { .. }
            | LearnKitError::ProjectNotInitialized { .. }
            | LearnKitError::NoActiveSession
            | LearnKitError::SelectionCancelled => 2,
            LearnKitError::AgentFilesModified { .. } | LearnKitError::GuardBlocked { .. } => 20,
            LearnKitError::Provider(_) => 30,
            LearnKitError::ExporterConstraint { .. } => 40,
            LearnKitError::ValidationFailed { .. } => 10,
            LearnKitError::Filesystem { .. } => 50,
        }
    }
}
