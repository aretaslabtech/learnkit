use crate::catalog::{is_supported, supported_profiles};
use learnkit_core::error::LearnKitError;

/// Validates `profile_id` against the supported catalog.
///
/// Returns the validated id unchanged on success, or a `ProfileUnsupported`
/// error listing the supported alternatives (FR-011).
pub fn resolve(profile_id: &str) -> Result<String, LearnKitError> {
    if is_supported(profile_id) {
        Ok(profile_id.to_string())
    } else {
        Err(LearnKitError::ProfileUnsupported {
            requested: profile_id.to_string(),
            supported: supported_profiles(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_profile() {
        assert_eq!(resolve("geography").unwrap(), "geography");
    }

    #[test]
    fn rejects_unsupported_profile() {
        let err = resolve("no-existe").unwrap_err();
        assert_eq!(err.code(), "PROFILE_UNSUPPORTED");
    }
}
