use learnkit_core::provider_error::ProviderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TranscriptionRequest {
    pub audio_path: std::path::PathBuf,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// What a provider or importer produces before normalization into the
/// canonical `transcript.json` — see `contracts/provider-traits.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptDraft {
    pub segments: Vec<Segment>,
    pub engine: EngineInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EngineInfo {
    Generated { provider: String, model: String },
    Imported { imported_from: String },
}

/// Never marks a workflow phase as valid by itself — see
/// `contracts/provider-traits.md` → Regla común.
pub trait TranscriptionProvider {
    fn transcribe(&self, request: &TranscriptionRequest) -> Result<TranscriptDraft, ProviderError>;
}

/// A trait sibling to `TranscriptionProvider` for pre-existing
/// transcriptions (SRT/VTT/TXT/JSON), so downstream code cannot tell
/// "generated" from "imported" except via `EngineInfo` (FR-005/FR-006).
pub trait TranscriptImporter {
    fn import(&self, path: &std::path::Path) -> Result<TranscriptDraft, ProviderError>;
}
