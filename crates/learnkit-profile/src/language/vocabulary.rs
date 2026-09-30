use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sense {
    pub gloss: String,
}

/// A usage example for a `VocabularyEntry` — FR-013b. Mirrors `Sense` in
/// style: a single-field wrapper, kept as a struct (not a bare `String`) so
/// it can grow (e.g. a source locator) without a breaking format change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Example {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabularyEntry {
    pub id: String,
    pub language: String,
    pub variety: String,
    pub lemma: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of_speech: Option<String>,
    pub senses: Vec<Sense>,
    pub sources: Vec<SourceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_by: Option<String>,
    /// IPA transcription — FR-013b. Optional: absence never blocks a card
    /// being complete, but it's shown on the card back when present.
    /// `#[serde(default)]` so an already-persisted entry without this key
    /// still deserializes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipa: Option<String>,
    /// Usage examples — FR-013b. Same backward-compatibility contract as
    /// `ipa`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Example>,
    /// Free-text topic/unit this entry belongs to (e.g. "Unit 5 — Food") —
    /// `odd/tasks/language-study-pack.md` T1. Same backward-compatibility
    /// contract as `ipa`/`examples`: `#[serde(default)]` so an
    /// already-persisted entry without this key still deserializes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Free-text notes about this entry — same contract as `topic`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

fn vocabulary_dir(project_root: &Path) -> PathBuf {
    project_root.join("knowledge").join("vocabulary")
}

fn normalize_lemma(lemma: &str) -> String {
    lemma.trim().to_lowercase()
}

fn slug_id(lemma: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for ch in normalize_lemma(lemma).chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(lemma.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    format!("vocab-en-{slug}-{}", &hash[..6])
}

fn load_all(project_root: &Path) -> std::io::Result<Vec<VocabularyEntry>> {
    let dir = vocabulary_dir(project_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let vocab: VocabularyEntry = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        entries.push(vocab);
    }
    Ok(entries)
}

fn save(project_root: &Path, entry: &VocabularyEntry) -> std::io::Result<()> {
    let path = vocabulary_dir(project_root).join(format!("{}.yaml", entry.id));
    let yaml = serde_yaml::to_string(entry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&path, yaml.as_bytes())
}

/// Lists every persisted `VocabularyEntry`, project-wide — used by
/// `learnkit export study-pack` (`odd/tasks/language-study-pack.md` T8) to
/// populate the "Vocabulario"/"Expresiones" tabs. Minimal public wrapper
/// around the existing private `load_all`; no change to the data model or
/// persistence format.
pub fn list_all(project_root: &Path) -> std::io::Result<Vec<VocabularyEntry>> {
    load_all(project_root)
}

/// Finds an existing entry with the same normalized `lemma`, project-wide
/// (vocabulary survives across sessions — FR-011).
pub fn find_by_lemma(project_root: &Path, lemma: &str) -> std::io::Result<Option<VocabularyEntry>> {
    let target = normalize_lemma(lemma);
    Ok(load_all(project_root)?
        .into_iter()
        .find(|e| normalize_lemma(&e.lemma) == target))
}

/// Creates a new `VocabularyEntry`, or reuses (and extends the source
/// traceability of) an existing one with the same `lemma` — FR-008/FR-009.
pub fn add_or_reuse(
    project_root: &Path,
    lemma: &str,
    gloss: &str,
    source_id: &str,
    locator: Option<&str>,
    suggested_by: Option<&str>,
    variety: &str,
) -> std::io::Result<VocabularyEntry> {
    let new_source = SourceRef {
        source_id: source_id.to_string(),
        locator: locator.map(|s| s.to_string()),
    };

    if let Some(mut existing) = find_by_lemma(project_root, lemma)? {
        if !existing.sources.contains(&new_source) {
            existing.sources.push(new_source);
            save(project_root, &existing)?;
        }
        return Ok(existing);
    }

    let entry = VocabularyEntry {
        id: slug_id(lemma),
        language: "en".to_string(),
        variety: variety.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: None,
        senses: vec![Sense {
            gloss: gloss.to_string(),
        }],
        sources: vec![new_source],
        suggested_by: suggested_by.map(|s| s.to_string()),
        ipa: None,
        examples: Vec::new(),
        topic: None,
        notes: None,
    };
    save(project_root, &entry)?;
    Ok(entry)
}

/// Finds the persisted entry with this exact `id`, if any — used by `cards
/// build` (T109) to look up `ipa`/`examples` for a `LearningItem` via its
/// `vocabulary_entry_id`, since `LearningItem` stays generic and never
/// carries language-profile-specific fields (Principle VI).
pub fn find_by_id(project_root: &Path, id: &str) -> std::io::Result<Option<VocabularyEntry>> {
    let path = vocabulary_dir(project_root).join(format!("{id}.yaml"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let entry: VocabularyEntry = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(entry))
}

/// Sets `ipa`/`examples` on a persisted entry and saves it — `learn
/// vocabulary add --ipa/--example` (T107). Only touches the fields actually
/// supplied (`None`/empty leaves the existing persisted value alone), so a
/// later `add_or_reuse` call for the same lemma without these flags never
/// wipes out previously recorded IPA/examples.
#[allow(clippy::too_many_arguments)]
pub fn set_details(
    project_root: &Path,
    entry: &mut VocabularyEntry,
    ipa: Option<&str>,
    examples: &[String],
    topic: Option<&str>,
    notes: Option<&str>,
) -> std::io::Result<()> {
    let mut changed = false;
    if let Some(ipa) = ipa {
        entry.ipa = Some(ipa.to_string());
        changed = true;
    }
    if !examples.is_empty() {
        entry.examples = examples
            .iter()
            .map(|text| Example {
                text: text.clone(),
            })
            .collect();
        changed = true;
    }
    if let Some(topic) = topic {
        entry.topic = Some(topic.to_string());
        changed = true;
    }
    if let Some(notes) = notes {
        entry.notes = Some(notes.to_string());
        changed = true;
    }
    if changed {
        save(project_root, entry)?;
    }
    Ok(())
}

/// Applies an in-place edit to a persisted entry and saves it — `learn
/// vocabulary edit` (T113/FR-012d). Unlike `set_details` (used by `add`,
/// where an absent flag simply means "nothing new to record yet"), here
/// every parameter is an explicit "was this flag supplied" choice from the
/// CLI layer: `None`/`false` always means "leave this field exactly as
/// persisted," never "clear it." `examples` replaces the list only when
/// `Some` (i.e. `--example` was passed at least once, or `--clear-examples`
/// was passed, in which case the CLI passes `Some(&[])`); `None` leaves the
/// existing examples untouched. `id` and `sources` are never touched here —
/// editing must never affect entry identity or traceability (FR-012d).
#[allow(clippy::too_many_arguments)]
pub fn set_fields(
    project_root: &Path,
    entry: &mut VocabularyEntry,
    sense: Option<&str>,
    part_of_speech: Option<&str>,
    ipa: Option<&str>,
    examples: Option<&[String]>,
    topic: Option<&str>,
    notes: Option<&str>,
) -> std::io::Result<()> {
    let mut changed = false;
    if let Some(sense) = sense {
        if let Some(first) = entry.senses.first_mut() {
            first.gloss = sense.to_string();
        } else {
            entry.senses.push(Sense {
                gloss: sense.to_string(),
            });
        }
        changed = true;
    }
    if let Some(part_of_speech) = part_of_speech {
        entry.part_of_speech = Some(part_of_speech.to_string());
        changed = true;
    }
    if let Some(ipa) = ipa {
        entry.ipa = Some(ipa.to_string());
        changed = true;
    }
    if let Some(examples) = examples {
        entry.examples = examples
            .iter()
            .map(|text| Example {
                text: text.clone(),
            })
            .collect();
        changed = true;
    }
    if let Some(topic) = topic {
        entry.topic = Some(topic.to_string());
        changed = true;
    }
    if let Some(notes) = notes {
        entry.notes = Some(notes.to_string());
        changed = true;
    }
    if changed {
        save(project_root, entry)?;
    }
    Ok(())
}

/// Deletes the persisted `VocabularyEntry` with this `id`, if any —
/// FR-012c. Returns `false` (not an error) when it didn't exist, so a
/// caller can distinguish "already gone" from a real filesystem failure.
pub fn remove(project_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = vocabulary_dir(project_root).join(format!("{id}.yaml"));
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_new_entry_with_traceability() {
        let dir = tempfile::tempdir().unwrap();
        let entry = add_or_reuse(
            dir.path(),
            "get away with",
            "hacer algo malo sin castigo",
            "src-transcript-1",
            Some("segment:00:12:34"),
            Some("agent"),
            "en-GB",
        )
        .unwrap();

        assert_eq!(entry.lemma, "get away with");
        assert_eq!(entry.sources.len(), 1);
        assert_eq!(entry.suggested_by, Some("agent".to_string()));
    }

    #[test]
    fn reuses_entry_with_same_lemma_and_adds_new_source() {
        let dir = tempfile::tempdir().unwrap();
        let first = add_or_reuse(
            dir.path(),
            "get away with",
            "gloss",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();
        let second = add_or_reuse(
            dir.path(),
            "Get Away With",
            "gloss",
            "src-2",
            None,
            None,
            "en-GB",
        )
        .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.sources.len(), 2);
    }

    #[test]
    fn manual_entry_without_any_suggestion_works() {
        let dir = tempfile::tempdir().unwrap();
        let entry = add_or_reuse(
            dir.path(),
            "whiteboard",
            "pizarra",
            "src-notes",
            None,
            None,
            "en-GB",
        )
        .unwrap();
        assert_eq!(entry.suggested_by, None);
    }

    /// T106: an entry persisted before `ipa`/`examples` existed must still
    /// deserialize — `#[serde(default)]` is what makes this safe.
    #[test]
    fn deserializes_a_legacy_entry_missing_ipa_and_examples() {
        let yaml = r#"
id: vocab-en-whiteboard-abc123
language: en
variety: en-GB
lemma: whiteboard
senses:
  - gloss: pizarra
sources:
  - source_id: src-1
"#;
        let entry: VocabularyEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.lemma, "whiteboard");
        assert_eq!(entry.ipa, None);
        assert!(entry.examples.is_empty());
    }

    #[test]
    fn set_details_persists_ipa_and_examples_without_overwriting_when_omitted() {
        let dir = tempfile::tempdir().unwrap();
        let mut entry = add_or_reuse(
            dir.path(),
            "whiteboard",
            "pizarra",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();

        set_details(
            dir.path(),
            &mut entry,
            Some("ˈwaɪtbɔːd"),
            &["Write it on the whiteboard.".to_string()],
            None,
            None,
        )
        .unwrap();

        assert_eq!(entry.ipa.as_deref(), Some("ˈwaɪtbɔːd"));
        assert_eq!(entry.examples.len(), 1);
        assert_eq!(entry.examples[0].text, "Write it on the whiteboard.");

        let reloaded = find_by_id(dir.path(), &entry.id).unwrap().unwrap();
        assert_eq!(reloaded.ipa.as_deref(), Some("ˈwaɪtbɔːd"));

        // Calling again with nothing supplied must not wipe the existing values.
        set_details(dir.path(), &mut entry, None, &[], None, None).unwrap();
        assert_eq!(entry.ipa.as_deref(), Some("ˈwaɪtbɔːd"));
        assert_eq!(entry.examples.len(), 1);
    }

    /// T1 (language-study-pack): `topic`/`notes` are persisted on creation
    /// (via `set_details`) and can be edited afterwards (via `set_fields`)
    /// without disturbing other fields.
    #[test]
    fn topic_and_notes_are_persisted_and_editable() {
        let dir = tempfile::tempdir().unwrap();
        let mut entry = add_or_reuse(
            dir.path(),
            "get away with",
            "hacer algo malo sin castigo",
            "src-1",
            None,
            None,
            "en-GB",
        )
        .unwrap();
        assert_eq!(entry.topic, None);
        assert_eq!(entry.notes, None);

        set_details(
            dir.path(),
            &mut entry,
            None,
            &[],
            Some("Unit 5 — Idioms"),
            Some("frequently confused with 'get away'"),
        )
        .unwrap();

        assert_eq!(entry.topic.as_deref(), Some("Unit 5 — Idioms"));
        assert_eq!(
            entry.notes.as_deref(),
            Some("frequently confused with 'get away'")
        );

        let reloaded = find_by_id(dir.path(), &entry.id).unwrap().unwrap();
        assert_eq!(reloaded.topic.as_deref(), Some("Unit 5 — Idioms"));

        set_fields(
            dir.path(),
            &mut entry,
            None,
            None,
            None,
            None,
            Some("Unit 6 — Phrasal verbs"),
            None,
        )
        .unwrap();

        assert_eq!(entry.topic.as_deref(), Some("Unit 6 — Phrasal verbs"));
        // notes untouched by the edit above.
        assert_eq!(
            entry.notes.as_deref(),
            Some("frequently confused with 'get away'")
        );
    }

    /// T1: a YAML entry persisted before `topic`/`notes` existed must still
    /// deserialize — `#[serde(default)]` is what makes this safe.
    #[test]
    fn deserializes_a_legacy_entry_missing_topic_and_notes() {
        let yaml = r#"
id: vocab-en-whiteboard-abc123
language: en
variety: en-GB
lemma: whiteboard
senses:
  - gloss: pizarra
sources:
  - source_id: src-1
"#;
        let entry: VocabularyEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.lemma, "whiteboard");
        assert_eq!(entry.topic, None);
        assert_eq!(entry.notes, None);
    }

    #[test]
    fn find_by_id_returns_none_for_unknown_id() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(find_by_id(dir.path(), "does-not-exist").unwrap(), None);
    }
}
