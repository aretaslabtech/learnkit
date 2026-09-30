use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The single accumulated language level for a project — deliberately one
/// file, not a history: `odd/tasks/language-study-pack.md` T4/decision 3.
/// `set` overwrites it wholesale each time; there is only ever one "current"
/// level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageLevel {
    pub language: String,
    pub variety: String,
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// ISO 8601 UTC timestamp, e.g. `2026-09-29T12:34:56Z`.
    pub updated_at: String,
}

fn level_path(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("language-level.yaml")
}

/// Formats the current time as an ISO 8601 UTC timestamp
/// (`YYYY-MM-DDTHH:MM:SSZ`) without pulling in a date/time crate — the
/// workspace has none today (`odd/tasks/language-study-pack.md` T4).
/// Converts seconds-since-epoch to a civil (Gregorian) date using Howard
/// Hinnant's well-known `civil_from_days` algorithm.
fn iso8601_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = (secs / 86_400) as i64;
    let time_of_day = secs % 86_400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    let (year, month, day) = civil_from_days(days);

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days since the Unix epoch (1970-01-01) -> (year, month, day). See
/// <http://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Overwrites the persisted `LanguageLevel` wholesale — `learn level set`.
pub fn set(
    project_root: &Path,
    language: &str,
    variety: &str,
    level: &str,
    notes: Option<&str>,
) -> std::io::Result<LanguageLevel> {
    let entry = LanguageLevel {
        language: language.to_string(),
        variety: variety.to_string(),
        level: level.to_string(),
        notes: notes.map(|s| s.to_string()),
        updated_at: iso8601_now(),
    };

    let yaml = serde_yaml::to_string(&entry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&level_path(project_root), yaml.as_bytes())?;

    Ok(entry)
}

/// Reads the persisted `LanguageLevel`, if any has ever been set — `learn
/// level show`. `None` (not an error) means no level has been set yet.
pub fn show(project_root: &Path) -> std::io::Result<Option<LanguageLevel>> {
    let path = level_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let entry: LanguageLevel = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_returns_none_when_never_set() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(show(dir.path()).unwrap(), None);
    }

    #[test]
    fn set_persists_and_show_reads_it_back() {
        let dir = tempfile::tempdir().unwrap();
        let level = set(
            dir.path(),
            "en",
            "en-GB",
            "B1",
            Some("solid on past tenses, weak on conditionals"),
        )
        .unwrap();

        assert_eq!(level.level, "B1");
        assert!(!level.updated_at.is_empty());

        let reloaded = show(dir.path()).unwrap().unwrap();
        assert_eq!(reloaded, level);
    }

    #[test]
    fn set_overwrites_the_previous_level_wholesale() {
        let dir = tempfile::tempdir().unwrap();
        set(dir.path(), "en", "en-GB", "A2", None).unwrap();
        set(dir.path(), "en", "en-GB", "B1", None).unwrap();

        let reloaded = show(dir.path()).unwrap().unwrap();
        assert_eq!(reloaded.level, "B1");
    }

    #[test]
    fn iso8601_now_produces_a_well_formed_timestamp() {
        let ts = iso8601_now();
        // e.g. "2026-09-29T12:34:56Z"
        assert_eq!(ts.len(), 20);
        assert_eq!(ts.as_bytes()[4], b'-');
        assert_eq!(ts.as_bytes()[7], b'-');
        assert_eq!(ts.as_bytes()[10], b'T');
        assert_eq!(ts.as_bytes()[13], b':');
        assert_eq!(ts.as_bytes()[16], b':');
        assert_eq!(ts.as_bytes()[19], b'Z');
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        // 1970-01-01 is day 0.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 is a well-known reference point in the algorithm.
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
    }
}
