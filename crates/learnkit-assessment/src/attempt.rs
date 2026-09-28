use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Immutable evidence of a single answer — per `data-model.md` → Attempt.
/// Lives at the project level (`attempts/attempts.jsonl`, per
/// `docs/10-storage-git.md §2`), never per-session, since progress is
/// meant to aggregate across a user's whole history (FR-025).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub attempt_id: String,
    pub assessment_id: String,
    pub item_id: String,
    pub learning_item_ids: Vec<String>,
    /// Milliseconds since the Unix epoch — a plain integer, not a debug-
    /// formatted timestamp, so attempts can be sorted chronologically for
    /// "recent N" progress aggregation (FR-025) without a date library.
    pub at_ms: u128,
    pub correct: bool,
    pub score: f64,
    pub skill: String,
}

/// Shape of a single answer inside an exam's downloaded `results.json`
/// (see `exam_html::render`'s embedded JS payload).
#[derive(Debug, Clone, Deserialize)]
struct ResultAttempt {
    item_id: String,
    skill: String,
    correct: bool,
    score: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct ResultsPayload {
    assessment_id: String,
    attempts: Vec<ResultAttempt>,
}

fn attempts_path(project_root: &Path) -> PathBuf {
    project_root.join("attempts").join("attempts.jsonl")
}

fn generate_attempt_id(item_id: &str, index: usize) -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(item_id.as_bytes());
    hasher.update(index.to_le_bytes());
    hasher.update(now_nanos.to_le_bytes());
    let digest = hasher.finalize();
    let suffix = digest
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("attempt-{suffix}")
}

/// Appends `attempts` to the project-wide log — never truncates or
/// overwrites existing lines (FR-024).
pub fn append(project_root: &Path, attempts: &[Attempt]) -> std::io::Result<()> {
    let path = attempts_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for attempt in attempts {
        let line = serde_json::to_string(attempt)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        writeln!(file, "{line}")?;
    }
    Ok(())
}

pub fn read_all(project_root: &Path) -> std::io::Result<Vec<Attempt>> {
    let path = attempts_path(project_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path)?;
    let mut attempts = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let attempt: Attempt = serde_json::from_str(&line)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        attempts.push(attempt);
    }
    Ok(attempts)
}

/// Parses a `results.json` (from the exam HTML's download button) and
/// appends its answers as canonical `Attempt` events. `item_learning_items`
/// maps each `AssessmentItem` id to the learning item(s) it evaluates, so
/// attempts can be aggregated per learning item later (FR-025) without
/// re-reading the question bank at query time.
pub fn import_results(
    project_root: &Path,
    results_json: &str,
    item_learning_items: &std::collections::HashMap<String, Vec<String>>,
) -> std::io::Result<usize> {
    let payload: ResultsPayload = serde_json::from_str(results_json)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let attempts: Vec<Attempt> = payload
        .attempts
        .into_iter()
        .enumerate()
        .map(|(index, r)| Attempt {
            attempt_id: generate_attempt_id(&r.item_id, index),
            assessment_id: payload.assessment_id.clone(),
            item_id: r.item_id.clone(),
            learning_item_ids: item_learning_items
                .get(&r.item_id)
                .cloned()
                .unwrap_or_default(),
            at_ms: now_ms,
            correct: r.correct,
            score: r.score,
            skill: r.skill,
        })
        .collect();

    let count = attempts.len();
    append(project_root, &attempts)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn append_is_additive_across_calls() {
        let dir = tempfile::tempdir().unwrap();
        let attempt = Attempt {
            attempt_id: "a1".to_string(),
            assessment_id: "exam-1".to_string(),
            item_id: "q-1".to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            at_ms: 0,
            correct: true,
            score: 1.0,
            skill: "recognition".to_string(),
        };

        append(dir.path(), std::slice::from_ref(&attempt)).unwrap();
        append(dir.path(), &[attempt]).unwrap();

        assert_eq!(read_all(dir.path()).unwrap().len(), 2);
    }

    #[test]
    fn import_results_parses_exam_download_payload() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
            "assessment_id": "exam-1",
            "attempts": [
                {"item_id": "q-recognition-li-1", "skill": "recognition", "selected_option_ids": ["opt-correct"], "correct": true, "score": 1.0},
                {"item_id": "q-production-li-1", "skill": "production", "selected_option_ids": [], "correct": false, "score": 0.0}
            ]
        }"#;
        let mut map = HashMap::new();
        map.insert("q-recognition-li-1".to_string(), vec!["li-1".to_string()]);
        map.insert("q-production-li-1".to_string(), vec!["li-1".to_string()]);

        let count = import_results(dir.path(), json, &map).unwrap();

        assert_eq!(count, 2);
        let attempts = read_all(dir.path()).unwrap();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].learning_item_ids, vec!["li-1".to_string()]);
    }

    #[test]
    fn reimporting_the_same_results_file_never_overwrites_previous_attempts() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{"assessment_id": "exam-1", "attempts": [{"item_id": "q-1", "skill": "recognition", "selected_option_ids": [], "correct": true, "score": 1.0}]}"#;
        let map = HashMap::new();

        import_results(dir.path(), json, &map).unwrap();
        import_results(dir.path(), json, &map).unwrap();

        assert_eq!(read_all(dir.path()).unwrap().len(), 2);
    }
}
