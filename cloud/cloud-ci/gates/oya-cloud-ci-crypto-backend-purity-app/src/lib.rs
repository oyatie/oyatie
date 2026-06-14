//! # cloud-ci-crypto-backend-purity (ADR-0506)
//!
//! The crypto-backend-purity gate. ADR-0506 mandates `aws-lc-rs` as the SINGLE canonical Phase-1
//! crypto backend and FORBIDS `ring`. The ADR DESCRIBES the zero-ring invariant — "`cargo tree -i
//! ring --target all` now prints nothing" and "`buck2 cquery \"deps(//...)\" | grep ring-0.17`
//! returns 0" — but NOTHING mechanically enforced it. This gate is that enforcement (founder
//! doctrine: flag-only/manual = incomplete; construction > reaction; automate everything
//! automatable).
//!
//! ## The signal is feature-resolved ACTIVATION, not the dependency SUPERSET
//! This is the load-bearing distinction the gate exists to draw correctly.
//!
//! `Cargo.lock` retains a `ring 0.17` stanza as an UNACTIVATED optional-dependency phantom:
//! `reqwest`'s optional, never-enabled `http3` feature pulls `quinn -> quinn-proto -> ring`, and
//! `rustls-webpki` carries an OFF `ring` feature. **Cargo stores resolved versions including
//! optional deps, NOT feature activation** (ADR-0506). The SAME phantom is present in `cargo
//! metadata`'s `resolve.nodes[].dependencies` AND `resolve.nodes[].deps[]` — neither prunes an
//! optional edge whose feature is off, because `cargo metadata` reports the resolved superset, not
//! the activated selection. Asserting on the lock text or on cargo-metadata resolve nodes would
//! therefore FALSE-RED on a harmless phantom that is in no build graph and is never compiled.
//!
//! The correct, portable signal — the one ADR-0506 itself cites — is `cargo tree -i <crate>
//! --target all`. The `-i` (inverse) view is FEATURE-RESOLVED: it prunes an optional dependency
//! edge whose activating feature is off, so it prints "nothing to print" exactly when the backend
//! is never activated for any target/feature, and prints the real activated dependents when it is.
//! [`collect_activated_backends`] runs that command per forbidden crate; [`evaluate_keyed`] fails
//! iff any forbidden backend has ≥1 ACTIVATED dependent. The gate thus distinguishes an ACTIVATED
//! ring (FAIL) from the documented lock-superset phantom (OK).
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the forbidden-backend set, the mandated
//! backend, the package-census floor — are DATA in `crypto-backend-purity-policy.json`. Nothing
//! oyatie-specific is hardcoded in Rust; a different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_activated_backends`] `(root, policy) -> observed` is the ONLY I/O: it shells
//!   `cargo tree -i <crate> --target all` for each forbidden crate and parses the activated
//!   dependents. Read-only; writes no temp files.
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
//!   the feature-resolved `cargo tree -i <crate> --target all` graph.
//! - `CBP-EMPTY-SCAN`                  — the workspace package census is below the policy floor
//!   (catches a broken CWD / cargo invocation that would otherwise be a false-green).
//! - `CBP-POLICY-GATE-ID-MISMATCH`     — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `CBP-POLICY-MALFORMED`            — the policy `forbidden` list is missing/malformed.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

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

/// The sentinel `cargo tree -i` emits (on stdout AND/OR stderr) when the queried crate has NO
/// activated dependent in the feature-resolved graph. This is the OK signal — the backend is never
/// compiled for any target/feature. We match on this substring rather than relying on an empty
/// stdout, because cargo prints the message plus a hint, and routes it to stderr.
pub const NOTHING_TO_PRINT: &str = "nothing to print";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only subprocess)
// ---------------------------------------------------------------------------

/// Errors collecting the observed activation graph. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — a failed `cargo tree` is a fail-closed error,
/// never a silently skipped backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// `cargo` could not be spawned at all (no cargo on PATH).
    Spawn(String),
    /// `cargo tree` ran but exited non-zero for a reason OTHER than "nothing to print"
    /// (e.g. a manifest error). Fail closed: the activation view is unknown, not empty.
    CargoTree { crate_name: String, message: String },
    /// `cargo metadata` (the package-census probe) could not be run or parsed.
    Census(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Spawn(message) => write!(f, "crypto-backend-purity spawn cargo: {message}"),
            CollectError::CargoTree { crate_name, message } => write!(
                f,
                "crypto-backend-purity `cargo tree -i {crate_name} --target all` failed (fail-closed): {message}"
            ),
            CollectError::Census(message) => {
                write!(f, "crypto-backend-purity package census: {message}")
            }
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
/// For each forbidden crate, runs `cargo tree -i <crate> --target all` at `root` and records the
/// ACTIVATED dependent lines (empty iff cargo printed the "nothing to print" sentinel — the backend
/// is never compiled). Also probes the workspace package census via `cargo metadata --no-deps` so a
/// broken CWD / cargo invocation fails closed via `CBP-EMPTY-SCAN` rather than passing as a
/// false-green. Emits:
/// `{ "workspace_packages_found": <usize>, "backends": [ { "crate": <name>,
///    "activated_dependents": [ <line>, .. ] } ] }`.
pub fn collect_activated_backends(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let census = workspace_package_count(root)?;

    let mut backends = Vec::new();
    for crate_name in forbidden_crates(policy) {
        let activated = activated_dependents_of(root, &crate_name)?;
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

/// Run `cargo tree -i <crate_name> --target all` at `root` and return the ACTIVATED dependent
/// lines. Returns an empty Vec iff cargo reported the "nothing to print" sentinel (the backend has
/// no activated dependent — the OK signal). Any OTHER non-zero exit fails closed: we must never
/// treat an unknown activation view as empty.
fn activated_dependents_of(root: &Path, crate_name: &str) -> Result<Vec<String>, CollectError> {
    let output = Command::new("cargo")
        .args(["tree", "--invert", crate_name, "--target", "all"])
        .current_dir(root)
        .output()
        .map_err(|e| CollectError::Spawn(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The "nothing to print" sentinel is the OK signal — the queried crate has no activated
    // dependent for any target/feature. cargo routes it to stderr (with a hint) rather than
    // stdout, so probe BOTH streams. `cargo tree` exits 0 in this case, so this branch is taken
    // before the success check below.
    if stdout.contains(NOTHING_TO_PRINT) || stderr.contains(NOTHING_TO_PRINT) {
        return Ok(Vec::new());
    }

    if !output.status.success() {
        // A genuine failure (manifest error, lock out of date, unknown crate handling, …). The
        // activation view is UNKNOWN, not empty: fail closed so a broken probe cannot false-green.
        let message = stderr.trim();
        return Err(CollectError::CargoTree {
            crate_name: crate_name.to_owned(),
            message: if message.is_empty() {
                format!("exit status {:?} with empty stderr", output.status.code())
            } else {
                message.to_owned()
            },
        });
    }

    Ok(parse_cargo_tree_dependents(&stdout))
}

/// Parse the activated dependent lines from `cargo tree -i` output. Each non-blank line names a
/// crate that ACTIVATES (directly or transitively depends, with the feature on) the queried
/// backend; the first line is the queried crate itself. We keep ALL crate lines as the evidence
/// set (their presence at all means the backend is activated).
///
/// The "nothing to print" sentinel SHORT-CIRCUITS to an empty set: if it appears anywhere in the
/// output, the queried crate has no activated dependent (the OK signal), regardless of any
/// accompanying `warning:`/`hint:` advisory lines cargo prints alongside it. Advisory lines
/// (`warning:`, `hint:`, `note:`) are filtered so they are never mistaken for crate lines. Pure
/// helper, exposed for tests so the fixture-driven RED/GREEN cases need no live cargo.
pub fn parse_cargo_tree_dependents(stdout: &str) -> Vec<String> {
    if stdout.contains(NOTHING_TO_PRINT) {
        return Vec::new();
    }
    stdout
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("warning:")
                || trimmed.starts_with("hint:")
                || trimmed.starts_with("note:"))
        })
        .map(str::to_owned)
        .collect()
}

/// Probe the workspace package census via `cargo metadata --no-deps` (the sole sanctioned cargo
/// metadata invocation — the gate's own fail-closed floor). A too-small census trips
/// `CBP-EMPTY-SCAN`, catching a broken CWD / cargo invocation that would otherwise pass as a
/// false-green.
fn workspace_package_count(root: &Path) -> Result<u64, CollectError> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
        .map_err(|e| CollectError::Spawn(e.to_string()))?;
    if !output.status.success() {
        return Err(CollectError::Census(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CollectError::Census(format!("parse cargo metadata: {e}")))?;
    let count = metadata
        .get("packages")
        .and_then(Value::as_array)
        .map(|packages| packages.len() as u64)
        .unwrap_or(0);
    Ok(count)
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
/// collector derives from `cargo tree -i <crate> --target all` — the FEATURE-RESOLVED activation
/// view. It does NOT (and must not) read Cargo.lock text nor cargo-metadata resolve-node
/// dependency lists: those retain the documented unactivated optional-dep `ring` phantom (reqwest's
/// off `http3`/quinn chain; rustls-webpki's off `ring` feature) and would false-RED on a harmless
/// stanza that is never compiled. An ACTIVATED forbidden backend (non-empty dependents) is a FAIL;
/// the lock-superset phantom (which never appears here because the feature is off) is OK.
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
                "workspace package census {census} is below the policy floor of {min_expected}; the CWD or the cargo invocation is likely broken (fail-closed against a silent false-green where `cargo tree` saw an empty graph)"
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
                    "no activation observation was produced for forbidden backend `{crate_name}`; the activation view is unknown — failing closed (re-run the collector / check `cargo tree -i {crate_name} --target all`)"
                ),
            ));
            continue;
        };
        let dependents = entry
            .get("activated_dependents")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_owned).collect::<Vec<_>>())
            .unwrap_or_default();
        if dependents.is_empty() {
            // The OK case: zero ACTIVATED dependents. (`cargo tree -i` printed "nothing to print".)
            // The Cargo.lock / cargo-metadata SUPERSET may still list this crate as an unactivated
            // optional-dep phantom — that is harmless (never compiled) and is deliberately NOT a
            // finding (ADR-0506).
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
                "forbidden crypto backend `{crate_name}` is ACTIVATED in the feature-resolved graph (`cargo tree -i {crate_name} --target all` shows activated dependents): {activators}. ADR-0506 forbids `{crate_name}` — find the feature that activates it (a rustls/sqlx backend flag flipped to ring, or reqwest's http3/quinn enabled) and switch it to the mandated backend.{replacement_note}"
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
        out.push_str(&format!("    - {} {}\n        {}\n", finding.code, finding.key, finding.detail));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // The current-tree state: `cargo tree -i ring --target all` printed "nothing to print",
        // so activated_dependents is empty. The gate PASSES even though Cargo.lock retains the
        // unactivated optional-dep ring phantom (which never reaches this view).
        let report = evaluate(&policy(), &observed(100, &[]));
        assert_eq!(report.verdict, Verdict::Green, "no activated ring ⇒ green");
        assert!(report.violations.is_empty());
    }

    #[test]
    fn red_when_a_crate_activates_ring() {
        // RED fixture: a crate ACTIVATES ring (e.g. a rustls/sqlx feature flipped back). The
        // `cargo tree -i ring --target all` view shows real activated dependents ⇒ FAIL.
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
        assert!(f.detail.contains("aws-lc-rs"), "remediation must name the mandated replacement: {f:?}");
        assert!(f.detail.contains("some-workspace-crate"), "remediation must surface the activator: {f:?}");
        assert_eq!(evaluate(&policy(), &observed(100, activated)).verdict, Verdict::Red);
    }

    #[test]
    fn the_lock_superset_phantom_is_not_a_finding() {
        // The crux assertion: the gate must distinguish ACTIVATED ring (FAIL) from the documented
        // Cargo.lock / cargo-metadata SUPERSET phantom (OK). The collector derives
        // activated_dependents from `cargo tree -i` (the feature-resolved view), which prunes the
        // unactivated optional edge — so the phantom NEVER appears in activated_dependents. An
        // empty activated_dependents (the live-tree state) is GREEN by construction.
        let report = evaluate(&policy(), &observed(200, &[]));
        assert_eq!(report.verdict, Verdict::Green);
        let rendered = render_findings(&evaluate_keyed(&policy(), &observed(200, &[])));
        assert!(rendered.contains("passed"), "phantom-only tree must read as passed: {rendered}");
        assert!(rendered.contains("unactivated"), "the rendered pass must name the phantom distinction: {rendered}");
    }

    #[test]
    fn empty_scan_fails_closed() {
        // A broken probe (census below floor, e.g. cargo ran in the wrong CWD and saw nothing)
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
        assert!(findings.iter().any(|f| f.code == "CBP-POLICY-GATE-ID-MISMATCH"));
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
    fn parse_cargo_tree_dependents_drops_blanks_and_sentinel() {
        // "nothing to print" parses to ZERO dependents (the OK signal).
        assert!(parse_cargo_tree_dependents("warning: nothing to print.\n\nhint: ...\n").is_empty());
        // Real output parses to the non-blank crate lines.
        let out = "ring v0.17.14\n├── rustls v0.23.40\n│   └── my-crate v0.1.0\n";
        let parsed = parse_cargo_tree_dependents(out);
        assert_eq!(parsed.len(), 3, "three non-blank crate lines: {parsed:?}");
        assert!(parsed[0].contains("ring"));
    }

    #[test]
    fn forbidden_crates_are_sorted_and_deduped() {
        let p = json!({
            "gate_id": GATE_ID,
            "forbidden": [{"crate": "ring"}, {"crate": "openssl-sys"}, {"crate": "ring"}]
        });
        assert_eq!(forbidden_crates(&p), vec!["openssl-sys".to_owned(), "ring".to_owned()]);
    }
}
