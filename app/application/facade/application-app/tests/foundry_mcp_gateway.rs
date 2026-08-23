// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_app::{
    AutonomyTier, CapabilityAction, CapabilityMcpContract, CapabilityRegistration,
    CostBudgetRegistration, DISCOVER_SCOPE, DataClass, Foundation, FoundationError,
    IdentityRegistration, McpAccessTokenClaims, McpDiscoveryRequest, McpRateLimitPolicy,
    McpToolCallRequest, Purpose, SubjectClass, TenantCapabilityGrant, TenantRegistration,
    scope_for_tool_name,
};

#[test]
fn foundation_projects_per_tenant_mcp_descriptor_without_cross_tenant_leakage() {
    let mut foundation = Foundation::default();
    onboard_tenant(&mut foundation, "ten_mcp", AutonomyTier::T2Advisory);
    onboard_tenant(&mut foundation, "ten_other", AutonomyTier::T4AutoExecute);

    register_internal_capability(
        &mut foundation,
        "cap.demo.visible",
        AutonomyTier::T1ViewOnly,
        true,
    );
    register_internal_capability(
        &mut foundation,
        "cap.demo.too-high",
        AutonomyTier::T3ExecuteWithApproval,
        true,
    );
    register_internal_capability(
        &mut foundation,
        "cap.demo.hidden",
        AutonomyTier::T1ViewOnly,
        false,
    );

    let descriptor = foundation
        .discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_mcp".into(),
            access_token: access_token("ten_mcp", vec![DISCOVER_SCOPE.into()], 100),
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        })
        .expect("tenant-scoped MCP discovery succeeds");

    assert_eq!(
        descriptor.endpoint.url(),
        "https://mcp.foundry.region-home.oyatie.test/tenants/ten_mcp"
    );
    assert_eq!(
        descriptor
            .tools
            .iter()
            .map(|tool| tool.name.value.as_str())
            .collect::<Vec<_>>(),
        vec!["cap.demo.visible"]
    );

    assert_eq!(
        foundation.discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_mcp".into(),
            access_token: access_token("ten_other", vec![DISCOVER_SCOPE.into()], 100),
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        }),
        Err(FoundationError::McpAccessDenied)
    );

    assert_eq!(
        foundation.discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_mcp".into(),
            access_token: access_token("ten_mcp", vec![], 100),
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        }),
        Err(FoundationError::McpAccessDenied)
    );

    let mut wrong_audience = access_token("ten_mcp", vec![DISCOVER_SCOPE.into()], 100);
    wrong_audience.audience =
        "https://mcp.foundry.region-recovery.oyatie.test/tenants/ten_mcp".into();
    assert_eq!(
        foundation.discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_mcp".into(),
            access_token: wrong_audience,
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        }),
        Err(FoundationError::McpAccessDenied)
    );
}

#[test]
fn foundation_registration_projects_authored_mcp_contract_to_tenant_descriptor() {
    let mut foundation = Foundation::default();
    onboard_tenant(&mut foundation, "ten_mcp", AutonomyTier::T2Advisory);
    support::seed_passing_eval(&mut foundation, "cap.demo.authored-foundation");

    let contract = CapabilityMcpContract::new(
        "Agent-authored release readiness check.".into(),
        "Human release readiness guide.".into(),
        r#"{"type":"object","required":["release_id"]}"#.into(),
        r#"{"type":"object","required":["verdict"]}"#.into(),
    )
    .unwrap();
    foundation
        .register_capability_with_mcp_contract(
            CapabilityRegistration {
                capability_id: "cap.demo.authored-foundation".into(),
                namespace: "demo".into(),
                action: CapabilityAction::Other,
                required_tier: AutonomyTier::T1ViewOnly,
                touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                    DataClass::InternalOnly,
                ])
                .unwrap(),
                evidence_topic: "oya.foundry.capability.invoked".into(),
            },
            contract,
        )
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_mcp".into(),
            capability_id: "cap.demo.authored-foundation".into(),
            mcp_visible: true,
        })
        .unwrap();

    let descriptor = foundation
        .discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_mcp".into(),
            access_token: access_token("ten_mcp", vec![DISCOVER_SCOPE.into()], 100),
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        })
        .expect("tenant-scoped MCP discovery succeeds");

    let tool = &descriptor.tools[0];
    assert_eq!(
        tool.description.value,
        "Agent-authored release readiness check."
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
fn mcp_tool_call_requires_scope_then_invokes_through_foundry_hot_path() {
    let mut foundation = Foundation::default();
    onboard_tenant(&mut foundation, "ten_mcp", AutonomyTier::T2Advisory);
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_mcp".into(),
            user_id: "usr_operator".into(),
            primary_identifier: "operator@mcp.oyatie.test".into(),
            display_name: "MCP Operator".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    support::allow_capability_invocation(&mut foundation, "ten_mcp", "tenant-admin");
    foundation
        .grant_data_use(
            "ten_mcp",
            Purpose::CapabilityInvocation,
            support::privacy_data_class(DataClass::InternalOnly),
        )
        .unwrap();
    register_internal_capability(
        &mut foundation,
        "cap.demo.invoke",
        AutonomyTier::T1ViewOnly,
        true,
    );
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_mcp".into(),
            capability_id: None,
            window_id: "mcp-window".into(),
            monthly_limit_micros: 1_000_000,
            per_invocation_limit_micros: 1_000,
            warning_threshold_percent: 80,
        })
        .unwrap();
    foundation
        .configure_mcp_rate_limit(McpRateLimitPolicy::new(1, 60).unwrap())
        .unwrap();

    let base_request = |scopes: Vec<String>| McpToolCallRequest {
        tenant_id: "ten_mcp".into(),
        user_id: "usr_operator".into(),
        tool_name: "cap.demo.invoke".into(),
        access_token: access_token("ten_mcp", scopes, 10_000),
        tld: "test".into(),
        authorization_server: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        purpose: Purpose::CapabilityInvocation,
        subject_class: SubjectClass::Adult,
        budget_window_id: "mcp-window".into(),
        projected_cost_micros: 125,
        started_at_epoch_seconds: 2_000,
    };

    assert_eq!(
        foundation.invoke_capability_via_mcp(base_request(vec![DISCOVER_SCOPE.into()])),
        Err(FoundationError::McpAccessDenied)
    );

    let mut subject_mismatch = base_request(vec![scope_for_tool_name("cap.demo.invoke")]);
    subject_mismatch.access_token.subject_id = "usr_impersonator".into();
    assert_eq!(
        foundation.invoke_capability_via_mcp(subject_mismatch),
        Err(FoundationError::McpAccessDenied)
    );

    let receipt = foundation
        .invoke_capability_via_mcp(base_request(vec![scope_for_tool_name("cap.demo.invoke")]))
        .expect("MCP call enters the Foundry invocation path");

    assert_eq!(receipt.capability_id, "cap.demo.invoke");
    assert!(receipt.run_id.is_some());
    assert_eq!(
        foundation
            .invoke_capability_via_mcp(base_request(vec![scope_for_tool_name("cap.demo.invoke")])),
        Err(FoundationError::McpRateLimited)
    );
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| event.surface == "foundry.mcp.tool.call" && event.decision == "ALLOW")
    );
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| event.surface == "foundry.capability.invoke" && event.decision == "ALLOW")
    );
}

fn onboard_tenant(foundation: &mut Foundation, tenant_id: &str, autonomy_ceiling: AutonomyTier) {
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: tenant_id.into(),
            legal_name: format!("{tenant_id} tenant"),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling,
        })
        .unwrap();
}

fn access_token(
    tenant_id: &str,
    scopes: Vec<String>,
    expires_at_epoch_seconds: u64,
) -> McpAccessTokenClaims {
    McpAccessTokenClaims {
        tenant_id: tenant_id.into(),
        subject_id: "usr_operator".into(),
        issuer: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        audience: "https://mcp.foundry.region-home.oyatie.test/tenants/ten_mcp".into(),
        expires_at_epoch_seconds,
        scopes,
    }
}

fn register_internal_capability(
    foundation: &mut Foundation,
    capability_id: &str,
    required_tier: AutonomyTier,
    mcp_visible: bool,
) {
    support::seed_passing_eval(foundation, capability_id);
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: capability_id.into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_mcp".into(),
            capability_id: capability_id.into(),
            mcp_visible,
        })
        .unwrap();
}
