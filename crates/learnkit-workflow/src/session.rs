use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub profile: String,
    pub created_at: String,
    pub source_dir: String,
    pub workflow: String,
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = true;
    for ch in title.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn generate_id(title: &str) -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();

    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(now_nanos.to_le_bytes());
    let digest = hasher.finalize();
    let suffix = digest
        .iter()
        .take(4)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let slug = slugify(title);
    if slug.is_empty() {
        suffix
    } else {
        format!("{slug}-{suffix}")
    }
}

fn session_yaml_path(project_root: &Path, session_id: &str) -> std::path::PathBuf {
    learnkit_store::session_paths::SessionPaths::new(project_root, session_id).session_yaml()
}

/// Creates a new session with its own directory, per `data-model.md` → Session.
pub fn create_session(project_root: &Path, title: &str, profile: &str) -> std::io::Result<Session> {
    let id = generate_id(title);
    let paths = learnkit_store::session_paths::SessionPaths::new(project_root, &id);

    let session = Session {
        id: id.clone(),
        title: title.to_string(),
        profile: profile.to_string(),
        created_at: format!("{:?}", SystemTime::now()),
        source_dir: "input".to_string(),
        workflow: "default".to_string(),
    };

    let yaml = serde_yaml::to_string(&session)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&paths.session_yaml(), yaml.as_bytes())?;
    fs::create_dir_all(paths.input())?;

    Ok(session)
}

pub fn read_session(project_root: &Path, session_id: &str) -> std::io::Result<Option<Session>> {
    let path = session_yaml_path(project_root, session_id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let session: Session = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(session))
}

/// Lists every session under `project_root/sessions/`.
pub fn list_sessions(project_root: &Path) -> std::io::Result<Vec<Session>> {
    let sessions_dir = project_root.join("sessions");
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if let Some(session) = read_session(project_root, &id)? {
            sessions.push(session);
        }
    }
    sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_session_with_stable_id_and_input_dir() {
        let dir = tempfile::tempdir().unwrap();

        let session = create_session(dir.path(), "EOI — Unit 5", "language-en").unwrap();

        assert!(!session.id.is_empty());
        assert_eq!(session.profile, "language-en");
        let paths = learnkit_store::session_paths::SessionPaths::new(dir.path(), &session.id);
        assert!(paths.session_yaml().exists());
        assert!(paths.input().is_dir());
    }

    #[test]
    fn reads_back_a_created_session() {
        let dir = tempfile::tempdir().unwrap();
        let created = create_session(dir.path(), "Unit 5", "language-en").unwrap();

        let read_back = read_session(dir.path(), &created.id).unwrap().unwrap();
        assert_eq!(read_back, created);
    }

    #[test]
    fn missing_session_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_session(dir.path(), "does-not-exist").unwrap(), None);
    }

    #[test]
    fn lists_all_created_sessions() {
        let dir = tempfile::tempdir().unwrap();
        create_session(dir.path(), "Unit 5", "language-en").unwrap();
        create_session(dir.path(), "Unit 6", "language-en").unwrap();

        let sessions = list_sessions(dir.path()).unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
