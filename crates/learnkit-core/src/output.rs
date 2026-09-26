use serde::Serialize;

/// Stable `{"ok", "code", ...}` envelope shared by every command's `--json`
/// output, per `docs/05-cli-spec.md` §13 and `contracts/cli-commands.md`.
///
/// `T` carries the command-specific fields (e.g. `profile_id`,
/// `installed_agents`) and is flattened alongside `ok`/`code`.
#[derive(Debug, Serialize)]
pub struct Envelope<T: Serialize> {
    pub ok: bool,
    pub code: String,
    #[serde(flatten)]
    pub data: T,
}

impl<T: Serialize> Envelope<T> {
    pub fn ok(code: impl Into<String>, data: T) -> Self {
        Self {
            ok: true,
            code: code.into(),
            data,
        }
    }

    pub fn err(code: impl Into<String>, data: T) -> Self {
        Self {
            ok: false,
            code: code.into(),
            data,
        }
    }

    /// Serializes the envelope as pretty JSON to stdout.
    pub fn print_json(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => println!("{json}"),
            Err(err) => eprintln!("failed to serialize --json output: {err}"),
        }
    }
}

/// Marker type for envelopes with no command-specific fields.
#[derive(Debug, Serialize)]
pub struct NoData {}
