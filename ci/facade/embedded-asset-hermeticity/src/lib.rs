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
//!   the owning target's `srcs`/`mapped_srcs` destinations.
//! - `embedded_asset_no_target`               — a `.rs` with an include site has no bindable BUCK target.
//! - `embedded_asset_unparseable_skip`        — the include literal or the owning target's
//!   `srcs`/`mapped_srcs` is not statically resolvable.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub use oya_buck_syntax_kernel::glob_match;
use oya_buck_syntax_kernel::{
    BuckDoc, Env, Expr, PreImageRegistry, Stmt, call_strings, dict_values, eval_string,
    find_target, guarded_rewrite, insert_dict_entry, insert_kwarg, replace_span, resolve_dict_var,
};
use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the oya-ci registry id.
pub const GATE_ID: &str = "cloud-ci-embedded-asset-hermeticity";

/// The BLOCKING violation codes — the single source of truth for the verdict. A `Finding` whose code
/// is in this set flips the verdict to Red; any other code (the `SKIP_CODES`) is surfaced but does
/// NOT flip the verdict (it is baselined + capped). Per the ralplan consensus: `Report.violations`
/// filters by membership in `VIOLATION_CODES`, never by the `skip_` name prefix.
pub const VIOLATION_CODES: [&str; 2] = [
    "embedded_asset_unmapped_include",
    "embedded_asset_policy_gate_id_mismatch",
];

/// The non-blocking SKIP codes the gate can emit. Surfaced (counted, baselined, set-equality
/// asserted) but never verdict-flipping — a site the gate cannot fully resolve must fail SAFE
/// (visible) rather than fail OPEN (silent) or fail CLOSED (false RED).
pub const SKIP_CODES: [&str; 5] = [
    "skip_non_literal_argument",
    "skip_absolute_literal",
    "skip_build_output_path",
    "skip_no_owning_target",
    "skip_buck_unparseable",
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
                write!(
                    f,
                    "policy `scan_roots` must be a non-empty array of strings"
                )
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
    /// Build the report from the keyed findings. ONLY codes in [`VIOLATION_CODES`] count toward the
    /// verdict; skip-class findings are present in the keyed set (so they are reported + baselined)
    /// but are excluded here, so they never flip the verdict. `VIOLATION_CODES` is the single source
    /// of truth — the `skip_` name prefix is documentation, never the filter.
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .filter(|code| VIOLATION_CODES.contains(&code.as_str()))
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
        // The collector emits a `status` that names the outcome class; the evaluator maps it to a
        // finding code. `resolved` produces no finding. A `status` already equal to a known code
        // (collector emits skip codes directly) is passed through verbatim, so the taxonomy is shared
        // between collector and evaluator with no second mapping table.
        match status {
            "resolved" => {}
            "unmapped" => {
                findings.insert(Finding::new("embedded_asset_unmapped_include", key, detail));
            }
            other if VIOLATION_CODES.contains(&other) || SKIP_CODES.contains(&other) => {
                findings.insert(Finding::new(other, key, detail));
            }
            // Any unknown status contributes no finding (defensive: never silently fabricate one).
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
/// `literal: None` (the gate classifies these as `skip_non_literal_argument`). Pure; no filesystem.
///
/// Context-aware: a mini-lexer tracks line/block comments, string literals (incl. raw strings
/// `r"..."` / `r#"..."#`), and char literals, so a macro NAME appearing inside a comment, a string
/// (e.g. this crate's own `const MACROS = ["include_str!", ..]`), or a doc example is NOT treated as
/// a real invocation. This is the correctness fix that stops the gate from flagging its own source.
/// It is not a full Rust parser; it recognizes exactly the lexical contexts needed to avoid false
/// macro hits, and reads the macro argument only when it is a single plain `"..."` string literal.
pub fn extract_include_sites(source: &str) -> Vec<IncludeSite> {
    const MACROS: [(&str, &str); 2] = [
        ("include_str!", "include_str"),
        ("include_bytes!", "include_bytes"),
    ];
    let mut sites = Vec::new();
    let bytes = source.as_bytes();
    let len = source.len();
    let mut i = 0usize;
    let mut line = 1usize;

    while i < len {
        let c = bytes[i];
        if c == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        // Line comment.
        if c == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment (nesting-aware, as Rust allows `/* /* */ */`).
        if c == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            let mut depth = 1usize;
            while i < len && depth > 0 {
                if bytes[i] == b'\n' {
                    line += 1;
                    i += 1;
                } else if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Raw string `r"..."` or `r#"..."#` (any number of `#`).
        if (c == b'r' || c == b'b')
            && raw_string_start(bytes, i)
            && !is_ident_byte(if i > 0 { bytes[i - 1] } else { b' ' })
        {
            let (next, newlines) = skip_raw_string(bytes, i);
            line += newlines;
            i = next;
            continue;
        }
        // Plain string literal.
        if c == b'"' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                if bytes[i] == b'\n' {
                    line += 1;
                }
                i += 1;
            }
            continue;
        }
        // Char literal `'x'` / `'\n'` — distinguish from a lifetime `'a` (no closing quote soon).
        if c == b'\'' && is_char_literal(bytes, i) {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'\'' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        // A potential macro invocation in real code. Only attempt a match when `bytes[i]` is the
        // ASCII letter the needles begin with (`i` of "include_…"); this both narrows the work and
        // guarantees `source[i..]` slices at a char boundary (ASCII bytes are always boundaries),
        // avoiding a panic when the byte cursor is over a multibyte character.
        let mut matched = false;
        if c == b'i' && (i == 0 || !is_ident_byte(bytes[i - 1])) {
            for (needle, base) in MACROS {
                if source[i..].starts_with(needle) {
                    let after = &source[i + needle.len()..];
                    let literal = parse_macro_string_arg(after);
                    sites.push(IncludeSite {
                        macro_name: base.to_owned(),
                        literal,
                        line,
                    });
                    i += needle.len();
                    matched = true;
                    break;
                }
            }
        }
        if matched {
            continue;
        }
        // Advance by the full width of the current UTF-8 character so the cursor never lands inside a
        // multibyte char (which would make a later `source[i..]` slice panic).
        i += utf8_char_width(c);
    }

    sites.sort_by(|a, b| a.line.cmp(&b.line).then(a.macro_name.cmp(&b.macro_name)));
    sites
}

fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// UTF-8 encoded width (1–4 bytes) of the character whose first byte is `b`. A continuation byte
/// (0b10xxxxxx, only reachable if the cursor is already mid-char) returns 1 so the scan still makes
/// progress without panicking.
fn utf8_char_width(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >= 0xF0 {
        4
    } else if b >= 0xE0 {
        3
    } else if b >= 0xC0 {
        2
    } else {
        1
    }
}

/// True if `bytes[i]` begins a raw string: `r"`, `r#`, `br"`, `br#` (the `b` prefix handled by the
/// caller passing index at `r`). Accepts the form `r` (`#`*) `"`.
fn raw_string_start(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    if bytes.get(j) == Some(&b'b') {
        j += 1;
    }
    if bytes.get(j) != Some(&b'r') {
        return false;
    }
    j += 1;
    while bytes.get(j) == Some(&b'#') {
        j += 1;
    }
    bytes.get(j) == Some(&b'"')
}

/// Skip a raw string starting at `i`; returns (index past closing quote+hashes, newlines consumed).
fn skip_raw_string(bytes: &[u8], i: usize) -> (usize, usize) {
    let mut j = i;
    if bytes.get(j) == Some(&b'b') {
        j += 1;
    }
    // consume `r`
    j += 1;
    let mut hashes = 0usize;
    while bytes.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    // consume opening quote
    j += 1;
    let mut newlines = 0usize;
    while j < bytes.len() {
        if bytes[j] == b'\n' {
            newlines += 1;
            j += 1;
            continue;
        }
        if bytes[j] == b'"' {
            // check for the right number of closing hashes
            let mut k = j + 1;
            let mut count = 0usize;
            while count < hashes && bytes.get(k) == Some(&b'#') {
                count += 1;
                k += 1;
            }
            if count == hashes {
                return (k, newlines);
            }
        }
        j += 1;
    }
    (j, newlines)
}

/// Distinguish a char literal `'x'`/`'\n'`/`'\\''` from a lifetime token `'a`. A char literal has a
/// closing `'` within a few bytes; a lifetime is `'` followed by an identifier and NOT a closing `'`.
fn is_char_literal(bytes: &[u8], i: usize) -> bool {
    // `'\...'` escape form is always a char literal.
    if bytes.get(i + 1) == Some(&b'\\') {
        return true;
    }
    // `'x'` — char then closing quote.
    if bytes.get(i + 2) == Some(&b'\'') {
        return true;
    }
    false
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

/// Parse the BUCK text of a crate dir into the targets the gate cares about, via the shared
/// sound `oya-buck-syntax-kernel` parser (ADR-0549). Pure over text + (for glob expansion) the
/// on-disk crate tree, which the caller passes via `crate_files` (crate-relative paths).
/// Conservative: a target whose srcs/mapped_srcs uses a construct outside the modeled subset is
/// marked `unparseable` rather than silently treated as empty; a BUCK text that does not parse
/// soundly yields NO bindable targets (the collector then surfaces a visible skip, never a
/// silent pass — fail-safe).
pub fn parse_buck_targets(buck_text: &str, crate_files: &[String]) -> Vec<BuckTarget> {
    const KINDS: [&str; 3] = ["rust_library", "rust_binary", "rust_test"];
    let Ok(doc) = oya_buck_syntax_kernel::parse(buck_text) else {
        return Vec::new();
    };
    // Top-level `IDENT = "string"` and `IDENT = glob([...])` assignments, so crate_root / srcs /
    // mapped_srcs can reference them (the cedar-adapter ADAPTER_ROOT/ADAPTER_SRCS shape).
    let env = Env::from_doc(&doc);

    let mut targets = Vec::new();
    // Kind-grouped order preserved from the pre-kernel scanner (callers report the first
    // covering target's resolution; keep that deterministic ordering identical).
    for kind in KINDS {
        for stmt in &doc.stmts {
            let Stmt::Call(call) = stmt else { continue };
            if call.func != kind {
                continue;
            }
            let name = call
                .kwarg("name")
                .and_then(|arg| eval_string(&arg.value, &env))
                .unwrap_or_default();
            let crate_root = call
                .kwarg("crate_root")
                .and_then(|arg| eval_string(&arg.value, &env))
                .map(|p| crate_relative(&p));
            let (explicit_srcs, srcs_globs, srcs_unparseable) = parse_srcs(call, &env);
            let (mapped_dest, mapped_unparseable) = parse_mapped_srcs(call, &env, crate_files);
            // A positional opaque tail argument means an expression the subset could not model
            // sat NEXT TO a modeled kwarg value (e.g. `srcs = [...] if c else [...]` parses the
            // first list plus an opaque tail). The narrowed value must not be trusted as the
            // whole truth — demote the target to unparseable (fail-safe: the collector surfaces
            // a visible skip, never a silent narrow).
            let opaque_tail = call
                .args
                .iter()
                .any(|arg| arg.name.is_none() && arg.value.has_opaque());

            targets.push(BuckTarget {
                name,
                kind: kind.to_owned(),
                crate_root,
                explicit_srcs,
                srcs_globs,
                mapped_dest,
                unparseable: srcs_unparseable || mapped_unparseable || opaque_tail,
            });
        }
    }
    targets
}

/// The effective sandbox DESTINATION set for a target: glob-expanded `srcs` (crate-relative) plus
/// explicit srcs plus mapped_srcs values. `crate_files` is the crate's on-disk file list
/// (crate-relative, `/`-separated). Returns crate-relative destinations for in-crate srcs and the
/// raw mapped values (which are already sandbox-rooted, e.g. `cloud/.../policy/x.cedar`). Pure.
pub fn target_destinations(
    target: &BuckTarget,
    crate_dir: &str,
    crate_files: &[String],
) -> BTreeSet<String> {
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
// BUCK syntax (delegated to the shared sound parser — oya-buck-syntax-kernel, ADR-0549)
// ---------------------------------------------------------------------------

/// Top-level `IDENT = "string"` assignments. Used to resolve concat operands like `ADAPTER_ROOT`.
fn top_level_string_vars(buck_text: &str) -> Vec<(String, String)> {
    oya_buck_syntax_kernel::parse(buck_text)
        .map(|doc| Env::from_doc(&doc).string_vars.into_iter().collect())
        .unwrap_or_default()
}

/// Top-level `IDENT = glob([...])` assignments -> the glob patterns. Used to resolve a
/// srcs/mapped comprehension `for src in SRCS`.
fn top_level_glob_vars(buck_text: &str) -> Vec<(String, Vec<String>)> {
    oya_buck_syntax_kernel::parse(buck_text)
        .map(|doc| Env::from_doc(&doc).glob_vars.into_iter().collect())
        .unwrap_or_default()
}

/// Parse a target's `srcs` kwarg. Returns (explicit_srcs, glob_patterns, unparseable).
fn parse_srcs(
    call: &oya_buck_syntax_kernel::CallExpr,
    env: &Env,
) -> (Vec<String>, Vec<String>, bool) {
    let Some(arg) = call.kwarg("srcs") else {
        // No srcs field (e.g. a target driven purely by crate_root + mapped_srcs): not
        // unparseable; crate_root alone still yields a destination.
        return (Vec::new(), Vec::new(), false);
    };
    match &arg.value.expr {
        Expr::List(list) if list.elements.is_empty() => (Vec::new(), Vec::new(), false),
        Expr::List(_) => (
            oya_buck_syntax_kernel::expr_strings(&arg.value),
            Vec::new(),
            false,
        ),
        Expr::Call(call) if call.func == "glob" => (Vec::new(), call_strings(call), false),
        Expr::Ident(name) => {
            // A bare var reference, e.g. `srcs = ADAPTER_SRCS` (a glob var).
            match env.glob_vars.get(name) {
                Some(patterns) => (Vec::new(), patterns.clone(), false),
                None => (Vec::new(), Vec::new(), true),
            }
        }
        // An unmodeled srcs construct.
        _ => (Vec::new(), Vec::new(), true),
    }
}

/// Parse a target's `mapped_srcs` kwarg into its destination VALUES. A bare var reference is
/// resolved at the FILE level (see [`resolve_mapped_var`]) — here it yields empty + parseable,
/// exactly like the pre-kernel implementation, and the collector augments via the file-level
/// resolver.
fn parse_mapped_srcs(
    call: &oya_buck_syntax_kernel::CallExpr,
    env: &Env,
    crate_files: &[String],
) -> (Vec<String>, bool) {
    let Some(arg) = call.kwarg("mapped_srcs") else {
        return (Vec::new(), false);
    };
    match &arg.value.expr {
        Expr::Ident(_) => (Vec::new(), false),
        Expr::Dict(dict) => (dict_values(dict, env, crate_files), false),
        _ => (Vec::new(), true),
    }
}

/// Resolve the destination VALUES of a top-level mapped_srcs variable assembled as
/// `VAR = {src: ROOT + "/" + src for src in SRCS}` plus `VAR["k"] = ROOT + "/path"` assignments.
/// `var` is the variable name referenced by a target's `mapped_srcs = VAR`. Pure over the BUCK
/// text; the caller-supplied var slices keep the migrated signature behavior-identical.
pub fn resolve_mapped_var(
    buck_text: &str,
    var: &str,
    string_vars: &[(String, String)],
    glob_vars: &[(String, Vec<String>)],
    crate_files: &[String],
) -> Vec<String> {
    let Ok(doc) = oya_buck_syntax_kernel::parse(buck_text) else {
        return Vec::new();
    };
    let env = Env::from_slices(string_vars, glob_vars);
    resolve_dict_var(&doc, var, &env, crate_files)
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
                    "literal": site.literal, "status": "skip_no_owning_target",
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
        let mut targets = parse_buck_targets(&buck_text, &crate_files);
        // Normalize each target's crate_root to crate-relative: a `crate_root = ROOT + "/src/lib.rs"`
        // concat resolves to a REPO-relative path (e.g. `cloud/.../src/lib.rs`); strip the crate-dir
        // prefix so it compares cleanly against the crate-relative `.rs` file path. Without this the
        // cedar-adapter shape (srcs=[] + ROOT-prefixed crate_root) binds to no file and is wrongly
        // skipped instead of resolved.
        let crate_prefix = format!("{crate_dir_rel}/");
        for t in &mut targets {
            if let Some(root) = &t.crate_root
                && let Some(stripped) = root.strip_prefix(&crate_prefix)
            {
                t.crate_root = Some(stripped.to_owned());
            }
        }

        // The including file's crate-relative path.
        let rs_in_crate = strip_prefix_slash(&rs_rel, &crate_dir_rel);

        // Every target that compiles this file (a file is often in a lib + its rust_test + a binary,
        // each with its own __srcs tree); the include is hermetic if mapped by ANY of them.
        let covering = covering_targets(&targets, &rs_in_crate, &crate_files);

        for site in &sites {
            // key = T::F::L once the owning target is known; until then F:line for the early skips.
            let file_line = format!("{rs_rel}:{}", site.line);
            let Some(literal) = &site.literal else {
                sites_out.push(json!({
                    "key": file_line, "rs": rs_rel, "macro": site.macro_name, "literal": Value::Null,
                    "status": "skip_non_literal_argument",
                    "detail": "include argument is not a single string literal (concat!/env!/raw string/expression)"
                }));
                continue;
            };

            // Absolute literal: rustc resolves an absolute include against the filesystem, not the
            // sandbox tree — the hermeticity rule does not apply, so skip (surfaced, not silent).
            if literal.starts_with('/') {
                sites_out.push(json!({
                    "key": file_line, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "status": "skip_absolute_literal",
                    "detail": "include path is absolute; sandbox-tree hermeticity does not apply"
                }));
                continue;
            }

            // Out-of-scope build-output paths: a literal under a configured build-output dir whose
            // extension is NOT an embedded asset extension is a generated buck2 action output, not a
            // tracked source the BUCK target must map -> skip.
            if is_out_of_scope(literal, &oos_prefixes, &embedded_exts) {
                sites_out.push(json!({
                    "key": file_line, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "status": "skip_build_output_path",
                    "detail": "literal resolves into an out-of-scope build-output tree (not a tracked embedded asset)"
                }));
                continue;
            }

            if covering.is_empty() {
                sites_out.push(json!({
                    "key": file_line, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "status": "skip_no_owning_target",
                    "detail": format!("no BUCK target in {crate_dir_rel}/BUCK owns {rs_in_crate}")
                }));
                continue;
            }

            // Resolve PER COVERING TARGET. The include resolves lexically from the dir of the
            // including file's destination IN THAT TARGET's `__srcs` tree: `dest_T(F)` is F's mapped
            // VALUE if F is itself a mapped source of T (the comprehension shape places `src/lib.rs`
            // at `ROOT/src/lib.rs`), else F's package-relative SHORT path (`src/lib.rs`). This is the
            // load-bearing distinction the buck2 build proved: with a short crate_root the include
            // base is `src/`, so `../../../policy` ESCAPES the tree (RED); with a ROOT-prefixed
            // crate_root the base is `ROOT/src/`, so it resolves to the mapped VALUE (GREEN). A site
            // is hermetic iff SOME covering target resolves it within its own destination set.
            let mut target_names: Vec<String> = Vec::new();
            let mut any_unparseable = false;
            let mut member = false;
            let mut representative_resolved = String::new();
            for target in &covering {
                target_names.push(target.name.clone());
                any_unparseable |= target.unparseable;
                // D(T): short-path srcs ∪ mapped VALUES (inline + bare-var comprehension).
                let mut dests = target_destinations(target, &crate_dir_rel, &crate_files);
                if let Some(var) = mapped_srcs_var_for(&buck_text, &target.name) {
                    for v in
                        resolve_mapped_var(&buck_text, &var, &string_vars, &glob_vars, &crate_files)
                    {
                        dests.insert(normalize_rel(&v));
                    }
                }
                // dest_T(F): F's mapped VALUE in T if mapped here, else F's short path. The short
                // path is `rs_in_crate` UNLESS T's crate_root is ROOT-prefixed, in which case T maps
                // every src to `ROOT/<short>` and F's destination is the crate-prefixed path.
                let dest_f = file_destination(target, &rs_in_crate, &crate_dir_rel, &dests);
                let site_dir = parent_dir(&dest_f);
                let resolved = resolve_include(&site_dir, literal);
                if representative_resolved.is_empty() {
                    representative_resolved = resolved.clone();
                }
                if dests.contains(&resolved) {
                    member = true;
                    representative_resolved = resolved;
                    break;
                }
            }
            target_names.sort();
            let primary = target_names.first().cloned().unwrap_or_default();
            let key = format!("{primary}::{rs_in_crate}::{literal}");
            let resolved = representative_resolved;

            // If a covering target uses an unmodelled BUCK construct AND we cannot positively confirm
            // membership, skip rather than risk a false RED (fail-safe over fail-closed).
            if any_unparseable && !member {
                sites_out.push(json!({
                    "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                    "resolved": resolved, "targets": target_names,
                    "status": "skip_buck_unparseable",
                    "detail": "a covering BUCK target srcs/mapped_srcs uses a construct the minimal parser cannot fully resolve"
                }));
                continue;
            }

            // Membership: R ∈ D(some covering T) is hermetic. Non-membership (incl. an escaped `..`
            // path, which can never be a sandbox tree member) is the blocking unmapped-include defect.
            let status = if member { "resolved" } else { "unmapped" };
            let detail = if status == "unmapped" {
                format!(
                    "include `{literal}` from {rs_in_crate} resolves to sandbox path `{resolved}` which is NOT in any covering target's srcs/mapped_srcs destinations (targets: {}); run --fix to derive the mapped_srcs entry",
                    target_names.join(", ")
                )
            } else {
                String::new()
            };
            sites_out.push(json!({
                "key": key, "rs": rs_rel, "macro": site.macro_name, "literal": literal,
                "resolved": resolved, "targets": target_names,
                "status": status, "detail": detail
            }));
        }
    }

    Ok(json!({ "sites": sites_out }))
}

/// The sandbox destination of the including file `F` inside target `T`'s `__srcs` tree. buck2 places
/// each src at its declared destination: a plain/glob src lands at its package-relative SHORT path
/// (`src/lib.rs`); a `mapped_srcs` comprehension `{src: ROOT + "/" + src for ...}` lands it at the
/// crate-prefixed path (`crate_dir/src/lib.rs`). We pick whichever candidate is actually a member of
/// `T`'s destination set `dests` — that is exactly where buck2 will materialize F. If neither is a
/// member (F not in this target's srcs at all, or an unmodelled construct), fall back to the short
/// path, the buck2 default for a plain src.
fn file_destination(
    _target: &BuckTarget,
    rs_in_crate: &str,
    crate_dir_rel: &str,
    dests: &BTreeSet<String>,
) -> String {
    let full = join_crate(crate_dir_rel, rs_in_crate);
    if dests.contains(&full) {
        return full;
    }
    if dests.contains(rs_in_crate) {
        return rs_in_crate.to_owned();
    }
    rs_in_crate.to_owned()
}

/// Every BUCK target that compiles the crate-relative `.rs` file. A single source file is commonly
/// compiled by several targets (a `rust_library`, its `rust_test` sibling, and a `rust_binary`),
/// each with its OWN sandbox `__srcs` tree. The hermeticity check is per-tree, but because the gate
/// cannot statically resolve `#[cfg(test)]` gating (a `tests/` fixture include lives only in the
/// `rust_test`'s tree, never the lib's), the collector resolves an include against the UNION of all
/// covering trees and treats membership in ANY of them as hermetic. This is the no-false-RED posture:
/// an include that is mapped by the target that actually compiles it passes; only an include mapped
/// by NO covering target (the FRIC-1781131000 defect) is the blocking unmapped finding.
fn covering_targets(
    targets: &[BuckTarget],
    rs_in_crate: &str,
    crate_files: &[String],
) -> Vec<BuckTarget> {
    let mut covering: Vec<BuckTarget> = Vec::new();
    for t in targets {
        let owns = t.crate_root.as_deref() == Some(rs_in_crate)
            || t.srcs_globs.iter().any(|p| glob_match(p, rs_in_crate))
            || t.explicit_srcs.iter().any(|s| s == rs_in_crate)
            || globs_via_crate_root(t, rs_in_crate, crate_files);
        if owns {
            covering.push(t.clone());
        }
    }
    covering
}

fn globs_via_crate_root(t: &BuckTarget, rs_in_crate: &str, _crate_files: &[String]) -> bool {
    // A library whose crate_root is src/lib.rs implicitly owns src/**.rs in the common layout.
    if let Some(root) = &t.crate_root
        && let Some(dir) = root.rsplit_once('/').map(|(d, _)| d)
    {
        return rs_in_crate.starts_with(&format!("{dir}/"));
    }
    false
}

/// Find the bare-variable mapped_srcs reference for a named target. Sound binding by the actual
/// `name` kwarg via the shared kernel — never a first-occurrence substring match (the ADR-0545
/// "first-occurrence name binding" residual the kernel retires).
fn mapped_srcs_var_for(buck_text: &str, target_name: &str) -> Option<String> {
    let doc = oya_buck_syntax_kernel::parse(buck_text).ok()?;
    let env = Env::from_doc(&doc);
    let call = find_target(&doc, None, target_name, &env)?;
    match &call.kwarg("mapped_srcs")?.value.expr {
        Expr::Ident(var) => Some(var.clone()),
        _ => None,
    }
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
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some(rust_ext)
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
        } else if file_type.is_file()
            && let Ok(rel) = path.strip_prefix(base)
        {
            out.push(rel.to_string_lossy().replace('\\', "/"));
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
        dir = dir.parent()?;
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
fn is_out_of_scope(
    literal: &str,
    oos_prefixes: &[String],
    embedded_exts: &BTreeSet<String>,
) -> bool {
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

// ---------------------------------------------------------------------------
// Auto-remediation (`--fix`) — automation-default layer (founder directive 2026-06-11:
// "automation should be the default; enforcement is the extra layer"). The gate DETECTS, the fixer
// DERIVES+APPLIES the corrected BUCK mapping, and the blocking `*-gate` rust_test is the backstop for
// what cannot be safely auto-derived. Precedent: the cloud-ci face-settle tool (`--settle --commit`
// is the default path; the freshness gate is the backstop).
// ---------------------------------------------------------------------------

/// How the unmapped include is to be remediated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemediationKind {
    /// The target already places the including file at the deep crate-prefixed path (a comprehension
    /// / `ROOT + "/..."` crate_root): a single `mapped_srcs[key] = value` add/replace is the fix.
    MappedEntry,
    /// The target uses a SHORT `crate_root` (`src/lib.rs`) + glob srcs, so a cross-package `../`
    /// include escapes its `__srcs` tree. The fix is the proven cedar comprehension rewrite of the
    /// whole target: `ROOT` var, `srcs = []`, `crate_root = ROOT + "/src/lib.rs"`, and
    /// `mapped_srcs = {comprehension}` + the explicit asset entry — placing every src at the deep
    /// path so the include resolves to the mapped VALUE.
    ComprehensionRewrite,
}

/// A derived remediation for one unmapped include site. `applicable` is the safe-to-auto-apply case
/// (a cross-package or in-tree asset whose file exists on disk at a derivable source); otherwise the
/// fixer reports it for manual handling (the backstop), never guessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remediation {
    /// Repo-relative BUCK file to patch.
    pub buck_path: String,
    /// The target name whose mapped_srcs must gain the entry.
    pub target: String,
    /// The mapped_srcs KEY to add — an `//pkg:name` export_file label if the asset lives in another
    /// package (the file's own BUCK exports it), else the asset's repo-relative source path.
    pub mapped_key: String,
    /// The mapped_srcs VALUE — the include-relative sandbox destination the asset must land at.
    pub mapped_value: String,
    /// The transform required to make the target hermetic.
    pub kind: RemediationKind,
    /// True if the fixer can apply this automatically; false = report-only (manual, the backstop).
    pub applicable: bool,
    /// Human-readable rationale / manual instruction when `!applicable`.
    pub note: String,
}

/// Derive remediations for an unmapped site — one per covering target. `site` is an observed
/// `unmapped` row; `root` is the repo root (read-only). Returns an empty vec for non-unmapped rows.
///
/// Finding 3 fix: the gate evaluates membership in the UNION of all covering targets
/// (src/lib.rs collector, `member = ANY covering T`). The fixer must therefore patch ALL covering
/// targets of the flagged include — patching only targets[0] while a sibling unittest target lacks
/// the mapping creates a gate-GREEN / build-RED state. We return one Remediation per target and
/// `derive_all_remediations` flat-maps them so every covering target gets the entry.
pub fn derive_remediation(root: &Path, site: &Value) -> Vec<Remediation> {
    if site.get("status").and_then(Value::as_str) != Some("unmapped") {
        return Vec::new();
    }
    let Some(resolved) = site
        .get("resolved")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return Vec::new();
    };

    // Collect ALL covering targets (the union the gate used for membership).
    let all_targets: Vec<String> = site
        .get("targets")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    // Fall back to legacy single-target field for backward compat.
    let targets_to_patch: Vec<String> = if all_targets.is_empty() {
        site.get("target")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(|s| vec![s.to_owned()])
            .unwrap_or_default()
    } else {
        all_targets
    };

    if targets_to_patch.is_empty() {
        // No bindable target — produce one non-applicable remediation to surface in the report.
        return vec![Remediation {
            buck_path: String::new(),
            target: String::new(),
            mapped_key: String::new(),
            mapped_value: resolved,
            kind: RemediationKind::MappedEntry,
            applicable: false,
            note: "could not bind the unmapped include to any owning target; manual fix required"
                .to_owned(),
        }];
    }

    // Shared fields derived once (they are the same for every covering target of this include).
    let Some(rs) = site.get("rs").and_then(Value::as_str).map(str::to_owned) else {
        return Vec::new();
    };
    let Some(buck_path) = nearest_buck_path_rel(root, &rs) else {
        return Vec::new();
    };
    let crate_dir = buck_path.trim_end_matches("/BUCK").to_owned();
    let (mapped_key, label_ok) = derive_mapped_key(root, &resolved, &crate_dir);
    let asset_exists = root.join(&resolved).is_file();

    targets_to_patch
        .iter()
        .map(|target| {
            let kind = match buck_crate_root_is_root_prefixed(root, &buck_path, target, &crate_dir) {
                Some(true) => RemediationKind::MappedEntry,
                Some(false) => RemediationKind::ComprehensionRewrite,
                None => RemediationKind::MappedEntry,
            };
            let applicable = asset_exists && label_ok;
            let note = if !asset_exists {
                format!("asset not found on disk at `{resolved}` — the include path is wrong, or the asset must be created; manual fix required")
            } else if !label_ok {
                format!("could not derive a hermetic mapped_srcs key for `{resolved}`; add the entry by hand (see ADR-0545)")
            } else {
                match kind {
                    RemediationKind::MappedEntry => format!(
                        "add/replace mapped_srcs[{mapped_key}] = \"{resolved}\" in target `{target}` ({buck_path})"
                    ),
                    RemediationKind::ComprehensionRewrite => format!(
                        "rewrite target `{target}` to the cedar comprehension form (ROOT var, srcs=[], crate_root=ROOT+\"/src/lib.rs\", mapped_srcs comprehension + mapped_srcs[{mapped_key}]=\"{resolved}\") in {buck_path}"
                    ),
                }
            };
            Remediation {
                buck_path: buck_path.clone(),
                target: target.clone(),
                mapped_key: mapped_key.clone(),
                mapped_value: resolved.clone(),
                kind,
                applicable,
                note,
            }
        })
        .collect()
}

/// Read `target`'s `crate_root` from its BUCK and decide whether it is ROOT-prefixed (the deep form,
/// e.g. `ROOT + "/src/lib.rs"` or a literal containing the crate dir) vs a short package-relative
/// path (`src/lib.rs`). Returns None if the target or crate_root cannot be read.
fn buck_crate_root_is_root_prefixed(
    root: &Path,
    buck_path: &str,
    target: &str,
    crate_dir: &str,
) -> Option<bool> {
    let text = std::fs::read_to_string(root.join(buck_path)).ok()?;
    let doc = oya_buck_syntax_kernel::parse(&text).ok()?;
    let env = Env::from_doc(&doc);
    let call = find_target(&doc, None, target, &env)?;
    let crate_root = &call.kwarg("crate_root")?.value;
    // A `ROOT + "..."` concat or a literal already containing the crate dir is deep; a bare short
    // literal (`"src/lib.rs"`) is the escaping short form.
    if matches!(crate_root.expr, Expr::Plus(_)) {
        return Some(true);
    }
    let resolved = eval_string(crate_root, &env).unwrap_or_else(|| {
        crate_root
            .span
            .slice(&text)
            .trim()
            .trim_matches('"')
            .to_owned()
    });
    Some(resolved.starts_with(&format!("{crate_dir}/")) || resolved.starts_with(crate_dir))
}

/// Derive the mapped_srcs KEY for an asset at repo-relative `resolved`, given the consuming crate
/// dir. Returns (key, derivable). If the asset lives in a different package that exports it via an
/// `export_file`, the key is the `//pkg:name` label; else (same package) the crate-relative source.
fn derive_mapped_key(root: &Path, resolved: &str, crate_dir: &str) -> (String, bool) {
    // Find the asset's owning package = nearest ancestor dir (of the asset) holding a BUCK file.
    let asset_abs = root.join(resolved);
    let pkg_dir = match nearest_buck_dir(root, &asset_abs) {
        Some(d) => rel_to(root, &d),
        None => return (resolved.to_owned(), false),
    };
    let name = resolved.rsplit('/').next().unwrap_or(resolved);
    if pkg_dir == crate_dir {
        // Same package: map the source short path to itself (rare; usually glob already covers it).
        let short = resolved
            .strip_prefix(&format!("{crate_dir}/"))
            .unwrap_or(resolved);
        (short.to_owned(), true)
    } else {
        // Cross-package: reference the sibling package's export_file label.
        (format!("//{pkg_dir}:{name}"), true)
    }
}

/// The repo-relative path of the nearest ancestor `BUCK` for a repo-relative `.rs` file.
fn nearest_buck_path_rel(root: &Path, rs_rel: &str) -> Option<String> {
    let abs = root.join(rs_rel);
    let dir = nearest_buck_dir(root, &abs)?;
    Some(format!("{}/BUCK", rel_to(root, &dir)))
}

/// Apply a derived, applicable [`Remediation`] to a BUCK text by inserting (or augmenting) the
/// target's `mapped_srcs` with the `mapped_key -> mapped_value` entry, returning the patched text.
/// Pure over the BUCK text. Returns `Err` describing why it could not edit safely (the caller then
/// reports it as manual — the backstop). It NEVER edits the include literal (that would break cargo).
///
/// ADR-0549 migration: target binding, span computation, and comma placement are delegated to the
/// shared sound `oya-buck-syntax-kernel`, and the result is routed through its write-through
/// harness (reparse + the gate's semantic round-trip). Comment-bearing target blocks — which the
/// pre-kernel implementation REFUSED because its comma heuristic could emit a double comma — are
/// now edited soundly: the kernel reads comma positions from parsed spans, never trimmed text.
pub fn apply_remediation(buck_text: &str, rem: &Remediation) -> Result<String, String> {
    if !rem.applicable {
        return Err(rem.note.clone());
    }
    let doc = oya_buck_syntax_kernel::parse(buck_text)
        .map_err(|e| format!("BUCK text does not parse soundly ({e}); fix by hand"))?;
    let env = Env::from_doc(&doc);
    // Sound binding by the actual `name` kwarg (never first-occurrence substring match).
    let call = find_target(&doc, None, &rem.target, &env)
        .ok_or_else(|| format!("target `{}` not found in BUCK", rem.target))?;

    // ComprehensionRewrite: regenerate the whole target in the deep cedar form so the include
    // resolves to the mapped VALUE. Only safe for rust_library/rust_test/rust_binary whose
    // crate_root is a src/lib.rs|main.rs; anything else falls back to a manual report.
    let candidate = if rem.kind == RemediationKind::ComprehensionRewrite {
        let rewritten = rewrite_to_comprehension(buck_text, call, &env, rem)?;
        replace_span(buck_text, call.span, &rewritten).map_err(|e| e.to_string())?
    } else if let Some(mapped_arg) = call.kwarg("mapped_srcs") {
        match &mapped_arg.value.expr {
            Expr::Dict(dict) if dict.comprehension.is_none() => {
                // If the KEY is already present (mapped to the WRONG value — the original
                // FRIC-1781131000 defect shape), REPLACE that entry's value rather than inserting
                // a duplicate key (which buck2 rejects). Otherwise insert a new entry.
                let existing = dict.entries.iter().find(
                    |entry| matches!(&entry.key.expr, Expr::Str(key) if key == &rem.mapped_key),
                );
                match existing {
                    Some(entry) => replace_span(
                        buck_text,
                        entry.value.span,
                        &format!("\"{}\"", rem.mapped_value),
                    )
                    .map_err(|e| e.to_string())?,
                    None => insert_dict_entry(buck_text, dict, &rem.mapped_key, &rem.mapped_value)
                        .map_err(|e| e.to_string())?,
                }
            }
            Expr::Dict(_) => {
                return Err(
                    "mapped_srcs is a dict comprehension; add the entry to its variable assembly site by hand"
                        .to_owned(),
                );
            }
            Expr::Ident(_) => {
                return Err(
                    "mapped_srcs is not an inline dict (a variable?); add the entry to its definition by hand"
                        .to_owned(),
                );
            }
            _ => {
                return Err(
                    "mapped_srcs has a shape the sound parser does not model; add the entry by hand"
                        .to_owned(),
                );
            }
        }
    } else {
        // No mapped_srcs: add one as a new kwarg. The kernel supplies the separating comma from
        // PARSED spans, so trailing comments can no longer produce a double comma (the
        // FRIC-1781190000 corruption class this gate previously refused away).
        insert_kwarg(
            buck_text,
            call,
            &format!(
                "mapped_srcs = {{\n        \"{}\": \"{}\",\n    }}",
                rem.mapped_key, rem.mapped_value
            ),
        )
        .map_err(|e| e.to_string())?
    };

    // Self-validation via the shared write-through harness: reparse the candidate with the
    // kernel, then run this gate's semantic round-trip (target findable + injected value visible
    // through the REAL detector parse path). Refuse and return the pre-image on any failure —
    // never write corrupt output (enforcement-layering doctrine: structural impossibility).
    let mut registry = PreImageRegistry::new();
    guarded_rewrite("<buck>", buck_text, &candidate, &mut registry, |_, out| {
        validate_remediation_output(out, rem)
    })
    .map_err(|refusal| refusal.reason)
}

/// Real-parse validation: run the rewritten BUCK text through `parse_buck_targets` (the same path
/// the detector uses) and confirm the target is still present and well-formed. A shallow
/// findability check is not sufficient — the validator must confirm the full parser round-trip
/// succeeds, not just that a string token exists. Returns `Err` if the round-trip fails or the
/// target disappears; the caller then refuses the write. Pure over text; no filesystem access.
fn validate_remediation_output(out: &str, rem: &Remediation) -> Result<(), String> {
    // parse_buck_targets needs a crate_files slice for glob expansion; pass empty — we only need
    // the structural parse to succeed, not full glob expansion.
    let targets = parse_buck_targets(out, &[]);
    if targets.is_empty() {
        return Err(format!(
            "self-validation: parse_buck_targets returned no targets from rewritten BUCK — \
             output is structurally unparseable; refusing write to avoid corruption \
             (target: `{}`)",
            rem.target
        ));
    }
    let found = targets.iter().any(|t| t.name == rem.target);
    if !found {
        return Err(format!(
            "self-validation: target `{}` not found in parse_buck_targets output after rewrite \
             — rewritten BUCK is corrupt; refusing write",
            rem.target
        ));
    }
    // For MappedEntry: confirm the injected mapped_srcs destination VALUE is visible in the
    // reparsed target. This catches any edit that silently drops the mapped_srcs block even
    // while the target name remains findable.
    if rem.kind == RemediationKind::MappedEntry {
        let value_present = targets.iter().filter(|t| t.name == rem.target).any(|t| {
            t.mapped_dest.iter().any(|v| v == &rem.mapped_value)
                || t.mapped_dest
                    .iter()
                    .any(|v| normalize_rel(v) == normalize_rel(&rem.mapped_value))
        });
        if !value_present {
            return Err(format!(
                "self-validation: mapped_srcs value `{}` not found in reparsed target `{}` \
                 — rewritten output dropped the injected entry; refusing write to avoid corruption",
                rem.mapped_value, rem.target
            ));
        }
    }
    Ok(())
}

/// Rewrite a short-crate_root Rust target block into the deep cedar comprehension form, preserving
/// name/crate/visibility/deps and the existing srcs glob, so a cross-package `../` include resolves to
/// the mapped VALUE. Handles `rust_library`, `rust_test`, and `rust_binary` (the three kinds that
/// appear as covering targets). Returns `Err` (→ manual report) for any other kind. The include
/// literal is never touched. The emitted form mirrors the proven cloud-intelligence adapter.
fn rewrite_to_comprehension(
    buck_text: &str,
    call: &oya_buck_syntax_kernel::CallExpr,
    env: &Env,
    rem: &Remediation,
) -> Result<String, String> {
    let kind = match call.func.as_str() {
        kind @ ("rust_library" | "rust_test" | "rust_binary") => kind,
        _ => {
            return Err(
                "comprehension rewrite only supports rust_library/rust_test/rust_binary; \
             got a different kind — fix by hand"
                    .to_owned(),
            );
        }
    };
    let crate_dir = rem.buck_path.trim_end_matches("/BUCK").to_owned();
    let name = call
        .kwarg("name")
        .and_then(|arg| eval_string(&arg.value, env))
        .ok_or_else(|| "could not read target name".to_owned())?;
    let crate_field = call
        .kwarg("crate")
        .and_then(|arg| eval_string(&arg.value, env));
    let crate_root_expr = call
        .kwarg("crate_root")
        .and_then(|arg| eval_string(&arg.value, env))
        .ok_or_else(|| "could not read crate_root".to_owned())?;
    if !(crate_root_expr.ends_with("/lib.rs") || crate_root_expr.ends_with("/main.rs")) {
        return Err("crate_root is not a src/lib.rs|main.rs; rewrite by hand".to_owned());
    }
    // Raw expression text for fields carried through verbatim (span-sliced, comment-safe).
    let raw = |field: &str| -> Option<String> {
        call.kwarg(field)
            .map(|arg| arg.value.span.slice(buck_text).to_owned())
    };
    let srcs_expr = raw("srcs").unwrap_or_else(|| "[]".to_owned());
    let srcs_glob = if srcs_expr.trim_start().starts_with("glob(") {
        srcs_expr.trim().to_owned()
    } else {
        // Default to the standard asset glob if srcs was empty/list — preserves all sources.
        "glob([\"src/**/*.rs\", \"**/*.cedar\", \"**/*.sql\", \"**/*.json\", \"**/*.toml\", \"**/*.yaml\", \"**/*.yml\", \"**/*.proto\", \"**/*.html\", \"**/*.css\", \"**/*.txt\"])".to_owned()
    };
    let visibility = raw("visibility").unwrap_or_else(|| "[\"PUBLIC\"]".to_owned());
    let deps = raw("deps").unwrap_or_else(|| "[]".to_owned());
    let crate_decl = match crate_field {
        Some(c) => format!("    crate = \"{c}\",\n"),
        None => String::new(),
    };

    // ROOT var = crate dir; comprehension maps every src to ROOT/<src>; explicit entry maps the
    // cross-package asset to its include-relative deep destination (rem.mapped_value).
    let root_var = "ASSET_ROOT";
    let srcs_var = "ASSET_SRCS";
    let mapped_var = "ASSET_MAPPED_SRCS";
    Ok(format!(
        "# ADR-0545 embedded-asset hermeticity --fix: rewritten to the cedar comprehension form so the\n\
         # cross-package include resolves to its include-relative sandbox destination. Include literal\n\
         # unchanged (editing it would break cargo).\n\
         {root_var} = \"{crate_dir}\"\n\
         {srcs_var} = {srcs_glob}\n\
         {mapped_var} = {{src: {root_var} + \"/\" + src for src in {srcs_var}}}\n\
         {mapped_var}[\"{key}\"] = \"{value}\"\n\
         \n\
         {kind}(\n\
         \x20   name = \"{name}\",\n\
         \x20   srcs = [],\n\
         {crate_decl}\
         \x20   crate_root = {root_var} + \"/{root_suffix}\",\n\
         \x20   visibility = {visibility},\n\
         \x20   mapped_srcs = {mapped_var},\n\
         \x20   deps = {deps},\n\
         )",
        key = rem.mapped_key,
        value = rem.mapped_value,
        root_suffix = crate_root_expr.trim_start_matches(&format!("{crate_dir}/")),
    ))
}

/// Collect every unmapped site from an `observed` value and derive remediations — one per covering
/// target per site — so every target that must be patched is represented. Read-only.
pub fn derive_all_remediations(root: &Path, observed: &Value) -> Vec<Remediation> {
    observed
        .get("sites")
        .and_then(Value::as_array)
        .map(|sites| {
            sites
                .iter()
                .flat_map(|s| derive_remediation(root, s))
                .collect()
        })
        .unwrap_or_default()
}

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
        assert_eq!(
            normalize_rel("crate/src/../policy/x.cedar"),
            "crate/policy/x.cedar"
        );
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
        assert_eq!(
            resolve_include("crate/src", "bundled/x.json"),
            "crate/src/bundled/x.json"
        );
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
        assert!(
            sites[0].literal.is_none(),
            "concat! arg must be non-literal"
        );
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
    fn ternary_srcs_demotes_target_to_unparseable_not_a_silent_narrow() {
        // Reviewer LOW closure: `srcs = [...] if c else [...]` parses the first list plus an
        // opaque tail. The target must be UNPARSEABLE (visible skip), never silently narrowed
        // to the first branch.
        let buck = "rust_library(\n    name = \"c\",\n    srcs = [\"a.rs\"] if c else [\"b.rs\"],\n    crate_root = \"src/lib.rs\",\n)\n";
        let targets = parse_buck_targets(buck, &[]);
        assert_eq!(targets.len(), 1, "{targets:?}");
        assert!(
            targets[0].unparseable,
            "a ternary srcs must demote the target to unparseable: {targets:?}"
        );
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
        let values = resolve_mapped_var(
            buck,
            "ADAPTER_MAPPED_SRCS",
            &string_vars,
            &glob_vars,
            &files,
        );
        assert!(
            values
                .iter()
                .any(|v| v == "cloud/ci/adapter/policy/x.cedar"),
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
            &observed(vec![
                json!({"key":"a.rs:1","status":"unmapped","detail":"d"}),
            ]),
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == "embedded_asset_unmapped_include" && f.key == "a.rs:1")
        );
    }

    #[test]
    fn skip_codes_are_surfaced_but_do_not_flip_the_verdict() {
        // Skips are present in the keyed findings (so they get baselined) but, because they are NOT
        // in VIOLATION_CODES, the Report verdict stays Green. This is the fail-SAFE posture: a site
        // the gate cannot resolve is visible, never a silent pass nor a false RED.
        let input = observed(vec![
            json!({"key":"a.rs:1","status":"skip_non_literal_argument"}),
            json!({"key":"b.rs:2","status":"skip_no_owning_target"}),
            json!({"key":"c.rs:3","status":"skip_build_output_path"}),
            json!({"key":"d.rs:4","status":"skip_buck_unparseable"}),
        ]);
        let findings = evaluate_keyed(&policy(), &input);
        assert_eq!(findings.len(), 4, "every skip is surfaced: {findings:#?}");
        for code in [
            "skip_non_literal_argument",
            "skip_no_owning_target",
            "skip_build_output_path",
            "skip_buck_unparseable",
        ] {
            assert!(findings.iter().any(|f| f.code == code), "missing {code}");
        }
        let report = evaluate(&policy(), &input);
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "skips must not flip the verdict"
        );
        assert!(
            report.violations.is_empty(),
            "no blocking codes: {:?}",
            report.violations
        );
    }

    #[test]
    fn unmapped_plus_skip_is_red_but_skip_is_not_a_violation() {
        let input = observed(vec![
            json!({"key":"T::a.rs::x","status":"unmapped"}),
            json!({"key":"b.rs:2","status":"skip_non_literal_argument"}),
        ]);
        let report = evaluate(&policy(), &input);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(
            report.violations,
            ["embedded_asset_unmapped_include".to_owned()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            "only the blocking code is a violation; the skip is excluded"
        );
    }

    #[test]
    fn gate_id_mismatch_fails_closed() {
        let mut bad = policy();
        bad["gate_id"] = json!("wrong");
        let report = evaluate(&bad, &observed(vec![]));
        assert!(
            report
                .violations
                .contains("embedded_asset_policy_gate_id_mismatch")
        );
        assert_eq!(report.verdict, Verdict::Red);
    }

    #[test]
    fn evaluate_violations_are_blocking_codes_only() {
        let input = observed(vec![
            json!({"key":"T::a.rs::x","status":"unmapped"}),
            json!({"key":"b.rs:2","status":"skip_non_literal_argument"}),
        ]);
        let blocking: BTreeSet<String> = evaluate_keyed(&policy(), &input)
            .into_iter()
            .map(|f| f.code)
            .filter(|c| VIOLATION_CODES.contains(&c.as_str()))
            .collect();
        assert_eq!(evaluate(&policy(), &input).violations, blocking);
    }

    #[test]
    fn violation_and_skip_codes_are_disjoint_and_emittable() {
        // Drift guard: the two code sets must not overlap, and the evaluator must only ever emit
        // codes declared in one of them (review LOW: keep the const the single source of truth).
        for v in VIOLATION_CODES {
            assert!(!SKIP_CODES.contains(&v), "{v} must not be in both sets");
        }
        let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().chain(SKIP_CODES).collect();
        let input = observed(vec![
            json!({"key":"T::a::x","status":"unmapped"}),
            json!({"key":"b:1","status":"skip_non_literal_argument"}),
            json!({"key":"c:2","status":"skip_build_output_path"}),
        ]);
        let mut bad = policy();
        bad["gate_id"] = json!("wrong");
        for f in evaluate_keyed(&bad, &input) {
            assert!(
                declared.contains(f.code.as_str()),
                "emitted undeclared code {}",
                f.code
            );
        }
    }

    // ---- anti-KEY regression + explicit-override-wins (consensus-mandated) -----

    #[test]
    fn membership_is_against_values_not_keys() {
        // The refuted alternative: matching mapped_srcs KEYS would PASS the original defect and
        // false-RED cedar. The destination set must contain the VALUE, and a resolved path equal to
        // the KEY (a `//path:name` label) must NOT be considered a member.
        let buck = r#"
ROOT = "cloud/ci/adapter"
SRCS = glob(["src/**/*.rs"])
MAPPED = {src: ROOT + "/" + src for src in SRCS}
MAPPED["//cloud/ci/policy:x.cedar"] = ROOT + "/policy/x.cedar"

rust_library(
    name = "adapter",
    srcs = [],
    crate_root = ROOT + "/src/lib.rs",
    mapped_srcs = MAPPED,
)
"#;
        let files = vec!["src/lib.rs".to_owned()];
        let string_vars = top_level_string_vars(buck);
        let glob_vars = top_level_glob_vars(buck);
        let values = resolve_mapped_var(buck, "MAPPED", &string_vars, &glob_vars, &files);
        assert!(
            values
                .iter()
                .any(|v| v == "cloud/ci/adapter/policy/x.cedar"),
            "VALUE present"
        );
        assert!(
            !values.iter().any(|v| v == "//cloud/ci/policy:x.cedar"),
            "the mapped_srcs KEY must never enter the destination set: {values:?}"
        );
    }

    #[test]
    fn explicit_assignment_value_resolves_independently_of_comprehension() {
        // explicit MAPPED["k"]=v must produce its own VALUE even though the comprehension also runs.
        let buck = r#"
ROOT = "p/q"
SRCS = glob(["src/**/*.rs"])
MAPPED = {src: ROOT + "/" + src for src in SRCS}
MAPPED["//p/policy:y.cedar"] = ROOT + "/policy/y.cedar"

rust_library(name = "t", srcs = [], crate_root = ROOT + "/src/lib.rs", mapped_srcs = MAPPED)
"#;
        let files = vec!["src/lib.rs".to_owned()];
        let sv = top_level_string_vars(buck);
        let gv = top_level_glob_vars(buck);
        let values = resolve_mapped_var(buck, "MAPPED", &sv, &gv, &files);
        assert!(
            values.iter().any(|v| v == "p/q/policy/y.cedar"),
            "explicit value: {values:?}"
        );
        assert!(
            values.iter().any(|v| v == "p/q/src/lib.rs"),
            "comprehension value: {values:?}"
        );
    }

    // ---- scanner context-awareness (the self-flag correctness fix) ------------

    #[test]
    fn macro_text_inside_strings_and_comments_is_not_a_site() {
        let src = r#"
            // example: include_str!("../in/comment.json") must be ignored
            /* block include_bytes!("../in/block.json") ignored too */
            const MACROS: [&str; 1] = ["include_str!"];
            let s = "include_bytes!(\"in/string.json\")";
            const REAL: &str = include_str!("real/asset.json");
        "#;
        let sites = extract_include_sites(src);
        assert_eq!(sites.len(), 1, "only the real code site counts: {sites:#?}");
        assert_eq!(sites[0].literal.as_deref(), Some("real/asset.json"));
    }

    #[test]
    fn raw_string_with_macro_text_is_not_a_site() {
        let src = "const D: &str = r#\"include_str!(\\\"x\\\")\"#; const R: &str = include_str!(\"y.json\");";
        let sites = extract_include_sites(src);
        assert_eq!(sites.len(), 1, "raw string content ignored: {sites:#?}");
        assert_eq!(sites[0].literal.as_deref(), Some("y.json"));
    }

    #[test]
    fn is_out_of_scope_classifies_build_outputs() {
        let exts: BTreeSet<String> = ["cedar", "json"].into_iter().map(str::to_owned).collect();
        assert!(is_out_of_scope(
            "../../../out/svc.elf",
            &["../../../out/".to_owned()],
            &exts
        ));
        // a .cedar under out/ is still an embedded asset -> NOT out of scope.
        assert!(!is_out_of_scope(
            "../../../out/policy.cedar",
            &["../../../out/".to_owned()],
            &exts
        ));
    }

    // ---- auto-remediation (`--fix`) -------------------------------------------

    #[test]
    fn apply_remediation_adds_entry_to_existing_inline_dict() {
        // A target with an inline mapped_srcs dict gains the derived entry; the include literal is
        // never touched. This is the FRIC-1781131000 broken-mapping -> --fix -> mapped shape.
        let buck = "rust_library(\n    name = \"adapter\",\n    srcs = [],\n    crate_root = \"x/src/lib.rs\",\n    mapped_srcs = {\n        \"//x/policy:wrong.cedar\": \"x/policy/wrong.cedar\",\n    },\n)\n";
        let rem = Remediation {
            buck_path: "x/BUCK".to_owned(),
            target: "adapter".to_owned(),
            mapped_key: "//y/policy:right.cedar".to_owned(),
            mapped_value: "y/policy/right.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem).expect("apply");
        assert!(
            patched.contains("\"//y/policy:right.cedar\": \"y/policy/right.cedar\""),
            "patched dict must contain the new entry: {patched}"
        );
        // The original entry and the include-free structure remain intact.
        assert!(patched.contains("\"//x/policy:wrong.cedar\""));
        assert!(patched.contains("name = \"adapter\""));
    }

    #[test]
    fn apply_remediation_replaces_wrong_value_for_existing_key() {
        // The FRIC-1781131000 defect shape: the KEY exists but is mapped to the WRONG sandbox path.
        // --fix must REPLACE the value, never insert a duplicate key (buck2 rejects duplicate keys).
        let buck = "rust_library(\n    name = \"adapter\",\n    crate_root = \"x/src/lib.rs\",\n    mapped_srcs = {\n        \"//y/policy:p.cedar\": \"policy/p.cedar\",\n    },\n)\n";
        let rem = Remediation {
            buck_path: "x/BUCK".to_owned(),
            target: "adapter".to_owned(),
            mapped_key: "//y/policy:p.cedar".to_owned(),
            mapped_value: "y/policy/p.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem).expect("apply");
        assert!(
            patched.contains("\"//y/policy:p.cedar\": \"y/policy/p.cedar\""),
            "value replaced: {patched}"
        );
        assert!(
            !patched.contains("\"policy/p.cedar\""),
            "old wrong value must be gone: {patched}"
        );
        // Exactly one occurrence of the key (no duplicate).
        assert_eq!(
            patched.matches("//y/policy:p.cedar").count(),
            1,
            "no duplicate key: {patched}"
        );
    }

    #[test]
    fn apply_remediation_creates_mapped_srcs_when_absent() {
        let buck = "rust_library(\n    name = \"t\",\n    srcs = glob([\"src/**/*.rs\"]),\n    crate_root = \"src/lib.rs\",\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "t".to_owned(),
            mapped_key: "//q:a.cedar".to_owned(),
            mapped_value: "q/a.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem).expect("apply");
        assert!(
            patched.contains("mapped_srcs = {"),
            "must add a mapped_srcs dict: {patched}"
        );
        assert!(patched.contains("\"//q:a.cedar\": \"q/a.cedar\""));
    }

    #[test]
    fn non_applicable_remediation_is_reported_not_applied() {
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "t".to_owned(),
            mapped_key: String::new(),
            mapped_value: "q/a.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: false,
            note: "asset not found on disk; manual fix required".to_owned(),
        };
        let err = apply_remediation("rust_library(name = \"t\")", &rem).unwrap_err();
        assert!(
            err.contains("manual fix required"),
            "must surface the manual note: {err}"
        );
    }

    #[test]
    fn derive_remediation_ignores_non_unmapped_rows() {
        let resolved = json!({"status": "resolved", "resolved": "a", "rs": "x.rs"});
        assert!(derive_remediation(Path::new("/nonexistent"), &resolved).is_empty());
    }

    #[test]
    fn comprehension_rewrite_produces_deep_cedar_form() {
        // The escaping short-crate_root shape (FRIC-1781131000): --fix rewrites the whole rust_library
        // to the deep comprehension form so the cross-package `../` include resolves to the VALUE.
        let buck = "rust_library(\n    name = \"adapter\",\n    srcs = glob([\"src/**/*.rs\", \"**/*.cedar\"]),\n    crate = \"adapter_crate\",\n    crate_root = \"src/lib.rs\",\n    visibility = [\"PUBLIC\"],\n    mapped_srcs = {\n        \"//svc/policy:p.cedar\": \"policy/p.cedar\",\n    },\n    deps = [\n        \"third-party//:cedar-policy\",\n    ],\n)\n";
        let rem = Remediation {
            buck_path: "svc/crates/adapter/BUCK".to_owned(),
            target: "adapter".to_owned(),
            mapped_key: "//svc/policy:p.cedar".to_owned(),
            mapped_value: "svc/policy/p.cedar".to_owned(),
            kind: RemediationKind::ComprehensionRewrite,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem).expect("rewrite");
        assert!(
            patched.contains("ASSET_ROOT = \"svc/crates/adapter\""),
            "ROOT var: {patched}"
        );
        assert!(
            patched.contains("crate_root = ASSET_ROOT + \"/src/lib.rs\""),
            "deep crate_root: {patched}"
        );
        assert!(
            patched.contains("{src: ASSET_ROOT + \"/\" + src for src in ASSET_SRCS}"),
            "comprehension: {patched}"
        );
        assert!(
            patched
                .contains("ASSET_MAPPED_SRCS[\"//svc/policy:p.cedar\"] = \"svc/policy/p.cedar\""),
            "explicit deep value: {patched}"
        );
        assert!(patched.contains("srcs = [],"), "srcs emptied: {patched}");
        assert!(
            patched.contains("crate = \"adapter_crate\""),
            "crate preserved: {patched}"
        );
        assert!(
            !patched.contains("crate_root = \"src/lib.rs\""),
            "short crate_root removed: {patched}"
        );
        assert!(
            !patched.contains("\"policy/p.cedar\""),
            "wrong crate-local value removed: {patched}"
        );
    }

    #[test]
    fn comprehension_rewrite_refuses_non_library_targets() {
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "b".to_owned(),
            mapped_key: "//q:a".to_owned(),
            mapped_value: "q/a".to_owned(),
            kind: RemediationKind::ComprehensionRewrite,
            applicable: true,
            note: String::new(),
        };
        // rust_binary is now supported by ComprehensionRewrite; use an unsupported kind (rust_proc_macro)
        // to confirm the guard still refuses unknown kinds. We test rust_binary support separately.
        // Re-purpose this fixture: pass a proc-macro-shaped block (unsupported kind).
        let buck_proc = "rust_proc_macro(\n    name = \"b\",\n    srcs = glob([\"src/**/*.rs\"]),\n    crate_root = \"src/main.rs\",\n)\n";
        assert!(
            apply_remediation(buck_proc, &rem).is_err(),
            "proc_macro must be refused"
        );
    }

    // ---- Finding 1 fix: no-trailing-comma + self-validation -----------------

    #[test]
    fn apply_remediation_no_trailing_comma_creates_valid_mapped_srcs() {
        // A target whose last field lacks a trailing comma (valid Starlark). Before the fix, the
        // fixer emitted `mapped_srcs` with no separating comma — buck2 parse error. After the fix,
        // the output is parseable and the self-validation reparse confirms the target is intact.
        let buck = "rust_library(\n    name = \"lib\",\n    srcs = glob([\"src/**/*.rs\"]),\n    crate_root = \"src/lib.rs\"\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "lib".to_owned(),
            mapped_key: "//q:asset.cedar".to_owned(),
            mapped_value: "q/asset.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched =
            apply_remediation(buck, &rem).expect("must not corrupt on no-trailing-comma input");
        // The output must contain a comma separating the last existing field from mapped_srcs.
        assert!(
            patched.contains(",\n    mapped_srcs"),
            "separator comma required before mapped_srcs: {patched}"
        );
        assert!(
            patched.contains("\"//q:asset.cedar\": \"q/asset.cedar\""),
            "entry present: {patched}"
        );
        // Self-validation: reparsing must succeed (target block findable + key visible).
        // apply_remediation itself runs validate_remediation_output; if we got Ok above, it passed.
    }

    #[test]
    fn apply_remediation_trailing_comment_block_is_edited_soundly() {
        // ADR-0549 gap closure (RED→GREEN): `deps = [],  # trailing comment` before `)` is the
        // exact probe that broke the v1 needs_comma heuristic and forced the pre-kernel comment
        // guard to REFUSE every comment-bearing block. The sound kernel reads comma positions
        // from parsed spans, so this block is now edited correctly — no double comma, comment
        // preserved, full parser round-trip green.
        let buck = "rust_library(\n    name = \"lib\",\n    srcs = [],\n    crate_root = \"src/lib.rs\",\n    deps = [],  # trailing comment\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "lib".to_owned(),
            mapped_key: "//q:a.cedar".to_owned(),
            mapped_value: "q/a.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem)
            .expect("comment-bearing block must now be edited soundly, not refused");
        assert!(
            !patched.contains(",,"),
            "no double comma allowed: {patched}"
        );
        assert!(
            patched.contains("# trailing comment"),
            "comment preserved: {patched}"
        );
        assert!(
            patched.contains("\"//q:a.cedar\": \"q/a.cedar\""),
            "entry present: {patched}"
        );
        // Full round-trip through the real detector parse path.
        let targets = parse_buck_targets(&patched, &[]);
        let lib = targets
            .iter()
            .find(|t| t.name == "lib")
            .expect("target intact");
        assert!(
            lib.mapped_dest.iter().any(|v| v == "q/a.cedar"),
            "injected value visible after reparse: {targets:?}"
        );
    }

    #[test]
    fn apply_remediation_self_validation_refuses_corrupt_output() {
        // Construct a Remediation whose target name will not be found after patching
        // (simulating a corrupt rewrite) by using a target name that does not exist in the BUCK
        // text at all — apply_remediation must return Err, never write corrupt content.
        let buck = "rust_library(\n    name = \"real\",\n    srcs = [],\n    crate_root = \"src/lib.rs\",\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "ghost".to_owned(), // does not exist -> self-validation must refuse
            mapped_key: "//q:a.cedar".to_owned(),
            mapped_value: "q/a.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let result = apply_remediation(buck, &rem);
        assert!(
            result.is_err(),
            "self-validation must refuse when target not found"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("ghost") || msg.contains("not found"),
            "error must name the target or say not found: {msg}"
        );
    }

    // ---- Finding 2 fix: multibyte chars in target block ---------------------

    #[test]
    fn multibyte_comment_in_target_block_parses_and_binds_fields() {
        // An em-dash (U+2014, 3 UTF-8 bytes) in a comment inside the target block must not
        // cause a slice-panic or shift field binding (the pre-kernel find_top_level fixture,
        // re-expressed through the sound parser).
        let buck = "rust_library(\n    # house style — em-dash comment\n    name = \"t\",\n    srcs = [],\n)\n";
        let targets = parse_buck_targets(buck, &[]);
        assert_eq!(targets.len(), 1, "must parse despite em-dash: {targets:?}");
        assert_eq!(
            targets[0].name, "t",
            "name field bound correctly: {targets:?}"
        );
    }

    #[test]
    fn multibyte_comment_in_comprehension_file_resolves() {
        // The pre-kernel find_top_level_keyword multibyte fixture, re-expressed: an em-dash
        // comment near a comprehension must not panic or break resolution.
        let buck = "ROOT = \"p\"  # em-dash — here\nSRCS = glob([\"src/**/*.rs\"])\nM = {src: ROOT + \"/\" + src for src in SRCS}\n";
        let sv = top_level_string_vars(buck);
        let gv = top_level_glob_vars(buck);
        let files = vec!["src/lib.rs".to_owned()];
        let values = resolve_mapped_var(buck, "M", &sv, &gv, &files);
        assert_eq!(values, vec!["p/src/lib.rs".to_owned()], "{values:?}");
    }

    #[test]
    fn apply_remediation_comment_block_with_missing_comma_is_edited_soundly() {
        // ADR-0549 gap closure (RED→GREEN): a comment-bearing block whose last field ALSO lacks
        // a trailing comma — the compound shape the pre-kernel guard refused outright. The
        // kernel attaches the separating comma to the parsed last-value span (never after the
        // comment), edits soundly, and the round-trip validates.
        let buck = "rust_library(\n    # em-dash \u{2014} style\n    name = \"lib\",\n    srcs = [],\n    crate_root = \"src/lib.rs\"\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "lib".to_owned(),
            mapped_key: "//q:a.cedar".to_owned(),
            mapped_value: "q/a.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem)
            .expect("comment + missing-comma block must be edited soundly");
        assert!(
            patched.contains("\"src/lib.rs\","),
            "comma attached to the value: {patched}"
        );
        assert!(!patched.contains(",,"), "no double comma: {patched}");
        assert!(
            patched.contains("# em-dash \u{2014} style"),
            "comment preserved: {patched}"
        );
        let targets = parse_buck_targets(&patched, &[]);
        assert!(
            targets
                .iter()
                .any(|t| t.name == "lib" && t.mapped_dest.iter().any(|v| v == "q/a.cedar")),
            "round-trip green: {targets:?}"
        );
    }

    #[test]
    fn apply_remediation_multibyte_without_comment_succeeds() {
        // A target block with a multibyte char in a string value (not a comment) must still
        // be patched correctly — the comment guard must not trigger for in-string multibyte.
        let buck = "rust_library(\n    name = \"lib\",\n    srcs = [],\n    crate_root = \"src/lib.rs\",\n    mapped_srcs = {\n        \"//q:caf\u{00e9}.cedar\": \"q/caf\u{00e9}.cedar\",\n    },\n)\n";
        let rem = Remediation {
            buck_path: "p/BUCK".to_owned(),
            target: "lib".to_owned(),
            mapped_key: "//r:new.cedar".to_owned(),
            mapped_value: "r/new.cedar".to_owned(),
            kind: RemediationKind::MappedEntry,
            applicable: true,
            note: String::new(),
        };
        let patched = apply_remediation(buck, &rem).expect("no comment — must succeed");
        assert!(
            patched.contains("\"//r:new.cedar\": \"r/new.cedar\""),
            "entry injected: {patched}"
        );
    }

    // ---- Finding 3 fix: multi-target patching --------------------------------

    #[test]
    fn derive_remediation_returns_one_entry_per_covering_target() {
        // An observed site with two covering targets (lib + unittest) must yield two Remediations
        // so derive_all_remediations patches both BUCK targets, preventing gate-GREEN/build-RED.
        // Use a real tempdir with a BUCK file so nearest_buck_path_rel can bind the rs path.
        let tmp = std::env::temp_dir().join("oya-test-multi-target");
        let svc_src = tmp.join("svc").join("src");
        std::fs::create_dir_all(&svc_src).unwrap();
        std::fs::write(tmp.join("svc").join("BUCK"), "").unwrap();
        std::fs::write(svc_src.join("lib.rs"), "").unwrap();
        // Asset does NOT need to exist on disk; applicable will be false but remediations are produced.
        let site = json!({
            "status": "unmapped",
            "resolved": "svc/policy/asset.cedar",
            "rs": "svc/src/lib.rs",
            "targets": ["svc-lib", "svc-lib-unittest"],
            "macro": "include_str",
            "literal": "../policy/asset.cedar"
        });
        let rems = derive_remediation(&tmp, &site);
        assert_eq!(
            rems.len(),
            2,
            "one remediation per covering target: {rems:?}"
        );
        let targets: Vec<&str> = rems.iter().map(|r| r.target.as_str()).collect();
        assert!(
            targets.contains(&"svc-lib"),
            "first target present: {targets:?}"
        );
        assert!(
            targets.contains(&"svc-lib-unittest"),
            "second target present: {targets:?}"
        );
        // All entries are non-applicable (asset not on disk), so --fix reports them as manual.
        assert!(
            rems.iter().all(|r| !r.applicable),
            "non-applicable without asset on disk"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn derive_remediation_empty_for_non_unmapped() {
        let resolved = json!({"status": "resolved", "resolved": "a", "rs": "x.rs"});
        assert!(derive_remediation(Path::new("/nonexistent"), &resolved).is_empty());
    }
}
