// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_capability_domain::{AutonomyTier, Capability, CapabilityMcpContract};
use oya_data_boundary_kernel::{DataClass, privacy_data_classes_from};
use oya_intelligence_mcp_gateway_domain::{
    DISCOVER_SCOPE, MCP_PROTOCOL_VERSION, McpAccessTokenClaims, McpAuthorizationChallenge,
    McpGatewayDescriptor, McpGatewayError, McpPrincipal, McpRateLimitPolicy, McpRateLimiter,
    McpTenantEndpoint, authorize_tool_call, validate_access_token,
};

#[test]
fn tenant_endpoint_and_descriptor_project_mcp_tools_and_prompts() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-home".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();
    let principal = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T2Advisory,
        vec![
            DISCOVER_SCOPE.into(),
            "foundry.capability.invoke:cap.demo.readiness".into(),
        ],
    )
    .unwrap();
    let descriptor = McpGatewayDescriptor::new(
        endpoint.clone(),
        &principal,
        &[capability_with_data_classes(
            "cap.demo.readiness",
            AutonomyTier::T1ViewOnly,
            &[DataClass::InternalOnly, DataClass::PiiIdentifying],
        )],
    )
    .unwrap();

    assert_eq!(MCP_PROTOCOL_VERSION, "2025-11-25");
    assert_eq!(
        endpoint.url(),
        "https://mcp.foundry.region-home.oyatie.test/tenants/ten_alpha"
    );
    assert_eq!(
        endpoint.protected_resource_metadata_uri(),
        "https://mcp.foundry.region-home.oyatie.test/.well-known/oauth-protected-resource/tenants/ten_alpha"
    );
    assert_eq!(descriptor.tools.len(), 1);
    assert_eq!(descriptor.tools[0].name.value, "cap.demo.readiness");
    assert_eq!(
        descriptor.tools[0].required_scope.value,
        "foundry.capability.invoke:cap.demo.readiness"
    );
    assert!(
        descriptor.tools[0]
            .input_schema
            .value
            .contains("additionalProperties")
    );
    assert_eq!(
        descriptor.tools[0].legacy_data_classes(),
        vec![DataClass::InternalOnly, DataClass::PiiIdentifying]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            descriptor.tools[0].data_classes(),
            descriptor.tools[0].legacy_data_classes()
        );
    }
    assert_eq!(
        descriptor.tools[0].privacy_data_classes(),
        privacy_data_classes_from(&[DataClass::InternalOnly, DataClass::PiiIdentifying])
            .unwrap()
            .as_slice()
    );
    assert!(
        descriptor
            .prompts
            .iter()
            .any(|prompt| prompt.name.value == "capability-publish")
    );
}

#[test]
fn gateway_projects_capability_authored_agent_description_and_schemas() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-home".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();
    let principal = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T2Advisory,
        vec![DISCOVER_SCOPE.into()],
    )
    .unwrap();
    let capability = Capability::new_with_mcp_contract(
        "cap.demo.authored".into(),
        "demo".into(),
        AutonomyTier::T1ViewOnly,
        privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
        "oya.foundry.capability.invoked".into(),
        CapabilityMcpContract::new(
            "Use this only for authored readiness evidence.".into(),
            "Operator-facing readiness evidence tool.".into(),
            r#"{"type":"object","required":["release_id"]}"#.into(),
            r#"{"type":"object","required":["verdict"]}"#.into(),
        )
        .unwrap(),
    )
    .unwrap();

    let descriptor = McpGatewayDescriptor::new(endpoint, &principal, &[capability]).unwrap();
    let tool = &descriptor.tools[0];
    assert_eq!(
        tool.description.value,
        "Use this only for authored readiness evidence."
    );
    assert!(tool.input_schema.value.contains("release_id"));
    assert!(
        tool.output_schema
            .as_ref()
            .expect("output schema is projected")
            .value
            .contains("verdict")
    );
}

#[test]
fn gateway_refuses_cross_tenant_discovery_and_missing_scopes() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-recovery".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();
    let cross_tenant = McpPrincipal::new(
        "ten_beta".into(),
        "usr_operator".into(),
        AutonomyTier::T4AutoExecute,
        vec![DISCOVER_SCOPE.into()],
    )
    .unwrap();
    assert_eq!(
        McpGatewayDescriptor::new(endpoint.clone(), &cross_tenant, &[]),
        Err(McpGatewayError::TenantMismatch)
    );

    let missing_scope = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T4AutoExecute,
        vec![],
    )
    .unwrap();
    assert_eq!(
        McpGatewayDescriptor::new(endpoint, &missing_scope, &[]),
        Err(McpGatewayError::MissingScope)
    );
}

#[test]
fn tool_call_authorization_requires_scope_and_autonomy_ceiling() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-home".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();
    let descriptor = McpGatewayDescriptor::new(
        endpoint.clone(),
        &McpPrincipal::new(
            "ten_alpha".into(),
            "usr_operator".into(),
            AutonomyTier::T4AutoExecute,
            vec![DISCOVER_SCOPE.into()],
        )
        .unwrap(),
        &[capability(
            "cap.demo.execute",
            AutonomyTier::T3ExecuteWithApproval,
        )],
    )
    .unwrap();
    let tool = &descriptor.tools[0];

    let missing_scope = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T4AutoExecute,
        vec![DISCOVER_SCOPE.into()],
    )
    .unwrap();
    assert_eq!(
        authorize_tool_call(&endpoint, &missing_scope, tool),
        Err(McpGatewayError::MissingScope)
    );

    let too_low = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T2Advisory,
        vec![tool.required_scope.value.clone()],
    )
    .unwrap();
    assert_eq!(
        authorize_tool_call(&endpoint, &too_low, tool),
        Err(McpGatewayError::AutonomyCeilingExceeded)
    );

    let allowed = McpPrincipal::new(
        "ten_alpha".into(),
        "usr_operator".into(),
        AutonomyTier::T3ExecuteWithApproval,
        vec![tool.required_scope.value.clone()],
    )
    .unwrap();
    assert_eq!(authorize_tool_call(&endpoint, &allowed, tool), Ok(()));
}

#[test]
fn authorization_challenges_follow_mcp_oauth_resource_metadata_shape() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-home".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();

    let challenge =
        McpAuthorizationChallenge::missing_token(&endpoint, vec![DISCOVER_SCOPE.into()]);
    assert_eq!(challenge.status_code, 401);
    assert_eq!(
        challenge.required_scopes.value,
        vec![DISCOVER_SCOPE.to_string()]
    );
    assert_eq!(
        challenge.www_authenticate_header(),
        r#"Bearer resource_metadata="https://mcp.foundry.region-home.oyatie.test/.well-known/oauth-protected-resource/tenants/ten_alpha", scope="foundry.capability.discover""#
    );

    let insufficient = McpAuthorizationChallenge::insufficient_scope(
        &endpoint,
        vec!["foundry.capability.invoke:cap.demo.readiness".into()],
    );
    assert_eq!(insufficient.status_code, 403);
    assert!(
        insufficient
            .www_authenticate_header()
            .contains(r#"error="insufficient_scope""#)
    );
}

#[test]
fn access_token_claims_are_bound_to_endpoint_resource_issuer_and_expiry() {
    let endpoint = McpTenantEndpoint::new(
        "ten_alpha".into(),
        "region-home".into(),
        "test".into(),
        "https://auth.oyatie.test/tenants/ten_alpha".into(),
    )
    .unwrap();
    let claims = token_claims(&endpoint, 1_000);

    let principal =
        validate_access_token(&endpoint, claims.clone(), 999, AutonomyTier::T2Advisory).unwrap();
    assert_eq!(principal.tenant_id.value, "ten_alpha");
    assert_eq!(principal.subject_id.value, "usr_operator");
    assert!(principal.has_scope(DISCOVER_SCOPE));
    assert_eq!(principal.autonomy_ceiling, AutonomyTier::T2Advisory);

    let mut wrong_audience = claims.clone();
    wrong_audience.audience =
        "https://mcp.foundry.region-recovery.oyatie.test/tenants/ten_alpha".into();
    assert_eq!(
        validate_access_token(&endpoint, wrong_audience, 999, AutonomyTier::T2Advisory),
        Err(McpGatewayError::TokenAudienceMismatch)
    );

    let mut wrong_issuer = claims.clone();
    wrong_issuer.issuer = "https://auth.other.test/tenants/ten_alpha".into();
    assert_eq!(
        validate_access_token(&endpoint, wrong_issuer, 999, AutonomyTier::T2Advisory),
        Err(McpGatewayError::TokenIssuerMismatch)
    );

    assert_eq!(
        validate_access_token(&endpoint, claims, 1_000, AutonomyTier::T2Advisory),
        Err(McpGatewayError::TokenExpired)
    );
}

#[test]
fn rate_limiter_isolates_tenant_tool_windows_and_resets_after_window() {
    let mut limiter = McpRateLimiter::new(McpRateLimitPolicy::new(2, 60).unwrap());

    assert_eq!(
        limiter.check_and_record("ten_alpha", "cap.demo.invoke", 1),
        Ok(())
    );
    assert_eq!(
        limiter.check_and_record("ten_alpha", "cap.demo.invoke", 2),
        Ok(())
    );
    assert_eq!(
        limiter.check_and_record("ten_alpha", "cap.demo.invoke", 3),
        Err(McpGatewayError::RateLimitExceeded)
    );

    assert_eq!(
        limiter.check_and_record("ten_alpha", "cap.demo.other", 3),
        Ok(())
    );
    assert_eq!(
        limiter.check_and_record("ten_beta", "cap.demo.invoke", 3),
        Ok(())
    );
    assert_eq!(
        limiter.check_and_record("ten_alpha", "cap.demo.invoke", 61),
        Ok(())
    );
}

fn capability(id: &str, tier: AutonomyTier) -> Capability {
    capability_with_data_classes(id, tier, &[DataClass::InternalOnly])
}

fn capability_with_data_classes(
    id: &str,
    tier: AutonomyTier,
    data_classes: &[DataClass],
) -> Capability {
    Capability::new(
        id.into(),
        "demo".into(),
        tier,
        privacy_data_classes_from(data_classes).expect("test fixture uses privacy data classes"),
        "oya.foundry.capability.invoked".into(),
    )
    .unwrap()
}

fn token_claims(
    endpoint: &McpTenantEndpoint,
    expires_at_epoch_seconds: u64,
) -> McpAccessTokenClaims {
    McpAccessTokenClaims {
        tenant_id: endpoint.tenant_id.value.clone(),
        subject_id: "usr_operator".into(),
        issuer: endpoint.authorization_server.value.clone(),
        audience: endpoint.url(),
        expires_at_epoch_seconds,
        scopes: vec![
            DISCOVER_SCOPE.into(),
            "foundry.capability.invoke:cap.demo.readiness".into(),
        ],
    }
}
