use application_app::{
    AutonomyTier, CapabilityAction, CapabilityInvocationPrincipal, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DISCOVER_SCOPE, Foundation, FoundationError,
    IdentityRegistration, McpAccessTokenClaims, McpDiscoveryRequest, McpToolCallRequest, Purpose,
    SubjectClass, TenantCapabilityGrant, TenantRegistration, scope_for_tool_name,
};

use crate::{foundation_fixture, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CrossTenantAccessFuzzValidateArgs;

pub(crate) fn parse_cross_tenant_access_fuzz_validate_args(
    args: Vec<String>,
) -> Result<CrossTenantAccessFuzzValidateArgs, String> {
    if args.is_empty() {
        Ok(CrossTenantAccessFuzzValidateArgs)
    } else {
        Err(usage())
    }
}

pub(crate) fn validate_cross_tenant_access_fuzz_gate(
    _args: CrossTenantAccessFuzzValidateArgs,
) -> Result<usize, String> {
    let mut foundation = Foundation::default();
    setup_cross_tenant_fixture(&mut foundation)?;

    let mut cases = 0usize;
    let tenant_a_cell = foundation
        .bind_cell("ten_alpha", "region-home-a", "cell-alpha")
        .map_err(|error| format!("tenant alpha cell bind failed: {error:?}"))?;
    let tenant_b_cell = foundation
        .bind_cell("ten_beta", "region-home-b", "cell-beta")
        .map_err(|error| format!("tenant beta cell bind failed: {error:?}"))?;
    if tenant_a_cell.cell_id.value == tenant_b_cell.cell_id.value {
        return Err("tenant fixture cells unexpectedly collapsed to one cell".to_string());
    }
    cases += 1;

    expect_foundation_error(
        "cell binding is immutable per tenant",
        foundation.bind_cell("ten_alpha", "region-home-c", "cell-tamper"),
        FoundationError::CellBindingImmutable,
    )?;
    cases += 1;

    expect_foundation_error(
        "MCP discovery rejects token from another tenant",
        foundation.discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: "ten_alpha".into(),
            access_token: cross_tenant_access_token(
                "ten_beta",
                vec![DISCOVER_SCOPE.into()],
                "ten_alpha",
                10_000,
            ),
            now_epoch_seconds: 10,
            tld: "test".into(),
            authorization_server: authorization_server_for("ten_alpha"),
        }),
        FoundationError::McpAccessDenied,
    )?;
    cases += 1;

    expect_foundation_error(
        "MCP tool call rejects token from another tenant",
        foundation.invoke_capability_via_mcp(McpToolCallRequest {
            tenant_id: "ten_alpha".into(),
            user_id: "usr_operator".into(),
            tool_name: "cap.cross-tenant.fixture".into(),
            access_token: cross_tenant_access_token(
                "ten_beta",
                vec![scope_for_tool_name("cap.cross-tenant.fixture")],
                "ten_alpha",
                10_000,
            ),
            tld: "test".into(),
            authorization_server: authorization_server_for("ten_alpha"),
            purpose: Purpose::CapabilityInvocation,
            subject_class: SubjectClass::Adult,
            budget_window_id: "cross-tenant-window".into(),
            projected_cost_micros: 100,
            started_at_epoch_seconds: 20,
        }),
        FoundationError::McpAccessDenied,
    )?;
    cases += 1;

    expect_foundation_error(
        "direct capability invocation rejects tenant without grant",
        foundation.invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: "ten_beta".into(),
                user_id: "usr_operator".into(),
                autonomy_ceiling: AutonomyTier::T2Advisory,
            },
            CapabilityInvocationRequest {
                tenant_id: "ten_beta".into(),
                user_id: "usr_operator".into(),
                capability_id: "cap.cross-tenant.fixture".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "cross-tenant-window".into(),
                projected_cost_micros: 100,
                started_at_epoch_seconds: 30,
            },
        ),
        FoundationError::CapabilityNotLicensed,
    )?;
    cases += 1;

    foundation
        .invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: "ten_alpha".into(),
                user_id: "usr_operator".into(),
                autonomy_ceiling: AutonomyTier::T2Advisory,
            },
            CapabilityInvocationRequest {
                tenant_id: "ten_alpha".into(),
                user_id: "usr_operator".into(),
                capability_id: "cap.cross-tenant.fixture".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "cross-tenant-window".into(),
                projected_cost_micros: 100,
                started_at_epoch_seconds: 40,
            },
        )
        .map_err(|error| format!("same-tenant control invocation failed: {error:?}"))?;
    cases += 1;

    if !foundation.audit_chain().verify() {
        return Err("cross-tenant fuzz audit chain did not verify".to_string());
    }
    cases += 1;

    Ok(cases)
}

fn setup_cross_tenant_fixture(foundation: &mut Foundation) -> Result<(), String> {
    for tenant_id in ["ten_alpha", "ten_beta"] {
        foundation
            .onboard_tenant(TenantRegistration {
                tenant_id: tenant_id.into(),
                legal_name: format!("{tenant_id} tenant"),
                home_region: "region-home".into(),
                residency_class: "strict_home_region".into(),
                regulatory_packs: vec!["oya-pack-alpha".into()],
                autonomy_ceiling: AutonomyTier::T2Advisory,
            })
            .map_err(|error| format!("tenant setup failed {tenant_id}: {error:?}"))?;
        foundation
            .upsert_identity(IdentityRegistration {
                tenant_id: tenant_id.into(),
                user_id: "usr_operator".into(),
                primary_identifier: format!("operator@{tenant_id}.oyatie.test"),
                display_name: format!("{tenant_id} Operator"),
                roles: vec!["tenant-admin".into()],
            })
            .map_err(|error| format!("identity setup failed {tenant_id}: {error:?}"))?;
        foundation_fixture::publish_capability_invocation_policy(
            foundation,
            tenant_id,
            "tenant-admin",
        )
        .map_err(|error| format!("invoke policy setup failed {tenant_id}: {error:?}"))?;
        foundation
            .grant_data_use(
                tenant_id,
                Purpose::CapabilityInvocation,
                foundation_fixture::internal_privacy_data_class(),
            )
            .map_err(|error| format!("data-use setup failed {tenant_id}: {error:?}"))?;
        foundation
            .configure_tenant_cost_budget(CostBudgetRegistration {
                tenant_id: tenant_id.into(),
                capability_id: None,
                window_id: "cross-tenant-window".into(),
                monthly_limit_micros: 1_000_000,
                per_invocation_limit_micros: 1_000,
                warning_threshold_percent: 80,
            })
            .map_err(|error| format!("cost budget setup failed {tenant_id}: {error:?}"))?;
    }

    foundation_fixture::seed_demo_eval(foundation, "cap.cross-tenant.fixture")
        .map_err(|error| format!("cross-tenant eval seed failed: {error:?}"))?;
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.cross-tenant.fixture".into(),
            namespace: "foundry.cross-tenant".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: foundation_fixture::internal_privacy_data_classes(),
            evidence_topic: "oya.foundry.cross-tenant.fixture".into(),
        })
        .map_err(|error| format!("capability setup failed: {error:?}"))?;
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_alpha".into(),
            capability_id: "cap.cross-tenant.fixture".into(),
            mcp_visible: true,
        })
        .map_err(|error| format!("capability grant setup failed: {error:?}"))?;
    Ok(())
}

fn expect_foundation_error<T>(
    case: &str,
    actual: Result<T, FoundationError>,
    expected: FoundationError,
) -> Result<(), String> {
    match actual {
        Err(error) if error == expected => Ok(()),
        Err(error) => Err(format!("{case}: expected {expected:?}, got {error:?}")),
        Ok(_) => Err(format!("{case}: unexpectedly allowed")),
    }
}

fn cross_tenant_access_token(
    token_tenant_id: &str,
    scopes: Vec<String>,
    endpoint_tenant_id: &str,
    expires_at_epoch_seconds: u64,
) -> McpAccessTokenClaims {
    McpAccessTokenClaims {
        tenant_id: token_tenant_id.into(),
        subject_id: "usr_operator".into(),
        issuer: authorization_server_for(endpoint_tenant_id),
        audience: mcp_audience_for(endpoint_tenant_id),
        expires_at_epoch_seconds,
        scopes,
    }
}

fn authorization_server_for(tenant_id: &str) -> String {
    format!("https://auth.oyatie.test/tenants/{tenant_id}")
}

fn mcp_audience_for(tenant_id: &str) -> String {
    format!("https://mcp.foundry.region-home.oyatie.test/tenants/{tenant_id}")
}
