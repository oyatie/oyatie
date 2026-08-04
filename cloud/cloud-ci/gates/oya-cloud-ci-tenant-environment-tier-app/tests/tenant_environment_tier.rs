#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_ci_tenant_environment_tier_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::json;

#[test]
fn green_test_staging_and_prod_rows_are_isolated() {
    let input = json!({"rows": [
        {
            "fixture_id": "test-tier",
            "tenant_id": "ten_test_alpha",
            "tier": "test",
            "api_key_prefix": "sk_test_",
            "expected_workload_pool": "pool-test",
            "observed_workload_pool": "pool-test",
            "expected_schema_or_database": "tenant_test",
            "observed_schema_or_database": "tenant_test",
            "outbound_mode_expected": "intercept",
            "outbound_mode_observed": "intercept",
            "cedar_key_issuance_role": "developer",
            "cedar_key_issuance_allowed": true,
            "prod_destructive_ack_required": false,
            "prod_destructive_ack_observed": false,
            "audit_chain_tag_present": true,
            "workflow_default_new_flow_tier": "test",
            "model_or_budget_tier_hook_present": true
        },
        {
            "fixture_id": "prod-tier",
            "tenant_id": "ten_prod_alpha",
            "tier": "prod",
            "api_key_prefix": "sk_live_",
            "observed_workload_pool": "pool-prod",
            "observed_schema_or_database": "tenant_prod",
            "outbound_mode_expected": "live",
            "outbound_mode_observed": "live",
            "cedar_key_issuance_role": "admin",
            "cedar_key_issuance_allowed": true,
            "prod_destructive_ack_required": true,
            "prod_destructive_ack_observed": true,
            "audit_chain_tag_present": true,
            "workflow_default_new_flow_tier": "test",
            "model_or_budget_tier_hook_present": true
        }
    ]});

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn test_key_must_not_route_to_prod() {
    let input = json!({"rows": [{
        "fixture_id": "test-key-prod-route",
        "tenant_id": "ten_test_alpha",
        "tier": "test",
        "api_key_prefix": "sk_test_",
        "observed_workload_pool": "pool-prod",
        "observed_schema_or_database": "tenant_prod",
        "outbound_mode_expected": "intercept",
        "outbound_mode_observed": "intercept",
        "audit_chain_tag_present": true
    }]});

    assert!(
        evaluate(&input)
            .violations
            .contains("test_key_routes_to_prod")
    );
}

#[test]
fn outbound_prod_ack_cedar_audit_workflow_and_budget_hooks_are_checked() {
    let input = json!({"rows": [{
        "fixture_id": "bad-prod-policy",
        "tenant_id": "ten_prod_alpha",
        "tier": "prod",
        "api_key_prefix": "sk_live_",
        "observed_workload_pool": "pool-prod",
        "observed_schema_or_database": "tenant_prod",
        "outbound_mode_expected": "live",
        "outbound_mode_observed": "intercept",
        "cedar_key_issuance_role": "developer",
        "cedar_key_issuance_allowed": true,
        "prod_destructive_ack_required": true,
        "prod_destructive_ack_observed": false,
        "audit_chain_tag_present": false,
        "workflow_default_new_flow_tier": "prod",
        "model_or_budget_tier_hook_present": false
    }]});

    let violations = evaluate(&input).violations;
    for code in [
        "outbound_mode_unenforced",
        "prod_destructive_ack_missing",
        "cedar_key_grant_missing",
        "audit_chain_env_tier_missing",
        "workflow_default_tier_not_test",
        "tier_model_budget_hook_missing",
    ] {
        assert!(violations.contains(code), "missing {code}: {violations:?}");
    }
}

#[test]
fn prefix_must_map_to_tier() {
    let input = json!({"rows": [{
        "fixture_id": "stage-prefix-mismatch",
        "tier": "staging",
        "api_key_prefix": "sk_live_",
        "outbound_mode_expected": "test_recipients",
        "outbound_mode_observed": "test_recipients",
        "audit_chain_tag_present": true
    }]});

    assert!(
        evaluate(&input)
            .violations
            .contains("api_key_prefix_unmapped")
    );
}

#[test]
fn missing_rows_fail_closed() {
    assert!(
        evaluate(&json!({"rows": []}))
            .violations
            .contains("env_tier_fixture_missing")
    );
}
