//! # cloud-ci-embedded-asset-hermeticity (ADR-0545)
//!
//! The embedded-asset hermeticity gate. Motivated by FRIC-1781131000: a crate's
//! `include_str!("../policy/x.cedar")` was mapped to the wrong sandbox path, so buck2 never built it
//! hermetically (rustc `couldn't read`, masked downstream as a `missing rmeta`). This gate statically
//! asserts that every Rust `include_str!`/`include_bytes!` STRING-LITERAL path resolves, relative to
//! the including source file's sandbox location, to a destination the owning BUCK target's
//! `srcs`/`mapped_srcs` actually provides — catching unmapped embedded assets before the build.
//!
//! It reimplements Bazel/Buck hermetic-action missing-input enforcement Rust-native (founder
//! doctrine: proven patterns, Rust reimplementation). Divergence from a full buck2 `aquery`: a
//! conservative STATIC parse so the gate runs in presubmit without the cold build it pre-empts.
//!
//! ## Resolution model (the kernel contract — load-bearing)
//! `include_str!`/`include_bytes!` resolve the literal RELATIVE TO THE DIRECTORY OF THE INCLUDING
//! SOURCE FILE. In the sandbox that file sits at the destination its target maps it to. So the gate
//! computes `include_base = dirname(sandbox-destination of the including .rs)`, resolves the literal
//! against it (lexical `..`/`.` normalization, NO filesystem touch), and asserts the resolved path is
//! present in the owning target's EFFECTIVE sandbox destination set =
//! (glob-expanded `srcs` relative-path destinations) ∪ (`mapped_srcs` dict VALUES). The check is
//! against sandbox DESTINATIONS, never the on-disk repo layout: the cedar-adapter file lives on disk
//! OUTSIDE its crate and is mapped INTO the sandbox at the include-relative location via a
//! `mapped_srcs` value.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — scan roots, exclude substrings, the rust +
//! embedded extension sets, the out-of-scope build-output prefixes — are DATA in
//! `embedded-asset-hermeticity-policy.json`. A different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_observed`] `(root, policy) -> {sites:[..]}` walks the policy roots, and for each Rust
//!   include site binds the owning BUCK target, builds its sandbox destination set, resolves the
//!   include literal, and records the outcome. This is the ONLY I/O (read-only, no temp files).
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it turns observed site statuses into findings.
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Fail-closed-conservative
//! A hard violation (`embedded_asset_unmapped_include`) is emitted ONLY for a site fully resolved:
//! a single string literal whose owning target `srcs`/`mapped_srcs` the minimal parser evaluates.
//! Anything not statically resolvable is surfaced as a SKIP key (`embedded_asset_unparseable_skip` /
//! `embedded_asset_no_target`), never a silent pass — skips are baselined + capped so skip-debt
//! cannot grow to launder a real miss.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `embedded_asset_policy_gate_id_mismatch` — policy `gate_id` != [`GATE_ID`].
//! - `embedded_asset_unmapped_include`        — a resolved include-relative sandbox path is NOT among
//!                                              the owning target's `srcs`/`mapped_srcs` destinations.
//! - `embedded_asset_no_target`               — a `.rs` with an include site has no bindable BUCK target.
//! - `embedded_asset_unparseable_skip`        — the include literal or the owning target's
//!                                              `srcs`/`mapped_srcs` is not statically resolvable.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the oya-ci registry id.
pub const GATE_ID: &str = "cloud-ci-embedded-asset-hermeticity";

/// The four violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 4] = [
    "embedded_asset_policy_gate_id_mismatch",
    "embedded_asset_unmapped_include",
    "embedded_asset_no_target",
    "embedded_asset_unparseable_skip",
];

/// The sentinel key for codes that are policy-level rather than per-site.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors collecting the observed include sites. The kernel returns these instead of panicking so
/// the caller (CI / a controller) decides how to surface them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// `scan_roots` policy field is missing or not a non-empty array of strings.
    MissingScanRoots,
    /// A filesystem read failed.
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::MissingScanRoots => {
                write!(f, "policy `scan_roots` must be a non-empty array of strings")
            }
            CollectError::Io(message) => write!(f, "embedded-asset scan io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

// ---------------------------------------------------------------------------
// Verdict / Finding / Report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

// ---------------------------------------------------------------------------
// Path normalization (lexical, no filesystem touch)
// ---------------------------------------------------------------------------

/// Lexically normalize a forward-slash path, resolving `.` and `..` without touching the filesystem.
/// A leading `..` that escapes the root is preserved (it signals an out-of-tree target). Returns the
/// normalized path with `/` separators and no trailing slash. Pure; deterministic.
pub fn normalize_rel(path: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for raw in path.split('/') {
        match raw {
            "" | "." => {}
            ".." => {
                if matches!(out.last().map(String::as_str), Some("..")) || out.is_empty() {
                    out.push("..".to_owned());
                } else {
                    out.pop();
                }
            }
            other => out.push(other.to_owned()),
        }
    }
    out.join("/")
}

/// Join an include literal onto the directory of the including file's sandbox destination, then
/// normalize. `site_dir` is already a sandbox-relative dir (e.g. `crate/src`); `literal` is the raw
/// include argument (e.g. `../policy/x.cedar`). Pure.
fn resolve_include(site_dir: &str, literal: &str) -> String {
    let joined = if site_dir.is_empty() {
        literal.to_owned()
    } else {
        format!("{site_dir}/{literal}")
    };
    normalize_rel(&joined)
}

// ---------------------------------------------------------------------------
// Pure evaluator
// ---------------------------------------------------------------------------

/// Pure evaluator. `policy` is DATA (`embedded-asset-hermeticity-policy.json`); `observed` is the
/// collected sites shaped as `{ "sites": [ <site>, .. ] }`. Each site carries a `status` of
/// `resolved` | `unmapped` | `no_target` | `unparseable` plus a `key` and `detail`.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if non_blank_str(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "embedded_asset_policy_gate_id_mismatch",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let sites = observed
        .get("sites")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for site in &sites {
        let status = site.get("status").and_then(Value::as_str).unwrap_or("");
        let key = site.get("key").and_then(Value::as_str).unwrap_or("");
        let detail = site
            .get("detail")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if key.is_empty() {
            continue;
        }
        match status {
            "resolved" => {}
            "unmapped" => {
                findings.insert(Finding::new("embedded_asset_unmapped_include", key, detail));
            }
            "no_target" => {
                findings.insert(Finding::new("embedded_asset_no_target", key, detail));
            }
            "unparseable" => {
                findings.insert(Finding::new("embedded_asset_unparseable_skip", key, detail));
            }
            // out_of_scope and any unknown status contribute no finding.
            _ => {}
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

// ---------------------------------------------------------------------------
// Policy accessors
// ---------------------------------------------------------------------------

fn non_blank_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn str_array(policy: &Value, field: &str) -> Vec<String> {
    policy
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Include-site extraction (pure over a Rust source's text)
// ---------------------------------------------------------------------------

/// One include site found in a Rust source: the macro name and the raw literal (if the argument is a
/// single string literal) or `None` (non-literal, e.g. concat!/env!/a macro), with a 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeSite {
    pub macro_name: String,
    pub literal: Option<String>,
    pub line: usize,
}

/// Extract every `include_str!`/`include_bytes!` site from a Rust source text. A site whose single
/// argument is a string literal yields `literal: Some(..)`; any other argument shape yields
/// `literal: None` (the gate classifies these as unparseable skips). Pure; no filesystem.
///
/// This is a deliberately minimal text scanner (not a full Rust parser): it finds the macro
/// invocation, skips whitespace, and reads a `"..."` argument if and only if the first token after
/// `(` is a string literal whose closing `)` immediately follows the closing quote (optionally with
/// a trailing comma / whitespace). Raw strings (`r"..."`, `r#"..."#`) and any expression argument are
/// reported as non-literal (skip) rather than mis-parsed.
pub fn extract_include_sites(source: &str) -> Vec<IncludeSite> {
    const MACROS: [&str; 2] = ["include_str!", "include_bytes!"];
    let mut sites = Vec::new();
    let bytes = source.as_bytes();
    for macro_name in MACROS {
        let mut from = 0usize;
        while let Some(rel) = source[from..].find(macro_name) {
            let start = from + rel;
            from = start + macro_name.len();
            // Reject identifiers that merely END with the macro text (e.g. `my_include_str!`): the
            // char before the match must not be an identifier char.
            if start > 0 {
                let prev = bytes[start - 1];
                if prev == b'_' || prev.is_ascii_alphanumeric() {
                    continue;
                }
            }
            let line = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
            let after = &source[from..];
            let literal = parse_macro_string_arg(after);
            sites.push(IncludeSite {
                macro_name: macro_name.trim_end_matches('!').to_owned(),
                literal,
                line,
            });
        }
    }
    sites.sort_by(|a, b| a.line.cmp(&b.line).then(a.macro_name.cmp(&b.macro_name)));
    sites
}

/// Given the text immediately after a macro name (starting at the `(`), return the single string
/// literal argument if the invocation is exactly `("...")` (trailing comma / whitespace allowed), or
/// `None` for any other argument shape (raw string, concat!, env!, an expression). Pure.
fn parse_macro_string_arg(after: &str) -> Option<String> {
    let mut chars = after.char_indices().peekable();
    // Expect an opening paren first (allow whitespace before it).
    loop {
        let (_, ch) = *chars.peek()?;
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '(' {
            chars.next();
            break;
        }
        return None;
    }
    // Skip whitespace before the argument.
    loop {
        let (_, ch) = *chars.peek()?;
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        break;
    }
    // The argument must START with a plain double-quote (not `r`, not an identifier/expression).
    let (quote_idx, quote_ch) = *chars.peek()?;
    if quote_ch != '"' {
        return None;
    }
    // Read the string literal honoring escapes.
    let rest = &after[quote_idx + 1..];
    let mut value = String::new();
    let mut escaped = false;
    let mut closed_at: Option<usize> = None;
    for (idx, ch) in rest.char_indices() {
        if escaped {
            // Preserve only the common escapes we expect in paths; unknown escapes pass through.
            match ch {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'r' => value.push('\r'),
                '\\' => value.push('\\'),
                '"' => value.push('"'),
                '0' => value.push('\0'),
                other => value.push(other),
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            closed_at = Some(idx);
            break;
        }
        value.push(ch);
    }
    let closed_at = closed_at?;
    // After the closing quote, only whitespace, an optional trailing comma, then `)` is allowed.
    let tail = &rest[closed_at + 1..];
    let mut seen_comma = false;
    for ch in tail.chars() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == ',' && !seen_comma {
            seen_comma = true;
            continue;
        }
        if ch == ')' {
            return Some(value);
        }
        // Anything else (a second argument, a method call, concatenation) -> not a simple literal.
        return None;
    }
    None
}

// ---------------------------------------------------------------------------
// Minimal BUCK parse: per-target crate_root + effective sandbox destination set
// ---------------------------------------------------------------------------

/// A parsed BUCK target relevant to embedded-asset resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuckTarget {
    pub name: String,
    pub kind: String,
    /// The crate-relative crate_root path (e.g. `src/lib.rs` or `tests/foo.rs`), if resolvable.
    pub crate_root: Option<String>,
    /// Explicit `srcs` relative paths (a `["a.rs", ..]` list), if the srcs were an explicit list.
    pub explicit_srcs: Vec<String>,
    /// Glob patterns from `srcs = glob([..])` (crate-relative), if srcs used a glob.
    pub srcs_globs: Vec<String>,
    /// `mapped_srcs` dict VALUES (sandbox destinations), explicit or comprehension-derived.
    pub mapped_dest: Vec<String>,
    /// True if `srcs`/`mapped_srcs` used a construct the minimal parser could not fully model.
    pub unparseable: bool,
}

/// Parse the BUCK text of a crate dir into the targets the gate cares about. Pure over text +
/// (for glob expansion) the on-disk crate tree, which the caller passes via `crate_files`
/// (crate-relative paths). Best-effort + conservative: a target whose srcs/mapped_srcs cannot be
/// modelled is marked `unparseable` rather than silently treated as empty.
pub fn parse_buck_targets(buck_text: &str, crate_files: &[String]) -> Vec<BuckTarget> {
    const KINDS: [&str; 3] = ["rust_library", "rust_binary", "rust_test"];
    // Resolve top-level `IDENT = "string"` and `IDENT = glob([...])` variable assignments so
    // crate_root / srcs / mapped_srcs can reference them (the cedar-adapter ADAPTER_ROOT/ADAPTER_SRCS
    // shape).
    let string_vars = top_level_string_vars(buck_text);
    let glob_vars = top_level_glob_vars(buck_text);

    let mut targets = Vec::new();
    for kind in KINDS {
        let mut search_from = 0usize;
        while let Some(rel) = buck_text[search_from..].find(&format!("{kind}(")) {
            let start = search_from + rel;
            // Reject `IDENT_rust_test(` style false hits: the char before must not be ident.
            if start > 0 {
                let prev = buck_text.as_bytes()[start - 1];
                if prev == b'_' || prev.is_ascii_alphanumeric() {
                    search_from = start + kind.len();
                    continue;
                }
            }
            let Some(block) = call_block(&buck_text[start..]) else {
                search_from = start + kind.len();
                continue;
            };
            search_from = start + block.len();
            let name = field_value_expr(&block, "name")
                .and_then(|e| unquote_concat(&e, &string_vars))
                .unwrap_or_default();

            let crate_root = field_value_expr(&block, "crate_root")
                .and_then(|e| unquote_concat(&e, &string_vars))
                .map(|p| crate_relative(&p));

            let (explicit_srcs, srcs_globs, srcs_unparseable) =
                parse_srcs(&block, &glob_vars);
            let (mapped_dest, mapped_unparseable) =
                parse_mapped_srcs(&block, &string_vars, &glob_vars, crate_files);

            targets.push(BuckTarget {
                name,
                kind: kind.to_owned(),
                crate_root,
                explicit_srcs,
                srcs_globs,
                mapped_dest,
                unparseable: srcs_unparseable || mapped_unparseable,
            });
        }
    }
    targets
}

/// The effective sandbox DESTINATION set for a target: glob-expanded `srcs` (crate-relative) plus
/// explicit srcs plus mapped_srcs values. `crate_files` is the crate's on-disk file list
/// (crate-relative, `/`-separated). Returns crate-relative destinations for in-crate srcs and the
/// raw mapped values (which are already sandbox-rooted, e.g. `cloud/.../policy/x.cedar`). Pure.
pub fn target_destinations(target: &BuckTarget, crate_dir: &str, crate_files: &[String]) -> BTreeSet<String> {
    let mut dests = BTreeSet::new();
    // crate_root itself is always a destination.
    if let Some(root) = &target.crate_root {
        dests.insert(join_crate(crate_dir, root));
    }
    for src in &target.explicit_srcs {
        dests.insert(join_crate(crate_dir, src));
    }
    for pattern in &target.srcs_globs {
        for file in crate_files {
            if glob_match(pattern, file) {
                dests.insert(join_crate(crate_dir, file));
            }
        }
    }
    for value in &target.mapped_dest {
        // mapped_srcs VALUES are already sandbox-rooted (repo-relative), not crate-relative.
        dests.insert(normalize_rel(value));
    }
    dests
}

/// Join a crate dir and a crate-relative path into a sandbox-rooted, normalized path.
fn join_crate(crate_dir: &str, rel: &str) -> String {
    if crate_dir.is_empty() {
        normalize_rel(rel)
    } else {
        normalize_rel(&format!("{crate_dir}/{rel}"))
    }
}

/// Strip a leading `ROOT + "/"` style crate-prefix is NOT done here; this just normalizes a path that
/// is already crate-relative (drops a leading `./`).
fn crate_relative(path: &str) -> String {
    normalize_rel(path)
}

// ---------------------------------------------------------------------------
// BUCK text helpers (balanced, string-aware — mirrors oya-buck-test-wiring-app)
// ---------------------------------------------------------------------------

/// Return the full `name( … )` call block starting at `text[0]`, balancing parens and ignoring
/// string contents. `text` must start at the call's opening identifier.
fn call_block(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in text[open..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let end = open + offset + ch.len_utf8();
                    return Some(text[..end].to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract the raw expression assigned to `field =` inside a call block (balanced, string-aware).
/// Returns the trimmed expression text up to the top-level comma that ends the argument.
fn field_value_expr(block: &str, field: &str) -> Option<String> {
    let key = format!("{field} =");
    let key_pos = find_top_level(block, &key)?;
    let rest = &block[key_pos + key.len()..];
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = rest.len();
    for (offset, ch) in rest.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' | '(' | '{' => depth += 1,
            ']' | ')' | '}' => {
                if depth == 0 {
                    end = offset;
                    break;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                end = offset;
                break;
            }
            _ => {}
        }
    }
    Some(rest[..end].trim().to_owned())
}

/// Find `needle` at a position that is NOT inside a string literal or nested bracket of `block`.
/// Used to locate `field =` only at the call's top argument level.
fn find_top_level(block: &str, needle: &str) -> Option<usize> {
    let bytes = block.as_bytes();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0usize;
    // Start after the first `(` so we are inside the argument list.
    let open = block.find('(')? + 1;
    i = open;
    while i < block.len() {
        let ch = bytes[i] as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {
                // The top argument level sits at depth 0 here: we began scanning just AFTER the
                // call's opening `(`, so that paren was never counted. Nested `glob([...])` etc.
                // raise depth above 0, excluding their inner `=`/keys from matching.
                if depth == 0 && block[i..].starts_with(needle) {
                    // Ensure the char before is not an identifier char (so `srcs =` doesn't match
                    // `mapped_srcs =`).
                    let prev_ok = i == 0 || {
                        let p = bytes[i - 1];
                        !(p == b'_' || (p as char).is_ascii_alphanumeric())
                    };
                    if prev_ok {
                        return Some(i);
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Collect every double-quoted string in an expression (escape-aware).
fn quoted_strings(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start: Option<usize> = None;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if let Some(value_start) = start {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                values.push(text[value_start..index].to_owned());
                start = None;
            }
        } else if ch == '"' {
            start = Some(index + ch.len_utf8());
        }
    }
    values
}

/// Top-level `IDENT = "string"` assignments (single-line). Used to resolve concat operands like
/// `ADAPTER_ROOT`.
fn top_level_string_vars(buck_text: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for line in buck_text.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        let Some(eq) = t.find('=') else { continue };
        let (lhs, rhs) = (t[..eq].trim(), t[eq + 1..].trim());
        if lhs.is_empty() || !is_ident(lhs) {
            continue;
        }
        let strings = quoted_strings(rhs);
        // Only treat as a string var if the RHS is exactly one quoted string (no operators).
        if strings.len() == 1 && rhs.starts_with('"') && rhs.trim_end_matches(',').ends_with('"') {
            vars.push((lhs.to_owned(), strings[0].clone()));
        }
    }
    vars
}

/// Top-level `IDENT = glob([...])` assignments → the glob patterns. Used to resolve a srcs/mapped
/// comprehension `for src in SRCS`.
fn top_level_glob_vars(buck_text: &str) -> Vec<(String, Vec<String>)> {
    let mut vars = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = buck_text[search_from..].find('=') {
        let eq = search_from + rel;
        // Identify the LHS ident on this line.
        let line_start = buck_text[..eq].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let lhs = buck_text[line_start..eq].trim();
        let after = buck_text[eq + 1..].trim_start();
        if is_ident(lhs) && after.starts_with("glob(") {
            if let Some(block) = call_block(&buck_text[eq + 1 + (buck_text[eq + 1..].len() - after.len())..]) {
                vars.push((lhs.to_owned(), quoted_strings(&block)));
                search_from = eq + 1;
                continue;
            }
        }
        search_from = eq + 1;
    }
    vars
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c == '_' || c.is_ascii_uppercase() || c.is_ascii_digit())
        && s.chars().next().map(|c| c != '0' && !c.is_ascii_digit()).unwrap_or(false)
        || (!s.is_empty()
            && s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
            && s.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false))
}

/// Resolve a `crate_root`/`name` expression that is a quoted string or a `VAR + "/suffix"` concat
/// against the known string vars. Returns the concatenated string, or `None` if any operand is
/// unresolved.
fn unquote_concat(expr: &str, string_vars: &[(String, String)]) -> Option<String> {
    let mut out = String::new();
    for operand in expr.split('+') {
        let op = operand.trim();
        if op.starts_with('"') {
            // a quoted literal
            let strings = quoted_strings(op);
            out.push_str(strings.first()?);
        } else if is_ident(op) {
            let val = string_vars.iter().find(|(k, _)| k == op).map(|(_, v)| v)?;
            out.push_str(val);
        } else {
            return None;
        }
    }
    Some(out)
}

/// Parse a target's `srcs` field. Returns (explicit_srcs, glob_patterns, unparseable).
fn parse_srcs(block: &str, glob_vars: &[(String, Vec<String>)]) -> (Vec<String>, Vec<String>, bool) {
    let Some(expr) = field_value_expr(block, "srcs") else {
        // No srcs field (e.g. a target driven purely by crate_root + mapped_srcs): not unparseable;
        // crate_root alone still yields a destination.
        return (Vec::new(), Vec::new(), false);
    };
    let trimmed = expr.trim();
    if trimmed == "[]" {
        return (Vec::new(), Vec::new(), false);
    }
    if trimmed.starts_with("glob(") {
        return (Vec::new(), quoted_strings(trimmed), false);
    }
    if trimmed.starts_with('[') {
        // Explicit list of quoted paths.
        return (quoted_strings(trimmed), Vec::new(), false);
    }
    if is_ident(trimmed) {
        // A bare var reference, e.g. `srcs = ADAPTER_SRCS` (a glob var).
        if let Some((_, patterns)) = glob_vars.iter().find(|(k, _)| k == trimmed) {
            return (Vec::new(), patterns.clone(), false);
        }
        return (Vec::new(), Vec::new(), true);
    }
    // An unmodelled srcs construct.
    (Vec::new(), Vec::new(), true)
}

/// Parse a target's `mapped_srcs` field into its destination VALUES. Handles: a bare var reference to
/// a comprehension/dict assembled at top level is NOT supported here (we parse the in-block dict and
/// the cedar-adapter comprehension form). Returns (values, unparseable).
fn parse_mapped_srcs(
    block: &str,
    string_vars: &[(String, String)],
    glob_vars: &[(String, Vec<String>)],
    crate_files: &[String],
) -> (Vec<String>, bool) {
    let Some(expr) = field_value_expr(block, "mapped_srcs") else {
        return (Vec::new(), false);
    };
    let trimmed = expr.trim();
    // The common oyatie shape is `mapped_srcs = SOME_VAR`, where SOME_VAR is assembled at top level
    // as a comprehension plus explicit `VAR["k"]=v` assignments. We resolve that var from the WHOLE
    // BUCK text via resolve_mapped_var (the caller passes the block; the var assembly lives at file
    // scope, so we cannot see it here). Treat a bare-var mapped_srcs as a signal handled by the
    // file-level resolver: report unparseable here and let the file-level pass fill it.
    if is_ident(trimmed) {
        // Resolved at the file level (see resolve_mapped_var); mark unparseable=false but empty —
        // the file-level resolver augments. To keep this function pure-on-block, we return empty +
        // false; the collector calls resolve_mapped_var with full text for the bare-var case.
        return (Vec::new(), false);
    }
    if trimmed.starts_with('{') {
        return (mapped_dict_values(trimmed, string_vars, glob_vars, crate_files), false);
    }
    (Vec::new(), true)
}

/// Resolve the destination VALUES of a top-level mapped_srcs variable assembled as
/// `VAR = {src: ROOT + "/" + src for src in SRCS}` plus `VAR["k"] = ROOT + "/path"` assignments.
/// `var` is the variable name referenced by a target's `mapped_srcs = VAR`. Pure over the BUCK text.
pub fn resolve_mapped_var(
    buck_text: &str,
    var: &str,
    string_vars: &[(String, String)],
    glob_vars: &[(String, Vec<String>)],
    crate_files: &[String],
) -> Vec<String> {
    let mut values = Vec::new();
    // 1. The comprehension assignment `VAR = { ... for src in SRCS }`.
    if let Some(eq) = find_var_assignment(buck_text, var) {
        let after = buck_text[eq..].trim_start();
        if after.starts_with('{') {
            if let Some(dict) = brace_block(after) {
                values.extend(mapped_dict_values(&dict, string_vars, glob_vars, crate_files));
            }
        }
    }
    // 2. Explicit `VAR["k"] = expr` value assignments.
    let needle = format!("{var}[");
    let mut from = 0usize;
    while let Some(rel) = buck_text[from..].find(&needle) {
        let idx = from + rel;
        from = idx + needle.len();
        // Find the `] = <expr>` after the key.
        let after_key = &buck_text[idx + needle.len()..];
        let Some(close) = after_key.find(']') else { continue };
        let after_bracket = after_key[close + 1..].trim_start();
        let Some(rest) = after_bracket.strip_prefix('=') else { continue };
        // Take the RHS to end of line.
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let rhs = rest[..line_end].trim().trim_end_matches(',').trim();
        if let Some(value) = unquote_concat(rhs, string_vars) {
            values.push(value);
        }
    }
    values
}

/// Evaluate a `{ ... }` mapped_srcs dict literal or comprehension into destination VALUES.
fn mapped_dict_values(
    dict: &str,
    string_vars: &[(String, String)],
    glob_vars: &[(String, Vec<String>)],
    crate_files: &[String],
) -> Vec<String> {
    let inner = dict.trim();
    let inner = inner
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(inner)
        .trim();
    // Comprehension form: `KEY_EXPR: VALUE_EXPR for src in SRCS`.
    if let Some(for_pos) = find_top_level_keyword(inner, " for ") {
        let head = inner[..for_pos].trim();
        let tail = inner[for_pos + " for ".len()..].trim();
        // tail: `src in SRCS`
        let parts: Vec<&str> = tail.splitn(3, ' ').collect();
        if parts.len() == 3 && parts[1] == "in" {
            let loop_var = parts[0].trim();
            let iter = parts[2].trim().trim_end_matches(',').trim();
            // Resolve the value expression (after the `:`) for each src.
            if let Some(colon) = find_top_level_keyword(head, ":") {
                let value_expr = head[colon + 1..].trim();
                let srcs = if let Some((_, patterns)) = glob_vars.iter().find(|(k, _)| k == iter) {
                    expand_globs(patterns, crate_files)
                } else {
                    Vec::new()
                };
                let mut out = Vec::new();
                for src in &srcs {
                    if let Some(v) = eval_value_expr(value_expr, loop_var, src, string_vars) {
                        out.push(v);
                    }
                }
                return out;
            }
        }
        return Vec::new();
    }
    // Explicit dict literal `{ "k": "v", ... }`: VALUES are every SECOND quoted string. Be robust:
    // split on top-level commas, take the substring after the top-level `:` of each entry.
    let mut out = Vec::new();
    for entry in split_top_level(inner, ',') {
        if let Some(colon) = find_top_level_keyword(&entry, ":") {
            let value_expr = entry[colon + 1..].trim();
            if let Some(v) = unquote_concat(value_expr, string_vars) {
                out.push(v);
            } else {
                let q = quoted_strings(value_expr);
                if let Some(first) = q.first() {
                    out.push(first.clone());
                }
            }
        }
    }
    out
}

/// Evaluate a comprehension value expression like `ROOT + "/" + src` for a concrete `src`.
fn eval_value_expr(
    expr: &str,
    loop_var: &str,
    src: &str,
    string_vars: &[(String, String)],
) -> Option<String> {
    let mut out = String::new();
    for operand in expr.split('+') {
        let op = operand.trim();
        if op == loop_var {
            out.push_str(src);
        } else if op.starts_with('"') {
            out.push_str(quoted_strings(op).first()?);
        } else if is_ident(op) {
            out.push_str(string_vars.iter().find(|(k, _)| k == op).map(|(_, v)| v)?);
        } else {
            return None;
        }
    }
    Some(out)
}

/// Expand glob patterns against a crate's file list (crate-relative paths).
fn expand_globs(patterns: &[String], crate_files: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for file in crate_files {
        if patterns.iter().any(|p| glob_match(p, file)) {
            out.push(file.clone());
        }
    }
    out
}

/// Find a top-level keyword/substring in `s` (not inside quotes or brackets). Returns byte index.
fn find_top_level_keyword(s: &str, kw: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0usize;
    while i < s.len() {
        let ch = bytes[i] as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {
                if depth == 0 && s[i..].starts_with(kw) {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

/// Split a string on a delimiter at the top level (not inside quotes/brackets).
fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0usize;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            c if c == delim && depth == 0 => {
                parts.push(s[start..i].trim().to_owned());
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }
    let last = s[start..].trim();
    if !last.is_empty() {
        parts.push(last.to_owned());
    }
    parts
}

/// Return the index just after `VAR =` for a top-level assignment, or None.
fn find_var_assignment(buck_text: &str, var: &str) -> Option<usize> {
    for (line_start, line) in line_offsets(buck_text) {
        let t = line.trim_start();
        if t.starts_with('#') {
            continue;
        }
        if let Some(rest) = t.strip_prefix(var) {
            let rest = rest.trim_start();
            if let Some(after_eq) = rest.strip_prefix('=') {
                // Ensure it's `VAR =` not `VAR[...] =` or `VAR_X =`.
                let after_eq_offset = line.len() - after_eq.len();
                return Some(line_start + after_eq_offset);
            }
        }
    }
    None
}

fn line_offsets(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        out.push((offset, line));
        offset += line.len();
    }
    out
}

/// Return a `{ ... }` brace block starting at `text[0] == '{'`, balanced + string-aware.
fn brace_block(text: &str) -> Option<String> {
    if !text.starts_with('{') {
        return None;
    }
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[..offset + ch.len_utf8()].to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

/// The bare-variable name a target's `mapped_srcs` references, if any (`mapped_srcs = VAR`).
fn mapped_srcs_var(block: &str) -> Option<String> {
    let expr = field_value_expr(block, "mapped_srcs")?;
    let t = expr.trim();
    if is_ident(t) { Some(t.to_owned()) } else { None }
}

// ---------------------------------------------------------------------------
// Minimal glob matcher (supports the buck2 `**` and `*` used in srcs globs)
// ---------------------------------------------------------------------------

/// Match a buck2-style glob (`**` = any number of path segments incl. zero; `*` = any chars except
/// `/`) against a crate-relative path. Segment-recursive so `src/**/*.rs` matches BOTH `src/lib.rs`
/// (zero intermediate segments) and `src/a/b.rs`. Pure; no filesystem. `?` and char classes are not
/// used by repo srcs globs.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let txt: Vec<&str> = path.split('/').collect();
    glob_segments(&pat, &txt)
}

/// Match path segments against pattern segments, where a `**` pattern segment matches zero or more
/// path segments.
fn glob_segments(pat: &[&str], txt: &[&str]) -> bool {
    if pat.is_empty() {
        return txt.is_empty();
    }
    if pat[0] == "**" {
        // `**` matches zero segments (advance the pattern) or one+ segments (consume a path segment).
        if glob_segments(&pat[1..], txt) {
            return true;
        }
        if !txt.is_empty() {
            return glob_segments(pat, &txt[1..]);
        }
        return false;
    }
    if txt.is_empty() {
        return false;
    }
    if !segment_match(pat[0].as_bytes(), txt[0].as_bytes()) {
        return false;
    }
    glob_segments(&pat[1..], &txt[1..])
}

/// Match a single path segment against a single pattern segment, where `*` matches any run of
/// characters within the segment (never crossing `/`, which cannot appear in a segment anyway).
fn segment_match(pat: &[u8], txt: &[u8]) -> bool {
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star_p, mut star_t): (Option<usize>, usize) = (None, 0);
    while t < txt.len() {
        if p < pat.len() && pat[p] == b'*' {
            star_p = Some(p);
            star_t = t;
            p += 1;
        } else if p < pat.len() && pat[p] == txt[t] {
            p += 1;
            t += 1;
        } else if let Some(sp) = star_p {
            p = sp + 1;
            star_t += 1;
            t = star_t;
        } else {
            return false;
        }
    }
    while p < pat.len() && pat[p] == b'*' {
        p += 1;
    }
    p == pat.len()
}

// ---------------------------------------------------------------------------
// Collection (the only I/O — read-only)
// ---------------------------------------------------------------------------

/// Collect every Rust include site under the policy `scan_roots`, bind each to its owning BUCK
/// target, resolve the include literal against that target's sandbox destination set, and record the
/// outcome. Read-only; writes no temp files. Output shape:
/// `{ "sites": [ { "key":.., "rs":.., "macro":.., "literal":.., "resolved":.., "target":..,
///   "status":"resolved"|"unmapped"|"no_target"|"unparseable"|"out_of_scope", "detail":.. } ] }`.
pub fn collect_observed(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let scan_roots = str_array(policy, "scan_roots");
    if scan_roots.is_empty() {
        return Err(CollectError::MissingScanRoots);
    }
    let excludes = str_array(policy, "exclude_path_substrings");
    let rust_ext = policy
        .get("rust_extension")
        .and_then(Value::as_str)
        .unwrap_or("rs")
        .to_owned();
    let embedded_exts: BTreeSet<String> = str_array(policy, "embedded_extensions")
        .into_iter()
        .map(|e| e.to_ascii_lowercase())
        .collect();
    let oos_prefixes = str_array(policy, "out_of_scope_path_prefixes");

    // 1. Enumerate Rust source files under the scan roots.
    let mut rust_files: Vec<PathBuf> = Vec::new();
    for scan in &scan_roots {
        let base = root.join(scan);
        if !base.exists() {
            continue;
        }
        walk_rust(&base, &rust_ext, &excludes, &mut rust_files)?;
    }
    rust_files.sort();

    let mut sites_out: Vec<Value> = Vec::new();

    for rs_abs in &rust_files {
        let rs_rel = rel_to(root, rs_abs);
        let source = match fs::read_to_string(rs_abs) {
            Ok(s) => s,
            Err(e) => return Err(CollectError::Io(format!("read {}: {e}", rs_abs.display()))),
        };
        let sites = extract_include_sites(&source);
        if sites.is_empty() {
            continue;
        }

        // Bind the crate dir (nearest ancestor with a BUCK file) once per file.
        let crate_dir_abs = nearest_buck_dir(root, rs_abs);
        let Some(crate_dir_abs) = crate_dir_abs else {
            for site in &sites {
                let key = format!("{rs_rel}:{}", site.line);
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name,
                    "literal": site.literal, "status": "no_target",
                    "detail": "no ancestor BUCK file found for this Rust source"
                }));
            }
            continue;
        };
        let crate_dir_rel = rel_to(root, &crate_dir_abs);
        let buck_text = match fs::read_to_string(crate_dir_abs.join("BUCK")) {
            Ok(t) => t,
            Err(e) => {
                return Err(CollectError::Io(format!(
                    "read {}: {e}",
                    crate_dir_abs.join("BUCK").display()
                )));
            }
        };
        let crate_files = list_crate_files(&crate_dir_abs, &crate_dir_rel, root)?;
        let string_vars = top_level_string_vars(&buck_text);
        let glob_vars = top_level_glob_vars(&buck_text);
        let targets = parse_buck_targets(&buck_text, &crate_files);

        // The including file's crate-relative path.
        let rs_in_crate = strip_prefix_slash(&rs_rel, &crate_dir_rel);

        // Pick the owning target: the one whose crate_root == rs_in_crate, else a target whose
        // srcs/globs cover it; prefer rust_library/rust_binary for src files, rust_test for the
        // exact test file.
        let owning = pick_owning_target(&targets, &rs_in_crate, &crate_dir_rel, &crate_files);

        for site in &sites {
            let key = format!("{rs_rel}:{}", site.line);
            let Some(literal) = &site.literal else {
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": Value::Null,
                    "status": "unparseable",
                    "detail": "include argument is not a single string literal (concat!/env!/raw string/expression)"
                }));
                continue;
            };

            // Out-of-scope build-output paths: a literal whose normalized form starts with an
            // out-of-scope prefix and whose extension is NOT an embedded asset extension is a
            // generated build artifact, not a tracked source -> no finding.
            if is_out_of_scope(literal, &oos_prefixes, &embedded_exts) {
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "status": "out_of_scope",
                    "detail": "literal resolves into an out-of-scope build-output tree (not a tracked embedded asset)"
                }));
                continue;
            }

            let Some(target) = &owning else {
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "status": "no_target",
                    "detail": format!("no BUCK target in {crate_dir_rel}/BUCK owns {rs_in_crate}")
                }));
                continue;
            };

            // The including file's sandbox destination = its crate_root dest, OR its crate-relative
            // path under the crate (glob default). For include resolution we need the dir of the
            // INCLUDING file's sandbox location, which for in-crate sources is
            // `<crate_dir>/<rs_in_crate>`.
            let including_dest = join_crate(&crate_dir_rel, &rs_in_crate);
            let site_dir = parent_dir(&including_dest);
            let resolved = resolve_include(&site_dir, literal);

            // Build the destination set, augmenting bare-var mapped_srcs from the file scope.
            let mut dests = target_destinations(target, &crate_dir_rel, &crate_files);
            if let Some(var) = mapped_srcs_var_for(&buck_text, &target.name) {
                for v in resolve_mapped_var(&buck_text, &var, &string_vars, &glob_vars, &crate_files) {
                    dests.insert(normalize_rel(&v));
                }
            }

            if target.unparseable && !dests.contains(&resolved) {
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "resolved": resolved, "target": target.name,
                    "status": "unparseable",
                    "detail": "owning BUCK target srcs/mapped_srcs uses a construct the minimal parser cannot fully resolve"
                }));
                continue;
            }

            let status = if dests.contains(&resolved) {
                "resolved"
            } else {
                "unmapped"
            };
            let detail = if status == "unmapped" {
                format!(
                    "include `{literal}` from {rs_in_crate} resolves to sandbox path `{resolved}` which is NOT in target `{}` srcs/mapped_srcs destinations",
                    target.name
                )
            } else {
                String::new()
            };
            sites_out.push(json!({
                "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                "resolved": resolved, "target": target.name,
                "status": status, "detail": detail
            }));
        }
    }

    Ok(json!({ "sites": sites_out }))
}

/// Pick the BUCK target that owns a crate-relative `.rs` path.
fn pick_owning_target(
    targets: &[BuckTarget],
    rs_in_crate: &str,
    crate_dir_rel: &str,
    crate_files: &[String],
) -> Option<BuckTarget> {
    // 1. Exact crate_root match (covers tests/x.rs and src/lib.rs|main.rs).
    if let Some(t) = targets.iter().find(|t| t.crate_root.as_deref() == Some(rs_in_crate)) {
        return Some(t.clone());
    }
    // 2. crate_root that is a path-prefix's crate (e.g. a lib at src/lib.rs owns src/**.rs);
    //    pick a rust_library/rust_binary whose glob covers the file.
    let mut covering: Vec<&BuckTarget> = targets
        .iter()
        .filter(|t| {
            (t.kind == "rust_library" || t.kind == "rust_binary")
                && (t.srcs_globs.iter().any(|p| glob_match(p, rs_in_crate))
                    || t.explicit_srcs.iter().any(|s| s == rs_in_crate)
                    || globs_via_crate_root(t, rs_in_crate, crate_files))
        })
        .collect();
    if let Some(t) = covering.pop() {
        return Some(t.clone());
    }
    // 3. A rust_test whose explicit srcs / glob covers it.
    if let Some(t) = targets.iter().find(|t| {
        t.kind == "rust_test"
            && (t.srcs_globs.iter().any(|p| glob_match(p, rs_in_crate))
                || t.explicit_srcs.iter().any(|s| s == rs_in_crate))
    }) {
        return Some(t.clone());
    }
    // 4. Fallback: the single rust_library, if any, even with empty srcs (mapped_srcs-only crate
    //    whose crate_root sits at src/lib.rs and the file is under src/).
    let _ = crate_dir_rel;
    targets
        .iter()
        .find(|t| t.kind == "rust_library" && rs_in_crate.starts_with("src/"))
        .cloned()
}

fn globs_via_crate_root(t: &BuckTarget, rs_in_crate: &str, _crate_files: &[String]) -> bool {
    // A library whose crate_root is src/lib.rs implicitly owns src/**.rs in the common layout.
    if let Some(root) = &t.crate_root {
        if let Some(dir) = root.rsplit_once('/').map(|(d, _)| d) {
            return rs_in_crate.starts_with(&format!("{dir}/"));
        }
    }
    false
}

/// Find the bare-variable mapped_srcs reference for a named target by re-locating its block.
fn mapped_srcs_var_for(buck_text: &str, target_name: &str) -> Option<String> {
    // Locate the target block by its name string, then read mapped_srcs var.
    let needle = format!("\"{target_name}\"");
    let name_pos = buck_text.find(&needle)?;
    // Find the enclosing call's start by scanning backward for the kind identifier before name_pos.
    let head = &buck_text[..name_pos];
    let open = head.rfind('(')?;
    // Walk back to the start of the identifier.
    let kind_start = head[..open]
        .rfind(|c: char| c == '\n' || c == ' ')
        .map(|p| p + 1)
        .unwrap_or(0);
    let block = call_block(&buck_text[kind_start..])?;
    mapped_srcs_var(&block)
}

// ---------------------------------------------------------------------------
// Filesystem helpers (read-only)
// ---------------------------------------------------------------------------

fn walk_rust(
    dir: &Path,
    rust_ext: &str,
    excludes: &[String],
    out: &mut Vec<PathBuf>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => return Err(CollectError::Io(format!("read_dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry = entry.map_err(|e| CollectError::Io(format!("dir entry: {e}")))?;
        let path = entry.path();
        let path_str = path.to_string_lossy().replace('\\', "/");
        if excludes.iter().any(|sub| path_str.contains(sub.as_str())) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            walk_rust(&path, rust_ext, excludes, out)?;
        } else if file_type.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some(rust_ext)
        {
            out.push(path);
        }
    }
    Ok(())
}

/// List a crate's source files (crate-relative, `/`-separated) for glob expansion. Includes the
/// common asset extensions + .rs; excludes target/.git. Read-only.
fn list_crate_files(
    crate_dir_abs: &Path,
    crate_dir_rel: &str,
    root: &Path,
) -> Result<Vec<String>, CollectError> {
    let mut files = Vec::new();
    let _ = root;
    walk_all(crate_dir_abs, crate_dir_abs, &mut files)?;
    let prefix = format!("{crate_dir_rel}/");
    let _ = prefix;
    Ok(files)
}

fn walk_all(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => return Err(CollectError::Io(format!("read_dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry = entry.map_err(|e| CollectError::Io(format!("dir entry: {e}")))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "target" || name == ".git" || name == "node_modules" {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            walk_all(base, &path, out)?;
        } else if file_type.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

/// Nearest ancestor directory (from `rs_abs` up to `root`) containing a `BUCK` file.
fn nearest_buck_dir(root: &Path, rs_abs: &Path) -> Option<PathBuf> {
    let mut dir = rs_abs.parent()?;
    loop {
        if dir.join("BUCK").is_file() {
            return Some(dir.to_path_buf());
        }
        if dir == root {
            return None;
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return None,
        }
    }
}

/// Relative path of `abs` under `root` as a `/`-separated string.
fn rel_to(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| abs.to_string_lossy().replace('\\', "/"))
}

/// Strip the `crate_dir_rel/` prefix from a repo-relative path, yielding the crate-relative path.
fn strip_prefix_slash(rs_rel: &str, crate_dir_rel: &str) -> String {
    if crate_dir_rel.is_empty() {
        return rs_rel.to_owned();
    }
    let prefix = format!("{crate_dir_rel}/");
    rs_rel.strip_prefix(&prefix).unwrap_or(rs_rel).to_owned()
}

/// Parent dir of a `/`-separated path (empty if no slash).
fn parent_dir(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir.to_owned(),
        None => String::new(),
    }
}

/// True if the include literal is an out-of-scope build-output path: it starts with one of the
/// out-of-scope prefixes AND its extension is not an embedded-asset extension.
fn is_out_of_scope(literal: &str, oos_prefixes: &[String], embedded_exts: &BTreeSet<String>) -> bool {
    let starts = oos_prefixes.iter().any(|p| literal.starts_with(p.as_str()));
    if !starts {
        return false;
    }
    let ext = Path::new(literal)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    !embedded_exts.contains(&ext)
}

// Re-export Component to keep the import used even if the path API evolves; the lexical normalizer
// does not need it but std::path::Component documents intent.
#[allow(dead_code)]
fn _component_marker(_c: Component<'_>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "scan_roots": ["x"],
            "embedded_extensions": ["cedar", "json", "txt"],
            "out_of_scope_path_prefixes": ["../../../out/"]
        })
    }

    fn observed(sites: Vec<Value>) -> Value {
        json!({ "sites": sites })
    }

    // ---- resolution / normalization -------------------------------------

    #[test]
    fn normalize_resolves_dotdot() {
        assert_eq!(normalize_rel("crate/src/../policy/x.cedar"), "crate/policy/x.cedar");
        assert_eq!(normalize_rel("a/./b"), "a/b");
        assert_eq!(normalize_rel("../../foo"), "../../foo");
        assert_eq!(normalize_rel("a/b/../../c"), "c");
    }

    #[test]
    fn resolve_include_is_site_dir_relative() {
        // The cedar-adapter case: from <crate>/src, `../policy/x.cedar` -> <crate>/policy/x.cedar.
        assert_eq!(
            resolve_include("cloud/ci/adapter/src", "../policy/cloud-intelligence.cedar"),
            "cloud/ci/adapter/policy/cloud-intelligence.cedar"
        );
        assert_eq!(resolve_include("crate/src", "bundled/x.json"), "crate/src/bundled/x.json");
    }

    // ---- include-site extraction ----------------------------------------

    #[test]
    fn extracts_string_literal_sites() {
        let src = r#"
            const A: &str = include_str!("../policy/x.cedar");
            const B: &[u8] = include_bytes!("data/y.json");
        "#;
        let sites = extract_include_sites(src);
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].macro_name, "include_str");
        assert_eq!(sites[0].literal.as_deref(), Some("../policy/x.cedar"));
        assert_eq!(sites[1].macro_name, "include_bytes");
        assert_eq!(sites[1].literal.as_deref(), Some("data/y.json"));
    }

    #[test]
    fn non_literal_include_yields_none() {
        let src = r#"const A: &str = include_str!(concat!("a", "b"));"#;
        let sites = extract_include_sites(src);
        assert_eq!(sites.len(), 1);
        assert!(sites[0].literal.is_none(), "concat! arg must be non-literal");
    }

    #[test]
    fn identifier_ending_in_macro_name_is_not_a_site() {
        let src = "fn my_include_str() {}";
        assert!(extract_include_sites(src).is_empty());
    }

    // ---- glob matcher ---------------------------------------------------

    #[test]
    fn glob_double_star_matches_nested() {
        assert!(glob_match("src/**/*.rs", "src/a/b/c.rs"));
        assert!(glob_match("src/**/*.rs", "src/lib.rs"));
        assert!(glob_match("**/*.cedar", "policy/x.cedar"));
        assert!(!glob_match("src/*.rs", "src/a/b.rs"));
        assert!(glob_match("src/*.rs", "src/lib.rs"));
    }

    // ---- BUCK destination construction ----------------------------------

    #[test]
    fn glob_srcs_destinations_are_crate_relative() {
        let buck = r#"
rust_library(
    name = "c",
    srcs = glob(["src/**/*.rs", "**/*.json"]),
    crate_root = "src/lib.rs",
)
"#;
        let files = vec!["src/lib.rs".to_owned(), "bundled/x.json".to_owned()];
        let targets = parse_buck_targets(buck, &files);
        assert_eq!(targets.len(), 1);
        let dests = target_destinations(&targets[0], "crate/c", &files);
        assert!(dests.contains("crate/c/src/lib.rs"));
        assert!(dests.contains("crate/c/bundled/x.json"));
    }

    #[test]
    fn mapped_srcs_comprehension_plus_explicit_value() {
        // The cedar-adapter shape: a comprehension var + an explicit value assignment.
        let buck = r#"
ADAPTER_ROOT = "cloud/ci/adapter"
ADAPTER_SRCS = glob(["src/**/*.rs", "**/*.cedar"])
ADAPTER_MAPPED_SRCS = {src: ADAPTER_ROOT + "/" + src for src in ADAPTER_SRCS}
ADAPTER_MAPPED_SRCS["//cloud/ci/policy:x.cedar"] = ADAPTER_ROOT + "/policy/x.cedar"

rust_library(
    name = "adapter",
    srcs = [],
    crate_root = ADAPTER_ROOT + "/src/lib.rs",
    mapped_srcs = ADAPTER_MAPPED_SRCS,
)
"#;
        let files = vec!["src/lib.rs".to_owned()];
        let string_vars = top_level_string_vars(buck);
        let glob_vars = top_level_glob_vars(buck);
        let values = resolve_mapped_var(buck, "ADAPTER_MAPPED_SRCS", &string_vars, &glob_vars, &files);
        assert!(
            values.iter().any(|v| v == "cloud/ci/adapter/policy/x.cedar"),
            "explicit mapped value must resolve: {values:?}"
        );
        assert!(
            values.iter().any(|v| v == "cloud/ci/adapter/src/lib.rs"),
            "comprehension value must resolve: {values:?}"
        );
    }

    // ---- evaluator ------------------------------------------------------

    #[test]
    fn resolved_site_is_green() {
        let report = evaluate(
            &policy(),
            &observed(vec![json!({"key":"a.rs:1","status":"resolved"})]),
        );
        assert_eq!(report.verdict, Verdict::Green);
    }

    #[test]
    fn unmapped_site_is_red() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![json!({"key":"a.rs:1","status":"unmapped","detail":"d"})]),
        );
        assert!(findings.iter().any(|f| f.code == "embedded_asset_unmapped_include" && f.key == "a.rs:1"));
    }

    #[test]
    fn skip_and_no_target_map_to_their_codes() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![
                json!({"key":"a.rs:1","status":"unparseable"}),
                json!({"key":"b.rs:2","status":"no_target"}),
            ]),
        );
        assert!(findings.iter().any(|f| f.code == "embedded_asset_unparseable_skip" && f.key == "a.rs:1"));
        assert!(findings.iter().any(|f| f.code == "embedded_asset_no_target" && f.key == "b.rs:2"));
    }

    #[test]
    fn out_of_scope_status_contributes_no_finding() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![json!({"key":"a.rs:1","status":"out_of_scope"})]),
        );
        assert!(findings.is_empty());
    }

    #[test]
    fn gate_id_mismatch_fails_closed() {
        let mut bad = policy();
        bad["gate_id"] = json!("wrong");
        let findings = evaluate_keyed(&bad, &observed(vec![]));
        assert!(findings.iter().any(|f| f.code == "embedded_asset_policy_gate_id_mismatch"));
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = observed(vec![
            json!({"key":"a.rs:1","status":"unmapped"}),
            json!({"key":"b.rs:2","status":"unparseable"}),
        ]);
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &input)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(evaluate(&policy(), &input).violations, projected);
    }

    #[test]
    fn is_out_of_scope_classifies_build_outputs() {
        let exts: BTreeSet<String> = ["cedar", "json"].into_iter().map(str::to_owned).collect();
        assert!(is_out_of_scope("../../../out/svc.elf", &["../../../out/".to_owned()], &exts));
        // a .cedar under out/ is still an embedded asset -> NOT out of scope.
        assert!(!is_out_of_scope("../../../out/policy.cedar", &["../../../out/".to_owned()], &exts));
    }
}
