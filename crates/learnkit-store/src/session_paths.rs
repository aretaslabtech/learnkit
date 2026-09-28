use std::path::{Path, PathBuf};

/// Filesystem layout for a session, per
/// `learnkit-implementation-spec/docs/10-storage-git.md §2`.
pub struct SessionPaths {
    root: PathBuf,
}

impl SessionPaths {
    pub fn new(project_root: &Path, session_id: &str) -> Self {
        Self {
            root: project_root.join("sessions").join(session_id),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn session_yaml(&self) -> PathBuf {
        self.root.join("session.yaml")
    }

    pub fn input(&self) -> PathBuf {
        self.root.join("input")
    }

    pub fn inventory(&self) -> PathBuf {
        self.root.join("inventory")
    }

    pub fn analysis(&self) -> PathBuf {
        self.root.join("analysis")
    }

    pub fn consolidated(&self) -> PathBuf {
        self.root.join("consolidated")
    }

    pub fn knowledge(&self) -> PathBuf {
        self.root.join("knowledge")
    }

    pub fn cards(&self) -> PathBuf {
        self.root.join("cards")
    }

    pub fn assessments(&self) -> PathBuf {
        self.root.join("assessments")
    }

    pub fn assets(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn validation(&self) -> PathBuf {
        self.root.join("validation")
    }

    pub fn dist(&self) -> PathBuf {
        self.root.join("dist")
    }

    pub fn phase_manifest(&self, phase: &str) -> PathBuf {
        self.validation().join(format!("{phase}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_relative_layout() {
        let paths = SessionPaths::new(Path::new("/proj"), "s1");
        assert_eq!(paths.root(), Path::new("/proj/sessions/s1"));
        assert_eq!(paths.input(), Path::new("/proj/sessions/s1/input"));
        assert_eq!(
            paths.phase_manifest("inventory"),
            Path::new("/proj/sessions/s1/validation/inventory.json")
        );
    }
}
