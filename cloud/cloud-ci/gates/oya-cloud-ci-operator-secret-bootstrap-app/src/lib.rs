//! # cloud-ci-operator-secret-bootstrap (GH #980)
//!
//! Least-privilege secret RBAC + a declarative join-token bootstrap contract for in-cluster
//! operators that PRODUCE a single Secret. The SVID-delivery operator (PR #793) shipped a Helm Role
//! granting `get/list/watch/create/update/patch` on ALL `secrets` in its namespace — broader than the
//! stated non-claim (it writes only `oya-cloud-iam-pdp-svid`) — and consumed an
//! `OYA_SVID_OPERATOR_JOIN_TOKEN` from a Secret the chart only *assumed* was pre-provisioned, with no
//! provisioning template, runbook, or preflight. This gate is the enforcement backstop for the fix.
//!
//! ## Two invariants, per governed operator (DATA in `operator-secret-bootstrap-policy.json`)
//! 1. **Name-scoped secret writes.** Kubernetes RBAC unconditionally honors `resourceNames` for the
//!    single-object verbs `get/update/patch/delete` (the `scoped_secret_verbs`). `create` (and
//!    `deletecollection`) can never be `resourceNames`-scoped (the object name is unknown at
//!    authorization time); `list/watch` can be, but only when the client sends a matching
//!    `metadata.name` field selector. So any Role rule that grants a scoped verb on `secrets` MUST
//!    carry `resourceNames` bound to exactly the operator's `produced_secret_name`. `list/watch/create`
//!    may stay namespace-wide — that is the documented RBAC floor (narrowable once the operator binary
//!    guarantees a field selector), justified in the chart, not a gate concern.
//! 2. **Bootstrap-Secret provisioning.** The operator-internal bootstrap Secret (the join token,
//!    named by `join_token_values_ref`) must be either provisioned declaratively (an `ExternalSecret`
//!    / `SealedSecret` / `Secret` template referencing that values path) OR guarded by a fail-closed
//!    Helm preflight (`fail` referencing the join-token values group). An operator enabled with an
//!    unprovisioned, unvalidated bootstrap Secret fails closed.
//!
//! ## Kernel contract
//! - [`collect_operators`] `(root, policy) -> observed` is the ONLY I/O: read-only `fs` reads of the
//!   governed RBAC template + a shallow scan of the chart templates dir. No shell, no network, no VCS.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `OSB-SECRET-RBAC-OVERBROAD`             — a secrets Role rule grants a scoped verb without any
//!   `resourceNames` (namespace-wide get/update/patch/delete).
//! - `OSB-SECRET-RBAC-RESOURCENAME-MISMATCH` — a scoped secrets rule carries `resourceNames`, but the
//!   set is not exactly `{produced_secret_name}` (scopes the wrong / extra Secret).
//! - `OSB-JOIN-TOKEN-UNPROVISIONED`          — the bootstrap Secret has neither a declarative
//!   provisioning template nor a fail-closed preflight in the chart.
//! - `OSB-POLICY-GATE-ID-MISMATCH`           — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `OSB-POLICY-MALFORMED`                  — the policy `operators` list is missing/malformed
//!   (fail-closed: the gate would have nothing to enforce).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-operator-secret-bootstrap";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 5] = [
    "OSB-SECRET-RBAC-OVERBROAD",
    "OSB-SECRET-RBAC-RESOURCENAME-MISMATCH",
    "OSB-JOIN-TOKEN-UNPROVISIONED",
    "OSB-POLICY-GATE-ID-MISMATCH",
    "OSB-POLICY-MALFORMED",
];

/// The sentinel key for policy-level (non-per-operator) findings.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only, hermetic — no shell / network / VCS)
// ---------------------------------------------------------------------------

/// Errors collecting the observed view. Returned instead of panicking so the caller decides how to
/// surface them — an unreadable chart or a malformed RBAC template is a fail-closed error, never a
/// silently skipped operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// A read-only filesystem operation failed (a governed chart could not be scanned).
    Io(String),
    /// An RBAC template did not parse as YAML after Helm-action neutralization (fail-closed: an
    /// unparseable template could hide an overbroad rule).
    Parse(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "operator-secret-bootstrap io: {message}"),
            CollectError::Parse(message) => write!(f, "operator-secret-bootstrap parse: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// One secrets RBAC rule reduced to the fields this gate audits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRule {
    /// The rule's `verbs`.
    pub verbs: Vec<String>,
    /// The rule's `resourceNames` (empty when the rule is namespace-wide).
    pub resource_names: Vec<String>,
}

/// The governed-operator descriptors declared in policy DATA.
fn operators(policy: &Value) -> Option<Vec<Value>> {
    policy
        .get("operators")
        .and_then(Value::as_array)
        .map(|list| list.to_vec())
}

fn str_field<'a>(operator: &'a Value, key: &str) -> Option<&'a str> {
    operator.get(key).and_then(Value::as_str)
}

/// Collect the observed view the policy asks about. Read-only — NO shell, NO network, NO VCS.
///
/// For each governed operator:
/// 1. Parse the RBAC template's `secrets` Role rules (verbs + resourceNames) — the only structural
///    YAML parse, over the static rules block after neutralizing the Helm actions.
/// 2. Shallow-scan the chart templates dir for join-token provisioning evidence: a provisioning-kind
///    doc (`ExternalSecret`/`SealedSecret`/`Secret`) referencing the join-token values group, and a
///    fail-closed Helm preflight (`fail` referencing that group).
///
/// Emits `{ "operators": [ { "name", "produced_secret_name", "secret_rules": [{verbs,resource_names}],
/// "has_provisioning_template": bool, "has_failclosed_preflight": bool } ] }`.
pub fn collect_operators(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let descriptors = operators(policy).unwrap_or_default();
    let mut out = Vec::new();
    for descriptor in &descriptors {
        let Some(name) = str_field(descriptor, "name") else {
            continue;
        };
        let rbac_rel = str_field(descriptor, "rbac_template").unwrap_or_default();
        let templates_rel = str_field(descriptor, "chart_templates_dir").unwrap_or_default();
        let produced = str_field(descriptor, "produced_secret_name").unwrap_or_default();
        let join_ref = str_field(descriptor, "join_token_values_ref").unwrap_or_default();

        let rbac_path = root.join(rbac_rel);
        let rbac_text = fs::read_to_string(&rbac_path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", rbac_path.display())))?;
        let rules = parse_secret_rules(&rbac_text)
            .map_err(|e| CollectError::Parse(format!("{}: {e}", rbac_path.display())))?;

        let (has_provisioning, has_preflight) =
            scan_join_token_evidence(&root.join(templates_rel), join_ref)?;

        out.push(json!({
            "name": name,
            "produced_secret_name": produced,
            "secret_rules": rules
                .iter()
                .map(|r| json!({ "verbs": r.verbs, "resource_names": r.resource_names }))
                .collect::<Vec<_>>(),
            "has_provisioning_template": has_provisioning,
            "has_failclosed_preflight": has_preflight,
        }));
    }
    Ok(json!({ "operators": out }))
}

/// The values-path "group" a provisioning template / preflight is expected to reference: the
/// join-token values ref minus its last `.segment` (`svidOperator.joinToken.secretName` ->
/// `svidOperator.joinToken`). Both the consumer (`...secretName`) and the preflight (`...joinToken`)
/// reference this group, so one token matches both.
fn join_token_group(join_ref: &str) -> &str {
    join_ref.rsplit_once('.').map(|(head, _)| head).unwrap_or(join_ref)
}

/// Shallow read-only scan of a chart templates dir for join-token provisioning evidence. Returns
/// `(has_provisioning_template, has_failclosed_preflight)`. A missing dir yields `(false, false)`
/// (fail-closed: no evidence). Only the top level is scanned (Helm templates are flat).
fn scan_join_token_evidence(dir: &Path, join_ref: &str) -> Result<(bool, bool), CollectError> {
    let group = join_token_group(join_ref);
    let mut has_provisioning = false;
    let mut has_preflight = false;
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((false, false)),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?
            .is_file();
        if !is_file {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(CollectError::Io(format!("read {}: {e}", path.display()))),
        };
        if scan_text_provisions_join_token(&text, group) {
            has_provisioning = true;
        }
        if scan_text_has_failclosed_preflight(&text, group) {
            has_preflight = true;
        }
    }
    Ok((has_provisioning, has_preflight))
}

/// Whether a template's text declares a provisioning-kind doc that references the join-token group.
/// Pure helper, exposed for tests.
pub fn scan_text_provisions_join_token(text: &str, group: &str) -> bool {
    if !text.contains(group) {
        return false;
    }
    text.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "kind: ExternalSecret"
            || trimmed == "kind: SealedSecret"
            || trimmed == "kind: Secret"
    })
}

/// Whether a template's text carries a fail-closed Helm preflight referencing the join-token group
/// (a single line containing both `fail` and the group token). Pure helper, exposed for tests.
pub fn scan_text_has_failclosed_preflight(text: &str, group: &str) -> bool {
    text.lines()
        .any(|line| line.contains("fail") && line.contains(group))
}

/// Parse the `secrets` Role rules out of a Helm-templated RBAC manifest. The rules block is static
/// YAML; the Helm actions live only in metadata/guards, so we neutralize them (drop standalone
/// `{{ ... }}` directive lines, replace inline `{{ ... }}` value expressions with a scalar
/// placeholder) and then parse each `---`-separated doc as YAML. FAIL-CLOSED: a non-empty doc that
/// does not parse is an `Err`, never a silently skipped (and potentially overbroad) rule. Pure
/// helper, exposed for tests.
pub fn parse_secret_rules(rbac_text: &str) -> Result<Vec<SecretRule>, String> {
    let neutralized = neutralize_helm(rbac_text);
    let mut rules = Vec::new();
    for chunk in neutralized.split("\n---") {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        let value: serde_yaml::Value =
            serde_yaml::from_str(chunk).map_err(|e| format!("yaml parse: {e}"))?;
        let kind = value.get("kind").and_then(serde_yaml::Value::as_str).unwrap_or("");
        if kind != "Role" && kind != "ClusterRole" {
            continue;
        }
        let Some(rule_seq) = value.get("rules").and_then(serde_yaml::Value::as_sequence) else {
            continue;
        };
        for rule in rule_seq {
            let resources = yaml_str_seq(rule.get("resources"));
            if !resources.iter().any(|r| r == "secrets") {
                continue;
            }
            rules.push(SecretRule {
                verbs: yaml_str_seq(rule.get("verbs")),
                resource_names: yaml_str_seq(rule.get("resourceNames")),
            });
        }
    }
    Ok(rules)
}

fn yaml_str_seq(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(serde_yaml::Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Neutralize Helm template syntax so the static YAML structure parses: drop standalone directive
/// lines (a trimmed line that both starts with `{{` and ends with `}}`, e.g. `{{- if .. }}`,
/// `{{- end }}`, `{{- /* .. */ -}}`) and replace each inline `{{ ... }}` value expression with a
/// scalar placeholder. Pure helper, exposed for tests.
pub fn neutralize_helm(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
            continue;
        }
        out.push_str(&replace_inline_actions(line));
        out.push('\n');
    }
    out
}

/// Replace each single-line `{{ ... }}` expression in `line` with a scalar placeholder. A line whose
/// `{{` has no closing `}}` (a multi-line action) is collapsed to a placeholder from the `{{` on.
fn replace_inline_actions(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                out.push_str("helmvalue");
                rest = &after[end + 2..];
            }
            None => {
                out.push_str("helmvalue");
                return out;
            }
        }
    }
    out.push_str(rest);
    out
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

fn scoped_secret_verbs(policy: &Value) -> Vec<String> {
    policy
        .get("scoped_secret_verbs")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn json_str_seq(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|seq| {
            seq.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Pure evaluator. `policy` is DATA (`operator-secret-bootstrap-policy.json`); `observed` is the view
/// shaped by [`collect_operators`]. RED iff any governed operator grants a scoped secret verb without
/// name-scoping to its produced Secret, scopes the wrong Secret, or consumes an unprovisioned
/// bootstrap Secret.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "OSB-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let scoped = scoped_secret_verbs(policy);
    if scoped.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `scoped_secret_verbs` must be a non-empty array (the RBAC verbs that honor resourceNames); correct the policy before the gate can evaluate",
        ));
        return findings;
    }

    let Some(descriptors) = policy.get("operators").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `operators` must be an array of governed-operator descriptors; correct the policy",
        ));
        return findings;
    };
    if descriptors.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `operators` resolved to zero operators; the gate would have nothing to enforce — correct the policy",
        ));
        return findings;
    }

    let observed_ops = observed
        .get("operators")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for descriptor in descriptors {
        let Some(name) = descriptor.get("name").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "OSB-POLICY-MALFORMED",
                POLICY_KEY,
                "an operator descriptor is missing its `name`",
            ));
            continue;
        };
        let produced = descriptor
            .get("produced_secret_name")
            .and_then(Value::as_str)
            .unwrap_or_default();

        // Fail closed if the collector saw no observed view for a policy-declared operator.
        let Some(observed_op) = observed_ops
            .iter()
            .find(|op| op.get("name").and_then(Value::as_str) == Some(name))
        else {
            findings.insert(Finding::new(
                "OSB-JOIN-TOKEN-UNPROVISIONED",
                name,
                "no observed view for this operator (collector did not see its chart) — fail-closed",
            ));
            continue;
        };

        evaluate_secret_rules(observed_op, name, produced, &scoped, &mut findings);

        let provisioned = observed_op
            .get("has_provisioning_template")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let preflight = observed_op
            .get("has_failclosed_preflight")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !provisioned && !preflight {
            findings.insert(Finding::new(
                "OSB-JOIN-TOKEN-UNPROVISIONED",
                name,
                "the operator-internal bootstrap Secret has no declarative provisioning template (ExternalSecret/SealedSecret/Secret) and no fail-closed chart preflight",
            ));
        }
    }

    findings
}

fn evaluate_secret_rules(
    observed_op: &Value,
    name: &str,
    produced: &str,
    scoped: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let rules = observed_op
        .get("secret_rules")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for rule in &rules {
        let verbs = json_str_seq(rule.get("verbs"));
        let resource_names = json_str_seq(rule.get("resource_names"));
        let scoped_present: Vec<&String> =
            verbs.iter().filter(|v| scoped.contains(v)).collect();
        if scoped_present.is_empty() {
            // list/watch/create-only rule: the irreducible namespace-wide RBAC floor.
            continue;
        }
        let verb_key = scoped_present
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if resource_names.is_empty() {
            findings.insert(Finding::new(
                "OSB-SECRET-RBAC-OVERBROAD",
                &format!("{name}:{verb_key}"),
                format!(
                    "secrets rule grants scoped verb(s) [{verb_key}] with NO resourceNames — must be name-scoped to {produced}"
                ),
            ));
            continue;
        }
        let expected: BTreeSet<&str> = std::iter::once(produced).collect();
        let actual: BTreeSet<&str> = resource_names.iter().map(String::as_str).collect();
        if actual != expected {
            findings.insert(Finding::new(
                "OSB-SECRET-RBAC-RESOURCENAME-MISMATCH",
                &format!("{name}:{verb_key}"),
                format!(
                    "scoped secrets rule resourceNames {actual:?} is not exactly {{{produced}}}"
                ),
            ));
        }
    }
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "scoped_secret_verbs": ["get", "update", "patch", "delete"],
            "operators": [{
                "name": "op",
                "rbac_template": "rbac.yaml",
                "chart_templates_dir": "templates",
                "produced_secret_name": "the-secret",
                "join_token_values_ref": "svidOperator.joinToken.secretName"
            }]
        })
    }

    fn green_observed() -> Value {
        json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [
                { "verbs": ["get", "update", "patch"], "resource_names": ["the-secret"] },
                { "verbs": ["list", "watch", "create"], "resource_names": [] }
            ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]})
    }

    #[test]
    fn least_privilege_chart_is_green() {
        let report = evaluate(&policy(), &green_observed());
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
        assert!(evaluate_keyed(&policy(), &green_observed()).is_empty());
    }

    #[test]
    fn unscoped_get_update_patch_is_overbroad() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [
                { "verbs": ["get", "list", "watch", "create", "update", "patch"], "resource_names": [] }
            ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-SECRET-RBAC-OVERBROAD"), "{codes:?}");
    }

    #[test]
    fn listwatchcreate_only_rule_is_allowed_unscoped() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["list", "watch", "create"], "resource_names": [] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": false
        }]});
        assert!(
            evaluate_keyed(&policy(), &observed).is_empty(),
            "list/watch/create without resourceNames is the irreducible RBAC floor"
        );
    }

    #[test]
    fn scoping_the_wrong_secret_is_a_mismatch() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["get", "update"], "resource_names": ["some-other-secret"] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-SECRET-RBAC-RESOURCENAME-MISMATCH"), "{codes:?}");
    }

    #[test]
    fn extra_scoped_secret_is_a_mismatch() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["patch"], "resource_names": ["the-secret", "extra"] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-SECRET-RBAC-RESOURCENAME-MISMATCH"), "{codes:?}");
    }

    #[test]
    fn unprovisioned_join_token_fires() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["get"], "resource_names": ["the-secret"] } ],
            "has_provisioning_template": false,
            "has_failclosed_preflight": false
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-JOIN-TOKEN-UNPROVISIONED"), "{codes:?}");
    }

    #[test]
    fn either_provisioning_or_preflight_satisfies_bootstrap() {
        for (prov, pre) in [(true, false), (false, true)] {
            let observed = json!({ "operators": [{
                "name": "op",
                "produced_secret_name": "the-secret",
                "secret_rules": [ { "verbs": ["get"], "resource_names": ["the-secret"] } ],
                "has_provisioning_template": prov,
                "has_failclosed_preflight": pre
            }]});
            assert!(
                evaluate_keyed(&policy(), &observed).is_empty(),
                "provisioning OR preflight should satisfy the bootstrap contract ({prov},{pre})"
            );
        }
    }

    #[test]
    fn missing_observed_operator_fails_closed() {
        let observed = json!({ "operators": [] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-JOIN-TOKEN-UNPROVISIONED"), "{codes:?}");
    }

    #[test]
    fn wrong_gate_id_fails_closed() {
        let mut p = policy();
        p["gate_id"] = json!("nope");
        assert!(
            evaluate_keyed(&p, &green_observed())
                .iter()
                .any(|f| f.code == "OSB-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn empty_operators_fails_closed() {
        let mut p = policy();
        p["operators"] = json!([]);
        assert_eq!(evaluate(&p, &green_observed()).verdict, Verdict::Red);
    }

    #[test]
    fn neutralize_drops_directives_and_inlines() {
        let src = "{{- if .Values.x }}\nname: {{ .Values.n | quote }}\nstatic: yes\n{{- end }}";
        let out = neutralize_helm(src);
        assert!(!out.contains("if .Values"));
        assert!(!out.contains("end"));
        assert!(out.contains("name: helmvalue"));
        assert!(out.contains("static: yes"));
    }

    #[test]
    fn parse_secret_rules_extracts_scoped_and_unscoped() {
        let rbac = "{{- if .Values.x }}\napiVersion: rbac.authorization.k8s.io/v1\nkind: Role\nmetadata:\n  name: {{ .Values.n | quote }}\nrules:\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    resourceNames: [\"the-secret\"]\n    verbs: [\"get\", \"update\", \"patch\"]\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    verbs: [\"list\", \"watch\", \"create\"]\n  - apiGroups: [\"\"]\n    resources: [\"events\"]\n    verbs: [\"create\"]\n{{- end }}";
        let rules = parse_secret_rules(rbac).unwrap();
        assert_eq!(rules.len(), 2, "two secrets rules (events excluded): {rules:?}");
        assert_eq!(rules[0].resource_names, vec!["the-secret".to_owned()]);
        assert!(rules[1].resource_names.is_empty());
    }

    #[test]
    fn provisioning_and_preflight_text_signals() {
        let group = "svidOperator.joinToken";
        let es = "kind: ExternalSecret\ntarget:\n  name: {{ .Values.svidOperator.joinToken.secretName }}";
        assert!(scan_text_provisions_join_token(es, group));
        assert!(!scan_text_has_failclosed_preflight(es, group));
        let pre = "{{- fail \"svidOperator.joinToken has no provisioning contract\" -}}";
        assert!(scan_text_has_failclosed_preflight(pre, group));
        let deployment = "kind: Deployment\nenv:\n  name: {{ .Values.svidOperator.joinToken.secretName }}";
        assert!(!scan_text_provisions_join_token(deployment, group), "consumer is not a provisioner");
    }
}
