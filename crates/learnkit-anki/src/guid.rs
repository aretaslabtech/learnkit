use sha2::{Digest, Sha256};

/// Deterministic Anki note GUID/id derived from `card_id`, so reimporting
/// the same cards never creates duplicate notes (FR-020). Anki's own `id`
/// columns are signed 64-bit millisecond timestamps by convention, but any
/// stable positive i64 works as a primary key; we derive one from the hash
/// instead of `now()` so it never changes across exports of the same card.
pub fn note_id_for(card_id: &str) -> i64 {
    stable_i64(card_id, "note")
}

pub fn card_id_for(card_id: &str) -> i64 {
    stable_i64(card_id, "card")
}

/// Anki's own `guid` field (used for note deduplication on import) — a
/// short opaque string, also stable per `card_id`.
pub fn note_guid_for(card_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"guid");
    hasher.update(card_id.as_bytes());
    let digest = hasher.finalize();
    digest.iter().take(10).map(|b| format!("{b:02x}")).collect()
}

fn stable_i64(card_id: &str, namespace: &str) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(card_id.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    // Mask the sign bit so we always get a positive i64 — SQLite integer
    // primary keys accept any i64, but a stable positive value is easier to
    // eyeball while debugging an exported collection.
    (i64::from_be_bytes(bytes) & i64::MAX) | 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_card_id_always_produces_the_same_ids() {
        assert_eq!(note_id_for("card-001"), note_id_for("card-001"));
        assert_eq!(card_id_for("card-001"), card_id_for("card-001"));
        assert_eq!(note_guid_for("card-001"), note_guid_for("card-001"));
    }

    #[test]
    fn different_card_ids_produce_different_ids() {
        assert_ne!(note_id_for("card-001"), note_id_for("card-002"));
    }

    #[test]
    fn note_and_card_ids_are_distinct_namespaces() {
        assert_ne!(note_id_for("card-001"), card_id_for("card-001"));
    }
}
