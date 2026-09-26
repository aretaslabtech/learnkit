/// Shell preferences supported by `learnkit init --shell` (FR-015).
///
/// Purely informational in this feature — see `spec.md` → Assumptions: no
/// script generation depends on this value yet.
pub const SUPPORTED_SHELLS: &[&str] = &["sh", "ps"];

pub fn is_supported(shell: &str) -> bool {
    SUPPORTED_SHELLS.contains(&shell)
}

pub fn supported_shells() -> Vec<String> {
    SUPPORTED_SHELLS.iter().map(|s| s.to_string()).collect()
}

/// Detects a reasonable default shell preference from the build target OS,
/// per `research.md` §11: `ps` on Windows, `sh` everywhere else.
pub fn detect_default() -> &'static str {
    if cfg!(target_os = "windows") {
        "ps"
    } else {
        "sh"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected_default_is_supported() {
        assert!(is_supported(detect_default()));
    }

    #[test]
    fn unknown_shell_is_not_supported() {
        assert!(!is_supported("fish"));
    }
}
