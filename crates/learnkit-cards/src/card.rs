use crate::template::{find, MediaPolicy};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Text { value: String },
    Image { asset_id: Option<String> },
    Audio { asset_id: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Side {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardDefinition {
    pub id: String,
    pub learning_item_ids: Vec<String>,
    pub template: String,
    pub front: Side,
    pub back: Side,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Completeness {
    Complete,
    PendingImage { side: &'static str, reason: String },
    PendingAudio { side: &'static str, reason: String },
}

fn cards_dir(session_root: &Path) -> PathBuf {
    session_root.join("cards")
}

fn card_path(session_root: &Path, id: &str) -> PathBuf {
    cards_dir(session_root).join(format!("{id}.yaml"))
}

pub fn save(session_root: &Path, card: &CardDefinition) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(card)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&card_path(session_root, &card.id), yaml.as_bytes())
}

/// Deletes the persisted card with this `id`, if any — used when the
/// vocabulary/learning item it depends on is removed (FR-012c), to avoid
/// leaving an orphaned card pointing at a nonexistent learning item.
/// Returns `false` (not an error) when it didn't exist.
pub fn remove(session_root: &Path, id: &str) -> std::io::Result<bool> {
    let path = card_path(session_root, id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

pub fn load_all(session_root: &Path) -> std::io::Result<Vec<CardDefinition>> {
    let dir = cards_dir(session_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut cards = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let card: CardDefinition = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        cards.push(card);
    }
    Ok(cards)
}

fn side_has(side: &Side, want_image: bool) -> Option<&Option<String>> {
    side.blocks.iter().find_map(|b| match b {
        Block::Image { asset_id } if want_image => Some(asset_id),
        Block::Audio { asset_id } if !want_image => Some(asset_id),
        _ => None,
    })
}

/// Recomputes whether `card` satisfies its template's media policy — never
/// cached, always evaluated against the card's current blocks (Hard Guards,
/// FR-015).
pub fn completeness(card: &CardDefinition) -> Completeness {
    let Some(template) = find(&card.template) else {
        return Completeness::PendingImage {
            side: "front",
            reason: format!("unknown template '{}'", card.template),
        };
    };

    let checks: [(MediaPolicy, &Side, bool, &'static str); 4] = [
        (template.front_image, &card.front, true, "front"),
        (template.front_audio, &card.front, false, "front"),
        (template.back_image, &card.back, true, "back"),
        (template.back_audio, &card.back, false, "back"),
    ];

    for (policy, side, is_image, side_name) in checks {
        if policy != MediaPolicy::Required {
            continue;
        }
        let has_asset = side_has(side, is_image)
            .map(|a| a.is_some())
            .unwrap_or(false);
        if !has_asset {
            let reason = format!(
                "template '{}' requires {} on the {side_name} side",
                card.template,
                if is_image { "an image" } else { "audio" }
            );
            return if is_image {
                Completeness::PendingImage {
                    side: side_name,
                    reason,
                }
            } else {
                Completeness::PendingAudio {
                    side: side_name,
                    reason,
                }
            };
        }
    }

    Completeness::Complete
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card_with(template: &str, front: Side, back: Side) -> CardDefinition {
        CardDefinition {
            id: "card-001".to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            template: template.to_string(),
            front,
            back,
        }
    }

    #[test]
    fn complete_when_required_image_present() {
        // FR-013b: `image-to-production-v1` now requires audio on both
        // sides too (the back reuses the front's resolved asset).
        let card = card_with(
            "image-to-production-v1",
            Side {
                blocks: vec![
                    Block::Image {
                        asset_id: Some("asset-1".to_string()),
                    },
                    Block::Audio {
                        asset_id: Some("audio-1".to_string()),
                    },
                ],
            },
            Side {
                blocks: vec![
                    Block::Text {
                        value: "whiteboard".to_string(),
                    },
                    Block::Audio {
                        asset_id: Some("audio-1".to_string()),
                    },
                ],
            },
        );

        assert_eq!(completeness(&card), Completeness::Complete);
    }

    #[test]
    fn pending_image_when_required_image_missing() {
        let card = card_with(
            "image-to-production-v1",
            Side { blocks: vec![] },
            Side { blocks: vec![] },
        );

        assert!(matches!(
            completeness(&card),
            Completeness::PendingImage { .. }
        ));
    }

    #[test]
    fn front_and_back_audio_are_independent() {
        let card = card_with(
            "sentence-listening-v1",
            Side {
                blocks: vec![Block::Audio {
                    asset_id: Some("front-audio".to_string()),
                }],
            },
            Side {
                blocks: vec![Block::Audio {
                    asset_id: Some("back-audio".to_string()),
                }],
            },
        );

        assert_eq!(completeness(&card), Completeness::Complete);
    }

    #[test]
    fn saves_and_loads_cards_from_session_dir() {
        let dir = tempfile::tempdir().unwrap();
        let card = card_with("word-to-meaning-v1", Side::default(), Side::default());

        save(dir.path(), &card).unwrap();
        let loaded = load_all(dir.path()).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, card.id);
    }
}
