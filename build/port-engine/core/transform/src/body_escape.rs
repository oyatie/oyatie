//! The source's escape vocabulary, spelled for the target.
//!
//! The two languages do not define the same escapes. The source has `\a`, `\b`, `\f`, `\v` and
//! three-digit OCTAL; the target has none of them. An escape cannot therefore be carried across as
//! text -- but it does not have to be guessed at either, because an escape denotes a NUMBER and the
//! number is the same in both. chi spells the terminal escape `'\033'`, which reached the renderer
//! as `b'\033'` and refused; 17 of its declarations were blocked on a literal whose value was never
//! in doubt.

/// Whether the target spells this rune literal exactly as the source does.
///
/// True for an unescaped character, and for the escapes both languages define the same way. A
/// `\xHH` is shared only up to `0x7f`: above it the target admits the byte form but not the
/// character one, so the two disagree and the caller respells by code point.
pub(crate) fn shared_escape(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    else {
        return false;
    };
    if !inner.starts_with('\\') {
        return true;
    }
    if matches!(inner, "\\n" | "\\r" | "\\t" | "\\\\" | "\\'") {
        return true;
    }
    inner
        .strip_prefix("\\x")
        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
        .is_some_and(|code| code <= 0x7f)
}

/// The code point a source rune literal denotes.
///
/// The source's escape vocabulary is WIDER than the target's — it has `\a`, `\b`, `\f`, `\v` and
/// three-digit OCTAL, none of which the target defines — so an escape cannot be copied across as
/// text. It can be computed: an escape denotes a number, and the number is the same in both. chi
/// spells the terminal escape `'\033'`, which reached the renderer as `b'\033'` and refused; 17 of
/// its declarations were blocked on a literal whose value was never in doubt.
///
/// `None` when the spelling is not one this decoder knows, which the caller refuses rather than
/// guesses.
pub(crate) fn rune_code_point(value: &str) -> Option<u32> {
    let inner = value.strip_prefix('\'')?.strip_suffix('\'')?;
    let Some(escape) = inner.strip_prefix('\\') else {
        let mut characters = inner.chars();
        let only = characters.next()?;
        return characters.next().is_none().then_some(only as u32);
    };
    match escape {
        "a" => Some(0x07),
        "b" => Some(0x08),
        "f" => Some(0x0c),
        "n" => Some(0x0a),
        "r" => Some(0x0d),
        "t" => Some(0x09),
        "v" => Some(0x0b),
        "\\" => Some(0x5c),
        "'" => Some(0x27),
        "\"" => Some(0x22),
        _ => {
            for prefix in ["x", "u", "U"] {
                if let Some(hex) = escape.strip_prefix(prefix) {
                    return u32::from_str_radix(hex, 16).ok();
                }
            }
            // OCTAL, which the source writes as exactly three digits and the target does not write
            // at all.
            let octal =
                escape.len() == 3 && escape.bytes().all(|digit| (b'0'..=b'7').contains(&digit));
            octal.then(|| u32::from_str_radix(escape, 8).ok())?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{rune_code_point, shared_escape};

    #[test]
    fn an_escape_the_target_also_defines_keeps_its_source_spelling() {
        for shared in [
            "'a'", "'\\n'", "'\\r'", "'\\t'", "'\\\\'", "'\\''", "'\\x7f'",
        ] {
            assert!(
                shared_escape(shared),
                "{shared} is spelled the same by both"
            );
        }
    }

    #[test]
    fn an_escape_only_the_source_defines_is_not_shared() {
        for source_only in ["'\\033'", "'\\a'", "'\\b'", "'\\f'", "'\\v'", "'\\x80'"] {
            assert!(
                !shared_escape(source_only),
                "{source_only} has no target spelling"
            );
        }
    }

    #[test]
    fn an_octal_escape_denotes_the_byte_the_target_spells_in_hex() {
        // The literal that blocked 17 chi declarations. `\033` is ESC.
        assert_eq!(rune_code_point("'\\033'"), Some(0x1b));
        assert_eq!(rune_code_point("'\\000'"), Some(0));
        assert_eq!(rune_code_point("'\\177'"), Some(0x7f));
    }

    #[test]
    fn the_escapes_the_target_lacks_decode_to_their_control_bytes() {
        assert_eq!(rune_code_point("'\\a'"), Some(0x07));
        assert_eq!(rune_code_point("'\\b'"), Some(0x08));
        assert_eq!(rune_code_point("'\\f'"), Some(0x0c));
        assert_eq!(rune_code_point("'\\v'"), Some(0x0b));
    }

    #[test]
    fn a_plain_character_denotes_itself() {
        assert_eq!(rune_code_point("'a'"), Some(u32::from(b'a')));
        assert_eq!(rune_code_point("'\u{e9}'"), Some(0xe9));
    }

    #[test]
    fn a_hex_escape_denotes_the_same_byte_in_both_languages() {
        assert_eq!(rune_code_point("'\\x1b'"), Some(0x1b));
        assert_eq!(rune_code_point("'\\x1b'"), rune_code_point("'\\033'"));
    }

    #[test]
    fn a_spelling_this_decoder_does_not_know_is_not_guessed_at() {
        // Two characters is not one rune, and an eight-digit octal is not the source's form.
        assert_eq!(rune_code_point("'ab'"), None);
        assert_eq!(rune_code_point("'\\9'"), None);
        assert_eq!(rune_code_point("not a literal"), None);
    }
}
