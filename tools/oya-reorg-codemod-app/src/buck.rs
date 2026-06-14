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
pub fn rewrite_moved_buck(buck_text: &str, cm: &CrateMove) -> (String, bool) {
    let old_name = &cm.old_cargo_name;
    let new_name = &cm.new_cargo_name;
    let old_ident = snake(old_name);
    let new_ident = snake(new_name);

    let mut out = buck_text.to_string();
    let mut changed = false;

    // name = "old" -> name = "new"  (kebab target name; e.g. rust_library/binary/test name).
    changed |= replace_quoted(&mut out, old_name, new_name);
    // crate = "old_snake" -> "new_snake"
    if old_ident != *old_name {
        changed |= replace_quoted(&mut out, &old_ident, &new_ident);
    } else {
        // old_name had no dash so kebab == snake; the replace above already covered it.
    }
    // same-package self-dep ":old-name" -> ":new-name"
    changed |= replace_token_label(&mut out, &format!(":{old_name}"), &format!(":{new_name}"));
    (out, changed)
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

/// Replace every `"needle"` (exactly quoted) with `"replacement"`. Used for `name`/`crate`
/// fields where the value is a standalone quoted token.
fn replace_quoted(haystack: &mut String, needle: &str, replacement: &str) -> bool {
    let from = format!("\"{needle}\"");
    let to = format!("\"{replacement}\"");
    if haystack.contains(&from) {
        *haystack = haystack.replace(&from, &to);
        true
    } else {
        false
    }
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
}
