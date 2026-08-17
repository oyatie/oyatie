//! # cloud-ci-port-placement (ADR-0570)
//!
//! The clean-arch port-placement gate. Founder ports-in-core doctrine: a storage/repository/port
//! trait is part of the **cutover-stable** seam and must be DEFINED in a `core`/`ports`/`kernel`
//! crate, not in a throwaway `*-adapter` crate. The adapter DEPENDS on core and IMPLEMENTS the
//! port; it must never OWN it. This gate productizes the defect class that #116 fixed by hand:
//! billing's `AccountingJournalStoragePort` had been defined in the in-memory adapter crate,
//! inverting the convention proven by tenancy's `TenantLifecycleStore` and SCIM's
//! `UserStore`/`GroupStore` (ports in core). No existing gate caught "a port trait DEFINED in an
//! `*/adapters/*` crate" — face-direction / tier-acyclicity check dep EDGES, kernel-purity checks
//! `*-kernel` dep containment — so this is a NEW detector, mirroring kernel-purity's STRUCTURE.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the forbidden layer-dir segment(s), the
//! port-name suffix set, the per-(crate, trait) allowlist, the member-count floor — are DATA in
//! `port-placement-policy.json`, and the frozen set of pre-existing violations is DATA in
//! `port-placement-baseline.json`. Nothing oyatie-specific is hardcoded in Rust; a
//! different repo adopts the gate by repointing the policy + baseline at its own crate tree.
//!
//! ## Kernel contract
//! - [`collect_port_traits`] `(root, policy) -> {member_traits:[..]}` enumerates workspace members,
//!   keeps those whose repo-relative path contains a forbidden layer-dir segment, and for each
//!   records every `pub trait <Name>` DEFINED in its `src/**/*.rs`. Read-only; writes no temp files.
//! - [`evaluate_keyed`] `(policy, baseline, observed) -> BTreeSet<Finding>` is PURE and
//!   unit-testable without a filesystem: it applies the port-name suffix heuristic to each
//!   adapter-defined trait, folds the allowlist, and subtracts the frozen baseline so only NEW
//!   violations are RED (born-advisory + enforce-no-regression).
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Automation posture (v1 flag-with-precise-remediation; auto-move = NOTED FOLLOW-UP)
//! Every finding carries a precise [`Finding::next_action`]: it names the trait, the adapter crate
//! that wrongly defines it, and the sibling core/ports crate it should move to (inferred from the
//! capability stem of the adapter path). A full auto-MOVE codemod — relocating the trait + its
//! port-definitional types into core and rewriting the adapter to depend-and-implement — is a
//! non-trivial design act and a NOTED FOLLOW-UP, NOT this slice. v1 is flag-only.
//!
//! ## Ratchet semantics
//! After #116 the live corpus still carries pre-existing storage-port traits defined in adapter
//! crates (the tenant-rbac / session / secret-provider / kms-domain-repo ports; their relocation is
//! separate). They are frozen in `port-placement-baseline.json` and the gate is
//! born-ADVISORY against them. A NEW port-suffix trait defined in an adapter (beyond the baseline)
//! fails closed. Relocating a baselined trait self-cleans via `PP-STALE-BASELINE`. The gate flips to
//! fully blocking when the baseline reaches 0.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `PP-PORT-IN-ADAPTER`     — a `pub trait <Name>` matching a port-name suffix is DEFINED in a
//!   crate whose path contains a forbidden layer-dir segment, and it is NOT in the frozen baseline
//!   nor the allowlist. This is the born-blocking enforce-no-regression finding.
//! - `PP-STALE-BASELINE`      — a frozen baseline entry matches no live adapter-defined port trait
//!   (the violation was relocated); remove it (the baseline is shrink-only).
//! - `PP-STALE-ALLOWLIST`     — a declared allowlist entry matches no live finding (self-cleaning).
//! - `PP-EMPTY-SCAN`          — the scan found fewer workspace members than
//!   `min_expected_member_crates` (catches a silently broken glob / CWD / collect that would
//!   otherwise be a false-green).
//! - `PP-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `PP-POLICY-MALFORMED`    — the policy `port_name_suffixes` / `forbidden_layer_dirs` /
//!   `allowlist` is malformed (fail-closed rather than silently dropping rules).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use oya_workspace_members_kernel::resolve_member_dirs;
use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-port-placement";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 6] = [
    "PP-PORT-IN-ADAPTER",
    "PP-STALE-BASELINE",
    "PP-STALE-ALLOWLIST",
    "PP-EMPTY-SCAN",
    "PP-POLICY-GATE-ID-MISMATCH",
    "PP-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-crate.
const POLICY_KEY: &str = "<policy>";

/// Errors collecting the observed adapter-defined trait set. The kernel returns these instead of
/// panicking so the caller (CI / a controller) decides how to surface them — a missing root or an
/// unreadable source file is a fail-closed error, never a silently skipped crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    ResolveMembers(String),
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::ResolveMembers(message) => {
                write!(f, "resolve workspace members: {message}")
            }
            CollectError::Io(message) => write!(f, "port-placement io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// One `pub trait` definition discovered in a member crate's source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TraitDef {
    name: String,
    /// Repo-relative source file the trait is defined in (e.g. `iam/adapters/foo/src/lib.rs`).
    file: String,
}

/// Collect the adapter-defined `pub trait` set described by the policy.
///
/// Enumerates workspace members, keeps those that are in a forbidden layer OR whose crate name
/// (the final path segment) ends with a forbidden crate-name suffix, and for each records every
/// `pub trait <Name>` DEFINED in its `src/**/*.rs`. Emits
/// `{ "member_crates_found": <usize>, "members": [ { "crate", "member_path", "traits":
/// [ { "name", "file" } ] } ] }`. Read-only.
pub fn collect_port_traits(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let forbidden_dirs = forbidden_layer_dirs(policy);
    let forbidden_suffixes = forbidden_crate_name_suffixes(policy);
    let member_dirs = resolve_member_dirs(root)
        .map_err(|error| CollectError::ResolveMembers(error.to_string()))?;

    let member_count = member_dirs.len();
    let mut members = Vec::new();
    for member_dir in &member_dirs {
        let in_forbidden_layer = path_in_forbidden_layer(member_dir, &forbidden_dirs);
        let has_forbidden_suffix = crate_in_forbidden_layer(member_dir, &forbidden_suffixes);
        if !in_forbidden_layer && !has_forbidden_suffix {
            continue;
        }
        let mut traits = collect_pub_traits(root, member_dir)?;
        traits.sort();
        if traits.is_empty() {
            continue;
        }
        let crate_tail = crate_tail(member_dir);
        members.push(json!({
            "crate": crate_tail,
            "member_path": member_dir,
            "traits": traits
                .iter()
                .map(|t| json!({ "name": t.name, "file": t.file }))
                .collect::<Vec<_>>(),
        }));
    }

    Ok(json!({
        "member_crates_found": member_count,
        "members": members,
    }))
}

/// True iff a repo-relative member dir contains any forbidden layer-dir as a whole path SEGMENT.
/// Segment-anchored so `adapters` matches `iam/adapters/foo` but not a crate literally named
/// `my-adapters-helper` (whole-segment, never substring).
fn path_in_forbidden_layer(member_dir: &str, forbidden: &[String]) -> bool {
    member_dir
        .split('/')
        .any(|segment| forbidden.iter().any(|f| f == segment))
}

/// True iff the crate name (last path segment of `member_dir`) ends with any of the
/// `forbidden_crate_name_suffixes`. Catches adapter-NAMED crates that live outside a
/// forbidden-layer directory (e.g. `oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory`
/// is NOT under any `adapters/` segment but its crate name ends with `-adapter-inmemory`).
fn crate_in_forbidden_layer(member_dir: &str, forbidden_suffixes: &[String]) -> bool {
    let tail = crate_tail(member_dir);
    forbidden_suffixes
        .iter()
        .any(|suffix| tail.ends_with(suffix.as_str()))
}

/// The crate-identifying tail of a member dir (`iam/adapters/tenant-rbac-storage-inmemory` ->
/// `tenant-rbac-storage-inmemory`). Used in the collected output `"crate"` field (human display
/// only); the gate KEYS on `member_path`, not this tail.
fn crate_tail(member_dir: &str) -> String {
    member_dir
        .rsplit('/')
        .next()
        .unwrap_or(member_dir)
        .to_owned()
}

/// Collect every `pub trait <Name>` DEFINED in `<member>/src/**/*.rs`.
///
/// The scan is a conservative line-based detector tuned for the SAFE (over-approximate) direction
/// of a born-advisory gate: it recognizes a trait DEFINITION header — a line whose first
/// non-whitespace tokens are `pub trait <Name>` (optionally `pub(crate)`/`pub(...)`, optionally
/// `unsafe`, optionally `async`) where `<Name>` is a normal Rust identifier. Single-line `//`
/// comments are stripped; lines inside `/* ... */` block comments are skipped so a commented-out
/// `pub trait` never false-positives. A `trait` used as a bound (`impl T: Store`) or in a where
/// clause never starts a line with `pub trait`, so it is not matched. The detector reports a trait
/// at most once per (name, file).
fn collect_pub_traits(root: &Path, member_dir: &str) -> Result<Vec<TraitDef>, CollectError> {
    let src_dir = root.join(member_dir).join("src");
    let mut out: BTreeSet<TraitDef> = BTreeSet::new();
    collect_traits_in_dir(&src_dir, root, &mut out)?;
    Ok(out.into_iter().collect())
}

fn collect_traits_in_dir(
    dir: &Path,
    root: &Path,
    out: &mut BTreeSet<TraitDef>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            collect_traits_in_dir(&path, root, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let text = fs::read_to_string(&path)
                .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            for name in extract_pub_trait_names(&text) {
                out.insert(TraitDef {
                    name,
                    file: rel.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Extract every `pub trait <Name>` definition name from Rust source text, comment-aware.
///
/// Pure + filesystem-free so it is directly unit-testable. Strips `//` line comments and skips
/// lines fully inside `/* ... */` block comments (a coarse but sound treatment for the
/// over-approximate detection direction — a `pub trait` mention inside a `/* */` block is trivia,
/// never a definition). A definition header is a line whose leading tokens (after optional
/// `pub(...)` / `unsafe` / `async` modifiers between `pub` and `trait`) are `pub trait <Ident>`.
pub fn extract_pub_trait_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_block_comment = false;
    for raw_line in text.lines() {
        let line = strip_comments(raw_line, &mut in_block_comment);
        if let Some(name) = pub_trait_name(&line) {
            names.push(name);
        }
    }
    names
}

/// Strip `//` line comments and `/* */` block-comment spans from a single line, threading the
/// `in_block_comment` state across lines. Returns the code-only remainder of the line. String
/// literals are not modeled (a `//` or `/*` inside a string is rare in a `pub trait` header line
/// and only ever causes the detector to drop a header, which is the safe direction for the OWN
/// trait-name extraction since a real definition header carries no such literal before `trait`).
fn strip_comments(line: &str, in_block_comment: &mut bool) -> String {
    let bytes = line.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if *in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                *in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            break; // rest of line is a line comment
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            *in_block_comment = true;
            i += 2;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// If `line` is a `pub trait <Ident>` definition header, return the trait name. Recognizes the
/// modifier shapes that may sit between `pub` and `trait`: a visibility qualifier `pub(crate)` /
/// `pub(in path)` / `pub(super)` (collapsed to `pub`), and `unsafe`. (`async trait` is not valid
/// Rust for a bare trait def, but `#[async_trait] pub trait` puts the attribute on its own line so
/// the `pub trait` header is still matched.) Returns None for any non-definition line.
fn pub_trait_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    // Must start with `pub` (optionally `pub(...)`).
    let rest = trimmed.strip_prefix("pub")?;
    // Allow a `(crate)` / `(in ...)` / `(super)` qualifier immediately after `pub`.
    let rest = if rest.starts_with('(') {
        let close = rest.find(')')?;
        &rest[close + 1..]
    } else {
        rest
    };
    let rest = rest.trim_start();
    // `pub` must be followed by whitespace then the keyword chain — guard against `public_x`.
    if rest.is_empty() {
        return None;
    }
    // Skip an optional `unsafe` modifier.
    let rest = rest
        .strip_prefix("unsafe")
        .map(str::trim_start)
        .unwrap_or(rest);
    // The keyword must be exactly `trait` followed by whitespace.
    let rest = rest.strip_prefix("trait")?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None; // e.g. `pub traitlike` — not a trait def
    }
    let rest = rest.trim_start();
    // Read the identifier: ASCII alnum + `_`, must start with alpha or `_`.
    let mut name = String::new();
    for ch in rest.chars() {
        if name.is_empty() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                name.push(ch);
            } else {
                return None;
            }
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }
    if name.is_empty() { None } else { Some(name) }
}

// ---------------------------------------------------------------------------
// Pure evaluation
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
    /// v1 is flag-only: no port-placement finding is mechanically auto-fixable (relocating a trait
    /// is a design act). Carried for parity with sibling gates and the render contract.
    pub auto_fixable: bool,
    /// The best next action printed to the contributor, always populated — never a bare FAIL.
    pub next_action: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
            auto_fixable: false,
            next_action: String::new(),
        }
    }

    fn with_action(mut self, next_action: impl Into<String>) -> Self {
        self.next_action = next_action.into();
        self
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

/// A single (member_path, trait) identity — the key shape used by the allowlist, the baseline,
/// and a PP-PORT-IN-ADAPTER finding. Keyed on the full repo-relative crate directory rather than
/// the crate-name tail to avoid false-GREEN masking when two sibling crates share the same tail
/// (e.g. two `rest` crates in different capability trees would collide on `rest:SomeTrait` if
/// keyed by tail; `intelligence/adapters/rest:SomeTrait` is unambiguous).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PortKey {
    member_path: String,
    trait_name: String,
}

impl PortKey {
    fn as_string(&self) -> String {
        format!("{}:{}", self.member_path, self.trait_name)
    }
}

/// Parse a list of `{ "member_path": .., "trait": .. }` entries (allowlist or baseline) from
/// DATA. Returns Err with a human-readable message on any malformed entry so the evaluator can
/// fail CLOSED instead of silently dropping rules.
fn parse_port_keys(value: Option<&Value>, label: &str) -> Result<Vec<PortKey>, String> {
    let Some(array) = value.and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, entry) in array.iter().enumerate() {
        let member_path = entry
            .get("member_path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{label}[{i}]: missing or non-string `member_path` key"))?
            .to_owned();
        let trait_name = entry
            .get("trait")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{label}[{i}] (member_path={member_path:?}): missing or non-string `trait` key"
                )
            })?
            .to_owned();
        out.push(PortKey {
            member_path,
            trait_name,
        });
    }
    Ok(out)
}

/// Parse the port-name suffix set from DATA. Returns Err if absent/empty/malformed (fail-closed:
/// an empty suffix set would silently match nothing and false-green the whole gate).
fn parse_suffixes(policy: &Value) -> Result<Vec<String>, String> {
    let suffixes: Vec<String> = policy
        .get("port_name_suffixes")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    if suffixes.is_empty() {
        return Err(
            "`port_name_suffixes` must be a non-empty array of strings; an empty set would match no \
             trait and silently false-green the gate"
                .to_owned(),
        );
    }
    Ok(suffixes)
}

/// True iff a trait name matches any port-name suffix.
fn trait_is_port(name: &str, suffixes: &[String]) -> bool {
    suffixes
        .iter()
        .any(|suffix| name.ends_with(suffix.as_str()))
}

/// Pure evaluator. `policy` is DATA (`port-placement-policy.json`); `baseline` is the frozen
/// pre-existing-violation set (`port-placement-baseline.json`, an array of
/// `{crate, trait}`); `observed` is the collected adapter-defined trait graph shaped by
/// [`collect_port_traits`].
///
/// Born-advisory + enforce-no-regression: a port-suffix trait DEFINED in an adapter crate is a
/// candidate violation; it is RED (`PP-PORT-IN-ADAPTER`) UNLESS it is allowlisted or present in the
/// frozen baseline. A baseline entry matching no live candidate is `PP-STALE-BASELINE`; an
/// allowlist entry matching no live candidate is `PP-STALE-ALLOWLIST`.
pub fn evaluate_keyed(policy: &Value, baseline: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "PP-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let suffixes = match parse_suffixes(policy) {
        Ok(suffixes) => suffixes,
        Err(message) => {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED",
                POLICY_KEY,
                format!("policy malformed — {message}; the policy must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };
    let allowlist = match parse_port_keys(policy.get("allowlist"), "allowlist") {
        Ok(allowlist) => allowlist,
        Err(message) => {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED",
                POLICY_KEY,
                format!("allowlist malformed — {message}; the policy must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };
    let baseline_keys = match parse_port_keys(baseline_array(baseline), "baseline") {
        Ok(keys) => keys,
        Err(message) => {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED",
                POLICY_KEY,
                format!("baseline malformed — {message}; the frozen baseline must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };

    let min_expected = policy
        .get("min_expected_member_crates")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let member_count = observed
        .get("member_crates_found")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if member_count < min_expected {
        findings.insert(Finding::new(
            "PP-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {member_count} workspace members, below the policy floor of {min_expected}; the member glob, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    // The set of live (member_path, trait) candidate violations: every port-suffix `pub trait`
    // defined in a forbidden-layer or adapter-named crate.
    let mut live: BTreeSet<PortKey> = BTreeSet::new();
    let mut live_files: std::collections::BTreeMap<PortKey, String> =
        std::collections::BTreeMap::new();
    let members = observed
        .get("members")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for member in &members {
        let crate_tail_display = member
            .get("crate")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let Some(member_path) = member.get("member_path").and_then(Value::as_str) else {
            continue;
        };
        let Some(traits) = member.get("traits").and_then(Value::as_array) else {
            continue;
        };
        for trait_value in traits {
            let Some(trait_name) = trait_value.get("name").and_then(Value::as_str) else {
                continue;
            };
            if !trait_is_port(trait_name, &suffixes) {
                continue;
            }
            let file = trait_value
                .get("file")
                .and_then(Value::as_str)
                .unwrap_or(member_path);
            let key = PortKey {
                member_path: member_path.to_owned(),
                trait_name: trait_name.to_owned(),
            };
            live.insert(key.clone());
            live_files
                .entry(key)
                .or_insert_with(|| format!("{} ({})", file, crate_tail_display));
        }
    }

    let allowlist_set: BTreeSet<PortKey> = allowlist.iter().cloned().collect();
    let baseline_set: BTreeSet<PortKey> = baseline_keys.iter().cloned().collect();

    // NEW violation = live candidate that is neither allowlisted nor baselined.
    for key in &live {
        if allowlist_set.contains(key) || baseline_set.contains(key) {
            continue;
        }
        let file_display = live_files
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.member_path.clone());
        let detail = format!(
            "storage-port trait `{}` is DEFINED in adapter crate `{}` ({}); a port belongs in a core/ports/kernel crate, not an adapter (ADR-0570; the #116 defect class)",
            key.trait_name, key.member_path, file_display
        );
        let action = format!(
            "DESIGN ACTION (not auto-applied): move `{}` and its port-definitional types out of `{}` into the sibling core/ports crate `{}` (mirror billing #802: the core OWNS the port, the adapter DEPENDS on core and IMPLEMENTS it). A full auto-move codemod is a noted follow-up.",
            key.trait_name,
            key.member_path,
            suggested_core_crate(&key.member_path)
        );
        findings.insert(
            Finding::new("PP-PORT-IN-ADAPTER", &key.as_string(), detail).with_action(action),
        );
    }

    // Stale baseline: a frozen entry that no longer matches a live candidate (relocated -> clean up).
    for key in &baseline_set {
        if !live.contains(key) {
            findings.insert(
                Finding::new(
                    "PP-STALE-BASELINE",
                    &key.as_string(),
                    format!(
                        "baseline entry `{}` trait `{}` matches no live adapter-defined port trait; the violation was relocated — remove it from port-placement-baseline.json (the baseline is shrink-only)",
                        key.member_path, key.trait_name
                    ),
                )
                .with_action(format!(
                    "Remove the {{\"member_path\": \"{}\", \"trait\": \"{}\"}} entry from the frozen baseline.",
                    key.member_path, key.trait_name
                )),
            );
        }
    }

    // Stale allowlist: a declared carve-out matching no live candidate (self-cleaning).
    for key in &allowlist_set {
        if !live.contains(key) {
            findings.insert(
                Finding::new(
                    "PP-STALE-ALLOWLIST",
                    &key.as_string(),
                    format!(
                        "allowlist entry `{}` trait `{}` matched no live finding; remove it (the allowlist is shrink-only)",
                        key.member_path, key.trait_name
                    ),
                )
                .with_action(format!(
                    "Remove the {{\"member_path\": \"{}\", \"trait\": \"{}\"}} entry from the policy allowlist.",
                    key.member_path, key.trait_name
                )),
            );
        }
    }

    findings
}

/// The baseline payload may be either a bare array of `{crate, trait}` entries OR an object with a
/// `"baseline"` array key (so the generated face can carry a `_comment`/`gate_id` header alongside).
fn baseline_array(baseline: &Value) -> Option<&Value> {
    if baseline.is_array() {
        return Some(baseline);
    }
    baseline.get("baseline")
}

/// Infer the sibling core/ports crate a port should move to, from an adapter member path. Pure
/// string heuristic for the remediation hint only — it never drives a code change. `iam/adapters/x`
/// -> `iam/core/<x-stem>` is the canonical destination shape (capability stem + `/core/`).
fn suggested_core_crate(member_path: &str) -> String {
    let segments: Vec<&str> = member_path.split('/').collect();
    if let Some(pos) = segments.iter().position(|s| *s == "adapters")
        && pos > 0
    {
        let capability = segments[..pos].join("/");
        return format!("{capability}/core/<port-crate>");
    }
    "the capability's core/ports crate".to_owned()
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, baseline: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, baseline, observed))
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its
/// `next_action`. v1 is flag-only (no auto-fixable subset).
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "port-placement gate passed: no NEW storage-port trait is defined in an adapter crate"
            .to_owned();
    }
    let mut out = String::from("port-placement gate failed:\n");
    out.push_str(
        "\n  DESIGN ACTIONS (not auto-applied — relocating a trait is a design decision; \
         auto-move codemod is a noted follow-up):\n",
    );
    for finding in findings {
        out.push_str(&format!(
            "    - {} {}\n        {}\n        {}\n",
            finding.code, finding.key, finding.detail, finding.next_action
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Policy helpers
// ---------------------------------------------------------------------------

fn forbidden_layer_dirs(policy: &Value) -> Vec<String> {
    policy
        .get("forbidden_layer_dirs")
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

/// Read the optional `forbidden_crate_name_suffixes` array from policy DATA. When present, any
/// crate whose name (last path segment) ends with one of these suffixes is treated as an adapter
/// crate regardless of whether its path contains a forbidden layer-dir segment — this catches
/// adapter-NAMED crates that live outside the canonical `adapters/` directory layout (e.g.
/// `oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory`).
fn forbidden_crate_name_suffixes(policy: &Value) -> Vec<String> {
    policy
        .get("forbidden_crate_name_suffixes")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "forbidden_layer_dirs": ["adapters"],
            "port_name_suffixes": ["StoragePort", "Repository", "Store", "Repo"],
            "min_expected_member_crates": 1,
            "allowlist": []
        })
    }

    fn empty_baseline() -> Value {
        json!([])
    }

    /// Build an observed payload: one adapter member with the given (trait-name, file) pairs.
    fn observed_adapter(crate_tail: &str, member_path: &str, traits: &[(&str, &str)]) -> Value {
        json!({
            "member_crates_found": 1000,
            "members": [{
                "crate": crate_tail,
                "member_path": member_path,
                "traits": traits
                    .iter()
                    .map(|(n, f)| json!({ "name": n, "file": f }))
                    .collect::<Vec<_>>(),
            }]
        })
    }

    // --- extract_pub_trait_names (pure source detector) ---

    #[test]
    fn extracts_plain_pub_trait() {
        let src = "pub trait FooStore {\n    fn get(&self);\n}\n";
        assert_eq!(extract_pub_trait_names(src), vec!["FooStore"]);
    }

    #[test]
    fn extracts_pub_trait_with_supertraits_and_modifiers() {
        let src = "pub unsafe trait BarRepository: Send + Sync {}\n\
                   pub(crate) trait BazPort {}\n";
        assert_eq!(
            extract_pub_trait_names(src),
            vec!["BarRepository", "BazPort"]
        );
    }

    #[test]
    fn ignores_non_pub_and_bounds_and_impls() {
        let src = "trait PrivateStore {}\n\
                   impl SomeStore for X {}\n\
                   fn f<T: Store>(t: T) {}\n\
                   // pub trait CommentedStore {}\n";
        assert!(extract_pub_trait_names(src).is_empty());
    }

    #[test]
    fn ignores_pub_trait_inside_block_comment() {
        let src = "/*\n  pub trait HiddenStore {}\n*/\npub fn real() {}\n";
        assert!(extract_pub_trait_names(src).is_empty());
    }

    #[test]
    fn does_not_match_pub_traitlike_token() {
        let src = "pub traitlike fn x() {}\npub trait_helper() {}\n";
        assert!(extract_pub_trait_names(src).is_empty());
    }

    // --- the port-name suffix heuristic ---

    #[test]
    fn port_named_trait_in_adapter_is_flagged() {
        let observed = observed_adapter(
            "foo-inmemory",
            "iam/adapters/foo-inmemory",
            &[("FooStore", "iam/adapters/foo-inmemory/src/lib.rs")],
        );
        let findings = evaluate_keyed(&policy(), &empty_baseline(), &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "PP-PORT-IN-ADAPTER"
                    && f.key == "iam/adapters/foo-inmemory:FooStore"),
            "a port-named trait in an adapter must be flagged: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &empty_baseline(), &observed).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn non_port_named_trait_in_adapter_is_clean() {
        // Behavioral adapter seams (Authorizer/Backend/Spawner/...) carry no port suffix.
        let observed = observed_adapter(
            "foo-adapter",
            "iam/adapters/foo-adapter",
            &[
                ("WorkloadAuthorizer", "iam/adapters/foo-adapter/src/lib.rs"),
                (
                    "ClaudeProcessSpawner",
                    "iam/adapters/foo-adapter/src/lib.rs",
                ),
                ("SvidIssuanceBackend", "iam/adapters/foo-adapter/src/lib.rs"),
            ],
        );
        let findings = evaluate_keyed(&policy(), &empty_baseline(), &observed);
        assert!(
            findings.is_empty(),
            "behavioral adapter traits must NOT be flagged: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &empty_baseline(), &observed).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn allowlisted_port_trait_is_clean() {
        let mut policy = policy();
        policy["allowlist"] = json!([{ "member_path": "iam/adapters/foo-inmemory", "trait": "FooStore", "reason": "internal cursor" }]);
        let observed = observed_adapter(
            "foo-inmemory",
            "iam/adapters/foo-inmemory",
            &[("FooStore", "iam/adapters/foo-inmemory/src/lib.rs")],
        );
        let findings = evaluate_keyed(&policy, &empty_baseline(), &observed);
        assert!(
            !findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"),
            "an allowlisted trait must not be a PP-PORT-IN-ADAPTER finding: {findings:#?}"
        );
    }

    #[test]
    fn baselined_existing_violation_is_not_red() {
        let baseline = json!([{ "member_path": "iam/adapters/foo-inmemory", "trait": "FooStore" }]);
        let observed = observed_adapter(
            "foo-inmemory",
            "iam/adapters/foo-inmemory",
            &[("FooStore", "iam/adapters/foo-inmemory/src/lib.rs")],
        );
        let findings = evaluate_keyed(&policy(), &baseline, &observed);
        assert!(
            !findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"),
            "a baselined existing violation must not be RED: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &baseline, &observed).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn new_violation_beyond_baseline_is_red() {
        // Baseline freezes FooStore; a NEW BarRepository in the same adapter is RED.
        let baseline = json!([{ "member_path": "iam/adapters/foo-inmemory", "trait": "FooStore" }]);
        let observed = observed_adapter(
            "foo-inmemory",
            "iam/adapters/foo-inmemory",
            &[
                ("FooStore", "iam/adapters/foo-inmemory/src/lib.rs"),
                ("BarRepository", "iam/adapters/foo-inmemory/src/lib.rs"),
            ],
        );
        let findings = evaluate_keyed(&policy(), &baseline, &observed);
        assert!(
            findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"
                && f.key == "iam/adapters/foo-inmemory:BarRepository"),
            "a NEW port trait beyond the baseline must be RED: {findings:#?}"
        );
        assert!(
            !findings
                .iter()
                .any(|f| f.key == "iam/adapters/foo-inmemory:FooStore"),
            "the baselined trait must stay green: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &baseline, &observed).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn relocated_baseline_entry_becomes_stale() {
        // Baseline freezes FooStore but the adapter no longer defines it -> PP-STALE-BASELINE.
        let baseline = json!([{ "member_path": "iam/adapters/foo-inmemory", "trait": "FooStore" }]);
        let observed = json!({ "member_crates_found": 1000, "members": [] });
        let findings = evaluate_keyed(&policy(), &baseline, &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "PP-STALE-BASELINE"
                    && f.key == "iam/adapters/foo-inmemory:FooStore"),
            "a relocated baseline entry must become PP-STALE-BASELINE: {findings:#?}"
        );
    }

    #[test]
    fn stale_allowlist_entry_is_reported() {
        let mut policy = policy();
        policy["allowlist"] =
            json!([{ "member_path": "cap/adapters/ghost", "trait": "GhostStore", "reason": "x" }]);
        let observed = json!({ "member_crates_found": 1000, "members": [] });
        let findings = evaluate_keyed(&policy, &empty_baseline(), &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "PP-STALE-ALLOWLIST" && f.key == "cap/adapters/ghost:GhostStore"),
            "an unused allowlist entry must be PP-STALE-ALLOWLIST: {findings:#?}"
        );
    }

    #[test]
    fn empty_scan_fails_closed() {
        let observed = json!({ "member_crates_found": 0, "members": [] });
        let policy_with_floor = json!({
            "gate_id": GATE_ID,
            "forbidden_layer_dirs": ["adapters"],
            "port_name_suffixes": ["Store"],
            "min_expected_member_crates": 100,
            "allowlist": []
        });
        let findings = evaluate_keyed(&policy_with_floor, &empty_baseline(), &observed);
        assert!(
            findings.iter().any(|f| f.code == "PP-EMPTY-SCAN"),
            "a member count below the floor must fail closed: {findings:#?}"
        );
    }

    #[test]
    fn gate_id_mismatch_fails_closed() {
        let mut policy = policy();
        policy["gate_id"] = json!("wrong-id");
        let findings = evaluate_keyed(
            &policy,
            &empty_baseline(),
            &json!({ "member_crates_found": 1000, "members": [] }),
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == "PP-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn empty_suffix_set_fails_closed() {
        let mut policy = policy();
        policy["port_name_suffixes"] = json!([]);
        let findings = evaluate_keyed(
            &policy,
            &empty_baseline(),
            &json!({ "member_crates_found": 1000, "members": [] }),
        );
        assert!(
            findings.iter().any(|f| f.code == "PP-POLICY-MALFORMED"),
            "an empty suffix set must fail closed, not silently match nothing: {findings:#?}"
        );
    }

    #[test]
    fn malformed_baseline_fails_closed() {
        let baseline = json!([{ "member_path": "cap/adapters/x" }]); // missing `trait`
        let findings = evaluate_keyed(
            &policy(),
            &baseline,
            &json!({ "member_crates_found": 1000, "members": [] }),
        );
        assert!(findings.iter().any(|f| f.code == "PP-POLICY-MALFORMED"));
    }

    #[test]
    fn baseline_object_form_is_accepted() {
        // The generated face carries a header object with a `baseline` array.
        let baseline = json!({
            "gate_id": GATE_ID,
            "baseline": [{ "member_path": "iam/adapters/foo-inmemory", "trait": "FooStore" }]
        });
        let observed = observed_adapter(
            "foo-inmemory",
            "iam/adapters/foo-inmemory",
            &[("FooStore", "iam/adapters/foo-inmemory/src/lib.rs")],
        );
        let findings = evaluate_keyed(&policy(), &baseline, &observed);
        assert!(
            !findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"),
            "the object-form baseline must be honored: {findings:#?}"
        );
    }

    #[test]
    fn forbidden_layer_segment_is_whole_segment_not_substring() {
        assert!(path_in_forbidden_layer(
            "iam/adapters/foo",
            &["adapters".to_owned()]
        ));
        assert!(!path_in_forbidden_layer(
            "iam/my-adapters-helper/foo",
            &["adapters".to_owned()]
        ));
        assert!(!path_in_forbidden_layer(
            "iam/core/foo",
            &["adapters".to_owned()]
        ));
    }

    #[test]
    fn suggested_core_crate_uses_capability_stem() {
        assert_eq!(
            suggested_core_crate("iam/adapters/tenant-rbac-storage-inmemory"),
            "iam/core/<port-crate>"
        );
    }

    // --- FIX 1: adapter-NAMED crates (outside adapters/ dir) are caught ---

    #[test]
    fn adapter_named_crate_outside_adapters_dir_is_flagged() {
        // A crate named `*-adapter-inmemory` that lives outside any `adapters/` segment must be
        // caught by `forbidden_crate_name_suffixes` (FIX 1). This is the
        // `oya-payroll-run-storage-adapter-inmemory` shape: not under `adapters/` but named
        // `*-adapter-inmemory`.
        let policy = json!({
            "gate_id": GATE_ID,
            "forbidden_layer_dirs": ["adapters"],
            "forbidden_crate_name_suffixes": ["-adapter-inmemory"],
            "port_name_suffixes": ["StoragePort", "Repository", "Store", "Repo"],
            "min_expected_member_crates": 1,
            "allowlist": []
        });
        let observed = json!({
            "member_crates_found": 1000,
            "members": [{
                "crate": "oya-payroll-run-storage-adapter-inmemory",
                "member_path": "oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory",
                "traits": [{ "name": "PayrollRunStoragePort",
                             "file": "oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory/src/lib.rs" }]
            }]
        });
        let findings = evaluate_keyed(&policy, &empty_baseline(), &observed);
        assert!(
            findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"
                && f.key
                    == "oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory:PayrollRunStoragePort"),
            "adapter-named crate outside adapters/ must be flagged: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy, &empty_baseline(), &observed).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn crate_in_forbidden_layer_matches_suffix_not_substring() {
        // Suffix check is applied to the TAIL (last segment) only — never a substring of an
        // intermediate dir.
        assert!(crate_in_forbidden_layer(
            "oya/payroll/crates/oya-payroll-run-storage-adapter-inmemory",
            &["-adapter-inmemory".to_owned()]
        ));
        // A crate in a dir that contains `-adapter-inmemory` as an INTERMEDIATE segment must not
        // match — only the last segment is tested.
        assert!(!crate_in_forbidden_layer(
            "oya/payroll/crates-adapter-inmemory/payroll-run-core",
            &["-adapter-inmemory".to_owned()]
        ));
        // A core crate must not match.
        assert!(!crate_in_forbidden_layer(
            "oya/payroll/crates/payroll-run-app",
            &["-adapter-inmemory".to_owned()]
        ));
    }

    // --- FIX 2: bare `Port` suffix is NOT in the default set ---

    #[test]
    fn behavioral_port_trait_is_not_flagged() {
        // `ClockPort`, `ProviderAuthPort`, `OperatorAlertPort` etc. end in `Port` but are
        // behavioral, not storage-shaped. With `Port` removed from port_name_suffixes (FIX 2)
        // they must NOT be flagged.
        let observed = observed_adapter(
            "foo-adapter",
            "iam/adapters/foo-adapter",
            &[
                ("ClockPort", "iam/adapters/foo-adapter/src/lib.rs"),
                ("ProviderAuthPort", "iam/adapters/foo-adapter/src/lib.rs"),
                ("OperatorAlertPort", "iam/adapters/foo-adapter/src/lib.rs"),
            ],
        );
        // policy() uses suffixes without bare "Port"
        let findings = evaluate_keyed(&policy(), &empty_baseline(), &observed);
        assert!(
            !findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"),
            "behavioral *Port traits must NOT be flagged when `Port` is not in suffix set: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &empty_baseline(), &observed).verdict,
            Verdict::Green
        );
    }

    // --- FIX 3: member_path key prevents same-tail collision masking ---

    #[test]
    fn same_crate_tail_different_member_path_no_collision() {
        // Two sibling crates both named `rest` in different capability trees: one is baselined
        // (intelligence/adapters/rest:SecretProviderStore), the other has a NEW violation
        // (payments/adapters/rest:PaymentStore). The new one must surface as RED even though
        // both crates share the tail `rest`. Under the old crate_tail key they would collide;
        // under the member_path key they are distinct.
        let baseline = json!([{ "member_path": "intelligence/adapters/rest", "trait": "SecretProviderStore" }]);
        let observed = json!({
            "member_crates_found": 1000,
            "members": [
                {
                    "crate": "rest",
                    "member_path": "intelligence/adapters/rest",
                    "traits": [{ "name": "SecretProviderStore",
                                 "file": "intelligence/adapters/rest/src/lib.rs" }]
                },
                {
                    "crate": "rest",
                    "member_path": "payments/adapters/rest",
                    "traits": [{ "name": "PaymentStore",
                                 "file": "payments/adapters/rest/src/lib.rs" }]
                }
            ]
        });
        let findings = evaluate_keyed(&policy(), &baseline, &observed);
        // The baselined one must stay green.
        assert!(
            !findings
                .iter()
                .any(|f| f.key == "intelligence/adapters/rest:SecretProviderStore"),
            "the baselined entry must stay green: {findings:#?}"
        );
        // The NEW violation in the sibling `rest` crate must be RED.
        assert!(
            findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"
                && f.key == "payments/adapters/rest:PaymentStore"),
            "a NEW violation in a same-tail sibling crate must be RED: {findings:#?}"
        );
        assert_eq!(
            evaluate(&policy(), &baseline, &observed).verdict,
            Verdict::Red
        );
    }
}
