use crate::provider::{EngineInfo, Segment, TranscriptDraft};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Canonical, persisted form of a transcript — see `data-model.md` → Transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub transcript_id: String,
    pub source_id: String,
    pub engine: EngineInfo,
    pub source_fingerprint: String,
    pub segments: Vec<Segment>,
}

fn transcripts_dir(session_root: &Path, source_id: &str) -> PathBuf {
    session_root
        .join("analysis")
        .join("transcripts")
        .join(source_id)
}

fn active_pointer_path(session_root: &Path, source_id: &str) -> PathBuf {
    transcripts_dir(session_root, source_id).join("active.json")
}

fn generate_transcript_id(source_id: &str) -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update(now_nanos.to_le_bytes());
    let digest = hasher.finalize();
    let suffix = digest
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("tr_{suffix}")
}

/// Persists `draft` as a brand-new transcript version for `source_id` and
/// marks it active — never overwrites a previous version (FR-006).
pub fn save_transcript(
    session_root: &Path,
    source_id: &str,
    source_fingerprint: &str,
    draft: TranscriptDraft,
) -> std::io::Result<Transcript> {
    let transcript = Transcript {
        transcript_id: generate_transcript_id(source_id),
        source_id: source_id.to_string(),
        engine: draft.engine,
        source_fingerprint: source_fingerprint.to_string(),
        segments: draft.segments,
    };

    let dir = transcripts_dir(session_root, source_id);
    let file_path = dir.join(format!("{}.json", transcript.transcript_id));
    let json = serde_json::to_string_pretty(&transcript)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&file_path, json.as_bytes())?;

    let pointer = serde_json::json!({ "active_transcript_id": transcript.transcript_id });
    learnkit_core::atomic::write_atomic(
        &active_pointer_path(session_root, source_id),
        serde_json::to_string_pretty(&pointer).unwrap().as_bytes(),
    )?;

    Ok(transcript)
}

/// Reads the currently active transcript for `source_id`, if any.
pub fn active_transcript(
    session_root: &Path,
    source_id: &str,
) -> std::io::Result<Option<Transcript>> {
    let pointer_path = active_pointer_path(session_root, source_id);
    if !pointer_path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&pointer_path)?;
    let pointer: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let active_id = pointer["active_transcript_id"].as_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing active_transcript_id",
        )
    })?;

    let transcript_path =
        transcripts_dir(session_root, source_id).join(format!("{active_id}.json"));
    let raw = fs::read_to_string(transcript_path)?;
    let transcript: Transcript = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(transcript))
}

/// Whether the active transcript for `source_id` no longer matches the
/// audio source's current fingerprint (FR-006). `None` means there is no
/// active transcript at all (a distinct case from "stale").
pub fn is_stale(
    session_root: &Path,
    source_id: &str,
    current_source_fingerprint: &str,
) -> std::io::Result<Option<bool>> {
    Ok(active_transcript(session_root, source_id)?
        .map(|t| t.source_fingerprint != current_source_fingerprint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EngineInfo;

    fn draft() -> TranscriptDraft {
        TranscriptDraft {
            segments: vec![Segment {
                start_ms: 0,
                end_ms: 1000,
                text: "hello".to_string(),
            }],
            engine: EngineInfo::Generated {
                provider: "whisper-cpp".to_string(),
                model: "base".to_string(),
            },
        }
    }

    #[test]
    fn saves_and_reads_back_active_transcript() {
        let dir = tempfile::tempdir().unwrap();
        let saved = save_transcript(dir.path(), "src-001", "fp1", draft()).unwrap();

        let active = active_transcript(dir.path(), "src-001").unwrap().unwrap();
        assert_eq!(active.transcript_id, saved.transcript_id);
        assert_eq!(active.segments.len(), 1);
    }

    #[test]
    fn saving_again_creates_a_new_version_without_deleting_the_old_one() {
        let dir = tempfile::tempdir().unwrap();
        let first = save_transcript(dir.path(), "src-001", "fp1", draft()).unwrap();
        let second = save_transcript(dir.path(), "src-001", "fp2", draft()).unwrap();

        assert_ne!(first.transcript_id, second.transcript_id);
        let first_path =
            transcripts_dir(dir.path(), "src-001").join(format!("{}.json", first.transcript_id));
        assert!(first_path.exists());

        let active = active_transcript(dir.path(), "src-001").unwrap().unwrap();
        assert_eq!(active.transcript_id, second.transcript_id);
    }

    #[test]
    fn detects_staleness_when_fingerprint_changes() {
        let dir = tempfile::tempdir().unwrap();
        save_transcript(dir.path(), "src-001", "fp1", draft()).unwrap();

        assert_eq!(is_stale(dir.path(), "src-001", "fp1").unwrap(), Some(false));
        assert_eq!(
            is_stale(dir.path(), "src-001", "fp-changed").unwrap(),
            Some(true)
        );
    }

    #[test]
    fn missing_transcript_is_none_not_stale() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(is_stale(dir.path(), "src-001", "fp1").unwrap(), None);
    }
}
