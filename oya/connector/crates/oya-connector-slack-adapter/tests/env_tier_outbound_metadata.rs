use oya_connector_slack_adapter::{
    ConnectorOutboundMetadataError, ConnectorOutboundMode, ConnectorOutboundPlanFixture,
    validate_connector_outbound_plan_fixture,
};

fn base_fixture() -> ConnectorOutboundPlanFixture {
    ConnectorOutboundPlanFixture {
        tenant_id: Some("tenant-alpha".to_owned()),
        env_tier: Some("test".to_owned()),
        caller_supplied_outbound_mode: None,
        attempts_external_delivery: false,
        connector_adapter_class: Some("slack.enterprise_adapter".to_owned()),
        destination_binding_ref: Some("connector-binding://tenant-alpha/slack/sandbox".to_owned()),
        destination_tenant_id: Some("tenant-alpha".to_owned()),
        qa_endpoint_ref: None,
        retirement_no_new_runtime_scope_evidence_ref: Some(
            "policy://oya/connector/no-new-runtime-scope#adapter-class-slack".to_owned(),
        ),
        policy_evidence_ref: Some(
            "cedar://oya/connector/connector-authorization#tenant-alpha".to_owned(),
        ),
        connector_authorization_evidence_ref: None,
        secret_material_probe: None,
    }
}

#[test]
fn valid_test_plan_derives_intercept_and_carries_required_metadata() {
    let plan = validate_connector_outbound_plan_fixture(base_fixture()).unwrap();

    assert_eq!(plan.tenant_id, "tenant-alpha");
    assert_eq!(plan.outbound_mode, ConnectorOutboundMode::Intercept);
    assert_eq!(plan.connector_adapter_class, "slack.enterprise_adapter");
    assert_eq!(
        plan.destination_binding_ref,
        "connector-binding://tenant-alpha/slack/sandbox"
    );
    assert_eq!(
        plan.retirement_no_new_runtime_scope_evidence_ref,
        "policy://oya/connector/no-new-runtime-scope#adapter-class-slack"
    );
    assert_eq!(
        plan.policy_evidence_ref,
        "cedar://oya/connector/connector-authorization#tenant-alpha"
    );
}

#[test]
fn valid_staging_plan_derives_test_recipients_only_with_qa_endpoint() {
    let mut fixture = base_fixture();
    fixture.env_tier = Some("staging".to_owned());
    fixture.qa_endpoint_ref = Some("qa-endpoint://tenant-alpha/slack/webhook-sandbox".to_owned());

    let plan = validate_connector_outbound_plan_fixture(fixture).unwrap();

    assert_eq!(plan.outbound_mode, ConnectorOutboundMode::TestRecipients);
}

#[test]
fn valid_prod_plan_derives_live_only_with_policy_and_connector_authorization() {
    let mut fixture = base_fixture();
    fixture.env_tier = Some("prod".to_owned());
    fixture.connector_authorization_evidence_ref =
        Some("cedar://oya/connector/connector-authorization#tenant-alpha-prod".to_owned());

    let plan = validate_connector_outbound_plan_fixture(fixture).unwrap();

    assert_eq!(plan.outbound_mode, ConnectorOutboundMode::Live);
    assert_eq!(
        plan.connector_authorization_evidence_ref.as_deref(),
        Some("cedar://oya/connector/connector-authorization#tenant-alpha-prod")
    );
}

#[test]
fn missing_env_tier_denied() {
    let mut fixture = base_fixture();
    fixture.env_tier = None;

    assert_eq!(
        validate_connector_outbound_plan_fixture(fixture),
        Err(ConnectorOutboundMetadataError::MissingEnvTier)
    );
}

#[test]
fn caller_supplied_outbound_mode_mismatch_denied() {
    let mut fixture = base_fixture();
    fixture.caller_supplied_outbound_mode = Some("live".to_owned());

    assert_eq!(
        validate_connector_outbound_plan_fixture(fixture),
        Err(ConnectorOutboundMetadataError::OutboundModeMismatch)
    );
}

#[test]
fn test_tier_attempting_external_connector_or_webhook_delivery_denied() {
    let mut fixture = base_fixture();
    fixture.attempts_external_delivery = true;

    assert_eq!(
        validate_connector_outbound_plan_fixture(fixture),
        Err(ConnectorOutboundMetadataError::TestTierExternalDelivery)
    );
}

#[test]
fn staging_without_qa_endpoint_denied() {
    let mut fixture = base_fixture();
    fixture.env_tier = Some("staging".to_owned());

    assert_eq!(
        validate_connector_outbound_plan_fixture(fixture),
        Err(ConnectorOutboundMetadataError::StagingMissingQaEndpoint)
    );
}

#[test]
fn prod_without_tenancy_policy_or_connector_authorization_denied() {
    let mut missing_policy = base_fixture();
    missing_policy.env_tier = Some("prod".to_owned());
    missing_policy.policy_evidence_ref = None;
    missing_policy.connector_authorization_evidence_ref =
        Some("cedar://oya/connector/connector-authorization#tenant-alpha-prod".to_owned());

    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_policy),
        Err(ConnectorOutboundMetadataError::MissingPolicyEvidenceRef)
    );

    let mut missing_authorization = base_fixture();
    missing_authorization.env_tier = Some("prod".to_owned());

    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_authorization),
        Err(ConnectorOutboundMetadataError::ProdMissingConnectorAuthorization)
    );
}

#[test]
fn cross_tenant_connector_endpoint_leakage_denied() {
    let mut fixture = base_fixture();
    fixture.destination_tenant_id = Some("tenant-beta".to_owned());

    assert_eq!(
        validate_connector_outbound_plan_fixture(fixture),
        Err(ConnectorOutboundMetadataError::CrossTenantDestinationLeakage)
    );
}

#[test]
fn raw_oauth_token_webhook_secret_or_api_key_denied() {
    for raw_probe in [
        "oauth_token=raw-oauth-token",
        "webhook_secret=raw-webhook-secret",
        "sk_test_raw_fixture_value",
        "sk_stage_raw_fixture_value",
        "sk_live_raw_fixture_value",
    ] {
        let mut fixture = base_fixture();
        fixture.secret_material_probe = Some(raw_probe.to_owned());

        assert_eq!(
            validate_connector_outbound_plan_fixture(fixture),
            Err(ConnectorOutboundMetadataError::RawSecretMaterial),
            "{raw_probe} must fail closed"
        );
    }
}

#[test]
fn required_connector_metadata_fields_denied_when_missing() {
    let mut missing_tenant = base_fixture();
    missing_tenant.tenant_id = None;
    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_tenant),
        Err(ConnectorOutboundMetadataError::MissingTenantId)
    );

    let mut missing_adapter = base_fixture();
    missing_adapter.connector_adapter_class = None;
    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_adapter),
        Err(ConnectorOutboundMetadataError::MissingConnectorAdapterClass)
    );

    let mut missing_destination = base_fixture();
    missing_destination.destination_binding_ref = None;
    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_destination),
        Err(ConnectorOutboundMetadataError::MissingDestinationBindingRef)
    );

    let mut missing_retirement = base_fixture();
    missing_retirement.retirement_no_new_runtime_scope_evidence_ref = None;
    assert_eq!(
        validate_connector_outbound_plan_fixture(missing_retirement),
        Err(ConnectorOutboundMetadataError::MissingRetirementNoNewRuntimeScopeEvidence)
    );
}
