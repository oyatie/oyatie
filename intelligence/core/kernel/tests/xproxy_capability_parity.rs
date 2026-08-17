use std::collections::BTreeSet;

use intelligence_kernel::xproxy_parity::{
    CapabilityParityMap, CapabilityStatus, EXTERNAL_PROXY_REFERENCE_BASELINE_JSON,
    EXTERNAL_PROXY_REFERENCE_DRAFT_TARGETS_JSON, ReferenceDraftParityTargets,
    render_capability_parity_report,
};
use serde_json::Value;

const PARITY_MAP_JSON: &str = EXTERNAL_PROXY_REFERENCE_BASELINE_JSON;
const DRAFT_TARGETS_JSON: &str = EXTERNAL_PROXY_REFERENCE_DRAFT_TARGETS_JSON;

const EXPECTED_XPROXY_IDS: &[&str] = &[
    "XPROXY-API-001",
    "XPROXY-API-002",
    "XPROXY-API-003",
    "XPROXY-API-004",
    "XPROXY-API-005",
    "XPROXY-API-006",
    "XPROXY-ROUTE-001",
    "XPROXY-ROUTE-002",
    "XPROXY-ROUTE-003",
    "XPROXY-ROUTE-004",
    "XPROXY-ROUTE-005",
    "XPROXY-ROUTE-006",
    "XPROXY-ROUTE-007",
    "XPROXY-WIRE-001",
    "XPROXY-WIRE-002",
    "XPROXY-WIRE-003",
    "XPROXY-WIRE-004",
    "XPROXY-WIRE-005",
    "XPROXY-WIRE-006",
    "XPROXY-WIRE-007",
    "XPROXY-WIRE-008",
    "XPROXY-WIRE-009",
    "XPROXY-WIRE-010",
    "XPROXY-AUTH-001",
    "XPROXY-AUTH-002",
    "XPROXY-AUTH-003",
    "XPROXY-AUTH-004",
    "XPROXY-AUTH-005",
    "XPROXY-AUTH-006",
    "XPROXY-AUTH-007",
    "XPROXY-AUTH-008",
    "XPROXY-COMPAT-001",
    "XPROXY-COMPAT-002",
    "XPROXY-COMPAT-003",
    "XPROXY-COMPAT-004",
    "XPROXY-COMPAT-005",
    "XPROXY-COMPAT-006",
    "XPROXY-OBS-001",
    "XPROXY-OBS-002",
    "XPROXY-OBS-003",
    "XPROXY-OBS-004",
    "XPROXY-OBS-005",
    "XPROXY-OBS-006",
    "XPROXY-SEC-001",
    "XPROXY-SEC-002",
    "XPROXY-SEC-003",
    "XPROXY-SEC-004",
    "XPROXY-DRIFT-001",
    "XPROXY-DRIFT-002",
    "XPROXY-DRIFT-003",
];

fn load_map() -> CapabilityParityMap {
    serde_json::from_str(PARITY_MAP_JSON).expect("parity map must parse")
}

fn load_draft_targets() -> ReferenceDraftParityTargets {
    serde_json::from_str(DRAFT_TARGETS_JSON).expect("reference draft targets must parse")
}

#[test]
fn parity_map_covers_exactly_all_50_xproxy_capabilities() {
    let map = load_map();
    let actual: BTreeSet<_> = map.capabilities.iter().map(|row| row.id.as_str()).collect();
    let expected: BTreeSet<_> = EXPECTED_XPROXY_IDS.iter().copied().collect();

    assert_eq!(map.artifact_family, "external-proxy-reference");
    assert_eq!(map.capability_namespace, "XPROXY");
    assert_eq!(actual.len(), 50);
    assert_eq!(actual, expected);
}

#[test]
fn parity_rows_have_allowed_statuses_and_linked_target_tests() {
    let map = load_map();
    let allowed = BTreeSet::from([
        CapabilityStatus::Planned,
        CapabilityStatus::Implemented,
        CapabilityStatus::Superseded,
        CapabilityStatus::ApprovedOutOfScope,
    ]);

    for row in &map.capabilities {
        assert!(
            allowed.contains(&row.status),
            "{} has invalid status",
            row.id
        );
        assert!(
            !row.target_tests.is_empty(),
            "{} must link at least one target test",
            row.id
        );
        assert!(
            row.target_tests.iter().all(|test| test.contains("xproxy")),
            "{} target tests must be traceable to XPROXY parity",
            row.id
        );
    }
}

fn source_name_terms(value: &Value) -> Vec<String> {
    let provenance = value
        .get("provenance")
        .and_then(Value::as_object)
        .expect("provenance object");
    let mut terms = BTreeSet::new();

    if let Some(repo) = provenance.get("source_repo").and_then(Value::as_str) {
        let path_segments: Vec<_> = repo
            .split('/')
            .filter(|part| !part.is_empty() && !part.contains(':'))
            .collect();
        for part in path_segments.iter().rev().take(2) {
            terms.insert(part.to_lowercase());
        }
    }

    if let Some(package_name) = provenance.get("package_name").and_then(Value::as_str) {
        for part in package_name
            .trim_start_matches('@')
            .split('/')
            .filter(|part| part.len() > 3)
        {
            terms.insert(part.to_lowercase());
        }
    }

    terms.into_iter().collect()
}

#[test]
fn parity_map_keeps_external_project_name_only_in_provenance_fields() {
    let value: Value = serde_json::from_str(PARITY_MAP_JSON).expect("json");
    let forbidden = source_name_terms(&value);
    let allowed_suffixes = BTreeSet::from([
        "/provenance/source_repo",
        "/provenance/package_name",
        "/provenance/commit_sha",
        "/provenance/pinned_tree_url",
    ]);

    fn scan(
        value: &Value,
        path: String,
        forbidden: &[String],
        allowed_suffixes: &BTreeSet<&str>,
        violations: &mut Vec<String>,
    ) {
        match value {
            Value::String(s) => {
                let lower = s.to_lowercase();
                if forbidden.iter().any(|term| lower.contains(term))
                    && !allowed_suffixes.contains(path.as_str())
                {
                    violations.push(format!("{path}: {s}"));
                }
            }
            Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    scan(
                        item,
                        format!("{path}/{idx}"),
                        forbidden,
                        allowed_suffixes,
                        violations,
                    );
                }
            }
            Value::Object(object) => {
                for (key, child) in object {
                    scan(
                        child,
                        format!("{path}/{key}"),
                        forbidden,
                        allowed_suffixes,
                        violations,
                    );
                }
            }
            _ => {}
        }
    }

    let mut violations = Vec::new();
    scan(
        &value,
        String::new(),
        &forbidden,
        &allowed_suffixes,
        &mut violations,
    );
    assert!(
        violations.is_empty(),
        "external name leaked outside provenance: {violations:#?}"
    );
}

#[test]
fn parity_targets_are_cloud_native_and_do_not_create_cli_or_tui_workflows() {
    let map = load_map();
    let raw = PARITY_MAP_JSON.to_lowercase();
    let forbidden_target_phrases = [
        " implement cli",
        " add cli",
        "local tui",
        "terminal ui",
        "command surface target",
    ];
    for phrase in forbidden_target_phrases {
        assert!(
            !raw.contains(phrase),
            "parity map must not introduce CLI/TUI target phrase: {phrase}"
        );
    }

    assert!(map.architecture.cloud_native_only);
    assert!(map.architecture.no_cli_tui_targets);
    assert_eq!(
        map.architecture.secret_resolution_boundary,
        "owned-secret-provider-port"
    );
    assert_eq!(
        map.architecture.authorization_boundary,
        "owned-policy-engine-port"
    );

    for row in &map.capabilities {
        assert!(
            row.target_boundary.contains("gateway")
                || row.target_boundary.contains("worker")
                || row.target_boundary.contains("controller")
                || row.target_boundary.contains("contract")
                || row.target_boundary.contains("ops")
                || row.target_boundary.contains("kubernetes")
                || row.target_boundary.contains("policy-engine")
                || row.target_boundary.contains("secret-provider"),
            "{} target boundary must be cloud-native, got {}",
            row.id,
            row.target_boundary
        );
    }
}

#[test]
fn parity_report_prints_provenance_all_ids_statuses_and_tests() {
    let map = load_map();
    let report = render_capability_parity_report(&map);

    assert!(report.contains(&format!("source_repo={}", map.provenance.source_repo)));
    assert!(report.contains(&format!(
        "package={}@{}",
        map.provenance.package_name, map.provenance.package_version
    )));
    assert!(report.contains(&format!("commit={}", map.provenance.commit_sha)));

    for id in EXPECTED_XPROXY_IDS {
        assert!(report.contains(id), "report missing {id}");
    }
    for status in [
        "planned",
        "implemented",
        "superseded",
        "approved_out_of_scope",
    ] {
        assert!(
            report.contains(status),
            "report missing status label {status}"
        );
    }
    assert!(report.contains("target_tests="));
}

#[test]
fn reference_draft_targets_map_source_ideas_to_existing_xproxy_rows() {
    let targets = load_draft_targets();
    let expected_ids: BTreeSet<_> = EXPECTED_XPROXY_IDS.iter().copied().collect();

    assert_eq!(targets.artifact_family, "external-proxy-reference");
    assert_eq!(targets.capability_namespace, "XPROXY");
    assert!(targets.scope.translations_owned_by_provider_adapters);
    assert!(!targets.scope.routing_advisors_may_generate);
    assert_eq!(
        targets.scope.generation_providers,
        ["openai-codex", "anthropic-claude", "google-gemini"]
    );
    assert!(
        targets
            .scope
            .routing_advisor_models
            .iter()
            .any(|model| model == "nemotron-3-ultra-550b-a55b")
    );

    let target_ids: BTreeSet<_> = targets
        .targets
        .iter()
        .map(|target| target.capability_id.as_str())
        .collect();
    assert!(
        target_ids.len() >= 8,
        "expected multiple mapped target rows"
    );
    for capability_id in target_ids {
        assert!(
            expected_ids.contains(capability_id),
            "draft target used unknown capability id {capability_id}"
        );
    }

    for target in &targets.targets {
        assert!(!target.extracted_feature_groups.is_empty());
        assert!(
            target
                .target_tests
                .iter()
                .all(|test| test.contains("xproxy")),
            "{} target tests must remain tied to XPROXY parity",
            target.capability_id
        );
    }
}

#[test]
fn reference_draft_targets_keep_source_repo_names_only_in_provenance_urls() {
    let value: Value = serde_json::from_str(DRAFT_TARGETS_JSON).expect("json");
    let forbidden = [
        "llm-router",
        "nemoclaw",
        "claw-code",
        "nvidia-ai-blueprints",
        "nousresearch",
        "ultraworkers",
    ];

    fn scan(value: &Value, path: String, forbidden: &[&str], violations: &mut Vec<String>) {
        match value {
            Value::String(s) => {
                let lower = s.to_lowercase();
                let allowed_source_url =
                    path.starts_with("/source_provenance/") && path.ends_with("/source_url");
                if forbidden.iter().any(|term| lower.contains(term)) && !allowed_source_url {
                    violations.push(format!("{path}: {s}"));
                }
            }
            Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    scan(item, format!("{path}/{idx}"), forbidden, violations);
                }
            }
            Value::Object(object) => {
                for (key, child) in object {
                    scan(child, format!("{path}/{key}"), forbidden, violations);
                }
            }
            _ => {}
        }
    }

    let mut violations = Vec::new();
    scan(&value, String::new(), &forbidden, &mut violations);
    assert!(
        violations.is_empty(),
        "reference source name leaked outside source provenance URL fields: {violations:#?}"
    );
}
