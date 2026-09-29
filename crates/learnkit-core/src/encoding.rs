/// Detects the classic "UTF-8 bytes misread as a legacy single-byte codepage"
/// mojibake pattern (e.g. Windows PowerShell 5.1 reading a BOM-less .ps1
/// script as its system codepage instead of UTF-8) and returns the original,
/// correctly-decoded text when found — the fix is exact, not a guess: the
/// corrupted string's codepoints ARE the original UTF-8 bytes, so re-encoding
/// them as raw bytes and decoding *that* as UTF-8 recovers the source text
/// exactly.
pub fn detect_mojibake(input: &str) -> Option<String> {
    // Fast bail-out: this pattern always produces 'Ã' or 'Â' (U+00C3/U+00C2,
    // the UTF-8 lead bytes for the Latin-1 Supplement block, misread as
    // their own codepoints) whenever the source had any accented Latin
    // character. No occurrence of either means this check does not apply.
    if !input.contains('Ã') && !input.contains('Â') {
        return None;
    }
    let mut bytes = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let cp = ch as u32;
        if cp > 0xFF {
            // Not consistent with a single-byte-codepage misread — bail
            // rather than guess.
            return None;
        }
        bytes.push(cp as u8);
    }
    match String::from_utf8(bytes) {
        Ok(fixed) if fixed != input => Some(fixed),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_and_fixes_real_mojibake() {
        assert_eq!(
            detect_mojibake("clasificaciÃ³n"),
            Some("clasificación".to_string())
        );
    }

    #[test]
    fn correct_text_with_accents_is_not_flagged() {
        assert_eq!(detect_mojibake("plátano"), None);
    }

    #[test]
    fn empty_string_is_not_flagged() {
        assert_eq!(detect_mojibake(""), None);
    }

    #[test]
    fn plain_ascii_without_accents_is_not_flagged() {
        assert_eq!(detect_mojibake("whiteboard"), None);
    }

    #[test]
    fn text_with_codepoint_above_0xff_never_false_positives() {
        // Contains 'Ã' but also an emoji (codepoint > 0xFF) — must bail
        // out rather than guess, since the single-byte-codepage theory
        // cannot account for the emoji.
        assert_eq!(detect_mojibake("Ã 😀"), None);
        // Same idea with a CJK character.
        assert_eq!(detect_mojibake("Ã 漢字"), None);
    }
}
