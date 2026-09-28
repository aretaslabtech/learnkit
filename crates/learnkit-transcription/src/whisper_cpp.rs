use crate::provider::{
    EngineInfo, Segment, TranscriptDraft, TranscriptionProvider, TranscriptionRequest,
};
use learnkit_core::provider_error::ProviderError;
use serde::Deserialize;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_BINARY: &str = "whisper-cli";
pub const PROVIDER_NAME: &str = "whisper-cpp";

/// Invokes the `whisper.cpp` CLI as a subprocess, per `research.md` §1.
/// Never marks a phase as valid itself — only returns a `TranscriptDraft`
/// for the caller (`learnkit-transcription::transcript`) to normalize and
/// for `learnkit-workflow` to validate.
pub struct WhisperCppProvider {
    pub binary: String,
    pub model_path: PathBuf,
}

impl WhisperCppProvider {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            binary: DEFAULT_BINARY.to_string(),
            model_path,
        }
    }
}

impl TranscriptionProvider for WhisperCppProvider {
    fn transcribe(&self, request: &TranscriptionRequest) -> Result<TranscriptDraft, ProviderError> {
        let output = Command::new(&self.binary)
            .arg("-m")
            .arg(&self.model_path)
            .arg("-f")
            .arg(&request.audio_path)
            .arg("-oj")
            .output()
            .map_err(|err| {
                if err.kind() == ErrorKind::NotFound {
                    ProviderError::ToolNotFound {
                        tool: self.binary.clone(),
                    }
                } else {
                    ProviderError::ExecutionFailed {
                        provider: PROVIDER_NAME.to_string(),
                        message: err.to_string(),
                    }
                }
            })?;

        if !output.status.success() {
            return Err(ProviderError::ExecutionFailed {
                provider: PROVIDER_NAME.to_string(),
                message: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let json_path = format!("{}.json", request.audio_path.display());
        let raw =
            std::fs::read_to_string(&json_path).map_err(|err| ProviderError::ExecutionFailed {
                provider: PROVIDER_NAME.to_string(),
                message: format!("expected output at {json_path}: {err}"),
            })?;

        let model_name = self
            .model_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        parse_whisper_json(&raw, &model_name)
    }
}

#[derive(Debug, Deserialize)]
struct WhisperJsonOutput {
    transcription: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    offsets: WhisperOffsets,
    text: String,
}

#[derive(Debug, Deserialize)]
struct WhisperOffsets {
    from: u64,
    to: u64,
}

/// Parses whisper.cpp's `-oj` JSON output into our canonical `TranscriptDraft`
/// segments. Pure/testable independently of actually spawning the binary.
pub fn parse_whisper_json(raw: &str, model: &str) -> Result<TranscriptDraft, ProviderError> {
    let parsed: WhisperJsonOutput =
        serde_json::from_str(raw).map_err(|err| ProviderError::UnparsableOutput {
            provider: PROVIDER_NAME.to_string(),
            message: err.to_string(),
        })?;

    let segments = parsed
        .transcription
        .into_iter()
        .map(|s| Segment {
            start_ms: s.offsets.from,
            end_ms: s.offsets.to,
            text: s.text.trim().to_string(),
        })
        .collect();

    Ok(TranscriptDraft {
        segments,
        engine: EngineInfo::Generated {
            provider: PROVIDER_NAME.to_string(),
            model: model.to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_whisper_cpp_json_output() {
        let raw = r#"{
            "transcription": [
                {"offsets": {"from": 0, "to": 2000}, "text": " Hello there."},
                {"offsets": {"from": 2000, "to": 4500}, "text": " He got away with it."}
            ]
        }"#;

        let draft = parse_whisper_json(raw, "ggml-base.en.bin").unwrap();

        assert_eq!(draft.segments.len(), 2);
        assert_eq!(draft.segments[1].text, "He got away with it.");
        assert!(matches!(draft.engine, EngineInfo::Generated { .. }));
    }

    #[test]
    fn missing_tool_is_reported_as_tool_not_found() {
        let provider = WhisperCppProvider {
            binary: "definitely-not-a-real-binary-xyz".to_string(),
            model_path: PathBuf::from("model.bin"),
        };
        let request = TranscriptionRequest {
            audio_path: PathBuf::from("audio.wav"),
            language: None,
        };

        let err = provider.transcribe(&request).unwrap_err();
        assert!(matches!(err, ProviderError::ToolNotFound { .. }));
    }
}
