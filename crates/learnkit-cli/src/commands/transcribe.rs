use crate::session_context::resolve_session;
use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_store::session_paths::SessionPaths;
use learnkit_transcription::import::SubtitleImporter;
use learnkit_transcription::provider::TranscriptImporter as _;
use learnkit_transcription::provider::{TranscriptionProvider, TranscriptionRequest};
use learnkit_transcription::transcript::{is_stale, save_transcript};
use learnkit_transcription::whisper_cpp::WhisperCppProvider;
use learnkit_workflow::inventory::{inventory, SourceKind};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct TranscribeArgs {
    #[command(subcommand)]
    action: Option<TranscribeAction>,

    /// Session whose audio sources should be transcribed.
    #[arg(long)]
    session: Option<String>,

    /// Only transcribe this source id (defaults to every audio source).
    #[arg(long)]
    source: Option<String>,

    /// Path to the whisper.cpp model file.
    #[arg(long)]
    model: Option<PathBuf>,

    #[arg(long)]
    path: Option<PathBuf>,

    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum TranscribeAction {
    /// Import an existing transcription file (SRT/VTT/TXT/JSON) instead of generating one.
    Import(ImportArgs),
}

#[derive(Args)]
struct ImportArgs {
    file: PathBuf,
    #[arg(long)]
    source: String,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct TranscriptReport {
    source_id: String,
    transcript_id: String,
    segments: usize,
}

#[derive(Debug, Serialize, Default)]
struct TranscribeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    transcripts: Option<Vec<TranscriptReport>>,
}

pub fn run(args: TranscribeArgs) -> i32 {
    if let Some(TranscribeAction::Import(import_args)) = args.action {
        return run_import(import_args);
    }

    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session.clone()) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let session_root_paths = SessionPaths::new(&root, &session_id);

    let sources = match inventory(&root, &session_id) {
        Ok(s) => s,
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    let model_path = args
        .model
        .unwrap_or_else(|| PathBuf::from("models/ggml-base.en.bin"));
    let provider = WhisperCppProvider::new(model_path);

    let mut reports = Vec::new();
    for inventoried in sources
        .iter()
        .filter(|s| s.source.kind == SourceKind::Audio)
        .filter(|s| args.source.as_deref().is_none_or(|id| id == s.source.id))
    {
        let already_current = is_stale(
            session_root_paths.root(),
            &inventoried.source.id,
            &inventoried.source.sha256,
        )
        .ok()
        .flatten()
            == Some(false);
        if already_current {
            continue;
        }

        let audio_path = session_root_paths.root().join(&inventoried.source.path);
        let request = TranscriptionRequest {
            audio_path,
            language: None,
        };

        match provider.transcribe(&request) {
            Ok(draft) => {
                match save_transcript(
                    session_root_paths.root(),
                    &inventoried.source.id,
                    &inventoried.source.sha256,
                    draft,
                ) {
                    Ok(transcript) => reports.push(TranscriptReport {
                        source_id: transcript.source_id,
                        transcript_id: transcript.transcript_id,
                        segments: transcript.segments.len(),
                    }),
                    Err(source) => {
                        return emit_error(
                            args.json,
                            LearnKitError::Filesystem {
                                path: root.display().to_string(),
                                source,
                            },
                        )
                    }
                }
            }
            Err(provider_error) => {
                return emit_error(args.json, LearnKitError::Provider(provider_error));
            }
        }
    }

    if args.json {
        Envelope::ok(
            "TRANSCRIPTION_COMPLETE",
            TranscribeData {
                transcripts: Some(reports),
            },
        )
        .print_json();
    } else {
        for r in &reports {
            println!(
                "{}\t{}\t{} segmento(s)",
                r.source_id, r.transcript_id, r.segments
            );
        }
    }
    0
}

fn run_import(args: ImportArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let session_id = match resolve_session(&root, args.session) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };
    let paths = SessionPaths::new(&root, &session_id);

    let draft = match SubtitleImporter.import(&args.file) {
        Ok(d) => d,
        Err(err) => return emit_error(args.json, LearnKitError::Provider(err)),
    };

    // The fingerprint recorded on an imported transcript MUST be the audio
    // source's real current hash (re-inventoried here), never the previous
    // transcript's fingerprint — otherwise a first import would record an
    // empty fingerprint and every future staleness check would be wrong.
    let current_fingerprint = match inventory(&root, &session_id) {
        Ok(sources) => sources
            .iter()
            .find(|s| s.source.id == args.source)
            .map(|s| s.source.sha256.clone())
            .unwrap_or_default(),
        Err(source) => {
            return emit_error(
                args.json,
                LearnKitError::Filesystem {
                    path: root.display().to_string(),
                    source,
                },
            )
        }
    };

    match save_transcript(paths.root(), &args.source, &current_fingerprint, draft) {
        Ok(transcript) => {
            if args.json {
                Envelope::ok(
                    "TRANSCRIPTION_IMPORTED",
                    TranscribeData {
                        transcripts: Some(vec![TranscriptReport {
                            source_id: transcript.source_id,
                            transcript_id: transcript.transcript_id.clone(),
                            segments: transcript.segments.len(),
                        }]),
                    },
                )
                .print_json();
            } else {
                println!("Transcripción importada: {}", transcript.transcript_id);
            }
            0
        }
        Err(source) => emit_error(
            args.json,
            LearnKitError::Filesystem {
                path: root.display().to_string(),
                source,
            },
        ),
    }
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), TranscribeData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
