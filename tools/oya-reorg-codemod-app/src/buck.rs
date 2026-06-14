//! BUCK transforms: rewrite absolute `//old/path:target` labels (in `deps`, `visibility`,
//! `env`, etc.) to `//new/path:new-target`, and rewrite the moved crate's own `name`,
//! `crate`, and `crate_root` fields. BUCK files are Starlark, not TOML, so the rewrites are
//! token-precise string substitutions over the label/name vocabulary the codemod owns.
//!
//! Two label flavors exist in the tree:
//! * **absolute** `//cloud/cloud-iam/crates/oya-x:oya-x` — root-anchored; rewritten to the
//!   new path + new target name wherever it appears in ANY BUCK file;
//! * **same-package** `:oya-x` — relative to the package; only the target NAME changes
//!   (handled by the in-file name rewrite of the moved crate's own BUCK).
//!
//! All substitutions are anchored on a non-identifier boundary so a label is never matched
//! as a substring of a longer label (e.g. `:oya-x` must not match inside `:oya-x-app`).

use std::collections::BTreeMap;

use crate::model::{snake, CrateMove};

/// Rewrite a MOVED crate's own `BUCK`: its `name = "..."`, `crate = "..."`,
/// `crate_root = "..."` (path stays relative so it is move-invariant), and any
/// same-package `:old-name` self-references in `deps`. Returns `(new_text, changed)`.
///
/// `name`/`crate` come in two flavors that need different match rules:
/// * EXACT-valued targets (`rust_library`/`rust_binary`) whose `name`/`crate` IS the crate
///   name — matched as the whole quoted token `"old"`. A `-bin` sibling like `"oya-x-bin"` is
///   a SEPARATE crate with its own move, so it must NOT be prefix-matched here.
/// * SUFFIXED `rust_test` targets whose `name`/`crate` is `<crate-name><sep><suffix>` (e.g.
///   `oya-x-domain-unittest`, crate `oya_x_domain_file_outbox`). The codemod owns the
///   crate-name PREFIX; the suffix (`-unittest`, `_file_outbox`) is test-author-chosen and is
///   preserved. To stay safe against the `-bin`-sibling ambiguity, the prefix rewrite is scoped
///   to the interior of `rust_test( ... )` stanzas ONLY.
pub fn rewrite_moved_buck(buck_text: &str, cm: &CrateMove) -> (String, bool) {
    let old_name = &cm.old_cargo_name;
    let new_name = &cm.new_cargo_name;
    let old_ident = snake(old_name);
    let new_ident = snake(new_name);

    // 1. Exact whole-token rewrite of `name`/`crate` everywhere (lib/binary/test exact values,
    //    same-package self-dep). This preserves the invariant that a `-bin` sibling target is
    //    untouched (its quoted value never equals the bare crate name).
    let mut out = buck_text.to_string();
    let mut changed = false;
    changed |= replace_quoted_exact(&mut out, old_name, new_name);
    if old_ident != *old_name {
        changed |= replace_quoted_exact(&mut out, &old_ident, &new_ident);
    }

    // 2. Suffixed prefix rewrite, scoped to `rust_test(...)` stanzas: rewrite a `name`/`crate`
    //    whose value is `"<old><sep>..."`, preserving the suffix. Boundary after the prefix must
    //    be the kebab (`-`) / snake (`_`) separator, so a longer sibling token is never clobbered.
    out = rewrite_rust_test_stanzas(
        &out,
        |stanza| {
            let mut s = stanza.to_string();
            let mut c = false;
            c |= replace_quoted_prefixed(&mut s, old_name, new_name, b'-');
            c |= replace_quoted_prefixed(&mut s, &old_ident, &new_ident, b'_');
            (s, c)
        },
        &mut changed,
    );

    // 3. same-package self-dep ":old-name" -> ":new-name"
    changed |= replace_token_label(&mut out, &format!(":{old_name}"), &format!(":{new_name}"));
    (out, changed)
}

/// Apply `f` to the interior text of every top-level `rust_test( ... )` stanza, splicing the
/// rewritten interior back in place. The stanza is delimited by `rust_test(` and its matching
/// close paren (paren-depth balanced, quote-aware so a `)` inside a string literal does not end
/// it). Non-`rust_test` text is copied verbatim. `changed` is OR-ed with any interior change.
fn rewrite_rust_test_stanzas(
    text: &str,
    f: impl Fn(&str) -> (String, bool),
    changed: &mut bool,
) -> String {
    const HEAD: &str = "rust_test(";
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if text[i..].starts_with(HEAD) {
            // Find the matching close paren for this call, quote-aware.
            let open = i + HEAD.len();
            if let Some(close) = matching_close_paren(text, open) {
                let interior = &text[open..close];
                let (new_interior, c) = f(interior);
                *changed |= c;
                out.push_str(HEAD);
                out.push_str(&new_interior);
                out.push(')');
                i = close + 1;
                continue;
            }
        }
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Given the byte index just AFTER an opening `(`, return the index of its matching `)`,
/// tracking nested parens and skipping `(`/`)` inside `"..."` string literals. Returns `None`
/// if unbalanced (in which case the caller leaves the stanza untouched, fail-safe).
fn matching_close_paren(text: &str, after_open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1i32;
    let mut i = after_open;
    let mut in_str = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'"' {
                in_str = false;
            }
            // BUCK string literals do not span the constructs we rewrite; no escape handling
            // needed for the codemod's vocabulary (no escaped quotes in name/crate values).
        } else {
            match b {
                b'"' => in_str = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += utf8_len(b);
    }
    None
}

/// Rewrite, in ANY BUCK file, every ABSOLUTE label that points at a moved crate:
/// `//old_path:old_target` -> `//new_path:new_target`. `moves_by_old_path` is keyed by the
/// moved crate's OLD repo-relative dir. Returns `(new_text, changed)`.
///
/// We rewrite the full `//old_path:` prefix first (covers the common `name == target` and
/// multi-target-in-package cases), then, for the canonical single-target label
/// `//old_path:old_name`, also rewrite the target component to `new_name`.
pub fn rewrite_buck_labels(
    buck_text: &str,
    moves_by_old_path: &BTreeMap<&str, &CrateMove>,
) -> (String, bool) {
    let mut out = buck_text.to_string();
    let mut changed = false;
    // Deterministic order: BTreeMap iterates sorted; but to avoid a shorter old_path being a
    // prefix of a longer one (e.g. cloud/a vs cloud/ab), process LONGEST old_path first.
    let mut entries: Vec<(&&str, &&CrateMove)> = moves_by_old_path.iter().collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(b.0)));
    for (old_path, cm) in entries {
        // Canonical single-target label //old:old_name -> //new:new_name (do this BEFORE the
        // prefix rewrite so the prefix rewrite does not strand a stale target component).
        let old_label = format!("//{}:{}", old_path, cm.old_cargo_name);
        let new_label = format!("//{}:{}", cm.new_path, cm.new_cargo_name);
        changed |= replace_token_label(&mut out, &old_label, &new_label);
        // Any other target in the moved package: //old:other -> //new:other (path only).
        let old_prefix = format!("//{old_path}:");
        let new_prefix = format!("//{}:", cm.new_path);
        changed |= replace_label_prefix(&mut out, &old_prefix, &new_prefix);
        // Bare package label //old (no colon, used for whole-package visibility) -> //new.
        let old_pkg = format!("//{old_path}");
        let new_pkg = format!("//{}", cm.new_path);
        changed |= replace_token_label(&mut out, &old_pkg, &new_pkg);
    }
    (out, changed)
}

/// Replace every `"needle"` (the WHOLE quoted token, exactly) with `"replacement"`. Used for
/// `name`/`crate` fields whose value IS the crate name (the `rust_library`/`rust_binary` case)
/// and for the same-package self-dep. The quote-delimited match guarantees a longer sibling
/// token (e.g. `"oya-x-bin"`) is never matched by the bare crate name `oya-x`.
fn replace_quoted_exact(haystack: &mut String, needle: &str, replacement: &str) -> bool {
    let from = format!("\"{needle}\"");
    let to = format!("\"{replacement}\"");
    if haystack.contains(&from) {
        *haystack = haystack.replace(&from, &to);
        true
    } else {
        false
    }
}

/// Replace, inside a quoted BUCK value, the crate-name PREFIX `old` with `new`, preserving any
/// suffix. A quoted value matches when it is exactly `"old"` OR `"old<sep>..."` — the `rust_test`
/// case, where the target carries a test-author suffix like `-unittest` / `_file_outbox`. `sep`
/// is the kebab (`-`) or snake (`_`) separator; requiring it as the boundary after the prefix (or
/// the closing quote) prevents clobbering a sibling whose name merely shares the prefix (e.g.
/// `"oya-x-domainx"` is NOT matched by `oya-x`). Callers scope this to `rust_test(...)` interiors
/// so a `-bin` sibling outside a test stanza is unaffected.
fn replace_quoted_prefixed(haystack: &mut String, old: &str, new: &str, sep: u8) -> bool {
    if old.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut result = String::with_capacity(haystack.len());
    let mut changed = false;
    let mut i = 0usize;
    while i < bytes.len() {
        // Look for an opening quote immediately followed by the prefix.
        if bytes[i] == b'"' && haystack[i + 1..].starts_with(old) {
            let after = i + 1 + old.len();
            // The char after the prefix must be the closing quote (exact match) or the separator
            // (suffixed match); anything else (alnum, other sep) means a different token.
            if after < bytes.len() && (bytes[after] == b'"' || bytes[after] == sep) {
                result.push('"');
                result.push_str(new);
                changed = true;
                i = after; // continue copying from the suffix (or the closing quote).
                continue;
            }
        }
        let ch_len = utf8_len(bytes[i]);
        result.push_str(&haystack[i..i + ch_len]);
        i += ch_len;
    }
    if changed {
        *haystack = result;
    }
    changed
}

/// Replace a full label token `from` with `to`, but only when `from` is NOT immediately
/// followed by an identifier character (so `:oya-x` does not match inside `:oya-x-app`, and
/// `//a/b` does not match inside `//a/bc`). The char before is already a label boundary
/// (quote, slash, comma, etc.) by construction of our `from` values.
fn replace_token_label(haystack: &mut String, from: &str, to: &str) -> bool {
    if from.is_empty() {
        return false;
    }
    let mut result = String::with_capacity(haystack.len());
    let mut changed = false;
    let bytes = haystack.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if haystack[i..].starts_with(from) {
            let after = i + from.len();
            let next_ok = after >= bytes.len() || !is_label_char(bytes[after]);
            if next_ok {
                result.push_str(to);
                i = after;
                changed = true;
                continue;
            }
        }
        // push one char (respecting UTF-8 boundaries; BUCK is ASCII in practice).
        let ch_len = utf8_len(bytes[i]);
        result.push_str(&haystack[i..i + ch_len]);
        i += ch_len;
    }
    if changed {
        *haystack = result;
    }
    changed
}

/// Replace a label PREFIX `from` (ending in `:`) with `to` (ending in `:`), preserving the
/// target component that follows. Boundary on the preceding char is guaranteed by `from`
/// starting with `//`. The target component after `:` is copied verbatim.
fn replace_label_prefix(haystack: &mut String, from: &str, to: &str) -> bool {
    if haystack.contains(from) {
        *haystack = haystack.replace(from, to);
        true
    } else {
        false
    }
}

/// A label/identifier continuation char: alnum, `-`, `_`, `/`, `.`. A `:` or quote ends a
/// label. This boundary prevents prefix collisions between sibling crate names.
fn is_label_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'/' || b == b'.'
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

    fn cm(old_path: &str, new_path: &str, old_name: &str, new_name: &str) -> CrateMove {
        CrateMove {
            old_path: old_path.to_string(),
            new_path: new_path.to_string(),
            old_cargo_name: old_name.to_string(),
            new_cargo_name: new_name.to_string(),
        }
    }

    #[test]
    fn moved_buck_rewrites_name_crate_and_self_dep() {
        let text = r#"rust_library(
    name = "oya-iam",
    crate = "oya_iam",
    crate_root = "src/lib.rs",
    deps = [],
)

rust_binary(
    name = "oya-iam-bin",
    crate_root = "src/main.rs",
    deps = [":oya-iam"],
)
"#;
        let m = cm(
            "cloud/cloud-iam/crates/oya-iam",
            "iam/core/iam",
            "oya-iam",
            "iam-core",
        );
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);
        assert!(out.contains("name = \"iam-core\""));
        assert!(out.contains("crate = \"iam_core\""));
        assert!(out.contains("deps = [\":iam-core\"]"));
        // crate_root path is move-invariant (relative to package).
        assert!(out.contains("crate_root = \"src/lib.rs\""));
        // The -bin sibling name must NOT be clobbered by the :oya-iam self-dep rewrite.
        assert!(out.contains("name = \"oya-iam-bin\""));
    }

    #[test]
    fn absolute_label_rewritten_path_and_target() {
        let text = r#"deps = [
    "//cloud/cloud-iam/crates/oya-iam:oya-iam",
    "//libs/oya-kernel:oya-kernel",
]
"#;
        let m = cm(
            "cloud/cloud-iam/crates/oya-iam",
            "iam/core/iam",
            "oya-iam",
            "iam-core",
        );
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/cloud-iam/crates/oya-iam", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed);
        assert!(out.contains("//iam/core/iam:iam-core"));
        // unrelated label untouched.
        assert!(out.contains("//libs/oya-kernel:oya-kernel"));
    }

    #[test]
    fn sibling_label_prefix_not_clobbered() {
        // moving cloud/a but NOT cloud/ab; ensure //cloud/ab:... is untouched.
        let text = r#"deps = ["//cloud/a:a", "//cloud/ab:ab"]"#;
        let m = cm("cloud/a", "x/a", "a", "a2");
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/a", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed);
        assert!(out.contains("//x/a:a2"));
        assert!(out.contains("//cloud/ab:ab"), "sibling label preserved: {out}");
    }

    #[test]
    fn self_dep_label_not_substring_matched() {
        // :oya-iam must not match inside :oya-iam-app.
        let text = r#"deps = [":oya-iam-app", ":oya-iam"]"#;
        let m = cm("c/x", "y/x", "oya-iam", "iam");
        let (out, _changed) = rewrite_moved_buck(text, &m);
        assert!(out.contains(":oya-iam-app"), "longer sibling preserved: {out}");
        assert!(out.contains(":iam\""));
    }

    #[test]
    fn no_op_buck_is_byte_identical() {
        let text = "rust_library(name = \"oya-unrelated\")\n";
        let by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(!changed);
        assert_eq!(out, text);
    }

    /// Regression for PR #735 (review MED): the moved-BUCK rewrite renamed `rust_library`
    /// `name`/`crate` but MISSED `rust_test` `name`/`crate`, which carry the crate name PLUS a
    /// suffix (`-unittest`, `_file_outbox`) and so never matched the exact-token rewrite. This
    /// asserts the suffixed test `name`/`crate` ARE renamed (prefix only, suffix preserved), that
    /// non-identifier fields are untouched, AND that a non-test `-bin` sibling target is NOT
    /// clobbered by the prefix rewrite.
    #[test]
    fn moved_buck_rewrites_suffixed_rust_test_name_and_crate() {
        let text = r#"rust_library(
    name = "oya-eventing-file-adapter",
    crate = "oya_eventing_file_adapter",
    crate_root = "src/lib.rs",
    srcs = ["src/lib.rs"],
)

rust_binary(
    name = "oya-eventing-file-adapter-bin",
    crate = "oya_eventing_file_adapter_bin",
    crate_root = "src/main.rs",
)

rust_test(
    name = "oya-eventing-file-adapter-file-outbox",
    srcs = ["tests/file_outbox.rs"],
    crate = "oya_eventing_file_adapter_file_outbox",
    crate_root = "tests/file_outbox.rs",
    visibility = ["PUBLIC"],
)
"#;
        let m = cm(
            "oya/eventing/crates/oya-eventing-file-adapter",
            "messaging/adapters/file",
            "oya-eventing-file-adapter",
            "messaging-file-adapter",
        );
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);

        // The library's own exact name/crate rewrite (unchanged behavior).
        assert!(out.contains("name = \"messaging-file-adapter\""), "{out}");
        assert!(out.contains("crate = \"messaging_file_adapter\""), "{out}");

        // THE GAP: the suffixed rust_test name/crate are now renamed, prefix-only.
        assert!(
            out.contains("name = \"messaging-file-adapter-file-outbox\""),
            "rust_test name prefix renamed + suffix preserved: {out}"
        );
        assert!(
            out.contains("crate = \"messaging_file_adapter_file_outbox\""),
            "rust_test crate prefix renamed + suffix preserved: {out}"
        );
        // The stale-prefix tokens that REMAIN belong ONLY to the `-bin` sibling (asserted below);
        // the rust_test and rust_library stanzas carry no stale prefix.
        assert!(
            !out.contains("name = \"oya-eventing-file-adapter-file-outbox\""),
            "stale rust_test name eliminated: {out}"
        );
        assert!(
            !out.contains("crate = \"oya_eventing_file_adapter_file_outbox\""),
            "stale rust_test crate eliminated: {out}"
        );

        // Non-identifier fields untouched.
        assert!(out.contains("srcs = [\"tests/file_outbox.rs\"]"), "{out}");
        assert!(
            out.contains("crate_root = \"tests/file_outbox.rs\""),
            "{out}"
        );
        assert!(out.contains("visibility = [\"PUBLIC\"]"), "{out}");
        assert!(out.contains("crate_root = \"src/lib.rs\""), "{out}");

        // The `-bin` sibling is a SEPARATE crate (own move); it must NOT be prefix-clobbered by
        // the library rename — its name/crate stay as-authored.
        assert!(
            out.contains("name = \"oya-eventing-file-adapter-bin\""),
            "non-test -bin sibling preserved: {out}"
        );
        assert!(
            out.contains("crate = \"oya_eventing_file_adapter_bin\""),
            "non-test -bin sibling crate preserved: {out}"
        );

        // Reversibility: the inverse move restores the original byte-for-byte.
        let inv = cm(
            "messaging/adapters/file",
            "oya/eventing/crates/oya-eventing-file-adapter",
            "messaging-file-adapter",
            "oya-eventing-file-adapter",
        );
        let (round, _c) = rewrite_moved_buck(&out, &inv);
        assert_eq!(round, text, "inverse must round-trip byte-identically");
    }

    /// The `-unittest` flavor (same-dir test target for a library), proving the kebab + snake
    /// suffix preservation for the `messaging-domain` crate shape too.
    #[test]
    fn moved_buck_rewrites_unittest_rust_test_target() {
        let text = r#"rust_library(
    name = "oya-eventing-domain",
    crate = "oya_eventing_domain",
    crate_root = "src/lib.rs",
)

rust_test(
    name = "oya-eventing-domain-unittest",
    crate = "oya_eventing_domain",
    crate_root = "src/lib.rs",
)
"#;
        let m = cm(
            "oya/eventing/crates/oya-eventing-domain",
            "messaging/core/domain",
            "oya-eventing-domain",
            "messaging-domain",
        );
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);
        assert!(
            out.contains("name = \"messaging-domain-unittest\""),
            "unittest target name renamed: {out}"
        );
        // The unittest target shares the library's snake crate ident (no suffix on `crate` here);
        // it is renamed by the exact pass.
        assert!(out.contains("crate = \"messaging_domain\""), "{out}");
        assert!(!out.contains("oya-eventing"), "{out}");
        assert!(!out.contains("oya_eventing"), "{out}");
    }
}
