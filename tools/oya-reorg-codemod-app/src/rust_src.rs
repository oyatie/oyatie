//! Rust source transforms: rewrite references to a moved crate's SNAKE crate identifier in
//! `use`, `extern crate`, and `::`-path positions. A crate rename from kebab `oya-iam` to
//! `iam-core` changes the in-source identifier from `oya_iam` to `iam_core`; every
//! `use oya_iam::...`, `extern crate oya_iam;`, and `oya_iam::Foo` path must follow.
//!
//! The rewrite is identifier-boundary-anchored: `oya_iam` is replaced only when it is a whole
//! identifier (not a substring of `oya_iam_app`). This is a token-level substitution, not a
//! full parse — sufficient and deterministic for the crate-ident rename class, which is the
//! only Rust-source change a path move induces (the move never edits item names).

use std::collections::BTreeMap;

/// Rewrite every whole-identifier occurrence of an old crate ident to its new ident across a
/// Rust source file. `ident_renames` maps OLD snake crate ident -> NEW snake crate ident.
/// Returns `(new_text, changed)`. A file with no matching idents is returned byte-identical.
pub fn rewrite_rust_source(
    source: &str,
    ident_renames: &BTreeMap<String, String>,
) -> (String, bool) {
    if ident_renames.is_empty() {
        return (source.to_string(), false);
    }
    let mut changed = false;
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if is_ident_start(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let ident = &source[start..i];
            if let Some(new_ident) = ident_renames.get(ident) {
                out.push_str(new_ident);
                changed = true;
            } else {
                out.push_str(ident);
            }
        } else {
            let ch_len = utf8_len(bytes[i]);
            out.push_str(&source[i..i + ch_len]);
            i += ch_len;
        }
    }
    if changed {
        (out, true)
    } else {
        (source.to_string(), false)
    }
}

fn is_ident_start(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphabetic()
}

fn is_ident_continue(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

fn utf8_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first >> 5 == 0b110 {
        2
    } else if first >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renames(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn rewrites_use_extern_and_path_positions() {
        let src = r#"extern crate oya_iam;
use oya_iam::domain::Principal;
use oya_iam::{Foo, Bar};

fn f() -> oya_iam::Result {
    oya_iam::init()
}
"#;
        let (out, changed) = rewrite_rust_source(src, &renames(&[("oya_iam", "iam_core")]));
        assert!(changed);
        assert!(out.contains("extern crate iam_core;"));
        assert!(out.contains("use iam_core::domain::Principal;"));
        assert!(out.contains("use iam_core::{Foo, Bar};"));
        assert!(out.contains("iam_core::init()"));
        assert!(!out.contains("oya_iam"));
    }

    #[test]
    fn does_not_match_substring_identifier() {
        // oya_iam must not match inside oya_iam_app.
        let src = "use oya_iam_app::X;\nuse oya_iam::Y;\n";
        let (out, _changed) = rewrite_rust_source(src, &renames(&[("oya_iam", "iam")]));
        assert!(out.contains("oya_iam_app::X"), "substring preserved: {out}");
        assert!(out.contains("iam::Y"));
    }

    #[test]
    fn no_rename_is_byte_identical() {
        let src = "use serde::Serialize;\nfn main() {}\n";
        let (out, changed) = rewrite_rust_source(src, &renames(&[("oya_iam", "iam")]));
        assert!(!changed);
        assert_eq!(out, src);
    }

    #[test]
    fn empty_renames_is_no_op() {
        let src = "use oya_iam::X;";
        let (out, changed) = rewrite_rust_source(src, &BTreeMap::new());
        assert!(!changed);
        assert_eq!(out, src);
    }

    #[test]
    fn multiple_renames_applied() {
        let src = "use oya_iam::X; use oya_kms::Y;";
        let (out, _) = rewrite_rust_source(
            src,
            &renames(&[("oya_iam", "iam"), ("oya_kms", "secrets_kms")]),
        );
        assert!(out.contains("iam::X"));
        assert!(out.contains("secrets_kms::Y"));
    }
}
