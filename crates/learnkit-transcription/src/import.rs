use crate::provider::{EngineInfo, Segment, TranscriptDraft, TranscriptImporter};
use learnkit_core::provider_error::ProviderError;
use std::path::Path;

pub struct SubtitleImporter;

impl TranscriptImporter for SubtitleImporter {
    fn import(&self, path: &Path) -> Result<TranscriptDraft, ProviderError> {
        let raw = std::fs::read_to_string(path).map_err(|err| ProviderError::ExecutionFailed {
            provider: "import".to_string(),
            message: err.to_string(),
        })?;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let segments = match extension.as_str() {
            "srt" => parse_srt(&raw)?,
            "vtt" => parse_vtt(&raw)?,
            "txt" => vec![Segment {
                start_ms: 0,
                end_ms: 0,
                text: raw.trim().to_string(),
            }],
            "json" => return import_canonical_json(&raw),
            other => {
                return Err(ProviderError::UnparsableOutput {
                    provider: "import".to_string(),
                    message: format!("unsupported transcript import format: .{other}"),
                })
            }
        };

        Ok(TranscriptDraft {
            segments,
            engine: EngineInfo::Imported {
                imported_from: path.display().to_string(),
            },
        })
    }
}

fn import_canonical_json(raw: &str) -> Result<TranscriptDraft, ProviderError> {
    let segments: Vec<Segment> =
        serde_json::from_str(raw).map_err(|err| ProviderError::UnparsableOutput {
            provider: "import".to_string(),
            message: err.to_string(),
        })?;
    Ok(TranscriptDraft {
        segments,
        engine: EngineInfo::Imported {
            imported_from: "json".to_string(),
        },
    })
}

/// Parses `HH:MM:SS,mmm` (SRT) or `HH:MM:SS.mmm` (VTT) into milliseconds.
fn parse_timestamp(raw: &str) -> Option<u64> {
    let raw = raw.trim().replace(',', ".");
    let (hms, millis) = raw.split_once('.')?;
    let mut parts = hms.split(':');
    let hours: u64 = parts.next()?.parse().ok()?;
    let minutes: u64 = parts.next()?.parse().ok()?;
    let seconds: u64 = parts.next()?.parse().ok()?;
    let millis: u64 = millis.parse().ok()?;
    Some(((hours * 3600 + minutes * 60 + seconds) * 1000) + millis)
}

fn parse_time_range(line: &str) -> Option<(u64, u64)> {
    let (from, to) = line.split_once("-->")?;
    Some((
        parse_timestamp(from)?,
        parse_timestamp(to.split_whitespace().next().unwrap_or(to))?,
    ))
}

fn parse_srt(raw: &str) -> Result<Vec<Segment>, ProviderError> {
    let mut segments = Vec::new();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        // Optional numeric index line, then the time range line.
        let time_line = if line.trim().chars().all(|c| c.is_ascii_digit()) {
            match lines.next() {
                Some(l) => l,
                None => break,
            }
        } else {
            line
        };

        let Some((start_ms, end_ms)) = parse_time_range(time_line) else {
            continue;
        };

        let mut text_lines = Vec::new();
        while let Some(next) = lines.peek() {
            if next.trim().is_empty() {
                break;
            }
            text_lines.push(lines.next().unwrap().trim().to_string());
        }

        segments.push(Segment {
            start_ms,
            end_ms,
            text: text_lines.join(" "),
        });
    }

    Ok(segments)
}

fn parse_vtt(raw: &str) -> Result<Vec<Segment>, ProviderError> {
    let body = raw.strip_prefix("WEBVTT").unwrap_or(raw);
    parse_srt(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_srt_with_two_segments() {
        let raw = "1\n00:00:00,000 --> 00:00:02,000\nHello there.\n\n2\n00:00:02,000 --> 00:00:04,500\nHe got away with it.\n";
        let segments = parse_srt(raw).unwrap();

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 2000);
        assert_eq!(segments[1].text, "He got away with it.");
    }

    #[test]
    fn parses_vtt_with_dot_separated_millis() {
        let raw = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\nHello there.\n";
        let segments = parse_vtt(raw).unwrap();

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].end_ms, 2000);
    }

    #[test]
    fn importer_marks_engine_as_imported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clase.srt");
        std::fs::write(&path, "1\n00:00:00,000 --> 00:00:01,000\nHi.\n").unwrap();

        let draft = SubtitleImporter.import(&path).unwrap();
        assert!(matches!(draft.engine, EngineInfo::Imported { .. }));
        assert_eq!(draft.segments.len(), 1);
    }
}
