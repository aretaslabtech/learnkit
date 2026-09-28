use learnkit_core::provider_error::ProviderError;
use serde::Deserialize;

pub const PROVIDER_NAME: &str = "wikimedia-commons";
const API_BASE: &str = "https://commons.wikimedia.org/w/api.php";

/// Licenses accepted as "reuse with attribution" per `spec.md` → Assumptions
/// (Creative Commons family). Anything else is treated as not license-valid
/// and filtered out before it ever becomes a candidate. Matches Wikimedia
/// Commons' real `LicenseShortName` values, e.g. "CC BY-SA 4.0", "CC0 1.0",
/// "Public domain".
const ACCEPTED_LICENSE_PREFIXES: &[&str] = &["cc", "public domain"];

#[derive(Debug, Clone)]
pub struct ImageRequest {
    pub query: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageCandidate {
    pub url: String,
    pub author: String,
    pub license_name: String,
    pub license_url: String,
    pub source_url: String,
}

/// Never marks a workflow phase as valid by itself — see
/// `contracts/provider-traits.md` → Regla común. Returns `Ok(vec![])`, not
/// an error, when no license-valid candidate exists (FR-017e) — that is a
/// normal outcome, not a provider failure.
pub trait ImageProvider {
    fn search(&self, request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError>;
}

pub struct WikimediaCommonsProvider {
    client: reqwest::blocking::Client,
}

impl Default for WikimediaCommonsProvider {
    fn default() -> Self {
        Self {
            // A short timeout matters here: this is the one network call in
            // the whole feature, and it must never hang the CLI when there
            // is no connectivity (spec.md → Assumptions: only Wikimedia
            // Commons search requires network; everything else is offline).
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .user_agent("LearnKit/0.1 (https://github.com/aretaslabtech/learnkit)")
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        }
    }
}

/// Rejects a downloaded response that isn't actually an image — this is
/// exactly the check that was missing when Wikimedia's media servers
/// rejected an unauthenticated (no `User-Agent`) request and returned an
/// HTML/text error page that got silently accepted as image bytes
/// (`research.md` §4, real bug found during manual Anki verification).
fn is_image_content_type(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

fn is_license_acceptable(license_short_name: &str) -> bool {
    let lower = license_short_name.to_lowercase();
    ACCEPTED_LICENSE_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

impl WikimediaCommonsProvider {
    /// Downloads the actual bytes of a candidate's `url`, reusing this
    /// provider's own client (same `User-Agent`/timeout as the search call —
    /// Wikimedia's media servers enforce the same policy and reject
    /// requests without one, per `research.md` §4). Rejects a response
    /// whose `Content-Type` is not an image instead of silently accepting
    /// whatever came back (e.g. an HTML/text error page).
    pub fn download(&self, url: &str) -> Result<Vec<u8>, ProviderError> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|err| ProviderError::NetworkError {
                provider: PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if !is_image_content_type(&content_type) {
            return Err(ProviderError::UnparsableOutput {
                provider: PROVIDER_NAME.to_string(),
                message: format!("expected an image response, got content-type '{content_type}'"),
            });
        }

        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|err| ProviderError::NetworkError {
                provider: PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })
    }
}

impl ImageProvider for WikimediaCommonsProvider {
    fn search(&self, request: &ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError> {
        let response = self
            .client
            .get(API_BASE)
            .query(&[
                ("action", "query"),
                ("format", "json"),
                ("generator", "search"),
                ("gsrnamespace", "6"),
                ("gsrsearch", request.query.as_str()),
                ("gsrlimit", "5"),
                ("prop", "imageinfo"),
                ("iiprop", "url|extmetadata"),
                // Downloads a ~800px-wide thumbnail instead of the original
                // (Guía maestra §12.5 paso 6: 600-1000px, evitar originales
                // innecesariamente grandes). Wikimedia's API returns a
                // `thumburl` alongside `url` when this is set; `to_candidate`
                // prefers it, falling back to `url` if the API ever omits it.
                ("iiurlwidth", "800"),
            ])
            .send()
            .map_err(|err| ProviderError::NetworkError {
                provider: PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })?;

        let body: ApiResponse = response
            .json()
            .map_err(|err| ProviderError::UnparsableOutput {
                provider: PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })?;

        let candidates = body
            .query
            .and_then(|q| q.pages)
            .map(|pages| pages.into_values().collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|page| page.imageinfo.into_iter().next())
            .filter_map(to_candidate)
            .collect();

        Ok(candidates)
    }
}

fn to_candidate(info: ImageInfo) -> Option<ImageCandidate> {
    let meta = info.extmetadata?;
    let license_name = meta.license_short_name.value;
    if !is_license_acceptable(&license_name) {
        return None;
    }
    Some(ImageCandidate {
        url: info.thumburl.unwrap_or(info.url),
        author: meta
            .artist
            .map(|a| a.value)
            .unwrap_or_else(|| "unknown".to_string()),
        license_name,
        license_url: meta.license_url.map(|l| l.value).unwrap_or_default(),
        source_url: info.descriptionurl.unwrap_or_default(),
    })
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    query: Option<ApiQuery>,
}

#[derive(Debug, Deserialize)]
struct ApiQuery {
    pages: Option<std::collections::HashMap<String, ApiPage>>,
}

#[derive(Debug, Deserialize)]
struct ApiPage {
    #[serde(default)]
    imageinfo: Vec<ImageInfo>,
}

#[derive(Debug, Deserialize)]
struct ImageInfo {
    url: String,
    /// Present when the request includes `iiurlwidth` — a resized rendition
    /// at (up to) that width, preferred over the full-size `url`.
    #[serde(default)]
    thumburl: Option<String>,
    descriptionurl: Option<String>,
    extmetadata: Option<ExtMetadata>,
}

#[derive(Debug, Deserialize)]
struct ExtMetadata {
    #[serde(rename = "LicenseShortName")]
    license_short_name: MetaValue,
    #[serde(rename = "LicenseUrl")]
    license_url: Option<MetaValue>,
    #[serde(rename = "Artist")]
    artist: Option<MetaValue>,
}

#[derive(Debug, Deserialize)]
struct MetaValue {
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_image_content_type() {
        assert!(!is_image_content_type("text/html; charset=utf-8"));
        assert!(!is_image_content_type(""));
    }

    #[test]
    fn accepts_real_image_content_types() {
        assert!(is_image_content_type("image/jpeg"));
        assert!(is_image_content_type("image/png"));
    }

    #[test]
    fn accepts_creative_commons_licenses() {
        assert!(is_license_acceptable("CC BY-SA 4.0"));
        assert!(is_license_acceptable("cc0"));
        assert!(is_license_acceptable("Public Domain"));
    }

    #[test]
    fn rejects_non_free_licenses() {
        assert!(!is_license_acceptable("All rights reserved"));
        assert!(!is_license_acceptable(""));
    }

    #[test]
    fn to_candidate_prefers_thumburl_over_full_size_url() {
        let info = ImageInfo {
            url: "https://upload.wikimedia.org/original-huge.jpg".to_string(),
            thumburl: Some("https://upload.wikimedia.org/thumb/resized-800.jpg".to_string()),
            descriptionurl: None,
            extmetadata: Some(ExtMetadata {
                license_short_name: MetaValue {
                    value: "CC0".to_string(),
                },
                license_url: None,
                artist: None,
            }),
        };
        let candidate = to_candidate(info).unwrap();
        assert_eq!(candidate.url, "https://upload.wikimedia.org/thumb/resized-800.jpg");
    }

    #[test]
    fn to_candidate_falls_back_to_full_size_url_when_no_thumburl() {
        let info = ImageInfo {
            url: "https://upload.wikimedia.org/original.jpg".to_string(),
            thumburl: None,
            descriptionurl: None,
            extmetadata: Some(ExtMetadata {
                license_short_name: MetaValue {
                    value: "CC0".to_string(),
                },
                license_url: None,
                artist: None,
            }),
        };
        let candidate = to_candidate(info).unwrap();
        assert_eq!(candidate.url, "https://upload.wikimedia.org/original.jpg");
    }

    #[test]
    fn parses_api_response_into_candidates_filtering_by_license() {
        let raw = r#"{
            "query": {
                "pages": {
                    "1": {
                        "imageinfo": [{
                            "url": "https://upload.wikimedia.org/img.jpg",
                            "descriptionurl": "https://commons.wikimedia.org/wiki/File:img.jpg",
                            "extmetadata": {
                                "LicenseShortName": {"value": "CC BY-SA 4.0"},
                                "LicenseUrl": {"value": "https://creativecommons.org/licenses/by-sa/4.0"},
                                "Artist": {"value": "Jane Doe"}
                            }
                        }]
                    },
                    "2": {
                        "imageinfo": [{
                            "url": "https://upload.wikimedia.org/other.jpg",
                            "descriptionurl": null,
                            "extmetadata": {
                                "LicenseShortName": {"value": "All rights reserved"}
                            }
                        }]
                    }
                }
            }
        }"#;

        let response: ApiResponse = serde_json::from_str(raw).unwrap();
        let candidates: Vec<ImageCandidate> = response
            .query
            .and_then(|q| q.pages)
            .map(|pages| pages.into_values().collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|page| page.imageinfo.into_iter().next())
            .filter_map(to_candidate)
            .collect();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].author, "Jane Doe");
    }
}
