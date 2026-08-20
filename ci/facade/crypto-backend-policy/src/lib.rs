//! # cloud-ci-crypto-backend-purity (ADR-0506)
//!
//! The crypto-backend-purity gate. ADR-0506 mandates `aws-lc-rs` as the SINGLE canonical Phase-1
//! crypto backend and FORBIDS `ring`. The ADR DESCRIBES the zero-ring invariant — the forbidden
//! backend must be absent from the Buck2 build graph — but NOTHING mechanically enforced it.
//! This gate is that enforcement (founder doctrine: flag-only/manual = incomplete; construction >
//! reaction; automate everything automatable).
//!
//! ## The signal is feature-resolved ACTIVATION, not the dependency SUPERSET
//! This is the load-bearing distinction the gate exists to draw correctly.
//!
//! `Cargo.lock` may retain a `ring 0.17` stanza as an UNACTIVATED optional-dependency phantom:
//! a disabled feature can keep a resolved version in the lock without compiling that backend.
//! Lock text is therefore not authority. This gate reads the local Buck graph instead:
//! first-party BUCK files provide the build roots, generated `third-party/BUCK` provides the
//! vendored crate edges, and [`evaluate_keyed`] fails iff a forbidden backend is reachable from
//! a first-party Buck target. The gate thus distinguishes an ACTIVATED ring (FAIL) from the
//! documented lock-superset phantom (OK) without touching the network or invoking Cargo.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the forbidden-backend set, the mandated
//! backend, the package-census floor — are DATA in `crypto-backend-purity-policy.json`. Nothing
//! oyatie-specific is hardcoded in Rust; a different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_activated_backends`] `(root, policy) -> observed` is the ONLY I/O: it reads local
//!   BUCK files and generated `third-party/BUCK`. Read-only; writes no temp files; runs no
//!   subprocesses.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without
//!   a filesystem or subprocess; it applies the forbidden set to the observed activation view.
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Ratchet semantics
//! The workspace is ring-free in activation today (PR #725 / ADR-0506 G002 zero-ring), so the gate
//! ships frozen-empty: any new ACTIVATION (a rustls/sqlx feature flipped back to ring, reqwest's
//! http3/quinn enabled) fails closed. There is no shrink-only legacy baseline.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `CBP-FORBIDDEN-BACKEND-ACTIVATED` — a forbidden crypto backend has ≥1 ACTIVATED dependent in
//!   the local Buck graph rooted at first-party BUCK targets.
//! - `CBP-EMPTY-SCAN`                  — the workspace package census is below the policy floor
//!   (catches a broken repo-root invocation that would otherwise be a false-green).
//! - `CBP-POLICY-GATE-ID-MISMATCH`     — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `CBP-POLICY-MALFORMED`            — the policy `forbidden` list is missing/malformed.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

use oya_buck_syntax_kernel::{ExprNode, Stmt, call_strings, expr_strings, parse};
use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-crypto-backend-purity";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 4] = [
    "CBP-FORBIDDEN-BACKEND-ACTIVATED",
    "CBP-EMPTY-SCAN",
    "CBP-POLICY-GATE-ID-MISMATCH",
    "CBP-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-backend.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; local BUCK reads)
// ---------------------------------------------------------------------------

/// Errors collecting the observed activation graph. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — an unreadable or unparseable Buck graph is a
/// fail-closed error, never a silently skipped backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// A local graph file could not be read.
    Io { path: String, message: String },
    /// The local Buck graph could not be interpreted enough to enforce the policy.
    BuckGraph(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io { path, message } => {
                write!(f, "crypto-backend-purity read {path}: {message}")
            }
            CollectError::BuckGraph(message) => write!(
                f,
                "crypto-backend-purity Buck graph collection failed closed: {message}"
            ),
        }
    }
}

impl std::error::Error for CollectError {}

/// The forbidden crate names declared in policy DATA, in canonical (sorted) order.
pub fn forbidden_crates(policy: &Value) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    if let Some(list) = policy.get("forbidden").and_then(Value::as_array) {
        for entry in list {
            if let Some(name) = entry.get("crate").and_then(Value::as_str) {
                names.insert(name.to_owned());
            }
        }
    }
    names.into_iter().collect()
}

/// Collect the observed activation graph the policy asks about.
///
/// For each forbidden crate, walks from first-party BUCK references into generated
/// `third-party/BUCK` local labels and records reachable forbidden targets. Also probes the
/// first-party manifest census from local `Cargo.toml` files so a broken repo root fails closed via
/// `CBP-EMPTY-SCAN` rather than passing as a false-green. Emits:
/// `{ "workspace_packages_found": <usize>, "backends": [ { "crate": <name>,
///    "activated_dependents": [ <line>, .. ] } ] }`.
pub fn collect_activated_backends(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let census = workspace_package_count(root)?;
    let graph = ThirdPartyGraph::load(root)?;
    let roots = collect_first_party_third_party_roots(root)?;

    let mut backends = Vec::new();
    for crate_name in forbidden_crates(policy) {
        let activated = graph.activated_dependents_of(&roots, &crate_name);
        backends.push(json!({
            "crate": crate_name,
            "activated_dependents": activated,
        }));
    }

    Ok(json!({
        "workspace_packages_found": census,
        "backends": backends,
    }))
}

#[derive(Debug, Default)]
struct ThirdPartyGraph {
    deps_by_target: BTreeMap<String, BTreeSet<String>>,
    crate_names_by_target: BTreeMap<String, BTreeSet<String>>,
}

impl ThirdPartyGraph {
    fn load(root: &Path) -> Result<Self, CollectError> {
        let path = root.join("third-party/BUCK");
        let text = read_file(&path)?;
        parse(&text).map_err(|e| {
            CollectError::BuckGraph(format!(
                "{} could not be parsed as BUCK: {e}",
                path.display()
            ))
        })?;
        let mut graph = ThirdPartyGraph::default();
        for block in generated_buck_blocks(&text) {
            let Some(target) = generated_field(block, "name") else {
                continue;
            };
            graph
                .deps_by_target
                .entry(target.clone())
                .or_default()
                .extend(generated_local_deps(block));
            for crate_name in crate_names_implied_by_target(&target, block) {
                graph
                    .crate_names_by_target
                    .entry(target.clone())
                    .or_default()
                    .insert(crate_name);
            }
        }
        if graph.deps_by_target.is_empty() {
            return Err(CollectError::BuckGraph(format!(
                "{} contained no generated targets",
                path.display()
            )));
        }
        Ok(graph)
    }

    fn activated_dependents_of(
        &self,
        first_party_roots: &BTreeSet<String>,
        crate_name: &str,
    ) -> Vec<String> {
        let forbidden_targets = self.targets_for_crate(crate_name);
        let mut activated = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut visited = BTreeSet::new();

        for root in first_party_roots {
            queue.push_back((root.clone(), root.clone()));
        }

        while let Some((root, target)) = queue.pop_front() {
            if !visited.insert((root.clone(), target.clone())) {
                continue;
            }
            if forbidden_targets.contains(&target)
                || self.target_has_crate_name(&target, crate_name)
                || target_belongs_to_crate(&target, crate_name)
            {
                activated.insert(format!(
                    "third-party//:{target} reachable from first-party BUCK dependency third-party//:{root}"
                ));
                continue;
            }
            if let Some(deps) = self.deps_by_target.get(&target) {
                for dep in deps {
                    queue.push_back((root.clone(), dep.clone()));
                }
            }
        }

        activated.into_iter().collect()
    }

    fn targets_for_crate(&self, crate_name: &str) -> BTreeSet<String> {
        let mut targets = BTreeSet::from([crate_name.to_owned()]);
        for (target, names) in &self.crate_names_by_target {
            if names.contains(crate_name) || target_belongs_to_crate(target, crate_name) {
                targets.insert(target.clone());
            }
        }
        targets
    }

    fn target_has_crate_name(&self, target: &str, crate_name: &str) -> bool {
        self.crate_names_by_target
            .get(target)
            .is_some_and(|names| names.contains(crate_name))
    }
}

/// Probe the first-party package census from local manifests. A too-small census trips
/// `CBP-EMPTY-SCAN`, catching a broken repo root that would otherwise pass as a false-green.
fn workspace_package_count(root: &Path) -> Result<u64, CollectError> {
    let mut count = 0u64;
    visit_files(root, &mut |path| {
        if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
            count += 1;
        }
        Ok(())
    })?;
    Ok(count)
}

fn collect_first_party_third_party_roots(root: &Path) -> Result<BTreeSet<String>, CollectError> {
    let third_party = root.join("third-party");
    let mut roots = BTreeSet::new();
    visit_files(root, &mut |path| {
        if path.starts_with(&third_party) {
            return Ok(());
        }
        if path.file_name().and_then(|name| name.to_str()) != Some("BUCK") {
            return Ok(());
        }
        let text = read_file(path)?;
        collect_thirdparty_tokens_from_buck(path, &text, &mut roots)?;
        Ok(())
    })?;
    if roots.is_empty() {
        return Err(CollectError::BuckGraph(
            "no first-party BUCK references to third-party// targets were found".to_owned(),
        ));
    }
    Ok(roots)
}

fn collect_thirdparty_tokens_from_buck(
    path: &Path,
    text: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), CollectError> {
    match parse(text) {
        Ok(doc) => {
            doc.visit_calls(&mut |call| {
                if call.has_opaque() {
                    collect_thirdparty_tokens(call.span.slice(text), out);
                } else {
                    for s in call_strings(call) {
                        collect_thirdparty_tokens(&s, out);
                    }
                }
            });
            for stmt in &doc.stmts {
                match stmt {
                    Stmt::Assign { value, .. } => {
                        collect_thirdparty_tokens_from_expr(text, value, out);
                    }
                    Stmt::IndexAssign { key, value, .. } => {
                        collect_thirdparty_tokens_from_expr(text, key, out);
                        collect_thirdparty_tokens_from_expr(text, value, out);
                    }
                    Stmt::Opaque { .. } => collect_thirdparty_tokens(stmt.span().slice(text), out),
                    Stmt::Call(_) => {}
                }
            }
            Ok(())
        }
        Err(e) => Err(CollectError::BuckGraph(format!(
            "{} could not be parsed as BUCK: {e}",
            path.display()
        ))),
    }
}

fn collect_thirdparty_tokens_from_expr(text: &str, node: &ExprNode, out: &mut BTreeSet<String>) {
    if node.has_opaque() {
        collect_thirdparty_tokens(node.span.slice(text), out);
    }
    for s in expr_strings(node) {
        collect_thirdparty_tokens(&s, out);
    }
}

fn collect_thirdparty_tokens(text: &str, out: &mut BTreeSet<String>) {
    let marker = "third-party//:";
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(marker) {
        let start = from + rel + marker.len();
        let name: String = text[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
        from = start;
    }
}

fn generated_buck_blocks(text: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = None;
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if start.is_none() && trimmed.ends_with('(') && !trimmed.starts_with('#') {
            start = Some(offset);
        }
        if trimmed == ")"
            && let Some(byte_start) = start.take()
        {
            blocks.push(&text[byte_start..offset + line.len()]);
        }
        offset += line.len();
    }
    blocks
}

fn generated_field(block: &str, field: &str) -> Option<String> {
    let prefix = format!("{field} = ");
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(&prefix) {
            continue;
        }
        return quoted_strings(trimmed).into_iter().next();
    }
    None
}

fn crate_names_implied_by_target(target: &str, block: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    if let Some(crate_name) = generated_field(block, "crate") {
        insert_crate_name_spellings(&mut names, &crate_name);
    }
    for line in block.lines() {
        if line.contains("\"CARGO_PKG_NAME\"")
            && let Some(crate_name) = quoted_strings(line).into_iter().last()
        {
            insert_crate_name_spellings(&mut names, &crate_name);
        }
    }
    if let Some((name, _version)) = target.rsplit_once('-')
        && target_belongs_to_crate(target, name)
    {
        names.insert(name.to_owned());
    }
    names
}

fn insert_crate_name_spellings(names: &mut BTreeSet<String>, crate_name: &str) {
    names.insert(crate_name.to_owned());
    names.insert(crate_name.replace('_', "-"));
}

fn generated_local_deps(block: &str) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    for quoted in quoted_strings(block) {
        collect_colon_labels(&quoted, &mut deps);
    }
    deps
}

fn collect_colon_labels(text: &str, out: &mut BTreeSet<String>) {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b':' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < bytes.len() {
            let ch = bytes[end] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                end += 1;
            } else {
                break;
            }
        }
        if end > start {
            out.insert(text[start..end].to_owned());
        }
        index = end.max(index + 1);
    }
}

fn quoted_strings(text: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((_start, ch)) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut escaped = false;
        let mut value = String::new();
        let mut closed = false;
        for (_index, next) in chars.by_ref() {
            if escaped {
                value.push(next);
                escaped = false;
            } else if next == '\\' {
                escaped = true;
            } else if next == '"' {
                closed = true;
                break;
            } else {
                value.push(next);
            }
        }
        if closed {
            strings.push(value);
        } else {
            break;
        }
    }
    strings
}

fn target_belongs_to_crate(target: &str, crate_name: &str) -> bool {
    let Some(rest) = target.strip_prefix(crate_name) else {
        return false;
    };
    if rest.is_empty() {
        return true;
    }
    rest.strip_prefix('-')
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|ch| ch.is_ascii_digit())
}

fn visit_files<F>(root: &Path, visit: &mut F) -> Result<(), CollectError>
where
    F: FnMut(&Path) -> Result<(), CollectError>,
{
    let mut queue = VecDeque::from([root.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        for entry in fs::read_dir(&dir).map_err(|e| CollectError::Io {
            path: dir.display().to_string(),
            message: e.to_string(),
        })? {
            let entry = entry.map_err(|e| CollectError::Io {
                path: dir.display().to_string(),
                message: e.to_string(),
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| CollectError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            })?;
            if file_type.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                queue.push_back(path);
            } else if file_type.is_file() {
                visit(&path)?;
            }
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "buck-out" | "target" | "third-party")
    )
}

fn read_file(path: &Path) -> Result<String, CollectError> {
    fs::read_to_string(path).map_err(|e| CollectError::Io {
        path: path.display().to_string(),
        message: e.to_string(),
    })
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

/// The mandated-replacement crate for a forbidden backend, for remediation text. Empty if none.
fn mandated_replacement_for(policy: &Value, crate_name: &str) -> String {
    policy
        .get("forbidden")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("crate").and_then(Value::as_str) == Some(crate_name))
        .and_then(|entry| entry.get("mandated_replacement").and_then(Value::as_str))
        .unwrap_or("")
        .to_owned()
}

/// Pure evaluator. `policy` is DATA (`crypto-backend-purity-policy.json`); `observed` is the
/// collected activation graph shaped by [`collect_activated_backends`].
///
/// CRITICAL (ADR-0506): the evaluator reads `observed.backends[].activated_dependents`, which the
/// collector derives from local first-party BUCK roots plus generated `third-party/BUCK` edges. It
/// does NOT (and must not) read Cargo.lock text nor cargo-metadata resolve-node dependency lists:
/// those retain the documented unactivated optional-dep `ring` phantom and would false-RED on a
/// harmless stanza that is never compiled. A reachable forbidden backend is a FAIL; the
/// lock-superset phantom (which never appears in the Buck graph) is OK.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "CBP-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a missing/malformed forbidden list rather than silently passing with nothing
    // to check.
    let forbidden = forbidden_crates(policy);
    if policy.get("forbidden").and_then(Value::as_array).is_none() {
        findings.insert(Finding::new(
            "CBP-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden` must be a non-null array of {crate, ...} entries; the policy must be corrected before the gate can evaluate",
        ));
        return findings;
    }
    if forbidden.is_empty() {
        findings.insert(Finding::new(
            "CBP-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden` resolved to zero crate names (every entry missing a string `crate` key); the gate would have nothing to enforce — correct the policy",
        ));
        return findings;
    }

    let min_expected = policy
        .get("min_expected_workspace_packages")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let census = observed
        .get("workspace_packages_found")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if census < min_expected {
        findings.insert(Finding::new(
            "CBP-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "workspace package census {census} is below the policy floor of {min_expected}; the repo root is likely wrong (fail-closed against a silent false-green where the Buck graph saw an empty workspace)"
            ),
        ));
    }

    // Index the observed activation view by forbidden crate.
    let backends = observed
        .get("backends")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for crate_name in &forbidden {
        let Some(entry) = backends
            .iter()
            .find(|b| b.get("crate").and_then(Value::as_str) == Some(crate_name.as_str()))
        else {
            // The collector did not produce an observation for a forbidden crate — fail closed:
            // the activation view for that backend is UNKNOWN, not empty.
            findings.insert(Finding::new(
                "CBP-FORBIDDEN-BACKEND-ACTIVATED",
                crate_name,
                format!(
                    "no activation observation was produced for forbidden backend `{crate_name}`; the Buck activation view is unknown — failing closed (re-run the collector / inspect first-party BUCK roots and generated third-party/BUCK)"
                ),
            ));
            continue;
        };
        let dependents = entry
            .get("activated_dependents")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if dependents.is_empty() {
            // The OK case: zero ACTIVATED dependents in the Buck graph. Cargo.lock may still list
            // this crate as an unactivated optional-dep phantom — that is harmless (never compiled)
            // and is deliberately NOT a finding (ADR-0506).
            continue;
        }
        let replacement = mandated_replacement_for(policy, crate_name);
        let replacement_note = if replacement.is_empty() {
            String::new()
        } else {
            format!(" Mandated replacement: `{replacement}` (ADR-0506).")
        };
        let activators = dependents.join("; ");
        findings.insert(Finding::new(
            "CBP-FORBIDDEN-BACKEND-ACTIVATED",
            crate_name,
            format!(
                "forbidden crypto backend `{crate_name}` is ACTIVATED in the local Buck graph: {activators}. ADR-0506 forbids `{crate_name}` — find the first-party BUCK target or generated third-party edge that activates it and switch it to the mandated backend.{replacement_note}"
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
        return "crypto-backend-purity gate passed: no forbidden crypto backend is ACTIVATED (zero-ring invariant holds; any Cargo.lock optional-dep phantom is unactivated and harmless — ADR-0506)".to_owned();
    }
    let mut out = String::from("crypto-backend-purity gate failed (ADR-0506):\n");
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
    use std::path::{Path, PathBuf};

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "min_expected_workspace_packages": 1,
            "forbidden": [
                {"crate": "ring", "mandated_replacement": "aws-lc-rs"}
            ],
            "mandated": [{"crate": "aws-lc-rs"}]
        })
    }

    struct TempRepo {
        root: PathBuf,
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let root =
                std::env::temp_dir().join(format!("oya-cbp-{name}-{}-{nanos}", std::process::id()));
            std::fs::create_dir_all(&root).expect("create temp repo");
            Self { root }
        }

        fn write(&self, relative: &str, text: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create temp parent");
            }
            std::fs::write(&path, text).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        }

        fn root(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn write_fixture_repo(repo: &TempRepo, first_party_buck: &str, third_party_buck: &str) {
        repo.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        );
        repo.write("app/BUCK", first_party_buck);
        repo.write("third-party/BUCK", third_party_buck);
    }

    fn observed(census: u64, ring_dependents: &[&str]) -> Value {
        json!({
            "workspace_packages_found": census,
            "backends": [
                {
                    "crate": "ring",
                    "activated_dependents": ring_dependents,
                }
            ]
        })
    }

    #[test]
    fn green_when_no_forbidden_backend_is_activated() {
        // The current-tree state: the Buck graph has no reachable forbidden backend. The gate
        // PASSES even when Cargo.lock retains an unactivated optional-dep phantom that never
        // reaches the Buck graph.
        let report = evaluate(&policy(), &observed(100, &[]));
        assert_eq!(report.verdict, Verdict::Green, "no activated ring ⇒ green");
        assert!(report.violations.is_empty());
    }

    #[test]
    fn red_when_a_crate_activates_ring() {
        // RED fixture: a crate ACTIVATES ring (e.g. a rustls/sqlx feature flipped back). The
        // observed Buck view shows real activated dependents ⇒ FAIL.
        let activated = &[
            "ring v0.17.14",
            "rustls v0.23.40",
            "some-workspace-crate v0.1.0 (/repo/crates/some-workspace-crate)",
        ];
        let findings = evaluate_keyed(&policy(), &observed(100, activated));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "CBP-FORBIDDEN-BACKEND-ACTIVATED" && f.key == "ring"),
            "an activated ring must produce CBP-FORBIDDEN-BACKEND-ACTIVATED: {findings:?}"
        );
        let f = findings.iter().find(|f| f.key == "ring").unwrap();
        assert!(
            f.detail.contains("aws-lc-rs"),
            "remediation must name the mandated replacement: {f:?}"
        );
        assert!(
            f.detail.contains("some-workspace-crate"),
            "remediation must surface the activator: {f:?}"
        );
        assert_eq!(
            evaluate(&policy(), &observed(100, activated)).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn the_lock_superset_phantom_is_not_a_finding() {
        // The crux assertion: the gate must distinguish ACTIVATED ring (FAIL) from the documented
        // Cargo.lock / cargo-metadata SUPERSET phantom (OK). The collector derives
        // activated_dependents from the Buck graph, so an uncompiled lock-only phantom never
        // appears in activated_dependents. An empty activated_dependents view is GREEN by
        // construction.
        let report = evaluate(&policy(), &observed(200, &[]));
        assert_eq!(report.verdict, Verdict::Green);
        let rendered = render_findings(&evaluate_keyed(&policy(), &observed(200, &[])));
        assert!(
            rendered.contains("passed"),
            "phantom-only tree must read as passed: {rendered}"
        );
        assert!(
            rendered.contains("unactivated"),
            "the rendered pass must name the phantom distinction: {rendered}"
        );
    }

    #[test]
    fn empty_scan_fails_closed() {
        // A broken probe (census below floor, e.g. the repo root is wrong and saw nothing)
        // must fail closed rather than pass as a false-green.
        let findings = evaluate_keyed(&policy(), &observed(0, &[]));
        assert!(
            findings.iter().any(|f| f.code == "CBP-EMPTY-SCAN"),
            "a below-floor census must trip CBP-EMPTY-SCAN: {findings:?}"
        );
    }

    #[test]
    fn missing_observation_for_a_forbidden_crate_fails_closed() {
        // If the collector produced no observation for a forbidden crate, the activation view is
        // UNKNOWN, not empty — fail closed.
        let obs = json!({ "workspace_packages_found": 100, "backends": [] });
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "CBP-FORBIDDEN-BACKEND-ACTIVATED" && f.key == "ring"),
            "a missing observation for `ring` must fail closed: {findings:?}"
        );
    }

    #[test]
    fn policy_gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = Value::from("wrong-id");
        let findings = evaluate_keyed(&p, &observed(100, &[]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "CBP-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn malformed_policy_with_no_forbidden_list_fails_closed() {
        let p = json!({ "gate_id": GATE_ID, "min_expected_workspace_packages": 1 });
        let findings = evaluate_keyed(&p, &observed(100, &[]));
        assert!(
            findings.iter().any(|f| f.code == "CBP-POLICY-MALFORMED"),
            "a missing `forbidden` list must fail closed: {findings:?}"
        );
    }

    #[test]
    fn buck_graph_collector_detects_forbidden_backend_reachable_from_first_party_buck() {
        let repo = TempRepo::new("reachable-ring");
        write_fixture_repo(
            &repo,
            r#"rust_library(
    name = "app",
    deps = ["third-party//:tls-stack"],
)
"#,
            r#"alias(
    name = "tls-stack",
    actual = ":tls-stack-1",
    visibility = ["PUBLIC"],
)

cargo.rust_library(
    name = "tls-stack-1",
    crate = "tls_stack",
    env = {
        "CARGO_PKG_NAME": "tls-stack",
    },
    deps = [
        ":ring-0.17",
    ],
)

cargo.rust_library(
    name = "ring-0.17",
    crate = "ring",
    env = {
        "CARGO_PKG_NAME": "ring",
    },
    visibility = [],
)
"#,
        );

        let observed = collect_activated_backends(repo.root(), &policy()).expect("collect graph");
        let findings = evaluate_keyed(&policy(), &observed);
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
        let detail = findings
            .iter()
            .find(|finding| finding.key == "ring")
            .expect("ring finding")
            .detail
            .clone();
        assert!(
            detail.contains("third-party//:ring-0.17")
                && detail.contains("third-party//:tls-stack"),
            "finding should show the generated forbidden target and first-party root: {detail}"
        );
    }

    #[test]
    fn buck_graph_collector_detects_variable_backed_first_party_deps() {
        let repo = TempRepo::new("variable-backed-ring");
        write_fixture_repo(
            &repo,
            r#"_DEPS = [
    "third-party//:tls-stack",
]

rust_library(
    name = "app",
    deps = _DEPS,
)
"#,
            r#"alias(
    name = "tls-stack",
    actual = ":tls-stack-1",
    visibility = ["PUBLIC"],
)

cargo.rust_library(
    name = "tls-stack-1",
    crate = "tls_stack",
    env = {
        "CARGO_PKG_NAME": "tls-stack",
    },
    deps = [
        ":ring-0.17",
    ],
)

cargo.rust_library(
    name = "ring-0.17",
    crate = "ring",
    env = {
        "CARGO_PKG_NAME": "ring",
    },
    visibility = [],
)
"#,
        );

        let observed = collect_activated_backends(repo.root(), &policy()).expect("collect graph");
        assert_eq!(
            evaluate(&policy(), &observed).verdict,
            Verdict::Red,
            "variable-backed BUCK deps must seed first-party third-party roots"
        );
    }

    #[test]
    fn buck_graph_collector_ignores_unreachable_generated_forbidden_target() {
        let repo = TempRepo::new("unreachable-ring");
        write_fixture_repo(
            &repo,
            r#"rust_library(
    name = "app",
    deps = ["third-party//:safe-stack"],
)
"#,
            r#"alias(
    name = "safe-stack",
    actual = ":safe-stack-1",
    visibility = ["PUBLIC"],
)

cargo.rust_library(
    name = "safe-stack-1",
    crate = "safe_stack",
    env = {
        "CARGO_PKG_NAME": "safe-stack",
    },
    visibility = [],
)

cargo.rust_library(
    name = "ring-0.17",
    crate = "ring",
    env = {
        "CARGO_PKG_NAME": "ring",
    },
    visibility = [],
)
"#,
        );

        let observed = collect_activated_backends(repo.root(), &policy()).expect("collect graph");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Green);
        assert!(
            evaluate_keyed(&policy(), &observed).is_empty(),
            "unreachable generated ring target must not fail the gate: {observed}"
        );
    }

    #[test]
    fn buck_graph_collector_fails_closed_when_first_party_roots_are_empty() {
        let repo = TempRepo::new("empty-roots");
        write_fixture_repo(
            &repo,
            r#"rust_library(
    name = "app",
    deps = [],
)
"#,
            r#"cargo.rust_library(
    name = "ring-0.17",
    crate = "ring",
    env = {
        "CARGO_PKG_NAME": "ring",
    },
    visibility = [],
)
"#,
        );

        let err = collect_activated_backends(repo.root(), &policy())
            .expect_err("empty roots fail closed");
        assert!(
            err.to_string().contains("no first-party BUCK references"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn buck_graph_collector_fails_closed_on_unparseable_first_party_buck() {
        let repo = TempRepo::new("bad-first-party-buck");
        write_fixture_repo(
            &repo,
            r#"rust_library(
    name = "app",
    deps = ["third-party//:tls-stack"],
"#,
            r#"cargo.rust_library(
    name = "tls-stack",
    crate = "tls_stack",
    env = {
        "CARGO_PKG_NAME": "tls-stack",
    },
    visibility = [],
)
"#,
        );

        let err = collect_activated_backends(repo.root(), &policy())
            .expect_err("unparseable first-party BUCK must fail closed");
        assert!(
            err.to_string().contains("app/BUCK could not be parsed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn buck_graph_collector_fails_closed_on_unparseable_third_party_buck() {
        let repo = TempRepo::new("bad-third-party-buck");
        write_fixture_repo(
            &repo,
            r#"rust_library(
    name = "app",
    deps = ["third-party//:tls-stack"],
)
"#,
            r#"BROKEN = [":ring-0.17",,]

cargo.rust_library(
    name = "tls-stack",
    crate = "tls_stack",
    deps = [":ring-0.17"],
    env = {
        "CARGO_PKG_NAME": "tls-stack",
    },
    visibility = [],
)
"#,
        );

        let err = collect_activated_backends(repo.root(), &policy())
            .expect_err("unparseable third-party BUCK must fail closed");
        assert!(
            err.to_string()
                .contains("third-party/BUCK could not be parsed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn crate_target_matching_does_not_confuse_ring_feature_names() {
        assert!(target_belongs_to_crate("ring", "ring"));
        assert!(target_belongs_to_crate("ring-0.17", "ring"));
        assert!(!target_belongs_to_crate("ring-io-0.1", "ring"));
        assert!(!target_belongs_to_crate("ring-sig-verify", "ring"));
    }

    #[test]
    fn forbidden_crates_are_sorted_and_deduped() {
        let p = json!({
            "gate_id": GATE_ID,
            "forbidden": [{"crate": "ring"}, {"crate": "openssl-sys"}, {"crate": "ring"}]
        });
        assert_eq!(
            forbidden_crates(&p),
            vec!["openssl-sys".to_owned(), "ring".to_owned()]
        );
    }
}
