//! # cloud-ci-canonical-json (ADR-0546)
//!
//! A canonical-JSON determinism gate. Founder R0 hermetic-output directive: the same logical JSON
//! content must serialize to the same bytes on every run, on any repo. Converts FRIC-1781130000 —
//! a lane silently re-encoded `specs/root-hub-pointers.json` from escaped-unicode (`\uXXXX`,
//! `ensure_ascii=true`) to literal UTF-8 (`→ µ § —`), producing ~30 lines of churn on content
//! unrelated to the lane's one intentional pointer addition. That is a hermetic-output failure:
//! same logical content, non-deterministic bytes. Such drift pollutes diffs, risks merge conflicts,
//! and defeats cross-artifact agreement. This gate makes non-deterministic JSON serialization
//! impossible to ship and ships a one-command fixer.
//!
//! ## Born pack-shaped (R0)
//! The crate is a NEUTRAL engine. All repo-specifics — the governed roots, the canonical-form
//! parameters (`ensure_ascii`, `indent_width`, `sort_keys`, `trailing_newline`, `newline`,
//! `utf8_bom`), and the exclusions — are DATA in `canonical-json-policy.json`. Nothing oyatie-
//! specific is hardcoded in Rust; a different repo adopts the gate by repointing the policy at its
//! own roots. The kernel fixes only the canonical-form *algorithm*, not any repo path.
//!
//! ## Why a self-contained lexical canonicalizer (NOT serde_json::to_string_pretty)
//! `serde_json`'s `preserve_order` and `arbitrary_precision` features are reindeer-unioned ON
//! workspace-wide under buck2; routing the canonical bytes through `serde_json` would make this
//! gate's output depend on a build-system feature union it does not control, the exact silent-byte-
//! drift class it polices. So the canonical path is a hand-written lexer → CST → formatter with ZERO
//! `serde_json` in it. `serde_json` is used ONLY for the live-corpus enumeration's parse-validity
//! pre-check and in tests — never to *produce* canonical bytes. This mirrors the faces serializer's
//! intent (`accounting-registry::to_canonical_json`: 2-space, literal UTF-8, trailing newline) while
//! being independent of serde feature unions.
//!
//! ## Kernel contract
//! - [`canonicalize`] `(bytes, &CanonicalForm) -> Result<String, CanonError>` is the PURE core,
//!   shared by the gate and the fixer. Check == fix by construction (`cargo fmt --check` precedent):
//!   a file is canonical iff its bytes already equal `canonicalize` of themselves.
//! - [`collect_observed`] `(root, policy) -> Observed` is the only I/O (read-only filesystem walk of
//!   the governed roots).
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it keys every finding by the offending file path.
//! - [`evaluate`] is the bare-code projection (Green/Red verdict).
//!
//! ## Ratchet semantics
//! A finding per non-canonical / unparseable / duplicate-key file, keyed by path. The live corpus is
//! born-blocking green after this PR's 7-file fix (zero baseline), so any NEW non-canonical governed
//! JSON fails closed. Appending a *canonical* JSON file never fails the gate.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `json_not_canonical`  — committed bytes != `canonicalize(bytes)` (the FRIC drift class; fixable).
//! - `json_parse_error`    — not valid JSON under the canonical grammar (incl. lone surrogates,
//!   NaN/Infinity, leading-zero numbers, trailing data).
//! - `json_duplicate_key`  — an object has two members with the same key (canonical form is undefined;
//!   the fixer refuses rather than silently drop one).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The gate id, matching the buck2 target + the oya-ci registry id + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-canonical-json";

/// The three blocking violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 3] = [
    "json_not_canonical",
    "json_parse_error",
    "json_duplicate_key",
];

/// A recursion guard so a pathological deeply-nested document errors instead of overflowing the
/// stack (the no-panic doctrine forbids an abort). Far deeper than any real config file.
const MAX_DEPTH: usize = 256;

// ───────────────────────────── canonical form (policy DATA) ─────────────────────────────

/// The canonical-form parameters, parsed from the policy `canonical_form` object. Every field is
/// DATA — the kernel hardcodes no values, so an adopting repo settles its own dialect in the policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalForm {
    /// `true` => non-ASCII escaped as `\uXXXX` (surrogate pairs for astral); `false` => literal UTF-8.
    pub ensure_ascii: bool,
    /// Spaces per indent level (pretty-print width).
    pub indent_width: usize,
    /// `true` => object keys emitted in sorted order (Unicode-scalar / UTF-8-byte collation, via
    /// `BTreeMap<&str,_>`); `false` => source order preserved.
    pub sort_keys: bool,
    /// `true` => exactly one trailing line terminator at end of file.
    pub trailing_newline: bool,
    /// The line terminator the canonical form emits (every `\n` the formatter produces). DATA so an
    /// adopting repo can choose CRLF; oyatie pins LF.
    pub newline: Newline,
    /// `true` => the canonical form begins with a UTF-8 BOM; `false` => no BOM (a committed BOM is
    /// drift). DATA so an adopting repo can require a BOM; oyatie pins false.
    pub utf8_bom: bool,
}

/// The line terminator a canonical file uses. Pack DATA via `canonical_form.newline` ("lf"|"crlf").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Newline {
    Lf,
    Crlf,
}

impl Newline {
    fn as_str(self) -> &'static str {
        match self {
            Newline::Lf => "\n",
            Newline::Crlf => "\r\n",
        }
    }

    fn from_policy_str(value: &str) -> Option<Self> {
        match value {
            "lf" => Some(Newline::Lf),
            "crlf" => Some(Newline::Crlf),
            _ => None,
        }
    }
}

impl Default for CanonicalForm {
    /// The repo's settled dialect (ADR-0546): literal UTF-8, 2-space, source-order, LF, trailing
    /// newline, no BOM — consistent with the faces serializer. Used only as a fallback; the live gate
    /// reads policy DATA.
    fn default() -> Self {
        Self {
            ensure_ascii: false,
            indent_width: 2,
            sort_keys: false,
            trailing_newline: true,
            newline: Newline::Lf,
            utf8_bom: false,
        }
    }
}

impl CanonicalForm {
    /// Parse the `canonical_form` object out of a policy value. Missing fields fall back to the
    /// settled ADR-0546 defaults so a terse policy still expresses the intended dialect.
    pub fn from_policy(policy: &Value) -> Self {
        let form = policy.get("canonical_form");
        let read_bool = |key: &str, fallback: bool| -> bool {
            form.and_then(|f| f.get(key))
                .and_then(Value::as_bool)
                .unwrap_or(fallback)
        };
        let read_usize = |key: &str, fallback: usize| -> usize {
            form.and_then(|f| f.get(key))
                .and_then(Value::as_u64)
                .map(|v| v as usize)
                .unwrap_or(fallback)
        };
        let d = CanonicalForm::default();
        let newline = form
            .and_then(|f| f.get("newline"))
            .and_then(Value::as_str)
            .and_then(Newline::from_policy_str)
            .unwrap_or(d.newline);
        CanonicalForm {
            ensure_ascii: read_bool("ensure_ascii", d.ensure_ascii),
            indent_width: read_usize("indent_width", d.indent_width),
            sort_keys: read_bool("sort_keys", d.sort_keys),
            trailing_newline: read_bool("trailing_newline", d.trailing_newline),
            newline,
            utf8_bom: read_bool("utf8_bom", d.utf8_bom),
        }
    }
}

// ───────────────────────────── canonical errors ─────────────────────────────

/// Why a byte sequence cannot be canonicalized. Returned instead of panicking so the caller (the
/// gate / the fixer) maps it to a finding or a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    /// The bytes are not valid JSON under the canonical grammar (with a human-readable reason).
    Parse(String),
    /// An object carried two members with the same key — the canonical form is undefined.
    DuplicateKey(String),
}

impl std::fmt::Display for CanonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanonError::Parse(reason) => write!(f, "json parse error: {reason}"),
            CanonError::DuplicateKey(key) => write!(f, "duplicate object key: {key}"),
        }
    }
}

impl std::error::Error for CanonError {}

impl CanonError {
    /// The violation code this error maps to in the gate.
    pub fn code(&self) -> &'static str {
        match self {
            CanonError::Parse(_) => "json_parse_error",
            CanonError::DuplicateKey(_) => "json_duplicate_key",
        }
    }
}

// ───────────────────────────── CST (concrete-ish JSON tree) ─────────────────────────────

/// A minimal JSON tree. Strings hold the DECODED scalar so the formatter re-escapes under the
/// configured `ensure_ascii`. Numbers hold the SOURCE lexeme verbatim so no precision/format is lost
/// (the canonical form never rewrites a number's spelling). Object members keep source order; the
/// formatter sorts only when `sort_keys` is set.
#[derive(Debug, Clone, PartialEq)]
enum Node {
    Null,
    Bool(bool),
    /// Source number lexeme, verbatim (already validated by the lexer).
    Number(String),
    /// Decoded string value (un-escaped).
    Str(String),
    Array(Vec<Node>),
    /// (key, value) members in source order.
    Object(Vec<(String, Node)>),
}

// ───────────────────────────── public canonical core ─────────────────────────────

/// Canonicalize `bytes` to the byte form prescribed by `form`. PURE — no I/O, no serde_json.
///
/// A file is canonical iff `bytes == canonicalize(bytes, form)`. This makes the gate (check) and the
/// fixer (write) share one definition: the fixer writes `canonicalize(bytes)`; the gate flags any
/// file where the two differ. Idempotent by construction: the output is itself canonical, so
/// `canonicalize(canonicalize(x)) == canonicalize(x)`.
pub fn canonicalize(bytes: &str, form: &CanonicalForm) -> Result<String, CanonError> {
    let node = Parser::new(bytes).parse_document()?;
    // The formatter emits LF (`\n`) internally; a single final pass applies the policy newline and
    // BOM so those two DATA knobs are honored at exactly one site (no `\n` scattered through the form).
    let mut body = String::with_capacity(bytes.len());
    write_node(&mut body, &node, form, 0);
    if form.trailing_newline {
        body.push('\n');
    }
    let body = match form.newline {
        Newline::Lf => body,
        Newline::Crlf => body.replace('\n', "\r\n"),
    };
    let mut out = String::with_capacity(body.len() + 3);
    if form.utf8_bom {
        out.push('\u{feff}');
    }
    out.push_str(&body);
    Ok(out)
}

/// Is `bytes` already canonical under `form`? Convenience over [`canonicalize`].
pub fn is_canonical(bytes: &str, form: &CanonicalForm) -> Result<bool, CanonError> {
    Ok(canonicalize(bytes, form)? == bytes)
}

// ───────────────────────────── parser (hand-written, no serde_json) ─────────────────────────────

struct Parser<'a> {
    bytes: &'a [u8],
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            bytes: src.as_bytes(),
            src,
            pos: 0,
        }
    }

    fn parse_document(&mut self) -> Result<Node, CanonError> {
        // A canonical file may carry a single leading UTF-8 BOM in source; we strip it (BOM is drift,
        // never part of the canonical output). RFC 8259 §8.1 says implementations MAY ignore a BOM.
        if self.src.starts_with('\u{feff}') {
            self.pos += '\u{feff}'.len_utf8();
        }
        self.skip_ws();
        let node = self.parse_value(0)?;
        self.skip_ws();
        if self.pos != self.bytes.len() {
            return Err(CanonError::Parse(format!(
                "trailing data after JSON value at byte {}",
                self.pos
            )));
        }
        Ok(node)
    }

    fn parse_value(&mut self, depth: usize) -> Result<Node, CanonError> {
        if depth > MAX_DEPTH {
            return Err(CanonError::Parse(format!(
                "nesting deeper than the {MAX_DEPTH} canonical depth bound"
            )));
        }
        self.skip_ws();
        let Some(&byte) = self.bytes.get(self.pos) else {
            return Err(CanonError::Parse("unexpected end of input".to_owned()));
        };
        match byte {
            b'{' => self.parse_object(depth),
            b'[' => self.parse_array(depth),
            b'"' => Ok(Node::Str(self.parse_string()?)),
            b't' => self.parse_literal("true", Node::Bool(true)),
            b'f' => self.parse_literal("false", Node::Bool(false)),
            b'n' => self.parse_literal("null", Node::Null),
            b'-' | b'0'..=b'9' => self.parse_number(),
            other => Err(CanonError::Parse(format!(
                "unexpected character {:?} at byte {}",
                other as char, self.pos
            ))),
        }
    }

    fn parse_literal(&mut self, word: &str, node: Node) -> Result<Node, CanonError> {
        if self.src[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(node)
        } else {
            Err(CanonError::Parse(format!(
                "invalid literal at byte {} (expected `{word}`)",
                self.pos
            )))
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Node, CanonError> {
        self.pos += 1; // consume '{'
        let mut members: Vec<(String, Node)> = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Node::Object(members));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(CanonError::Parse(format!(
                    "expected string object key at byte {}",
                    self.pos
                )));
            }
            let key = self.parse_string()?;
            if !seen.insert(key.clone()) {
                return Err(CanonError::DuplicateKey(key));
            }
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(CanonError::Parse(format!(
                    "expected ':' after object key at byte {}",
                    self.pos
                )));
            }
            self.pos += 1; // consume ':'
            let value = self.parse_value(depth + 1)?;
            members.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    return Err(CanonError::Parse(format!(
                        "expected ',' or '}}' in object at byte {}",
                        self.pos
                    )));
                }
            }
        }
        Ok(Node::Object(members))
    }

    fn parse_array(&mut self, depth: usize) -> Result<Node, CanonError> {
        self.pos += 1; // consume '['
        let mut items: Vec<Node> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Node::Array(items));
        }
        loop {
            let value = self.parse_value(depth + 1)?;
            items.push(value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    return Err(CanonError::Parse(format!(
                        "expected ',' or ']' in array at byte {}",
                        self.pos
                    )));
                }
            }
        }
        Ok(Node::Array(items))
    }

    /// Parse a JSON string starting at the current `"`, returning the DECODED scalar value. Rejects
    /// control chars, bad escapes, and lone surrogates (so the formatter re-encodes losslessly).
    fn parse_string(&mut self) -> Result<String, CanonError> {
        self.pos += 1; // consume opening '"'
        let mut out = String::new();
        loop {
            let Some(&byte) = self.bytes.get(self.pos) else {
                return Err(CanonError::Parse("unterminated string".to_owned()));
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    self.parse_escape(&mut out)?;
                }
                0x00..=0x1f => {
                    return Err(CanonError::Parse(format!(
                        "unescaped control character U+{byte:04X} in string"
                    )));
                }
                _ => {
                    // Copy one whole UTF-8 scalar from the source (the source is valid UTF-8 because
                    // it is a &str), advancing by the scalar's byte length.
                    let ch = self.next_char_at(self.pos)?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn parse_escape(&mut self, out: &mut String) -> Result<(), CanonError> {
        let Some(&byte) = self.bytes.get(self.pos) else {
            return Err(CanonError::Parse(
                "dangling escape at end of string".to_owned(),
            ));
        };
        self.pos += 1;
        match byte {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{0008}'),
            b'f' => out.push('\u{000c}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let first = self.parse_hex4()?;
                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    // high surrogate — must be followed by \uDC00..\uDFFF
                    if self.bytes.get(self.pos) == Some(&b'\\')
                        && self.bytes.get(self.pos + 1) == Some(&b'u')
                    {
                        self.pos += 2;
                        let second = self.parse_hex4()?;
                        if !(0xdc00..=0xdfff).contains(&second) {
                            return Err(CanonError::Parse(
                                "high surrogate not followed by a low surrogate".to_owned(),
                            ));
                        }
                        0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00)
                    } else {
                        return Err(CanonError::Parse("lone high surrogate escape".to_owned()));
                    }
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(CanonError::Parse("lone low surrogate escape".to_owned()));
                } else {
                    first
                };
                match char::from_u32(scalar) {
                    Some(ch) => out.push(ch),
                    None => {
                        return Err(CanonError::Parse(format!(
                            "escape \\u{first:04x} is not a Unicode scalar"
                        )));
                    }
                }
            }
            other => {
                return Err(CanonError::Parse(format!(
                    "invalid escape \\{:?}",
                    other as char
                )));
            }
        }
        Ok(())
    }

    fn parse_hex4(&mut self) -> Result<u32, CanonError> {
        let slice = self
            .src
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| CanonError::Parse("truncated \\u escape".to_owned()))?;
        // RFC 8259 §7: EXACTLY 4 hex digits. `u32::from_str_radix` alone is too lenient (it accepts
        // a leading sign, so `\u+12f` would decode as U+012F and misclassify a strictly-invalid
        // escape as fixable drift instead of a parse error).
        if !slice.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(CanonError::Parse(format!("invalid \\u escape `{slice}`")));
        }
        let value = u32::from_str_radix(slice, 16)
            .map_err(|_| CanonError::Parse(format!("invalid \\u escape `{slice}`")))?;
        self.pos += 4;
        Ok(value)
    }

    /// Parse a JSON number, returning the SOURCE lexeme verbatim (no reformatting). Rejects the
    /// non-JSON spellings (`NaN`, `Infinity`, leading `+`, leading zeros, bare `.5`) that a lenient
    /// serializer might emit or accept.
    fn parse_number(&mut self) -> Result<Node, CanonError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        // integer part
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                // a leading zero may not be followed by another digit
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(CanonError::Parse(format!(
                        "leading zero in number at byte {start}"
                    )));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => {
                return Err(CanonError::Parse(format!("invalid number at byte {start}")));
            }
        }
        // fraction
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(CanonError::Parse(format!(
                    "number fraction needs a digit at byte {}",
                    self.pos
                )));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        // exponent
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(CanonError::Parse(format!(
                    "number exponent needs a digit at byte {}",
                    self.pos
                )));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let lexeme = self.src[start..self.pos].to_owned();
        Ok(Node::Number(lexeme))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn next_char_at(&self, index: usize) -> Result<char, CanonError> {
        self.src[index..]
            .chars()
            .next()
            .ok_or_else(|| CanonError::Parse("unexpected end of input in string".to_owned()))
    }
}

// ───────────────────────────── formatter (pretty-print to canonical bytes) ─────────────────────────────

fn write_node(out: &mut String, node: &Node, form: &CanonicalForm, depth: usize) {
    match node {
        Node::Null => out.push_str("null"),
        Node::Bool(true) => out.push_str("true"),
        Node::Bool(false) => out.push_str("false"),
        Node::Number(lexeme) => out.push_str(lexeme),
        Node::Str(value) => write_string(out, value, form),
        Node::Array(items) => write_array(out, items, form, depth),
        Node::Object(members) => write_object(out, members, form, depth),
    }
}

fn write_array(out: &mut String, items: &[Node], form: &CanonicalForm, depth: usize) {
    if items.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push('[');
    out.push('\n');
    let inner = depth + 1;
    for (index, item) in items.iter().enumerate() {
        push_indent(out, form, inner);
        write_node(out, item, form, inner);
        if index + 1 != items.len() {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, form, depth);
    out.push(']');
}

fn write_object(out: &mut String, members: &[(String, Node)], form: &CanonicalForm, depth: usize) {
    if members.is_empty() {
        out.push_str("{}");
        return;
    }
    let inner = depth + 1;
    // sort_keys reorders by the unicode scalar sequence (BTreeMap collation) only when configured;
    // the default canonical form preserves the source order of object members. We materialize one
    // ordered list of (&key, &value) either way so the emit loop has a single shape.
    let ordered: Vec<(&str, &Node)> = if form.sort_keys {
        let sorted: BTreeMap<&str, &Node> = members
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect();
        sorted.into_iter().collect()
    } else {
        members
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect()
    };
    out.push('{');
    out.push('\n');
    for (index, (key, value)) in ordered.iter().enumerate() {
        push_indent(out, form, inner);
        write_string(out, key, form);
        out.push_str(": ");
        write_node(out, value, form, inner);
        if index + 1 != ordered.len() {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, form, depth);
    out.push('}');
}

fn push_indent(out: &mut String, form: &CanonicalForm, depth: usize) {
    for _ in 0..(depth * form.indent_width) {
        out.push(' ');
    }
}

/// Emit a JSON string literal: the mandatory escapes always, plus `\uXXXX` for non-ASCII iff
/// `ensure_ascii`. The solidus `/` is emitted bare (canonical: never `\/`).
fn write_string(out: &mut String, value: &str, form: &CanonicalForm) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                // other control characters: always \u escaped (mandatory)
                push_u_escape(out, c);
            }
            c if (c as u32) < 0x80 => out.push(c),
            c => {
                if form.ensure_ascii {
                    push_u_escape(out, c);
                } else {
                    out.push(c);
                }
            }
        }
    }
    out.push('"');
}

/// Append `\uXXXX` (a surrogate pair for astral-plane scalars), lowercase hex (serde_json parity).
fn push_u_escape(out: &mut String, ch: char) {
    let code = ch as u32;
    if code > 0xffff {
        let adjusted = code - 0x10000;
        let high = 0xd800 + (adjusted >> 10);
        let low = 0xdc00 + (adjusted & 0x3ff);
        out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
    } else {
        out.push_str(&format!("\\u{code:04x}"));
    }
}

// ───────────────────────────── findings + verdict ─────────────────────────────

/// One gate finding, keyed by the offending file path (repo-relative).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

/// The bare-code verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// The full gate outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
}

// ───────────────────────────── collection (the only I/O) ─────────────────────────────

/// Errors collecting the governed corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    MissingGovernedRoots,
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::MissingGovernedRoots => {
                write!(f, "policy `governed_roots` must be a non-empty array")
            }
            CollectError::Io(message) => write!(f, "canonical-json collection io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// One observed governed file: its repo-relative path and its raw committed bytes (as a String, since
/// the canonical grammar requires valid UTF-8; non-UTF-8 files are surfaced as a parse error row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFile {
    pub path: String,
    /// `Some(bytes)` for a readable UTF-8 file; `None` if the file was not valid UTF-8.
    pub bytes: Option<String>,
}

/// The collected governed corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    pub files: Vec<ObservedFile>,
}

/// Read every governed `*.json` under the policy's `governed_roots`, applying the suffix/path-prefix
/// exclusions. Read-only filesystem walk (no temp files, no git). Paths are repo-relative and sorted.
pub fn collect_observed(root: &Path, policy: &Value) -> Result<Observed, CollectError> {
    let roots = policy
        .get("governed_roots")
        .and_then(Value::as_array)
        .filter(|roots| !roots.is_empty())
        .ok_or(CollectError::MissingGovernedRoots)?;
    let suffix_exclusions = string_array(policy, &["exclusions", "suffixes"]);
    let path_prefix_exclusions = string_array(policy, &["exclusions", "path_prefixes"]);

    let mut rel_paths: BTreeSet<String> = BTreeSet::new();
    for root_value in roots {
        let Some(rel_root) = root_value.as_str() else {
            continue;
        };
        let abs_root = root.join(rel_root);
        if !abs_root.exists() {
            continue;
        }
        walk_json(&abs_root, root, &mut rel_paths)?;
    }

    let mut files = Vec::new();
    for rel in rel_paths {
        if suffix_exclusions.iter().any(|suffix| rel.ends_with(suffix)) {
            continue;
        }
        if path_prefix_exclusions
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            continue;
        }
        let abs = root.join(&rel);
        let bytes = match fs::read(&abs) {
            Ok(raw) => String::from_utf8(raw).ok(),
            Err(error) => return Err(CollectError::Io(format!("read {rel}: {error}"))),
        };
        files.push(ObservedFile { path: rel, bytes });
    }
    Ok(Observed { files })
}

fn string_array(policy: &Value, path: &[&str]) -> Vec<String> {
    let mut cursor = policy;
    for segment in path {
        match cursor.get(segment) {
            Some(next) => cursor = next,
            None => return Vec::new(),
        }
    }
    cursor
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn walk_json(dir: &Path, repo_root: &Path, out: &mut BTreeSet<String>) -> Result<(), CollectError> {
    let entries = fs::read_dir(dir)
        .map_err(|error| CollectError::Io(format!("read dir {}: {error}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            CollectError::Io(format!("dir entry in {}: {error}", dir.display()))
        })?;
        let path = entry.path();
        // `file_type()` does NOT follow symlinks — recurse only into real directories (so a symlinked
        // directory can never cause a walk loop). `metadata()` DOES follow symlinks, so a symlinked
        // `*.json` resolving to a regular file is still governed (a symlink is not an escape hatch).
        let file_type = entry
            .file_type()
            .map_err(|error| CollectError::Io(format!("file type {}: {error}", path.display())))?;
        if file_type.is_dir() {
            walk_json(&path, repo_root, out)?;
            continue;
        }
        let is_regular_file = if file_type.is_symlink() {
            fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false)
        } else {
            file_type.is_file()
        };
        // Match `.json` case-insensitively so an uppercase `.JSON` cannot evade governance.
        let is_json = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        if is_regular_file
            && is_json
            && let Ok(rel) = path.strip_prefix(repo_root)
            && let Some(rel_str) = rel.to_str()
        {
            out.insert(rel_str.replace('\\', "/"));
        }
    }
    Ok(())
}

// ───────────────────────────── evaluation (pure) ─────────────────────────────

/// The pure kernel: fold the observed corpus into findings under the policy's canonical form. No I/O.
/// One finding per offending file, keyed by repo-relative path.
pub fn evaluate_keyed(policy: &Value, observed: &Observed) -> BTreeSet<Finding> {
    let form = CanonicalForm::from_policy(policy);
    let mut findings = BTreeSet::new();
    for file in &observed.files {
        let Some(bytes) = &file.bytes else {
            findings.insert(Finding::new(
                "json_parse_error",
                &file.path,
                "file is not valid UTF-8; canonical JSON requires UTF-8".to_owned(),
            ));
            continue;
        };
        match canonicalize(bytes, &form) {
            Ok(canonical) => {
                if &canonical != bytes {
                    findings.insert(Finding::new(
                        "json_not_canonical",
                        &file.path,
                        "committed bytes differ from the canonical re-serialization; run the canonical-json fixer".to_owned(),
                    ));
                }
            }
            Err(error) => {
                findings.insert(Finding::new(error.code(), &file.path, error.to_string()));
            }
        }
    }
    findings
}

/// The bare-code projection: the verdict + an ordered findings vector.
pub fn evaluate(policy: &Value, observed: &Observed) -> Report {
    let findings: Vec<Finding> = evaluate_keyed(policy, observed).into_iter().collect();
    let verdict = if findings.is_empty() {
        Verdict::Green
    } else {
        Verdict::Red
    };
    Report { verdict, findings }
}

/// Render findings as a human-readable remediation message (used by the gate binary).
pub fn render_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "canonical-json gate passed".to_owned();
    }
    let mut output = String::from("canonical-json gate failed:\n");
    for finding in findings {
        output.push_str(&format!(
            "- {} {}: {}\n",
            finding.code, finding.key, finding.detail
        ));
    }
    output.push('\n');
    output.push_str(AUTO_FIX_COMMAND);
    output.push('\n');
    output
}

/// The exact auto-remediation command the gate prints on failure (founder 2026-06-11: automation is
/// the default, the blocking gate is the backstop, and its output must print the exact fix command).
/// `json_not_canonical` drift is mechanically derivable, so the canonical answer is to RUN this, not
/// to hand-edit bytes. `json_parse_error`/`json_duplicate_key` are the human-judgment residue the
/// fixer refuses; for those the listed detail is the instruction.
pub const AUTO_FIX_COMMAND: &str = "Auto-remediation (run this — do NOT hand-edit bytes; canonicalization is mechanically derivable):\n  \
     buck2 run //ci/facade/canonical-json:oya-cloud-ci-canonical-json-bin -- --fix\n\
     The fixer rewrites every `json_not_canonical` file to canonical form and refuses (never silently \
     rewrites) `json_parse_error`/`json_duplicate_key` defects — fix those by hand per the listed detail.";

// ───────────────────────────── fixer (write canonical bytes) ─────────────────────────────

/// The outcome of a fixer run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixReport {
    /// Files rewritten to canonical form.
    pub fixed: Vec<String>,
    /// Files the fixer refused (parse error / duplicate key) — these need a human, not a rewrite.
    pub refused: Vec<(String, String)>,
}

impl FixReport {
    pub fn is_clean(&self) -> bool {
        self.refused.is_empty()
    }
}

/// Rewrite every non-canonical governed file to canonical form. PURE-of-policy I/O: reads + writes
/// only the governed corpus. Refuses (never silently rewrites) a file that fails to parse or carries
/// duplicate keys — those are human-judgment defects, not formatting drift.
pub fn fix_observed(
    root: &Path,
    policy: &Value,
    observed: &Observed,
    dry_run: bool,
) -> Result<FixReport, CollectError> {
    let form = CanonicalForm::from_policy(policy);
    let mut fixed = Vec::new();
    let mut refused = Vec::new();
    for file in &observed.files {
        let Some(bytes) = &file.bytes else {
            refused.push((file.path.clone(), "not valid UTF-8".to_owned()));
            continue;
        };
        match canonicalize(bytes, &form) {
            Ok(canonical) => {
                if &canonical != bytes {
                    if !dry_run {
                        let abs = root.join(&file.path);
                        fs::write(&abs, canonical).map_err(|error| {
                            CollectError::Io(format!("write {}: {error}", file.path))
                        })?;
                    }
                    fixed.push(file.path.clone());
                }
            }
            Err(error) => refused.push((file.path.clone(), error.to_string())),
        }
    }
    Ok(FixReport { fixed, refused })
}

/// Load and parse the policy JSON at `policy_path` (relative to `root`). serde_json is fine here:
/// the policy is gate-internal config, not a governed corpus file.
pub fn load_policy(root: &Path, policy_path: &str) -> Result<Value, CollectError> {
    let abs = root.join(policy_path);
    let text = fs::read_to_string(&abs)
        .map_err(|error| CollectError::Io(format!("read policy: {error}")))?;
    serde_json::from_str(&text)
        .map_err(|error| CollectError::Io(format!("parse policy {policy_path}: {error}")))
}

/// The repo-relative path of the bundled policy DATA for this crate.
pub const POLICY_PATH: &str = "ci/facade/canonical-json/canonical-json-policy.json";

#[cfg(test)]
mod tests;
