//! # cloud-ci-root-workspace-hygiene (ADR-0600)
//!
//! Born-blocking, UNIVERSAL, HERMETIC root-workspace-hygiene gate that makes committed
//! repo-root scratch structurally impossible. The gate is productized policy, not Oyatie-only
//! glue: the legitimate root surface lives in DATA (`root-workspace-hygiene-policy.json`), while
//! this crate evaluates the portable contract that EVERY tracked file at the repository ROOT
//! matches the allowlist and EVERY tracked top-level directory is a permitted capability/meta home.
//! Runtime/agent state directories can additionally be marked restricted so only explicitly
//! allowlisted tracked config/provenance paths are admitted; local state stays ignored.
//!
//! ## Posture: default-DENY (allowlist), complementing the scratch DENYLIST
//! The existing `cloud-ci-total-accounting` `scratch_artifact` code is a DENYLIST: it catches
//! KNOWN scratch shapes (`*.log`, `run-slice.sh`, …) by name. This gate is the complement — an
//! ALLOWLIST: any tracked root file that matches NO allowlist rule fails, so a scratch shape that
//! nobody has named yet is STILL born-blocking. The two layers compose into "impossible to commit
//! unjustified repo-root scratch" (founder directive).
//!
//! ## Pure evaluator (zero I/O)
//! The producer side supplies the git-ls-files snapshot (scm-facts) as DATA; this crate is a pure
//! evaluator over `{ "rows": [{"path": "..."}] }` (the tracked-path inventory) plus the committed
//! allowlist policy. `evaluate_keyed` returns one `Finding{code,key,detail}` per violation;
//! `evaluate` is the bare report projection. No shell, net, clock, rand, or filesystem access.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The gate id (matches the buck2 target + the policy `gate_id`).
pub const GATE_ID: &str = "cloud-ci-root-workspace-hygiene";

/// The blocking violation codes (stable slugs).
pub const VIOLATION_CODES: [&str; 11] = [
    // The policy `gate_id` does not match GATE_ID (config integrity).
    "root_workspace_gate_id_mismatch",
    // A tracked file at the repo ROOT matches no allowlist rule — born-blocking root scratch.
    "root_workspace_unallowlisted_file",
    // A tracked path's top-level directory is not a permitted capability/meta home.
    "root_workspace_unallowlisted_dir",
    // A tracked path under a restricted runtime/state directory is not explicitly allowlisted.
    "root_workspace_restricted_dir_unallowlisted_path",
    // An allowlist rule is malformed (missing/blank id, kind, or value).
    "root_workspace_policy_malformed_rule",
    // A tracked UTF-8 document has a sensitive key subset from a generated Talos machine config.
    "credential_bearing_talos_machine_config",
    // Corpus-budget dimension (anti-friction wave 3): shrink-only counts over the tracked-path
    // inventory for the doc/evidence/planning classes. Growth is born-blocking; a deliberate
    // budget raise is a reviewed DATA edit of `corpus_budget.counts`.
    "corpus_budget_evidence_files_grew",
    "corpus_budget_planning_files_grew",
    "corpus_budget_docs_markdown_grew",
    "corpus_budget_live_adrs_grew",
    "corpus_budget_malformed",
];

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

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim)
}

/// A single parsed allowlist rule: a match `kind` over the file basename or full tracked path and
/// a `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AllowRule {
    id: String,
    kind: String,
    value: String,
}

/// True iff `basename` is admitted by this rule.
fn rule_matches(rule: &AllowRule, basename: &str) -> bool {
    match rule.kind.as_str() {
        "exact" => basename == rule.value,
        "suffix" => basename.ends_with(&rule.value),
        "prefix" => basename.starts_with(&rule.value),
        // `prefix_dot`: exact match OR starts-with `value` followed by `.` or `-`.
        // Tighter than bare `prefix`: `README` matches README and README.md but NOT READMEILY.
        // Pattern in DATA: `{ "kind": "prefix_dot", "value": "README" }`.
        "prefix_dot" => {
            basename == rule.value
                || basename.starts_with(&format!("{}.", rule.value))
                || basename.starts_with(&format!("{}-", rule.value))
        }
        // Unknown kinds never match (the malformed-rule finding flags them separately).
        _ => false,
    }
}

/// Parse a rule table, emitting `root_workspace_policy_malformed_rule` for any rule missing a
/// non-empty id/kind/value or carrying an unknown kind.
fn allow_rules_from(
    policy: &Value,
    table: &str,
    rule_subject: &str,
    findings: &mut BTreeSet<Finding>,
) -> Vec<AllowRule> {
    let mut rules = Vec::new();
    for (index, raw) in policy
        .get(table)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let id = string_field(raw, "id").unwrap_or("");
        let kind = string_field(raw, "kind").unwrap_or("");
        let value = string_field(raw, "value").unwrap_or("");
        let key = if id.is_empty() {
            format!("{table}[{index}]")
        } else {
            id.to_owned()
        };
        if id.is_empty()
            || value.is_empty()
            || !matches!(kind, "exact" | "suffix" | "prefix" | "prefix_dot")
        {
            findings.insert(Finding::new(
                "root_workspace_policy_malformed_rule",
                &key,
                format!("{rule_subject} rule must carry a non-empty `id`, a non-empty `value`, and a `kind` of exact|suffix|prefix|prefix_dot"),
            ));
            continue;
        }
        rules.push(AllowRule {
            id: id.to_owned(),
            kind: kind.to_owned(),
            value: value.to_owned(),
        });
    }
    rules
}

/// Parse the `allowed_root_files` rule table.
fn allow_rules(policy: &Value, findings: &mut BTreeSet<Finding>) -> Vec<AllowRule> {
    allow_rules_from(policy, "allowed_root_files", "root allowlist", findings)
}

/// The set of permitted top-level directory names (data-driven).
fn allowed_dirs(policy: &Value) -> BTreeSet<String> {
    policy
        .get("allowed_root_dirs")
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

/// Top-level directories whose tracked contents must be explicitly allowlisted path-by-path.
fn restricted_roots(policy: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    for (index, raw) in policy
        .get("restricted_tracked_roots")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(root) = raw.as_str().map(str::trim).filter(|root| !root.is_empty()) else {
            findings.insert(Finding::new(
                "root_workspace_policy_malformed_rule",
                &format!("restricted_tracked_roots[{index}]"),
                "restricted root entries must be non-empty top-level directory names",
            ));
            continue;
        };
        if root.contains('/') {
            findings.insert(Finding::new(
                "root_workspace_policy_malformed_rule",
                root,
                "restricted root entries must name a top-level directory and must not contain `/`",
            ));
            continue;
        }
        roots.insert(root.to_owned());
    }
    roots
}

/// Parse exact tracked path exceptions for restricted runtime/state roots.
fn allowed_tracked_paths(policy: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    allow_rules_from(
        policy,
        "allowed_tracked_paths",
        "tracked-path allowlist",
        findings,
    )
    .into_iter()
    .filter_map(|rule| {
        if rule.kind == "exact" {
            Some(rule.value)
        } else {
            findings.insert(Finding::new(
                "root_workspace_policy_malformed_rule",
                &rule.id,
                "tracked-path allowlist rules must use `kind: exact` to avoid broad runtime-state merge surfaces",
            ));
            None
        }
    })
    .collect()
}

/// The remediation printed for an unallowlisted root file (auto-fix, not flag-only).
fn root_file_remediation(path: &str) -> String {
    format!(
        "tracked repo-root file `{path}` matches no allowlist rule. AUTO-FIX: if it is process \
         scratch, `git rm` it (and rely on the .gitignore root-scratch backstop) or relocate it \
         under the repo's gitignored scratch home (e.g. `.omc/`); if it is a genuinely legitimate \
         root surface, add a reviewed allowlist rule to root-workspace-hygiene-policy.json \
         (allowed_root_files) — a DATA edit, never a scanner change."
    )
}

/// The remediation printed for an unallowlisted top-level directory.
fn root_dir_remediation(dir: &str) -> String {
    format!(
        "tracked path lives under unallowlisted top-level directory `{dir}/`. AUTO-FIX: relocate \
         the file under an existing capability/meta home, or — if a NEW top-level capability is \
         genuinely warranted — add `{dir}` to allowed_root_dirs in \
         root-workspace-hygiene-policy.json (a reviewed DATA edit)."
    )
}

/// The remediation printed for a tracked path inside a restricted runtime/state directory.
fn restricted_path_remediation(path: &str, root: &str) -> String {
    format!(
        "tracked path `{path}` lives under restricted runtime/state directory `{root}/` but is not \
         explicitly allowlisted. AUTO-FIX: if it is local runtime/cache/worktree state, `git rm` \
         it and keep it ignored; if it is intentional shared config or durable provenance, add a \
         reviewed exact tracked-path rule to root-workspace-hygiene-policy.json \
         (allowed_tracked_paths)."
    )
}

/// Pure evaluator. `policy` is DATA (`root-workspace-hygiene-policy.json`); `observed` is the
/// tracked-path inventory shaped as `{ "rows": [{"path": "..."}] }` (the producer's
/// git-ls-files snapshot). Every tracked path whose basename carries no `/` is a ROOT file and
/// must match the allowlist; every nested tracked path's first segment must be a permitted dir.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if string_field(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "root_workspace_gate_id_mismatch",
            "<policy>",
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let rules = allow_rules(policy, &mut findings);
    let dirs = allowed_dirs(policy);
    let restricted = restricted_roots(policy, &mut findings);
    let allowed_paths = allowed_tracked_paths(policy, &mut findings);

    // De-duplicate top-level dirs so each offending dir is reported once with a stable key.
    let mut unallowlisted_dirs: BTreeMap<String, ()> = BTreeMap::new();

    for row in observed
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = string_field(row, "path").filter(|p| !p.is_empty()) else {
            continue;
        };
        // Normalize away any leading "./" the snapshot might carry.
        let path = path.strip_prefix("./").unwrap_or(path);

        match path.split_once('/') {
            // Nested path: its first segment must be a permitted top-level directory.
            Some((top, _rest)) => {
                if !dirs.contains(top) {
                    unallowlisted_dirs.entry(top.to_owned()).or_insert(());
                }
                if restricted.contains(top) && !allowed_paths.contains(path) {
                    findings.insert(Finding::new(
                        "root_workspace_restricted_dir_unallowlisted_path",
                        path,
                        restricted_path_remediation(path, top),
                    ));
                }
            }
            // Root-level file (no '/'): basename must match an allowlist rule.
            None => {
                let admitted = rules.iter().any(|rule| rule_matches(rule, path));
                if !admitted {
                    findings.insert(Finding::new(
                        "root_workspace_unallowlisted_file",
                        path,
                        root_file_remediation(path),
                    ));
                }
            }
        }
    }

    for dir in unallowlisted_dirs.keys() {
        findings.insert(Finding::new(
            "root_workspace_unallowlisted_dir",
            dir,
            root_dir_remediation(dir),
        ));
    }

    findings.extend(evaluate_corpus_budget(policy, observed));

    findings
}

/// Bare-report projection of [`evaluate_keyed`].
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// Corpus-budget dimension (anti-friction wave 3, ADR-0716 doctrine): shrink-only counts over
/// the tracked-path inventory for the four sprawl classes — evidence files, planning artifacts
/// (tasks/ + plan/ + ci/evidence/), docs markdown, and live apex ADRs. Any class growing past its
/// frozen `corpus_budget.counts` ceiling is born-blocking with a one-in-one-out remediation.
/// A deliberate budget raise is a reviewed DATA edit of the policy (never a scanner change).
pub fn evaluate_corpus_budget(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(counts) = policy.get("corpus_budget").and_then(|budget| budget.get("counts")) else {
        return findings;
    };

    let mut counters: BTreeMap<&str, usize> = BTreeMap::new();
    for row in observed
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = string_field(row, "path").filter(|path| !path.is_empty()) else {
            continue;
        };
        let path = path.strip_prefix("./").unwrap_or(path);
        if path.starts_with("evidence/") {
            *counters.entry("evidence_files").or_default() += 1;
        }
        if path.starts_with("tasks/") || path.starts_with("plan/") || path.starts_with("ci/evidence/") {
            *counters.entry("planning_files").or_default() += 1;
        }
        if path.starts_with("docs/") && path.ends_with(".md") {
            *counters.entry("docs_markdown_files").or_default() += 1;
        }
        if path.starts_with("docs/decisions/ADR-") && path.ends_with(".md") {
            *counters.entry("live_adr_files").or_default() += 1;
        }
    }

    let classes: [(&str, &str, &str); 4] = [
        (
            "evidence_files",
            "corpus_budget_evidence_files_grew",
            "retire an evidence file in the same PR (one-in-one-out) or raise the reviewed corpus_budget counts",
        ),
        (
            "planning_files",
            "corpus_budget_planning_files_grew",
            "complete-then-delete a planning file in the same PR or raise the reviewed corpus_budget counts",
        ),
        (
            "docs_markdown_files",
            "corpus_budget_docs_markdown_grew",
            "retire a markdown doc in the same PR (markdown-retirement policy) or raise the reviewed corpus_budget counts",
        ),
        (
            "live_adr_files",
            "corpus_budget_live_adrs_grew",
            "retire a live ADR to the archive in the same PR (one-in-one-out) or raise the reviewed corpus_budget counts",
        ),
    ];
    for (class, code, remediation) in classes {
        let observed_count = counters.get(class).copied().unwrap_or(0);
        let Some(frozen_count) = counts.get(class).and_then(Value::as_u64) else {
            findings.insert(Finding::new(
                "corpus_budget_malformed",
                "<corpus_budget>",
                format!("corpus_budget.counts must carry a numeric {class}"),
            ));
            continue;
        };
        if observed_count > frozen_count as usize {
            findings.insert(Finding::new(
                code,
                &format!("<{class}> {observed_count} > {frozen_count}"),
                remediation,
            ));
        }
    }
    findings
}

/// Extract YAML mapping key paths without retaining or inspecting scalar values.
///
/// Generated Talos machine configurations are YAML mappings whose credential-bearing topology is
/// stable even when an operator renames the file. This deliberately small parser considers only
/// plain mapping keys and indentation. Text after the first `:` is discarded immediately, so a
/// finding can never echo a token, certificate, private key, or other scalar value. Reviewed Talos
/// patches/templates remain below the fingerprint when they contain no private key, token, or
/// secret path from the generated credential topology.
fn yaml_plain_mapping_key_paths(document: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut parents: Vec<(usize, String)> = Vec::new();
    let mut saw_mapping = false;

    for line in document.lines() {
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("---")
            || trimmed.starts_with("...")
        {
            continue;
        }

        let Some((raw_key, _discarded_scalar)) = trimmed.split_once(':') else {
            if !saw_mapping {
                return BTreeSet::new();
            }
            continue;
        };
        let key = raw_key.trim();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            if !saw_mapping {
                return BTreeSet::new();
            }
            continue;
        }
        if !saw_mapping && indent != 0 {
            return BTreeSet::new();
        }
        saw_mapping = true;

        while parents.last().is_some_and(|(depth, _)| *depth >= indent) {
            parents.pop();
        }

        let path = parents
            .iter()
            .map(|(_, parent)| parent.as_str())
            .chain(std::iter::once(key))
            .collect::<Vec<_>>()
            .join(".");
        paths.insert(path);
        parents.push((indent, key.to_owned()));
    }

    paths
}

fn has_generated_talos_machine_config_topology(document: &str) -> bool {
    let paths = yaml_plain_mapping_key_paths(document);
    [
        "machine.token",
        "machine.ca.key",
        "cluster.secret",
        "cluster.token",
        "cluster.secretboxEncryptionSecret",
        "cluster.ca.key",
    ]
    .into_iter()
    .any(|sensitive_path| paths.contains(sensitive_path))
}

/// Reject tracked UTF-8 documents that contain any sensitive credential-bearing key path from a
/// generated Talos machine configuration. Findings intentionally contain only the
/// repository-relative path and a fixed remediation; document contents and scalar values are
/// never included.
pub fn evaluate_talos_machine_config_documents<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> BTreeSet<Finding> {
    documents
        .into_iter()
        .filter_map(|(path, document)| {
            has_generated_talos_machine_config_topology(document).then(|| {
                Finding::new(
                    "credential_bearing_talos_machine_config",
                    path,
                    format!(
                        "tracked document `{path}` contains sensitive generated Talos machine-config credential topology. AUTO-FIX: remove it from git and regenerate outside the repository; retain only reviewed value-free patches/templates. Diagnostic output is value-redacted."
                    ),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "allowed_root_files": [
                { "id": "cargo-manifest", "kind": "exact",      "value": "Cargo.toml" },
                { "id": "readme",         "kind": "prefix_dot", "value": "README" },
                { "id": "license",        "kind": "prefix_dot", "value": "LICENSE" },
                { "id": "buckconfig",     "kind": "prefix_dot", "value": ".buckconfig" }
            ],
            "allowed_root_dirs": [".claude", ".codex", ".omc", "ci", "cloud", "libs", "docs"],
            "restricted_tracked_roots": [".claude", ".codex", ".omc", ".omx"],
            "allowed_tracked_paths": [
                { "id": "claude-settings", "kind": "exact", "value": ".claude/settings.json" },
                { "id": "codex-hooks", "kind": "exact", "value": ".codex/hooks.json" },
                { "id": "omc-ultragoal-owners", "kind": "exact", "value": ".omc/ultragoal/OWNERS" }
            ]
        })
    }

    fn observed(paths: &[&str]) -> Value {
        json!({ "rows": paths.iter().map(|p| json!({ "path": p })).collect::<Vec<_>>() })
    }

    #[test]
    fn generated_talos_machine_config_structure_is_redacted_and_red() {
        let document = r#"
version: v1alpha1
machine:
  type: controlplane
  token: DO_NOT_ECHO_MACHINE_TOKEN
  ca:
    crt: DO_NOT_ECHO_MACHINE_CERT
    key: DO_NOT_ECHO_MACHINE_KEY
cluster:
  id: DO_NOT_ECHO_CLUSTER_ID
  secret: DO_NOT_ECHO_CLUSTER_SECRET
  token: DO_NOT_ECHO_CLUSTER_TOKEN
  secretboxEncryptionSecret: DO_NOT_ECHO_SECRETBOX_KEY
  ca:
    crt: DO_NOT_ECHO_CLUSTER_CERT
    key: DO_NOT_ECHO_CLUSTER_KEY
"#;

        let findings = evaluate_talos_machine_config_documents([(
            "recovery/renamed-machine-config.yaml",
            document,
        )]);
        let finding = findings
            .iter()
            .find(|finding| finding.code == "credential_bearing_talos_machine_config")
            .expect("credential-bearing Talos machine-config structure must be rejected");

        assert_eq!(finding.key, "recovery/renamed-machine-config.yaml");
        assert!(
            !format!("{finding:?}").contains("DO_NOT_ECHO"),
            "the finding must contain path and remediation only, never document scalar values"
        );
    }

    #[test]
    fn partial_talos_credential_subsets_are_each_red() {
        for (path, document) in [
            (
                "recovery/controlplane",
                "machine:\n  token: DO_NOT_ECHO_MACHINE_TOKEN\n",
            ),
            (
                "recovery/controlplane.txt",
                "machine:\n  ca:\n    key: DO_NOT_ECHO_MACHINE_KEY\n",
            ),
            (
                "recovery/partial-cluster-config",
                "cluster:\n  secret: DO_NOT_ECHO_CLUSTER_SECRET\n",
            ),
            (
                "recovery/partial-cluster-config.txt",
                "cluster:\n  token: DO_NOT_ECHO_CLUSTER_TOKEN\n",
            ),
            (
                "recovery/partial-secretbox",
                "cluster:\n  secretboxEncryptionSecret: DO_NOT_ECHO_SECRETBOX_KEY\n",
            ),
            (
                "recovery/partial-cluster-ca",
                "cluster:\n  ca:\n    key: DO_NOT_ECHO_CLUSTER_KEY\n",
            ),
        ] {
            let findings = evaluate_talos_machine_config_documents([(path, document)]);
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.code == "credential_bearing_talos_machine_config"),
                "sensitive Talos credential subset at extension-independent path {path} must be rejected"
            );
            assert!(
                !format!("{findings:?}").contains("DO_NOT_ECHO"),
                "subset findings must remain value-redacted"
            );
        }
    }

    #[test]
    fn public_talos_certificate_topology_without_private_credentials_is_green() {
        let public_only = r#"
machine:
  ca:
    crt: public-certificate
cluster:
  ca:
    crt: public-certificate
"#;
        assert!(
            evaluate_talos_machine_config_documents([("review/public-ca.yaml", public_only)])
                .is_empty(),
            "public certificates without private keys/tokens/secrets are not credential-bearing"
        );
    }

    #[test]
    fn talos_patches_and_capi_templates_without_generated_credentials_are_green() {
        let patch = r#"
machine:
  install:
    disk: /dev/vda
cluster:
  network:
    cni:
      name: none
"#;
        let capi_template = r#"
apiVersion: bootstrap.cluster.x-k8s.io/v1alpha3
kind: TalosConfigTemplate
spec:
  template:
    spec:
      generateType: join
"#;

        assert!(
            evaluate_talos_machine_config_documents([
                ("infra/talos/controlplane.patch.yaml", patch),
                ("infra/capi/clusters/templates/clusters.yaml", capi_template),
            ])
            .is_empty(),
            "reviewable patches/templates without generated credential topology must remain allowed"
        );
    }

    #[test]
    fn clean_allowlisted_tree_is_green() {
        let report = evaluate(
            &policy(),
            &observed(&[
                "Cargo.toml",
                "README.md",
                "LICENSE",
                ".buckconfig",
                ".claude/settings.json",
                ".codex/hooks.json",
                ".omc/ultragoal/OWNERS",
                "ci/facade/x/src/lib.rs",
                "libs/oya-foo/Cargo.toml",
                "docs/adr-archive/ADR-0600-root-workspace-hygiene-allowlist-gate.md",
            ]),
        );
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.violations.is_empty(), "{report:#?}");
    }

    #[test]
    fn tracked_root_scratch_log_is_born_blocking_red() {
        // The load-bearing RED case: a `foo.log` tracked at root matches no allowlist rule.
        let findings = evaluate_keyed(&policy(), &observed(&["Cargo.toml", "foo.log"]));
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "root_workspace_unallowlisted_file" && f.key == "foo.log" }),
            "a tracked root scratch file must be born-blocking with its key surfaced; got {findings:#?}"
        );
        // The legitimate root file must NOT be flagged (no false positive).
        assert!(
            !findings.iter().any(|f| f.key == "Cargo.toml"),
            "an allowlisted root file must not be flagged"
        );
        assert_eq!(
            evaluate(&policy(), &observed(&["foo.log"])).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn the_actual_removed_scratch_shapes_are_red() {
        // The exact root scratch this PR removes must each fail the allowlist.
        for scratch in [
            "backfill-targets.txt",
            "branch-wired-members.txt",
            "final-targets.txt",
            "slice06-progress.log",
            "retest-targets.txt",
            "run-slice.sh",
            "premise.txt",
            "review-verdict.txt",
        ] {
            let findings = evaluate_keyed(&policy(), &observed(&[scratch]));
            assert!(
                findings
                    .iter()
                    .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == scratch),
                "{scratch} must be born-blocking"
            );
        }
    }

    #[test]
    fn finding_carries_a_concrete_auto_fix_remediation() {
        let findings = evaluate_keyed(&policy(), &observed(&["foo.log"]));
        let f = findings
            .iter()
            .find(|f| f.key == "foo.log")
            .expect("finding for foo.log");
        assert!(
            f.detail.contains("git rm") && f.detail.contains(".omc/"),
            "remediation must name the concrete auto-fix (relocate to .omc/ or git rm); got: {}",
            f.detail
        );
    }

    #[test]
    fn unallowlisted_top_level_dir_is_red_and_deduped() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(&["sandbox/a.rs", "sandbox/b.rs", "cloud/ok.rs"]),
        );
        let dir_findings: Vec<&Finding> = findings
            .iter()
            .filter(|f| f.code == "root_workspace_unallowlisted_dir")
            .collect();
        assert_eq!(
            dir_findings.len(),
            1,
            "the offending dir is reported once: {findings:#?}"
        );
        assert_eq!(dir_findings[0].key, "sandbox");
    }

    #[test]
    fn restricted_runtime_state_paths_are_born_blocking_red() {
        for path in [
            ".claude/worktrees/old-lane/marker",
            ".claude/settings.local.json",
            ".codex/.DS_Store",
            ".omc/state/team/mailbox.json",
            ".omx/state/team/mailbox.json",
        ] {
            let findings = evaluate_keyed(&policy(), &observed(&[path]));
            assert!(
                findings.iter().any(|f| {
                    f.code == "root_workspace_restricted_dir_unallowlisted_path" && f.key == path
                }),
                "{path} must be born-blocking under restricted runtime/state roots; got {findings:#?}"
            );
        }
    }

    #[test]
    fn explicit_shared_agent_config_paths_are_green() {
        let report = evaluate(
            &policy(),
            &observed(&[
                ".claude/settings.json",
                ".codex/hooks.json",
                ".omc/ultragoal/OWNERS",
            ]),
        );
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "explicit tracked config/provenance exceptions must remain green; got {report:#?}"
        );
    }

    #[test]
    fn gate_id_mismatch_is_red() {
        let bad = json!({ "gate_id": "wrong", "allowed_root_files": [], "allowed_root_dirs": [] });
        let findings = evaluate_keyed(&bad, &observed(&[]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_gate_id_mismatch")
        );
    }

    #[test]
    fn malformed_allowlist_rule_is_red() {
        let bad = json!({
            "gate_id": GATE_ID,
            "allowed_root_files": [ { "id": "", "kind": "nope", "value": "" } ],
            "allowed_root_dirs": [],
            "restricted_tracked_roots": [""],
            "allowed_tracked_paths": [ { "id": "", "kind": "nope", "value": "" } ]
        });
        let findings = evaluate_keyed(&bad, &observed(&[]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_policy_malformed_rule")
        );
    }

    #[test]
    fn tracked_path_allowlist_rejects_broad_match_kinds() {
        let mut bad = policy();
        bad["allowed_tracked_paths"] =
            json!([{ "id": "broad-omc", "kind": "prefix", "value": ".omc/" }]);
        let findings = evaluate_keyed(&bad, &observed(&[]));
        assert!(
            findings.iter().any(|f| {
                f.code == "root_workspace_policy_malformed_rule" && f.key == "broad-omc"
            }),
            "runtime/state tracked path exceptions must stay exact; got {findings:#?}"
        );
    }

    // --- prefix_dot tightening: RED cases (over-broad prefix would have allowed these) ---

    #[test]
    fn readme_family_without_separator_is_red() {
        // "READMEILY.md" starts with "README" but has no "." or "-" separator — must be blocked.
        let findings = evaluate_keyed(&policy(), &observed(&["READMEILY.md"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "READMEILY.md"),
            "READMEILY.md must be born-blocking (no separator after README); got {findings:#?}"
        );
    }

    #[test]
    fn readme_scratch_txt_without_separator_is_red() {
        let findings = evaluate_keyed(&policy(), &observed(&["README-scratch.txt"]));
        // README-scratch.txt HAS a "-" separator so prefix_dot admits it — this is intentional
        // (README-* is a legitimate family). Verify GREEN (no false-block).
        assert!(
            !findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file"
                    && f.key == "README-scratch.txt"),
            "README-scratch.txt has a '-' separator and should be admitted by prefix_dot; got {findings:#?}"
        );
    }

    #[test]
    fn notes_buckconfig_is_red() {
        // "notes.buckconfig" ends with ".buckconfig" (old suffix rule allowed it) but does NOT
        // start with ".buckconfig" — must now be born-blocking.
        let findings = evaluate_keyed(&policy(), &observed(&["notes.buckconfig"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file"
                    && f.key == "notes.buckconfig"),
            "notes.buckconfig must be born-blocking (suffix match removed); got {findings:#?}"
        );
    }

    // --- corpus-budget dimension (anti-friction wave 3) ---

    fn corpus_policy() -> Value {
        json!({
            "corpus_budget": {
                "counts": {
                    "evidence_files": 2,
                    "planning_files": 1,
                    "docs_markdown_files": 2,
                    "live_adr_files": 1
                }
            }
        })
    }

    fn corpus_observed(paths: &[&str]) -> Value {
        json!({ "rows": paths.iter().map(|path| json!({ "path": path })).collect::<Vec<_>>() })
    }

    #[test]
    fn corpus_budget_at_frozen_counts_is_green() {
        let findings = evaluate_corpus_budget(
            &corpus_policy(),
            &corpus_observed(&[
                "evidence/a.json",
                "evidence/b.json",
                "tasks/x-plan.md",
                "docs/a.md",
                "docs/decisions/ADR-0700-x.md",
            ]),
        );
        assert!(findings.is_empty(), "frozen corpus must be green; got {findings:#?}");
    }

    #[test]
    fn corpus_budget_growth_is_born_blocking_per_class() {
        let findings = evaluate_corpus_budget(
            &corpus_policy(),
            &corpus_observed(&[
                "evidence/a.json",
                "evidence/b.json",
                "evidence/c.json",
                "tasks/x-plan.md",
                "plan/extra.md",
                "docs/a.md",
                "docs/b.md",
                "docs/c.md",
                "docs/decisions/ADR-0700-x.md",
                "docs/decisions/ADR-0701-y.md",
            ]),
        );
        for code in [
            "corpus_budget_evidence_files_grew",
            "corpus_budget_planning_files_grew",
            "corpus_budget_docs_markdown_grew",
            "corpus_budget_live_adrs_grew",
        ] {
            assert!(
                findings.iter().any(|f| f.code == code),
                "growth must be born-blocking for {code}; got {findings:#?}"
            );
        }
    }

    #[test]
    fn corpus_budget_missing_count_is_malformed() {
        let policy = json!({ "corpus_budget": { "counts": { "evidence_files": 1 } } });
        let findings = evaluate_corpus_budget(&policy, &corpus_observed(&["evidence/a.json"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "corpus_budget_malformed"),
            "a missing class count must fail closed as malformed; got {findings:#?}"
        );
    }

    #[test]
    fn scratch_buckconfig_is_red() {
        let findings = evaluate_keyed(&policy(), &observed(&["scratch.buckconfig"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file"
                    && f.key == "scratch.buckconfig"),
            "scratch.buckconfig must be born-blocking; got {findings:#?}"
        );
    }

    // --- prefix_dot tightening: GREEN cases (legitimate files must still pass) ---

    #[test]
    fn legitimate_readme_and_license_and_buckconfig_still_pass() {
        let report = evaluate(
            &policy(),
            &observed(&[
                "README",
                "README.md",
                "README.rst",
                "LICENSE",
                "LICENSE.md",
                "LICENSE-Apache-2.0",
                ".buckconfig",
                ".buckconfig.local",
            ]),
        );
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "legitimate README/LICENSE/.buckconfig family must not be false-blocked; got {report:#?}"
        );
    }

    #[test]
    fn evaluator_only_emits_declared_violation_codes() {
        let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
        let bad = json!({
            "gate_id": "wrong",
            "allowed_root_files": [ { "id": "", "kind": "x", "value": "" } ],
            "allowed_root_dirs": [],
            "restricted_tracked_roots": [".claude"],
            "allowed_tracked_paths": [],
            "corpus_budget": { "counts": {
                "evidence_files": 0,
                "planning_files": 0,
                "docs_markdown_files": 0,
                "live_adr_files": 0
            } }
        });
        let mut findings = evaluate_keyed(
            &bad,
            &observed(&[
                "foo.log",
                "sandbox/x.rs",
                ".claude/worktrees/x",
                "evidence/x.json",
                "tasks/p.md",
                "docs/d.md",
                "docs/decisions/ADR-0001-a.md",
            ]),
        );
        findings.extend(evaluate_talos_machine_config_documents([(
            "renamed.yaml",
            r#"
machine:
  token: redacted
  ca:
    crt: redacted
    key: redacted
cluster:
  id: redacted
  secret: redacted
  token: redacted
  secretboxEncryptionSecret: redacted
  ca:
    crt: redacted
    key: redacted
"#,
        )]));
        for f in &findings {
            assert!(
                declared.contains(f.code.as_str()),
                "evaluator emitted `{}` which is not in VIOLATION_CODES",
                f.code
            );
        }
        // The malformed-counts code is exercised by a policy whose counts object is present
        // but carries no class entries.
        findings.extend(evaluate_keyed(
            &json!({
                "gate_id": GATE_ID,
                "allowed_root_files": [],
                "allowed_root_dirs": [],
                "restricted_tracked_roots": [],
                "allowed_tracked_paths": [],
                "corpus_budget": { "counts": {} }
            }),
            &observed(&["evidence/x.json"]),
        ));
        // All eleven codes are exercised by the path-policy, corpus-budget, and Talos fixtures.
        let emitted: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(emitted, declared.iter().map(|s| s.to_string()).collect());
    }
}
