use crate::guid::{card_id_for, note_guid_for, note_id_for};
use crate::schema::CREATE_TABLES_SQL;
use learnkit_cards::card::{Block, CardDefinition};
use learnkit_media::asset::Asset;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ApkgBuildResult {
    pub note_count: usize,
    pub card_count: usize,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn field_html(
    blocks: &[Block],
    assets: &HashMap<String, Asset>,
    media_names: &mut HashMap<String, String>,
) -> String {
    blocks
        .iter()
        .map(|block| match block {
            Block::Text { value } => value.clone(),
            Block::Image { asset_id } => asset_id
                .as_ref()
                .and_then(|id| assets.get(id))
                .map(|asset| {
                    let name = media_filename(asset, media_names);
                    format!("<img src=\"{name}\">")
                })
                .unwrap_or_default(),
            Block::Audio { asset_id } => asset_id
                .as_ref()
                .and_then(|id| assets.get(id))
                .map(|asset| {
                    let name = media_filename(asset, media_names);
                    format!("[sound:{name}]")
                })
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Returns the media filename to reference in note HTML for `asset`,
/// registering it in `media_names` (asset id -> filename) the first time
/// it is seen so the same asset used on multiple cards is only bundled once.
fn media_filename(asset: &Asset, media_names: &mut HashMap<String, String>) -> String {
    media_names
        .entry(asset.id.clone())
        .or_insert_with(|| {
            let ext = asset
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            format!("{}.{ext}", asset.id)
        })
        .clone()
}

fn checksum(sfld: &str) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(sfld.as_bytes());
    let digest = hasher.finalize();
    i64::from(u32::from_be_bytes([
        digest[0], digest[1], digest[2], digest[3],
    ]))
}

/// Builds `collection.anki2` in-memory-ish (a temp SQLite file) from `cards`,
/// bundles referenced media, and packages everything into `out_path` as a
/// `.apkg` (ZIP). Fails atomically: nothing is written at `out_path` unless
/// every step succeeds (FR-019, via a temp file + rename at the CLI layer).
pub fn build_apkg(
    cards: &[CardDefinition],
    assets: &HashMap<String, Asset>,
    out_path: &Path,
) -> std::io::Result<ApkgBuildResult> {
    let tmp_dir = tempfile::tempdir()?;
    let db_path = tmp_dir.path().join("collection.anki2");

    let model_id: i64 = 1;
    let deck_id: i64 = 1;
    let now = now_ms();

    let conn = Connection::open(&db_path).map_err(|err| std::io::Error::other(err.to_string()))?;
    conn.execute_batch(CREATE_TABLES_SQL)
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    let models = serde_json::json!({
        model_id.to_string(): {
            "id": model_id,
            "name": "LearnKit Basic",
            "type": 0,
            "mod": now / 1000,
            "usn": -1,
            "sortf": 0,
            "did": deck_id,
            "tmpls": [{
                "name": "Card 1",
                "ord": 0,
                "qfmt": "{{Front}}",
                "afmt": "{{FrontSide}}<hr id=answer>{{Back}}",
                "did": null,
                "bqfmt": "",
                "bafmt": ""
            }],
            "flds": [
                {"name": "Front", "ord": 0, "sticky": false, "rtl": false, "font": "Arial", "size": 20},
                {"name": "Back", "ord": 1, "sticky": false, "rtl": false, "font": "Arial", "size": 20}
            ],
            "css": ".card { font-family: arial; font-size: 20px; text-align: center; }",
            "latexPre": "",
            "latexPost": "",
            "req": [[0, "any", [0]]]
        }
    });
    let decks = serde_json::json!({
        deck_id.to_string(): {
            "id": deck_id,
            "name": "LearnKit",
            "mod": now / 1000,
            "usn": -1,
            "lrnToday": [0, 0],
            "revToday": [0, 0],
            "newToday": [0, 0],
            "timeToday": [0, 0],
            "collapsed": false,
            "conf": 1,
            "desc": "",
            "dyn": 0,
            "extendNew": 0,
            "extendRev": 0
        }
    });

    conn.execute(
        "INSERT INTO col (id, crt, mod, scm, ver, dty, usn, ls, conf, models, decks, dconf, tags) VALUES (1, ?1, ?1, ?1, 11, 0, 0, 0, '{}', ?2, ?3, '{}', '{}')",
        rusqlite::params![now / 1000, models.to_string(), decks.to_string()],
    )
    .map_err(|err| std::io::Error::other(err.to_string()))?;

    let mut media_names: HashMap<String, String> = HashMap::new();

    for card in cards {
        let front_html = field_html(&card.front.blocks, assets, &mut media_names);
        let back_html = field_html(&card.back.blocks, assets, &mut media_names);
        let flds = format!("{front_html}\u{1f}{back_html}");
        let note_id = note_id_for(&card.id);
        let guid = note_guid_for(&card.id);
        let sfld_checksum = checksum(&front_html);

        conn.execute(
            "INSERT INTO notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) VALUES (?1, ?2, ?3, ?4, 0, '', ?5, ?6, ?7, 0, '')",
            rusqlite::params![note_id, guid, model_id, now / 1000, flds, front_html, sfld_checksum],
        )
        .map_err(|err| std::io::Error::other(err.to_string()))?;

        let card_row_id = card_id_for(&card.id);
        conn.execute(
            "INSERT INTO cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data) VALUES (?1, ?2, ?3, 0, ?4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '')",
            rusqlite::params![card_row_id, note_id, deck_id, now / 1000],
        )
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    }

    drop(conn);

    write_zip(&db_path, &media_names, assets, out_path)?;

    Ok(ApkgBuildResult {
        note_count: cards.len(),
        card_count: cards.len(),
    })
}

fn write_zip(
    db_path: &Path,
    media_names: &HashMap<String, String>,
    assets: &HashMap<String, Asset>,
    out_path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_out = out_path.with_extension("apkg.tmp");
    let file = std::fs::File::create(&tmp_out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("collection.anki2", options)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    zip.write_all(&std::fs::read(db_path)?)?;

    let mut media_manifest = serde_json::Map::new();
    for (index, (asset_id, filename)) in media_names.iter().enumerate() {
        if let Some(asset) = assets.get(asset_id) {
            let content = std::fs::read(&asset.path)?;
            zip.start_file(index.to_string(), options)
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            zip.write_all(&content)?;
            media_manifest.insert(
                index.to_string(),
                serde_json::Value::String(filename.clone()),
            );
        }
    }

    zip.start_file("media", options)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    zip.write_all(
        serde_json::Value::Object(media_manifest)
            .to_string()
            .as_bytes(),
    )?;

    zip.finish()
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    std::fs::rename(&tmp_out, out_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use learnkit_cards::card::Side;
    use learnkit_media::asset::{AssetOrigin, AssetType};

    fn make_asset(id: &str, path: &Path) -> Asset {
        std::fs::write(path, vec![7u8; 32]).unwrap();
        Asset {
            id: id.to_string(),
            asset_type: AssetType::Image,
            path: path.to_path_buf(),
            sha256: "deadbeef".to_string(),
            mime: "image/png".to_string(),
            origin: AssetOrigin::Supplied,
        }
    }

    #[test]
    fn builds_a_valid_zip_with_collection_and_media() {
        let dir = tempfile::tempdir().unwrap();
        let asset_path = dir.path().join("asset-1.png");
        let asset = make_asset("asset-1", &asset_path);

        let card = CardDefinition {
            id: "card-001".to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            template: "image-to-production-v1".to_string(),
            front: Side {
                blocks: vec![Block::Image {
                    asset_id: Some("asset-1".to_string()),
                }],
            },
            back: Side {
                blocks: vec![Block::Text {
                    value: "whiteboard".to_string(),
                }],
            },
        };

        let mut assets = HashMap::new();
        assets.insert("asset-1".to_string(), asset);

        let out_path = dir.path().join("dist").join("deck.apkg");
        let result = build_apkg(&[card], &assets, &out_path).unwrap();

        assert_eq!(result.note_count, 1);
        assert!(out_path.exists());

        let file = std::fs::File::open(&out_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        assert!(names.contains(&"collection.anki2".to_string()));
        assert!(names.contains(&"media".to_string()));
        assert!(names.contains(&"0".to_string()));

        let mut media_entry = archive.by_name("media").unwrap();
        let mut media_raw = String::new();
        std::io::Read::read_to_string(&mut media_entry, &mut media_raw).unwrap();
        let media_json: serde_json::Value = serde_json::from_str(&media_raw).unwrap();
        assert_eq!(media_json["0"], "asset-1.png");
    }

    #[test]
    fn collection_anki2_has_one_note_and_one_card_per_card_definition() {
        let dir = tempfile::tempdir().unwrap();
        let card = CardDefinition {
            id: "card-002".to_string(),
            learning_item_ids: vec!["li-2".to_string()],
            template: "word-to-meaning-v1".to_string(),
            front: Side {
                blocks: vec![Block::Text {
                    value: "get away with".to_string(),
                }],
            },
            back: Side {
                blocks: vec![Block::Text {
                    value: "hacer algo malo sin castigo".to_string(),
                }],
            },
        };

        let out_path = dir.path().join("deck.apkg");
        build_apkg(&[card], &HashMap::new(), &out_path).unwrap();

        // Extract collection.anki2 and query it directly to verify the
        // schema is actually usable, not just present.
        let file = std::fs::File::open(&out_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut entry = archive.by_name("collection.anki2").unwrap();
        let extracted_path = dir.path().join("extracted.anki2");
        let mut out_file = std::fs::File::create(&extracted_path).unwrap();
        std::io::copy(&mut entry, &mut out_file).unwrap();
        drop(out_file);

        let conn = Connection::open(&extracted_path).unwrap();
        let note_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();
        let card_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
            .unwrap();
        assert_eq!(note_count, 1);
        assert_eq!(card_count, 1);

        let flds: String = conn
            .query_row("SELECT flds FROM notes", [], |row| row.get(0))
            .unwrap();
        assert!(flds.contains("get away with"));
        assert!(flds.contains("hacer algo malo sin castigo"));
    }

    #[test]
    fn reexporting_the_same_cards_produces_the_same_note_ids() {
        let dir = tempfile::tempdir().unwrap();
        let card = CardDefinition {
            id: "card-003".to_string(),
            learning_item_ids: vec!["li-3".to_string()],
            template: "word-to-meaning-v1".to_string(),
            front: Side {
                blocks: vec![Block::Text {
                    value: "put up with".to_string(),
                }],
            },
            back: Side { blocks: vec![] },
        };

        let out1 = dir.path().join("deck1.apkg");
        let out2 = dir.path().join("deck2.apkg");
        build_apkg(std::slice::from_ref(&card), &HashMap::new(), &out1).unwrap();
        build_apkg(std::slice::from_ref(&card), &HashMap::new(), &out2).unwrap();

        let extract_note_id = |path: &Path| -> i64 {
            let file = std::fs::File::open(path).unwrap();
            let mut archive = zip::ZipArchive::new(file).unwrap();
            let mut entry = archive.by_name("collection.anki2").unwrap();
            let extracted = path.with_extension("db");
            let mut out_file = std::fs::File::create(&extracted).unwrap();
            std::io::copy(&mut entry, &mut out_file).unwrap();
            drop(out_file);
            let conn = Connection::open(&extracted).unwrap();
            conn.query_row("SELECT id FROM notes", [], |row| row.get(0))
                .unwrap()
        };

        assert_eq!(extract_note_id(&out1), extract_note_id(&out2));
    }
}
