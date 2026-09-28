use clap::{Args, Subcommand};
use learnkit_core::error::LearnKitError;
use learnkit_core::output::Envelope;
use learnkit_store::project::read_project;
use learnkit_store::session_paths::SessionPaths;
use learnkit_transcription::transcript::active_transcript;
use learnkit_workflow::inventory::{inventory, SourceKind};
use learnkit_workflow::session::{create_session, list_sessions, read_session};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    action: SessionAction,
}

#[derive(Subcommand)]
enum SessionAction {
    /// Create a new session (and mark it as the active one).
    New(NewArgs),
    /// List existing sessions.
    List(ListArgs),
    /// Show a session's detail (defaults to the active session).
    Show(ShowArgs),
    /// Mark an existing session as the active one.
    Use(UseArgs),
}

#[derive(Args)]
struct UseArgs {
    session_id: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct NewArgs {
    title: String,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ShowArgs {
    session_id: Option<String>,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize, Default)]
struct SessionData {
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sessions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sources: Option<Vec<SourceSummary>>,
    /// Notes/transcript text — what the agent Skill (`learnkit-language`)
    /// reads to suggest vocabulary candidates (research.md §2).
    #[serde(skip_serializing_if = "Option::is_none")]
    text_material: Option<Vec<TextMaterial>>,
}

#[derive(Debug, Serialize)]
struct SourceSummary {
    id: String,
    kind: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct TextMaterial {
    source_id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcript_segments: Option<Vec<TranscriptSegmentSummary>>,
}

#[derive(Debug, Serialize)]
struct TranscriptSegmentSummary {
    start_ms: u64,
    end_ms: u64,
    text: String,
}

pub fn run(args: SessionArgs) -> i32 {
    match args.action {
        SessionAction::New(a) => run_new(a),
        SessionAction::List(a) => run_list(a),
        SessionAction::Show(a) => run_show(a),
        SessionAction::Use(a) => run_use(a),
    }
}

fn run_new(args: NewArgs) -> i32 {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));

    let profile = match resolve_profile(&root, args.profile.as_deref()) {
        Ok(p) => p,
        Err(err) => return emit_error(args.json, err),
    };

    match create_session(&root, &args.title, &profile) {
        Ok(session) => {
            // A newly created session becomes the active one (FR-003b) —
            // mirrors `.specify/feature.json` picking up the just-created
            // feature. Never fatal: if this write fails, the session itself
            // was still created successfully; the user can fix it with
            // `session use` afterwards.
            let _ = learnkit_store::active_session::set_active(&root, &session.id);

            if args.json {
                Envelope::ok(
                    "SESSION_CREATED",
                    SessionData {
                        session_id: Some(session.id),
                        title: Some(session.title),
                        profile: Some(session.profile),
                        ..Default::default()
                    },
                )
                .print_json();
            } else {
                println!(
                    "Sesión creada: {} ({}) — marcada como sesión activa.",
                    session.id, session.title
                );
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

fn run_use(args: UseArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    match read_session(&root, &args.session_id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ProjectNotInitialized {
                    path: format!("session '{}' not found", args.session_id),
                },
            )
        }
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

    match learnkit_store::active_session::set_active(&root, &args.session_id) {
        Ok(()) => {
            if args.json {
                Envelope::ok(
                    "ACTIVE_SESSION_SET",
                    SessionData {
                        session_id: Some(args.session_id.clone()),
                        ..Default::default()
                    },
                )
                .print_json();
            } else {
                println!("Sesión activa: {}", args.session_id);
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

fn run_list(args: ListArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    match list_sessions(&root) {
        Ok(sessions) => {
            let ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
            if args.json {
                Envelope::ok(
                    "SESSIONS_LISTED",
                    SessionData {
                        sessions: Some(ids),
                        ..Default::default()
                    },
                )
                .print_json();
            } else if sessions.is_empty() {
                println!("No hay sesiones todavía.");
            } else {
                for s in sessions {
                    println!("{}\t{}", s.id, s.title);
                }
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

fn run_show(args: ShowArgs) -> i32 {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));

    let session_id = match crate::session_context::resolve_session(&root, args.session_id) {
        Ok(id) => id,
        Err(err) => return emit_error(args.json, err),
    };

    let session = match read_session(&root, &session_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return emit_error(
                args.json,
                LearnKitError::ProjectNotInitialized {
                    path: format!("session '{session_id}' not found"),
                },
            )
        }
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

    let sources = inventory(&root, &session_id).unwrap_or_default();
    let session_paths = SessionPaths::new(&root, &session_id);

    let text_material: Vec<TextMaterial> = sources
        .iter()
        .filter_map(|s| match s.source.kind {
            SourceKind::Text => {
                let full_path = session_paths.root().join(&s.source.path);
                std::fs::read_to_string(full_path)
                    .ok()
                    .map(|text| TextMaterial {
                        source_id: s.source.id.clone(),
                        kind: "text".to_string(),
                        notes_text: Some(text),
                        transcript_segments: None,
                    })
            }
            SourceKind::Audio => active_transcript(session_paths.root(), &s.source.id)
                .ok()
                .flatten()
                .map(|transcript| TextMaterial {
                    source_id: s.source.id.clone(),
                    kind: "audio".to_string(),
                    notes_text: None,
                    transcript_segments: Some(
                        transcript
                            .segments
                            .into_iter()
                            .map(|seg| TranscriptSegmentSummary {
                                start_ms: seg.start_ms,
                                end_ms: seg.end_ms,
                                text: seg.text,
                            })
                            .collect(),
                    ),
                }),
            _ => None,
        })
        .collect();

    let source_summaries: Vec<SourceSummary> = sources
        .into_iter()
        .map(|s| SourceSummary {
            id: s.source.id,
            kind: format!("{:?}", s.source.kind).to_lowercase(),
            path: s.source.path,
            sha256: s.source.sha256,
        })
        .collect();

    if args.json {
        Envelope::ok(
            "SESSION_READY",
            SessionData {
                session_id: Some(session.id),
                title: Some(session.title),
                profile: Some(session.profile),
                sources: Some(source_summaries),
                text_material: Some(text_material),
                ..Default::default()
            },
        )
        .print_json();
    } else {
        println!("Sesión: {} ({})", session.id, session.title);
        println!("Perfil: {}", session.profile);
        println!("Fuentes: {}", source_summaries.len());
    }
    0
}

fn resolve_profile(root: &Path, explicit: Option<&str>) -> Result<String, LearnKitError> {
    if let Some(p) = explicit {
        return Ok(p.to_string());
    }
    match read_project(root) {
        Ok(Some(project)) => Ok(project.profile_id),
        Ok(None) => Ok(learnkit_profile::catalog::DEFAULT_PROFILE.to_string()),
        Err(source) => Err(LearnKitError::Filesystem {
            path: root.display().to_string(),
            source,
        }),
    }
}

fn emit_error(json: bool, err: LearnKitError) -> i32 {
    if json {
        Envelope::err(err.code(), SessionData::default()).print_json();
    } else {
        eprintln!("Error: {err}");
    }
    err.exit_code()
}
