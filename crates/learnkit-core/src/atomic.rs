use std::fs;
use std::io;
use std::path::Path;

/// Writes `contents` to `path` atomically: write to a sibling temp file, then
/// rename over the destination. Never leaves a partially-written file at
/// `path` if the process is interrupted mid-write.
///
/// Per `research.md` §6 / `docs/10-storage-git.md` §5.
pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = tmp_path_for(path);
    fs::write(&tmp_path, contents)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn tmp_path_for(path: &Path) -> std::path::PathBuf {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{file_name}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_new_file_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested").join("file.toml");

        write_atomic(&target, b"hello = true\n").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "hello = true\n");
        assert!(!tmp_path_for(&target).exists());
    }

    #[test]
    fn overwrites_existing_file_without_leaving_temp() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("file.toml");

        write_atomic(&target, b"a = 1\n").unwrap();
        write_atomic(&target, b"a = 2\n").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "a = 2\n");
        assert!(!tmp_path_for(&target).exists());
    }
}
