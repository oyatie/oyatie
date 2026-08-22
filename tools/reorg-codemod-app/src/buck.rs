//! BUCK transforms: rewrite absolute `//old/path:target` labels (in `deps`, `visibility`,
//! `env`, etc.) to `//new/path:new-target`, and rewrite the moved crate's own `name`,
//! `crate`, and `crate_root` fields. BUCK files are Starlark, not TOML, so the rewrites are
//! token-precise string substitutions over the label/name vocabulary the codemod owns.
//!
//! Two label flavors exist in the tree:
//! * **absolute** `//cloud/cloud-iam/crates/x:x` — root-anchored; rewritten to the
//!   new path + new target name wherever it appears in ANY BUCK file;
//! * **same-package** `:x` — relative to the package; only the target NAME changes
//!   (handled by the in-file name rewrite of the moved crate's own BUCK).
//!
//! All substitutions are anchored on a non-identifier boundary so a label is never matched
//! as a substring of a longer label (e.g. `:x` must not match inside `:x-app`).

use std::collections::BTreeMap;

use crate::model::{CrateMove, snake};

/// Rewrite a MOVED crate's own `BUCK`: its `name = "..."`, `crate = "..."`,
/// `crate_root = "..."` (path stays relative so it is move-invariant), and any
/// same-package `:old-name` self-references in `deps`. Returns `(new_text, changed)`.
///
/// `name`/`crate` come in two flavors that need different match rules:
/// * EXACT-valued targets (`rust_library`/`rust_binary`) whose `name`/`crate` IS the crate
///   name — matched as the whole quoted token `"old"`. A `-bin` sibling like `"x-bin"` is
///   a SEPARATE crate with its own move, so it must NOT be prefix-matched here.
/// * SUFFIXED `rust_test` targets whose `name`/`crate` is `<crate-name><sep><suffix>` (e.g.
///   `x-domain-unittest`, crate `x_domain_file_outbox`). The codemod owns the
///   crate-name PREFIX; the suffix (`-unittest`, `_file_outbox`) is test-author-chosen and is
///   preserved. To stay safe against the `-bin`-sibling ambiguity, the prefix rewrite is scoped
///   to the `name`/`crate` FIELDS of `rust_test( ... )` stanzas ONLY, and a value is rewritten
///   only when the LONGEST package crate name that prefixes it IS the moving crate — so the
///   `-bin` sibling's own test (`x-bin-unittest`, longest prefix `x-bin`) is left
///   untouched even though the bare `x` also prefixes it (the B1 silent-clobber fix).
pub fn rewrite_moved_buck(buck_text: &str, cm: &CrateMove) -> (String, bool) {
    let old_name = &cm.old_cargo_name;
    let new_name = &cm.new_cargo_name;
    let old_ident = snake(old_name);
    let new_ident = snake(new_name);

    // The package's defined crate vocabulary, collected from the ORIGINAL (as-authored) text so it
    // contains the moving crate's OLD name — the disambiguation set for step 2's longest-prefix
    // match against the still-old `rust_test` `name`/`crate` values. Must precede step 1's exact
    // rename (which would otherwise replace the moving crate's name in the set with its new name).
    let kebab_set = collect_crate_set(buck_text, false);
    let snake_set = collect_crate_set(buck_text, true);

    // 1. Exact whole-VALUE rewrite of the moved crate's own `name`/`crate` fields (lib/binary/test
    //    exact values). FIELD-KEY-anchored, NOT a blind text replace: the `name` field carries the
    //    KEBAB crate name and the `crate` field the SNAKE ident, so each field is mapped to its own
    //    flavor's new value. This matters when kebab == snake (a single-token name like `iam`, the
    //    #61 round-trip fix): a blind exact replace of `"iam"` would rewrite BOTH `name = "iam"` and
    //    `crate = "iam"` to the SAME (kebab) new value, so the inverse (`iam` -> `x`) could not
    //    restore `crate = "x"` (it produced `crate = "x"`), breaking byte round-trip. Keying
    //    on the field disambiguates: `name` -> kebab `new_name`, `crate` -> snake `new_ident`, even
    //    when the old values are textually identical. A `-bin` sibling's `"x-bin"` is never the
    //    whole value `"x"`, so the whole-value match still leaves it untouched.
    let mut out = buck_text.to_string();
    let mut changed = false;
    changed |= rewrite_field_exact(&mut out, "name", old_name, new_name);
    changed |= rewrite_field_exact(&mut out, "crate", &old_ident, &new_ident);

    // 2. Suffixed prefix rewrite, scoped to the `name`/`crate` FIELDS of `rust_test(...)` stanzas.
    //    A `rust_test` `name`/`crate` is `<crate-name><sep><suffix>` (e.g. `x-unittest`,
    //    `x_file_outbox`); the codemod owns the crate-name PREFIX and preserves the suffix.
    //    Disambiguation against a `-bin` sibling (whose own test target is `x-bin-unittest`)
    //    is by LONGEST crate-name prefix against the package's defined crate set: a value is only
    //    rewritten when the longest package crate that prefixes it IS the moving crate. If the
    //    longest prefix is a different crate (the `-bin` sibling) the value is left untouched —
    //    this is the B1 silent-clobber fix. Field-key anchoring (only `name`/`crate`) prevents
    //    rewriting arbitrary quoted values (`env`/`deps`/`srcs`) inside the stanza.
    out = rewrite_rust_test_stanzas(
        &out,
        |stanza| {
            let mut s = stanza.to_string();
            let mut c = false;
            c |= rewrite_test_field(&mut s, "name", old_name, new_name, b'-', &kebab_set);
            c |= rewrite_test_field(&mut s, "crate", &old_ident, &new_ident, b'_', &snake_set);
            (s, c)
        },
        &mut changed,
    );

    // 3. same-package self-dep ":old-name" -> ":new-name"
    changed |= replace_token_label(&mut out, &format!(":{old_name}"), &format!(":{new_name}"));
    (out, changed)
}

/// Apply `f` to the interior text of every top-level `rust_test( ... )` stanza, splicing the
/// rewritten interior back in place. A stanza head is the identifier `rust_test` (anchored on a
/// non-identifier boundary BEFORE it — start-of-file, whitespace, newline, `(`, or `,` — so a
/// longer macro like `custom_rust_test(` is NOT matched as a stanza head), followed by optional
/// whitespace and the opening `(`. The stanza is delimited by that `(` and its matching close
/// paren (paren-depth balanced, quote-aware so a `)` inside a string literal does not end it).
/// Non-`rust_test` text is copied verbatim. `changed` is OR-ed with any interior change.
fn rewrite_rust_test_stanzas(
    text: &str,
    f: impl Fn(&str) -> (String, bool),
    changed: &mut bool,
) -> String {
    const IDENT: &str = "rust_test";
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if text[i..].starts_with(IDENT) && (i == 0 || !is_ident_char(bytes[i - 1])) {
            // Skip optional whitespace between `rust_test` and `(` (the `rust_test (` space form).
            let mut j = i + IDENT.len();
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' {
                let open = j + 1;
                // Find the matching close paren for this call, quote-aware.
                if let Some(close) = matching_close_paren(text, open) {
                    let interior = &text[open..close];
                    let (new_interior, c) = f(interior);
                    *changed |= c;
                    out.push_str(&text[i..open]); // `rust_test` + optional ws + `(`
                    out.push_str(&new_interior);
                    out.push(')');
                    i = close + 1;
                    continue;
                }
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
/// `//old_path:old_target` -> `//new_path:new_target`, AND every NON-`//` repo-rooted
/// SOURCE-PATH LITERAL that points into a moved crate dir (`crate_root = "old_path/..."`,
/// `mapped_srcs` VALUES `"old_path/..."`). `moves_by_old_path` is keyed by the moved crate's OLD
/// repo-relative dir. Returns `(new_text, changed)`.
///
/// We rewrite the full `//old_path:` prefix first (covers the common `name == target` and
/// multi-target-in-package cases), then, for the canonical single-target label
/// `//old_path:old_name`, also rewrite the target component to `new_name`.
///
/// The source-path-literal rewrite (#63) handles the class the `//` label rewrite misses: a BUCK
/// may reference a path INSIDE a moved crate's dir through a repo-ROOTED (non-`//`) plain string,
/// e.g. `crate_root = "cloud/cloud-iac/crates/x/tests/t.rs"`, a `mapped_srcs` value
/// `"cloud/cloud-iac/crates/x/tests/t.rs": "tests/t.rs"`, or a `genrule` `cmd`/`data` entry
/// naming a file under the crate. These are plain Starlark strings, not `//labels`, so when the dir
/// moves they are left pointing at a now-dead path (a stale BUCK the build can no longer resolve).
/// The rewrite is deliberately FIELD-AGNOSTIC: it updates EVERY quoted repo-rooted literal whose
/// value is exactly `old_path` or begins with `old_path/`, in any attribute, replacing only the
/// `old_path` PREFIX with `new_path` and preserving the in-crate suffix. Field-agnostic is the
/// CORRECT scope, not an over-reach: any path UNDER the moved dir genuinely moved, so every
/// reference to it (crate_root, a mapped_srcs value OR a repo-rooted key, a genrule cmd/data entry)
/// must follow — narrowing to a fixed `{crate_root, mapped_srcs}` field set would strand the
/// genrule/data references. See [`replace_source_path_literal`] for the exact boundary rules and the
/// one accepted imprecision (a path embedded in a comment).
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
        // NON-`//` repo-rooted literal `"old_path"` / `"old_path/..."` (ANY quoted value under the
        // moved dir — crate_root, mapped_srcs, genrule cmd/data, Starlark vars) -> `"new_path..."`
        // (#63; field-agnostic by design — see `replace_source_path_literal`).
        changed |= replace_source_path_literal(&mut out, old_path, &cm.new_path);
    }
    (out, changed)
}

/// Rewrite every quoted repo-rooted literal whose value is exactly `old_path` or begins with
/// `old_path` + `/` — replacing only the `old_path` PREFIX with `new_path` and preserving the rest.
/// This is FIELD-AGNOSTIC by design (see [`rewrite_buck_labels`]): any quoted string naming a path
/// under the moved dir is a reference that moved, so it is rewritten regardless of which attribute
/// holds it (crate_root, a mapped_srcs value OR a repo-rooted key, a genrule cmd/data entry). The
/// match is anchored on the OPENING quote: the char before `old_path` must be `"` and the char after
/// the matched `old_path` must be `/` (descendant) or `"` (the whole value), so:
/// * a `"//old_path:target"` LABEL is NOT matched (its `old_path` is preceded by `//`, already
///   rewritten by the label passes) — preventing a double rewrite;
/// * a longer sibling dir literal `"old_path_extra/..."` is NOT matched (the boundary char after
///   `old_path` would be `_`, not `/` or `"`);
/// * a PACKAGE-RELATIVE key/value (`"tests/x.rs"`) is NOT matched — NOT because keys are excluded,
///   but because it does not start with the moved crate dir; a REPO-ROOTED key under the moved dir
///   WOULD be (correctly) rewritten.
///
/// One accepted imprecision: a path under the moved dir embedded in a COMMENT is also rewritten.
/// That is harmless — it keeps the comment accurate and still round-trips — and not worth a
/// Starlark-aware parser to avoid. Returns whether any literal changed.
fn replace_source_path_literal(haystack: &mut String, old_path: &str, new_path: &str) -> bool {
    if old_path.is_empty() || old_path == new_path {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut result = String::with_capacity(haystack.len());
    let mut changed = false;
    let mut i = 0usize;
    while i < bytes.len() {
        if haystack[i..].starts_with(old_path)
            // preceded by an opening quote (excludes `//old_path` labels + mid-path occurrences).
            && i > 0
            && bytes[i - 1] == b'"'
        {
            let after = i + old_path.len();
            // followed by a path boundary: `/` (descendant) or the closing `"` (the whole value).
            let boundary_ok = after < bytes.len() && (bytes[after] == b'/' || bytes[after] == b'"');
            if boundary_ok {
                result.push_str(new_path);
                i = after;
                changed = true;
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

/// Rewrite, anywhere in `haystack`, the value of a `key = "<old_value>"` field to
/// `key = "<new_value>"` when the WHOLE quoted value equals `old_value` exactly. FIELD-KEY-anchored
/// (the `key` must be a whole identifier on a non-identifier boundary, then ws* `=` ws* `"..."`), so
/// only the named field is touched and a value that merely contains the crate name in another field
/// (`crate_root`, `env`, a label inside `deps`) is left alone. The whole-VALUE equality guarantees a
/// longer sibling token (`"x-bin"`) is never matched by the bare crate name `x`.
///
/// Keying on the field — rather than a blind quoted-token replace — is what makes the moved-crate
/// `name` vs `crate` rename correct when the kebab name equals its snake ident (a single-token name,
/// the #61 fix): `name` carries the kebab vocabulary and `crate` the snake, so each is mapped to its
/// own new flavor even though the two old values are textually identical.
fn rewrite_field_exact(haystack: &mut String, key: &str, old_value: &str, new_value: &str) -> bool {
    if old_value.is_empty() || old_value == new_value {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut result = String::with_capacity(haystack.len());
    let mut changed = false;
    let mut i = 0usize;
    while i < bytes.len() {
        // Anchor on the field key: `<key>` (whole ident) ws* `=` ws* `"`.
        if haystack[i..].starts_with(key) && (i == 0 || !is_ident_char(bytes[i - 1])) {
            let mut j = i + key.len();
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                let mut k = j + 1;
                while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'"' {
                    let val_start = k + 1;
                    if let Some(end_rel) = haystack[val_start..].find('"') {
                        let val = &haystack[val_start..val_start + end_rel];
                        result.push_str(&haystack[i..val_start]); // field key + `= "` verbatim.
                        if val == old_value {
                            result.push_str(new_value);
                            changed = true;
                        } else {
                            result.push_str(val);
                        }
                        i = val_start + end_rel; // resume at the closing quote.
                        continue;
                    }
                }
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

/// Collect the package's defined crate set from `buck_text`: the `name` value of every
/// `rust_library(`/`rust_binary(` stanza (kebab when `snake == false`), or its snake `crate`
/// ident (when `snake == true`, derived from the kebab `name`). This is the disambiguation set
/// the `rust_test` `name`/`crate` rewrite matches a longest crate-name prefix against. Binaries
/// and libraries are exactly the crates whose `name`/`crate` IS the crate name, so their `name`
/// fields are the authoritative crate vocabulary defined in this BUCK.
fn collect_crate_set(buck_text: &str, want_snake: bool) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for head in ["rust_library", "rust_binary"] {
        let bytes = buck_text.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if buck_text[i..].starts_with(head) && (i == 0 || !is_ident_char(bytes[i - 1])) {
                let mut j = i + head.len();
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'(' {
                    let open = j + 1;
                    if let Some(close) = matching_close_paren(buck_text, open) {
                        let interior = &buck_text[open..close];
                        if let Some(name) = field_value(interior, "name") {
                            let crate_name = if want_snake { snake(&name) } else { name };
                            if !set.contains(&crate_name) {
                                set.push(crate_name);
                            }
                        }
                        i = close + 1;
                        continue;
                    }
                }
            }
            i += utf8_len(bytes[i]);
        }
    }
    set
}

/// Read the quoted value of a top-level `key = "..."` field inside a stanza interior, returning
/// the unquoted string. Field-key-anchored: `key` must be a whole identifier (non-identifier
/// boundary before it) followed by optional whitespace, `=`, optional whitespace, then `"...".`
/// Returns the FIRST such field's value, or `None` if absent.
fn field_value(interior: &str, key: &str) -> Option<String> {
    let bytes = interior.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if interior[i..].starts_with(key) && (i == 0 || !is_ident_char(bytes[i - 1])) {
            let mut j = i + key.len();
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'"' {
                    let val_start = j + 1;
                    if let Some(end_rel) = interior[val_start..].find('"') {
                        return Some(interior[val_start..val_start + end_rel].to_string());
                    }
                }
            }
        }
        i += utf8_len(bytes[i]);
    }
    None
}

/// Rewrite, inside a `rust_test(...)` stanza interior, the value of the `key` field (`name` or
/// `crate`) when its longest package-crate prefix is the moving crate. Field-key-anchored: only
/// `key = "..."` is touched, never arbitrary quoted values (`env`/`deps`/`srcs`) — that is the
/// MED-1 over-broad-rewrite fix. For the matched value V, the longest crate `C*` in `crate_set`
/// that prefixes V (V == C* OR V starts with `C* + sep`) is found; if `C* == old` the `C*`
/// portion is rewritten to `new` and the suffix preserved, else V is left UNCHANGED (the B1
/// `-bin`-sibling fix). Returns whether any field value changed.
fn rewrite_test_field(
    haystack: &mut String,
    key: &str,
    old: &str,
    new: &str,
    sep: u8,
    crate_set: &[String],
) -> bool {
    if old.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut result = String::with_capacity(haystack.len());
    let mut changed = false;
    let mut i = 0usize;
    while i < bytes.len() {
        // Anchor on the field key: `<key>` (whole ident) ws* `=` ws* `"`.
        if haystack[i..].starts_with(key) && (i == 0 || !is_ident_char(bytes[i - 1])) {
            let mut j = i + key.len();
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                let mut k = j + 1;
                while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'"' {
                    let val_start = k + 1;
                    if let Some(end_rel) = haystack[val_start..].find('"') {
                        let val = &haystack[val_start..val_start + end_rel];
                        // Copy the field key + `= "` verbatim.
                        result.push_str(&haystack[i..val_start]);
                        match longest_crate_prefix(val, crate_set, sep) {
                            Some(cstar) if cstar == old => {
                                // Rewrite the C* prefix to `new`, preserve the suffix.
                                result.push_str(new);
                                result.push_str(&val[old.len()..]);
                                changed = true;
                            }
                            _ => result.push_str(val), // different crate or no prefix: leave.
                        }
                        i = val_start + end_rel; // resume at the closing quote.
                        continue;
                    }
                }
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

/// Find the LONGEST crate name in `crate_set` that is a prefix of `val`, where "prefix" means
/// `val == C` OR `val` starts with `C + sep` (sep = `-` for kebab `name`, `_` for snake `crate`).
/// Returns the matched crate name, or `None` if none prefixes `val`. Longest-wins disambiguates a
/// lib `x` from its `-bin` sibling `x-bin`: for `x-bin-unittest`, both `x` and
/// `x-bin` prefix it, but the longest (`x-bin`) is returned, so a lib move leaves it.
fn longest_crate_prefix<'a>(val: &str, crate_set: &'a [String], sep: u8) -> Option<&'a str> {
    let mut best: Option<&str> = None;
    for c in crate_set {
        if c.is_empty() {
            continue;
        }
        let is_prefix = val == c.as_str()
            || (val.len() > c.len()
                && val.as_bytes()[c.len()] == sep
                && val.starts_with(c.as_str()));
        if is_prefix && best.map(|b| c.len() > b.len()).unwrap_or(true) {
            best = Some(c.as_str());
        }
    }
    best
}

/// An identifier continuation char (used to anchor stanza heads + field keys on a non-identifier
/// boundary). BUCK identifiers are alnum + `_`.
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Replace a full label token `from` with `to`, but only when `from` is NOT immediately
/// followed by an identifier character (so `:x` does not match inside `:x-app`, and
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
    name = "iam",
    crate = "iam",
    crate_root = "src/lib.rs",
    deps = [],
)

rust_binary(
    name = "iam-bin",
    crate_root = "src/main.rs",
    deps = [":iam"],
)
"#;
        let m = cm(
            "cloud/cloud-iam/crates/iam",
            "iam/core/iam",
            "iam",
            "iam-core",
        );
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);
        assert!(out.contains("name = \"iam-core\""));
        assert!(out.contains("crate = \"iam_core\""));
        assert!(out.contains("deps = [\":iam-core\"]"));
        // crate_root path is move-invariant (relative to package).
        assert!(out.contains("crate_root = \"src/lib.rs\""));
        // The -bin sibling name must NOT be clobbered by the :iam self-dep rewrite.
        assert!(out.contains("name = \"iam-bin\""));
    }

    #[test]
    fn absolute_label_rewritten_path_and_target() {
        let text = r#"deps = [
    "//cloud/cloud-iam/crates/iam:iam",
    "//libs/kernel:kernel",
]
"#;
        let m = cm(
            "cloud/cloud-iam/crates/iam",
            "iam/core/iam",
            "iam",
            "iam-core",
        );
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/cloud-iam/crates/iam", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed);
        assert!(out.contains("//iam/core/iam:iam-core"));
        // unrelated label untouched.
        assert!(out.contains("//libs/kernel:kernel"));
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
        assert!(
            out.contains("//cloud/ab:ab"),
            "sibling label preserved: {out}"
        );
    }

    #[test]
    fn self_dep_label_not_substring_matched() {
        // :iam must not match inside :iam-app.
        let text = r#"deps = [":iam-app", ":iam"]"#;
        let m = cm("c/x", "y/x", "iam", "iam");
        let (out, _changed) = rewrite_moved_buck(text, &m);
        assert!(
            out.contains(":iam-app"),
            "longer sibling preserved: {out}"
        );
        assert!(out.contains(":iam\""));
    }

    #[test]
    fn no_op_buck_is_byte_identical() {
        let text = "rust_library(name = \"unrelated\")\n";
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
    name = "eventing-file-adapter",
    crate = "eventing_file_adapter",
    crate_root = "src/lib.rs",
    srcs = ["src/lib.rs"],
)

rust_binary(
    name = "eventing-file-adapter-bin",
    crate = "eventing_file_adapter_bin",
    crate_root = "src/main.rs",
)

rust_test(
    name = "eventing-file-adapter-file-outbox",
    srcs = ["tests/file_outbox.rs"],
    crate = "eventing_file_adapter_file_outbox",
    crate_root = "tests/file_outbox.rs",
    visibility = ["PUBLIC"],
)
"#;
        let m = cm(
            "oya/eventing/crates/eventing-file-adapter",
            "messaging/adapters/file",
            "eventing-file-adapter",
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
            !out.contains("name = \"eventing-file-adapter-file-outbox\""),
            "stale rust_test name eliminated: {out}"
        );
        assert!(
            !out.contains("crate = \"eventing_file_adapter_file_outbox\""),
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
            out.contains("name = \"eventing-file-adapter-bin\""),
            "non-test -bin sibling preserved: {out}"
        );
        assert!(
            out.contains("crate = \"eventing_file_adapter_bin\""),
            "non-test -bin sibling crate preserved: {out}"
        );

        // Reversibility: the inverse move restores the original byte-for-byte.
        let inv = cm(
            "messaging/adapters/file",
            "oya/eventing/crates/eventing-file-adapter",
            "messaging-file-adapter",
            "eventing-file-adapter",
        );
        let (round, _c) = rewrite_moved_buck(&out, &inv);
        assert_eq!(round, text, "inverse must round-trip byte-identically");
    }

    /// The `-unittest` flavor (same-dir test target for a library), proving the kebab + snake
    /// suffix preservation for the `messaging-domain` crate shape too.
    #[test]
    fn moved_buck_rewrites_unittest_rust_test_target() {
        let text = r#"rust_library(
    name = "eventing-domain",
    crate = "eventing_domain",
    crate_root = "src/lib.rs",
)

rust_test(
    name = "eventing-domain-unittest",
    crate = "eventing_domain",
    crate_root = "src/lib.rs",
)
"#;
        let m = cm(
            "oya/eventing/crates/eventing-domain",
            "messaging/core/domain",
            "eventing-domain",
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
        assert!(!out.contains("eventing"), "{out}");
        assert!(!out.contains("eventing"), "{out}");
    }

    /// B1 (PR #735 review, BLOCKING): silent `-bin`-sibling clobber. A package with a library
    /// `x` AND a `-bin` binary sibling `x-bin` (whose own test target is
    /// `x-bin-unittest` / `x_bin_tests`). Moving the LIBRARY `x` -> `new-x` previously
    /// front-matched `"x"` + `-` inside `x-bin-unittest` and rewrote it to
    /// `new-x-bin-unittest`, leaving the binary `x-bin` itself untouched — a permanently
    /// inconsistent BUCK. The longest-crate-prefix match must leave the `-bin` sibling's test
    /// targets untouched (their longest package-crate prefix is `x-bin`, not the moving
    /// `x`), and must round-trip byte-identically through the inverse. (`new-x` is the realistic
    /// multi-segment destination name; the brief's worked examples use `new` as shorthand.)
    #[test]
    fn moved_buck_leaves_bin_sibling_rust_test_untouched() {
        let text = r#"rust_library(
    name = "x",
    crate = "x",
    crate_root = "src/lib.rs",
)

rust_binary(
    name = "x-bin",
    crate_root = "src/main.rs",
    deps = [":x"],
)

rust_test(
    name = "x-unittest",
    crate = "x_tests",
    crate_root = "src/lib.rs",
)

rust_test(
    name = "x-bin-unittest",
    crate = "x_bin_tests",
    crate_root = "src/main.rs",
)
"#;
        let m = cm("cloud/c/x", "cap/core/new-x", "x", "new-x");
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);

        // Library's own name/crate renamed (exact pass, unchanged behavior). snake(`x`)=`x`,
        // snake(`new-x`)=`new_x`, so lib `name "x"` -> `new-x`, `crate "x"` -> `new_x`.
        assert!(out.contains("name = \"new-x\""), "{out}");
        assert!(
            out.contains("crate = \"new_x\""),
            "lib crate is the new snake ident: {out}"
        );

        // Library's own `-unittest` test: longest kebab prefix `x` == moving -> `new-x-unittest`.
        assert!(
            out.contains("name = \"new-x-unittest\""),
            "lib unittest name renamed: {out}"
        );
        // Its `crate = "x_tests"`: longest snake prefix `x` == moving -> `new_x_tests`.
        assert!(
            out.contains("crate = \"new_x_tests\""),
            "lib unittest crate renamed: {out}"
        );

        // THE B1 FIX: the binary `-bin` and its `-bin-unittest` test are UNCHANGED.
        assert!(
            out.contains("name = \"x-bin\""),
            "binary name preserved: {out}"
        );
        assert!(
            out.contains("name = \"x-bin-unittest\""),
            "B1: bin-unittest name NOT clobbered: {out}"
        );
        assert!(
            out.contains("crate = \"x_bin_tests\""),
            "B1: bin-unittest crate NOT clobbered: {out}"
        );
        // The clobbered shape that the un-fixed code produced must NOT appear.
        assert!(
            !out.contains("new-x-bin-unittest"),
            "B1: no front-clobbered name: {out}"
        );
        assert!(
            !out.contains("new_x_bin_tests"),
            "B1: no front-clobbered crate: {out}"
        );
        // The self-dep `:x` IS rewritten (exact label), but `:x-bin` is not present here.
        assert!(
            out.contains("deps = [\":new-x\"]"),
            "self-dep rewritten: {out}"
        );

        // Inverse round-trips byte-identically, including the untouched `-bin-unittest`.
        let inv = cm("cap/core/new-x", "cloud/c/x", "new-x", "x");
        let (round, _c) = rewrite_moved_buck(&out, &inv);
        assert_eq!(round, text, "inverse must round-trip byte-identically");
    }

    /// #61 (single-token destination round-trip): when a destination cargo name is a single
    /// hyphen-free token, its kebab name EQUALS its snake ident (`snake("iam") == "iam"`). The old
    /// step-1 used a blind quoted-token replace and SKIPPED the snake pass when kebab == snake, so on
    /// the INVERSE move (`iam` -> `x`) the one exact pass rewrote BOTH `name = "iam"` AND
    /// `crate = "iam"` to the kebab `x`, producing `crate = "x"` instead of `crate = "x"`
    /// — the tree could not round-trip byte-identically. The field-aware exact rewrite maps `name` to
    /// the kebab and `crate` to the snake ident independently, so forward + inverse restore the bytes.
    #[test]
    fn single_token_destination_round_trips_name_and_crate() {
        let text = r#"rust_library(
    name = "x",
    crate = "x",
    crate_root = "src/lib.rs",
    srcs = ["src/lib.rs"],
)

rust_binary(
    name = "x-bin",
    crate = "x_bin",
    crate_root = "src/main.rs",
    deps = [":x"],
)
"#;
        // Destination is a SINGLE hyphen-free token: kebab `iam` == snake `iam`.
        let fwd_move = cm("cloud/c/x", "iam/core/iam", "x", "iam");
        let (fwd, fwd_changed) = rewrite_moved_buck(text, &fwd_move);
        assert!(fwd_changed);
        // name -> kebab `iam`, crate -> snake `iam` (both are `iam` here, but via DIFFERENT flavors).
        assert!(fwd.contains("name = \"iam\""), "lib name -> kebab: {fwd}");
        assert!(fwd.contains("crate = \"iam\""), "lib crate -> snake: {fwd}");
        // crate_root (a path) is move-invariant and never a crate-name field.
        assert!(fwd.contains("crate_root = \"src/lib.rs\""), "{fwd}");
        // the `-bin` sibling is a SEPARATE crate (its own move) — whole-value match leaves it.
        assert!(
            fwd.contains("name = \"x-bin\""),
            "bin sibling name preserved: {fwd}"
        );
        assert!(
            fwd.contains("crate = \"x_bin\""),
            "bin sibling crate preserved: {fwd}"
        );
        // self-dep on the moved lib IS rewritten.
        assert!(
            fwd.contains("deps = [\":iam\"]"),
            "self-dep rewritten: {fwd}"
        );

        // THE FIX: the inverse restores the bytes EXACTLY — in particular `crate = "x"` (snake),
        // NOT `crate = "x"` (the kebab clobber the un-fixed code produced).
        let inv = cm("iam/core/iam", "cloud/c/x", "iam", "x");
        let (round, _c) = rewrite_moved_buck(&fwd, &inv);
        assert_eq!(
            round, text,
            "single-token destination must round-trip byte-identically"
        );
    }

    /// MED-2 / LOW: a longer macro whose name merely ends in `rust_test` (e.g. `custom_rust_test(`)
    /// must NOT be treated as a `rust_test` stanza head, so its interior is not prefix-rewritten.
    #[test]
    fn substring_macro_head_is_not_a_rust_test_stanza() {
        let text = r#"rust_library(
    name = "x",
    crate = "x",
)

custom_rust_test(
    name = "x-foo",
    crate = "x_foo",
)
"#;
        let m = cm("cloud/c/x", "cap/core/new", "x", "new");
        let (out, _changed) = rewrite_moved_buck(text, &m);
        // Library renamed by the exact pass.
        assert!(out.contains("name = \"new\""), "{out}");
        // The custom_rust_test interior is NOT a rust_test stanza -> its name/crate are left as-is
        // (the prefixed pass never runs on it; the exact pass does not whole-token match either).
        assert!(
            out.contains("name = \"x-foo\""),
            "custom macro name not prefix-rewritten: {out}"
        );
        assert!(
            out.contains("crate = \"x_foo\""),
            "custom macro crate not prefix-rewritten: {out}"
        );
    }

    /// #63 (non-`//` sandbox source-path literals): a BUCK may reference a moved crate's sources
    /// through a repo-ROOTED (non-`//`) path STRING — `crate_root = "old_path/.../t.rs"` and
    /// `mapped_srcs` VALUES `"old_path/.../t.rs": "tests/t.rs"`. These are plain Starlark strings,
    /// not `//labels`, so the label rewriter left them pointing at a now-dead path after the move.
    /// The source-path-literal rewrite must repoint them (prefix only, in-crate suffix preserved),
    /// must NOT double-rewrite the `//old_path:target` label form, must leave package-relative
    /// `mapped_srcs` KEYS alone, and must round-trip byte-identically through the inverse.
    #[test]
    fn source_path_literals_in_crate_root_and_mapped_srcs_are_rewritten() {
        // This is the iac/facade/app/BUCK shape: a moved crate's tests are referenced by a repo-
        // rooted source path; the same path appears as a `//` label (already handled) and as bare
        // path literals (the #63 class). The moving crate dir is cloud/cloud-iac/crates/x.
        let text = r#"rust_test(
    name = "x-test",
    crate = "x_test",
    crate_root = "cloud/cloud-iac/crates/x/tests/t.rs",
    mapped_srcs = {
        "tests/t.rs": "cloud/cloud-iac/crates/x/tests/t.rs",
        "//cloud/cloud-iac/crates/x:x": "cloud/cloud-iac/crates/x/data.json",
    },
    deps = ["//cloud/cloud-iac/crates/x:x"],
)
"#;
        let m = cm(
            "cloud/cloud-iac/crates/x",
            "iac/facade/app",
            "x",
            "iac-app",
        );
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/cloud-iac/crates/x", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed);

        // crate_root repo-rooted literal repointed (prefix rewritten, suffix preserved).
        assert!(
            out.contains("crate_root = \"iac/facade/app/tests/t.rs\""),
            "crate_root source-path literal rewritten: {out}"
        );
        // mapped_srcs VALUE (repo-rooted source) repointed; the package-relative KEY untouched.
        assert!(
            out.contains("\"tests/t.rs\": \"iac/facade/app/tests/t.rs\""),
            "mapped_srcs value rewritten, key preserved: {out}"
        );
        assert!(
            out.contains("\"iac/facade/app/data.json\""),
            "exact-tail mapped_srcs value rewritten: {out}"
        );
        // The `//old:target` LABEL form is handled by the label passes (-> //new:target); the
        // source-path rewrite must NOT also touch it (no double rewrite / no stale fragment).
        assert!(
            out.contains("\"//iac/facade/app:iac-app\""),
            "self-target label rewritten by label pass: {out}"
        );
        assert!(
            out.contains("\"//iac/facade/app:x\""),
            "mapped_srcs key label rewritten by label pass: {out}"
        );
        // No stale old path anywhere.
        assert!(
            !out.contains("cloud/cloud-iac/crates/x"),
            "no stale old path: {out}"
        );

        // Inverse round-trips byte-identically.
        let inv = cm(
            "iac/facade/app",
            "cloud/cloud-iac/crates/x",
            "iac-app",
            "x",
        );
        let mut by_new: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_new.insert("iac/facade/app", &inv);
        let (round, _c) = rewrite_buck_labels(&out, &by_new);
        assert_eq!(
            round, text,
            "inverse must round-trip the source-path literals byte-identically"
        );
    }

    /// #63 boundary safety: a sibling dir whose name SHARES the moved crate's path as a non-`/`
    /// prefix (`.../x-extra/...`) must NOT be rewritten, and a moved-crate's OWN relative
    /// `crate_root = "src/lib.rs"` (package-relative, move-invariant) must stay byte-identical.
    #[test]
    fn source_path_literal_respects_dir_boundary_and_relative_roots() {
        let text = r#"rust_library(
    name = "rel",
    crate_root = "src/lib.rs",
    srcs = ["cloud/a/src/lib.rs", "cloud/a-extra/src/lib.rs"],
)
"#;
        let m = cm("cloud/a", "x/core/a", "a", "a-core");
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/a", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed, "the cloud/a source literal IS rewritten");
        // cloud/a -> x/core/a (descendant boundary on `/`).
        assert!(
            out.contains("\"x/core/a/src/lib.rs\""),
            "moved src rewritten: {out}"
        );
        // cloud/a-extra is a DIFFERENT dir (boundary char is `-`, not `/`) -> untouched.
        assert!(
            out.contains("\"cloud/a-extra/src/lib.rs\""),
            "sibling dir preserved: {out}"
        );
        // the relative crate_root is package-relative, never a repo-rooted moved-dir literal.
        assert!(
            out.contains("crate_root = \"src/lib.rs\""),
            "relative root invariant: {out}"
        );
    }

    /// #63 / #769 review (MED-1): the source-path-literal rewrite is deliberately FIELD-AGNOSTIC —
    /// any quoted VALUE that begins with a path UNDER the moved dir is a reference that moved, so it
    /// is rewritten regardless of the attribute that holds it (a `genrule`/`filegroup` `srcs` entry,
    /// a Starlark variable), not only `crate_root`/`mapped_srcs`. Narrowing to a fixed field set
    /// would STRAND these. A path NOT under the moved dir is left alone; the move round-trips
    /// byte-identically. This pins the broad-but-correct scope the docstrings now describe honestly.
    #[test]
    fn source_path_literal_is_field_agnostic_for_paths_under_the_moved_dir() {
        let text = r#"SEED = "cloud/cloud-iac/crates/x/data/seed.json"

genrule(
    name = "gen",
    srcs = [
        "cloud/cloud-iac/crates/x/data/seed.json",
        "cloud/other/keep.json",
    ],
    cmd = "$(location //tools/gen:gen) > $OUT",
)
"#;
        let m = cm(
            "cloud/cloud-iac/crates/x",
            "iac/adapters/gen",
            "x",
            "gen-adapter",
        );
        let mut by_old: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_old.insert("cloud/cloud-iac/crates/x", &m);
        let (out, changed) = rewrite_buck_labels(text, &by_old);
        assert!(changed);
        // A Starlark VARIABLE value under the moved dir is rewritten (NOT a crate_root/mapped_srcs).
        assert!(
            out.contains("SEED = \"iac/adapters/gen/data/seed.json\""),
            "Starlark variable path under moved dir rewritten: {out}"
        );
        // A `genrule` `srcs` entry under the moved dir is rewritten too.
        assert!(
            out.contains("\"iac/adapters/gen/data/seed.json\""),
            "genrule srcs path under moved dir rewritten: {out}"
        );
        // A path NOT under the moved dir is left untouched.
        assert!(
            out.contains("\"cloud/other/keep.json\""),
            "unrelated path preserved: {out}"
        );
        assert!(
            !out.contains("cloud/cloud-iac/crates/x"),
            "no stale old path: {out}"
        );

        // The move round-trips byte-identically through the inverse.
        let inv = cm(
            "iac/adapters/gen",
            "cloud/cloud-iac/crates/x",
            "gen-adapter",
            "x",
        );
        let mut by_new: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        by_new.insert("iac/adapters/gen", &inv);
        let (round, _c) = rewrite_buck_labels(&out, &by_new);
        assert_eq!(
            round, text,
            "field-agnostic rewrite round-trips byte-identically"
        );
    }

    /// MED-1: field-key anchoring. A `rust_test` with a quoted value matching the moving crate's
    /// prefix in a NON-`name`/`crate` field (`env`, a bare-quoted dep) must NOT be rewritten — only
    /// the `name` and `crate` fields are crate-name vocabulary.
    #[test]
    fn rust_test_non_name_crate_fields_are_not_rewritten() {
        let text = r#"rust_library(
    name = "x",
    crate = "x",
)

rust_test(
    name = "x-unittest",
    crate = "x",
    env = {"FIXTURE": "x-thing"},
    deps = ["//some/where:x-helper"],
)
"#;
        let m = cm("cloud/c/x", "cap/core/new", "x", "new");
        let (out, changed) = rewrite_moved_buck(text, &m);
        assert!(changed);
        // name/crate ARE rewritten.
        assert!(out.contains("name = \"new-unittest\""), "{out}");
        assert!(out.contains("crate = \"new\""), "{out}");
        // The env value and the bare-quoted dep entry are NOT rewritten by the prefix pass.
        assert!(
            out.contains("\"FIXTURE\": \"x-thing\""),
            "env value untouched: {out}"
        );
        assert!(
            out.contains("\"//some/where:x-helper\""),
            "dep entry untouched by prefix pass: {out}"
        );
    }
}
