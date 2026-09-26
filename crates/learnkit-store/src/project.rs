use learnkit_core::shell::detect_default as default_shell_preference;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_FILE_NAME: &str = "learnkit.toml";
pub const LEARNKIT_DIR_NAME: &str = ".learnkit";

fn default_shell() -> String {
    default_shell_preference().to_string()
}

/// On-disk shape of `learnkit.toml`. Kept intentionally minimal for this
/// feature — only what `init`/`status` need (see `data-model.md` → Project).
///
/// `shell_preference` defaults on read for configs written before FR-015
/// existed, so old projects keep parsing without a migration step.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub profile_id: String,
    #[serde(default = "default_shell")]
    pub shell_preference: String,
}

/// A `Project` recomputed from the filesystem — never a cached value, per the
/// Hard Guards principle. `None` means "no valid project at this path".
#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub profile_id: String,
    pub shell_preference: String,
}

/// Reads and validates the project at `root`, if any.
///
/// Returns `Ok(None)` when there simply is no project yet (missing config),
/// and `Err` only for a genuine read/parse failure on an existing file.
pub fn read_project(root: &Path) -> std::io::Result<Option<Project>> {
    let config_path = root.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)?;
    let config: ProjectConfig = toml::from_str(&raw).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid {CONFIG_FILE_NAME}: {err}"),
        )
    })?;

    Ok(Some(Project {
        profile_id: config.profile_id,
        shell_preference: config.shell_preference,
    }))
}

pub fn learnkit_dir(root: &Path) -> std::path::PathBuf {
    root.join(LEARNKIT_DIR_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_project_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_project(dir.path()).unwrap(), None);
    }

    #[test]
    fn existing_project_is_read_back() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "profile_id = \"geography\"\nshell_preference = \"sh\"\n",
        )
        .unwrap();

        let project = read_project(dir.path()).unwrap().unwrap();
        assert_eq!(project.profile_id, "geography");
        assert_eq!(project.shell_preference, "sh");
    }

    #[test]
    fn pre_fr015_config_without_shell_preference_defaults_on_read() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "profile_id = \"generic\"\n",
        )
        .unwrap();

        let project = read_project(dir.path()).unwrap().unwrap();
        assert_eq!(project.shell_preference, default_shell());
    }
}
