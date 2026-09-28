use crate::asset::content_hash;
use learnkit_core::provider_error::ProviderError;
use std::io::ErrorKind;
use std::process::Command;
use std::time::{Duration, Instant};

pub const PROVIDER_NAME: &str = "piper";
pub const DEFAULT_BINARY: &str = "piper";

/// Hard ceiling on how long a single `piper` invocation may run. Without
/// this, a misbehaving/hung subprocess (e.g. waiting on stdin it never
/// receives correctly) would block `cards build` forever with no way for
/// the user to tell it apart from "just slow" — found during real usage
/// with a 153-item vocabulary bank (research.md, cards build progress fix).
const PIPER_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct VoiceRequest {
    pub text: String,
    pub locale: String,
    pub voice_policy: String,
}

#[derive(Debug, Clone)]
pub struct AudioAssetDraft {
    pub content: Vec<u8>,
    pub mime: &'static str,
    pub extension: &'static str,
    pub provider: String,
    pub voice_policy: String,
}

/// `hash(normalized_text + locale + voice_policy)` per
/// `docs/07-anki-media.md §6` — lets the caller (`learnkit-media::resolve`
/// / `learnkit-cards`) dedup via `asset::register_or_reuse` (FR-016).
pub fn logical_fingerprint(request: &VoiceRequest) -> String {
    let normalized_text = request.text.trim().to_lowercase();
    content_hash(
        format!(
            "{normalized_text}|{}|{}",
            request.locale, request.voice_policy
        )
        .as_bytes(),
    )
}

/// Never marks a workflow phase as valid by itself — see
/// `contracts/provider-traits.md` → Regla común.
pub trait VoiceProvider {
    fn synthesize(&self, request: &VoiceRequest) -> Result<AudioAssetDraft, ProviderError>;
}

/// Invokes the `piper` CLI as a subprocess, per `research.md` §3.
pub struct PiperVoiceProvider {
    pub binary: String,
    pub model_path: std::path::PathBuf,
}

impl PiperVoiceProvider {
    pub fn new(model_path: std::path::PathBuf) -> Self {
        Self {
            binary: DEFAULT_BINARY.to_string(),
            model_path,
        }
    }
}

impl VoiceProvider for PiperVoiceProvider {
    fn synthesize(&self, request: &VoiceRequest) -> Result<AudioAssetDraft, ProviderError> {
        let out_dir = std::env::temp_dir();
        let out_path = out_dir.join(format!("piper-{}.wav", logical_fingerprint(request)));

        let mut child = Command::new(&self.binary)
            .arg("--model")
            .arg(&self.model_path)
            .arg("--output_file")
            .arg(&out_path)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == ErrorKind::NotFound {
                    ProviderError::ToolNotFound {
                        tool: self.binary.clone(),
                    }
                } else {
                    ProviderError::ExecutionFailed {
                        provider: PROVIDER_NAME.to_string(),
                        message: err.to_string(),
                    }
                }
            })?;

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            let _ = stdin.write_all(request.text.as_bytes());
        }

        let status = wait_with_timeout(&mut child, PIPER_TIMEOUT)?;

        if !status.success() {
            return Err(ProviderError::ExecutionFailed {
                provider: PROVIDER_NAME.to_string(),
                message: "piper exited with a non-zero status".to_string(),
            });
        }

        let content = std::fs::read(&out_path).map_err(|err| ProviderError::ExecutionFailed {
            provider: PROVIDER_NAME.to_string(),
            message: format!("expected output at {}: {err}", out_path.display()),
        })?;

        Ok(AudioAssetDraft {
            content,
            mime: "audio/wav",
            extension: "wav",
            provider: PROVIDER_NAME.to_string(),
            voice_policy: request.voice_policy.clone(),
        })
    }
}

pub const REST_PROVIDER_NAME: &str = "rest-tts";
const DEFAULT_BASE_URL: &str = "https://tts.davidpalazon.net";
const DEFAULT_VOICE: &str = "en-GB-SoniaNeural";
const API_KEY_ENV: &str = "TTS_API_KEY";
const REST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(serde::Serialize)]
struct SpeechRequestBody<'a> {
    model: &'a str,
    voice: &'a str,
    input: &'a str,
    response_format: &'a str,
}

/// Calls a REST TTS service compatible with OpenAI's `/v1/audio/speech`
/// endpoint (reference implementation: `travisvn/openai-edge-tts`, wrapping
/// Microsoft Edge's online neural voices via `edge-tts`) — the default
/// voice provider as of `research.md` §3 (revised 2026-09-27): `Piper`
/// assumed a local binary + model that turned out not to be installed on
/// the real machine used for study, while this REST service was already
/// deployed and verified by the user. `PiperVoiceProvider` remains
/// available for a fully offline flow.
pub struct RestVoiceProvider {
    client: reqwest::blocking::Client,
    base_url: String,
    voice: String,
    api_key: Option<String>,
}

impl Default for RestVoiceProvider {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(REST_TIMEOUT)
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            base_url: DEFAULT_BASE_URL.to_string(),
            voice: DEFAULT_VOICE.to_string(),
            api_key: std::env::var(API_KEY_ENV).ok(),
        }
    }
}

impl RestVoiceProvider {
    /// For tests and for callers who need a non-default deployment (a
    /// different base URL, voice, or an explicitly-supplied key rather than
    /// reading `TTS_API_KEY`).
    pub fn with_config(base_url: String, voice: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(REST_TIMEOUT)
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            base_url,
            voice,
            api_key,
        }
    }
}

impl VoiceProvider for RestVoiceProvider {
    fn synthesize(&self, request: &VoiceRequest) -> Result<AudioAssetDraft, ProviderError> {
        let api_key = self.api_key.as_ref().ok_or_else(|| ProviderError::ExecutionFailed {
            provider: REST_PROVIDER_NAME.to_string(),
            message: format!("missing {API_KEY_ENV} environment variable"),
        })?;

        let body = SpeechRequestBody {
            model: "tts-1",
            voice: &self.voice,
            input: &request.text,
            response_format: "mp3",
        };

        let response = self
            .client
            .post(format!("{}/v1/audio/speech", self.base_url))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .map_err(|err| ProviderError::NetworkError {
                provider: REST_PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(ProviderError::ExecutionFailed {
                provider: REST_PROVIDER_NAME.to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let content = response
            .bytes()
            .map_err(|err| ProviderError::NetworkError {
                provider: REST_PROVIDER_NAME.to_string(),
                message: err.to_string(),
            })?
            .to_vec();

        if content.is_empty() {
            return Err(ProviderError::ExecutionFailed {
                provider: REST_PROVIDER_NAME.to_string(),
                message: "empty audio response".to_string(),
            });
        }

        Ok(AudioAssetDraft {
            content,
            mime: "audio/mpeg",
            extension: "mp3",
            provider: REST_PROVIDER_NAME.to_string(),
            voice_policy: request.voice_policy.clone(),
        })
    }
}

/// Polls `child` for completion instead of a blocking `wait()`, so a hung
/// subprocess is killed and reported as a provider failure after `timeout`
/// rather than blocking the whole command indefinitely.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, ProviderError> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ProviderError::ExecutionFailed {
                        provider: PROVIDER_NAME.to_string(),
                        message: format!("piper did not finish within {timeout:?} — killed"),
                    });
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(err) => {
                return Err(ProviderError::ExecutionFailed {
                    provider: PROVIDER_NAME.to_string(),
                    message: err.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_fingerprint_is_stable_for_equivalent_requests() {
        let a = VoiceRequest {
            text: " Get away with ".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };
        let b = VoiceRequest {
            text: "get away with".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };

        assert_eq!(logical_fingerprint(&a), logical_fingerprint(&b));
    }

    #[test]
    fn different_text_has_a_different_fingerprint() {
        let a = VoiceRequest {
            text: "get away with".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };
        let b = VoiceRequest {
            text: "put up with".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };

        assert_ne!(logical_fingerprint(&a), logical_fingerprint(&b));
    }

    /// Spawns a one-shot local TCP server that writes back exactly
    /// `raw_response` (a full HTTP/1.1 response, status line + headers +
    /// body) to whatever connects to it, and returns its base URL. Good
    /// enough for these tests: we only need to exercise `RestVoiceProvider`'s
    /// own response handling, never a real TTS backend.
    fn spawn_fake_server(raw_response: Vec<u8>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::{Read, Write};
                let mut buf = [0u8; 4096];
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(&raw_response);
                let _ = stream.flush();
            }
        });
        format!("http://127.0.0.1:{port}")
    }

    fn sample_request() -> VoiceRequest {
        VoiceRequest {
            text: "hello".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        }
    }

    #[test]
    fn rest_provider_returns_response_bytes_on_success() {
        let body: &[u8] = b"fake-mp3-bytes";
        let mut raw = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        raw.extend_from_slice(body);
        let base_url = spawn_fake_server(raw);

        let provider =
            RestVoiceProvider::with_config(base_url, DEFAULT_VOICE.to_string(), Some("test-key".to_string()));

        let draft = provider.synthesize(&sample_request()).unwrap();
        assert_eq!(draft.content, body);
        assert_eq!(draft.mime, "audio/mpeg");
        assert_eq!(draft.provider, REST_PROVIDER_NAME);
    }

    #[test]
    fn rest_provider_reports_http_errors_from_the_service() {
        let raw = b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
        let base_url = spawn_fake_server(raw);

        let provider =
            RestVoiceProvider::with_config(base_url, DEFAULT_VOICE.to_string(), Some("bad-key".to_string()));

        let err = provider.synthesize(&sample_request()).unwrap_err();
        assert!(matches!(err, ProviderError::ExecutionFailed { .. }));
    }

    #[test]
    fn rest_provider_without_api_key_fails_without_making_a_network_call() {
        let provider = RestVoiceProvider::with_config(
            "http://127.0.0.1:1".to_string(),
            DEFAULT_VOICE.to_string(),
            None,
        );
        let request = VoiceRequest {
            text: "hello".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };

        let err = provider.synthesize(&request).unwrap_err();
        assert!(matches!(err, ProviderError::ExecutionFailed { .. }));
    }

    #[test]
    fn wait_with_timeout_returns_promptly_on_normal_exit() {
        let mut child = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", "exit 0"])
                .spawn()
                .unwrap()
        } else {
            std::process::Command::new("true").spawn().unwrap()
        };

        let status = wait_with_timeout(&mut child, Duration::from_secs(5)).unwrap();
        assert!(status.success());
    }

    #[test]
    fn wait_with_timeout_kills_a_process_that_outlives_the_timeout() {
        let mut child = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", "ping -n 60 127.0.0.1 >NUL"])
                .spawn()
                .unwrap()
        } else {
            std::process::Command::new("sleep").arg("60").spawn().unwrap()
        };

        let err = wait_with_timeout(&mut child, Duration::from_millis(200)).unwrap_err();
        assert!(matches!(err, ProviderError::ExecutionFailed { .. }));
    }

    #[test]
    fn missing_tool_is_reported_as_tool_not_found() {
        let provider = PiperVoiceProvider {
            binary: "definitely-not-a-real-binary-xyz".to_string(),
            model_path: std::path::PathBuf::from("voice.onnx"),
        };
        let request = VoiceRequest {
            text: "hello".to_string(),
            locale: "en-GB".to_string(),
            voice_policy: "project-default".to_string(),
        };

        let err = provider.synthesize(&request).unwrap_err();
        assert!(matches!(err, ProviderError::ToolNotFound { .. }));
    }
}
