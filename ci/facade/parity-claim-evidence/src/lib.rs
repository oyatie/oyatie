//! # cloud-ci-hyperscaler-parity-taxonomy
//!
//! Rust/Buck2 replacement for the retired
//! `scripts/tests/cloud_hyperscaler_parity_taxonomy_check.py` validator. The evaluator is
//! pure over `specs/cloud-hyperscaler-parity-taxonomy.json` and blocks taxonomy drift,
//! unofficial source evidence, external-SaaS/public-cloud CI evidence lanes, and forbidden
//! production/parity/readiness claims before this target artifact is backed by runtime evidence.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-hyperscaler-parity-taxonomy";

const REQUIRED_CATEGORIES: &[&str] = &[
    "identity_access_policy",
    "compute_instances",
    "containers_kubernetes",
    "serverless_functions",
    "storage_object_block_file",
    "networking_dns_edge",
    "databases_data_analytics",
    "kms_secrets_confidentiality",
    "observability_operations",
    "billing_finops_quotas",
    "marketplace_isv_ecosystem",
    "security_posture_guardrails",
    "cloud_native_platform_contract",
];

const PROVIDERS: &[&str] = &["aws", "google_cloud", "azure", "oci"];
const REQUIRED_NONCLAIMS: &[&str] = &[
    "hyperscaler_mature",
    "provider_feature_parity",
    "production_ready",
    "tenant_workload_ready",
    "public_sla_or_slo",
    "live_provider_provisioning",
];
const REQUIRED_CONTROLS: &[&str] = &[
    "strict_separation",
    "pure_dogfood",
    "self_hosted_ci_lane",
    "no_external_hyperscaler_runtime_dependency",
];
const FORBIDDEN_CONTROL_MARKERS: &[&str] = &[
    "github_actions_fallback",
    "external_saas_ci",
    "public_cloud_runtime_dependency",
];
const FORBIDDEN_EVIDENCE_LANES: &[&str] = &[
    "github actions",
    "external saas",
    "github-actions",
    "public cloud",
];
const REQUIRED_EVIDENCE_CLASSES: &[&str] = &[
    "official-provider-category-evidence",
    "machine-readable resource contract",
    "implementation or adapter boundary",
    "targeted tests plus governance gate evidence",
    "measured operational evidence before production claim",
];
const VAGUE_EVIDENCE_MARKERS: &[&str] = &["todo", "tbd", "later", "placeholder", "fixme"];
const FORBIDDEN_CAN_CLAIM_PHRASES: &[&str] = &[
    "feature parity",
    "same feature parity",
    "hyperscaler mature",
    "hyperscaler-mature",
    "hyperscaler maturity",
    "reaches hyperscaler maturity",
    "production ready",
    "production-ready",
    "production readiness",
    "production-readiness",
    "tenant workload ready",
    "tenant/product workload readiness",
    "public sla",
    "public slo",
    "live provider provisioning",
    "provisions real cloud resources",
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
}

impl Finding {
    fn new(code: &str, key: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let violations: BTreeSet<String> =
            findings.into_iter().map(|finding| finding.code).collect();
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

pub fn evaluate(spec: &Value) -> Report {
    Report::from_findings(evaluate_keyed(spec))
}

pub fn evaluate_keyed(spec: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    require_top_level_fields(spec, &mut findings);
    require_eq_str(
        spec,
        "status",
        "Proposed-target",
        "status_not_proposed_target",
        &mut findings,
    );
    require_array_superset(
        spec,
        "controls",
        REQUIRED_CONTROLS,
        "missing_required_control",
        &mut findings,
    );
    forbid_array_overlap(
        spec,
        "controls",
        FORBIDDEN_CONTROL_MARKERS,
        "forbidden_control_marker",
        &mut findings,
    );
    require_bool_path(
        spec,
        &["strict_separation_constraints", "no_external_saas_ci"],
        true,
        "strict_separation_allows_external_saas_ci",
        &mut findings,
    );
    require_bool_path(
        spec,
        &["strict_separation_constraints", "no_live_provider_apply"],
        true,
        "strict_separation_allows_live_provider_apply",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "strict_separation_constraints",
            "no_github_actions_fallback",
        ],
        true,
        "strict_separation_allows_github_actions_fallback",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "strict_separation_constraints",
            "no_public_cloud_runtime_dependency",
        ],
        true,
        "strict_separation_allows_public_cloud_runtime_dependency",
        &mut findings,
    );
    forbid_text_markers(
        spec.pointer("/strict_separation_constraints/allowed_evidence_lanes"),
        FORBIDDEN_EVIDENCE_LANES,
        "forbidden_allowed_evidence_lane",
        "strict_separation_constraints.allowed_evidence_lanes",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "pure_dogfood_constraints",
            "self_hosted_github_kubernetes_ci_lane",
        ],
        true,
        "missing_self_hosted_github_kubernetes_ci_lane",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "pure_dogfood_constraints",
            "dogfood_resource_substrate_required_before_external_provider_apply",
        ],
        true,
        "missing_dogfood_resource_substrate_constraint",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "pure_dogfood_constraints",
            "vfkit_linux_or_kubernetes_cluster_tests_must_be_recorded_before_kubernetes_readiness_claim",
        ],
        true,
        "missing_kubernetes_readiness_evidence_constraint",
        &mut findings,
    );
    require_bool_path(
        spec,
        &[
            "pure_dogfood_constraints",
            "g007_must_reconcile_historical_ci_wording",
        ],
        true,
        "missing_g007_reconciliation_constraint",
        &mut findings,
    );
    require_array_superset_at(
        spec,
        &["evidence_vocabulary", "required_category_evidence_classes"],
        REQUIRED_EVIDENCE_CLASSES,
        "missing_required_evidence_class",
        &mut findings,
    );

    let coverage_by_provider = validate_sources(spec, &mut findings);
    validate_categories(spec, &coverage_by_provider, &mut findings);
    validate_mapping(spec, &mut findings);
    validate_claim_matrix(spec, &mut findings);
    validate_nonclaims(spec, &mut findings);
    validate_next_goal_mapping(spec, &mut findings);
    findings
}

fn require_top_level_fields(spec: &Value, findings: &mut BTreeSet<Finding>) {
    for field in [
        "spec_id",
        "title",
        "status",
        "retrieved_at",
        "purpose",
        "official_source_evidence",
        "local_authority",
        "controls",
        "strict_separation_constraints",
        "pure_dogfood_constraints",
        "evidence_vocabulary",
        "category_taxonomy",
        "local_oyatie_mapping",
        "claim_matrix",
        "nonclaims",
        "next_goal_mapping",
    ] {
        if spec.get(field).is_none() {
            findings.insert(Finding::new("missing_top_level_field", field));
        }
    }
}

fn validate_sources(
    spec: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut source_providers = BTreeSet::new();
    let mut coverage_by_provider: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let Some(sources) = spec
        .get("official_source_evidence")
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "official_source_evidence_not_array",
            "official_source_evidence",
        ));
        return coverage_by_provider;
    };

    for source in sources {
        let provider = source
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let url = source
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if provider.is_empty() {
            findings.insert(Finding::new(
                "source_missing_provider",
                "<missing-provider>",
            ));
            continue;
        }
        source_providers.insert(provider.to_owned());
        if !official_url(provider, url) {
            findings.insert(Finding::new("source_url_not_official", provider));
        }
        if source
            .get("evidence_use")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            findings.insert(Finding::new("source_missing_evidence_use", provider));
        }
        let coverage = string_set(source.get("category_coverage"));
        if coverage.is_empty() {
            findings.insert(Finding::new("source_missing_category_coverage", provider));
        }
        coverage_by_provider
            .entry(provider.to_owned())
            .or_default()
            .extend(coverage);
    }
    for provider in PROVIDERS.iter().copied().chain(["kubernetes", "cncf"]) {
        if !source_providers.contains(provider) {
            findings.insert(Finding::new("missing_official_source_provider", provider));
        }
    }
    coverage_by_provider
}

fn official_url(provider: &str, url: &str) -> bool {
    match provider {
        "aws" => {
            url.starts_with("https://aws.amazon.com/")
                || url.starts_with("https://docs.aws.amazon.com/")
        }
        "google_cloud" => {
            url.starts_with("https://cloud.google.com/")
                || url.starts_with("https://docs.cloud.google.com/")
        }
        "azure" => {
            url.starts_with("https://azure.microsoft.com/")
                || url.starts_with("https://learn.microsoft.com/")
        }
        "oci" => {
            url.starts_with("https://www.oracle.com/")
                || url.starts_with("https://docs.oracle.com/")
        }
        "kubernetes" => url.starts_with("https://kubernetes.io/"),
        "cncf" => {
            url.starts_with("https://github.com/cncf/") || url.starts_with("https://www.cncf.io/")
        }
        _ => false,
    }
}

fn validate_categories(
    spec: &Value,
    coverage_by_provider: &BTreeMap<String, BTreeSet<String>>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(categories) = spec.get("category_taxonomy").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "category_taxonomy_not_array",
            "category_taxonomy",
        ));
        return;
    };
    if categories.is_empty() {
        findings.insert(Finding::new("category_taxonomy_empty", "category_taxonomy"));
    }
    let category_ids: BTreeSet<String> = categories
        .iter()
        .filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect();
    for required in REQUIRED_CATEGORIES {
        if !category_ids.contains(*required) {
            findings.insert(Finding::new("missing_required_category", required));
        }
    }

    for category in categories {
        let category_id = category
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-category-id>");
        if category
            .get("target_capability")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            findings.insert(Finding::new(
                "category_missing_target_capability",
                category_id,
            ));
        }
        let examples = category.get("provider_examples").and_then(Value::as_object);
        if category_id != "cloud_native_platform_contract" {
            let example_providers: BTreeSet<String> = examples
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            for provider in PROVIDERS {
                if !example_providers.contains(*provider) {
                    findings.insert(Finding::new(
                        "category_missing_provider_example",
                        &format!("{category_id}:{provider}"),
                    ));
                }
            }
        }
        let evidence = string_set(category.get("required_evidence"));
        if evidence.is_empty() {
            findings.insert(Finding::new(
                "category_missing_required_evidence",
                category_id,
            ));
        }
        for required in REQUIRED_EVIDENCE_CLASSES {
            if !evidence.contains(*required) {
                findings.insert(Finding::new(
                    "category_missing_required_evidence_class",
                    &format!("{category_id}:{required}"),
                ));
            }
        }
        let normalized_evidence =
            normalized_claim_text(category.get("required_evidence").unwrap_or(&Value::Null));
        for marker in VAGUE_EVIDENCE_MARKERS {
            if normalized_evidence
                .split_whitespace()
                .any(|token| token == *marker)
            {
                findings.insert(Finding::new(
                    "category_vague_evidence_marker",
                    &format!("{category_id}:{marker}"),
                ));
            }
        }
        let gates = string_set(category.get("hyperscaler_gates"));
        if gates.is_empty() {
            findings.insert(Finding::new(
                "category_missing_hyperscaler_gates",
                category_id,
            ));
        }
        if !gates.iter().any(|gate| gate.starts_with("HG-")) {
            findings.insert(Finding::new(
                "category_hyperscaler_gate_id_not_hg",
                category_id,
            ));
        }
        let source_providers_for_category: BTreeSet<&str> = coverage_by_provider
            .iter()
            .filter_map(|(provider, covered)| {
                covered.contains(category_id).then_some(provider.as_str())
            })
            .collect();
        let needed: &[&str] = if category_id == "cloud_native_platform_contract" {
            &["kubernetes", "cncf"]
        } else {
            PROVIDERS
        };
        for provider in needed {
            if !source_providers_for_category.contains(provider) {
                findings.insert(Finding::new(
                    "category_missing_official_source_coverage",
                    &format!("{category_id}:{provider}"),
                ));
            }
        }
    }
}

fn validate_mapping(spec: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(mappings) = spec.get("local_oyatie_mapping").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "local_oyatie_mapping_not_array",
            "local_oyatie_mapping",
        ));
        return;
    };
    let mapping_ids: BTreeSet<String> = mappings
        .iter()
        .filter_map(|m| {
            m.get("category_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    for required in REQUIRED_CATEGORIES {
        if !mapping_ids.contains(*required) {
            findings.insert(Finding::new("mapping_missing_required_category", required));
        }
    }
    for mapping in mappings {
        let category_id = mapping
            .get("category_id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-category-id>");
        match mapping.get("claim_status").and_then(Value::as_str) {
            Some("target_spec_only" | "metadata_foundation" | "evidence_required") => {}
            _ => {
                findings.insert(Finding::new("mapping_invalid_claim_status", category_id));
            }
        }
        if mapping
            .get("honest_claim")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .is_empty()
            || mapping
                .get("cannot_claim_yet")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
        {
            findings.insert(Finding::new(
                "mapping_missing_claim_or_nonclaim_text",
                category_id,
            ));
        }
        let blocked = string_set(mapping.get("blocked_claim_families"));
        for required in REQUIRED_NONCLAIMS {
            if !blocked.contains(*required) {
                findings.insert(Finding::new(
                    "mapping_missing_blocked_claim_family",
                    &format!("{category_id}:{required}"),
                ));
            }
        }
        if contains_forbidden_claim(mapping.get("honest_claim").unwrap_or(&Value::Null)) {
            findings.insert(Finding::new(
                "mapping_honest_claim_forbidden_overclaim",
                category_id,
            ));
        }
        let cannot_claim_text =
            flattened_text(mapping.get("cannot_claim_yet").unwrap_or(&Value::Null));
        if !(cannot_claim_text.contains("feature parity")
            && cannot_claim_text.contains("production"))
        {
            findings.insert(Finding::new(
                "mapping_nonclaims_missing_parity_or_production",
                category_id,
            ));
        }
    }
}

fn validate_claim_matrix(spec: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(matrix) = spec.get("claim_matrix") else {
        findings.insert(Finding::new("claim_matrix_missing", "claim_matrix"));
        return;
    };
    for field in [
        "can_claim_now",
        "cannot_claim_yet",
        "evidence_required_before_claim",
    ] {
        if matrix
            .get(field)
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            findings.insert(Finding::new("claim_matrix_field_empty", field));
        }
    }
    if contains_forbidden_claim(matrix.get("can_claim_now").unwrap_or(&Value::Null)) {
        findings.insert(Finding::new(
            "claim_matrix_can_claim_now_forbidden_overclaim",
            "claim_matrix.can_claim_now",
        ));
    }
    let Some(requirements) = matrix
        .get("evidence_required_before_claim")
        .and_then(Value::as_array)
    else {
        return;
    };
    let mut claim_families: BTreeMap<String, String> = BTreeMap::new();
    for item in requirements {
        if let Some(family) = item.get("claim_family").and_then(Value::as_str) {
            claim_families.insert(
                family.to_owned(),
                flattened_text(item.get("requires").unwrap_or(&Value::Null)),
            );
        }
    }
    let Some(strict_ci_text) = claim_families.get("strict_dogfood_ci") else {
        findings.insert(Finding::new(
            "missing_strict_dogfood_ci_requirement",
            "claim_matrix.evidence_required_before_claim",
        ));
        return;
    };
    if !strict_ci_text.contains("no external saas fallback") {
        findings.insert(Finding::new(
            "strict_dogfood_ci_allows_external_saas_fallback",
            "strict_dogfood_ci",
        ));
    }
    if !(strict_ci_text.contains("self-hosted") && strict_ci_text.contains("g007")) {
        findings.insert(Finding::new(
            "strict_dogfood_ci_missing_self_hosted_g007",
            "strict_dogfood_ci",
        ));
    }
}

fn validate_nonclaims(spec: &Value, findings: &mut BTreeSet<Finding>) {
    let nonclaims: BTreeSet<String> = spec
        .get("nonclaims")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    for required in REQUIRED_NONCLAIMS {
        if !nonclaims.contains(*required) {
            findings.insert(Finding::new("missing_required_nonclaim", required));
        }
    }
}

fn validate_next_goal_mapping(spec: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(mapping) = spec.get("next_goal_mapping").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "next_goal_mapping_not_object",
            "next_goal_mapping",
        ));
        return;
    };
    let values: BTreeSet<String> = mapping
        .values()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    for goal in ["G002", "G003", "G004", "G005", "G006", "G007"] {
        if !values.contains(goal) {
            findings.insert(Finding::new("next_goal_mapping_missing_goal", goal));
        }
    }
    if mapping.get("dogfood_ci_claim_path").and_then(Value::as_str) != Some("G007") {
        findings.insert(Finding::new(
            "dogfood_ci_claim_path_not_g007",
            "dogfood_ci_claim_path",
        ));
    }
}

fn require_eq_str(
    spec: &Value,
    field: &str,
    expected: &str,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if spec.get(field).and_then(Value::as_str) != Some(expected) {
        findings.insert(Finding::new(code, field));
    }
}

fn require_bool_path(
    spec: &Value,
    path: &[&str],
    expected: bool,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if value_at(spec, path).and_then(Value::as_bool) != Some(expected) {
        findings.insert(Finding::new(code, &path.join(".")));
    }
}

fn require_array_superset(
    spec: &Value,
    field: &str,
    required: &[&str],
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    require_array_superset_at(spec, &[field], required, code, findings);
}

fn require_array_superset_at(
    spec: &Value,
    path: &[&str],
    required: &[&str],
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let present = string_set(value_at(spec, path));
    for item in required {
        if !present.contains(*item) {
            findings.insert(Finding::new(code, &format!("{}:{item}", path.join("."))));
        }
    }
}

fn forbid_array_overlap(
    spec: &Value,
    field: &str,
    forbidden: &[&str],
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let present = string_set(spec.get(field));
    for item in forbidden {
        if present.contains(*item) {
            findings.insert(Finding::new(code, &format!("{field}:{item}")));
        }
    }
}

fn forbid_text_markers(
    value: Option<&Value>,
    markers: &[&str],
    code: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let text = flattened_text(value.unwrap_or(&Value::Null));
    for marker in markers {
        if text.contains(marker) {
            findings.insert(Finding::new(code, &format!("{key}:{marker}")));
        }
    }
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn string_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn flattened_text(value: &Value) -> String {
    match value {
        Value::Object(map) => map
            .values()
            .map(flattened_text)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
        Value::Array(items) => items
            .iter()
            .map(flattened_text)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
        Value::String(text) => text.to_lowercase(),
        other => other.to_string().to_lowercase(),
    }
}

fn normalized_claim_text(value: &Value) -> String {
    let flat = flattened_text(value);
    let normalized: String = flat
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect();
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_forbidden_claim(value: &Value) -> bool {
    let text = format!(" {} ", normalized_claim_text(value));
    FORBIDDEN_CAN_CLAIM_PHRASES.iter().any(|phrase| {
        let phrase = format!(
            " {} ",
            normalized_claim_text(&Value::String((*phrase).to_owned()))
        );
        text.contains(&phrase)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal_green() -> Value {
        let categories = REQUIRED_CATEGORIES
            .iter()
            .map(|category| {
                json!({
                    "id": category,
                    "target_capability": "target",
                    "provider_examples": {
                        "aws": ["example"],
                        "google_cloud": ["example"],
                        "azure": ["example"],
                        "oci": ["example"]
                    },
                    "required_evidence": REQUIRED_EVIDENCE_CLASSES,
                    "hyperscaler_gates": ["HG-001"]
                })
            })
            .collect::<Vec<_>>();
        let mappings = REQUIRED_CATEGORIES
            .iter()
            .map(|category| {
                json!({
                    "category_id": category,
                    "claim_status": "target_spec_only",
                    "honest_claim": "Repo has a machine-readable target spec for this category.",
                    "cannot_claim_yet": ["feature parity", "production readiness"],
                    "blocked_claim_families": REQUIRED_NONCLAIMS
                })
            })
            .collect::<Vec<_>>();
        json!({
            "spec_id": "EXAMPLE",
            "title": "Example",
            "status": "Proposed-target",
            "retrieved_at": "2026-06-26",
            "purpose": "test",
            "official_source_evidence": [
                {"provider":"aws","url":"https://aws.amazon.com/ec2/","evidence_use":"test","category_coverage": REQUIRED_CATEGORIES},
                {"provider":"google_cloud","url":"https://cloud.google.com/compute/","evidence_use":"test","category_coverage": REQUIRED_CATEGORIES},
                {"provider":"azure","url":"https://learn.microsoft.com/azure/","evidence_use":"test","category_coverage": REQUIRED_CATEGORIES},
                {"provider":"oci","url":"https://docs.oracle.com/iaas/","evidence_use":"test","category_coverage": REQUIRED_CATEGORIES},
                {"provider":"kubernetes","url":"https://kubernetes.io/docs/home/","evidence_use":"test","category_coverage": ["cloud_native_platform_contract"]},
                {"provider":"cncf","url":"https://www.cncf.io/projects/","evidence_use":"test","category_coverage": ["cloud_native_platform_contract"]}
            ],
            "local_authority": {},
            "controls": REQUIRED_CONTROLS,
            "strict_separation_constraints": {
                "no_external_saas_ci": true,
                "no_live_provider_apply": true,
                "no_github_actions_fallback": true,
                "no_public_cloud_runtime_dependency": true,
                "allowed_evidence_lanes": ["self-hosted Kubernetes"]
            },
            "pure_dogfood_constraints": {
                "self_hosted_github_kubernetes_ci_lane": true,
                "dogfood_resource_substrate_required_before_external_provider_apply": true,
                "vfkit_linux_or_kubernetes_cluster_tests_must_be_recorded_before_kubernetes_readiness_claim": true,
                "g007_must_reconcile_historical_ci_wording": true
            },
            "evidence_vocabulary": {"required_category_evidence_classes": REQUIRED_EVIDENCE_CLASSES},
            "category_taxonomy": categories,
            "local_oyatie_mapping": mappings,
            "claim_matrix": {
                "can_claim_now": [{"claim":"Machine-readable target taxonomy exists."}],
                "cannot_claim_yet": [{"claim":"Production and parity claims."}],
                "evidence_required_before_claim": [{"claim_family":"strict_dogfood_ci","requires":["no external SaaS fallback", "self-hosted G007 evidence"]}]
            },
            "nonclaims": REQUIRED_NONCLAIMS.iter().map(|id| json!({"id": id})).collect::<Vec<_>>(),
            "next_goal_mapping": {
                "a":"G002", "b":"G003", "c":"G004", "d":"G005", "e":"G006", "dogfood_ci_claim_path":"G007"
            }
        })
    }

    #[test]
    fn minimal_fixture_is_green() {
        assert!(evaluate_keyed(&minimal_green()).is_empty());
        assert_eq!(evaluate(&minimal_green()).verdict, Verdict::Green);
    }

    #[test]
    fn rejects_unofficial_provider_source() {
        let mut spec = minimal_green();
        spec["official_source_evidence"][0]["url"] = json!("https://example.com/not-official");
        assert!(
            evaluate(&spec)
                .violations
                .contains("source_url_not_official")
        );
    }

    #[test]
    fn rejects_forbidden_claim_variants() {
        let mut spec = minimal_green();
        spec["claim_matrix"]["can_claim_now"][0]["claim"] =
            json!("Oyatie Cloud has production-readiness and feature-parity.");
        assert!(
            evaluate(&spec)
                .violations
                .contains("claim_matrix_can_claim_now_forbidden_overclaim")
        );
    }

    #[test]
    fn rejects_external_saas_evidence_lane() {
        let mut spec = minimal_green();
        spec["strict_separation_constraints"]["allowed_evidence_lanes"]
            .as_array_mut()
            .unwrap()
            .push(json!("GitHub Actions"));
        assert!(
            evaluate(&spec)
                .violations
                .contains("forbidden_allowed_evidence_lane")
        );
    }

    #[test]
    fn rejects_missing_category_and_required_evidence() {
        let mut spec = minimal_green();
        spec["category_taxonomy"].as_array_mut().unwrap().pop();
        assert!(
            evaluate(&spec)
                .violations
                .contains("missing_required_category")
        );

        let mut spec = minimal_green();
        spec["category_taxonomy"][0]["required_evidence"] = json!(["TODO"]);
        let violations = evaluate(&spec).violations;
        assert!(violations.contains("category_missing_required_evidence_class"));
        assert!(violations.contains("category_vague_evidence_marker"));
    }
}
