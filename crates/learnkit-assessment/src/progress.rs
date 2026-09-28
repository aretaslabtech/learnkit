use crate::attempt::{read_all, Attempt};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// Number of most-recent attempts considered for "recent accuracy", per
/// group (learning item or skill). Not user-configurable in this feature —
/// see `spec.md` → Assumptions (no adaptive/ML tuning in V1).
const RECENT_WINDOW: usize = 10;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GroupProgress {
    pub key: String,
    pub accuracy_all_time: f64,
    pub accuracy_recent: f64,
    pub attempt_count: usize,
    pub last_attempt_at_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressReport {
    pub by_learning_item: Vec<GroupProgress>,
    pub by_skill: Vec<GroupProgress>,
}

fn accuracy(attempts: &[&Attempt]) -> f64 {
    if attempts.is_empty() {
        return 0.0;
    }
    let correct = attempts.iter().filter(|a| a.correct).count();
    correct as f64 / attempts.len() as f64
}

fn group_progress(key: &str, mut attempts: Vec<&Attempt>) -> GroupProgress {
    attempts.sort_by_key(|a| a.at_ms);
    let all_time = accuracy(&attempts);
    let recent_slice: Vec<&Attempt> = attempts.iter().rev().take(RECENT_WINDOW).copied().collect();
    let recent = accuracy(&recent_slice);
    let last_attempt_at_ms = attempts.last().map(|a| a.at_ms).unwrap_or(0);

    GroupProgress {
        key: key.to_string(),
        accuracy_all_time: all_time,
        accuracy_recent: recent,
        attempt_count: attempts.len(),
        last_attempt_at_ms,
    }
}

/// Recomputes progress from `attempts/attempts.jsonl` right now — never a
/// cached value (FR-026). `skill_filter` narrows the by-skill (and implied
/// by-learning-item) view to a single skill when given.
pub fn compute(project_root: &Path, skill_filter: Option<&str>) -> std::io::Result<ProgressReport> {
    let all_attempts = read_all(project_root)?;
    let attempts: Vec<&Attempt> = all_attempts
        .iter()
        .filter(|a| skill_filter.is_none_or(|s| a.skill == s))
        .collect();

    let mut by_item: HashMap<String, Vec<&Attempt>> = HashMap::new();
    let mut by_skill: HashMap<String, Vec<&Attempt>> = HashMap::new();

    for attempt in &attempts {
        for item_id in &attempt.learning_item_ids {
            by_item.entry(item_id.clone()).or_default().push(attempt);
        }
        by_skill
            .entry(attempt.skill.clone())
            .or_default()
            .push(attempt);
    }

    let mut by_learning_item: Vec<GroupProgress> = by_item
        .into_iter()
        .map(|(key, attempts)| group_progress(&key, attempts))
        .collect();
    by_learning_item.sort_by(|a, b| a.key.cmp(&b.key));

    let mut by_skill_progress: Vec<GroupProgress> = by_skill
        .into_iter()
        .map(|(key, attempts)| group_progress(&key, attempts))
        .collect();
    by_skill_progress.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(ProgressReport {
        by_learning_item,
        by_skill: by_skill_progress,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attempt::append;

    fn attempt(item_id: &str, li: &str, skill: &str, correct: bool, at_ms: u128) -> Attempt {
        Attempt {
            attempt_id: format!("a-{item_id}-{at_ms}"),
            assessment_id: "exam-1".to_string(),
            item_id: item_id.to_string(),
            learning_item_ids: vec![li.to_string()],
            at_ms,
            correct,
            score: if correct { 1.0 } else { 0.0 },
            skill: skill.to_string(),
        }
    }

    #[test]
    fn aggregates_accuracy_and_count_by_learning_item_and_skill() {
        let dir = tempfile::tempdir().unwrap();
        append(
            dir.path(),
            &[
                attempt("q-1", "li-1", "recognition", true, 1),
                attempt("q-2", "li-1", "production", false, 2),
                attempt("q-3", "li-2", "recognition", true, 3),
            ],
        )
        .unwrap();

        let report = compute(dir.path(), None).unwrap();

        let li1 = report
            .by_learning_item
            .iter()
            .find(|g| g.key == "li-1")
            .unwrap();
        assert_eq!(li1.attempt_count, 2);
        assert_eq!(li1.accuracy_all_time, 0.5);

        let recognition = report
            .by_skill
            .iter()
            .find(|g| g.key == "recognition")
            .unwrap();
        assert_eq!(recognition.attempt_count, 2);
        assert_eq!(recognition.accuracy_all_time, 1.0);
    }

    #[test]
    fn recomputes_on_every_call_reflecting_new_attempts() {
        let dir = tempfile::tempdir().unwrap();
        append(
            dir.path(),
            &[attempt("q-1", "li-1", "recognition", true, 1)],
        )
        .unwrap();
        let first = compute(dir.path(), None).unwrap();
        assert_eq!(first.by_learning_item[0].attempt_count, 1);

        append(
            dir.path(),
            &[attempt("q-1", "li-1", "recognition", false, 2)],
        )
        .unwrap();
        let second = compute(dir.path(), None).unwrap();
        assert_eq!(second.by_learning_item[0].attempt_count, 2);
    }

    #[test]
    fn skill_filter_narrows_the_view() {
        let dir = tempfile::tempdir().unwrap();
        append(
            dir.path(),
            &[
                attempt("q-1", "li-1", "recognition", true, 1),
                attempt("q-2", "li-1", "listening", true, 2),
            ],
        )
        .unwrap();

        let report = compute(dir.path(), Some("listening")).unwrap();
        assert_eq!(report.by_skill.len(), 1);
        assert_eq!(report.by_skill[0].key, "listening");
    }
}
