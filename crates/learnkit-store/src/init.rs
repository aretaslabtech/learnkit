use crate::project::{learnkit_dir, ProjectConfig, CONFIG_FILE_NAME};
use learnkit_core::atomic::write_atomic;
use std::path::Path;

/// Writes (or re-confirms, idempotently) the minimal project scaffold:
/// `learnkit.toml`, `.learnkit/workflow.toml`, and `.learnkit/profiles/<id>/`.
///
/// `profile_id` MUST already be validated (see `learnkit_profile::profile::resolve`).
/// Only ever writes files inside `root`/`.learnkit` or `root/learnkit.toml` —
/// never touches unrelated files (FR-007).
pub fn scaffold_project(
    root: &Path,
    profile_id: &str,
    shell_preference: &str,
) -> std::io::Result<()> {
    let config = ProjectConfig {
        profile_id: profile_id.to_string(),
        shell_preference: shell_preference.to_string(),
    };
    let config_toml = toml::to_string_pretty(&config)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    write_atomic(&root.join(CONFIG_FILE_NAME), config_toml.as_bytes())?;

    let dir = learnkit_dir(root);
    write_atomic(
        &dir.join("workflow.toml"),
        b"# LearnKit workflow definition (V1 placeholder).\n",
    )?;
    write_atomic(
        &dir.join("profiles").join(profile_id).join("profile.toml"),
        format!("id = \"{profile_id}\"\n").as_bytes(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::read_project;

    #[test]
    fn scaffolds_a_new_project() {
        let dir = tempfile::tempdir().unwrap();

        scaffold_project(dir.path(), "generic", "sh").unwrap();

        let project = read_project(dir.path()).unwrap().unwrap();
        assert_eq!(project.profile_id, "generic");
        assert_eq!(project.shell_preference, "sh");
        assert!(dir.path().join(".learnkit/workflow.toml").exists());
        assert!(dir
            .path()
            .join(".learnkit/profiles/generic/profile.toml")
            .exists());
    }

    #[test]
    fn scaffolding_twice_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();

        scaffold_project(dir.path(), "generic", "sh").unwrap();
        scaffold_project(dir.path(), "generic", "sh").unwrap();

        let project = read_project(dir.path()).unwrap().unwrap();
        assert_eq!(project.profile_id, "generic");
    }

    #[test]
    fn does_not_touch_unrelated_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "otro proyecto\n").unwrap();

        scaffold_project(dir.path(), "generic", "sh").unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "otro proyecto\n"
        );
    }
}
