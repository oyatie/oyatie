//! Rust source transforms: rewrite references to a moved crate's SNAKE crate identifier in
//! `use`, `extern crate`, and `::`-path positions. A crate rename from kebab `iam` to
//! `iam-core` changes the in-source identifier from `iam` to `iam_core`; every
//! `use iam::...`, `extern crate iam;`, and `iam::Foo` path must follow.
//!
//! The rewrite is identifier-boundary-anchored: `iam` is replaced only when it is a whole
//! identifier (not a substring of `iam_app`). This is a token-level substitution, not a
//! full parse — sufficient and deterministic for the crate-ident rename class, which is the
//! only Rust-source change a path move induces (the move never edits item names).

//! # Path literals: detect-and-REFUSE (not rewrite)
//!
//! `include!` / `include_bytes!` / `include_str!` / `#[path]` carry FILE-RELATIVE path
//! literals. A flat `crates/<crate>` -> two-level `<face>/<crate>` move changes both the
//! crate's name and its HOP COUNT to anything outside itself, so `../../../x` silently stops
//! meaning what it meant. This module DETECTS those literals and the engine refuses the move;
//! it deliberately does NOT rewrite them. Three reasons:
//!
//! 1. **No oracle can check a rewrite.** Both of this tool's oracles — `cargo metadata` and
//!    `buck2 targets //...` — resolve the graph WITHOUT compiling, so neither can observe a
//!    dangling `include!`. Shipping a rewrite whose only validation is "the arithmetic looked
//!    right" would recreate the exact defect this work exists to remove: an unverifiable
//!    transform reported as green.
//! 2. **The targets are not all in the plan.** The measured cloud-kernel literals point at
//!    `../../../out/*.elf` — a BUILD-OUTPUT directory that is not a crate, is not in any move
//!    plan, and may not exist when the codemod runs. Whether it should follow the crate, stay
//!    put, or be regenerated is a judgment the plan does not encode, so a rewrite would guess.
//! 3. **Refusal is cheap and precise.** The detector reports file:line and the resolved
//!    target, and only fires for literals that ESCAPE the moving crate — a literal pointing
//!    inside the crate's own directory keeps its meaning (the whole subtree moves together)
//!    and is silently allowed, which is what keeps the refusal from being noise.
//!
//! Refusing loudly cannot corrupt a tree; a clever rewrite can.

use std::collections::BTreeMap;

use crate::model::{EscapingPathLiteral, join_rel};

/// The path-literal carriers a crate move can invalidate. Each is matched as a literal prefix
/// followed (after optional whitespace) by a plain double-quoted string.
const PATH_LITERAL_CARRIERS: [(&str, &str); 4] = [
    ("include_bytes!", "("),
    ("include_str!", "("),
    ("include!", "("),
    ("#[path", "="),
];

/// Scan one Rust source file for path literals that resolve OUTSIDE `crate_dir`.
///
/// `file_rel` and `crate_dir` are repo-relative (the crate's PRE-move dir). A literal that
/// resolves inside `crate_dir` is omitted: the whole crate subtree moves together, so its
/// relative path is preserved by construction. A literal that escapes the repo root entirely
/// is reported with `resolves_to: None`.
///
/// ponytail: only plain string literals are matched. `include!(concat!(env!("OUT_DIR"), ...))`
/// is build-script driven and move-invariant, so skipping it is correct rather than lazy; a
/// hand-built `concat!("../", ...)` would be missed, which is why this is a REFUSAL gate on
/// what it does see rather than a completeness claim.
pub fn scan_escaping_path_literals(
    source: &str,
    file_rel: &str,
    crate_dir: &str,
) -> Vec<EscapingPathLiteral> {
    let file_dir = parent_dir(file_rel);
    let mut out = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        for (carrier, opener) in PATH_LITERAL_CARRIERS {
            let mut cursor = 0usize;
            while let Some(hit) = line[cursor..].find(carrier) {
                let after = cursor + hit + carrier.len();
                cursor = after;
                let Some(literal) = literal_after(line, after, opener) else {
                    continue;
                };
                // An absolute path is not a move-relative reference; leave it be.
                if literal.starts_with('/') {
                    continue;
                }
                let resolves_to = join_rel(&file_dir, &literal);
                let escapes = match resolves_to.as_deref() {
                    // Escaped the repo root -> unresolvable, always a refusal.
                    None => true,
                    Some(target) => !is_inside(target, crate_dir),
                };
                if escapes {
                    out.push(EscapingPathLiteral {
                        file: file_rel.to_string(),
                        line: line_idx + 1,
                        kind: carrier.trim_start_matches("#[").to_string(),
                        literal,
                        resolves_to,
                    });
                }
            }
        }
    }
    out
}

/// Extract the double-quoted string that follows `opener` at `from`, skipping whitespace.
/// Returns `None` when the next non-space token is not a plain string literal (so
/// `concat!(...)`, `env!(...)` and raw/byte strings are simply not matched).
fn literal_after(line: &str, from: usize, opener: &str) -> Option<String> {
    let rest = line.get(from..)?.trim_start();
    let rest = rest.strip_prefix(opener)?.trim_start();
    let body = rest.strip_prefix('"')?;
    let end = body.find('"')?;
    Some(body[..end].to_string())
}

/// True if repo-relative `target` is `dir` itself or lives beneath it.
fn is_inside(target: &str, dir: &str) -> bool {
    if dir.is_empty() {
        return true;
    }
    target == dir || target.starts_with(&format!("{dir}/"))
}

/// The repo-relative directory holding `rel` (empty at the repo root).
fn parent_dir(rel: &str) -> String {
    match rel.rfind('/') {
        Some(idx) => rel[..idx].to_string(),
        None => String::new(),
    }
}

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
        let src = r#"extern crate iam;
use iam::domain::Principal;
use iam::{Foo, Bar};

fn f() -> iam::Result {
    iam::init()
}
"#;
        let (out, changed) = rewrite_rust_source(src, &renames(&[("iam", "iam_core")]));
        assert!(changed);
        assert!(out.contains("extern crate iam_core;"));
        assert!(out.contains("use iam_core::domain::Principal;"));
        assert!(out.contains("use iam_core::{Foo, Bar};"));
        assert!(out.contains("iam_core::init()"));
        assert!(!out.contains("iam"));
    }

    #[test]
    fn does_not_match_substring_identifier() {
        // iam must not match inside iam_app.
        let src = "use iam_app::X;\nuse iam::Y;\n";
        let (out, _changed) = rewrite_rust_source(src, &renames(&[("iam", "iam")]));
        assert!(out.contains("iam_app::X"), "substring preserved: {out}");
        assert!(out.contains("iam::Y"));
    }

    #[test]
    fn no_rename_is_byte_identical() {
        let src = "use serde::Serialize;\nfn main() {}\n";
        let (out, changed) = rewrite_rust_source(src, &renames(&[("iam", "iam")]));
        assert!(!changed);
        assert_eq!(out, src);
    }

    #[test]
    fn empty_renames_is_no_op() {
        let src = "use iam::X;";
        let (out, changed) = rewrite_rust_source(src, &BTreeMap::new());
        assert!(!changed);
        assert_eq!(out, src);
    }

    /// D3 class 1, verbatim from `cloud/cloud-kernel/crates/cloud-kernel-arch-x86-64-adapter/
    /// src/user.rs`: a BUILD-OUTPUT target (`cloud/cloud-kernel/out/`) that is not a crate, is in
    /// no move plan, and may not exist when the codemod runs — the case a rewrite would have to
    /// guess at.
    #[test]
    fn escaping_build_output_include_literal_is_detected() {
        let src = "static SPAWN: &[u8] = include_bytes!(\"../../../out/user-spawn-x86_64.elf\");\n";
        let found = scan_escaping_path_literals(
            src,
            "cloud/cloud-kernel/crates/cloud-kernel-arch-x86-64-adapter/src/user.rs",
            "cloud/cloud-kernel/crates/cloud-kernel-arch-x86-64-adapter",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            found[0].resolves_to.as_deref(),
            Some("cloud/cloud-kernel/out/user-spawn-x86_64.elf")
        );
        assert_eq!(found[0].kind, "include_bytes!");
        assert_eq!(found[0].line, 1, "1-indexed line reported");
    }

    /// D3 class 2, verbatim from `cloud/cloud-kernel/crates/cloud-kernel-arch-aarch64-adapter/
    /// tests-host/src/lib.rs:46`: a SIBLING CRATE named by its old directory outright. Note the
    /// hop count is only correct at THIS depth — the same literal one directory shallower
    /// resolves somewhere else entirely, which is exactly why the engine refuses instead of
    /// recomputing.
    #[test]
    fn escaping_sibling_crate_include_literal_is_detected() {
        let src = "    include!(\"../../../cloud-kernel-user-layout-kernel/src/vfs.rs\");\n";
        let found = scan_escaping_path_literals(
            src,
            "cloud/cloud-kernel/crates/cloud-kernel-arch-aarch64-adapter/tests-host/src/lib.rs",
            "cloud/cloud-kernel/crates/cloud-kernel-arch-aarch64-adapter/tests-host",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            found[0].resolves_to.as_deref(),
            Some("cloud/cloud-kernel/crates/cloud-kernel-user-layout-kernel/src/vfs.rs"),
            "sibling-crate target resolved repo-relative"
        );
        assert_eq!(found[0].kind, "include!");
    }

    /// The refusal must NOT be noise: a literal pointing INSIDE the moving crate keeps its
    /// meaning, because the whole crate subtree relocates together. This is the common
    /// `tests/common/mod.rs` / `../src/x.rs` shape and must stay silent.
    #[test]
    fn literals_inside_the_moving_crate_are_not_reported() {
        let src = r#"
include!("../src/layout.rs");
include!("helper.rs");
"#;
        let found = scan_escaping_path_literals(
            src,
            "cloud/cloud-kernel/crates/cloud-kernel-user-layout-kernel/tests-buck/lib.rs",
            "cloud/cloud-kernel/crates/cloud-kernel-user-layout-kernel",
        );
        assert!(
            found.is_empty(),
            "self-contained literals must not fire: {found:?}"
        );
    }

    #[test]
    fn path_attribute_and_repo_escaping_literal_are_reported() {
        let src = "#[path = \"../../shared/mod.rs\"]\nmod shared;\ninclude!(\"../../../../../etc/x.rs\");\n";
        let found = scan_escaping_path_literals(src, "a/b/src/lib.rs", "a/b");
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].kind, "path", "#[path] carrier labelled");
        assert_eq!(found[0].resolves_to.as_deref(), Some("a/shared/mod.rs"));
        assert_eq!(
            found[1].resolves_to, None,
            "a literal escaping the repo root is unresolvable and still refused"
        );
    }

    /// Non-literal include forms are build-script driven and move-invariant; matching them
    /// would make the gate fire on paths a move cannot break.
    #[test]
    fn out_dir_concat_include_is_not_reported() {
        let src = "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));\n";
        let found = scan_escaping_path_literals(src, "a/b/src/lib.rs", "a/b");
        assert!(
            found.is_empty(),
            "OUT_DIR includes are move-invariant: {found:?}"
        );
    }

    #[test]
    fn multiple_renames_applied() {
        let src = "use iam::X; use kms::Y;";
        let (out, _) = rewrite_rust_source(
            src,
            &renames(&[("iam", "iam"), ("kms", "secrets_kms")]),
        );
        assert!(out.contains("iam::X"));
        assert!(out.contains("secrets_kms::Y"));
    }
}
