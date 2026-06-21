//! # cloud-ci-no-graphql-without-adr (ADR-0565)
//!
//! The zero-GraphQL reintroduction gate. ADR-0565 (founder 2026-06-21, door: one-way) removes ALL
//! GraphQL from the owned API surface: the canonical surface set is REST + gRPC + AsyncAPI + realtime
//! (SSE / WebSocket / gRPC-streaming), and GraphQL returns ONLY by a future ADR that explicitly
//! reverses ADR-0565. The drop PR (#775 / ADR-0565) deletes every GraphQL artifact; THIS gate is the
//! enforcement half (enforcement-layering doctrine: the drop is the construction, the gate is the
//! backstop that prevents recurrence). It fails CLOSED if a change reintroduces, repo-wide, WITHOUT
//! referencing an accepted authorizing ADR id, EITHER:
//! - a GraphQL execution/parse library in any `Cargo.toml` (`async-graphql`, `juniper`,
//!   `graphql-parser`, `cynic`, `apollo-*`, … — the forbidden set is DATA), OR
//! - any `.graphql` / `.gql` / `*.sdl` GraphQL schema file (added or edited).
//!
//! ## Candidate-tree evaluation, NOT a frozen merge-base
//! The gate scans the CANDIDATE tree directly — the live workspace `Cargo.toml` manifests resolved
//! via `oya-workspace-members-kernel` (glob-aware; NO `cargo metadata`/`buck2` shell-out) plus a
//! read-only walk of the candidate tree for GraphQL schema files. It does NOT diff a frozen
//! merge-base baseline. This is deliberate: a frozen-baseline predicate evaluated at PR-tier against
//! the merge-base but at push-tier against the integrated tip is the documented PR/push
//! baseline-asymmetry false-green (gate-baseline-pr-push-asymmetry memo) — a GraphQL artifact added
//! on dev between branch-point and merge would pass PR-tier and only fail on the integrated tip.
//! Evaluating the candidate tree means the verdict is the SAME at PR-tier and push-tier: any GraphQL
//! artifact present in the tree (without an authorizing ADR ref) is RED, full stop.
//!
//! ## Frozen baseline = EMPTY
//! The drop PR leaves the tree GraphQL-free, so this gate ships born-blocking with an EMPTY baseline:
//! there is no shrink-only legacy debt. Any NEW GraphQL artifact fails closed on arrival.
//!
//! ## ADR escape-hatch
//! GraphQL is admissible ONLY via a future ADR that explicitly reverses ADR-0565 (ADR-0565
//! "Decision"). The gate honors that: a forbidden artifact that REFERENCES an accepted authorizing
//! ADR id — an `ADR-NNNN` citation other than the forbidding ADR itself — is allowed. The reference
//! must be a DIFFERENT ADR than the forbidding ADR (`policy.forbidding_adr`) so a file cannot launder
//! itself by merely mentioning ADR-0565 (the rule it would be violating); it must cite the reversing
//! decision. This is the construction-over-flag escape: a real reversal is a reviewed ADR, and the
//! gate reads its citation, never a bare suppression comment.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the forbidden crate set (exact + prefix
//! rules), the GraphQL schema extensions, the forbidding-ADR id, the workspace-member floor — are
//! DATA in `no-graphql-without-adr-policy.json`. Nothing oyatie-specific is hardcoded in Rust; a
//! different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_graphql_artifacts`] `(root, policy) -> observed` is the ONLY I/O: read-only `fs`
//!   reads of the candidate tree (member `Cargo.toml` manifests + a `.graphql`/`.gql`/`.sdl` walk).
//!   No shell, no network, no VCS. Writes no temp files.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it applies the forbidden set + the ADR escape-hatch to the observed artifacts.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `NGQL-FORBIDDEN-LIB`     — a GraphQL execution/parse library is declared in a `Cargo.toml`
//!   dependency table without an authorizing ADR reference.
//! - `NGQL-SCHEMA-FILE`       — a `.graphql`/`.gql`/`.sdl` GraphQL schema file is present without an
//!   authorizing ADR reference.
//! - `NGQL-EMPTY-SCAN`        — the workspace member census is below the policy floor (catches a
//!   broken CWD / member-glob that would otherwise be a silent false-green).
//! - `NGQL-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `NGQL-POLICY-MALFORMED`  — the policy `forbidden_crates` / `schema_extensions` is missing or
//!   malformed (fail-closed: the gate would have nothing to enforce).
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
pub const GATE_ID: &str = "cloud-ci-no-graphql-without-adr";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 5] = [
    "NGQL-FORBIDDEN-LIB",
    "NGQL-SCHEMA-FILE",
    "NGQL-EMPTY-SCAN",
    "NGQL-POLICY-GATE-ID-MISMATCH",
    "NGQL-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-artifact.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only, hermetic — no shell / network / VCS)
// ---------------------------------------------------------------------------

/// Errors collecting the observed GraphQL-artifact view. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — a malformed manifest or unreadable tree is a
/// fail-closed error, never a silently skipped artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// Workspace-member resolution failed (the candidate-tree member universe is unknown).
    ResolveMembers(String),
    /// A read-only filesystem operation failed (the candidate tree could not be scanned).
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::ResolveMembers(message) => {
                write!(f, "no-graphql-without-adr resolve workspace members: {message}")
            }
            CollectError::Io(message) => write!(f, "no-graphql-without-adr io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// The forbidden GraphQL crate rules declared in policy DATA. Each rule denies a crate name either
/// EXACTLY or as a PREFIX (so `apollo-*` covers `apollo-router`, `apollo-compiler`, …). Returns the
/// rules in canonical (sorted-by-name) order; `None` if the `forbidden_crates` list is absent.
fn forbidden_crate_rules(policy: &Value) -> Option<Vec<CrateRule>> {
    let list = policy.get("forbidden_crates").and_then(Value::as_array)?;
    let mut out = Vec::new();
    for entry in list {
        let Some(name) = entry.get("crate").and_then(Value::as_str) else {
            continue;
        };
        let prefix = entry.get("match").and_then(Value::as_str) == Some("prefix");
        out.push(CrateRule {
            name: name.to_owned(),
            prefix,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Some(out)
}

/// One forbidden-crate rule: a crate name matched EXACTLY or as a PREFIX.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CrateRule {
    name: String,
    prefix: bool,
}

impl CrateRule {
    fn matches(&self, dep: &str) -> bool {
        if self.prefix {
            dep.starts_with(&self.name)
        } else {
            dep == self.name
        }
    }
}

/// The GraphQL schema-file extensions declared in policy DATA (lowercased, no leading dot), in
/// canonical order; `None` if the list is absent.
fn schema_extensions(policy: &Value) -> Option<Vec<String>> {
    let list = policy.get("schema_extensions").and_then(Value::as_array)?;
    let mut out: BTreeSet<String> = BTreeSet::new();
    for entry in list {
        if let Some(ext) = entry.as_str() {
            out.insert(ext.trim_start_matches('.').to_ascii_lowercase());
        }
    }
    Some(out.into_iter().collect())
}

/// Collect the candidate-tree GraphQL-artifact view the policy asks about.
///
/// Two read-only scans of the candidate tree — NO shell, NO network, NO VCS:
/// 1. Each resolved workspace member's `Cargo.toml`: its declared dependency-table crate names plus
///    whether that manifest references an authorizing ADR.
/// 2. A recursive walk for `.graphql`/`.gql`/`.sdl` schema files (skipping the policy-declared
///    `excluded_dirs`, e.g. `third-party/`), each tagged with whether it references an authorizing
///    ADR (in its own contents or in a sibling marker).
///
/// Emits:
/// `{ "workspace_members_found": <usize>,
///    "manifests": [ { "member_path", "deps":[<name>..], "references_authorizing_adr": <bool> } ],
///    "schema_files": [ { "path", "references_authorizing_adr": <bool> } ] }`.
pub fn collect_graphql_artifacts(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let forbidding_adr = forbidding_adr(policy);
    let exts = schema_extensions(policy).unwrap_or_default();
    let excluded = excluded_dirs(policy);

    // --- (1) member Cargo.toml manifests ---
    let member_dirs =
        resolve_member_dirs(root).map_err(|error| CollectError::ResolveMembers(error.to_string()))?;
    let members_found = member_dirs.len();
    let mut manifests = Vec::new();
    for member_dir in &member_dirs {
        let cargo_path = root.join(member_dir).join("Cargo.toml");
        let text = match fs::read_to_string(&cargo_path) {
            Ok(text) => text,
            // A member dir without a readable Cargo.toml contributes no manifest row (the
            // member resolver only returns dirs that hold one; a transient read miss is skipped
            // rather than failing the whole scan, since the file-walk leg still covers schema
            // files there).
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(CollectError::Io(format!("read {}: {e}", cargo_path.display())));
            }
        };
        let deps = parse_manifest_dep_names(&text);
        let references_authorizing_adr = references_authorizing_adr(&text, forbidding_adr.as_deref());
        manifests.push(json!({
            "member_path": member_dir,
            "deps": deps.into_iter().collect::<Vec<_>>(),
            "references_authorizing_adr": references_authorizing_adr,
        }));
    }

    // --- (2) GraphQL schema files anywhere in the candidate tree ---
    let mut schema_files: Vec<Value> = Vec::new();
    collect_schema_files(root, root, &exts, &excluded, forbidding_adr.as_deref(), &mut schema_files)?;
    schema_files.sort_by(|a, b| {
        a.get("path")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(b.get("path").and_then(Value::as_str).unwrap_or_default())
    });

    Ok(json!({
        "workspace_members_found": members_found,
        "manifests": manifests,
        "schema_files": schema_files,
    }))
}

/// The forbidding-ADR id (`policy.forbidding_adr`, e.g. `ADR-0565`) — the decision this gate enforces.
/// A forbidden artifact cannot launder itself by citing THIS id; the escape-hatch requires citing a
/// DIFFERENT (reversing/authorizing) ADR.
fn forbidding_adr(policy: &Value) -> Option<String> {
    policy
        .get("forbidding_adr")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// The directory prefixes the scan skips (policy DATA, e.g. `third-party/`, `.git/`). A vendored
/// third-party crate's `.graphql`/`.gql`/`.sdl` test fixtures are not part of the OWNED API surface
/// ADR-0565 governs, so they are excluded by DATA, not by a Rust hardcode.
fn excluded_dirs(policy: &Value) -> Vec<String> {
    policy
        .get("excluded_dirs")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(|d| d.trim_end_matches('/').to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `text` references an AUTHORIZING ADR — an `ADR-NNNN` citation other than the forbidding
/// ADR. The escape-hatch (ADR-0565: GraphQL admissible only via a reversing ADR): a file may carry
/// GraphQL iff it cites the reversing decision. A mention of the forbidding ADR itself (the rule it
/// would be violating) does NOT count — otherwise any file could self-launder by naming ADR-0565.
fn references_authorizing_adr(text: &str, forbidding_adr: Option<&str>) -> bool {
    adr_citations(text)
        .into_iter()
        .any(|cited| Some(cited.as_str()) != forbidding_adr)
}

/// Extract every `ADR-NNNN` citation from `text` (4+ digits after `ADR-`). Deterministic, pure;
/// case-sensitive on the `ADR-` prefix (the canonical decision-id form in this repo).
fn adr_citations(text: &str) -> BTreeSet<String> {
    let marker = "ADR-";
    let mut out = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(marker) {
        let digits_start = from + rel + marker.len();
        let mut end = digits_start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        // Require at least four digits (the canonical `ADR-NNNN` shape) so a stray `ADR-` token is
        // not a false citation.
        if end - digits_start >= 4 {
            out.insert(format!("{marker}{}", &text[digits_start..end]));
        }
        from = digits_start.max(from + rel + 1);
    }
    out
}

/// Parse the crate names declared in a `Cargo.toml`'s dependency tables —
/// `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and the `[target.*.*]` variants.
/// Honors `package = "<real>"` renames (denies on the REAL crate name, so a rename cannot smuggle a
/// forbidden lib). dev-dependencies ARE scanned: a GraphQL lib reintroduced as a dev-dep is still a
/// reintroduction of the surface ADR-0565 forbids. Pure helper, exposed for tests.
pub fn parse_manifest_dep_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(doc) = text.parse::<toml::Value>() else {
        // A manifest that does not parse contributes no deps from this leg; the gate is not a TOML
        // linter (manifest-hygiene owns that). Over-approximating here is unnecessary because a
        // forbidden lib in a broken manifest would also break the build.
        return names;
    };
    collect_dep_table_names(doc.get("dependencies"), &mut names);
    collect_dep_table_names(doc.get("dev-dependencies"), &mut names);
    collect_dep_table_names(doc.get("build-dependencies"), &mut names);
    if let Some(targets) = doc.get("target").and_then(toml::Value::as_table) {
        for target_cfg in targets.values() {
            collect_dep_table_names(target_cfg.get("dependencies"), &mut names);
            collect_dep_table_names(target_cfg.get("dev-dependencies"), &mut names);
            collect_dep_table_names(target_cfg.get("build-dependencies"), &mut names);
        }
    }
    names
}

/// Collect crate names from one dependency table into `names`, honoring `package = "<real>"`.
fn collect_dep_table_names(table: Option<&toml::Value>, names: &mut BTreeSet<String>) {
    let Some(table) = table.and_then(toml::Value::as_table) else {
        return;
    };
    for (dep_key, spec) in table {
        let real = spec
            .as_table()
            .and_then(|t| t.get("package").and_then(toml::Value::as_str))
            .unwrap_or(dep_key.as_str());
        names.insert(real.to_owned());
    }
}

/// Recursively walk `dir` collecting GraphQL schema files (by `exts`), skipping `excluded` directory
/// prefixes (relative to `root`). Each row carries its repo-relative path and whether it (or a
/// sibling `<file>.adr` marker) references an authorizing ADR. Read-only; missing dirs are skipped.
fn collect_schema_files(
    root: &Path,
    dir: &Path,
    exts: &[String],
    excluded: &[String],
    forbidding_adr: Option<&str>,
    out: &mut Vec<Value>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy();
        // Skip the always-irrelevant VCS dir and any policy-excluded prefix.
        if rel_str.starts_with(".git/") || rel_str == ".git" {
            continue;
        }
        if excluded.iter().any(|ex| rel_str == *ex || rel_str.starts_with(&format!("{ex}/"))) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            collect_schema_files(root, &path, exts, excluded, forbidding_adr, out)?;
        } else if has_graphql_ext(&rel_str, exts) {
            let body = fs::read_to_string(&path).unwrap_or_default();
            // The schema file may cite the authorizing ADR in its OWN body (a generated header
            // comment) OR in a sibling `<file>.adr` marker file alongside it.
            let marker = path.with_extension(format!(
                "{}.adr",
                path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ));
            let marker_body = fs::read_to_string(&marker).unwrap_or_default();
            let references_authorizing_adr =
                references_authorizing_adr(&body, forbidding_adr)
                    || references_authorizing_adr(&marker_body, forbidding_adr);
            out.push(json!({
                "path": rel_str,
                "references_authorizing_adr": references_authorizing_adr,
            }));
        }
    }
    Ok(())
}

/// Whether a path's extension is one of the GraphQL schema extensions (case-insensitive).
fn has_graphql_ext(path: &str, exts: &[String]) -> bool {
    let lower = path.to_ascii_lowercase();
    exts.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
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

/// Pure evaluator. `policy` is DATA (`no-graphql-without-adr-policy.json`); `observed` is the
/// candidate-tree GraphQL-artifact view shaped by [`collect_graphql_artifacts`].
///
/// RED iff the candidate tree carries a forbidden GraphQL library (in any member `Cargo.toml`) or a
/// GraphQL schema file, in EITHER case WITHOUT the artifact referencing an authorizing ADR. The
/// frozen baseline is EMPTY (the tree is GraphQL-free post-drop), so any such artifact fails closed.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "NGQL-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a missing/empty forbidden set or schema-extension set rather than silently
    // passing with nothing to enforce.
    let Some(crate_rules) = forbidden_crate_rules(policy) else {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden_crates` must be a non-null array of {crate, match?} entries; correct the policy before the gate can evaluate",
        ));
        return findings;
    };
    if crate_rules.is_empty() {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden_crates` resolved to zero crate names; the gate would have nothing to enforce — correct the policy",
        ));
        return findings;
    }
    let Some(exts) = schema_extensions(policy) else {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `schema_extensions` must be a non-null array of extension strings (e.g. \"graphql\", \"gql\", \"sdl\"); correct the policy",
        ));
        return findings;
    };
    if exts.is_empty() {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `schema_extensions` resolved to zero extensions; the gate would not catch any schema file — correct the policy",
        ));
        return findings;
    }

    let min_expected = policy
        .get("min_expected_workspace_members")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let members = observed
        .get("workspace_members_found")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if members < min_expected {
        findings.insert(Finding::new(
            "NGQL-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "workspace member census {members} is below the policy floor of {min_expected}; the CWD or the member glob is likely broken (fail-closed against a silent false-green where the scan saw an empty tree)"
            ),
        ));
    }

    let forbidding = policy
        .get("forbidding_adr")
        .and_then(Value::as_str)
        .unwrap_or("ADR-0565");

    // --- forbidden GraphQL libraries in member Cargo.toml manifests ---
    let manifests = observed
        .get("manifests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for manifest in &manifests {
        let member_path = manifest
            .get("member_path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown-member>");
        let references_adr = manifest
            .get("references_authorizing_adr")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let deps = manifest
            .get("deps")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        for dep in deps {
            let Some(rule) = crate_rules.iter().find(|rule| rule.matches(dep)) else {
                continue;
            };
            if references_adr {
                // Escape-hatch: this manifest cites a reversing/authorizing ADR — allowed.
                continue;
            }
            let match_note = if rule.prefix {
                format!(" (matched the forbidden prefix `{}`)", rule.name)
            } else {
                String::new()
            };
            findings.insert(Finding::new(
                "NGQL-FORBIDDEN-LIB",
                &format!("{member_path}/Cargo.toml:{dep}"),
                format!(
                    "`{member_path}/Cargo.toml` declares the GraphQL library `{dep}`{match_note}, which {forbidding} forbids in the owned stack (the canonical API surface is REST + gRPC + AsyncAPI + realtime). Remove the dependency. GraphQL is admissible ONLY via a future ADR that explicitly reverses {forbidding}; if such an ADR is accepted, cite its id in this Cargo.toml to authorize the dependency."
                ),
            ));
        }
    }

    // --- GraphQL schema files anywhere in the candidate tree ---
    let schema_files = observed
        .get("schema_files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for file in &schema_files {
        let path = file
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown-path>");
        let references_adr = file
            .get("references_authorizing_adr")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if references_adr {
            continue;
        }
        findings.insert(Finding::new(
            "NGQL-SCHEMA-FILE",
            path,
            format!(
                "`{path}` is a GraphQL schema file, which {forbidding} forbids in the owned stack (the canonical API surface is REST + gRPC + AsyncAPI + realtime). Remove the file. GraphQL is admissible ONLY via a future ADR that explicitly reverses {forbidding}; if such an ADR is accepted, cite its id in the schema file (or a sibling `{path}.adr` marker) to authorize it."
            ),
        ));
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its detail.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "no-graphql-without-adr gate passed: the candidate tree carries no GraphQL library nor schema file (the owned stack is GraphQL-free — ADR-0565)".to_owned();
    }
    let mut out = String::from("no-graphql-without-adr gate failed (ADR-0565):\n");
    for finding in findings {
        out.push_str(&format!(
            "    - {} {}\n        {}\n",
            finding.code, finding.key, finding.detail
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "forbidding_adr": "ADR-0565",
            "min_expected_workspace_members": 1,
            "forbidden_crates": [
                {"crate": "async-graphql", "match": "prefix"},
                {"crate": "async_graphql", "match": "prefix"},
                {"crate": "juniper", "match": "exact"},
                {"crate": "graphql-parser", "match": "exact"},
                {"crate": "graphql_parser", "match": "exact"},
                {"crate": "cynic", "match": "exact"},
                {"crate": "apollo-", "match": "prefix"}
            ],
            "schema_extensions": ["graphql", "gql", "sdl"],
            "excluded_dirs": ["third-party/"]
        })
    }

    fn observed(members: u64, manifests: Value, schema_files: Value) -> Value {
        json!({
            "workspace_members_found": members,
            "manifests": manifests,
            "schema_files": schema_files,
        })
    }

    #[test]
    fn green_on_a_clean_tree() {
        // The post-drop tree: no GraphQL lib, no schema file. The gate PASSES.
        let report = evaluate(
            &policy(),
            &observed(
                500,
                json!([{"member_path": "cloud/iam/pdp", "deps": ["serde", "tokio"], "references_authorizing_adr": false}]),
                json!([]),
            ),
        );
        assert_eq!(report.verdict, Verdict::Green, "clean tree ⇒ green");
        assert!(report.violations.is_empty());
        assert!(render_findings(&evaluate_keyed(&policy(), &observed(500, json!([]), json!([]))))
            .contains("passed"));
    }

    #[test]
    fn red_when_a_cargo_toml_adds_async_graphql() {
        let observed = observed(
            500,
            json!([{"member_path": "oya/studio/graphql", "deps": ["async-graphql", "serde"], "references_authorizing_adr": false}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-FORBIDDEN-LIB")
            .unwrap_or_else(|| panic!("async-graphql must be RED: {findings:?}"));
        assert_eq!(f.key, "oya/studio/graphql/Cargo.toml:async-graphql");
        assert!(f.detail.contains("ADR-0565"), "remediation must name the forbidding ADR: {f:?}");
        assert!(f.detail.contains("Remove the dependency"), "remediation must say how to fix: {f:?}");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn red_on_a_prefixed_apollo_crate() {
        let observed = observed(
            500,
            json!([{"member_path": "cloud/gw", "deps": ["apollo-router"], "references_authorizing_adr": false}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.ends_with("apollo-router")),
            "a prefixed apollo-* crate must be RED: {findings:?}"
        );
    }

    #[test]
    fn red_when_a_graphql_schema_file_is_present() {
        let observed = observed(
            500,
            json!([]),
            json!([{"path": "oya/analytics/contracts/graphql-v1.sdl", "references_authorizing_adr": false}]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-SCHEMA-FILE")
            .unwrap_or_else(|| panic!("a .sdl schema file must be RED: {findings:?}"));
        assert_eq!(f.key, "oya/analytics/contracts/graphql-v1.sdl");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn green_when_a_forbidden_lib_references_an_authorizing_adr() {
        // The escape-hatch: a manifest that cites a reversing/authorizing ADR (a DIFFERENT ADR than
        // the forbidding one) is allowed to carry GraphQL.
        let observed = observed(
            500,
            json!([{"member_path": "cloud/gw", "deps": ["async-graphql"], "references_authorizing_adr": true}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            !findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
            "an ADR-referenced GraphQL lib must be allowed: {findings:?}"
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Green);
    }

    #[test]
    fn green_when_a_schema_file_references_an_authorizing_adr() {
        let observed = observed(
            500,
            json!([]),
            json!([{"path": "oya/analytics/contracts/graphql-v2.graphql", "references_authorizing_adr": true}]),
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Green);
    }

    #[test]
    fn mentioning_only_the_forbidding_adr_does_not_self_launder() {
        // A file that cites ONLY the forbidding ADR (ADR-0565 — the rule it would be violating) does
        // NOT escape: references_authorizing_adr is computed against the forbidding id.
        assert!(!references_authorizing_adr("see ADR-0565", Some("ADR-0565")));
        // Citing a DIFFERENT (reversing) ADR escapes.
        assert!(references_authorizing_adr("reintroduced per ADR-0700 reversing ADR-0565", Some("ADR-0565")));
    }

    #[test]
    fn empty_scan_fails_closed() {
        let findings = evaluate_keyed(&policy(), &observed(0, json!([]), json!([])));
        assert!(
            findings.iter().any(|f| f.code == "NGQL-EMPTY-SCAN"),
            "a below-floor member census must trip NGQL-EMPTY-SCAN: {findings:?}"
        );
    }

    #[test]
    fn policy_gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = Value::from("wrong-id");
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(findings.iter().any(|f| f.code == "NGQL-POLICY-GATE-ID-MISMATCH"));
    }

    #[test]
    fn malformed_policy_with_no_forbidden_list_fails_closed() {
        let p = json!({ "gate_id": GATE_ID, "schema_extensions": ["graphql"] });
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(
            findings.iter().any(|f| f.code == "NGQL-POLICY-MALFORMED"),
            "a missing forbidden_crates list must fail closed: {findings:?}"
        );
    }

    #[test]
    fn malformed_policy_with_no_schema_extensions_fails_closed() {
        let p = json!({
            "gate_id": GATE_ID,
            "forbidden_crates": [{"crate": "juniper", "match": "exact"}]
        });
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(findings.iter().any(|f| f.code == "NGQL-POLICY-MALFORMED"));
    }

    #[test]
    fn parse_manifest_dep_names_covers_all_tables_and_renames() {
        let manifest = r#"
[package]
name = "x"
[dependencies]
serde = "1"
gql = { package = "async-graphql", version = "7" }
[dev-dependencies]
juniper = "0.16"
[build-dependencies]
graphql-parser = "0.4"
[target.'cfg(unix)'.dependencies]
cynic = "3"
"#;
        let names = parse_manifest_dep_names(manifest);
        // The rename is denied on the REAL crate name.
        assert!(names.contains("async-graphql"), "rename must resolve to real name: {names:?}");
        assert!(names.contains("juniper"));
        assert!(names.contains("graphql-parser"));
        assert!(names.contains("cynic"));
        assert!(names.contains("serde"));
        // The local rename key is NOT what we deny on.
        assert!(!names.contains("gql"));
    }

    #[test]
    fn adr_citations_requires_four_digits() {
        let cited = adr_citations("ADR-0565 and ADR-12 and ADR-0700 and ADR-1234");
        assert!(cited.contains("ADR-0565"));
        assert!(cited.contains("ADR-0700"));
        assert!(cited.contains("ADR-1234"));
        // Fewer than four digits is not a canonical citation.
        assert!(!cited.contains("ADR-12"));
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let obs = observed(
            500,
            json!([{"member_path": "a", "deps": ["juniper"], "references_authorizing_adr": false}]),
            json!([{"path": "b.graphql", "references_authorizing_adr": false}]),
        );
        let projected: BTreeSet<String> =
            evaluate_keyed(&policy(), &obs).into_iter().map(|f| f.code).collect();
        assert_eq!(evaluate(&policy(), &obs).violations, projected);
    }
}
