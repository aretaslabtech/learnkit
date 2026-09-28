use thiserror::Error;

/// Shared error type for external capability providers (transcription,
/// voice synthesis, image search). Mirrors the shape every provider trait
/// in this feature uses — see `specs/002-english-eoi-flow/contracts/provider-traits.md`.
///
/// A provider error is never a workflow guard decision by itself: callers
/// translate it into a `LearnKitError` with the exit code appropriate to
/// their command (typically 30, "provider/external tool failed").
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("required external tool '{tool}' was not found on PATH")]
    ToolNotFound { tool: String },

    #[error("provider '{provider}' failed: {message}")]
    ExecutionFailed { provider: String, message: String },

    #[error("provider '{provider}' returned output that could not be parsed: {message}")]
    UnparsableOutput { provider: String, message: String },

    #[error("network request to '{provider}' failed: {message}")]
    NetworkError { provider: String, message: String },
}
