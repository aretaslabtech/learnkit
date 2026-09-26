/// Profiles supported at this point in the implementation, per
/// `learnkit-implementation-spec/docs/06-profiles.md`.
///
/// Order matters: it is the order shown in the interactive menu, and the
/// first entry is the default (`generic`).
pub const SUPPORTED_PROFILES: &[&str] = &["generic", "language", "geography", "godot"];

pub const DEFAULT_PROFILE: &str = "generic";

pub fn is_supported(profile_id: &str) -> bool {
    SUPPORTED_PROFILES.contains(&profile_id)
}

pub fn supported_profiles() -> Vec<String> {
    SUPPORTED_PROFILES.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_is_supported_and_default() {
        assert!(is_supported(DEFAULT_PROFILE));
        assert_eq!(DEFAULT_PROFILE, "generic");
    }

    #[test]
    fn unknown_profile_is_not_supported() {
        assert!(!is_supported("no-existe"));
    }
}
