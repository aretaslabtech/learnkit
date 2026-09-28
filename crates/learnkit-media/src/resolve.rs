use crate::asset::{self, Asset, AssetOrigin, AssetType};
use crate::image::{ImageProvider, ImageRequest};
use crate::voice::{logical_fingerprint, VoiceProvider, VoiceRequest};
use std::path::Path;

/// Outcome of trying to resolve a card side's media requirement — never
/// fabricates a resource when none is available (FR-017e).
pub enum ResolvedMedia {
    Asset(Box<Asset>),
    Pending { reason: String },
}

/// Resolves an audio requirement: reuses `supplied_content` if given,
/// otherwise generates pronunciation audio via `voice_provider` (FR-017b).
pub fn resolve_audio(
    assets_dir: &Path,
    text: &str,
    locale: &str,
    voice_policy: &str,
    supplied_content: Option<&[u8]>,
    voice_provider: &dyn VoiceProvider,
) -> ResolvedMedia {
    if let Some(content) = supplied_content {
        let fingerprint = asset::content_hash(content);
        return match asset::register_or_reuse(
            assets_dir,
            &fingerprint,
            AssetType::Audio,
            content,
            "wav",
            "audio/wav",
            AssetOrigin::Supplied,
        ) {
            Ok(a) => ResolvedMedia::Asset(Box::new(a)),
            Err(err) => ResolvedMedia::Pending {
                reason: err.to_string(),
            },
        };
    }

    let request = VoiceRequest {
        text: text.to_string(),
        locale: locale.to_string(),
        voice_policy: voice_policy.to_string(),
    };

    match voice_provider.synthesize(&request) {
        Ok(draft) => {
            let fingerprint = logical_fingerprint(&request);
            match asset::register_or_reuse(
                assets_dir,
                &fingerprint,
                AssetType::Audio,
                &draft.content,
                draft.extension,
                draft.mime,
                AssetOrigin::Generated {
                    provider: draft.provider,
                    voice_policy: draft.voice_policy,
                },
            ) {
                Ok(a) => ResolvedMedia::Asset(Box::new(a)),
                Err(err) => ResolvedMedia::Pending {
                    reason: err.to_string(),
                },
            }
        }
        Err(err) => ResolvedMedia::Pending {
            reason: format!("voice provider failed: {err}"),
        },
    }
}

/// Resolves an image requirement: reuses `supplied_content` if given,
/// otherwise searches Wikimedia Commons (FR-017c/d/e). Downloading the
/// winning candidate's bytes is the caller's responsibility via
/// `download` — kept separate so this function stays testable without
/// network access.
pub fn resolve_image(
    assets_dir: &Path,
    query: &str,
    supplied_content: Option<&[u8]>,
    image_provider: &dyn ImageProvider,
    download: impl FnOnce(&str) -> std::io::Result<Vec<u8>>,
) -> ResolvedMedia {
    if let Some(content) = supplied_content {
        let fingerprint = asset::content_hash(content);
        return match asset::register_or_reuse(
            assets_dir,
            &fingerprint,
            AssetType::Image,
            content,
            "png",
            "image/png",
            AssetOrigin::Supplied,
        ) {
            Ok(a) => ResolvedMedia::Asset(Box::new(a)),
            Err(err) => ResolvedMedia::Pending {
                reason: err.to_string(),
            },
        };
    }

    let request = ImageRequest {
        query: query.to_string(),
    };

    const SEARCH_ATTEMPTS: u32 = 3;
    let mut last_err = None;
    let mut candidates = None;
    for attempt in 1..=SEARCH_ATTEMPTS {
        match image_provider.search(&request) {
            Ok(c) => {
                candidates = Some(c);
                break;
            }
            Err(err) => {
                last_err = Some(err);
                if attempt < SEARCH_ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(300 * attempt as u64));
                }
            }
        }
    }
    let candidates = match candidates {
        Some(c) => c,
        None => {
            return ResolvedMedia::Pending {
                reason: format!(
                    "image provider failed after {SEARCH_ATTEMPTS} attempts: {}",
                    last_err.expect("loop always sets last_err when candidates is None")
                ),
            }
        }
    };

    let Some(candidate) = candidates.into_iter().next() else {
        return ResolvedMedia::Pending {
            reason: "no license-compatible image found on Wikimedia Commons".to_string(),
        };
    };

    let content = match download(&candidate.url) {
        Ok(c) => c,
        Err(err) => {
            return ResolvedMedia::Pending {
                reason: format!("failed to download candidate image: {err}"),
            }
        }
    };

    let fingerprint = format!("wikimedia:{}", candidate.url);
    match asset::register_or_reuse(
        assets_dir,
        &fingerprint,
        AssetType::Image,
        &content,
        "jpg",
        "image/jpeg",
        AssetOrigin::Fetched {
            provider: crate::image::PROVIDER_NAME.to_string(),
            license_name: candidate.license_name,
            license_url: candidate.license_url,
            author: candidate.author,
            source_url: candidate.source_url,
        },
    ) {
        Ok(a) => ResolvedMedia::Asset(Box::new(a)),
        Err(err) => ResolvedMedia::Pending {
            reason: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageCandidate;
    use crate::voice::AudioAssetDraft;
    use learnkit_core::provider_error::ProviderError;

    struct FakeVoice;
    impl VoiceProvider for FakeVoice {
        fn synthesize(&self, _request: &VoiceRequest) -> Result<AudioAssetDraft, ProviderError> {
            Ok(AudioAssetDraft {
                content: vec![9u8; 32],
                mime: "audio/wav",
                extension: "wav",
                provider: "fake-voice".to_string(),
                voice_policy: "project-default".to_string(),
            })
        }
    }

    struct FakeImageWithResult;
    impl ImageProvider for FakeImageWithResult {
        fn search(&self, _request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError> {
            Ok(vec![ImageCandidate {
                url: "https://example.org/img.jpg".to_string(),
                author: "Jane Doe".to_string(),
                license_name: "CC BY-SA 4.0".to_string(),
                license_url: "https://creativecommons.org/licenses/by-sa/4.0".to_string(),
                source_url: "https://commons.wikimedia.org/wiki/File:img.jpg".to_string(),
            }])
        }
    }

    struct FakeImageNoResults;
    impl ImageProvider for FakeImageNoResults {
        fn search(&self, _request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError> {
            Ok(vec![])
        }
    }

    /// Fails the first `fail_times` calls with a transient-looking error,
    /// then succeeds — proves FR-017g's retry actually recovers instead of
    /// giving up on the first hiccup.
    struct FakeImageFailsThenSucceeds {
        fail_times: u32,
        calls: std::cell::Cell<u32>,
    }
    impl ImageProvider for FakeImageFailsThenSucceeds {
        fn search(&self, _request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError> {
            let call = self.calls.get();
            self.calls.set(call + 1);
            if call < self.fail_times {
                return Err(ProviderError::NetworkError {
                    provider: "fake".to_string(),
                    message: "transient failure".to_string(),
                });
            }
            Ok(vec![ImageCandidate {
                url: "https://example.org/img.jpg".to_string(),
                author: "Jane Doe".to_string(),
                license_name: "CC BY-SA 4.0".to_string(),
                license_url: "https://creativecommons.org/licenses/by-sa/4.0".to_string(),
                source_url: "https://commons.wikimedia.org/wiki/File:img.jpg".to_string(),
            }])
        }
    }

    struct FakeImageAlwaysFails;
    impl ImageProvider for FakeImageAlwaysFails {
        fn search(&self, _request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError> {
            Err(ProviderError::NetworkError {
                provider: "fake".to_string(),
                message: "always fails".to_string(),
            })
        }
    }

    #[test]
    fn resolve_audio_prefers_supplied_content() {
        let dir = tempfile::tempdir().unwrap();
        let content = vec![1u8; 32];

        let result = resolve_audio(
            dir.path(),
            "hello",
            "en-GB",
            "project-default",
            Some(&content),
            &FakeVoice,
        );

        match result {
            ResolvedMedia::Asset(asset) => assert!(matches!(asset.origin, AssetOrigin::Supplied)),
            ResolvedMedia::Pending { .. } => panic!("expected an asset"),
        }
    }

    #[test]
    fn resolve_audio_generates_when_nothing_supplied() {
        let dir = tempfile::tempdir().unwrap();

        let result = resolve_audio(
            dir.path(),
            "hello",
            "en-GB",
            "project-default",
            None,
            &FakeVoice,
        );

        match result {
            ResolvedMedia::Asset(asset) => {
                assert!(matches!(asset.origin, AssetOrigin::Generated { .. }))
            }
            ResolvedMedia::Pending { .. } => panic!("expected a generated asset"),
        }
    }

    #[test]
    fn resolve_image_fetches_from_wikimedia_with_license() {
        let dir = tempfile::tempdir().unwrap();

        let result = resolve_image(
            dir.path(),
            "whiteboard",
            None,
            &FakeImageWithResult,
            |_url| Ok(vec![2u8; 32]),
        );

        match result {
            ResolvedMedia::Asset(asset) => match asset.origin {
                AssetOrigin::Fetched { license_name, .. } => {
                    assert_eq!(license_name, "CC BY-SA 4.0")
                }
                _ => panic!("expected a fetched origin"),
            },
            ResolvedMedia::Pending { .. } => panic!("expected an asset"),
        }
    }

    #[test]
    fn resolve_image_retries_a_transient_provider_failure_and_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let provider = FakeImageFailsThenSucceeds {
            fail_times: 2,
            calls: std::cell::Cell::new(0),
        };

        let result = resolve_image(dir.path(), "whiteboard", None, &provider, |_url| {
            Ok(vec![2u8; 32])
        });

        assert!(matches!(result, ResolvedMedia::Asset(_)));
        assert_eq!(provider.calls.get(), 3);
    }

    #[test]
    fn resolve_image_gives_up_as_pending_after_exhausting_retries() {
        let dir = tempfile::tempdir().unwrap();

        let result = resolve_image(dir.path(), "whiteboard", None, &FakeImageAlwaysFails, |_url| {
            Ok(vec![2u8; 32])
        });

        match result {
            ResolvedMedia::Pending { reason } => assert!(reason.contains("3 attempts")),
            ResolvedMedia::Asset(_) => panic!("expected pending"),
        }
    }

    #[test]
    fn resolve_image_is_pending_when_no_candidates_found() {
        let dir = tempfile::tempdir().unwrap();

        let result = resolve_image(
            dir.path(),
            "whiteboard",
            None,
            &FakeImageNoResults,
            |_url| Ok(vec![2u8; 32]),
        );

        assert!(matches!(result, ResolvedMedia::Pending { .. }));
    }
}
