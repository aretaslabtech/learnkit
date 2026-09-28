use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Image,
    Audio,
}

/// Where an `Asset`'s bytes came from — see `data-model.md` → Asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetOrigin {
    Supplied,
    Generated {
        provider: String,
        voice_policy: String,
    },
    Fetched {
        provider: String,
        license_name: String,
        license_url: String,
        author: String,
        source_url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub sha256: String,
    pub mime: String,
    pub origin: AssetOrigin,
}

/// Minimum file size (bytes) for a deterministic "not obviously broken"
/// check, per `docs/07-anki-media.md §7/§8`. Real decode validation is a
/// per-format concern outside this generic registry.
const MIN_ASSET_BYTES: usize = 16;

/// Index mapping a caller-computed logical fingerprint (per
/// `docs/07-anki-media.md §6`: `hash(normalized_text + locale + voice_policy)`
/// for generated audio, an equivalent key for fetched images, or the content
/// `sha256` for supplied files) to an already-registered `Asset`, so a new
/// request for an equivalent resource reuses it instead of duplicating it
/// (FR-016).
fn index_path(assets_dir: &Path) -> PathBuf {
    assets_dir.join("index.json")
}

fn load_index(assets_dir: &Path) -> std::io::Result<HashMap<String, String>> {
    let path = index_path(assets_dir);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))
}

fn save_index(assets_dir: &Path, index: &HashMap<String, String>) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(index)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(&index_path(assets_dir), json.as_bytes())
}

fn asset_metadata_path(assets_dir: &Path, id: &str) -> PathBuf {
    assets_dir.join(format!("{id}.json"))
}

/// Loads a previously registered asset by its id, if any.
pub fn load(assets_dir: &Path, id: &str) -> std::io::Result<Option<Asset>> {
    let path = asset_metadata_path(assets_dir, id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let asset: Asset = serde_json::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(asset))
}

pub fn content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Registers a new asset under `assets_dir`, or returns the existing one if
/// `logical_fingerprint` was already registered (FR-016 dedup).
pub fn register_or_reuse(
    assets_dir: &Path,
    logical_fingerprint: &str,
    asset_type: AssetType,
    content: &[u8],
    extension: &str,
    mime: &str,
    origin: AssetOrigin,
) -> std::io::Result<Asset> {
    let mut index = load_index(assets_dir)?;

    if let Some(existing_id) = index.get(logical_fingerprint) {
        let raw = fs::read_to_string(asset_metadata_path(assets_dir, existing_id))?;
        let asset: Asset = serde_json::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        return Ok(asset);
    }

    let sha256 = content_hash(content);
    let id = format!("asset-{}", &sha256[..12]);
    let file_path = assets_dir.join(format!("{id}.{extension}"));

    learnkit_core::atomic::write_atomic(&file_path, content)?;

    let asset = Asset {
        id: id.clone(),
        asset_type,
        path: file_path,
        sha256,
        mime: mime.to_string(),
        origin,
    };

    let asset_json = serde_json::to_string_pretty(&asset)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(
        &asset_metadata_path(assets_dir, &id),
        asset_json.as_bytes(),
    )?;

    index.insert(logical_fingerprint.to_string(), id);
    save_index(assets_dir, &index)?;

    Ok(asset)
}

/// Deterministic validity check per `docs/07-anki-media.md §7/§8`: file
/// exists, is non-trivially small, and its registered hash still matches
/// its on-disk content (never trusted as a cached fact — recomputed here).
pub fn is_valid(asset: &Asset) -> bool {
    let Ok(content) = fs::read(&asset.path) else {
        return false;
    };
    content.len() >= MIN_ASSET_BYTES && content_hash(&content) == asset.sha256
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supplied() -> AssetOrigin {
        AssetOrigin::Supplied
    }

    #[test]
    fn registers_a_new_asset() {
        let dir = tempfile::tempdir().unwrap();
        let content = vec![1u8; 32];

        let asset = register_or_reuse(
            dir.path(),
            "fp-1",
            AssetType::Image,
            &content,
            "png",
            "image/png",
            supplied(),
        )
        .unwrap();

        assert!(asset.path.exists());
        assert!(is_valid(&asset));
    }

    #[test]
    fn reuses_asset_with_same_logical_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        let content = vec![2u8; 32];

        let first = register_or_reuse(
            dir.path(),
            "fp-shared",
            AssetType::Audio,
            &content,
            "wav",
            "audio/wav",
            supplied(),
        )
        .unwrap();
        let second = register_or_reuse(
            dir.path(),
            "fp-shared",
            AssetType::Audio,
            &content,
            "wav",
            "audio/wav",
            supplied(),
        )
        .unwrap();

        assert_eq!(first.id, second.id);
    }

    #[test]
    fn different_fingerprint_creates_a_different_asset() {
        let dir = tempfile::tempdir().unwrap();

        let a = register_or_reuse(
            dir.path(),
            "fp-a",
            AssetType::Image,
            &[3u8; 32],
            "png",
            "image/png",
            supplied(),
        )
        .unwrap();
        let b = register_or_reuse(
            dir.path(),
            "fp-b",
            AssetType::Image,
            &[4u8; 32],
            "png",
            "image/png",
            supplied(),
        )
        .unwrap();

        assert_ne!(a.id, b.id);
    }

    #[test]
    fn too_small_content_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let asset = register_or_reuse(
            dir.path(),
            "fp-tiny",
            AssetType::Image,
            &[1, 2, 3],
            "png",
            "image/png",
            supplied(),
        )
        .unwrap();

        assert!(!is_valid(&asset));
    }

    #[test]
    fn loads_a_registered_asset_by_id() {
        let dir = tempfile::tempdir().unwrap();
        let registered = register_or_reuse(
            dir.path(),
            "fp-load",
            AssetType::Image,
            &[5u8; 32],
            "png",
            "image/png",
            supplied(),
        )
        .unwrap();

        let loaded = load(dir.path(), &registered.id).unwrap().unwrap();
        assert_eq!(loaded.id, registered.id);
    }

    #[test]
    fn loading_an_unknown_id_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path(), "does-not-exist").unwrap().is_none());
    }
}
