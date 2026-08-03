// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use billing_domain::Money;
use billing_finops_api::authz::{
    CallerCredential, ConfiguredBearerPrincipalVerifier, FinopsReportAuthorizationError,
    FinopsReportAuthorizer, FinopsReportResource, FinopsReportScope, PrincipalVerifier,
    VerifiedPrincipal,
};
use billing_finops_api::{
    CLOUD_FINOPS_REPORT_SURFACE, CloudFinopsApiAuthorization, CloudFinopsApiBoundaryContext,
    CloudFinopsApiPrincipal, CloudFinopsAxisRef, CloudFinopsMoneyRecord, CloudFinopsPeriodRequest,
    CloudFinopsReportAnomalyPolicyRequest, CloudFinopsReportApiError, CloudFinopsReportApiRequest,
    CloudFinopsReportApiStatus, CloudFinopsReportGenerateIdempotencyLedger,
    CloudFinopsReportGenerateRequest, PLATFORM_AGGREGATE_TENANT_ID,
    generate_cloud_finops_report_from_api,
};
use billing_finops::{
    AxisBudgetCreate, CloudFinopsLedger, CostAllocationCreate, FinopsPeriod, RateCardLineCreate,
    STABLE_GROSS_MARGIN_TARGET_BPS, UnitRate,
};
use data_boundary_kernel::DataClass;
use billing_metering::{
    AxisId, MeterEvent, MeterEventCreate, MeterUnit, MeterUnitKind, PlaneTag,
};

const REPORT_ID: &str = "finr_kr_month";
const TENANT: &str = "ten_alpha";
const PRINCIPAL: &str = "sp_finops_admin";
const BEARER: &str = "br_finops_alpha_secret";
const REGION: &str = "region-home";
const RESOURCE: &str = "oya:cloud:region-home:ten_alpha:instance:vm-a";
const RATE_CARD: &str = "rate/kr-standard";

// ── Test PDP authorizers ───────────────────────────────────────────────────

/// A PDP that authorizes any principal for any resource. Used to prove the
/// blast-radius binding: even when the PDP would say yes, a cross-tenant request
/// is denied by the verified-identity cross-check BEFORE the PDP is reached.
struct AllowAll;
impl FinopsReportAuthorizer for AllowAll {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError> {
        Ok(())
    }
}

/// A PDP that denies every decision (default-deny). Proves PDP-deny → 403.
struct DenyAll;
impl FinopsReportAuthorizer for DenyAll {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError> {
        Err(FinopsReportAuthorizationError::Denied)
    }
}

/// A PDP that refuses (fault / unavailable). Proves a PDP fault is fail-closed
/// (treated as deny → 403), not fail-open.
struct RefuseAll;
impl FinopsReportAuthorizer for RefuseAll {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError> {
        Err(FinopsReportAuthorizationError::Refused)
    }
}

/// A PDP that authorizes ONLY tenant-scoped reads for the bound tenant, and
/// denies platform-scoped reads. Proves the platform blast-radius distinction:
/// a tenant-admin who self-asserts the platform tenant is denied at the PDP
/// because the resource is presented as Platform scope, not the caller's tenant.
struct TenantOnly {
    tenant_id: String,
}
impl FinopsReportAuthorizer for TenantOnly {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError> {
        match resource.scope {
            FinopsReportScope::Tenant
                if resource.tenant_id == self.tenant_id
                    && principal.tenant_id() == self.tenant_id =>
            {
                Ok(())
            }
            _ => Err(FinopsReportAuthorizationError::Denied),
        }
    }
}

/// Mint a verified principal the way production does: run a real
/// `ConfiguredBearerPrincipalVerifier` over a valid bearer credential. The test
/// crate cannot construct a `VerifiedPrincipal` by hand (private fields,
/// pub(crate) constructor) — exactly the unforgeability property under test.
fn verified_principal(principal_id: &str, tenant_id: &str) -> VerifiedPrincipal {
    let verifier =
        ConfiguredBearerPrincipalVerifier::new(BEARER, principal_id, tenant_id).expect("verifier");
    verifier
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER}")),
            claimed_principal_id: principal_id.to_string(),
            claimed_tenant_id: tenant_id.to_string(),
        })
        .expect("valid bearer verifies")
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudFinopsApiBoundaryContext {
    CloudFinopsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudFinopsApiPrincipal {
    CloudFinopsApiPrincipal {
        tenant_id: TENANT.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudFinopsApiAuthorization {
    CloudFinopsApiAuthorization {
        tenant_id: TENANT.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn money_krw(minor_units: u64) -> Money {
    Money::new("KRW", minor_units).expect("money fixture valid")
}

fn period() -> CloudFinopsPeriodRequest {
    CloudFinopsPeriodRequest {
        start_epoch_seconds: 1_000,
        end_epoch_seconds: 2_000,
    }
}

fn baseline_period() -> CloudFinopsPeriodRequest {
    CloudFinopsPeriodRequest {
        start_epoch_seconds: 1,
        end_epoch_seconds: 1_001,
    }
}

fn finops_period(input: &CloudFinopsPeriodRequest) -> FinopsPeriod {
    FinopsPeriod::new(input.start_epoch_seconds, input.end_epoch_seconds).expect("period")
}

fn meter_event(id: &str, axis: AxisId, quantity: u64, ts: u64) -> MeterEvent {
    MeterEvent::new(MeterEventCreate {
        id: id.to_string(),
        tenant_id: TENANT.to_string(),
        capability_id: "cap.cloud.compute.vm".to_string(),
        plane: PlaneTag::Data,
        units: vec![MeterUnit::new(MeterUnitKind::ResourceSecond, quantity).expect("unit")],
        source_axis: axis,
        recorded_at_epoch_seconds: ts,
        idempotency_key: format!("idem_{id}"),
        data_class: DataClass::Public,
    })
    .expect("meter event")
}

fn rate_line(axis: AxisId, rate: UnitRate) -> RateCardLineCreate {
    RateCardLineCreate {
        rate_card_ref: RATE_CARD.to_string(),
        region: REGION.to_string(),
        axis,
        unit_kind: MeterUnitKind::ResourceSecond,
        currency: "KRW".to_string(),
        rate,
        effective_period: FinopsPeriod::new(1, 3_000).expect("rate period"),
        data_class: DataClass::FinancialRegulatedCredit,
    }
}

fn allocation(id: &str, event: MeterEvent) -> CostAllocationCreate {
    CostAllocationCreate {
        id: id.to_string(),
        region: REGION.to_string(),
        resource_id: RESOURCE.to_string(),
        rate_card_ref: RATE_CARD.to_string(),
        meter_event: event,
        data_class: DataClass::FinancialRegulatedCredit,
    }
}

fn ledger_with_report_data() -> CloudFinopsLedger {
    let mut ledger = CloudFinopsLedger::default();
    ledger
        .add_rate_card_line(rate_line(
            AxisId::Cloud,
            UnitRate::new(2_000, 800).expect("cloud rate"),
        ))
        .expect("cloud rate line");
    ledger
        .add_rate_card_line(rate_line(
            AxisId::Saas,
            UnitRate::new(1_000, 600).expect("saas rate"),
        ))
        .expect("saas rate line");
    ledger
        .set_budget(AxisBudgetCreate {
            id: "fbg_cloud".to_string(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            axis: AxisId::Cloud,
            period: finops_period(&period()),
            budget: money_krw(3_000),
            soft_threshold_bps: 8_000,
            hard_threshold_bps: 10_000,
            data_class: DataClass::FinancialRegulatedCredit,
        })
        .expect("budget");
    ledger
        .record_allocation(allocation(
            "fca_base_cloud",
            meter_event("mtr_base_cloud", AxisId::Cloud, 1_000_000, 500),
        ))
        .expect("baseline allocation");
    ledger
        .record_allocation(allocation(
            "fca_current_cloud",
            meter_event("mtr_current_cloud", AxisId::Cloud, 2_000_000, 1_500),
        ))
        .expect("current cloud allocation");
    ledger
        .record_allocation(allocation(
            "fca_current_saas",
            meter_event("mtr_current_saas", AxisId::Saas, 1_000_000, 1_600),
        ))
        .expect("current saas allocation");
    ledger
}

fn report_body(id: &str) -> CloudFinopsReportGenerateRequest {
    CloudFinopsReportGenerateRequest {
        id: id.to_string(),
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        period: period(),
        baseline_period: Some(baseline_period()),
        axes: vec![
            CloudFinopsAxisRef {
                value: "cloud".to_string(),
            },
            CloudFinopsAxisRef {
                value: "saas".to_string(),
            },
        ],
        anomaly_policy: CloudFinopsReportAnomalyPolicyRequest {
            spend_growth_threshold_bps: 1_000,
            min_absolute_delta_minor_units: 100,
        },
        minimum_gross_margin_bps: STABLE_GROSS_MARGIN_TARGET_BPS,
        data_class: "FINANCIAL_REGULATED_CREDIT".to_string(),
    }
}

fn create_request(request_id: &str, idempotency_key: &str) -> CloudFinopsReportApiRequest {
    CloudFinopsReportApiRequest {
        path_report_id: REPORT_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(PRINCIPAL),
        authorization: authorization_for(PRINCIPAL, &[CLOUD_FINOPS_REPORT_SURFACE]),
        body: report_body(REPORT_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_FINOPS_REPORT_SURFACE, "cloud.finops.report");
    assert_eq!(CloudFinopsReportApiStatus::Created.code(), 201);
    assert_eq!(CloudFinopsReportApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudFinopsReportApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudFinopsReportApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudFinopsReportApiStatus::Conflict.code(), 409);
    assert_eq!(CloudFinopsReportApiStatus::UnprocessableEntity.code(), 422);
}

// ── HAPPY PATH ──────────────────────────────────────────────────────────────

#[test]
fn finops_report_api_generates_report_once_and_replays_same_idempotent_result() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = AllowAll;
    let request = create_request("req-finops-report-create", "idem_finops_report_create");

    let first = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request.clone(),
    )
    .expect("authorized report generation succeeds");
    let second = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.metadata.request_id, "req-finops-report-create");
    assert_eq!(first.metadata.tenant_id, TENANT);
    assert_eq!(first.metadata.region, REGION);
    assert_eq!(first.metadata.axis_count, 2);
    assert_eq!(first.metadata.resource_count, 2);
    assert_eq!(first.metadata.anomaly_count, 2);
    assert_eq!(first.data.id, REPORT_ID);
    assert_eq!(first.data.tenant_id, TENANT);
    assert_eq!(first.data.region, REGION);
    assert_eq!(
        first.data.total_cost,
        CloudFinopsMoneyRecord {
            currency: "KRW".to_string(),
            minor_units: 5_000
        }
    );
    assert_eq!(first.data.total_cost_of_revenue.minor_units, 2_200);
    assert_eq!(first.data.gross_margin_bps, 5_600);
    assert_eq!(
        first.data.axes,
        vec![
            CloudFinopsAxisRef {
                value: "cloud".to_string()
            },
            CloudFinopsAxisRef {
                value: "saas".to_string()
            }
        ]
    );
    assert_eq!(first.data.axis_costs.len(), 2);
    assert_eq!(first.data.resource_costs.len(), 2);
    assert_eq!(first.data.recommendations.len(), first.data.anomalies.len());
    assert_eq!(first.data.data_class, "FINANCIAL_REGULATED_CREDIT");
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(
        first
            .data
            .anomalies
            .iter()
            .map(|anomaly| anomaly.kind.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["budget_hard_limit", "spend_spike"])
    );
}

// ── RED/GREEN authz seam (must fail if the gate is removed) ──────────────────

/// FORGED / ABSENT credential → 401. The verifier rejects a wrong bearer, so no
/// `VerifiedPrincipal` can be minted — the report fn is unreachable without one.
#[test]
fn finops_report_api_rejects_forged_or_absent_credential_with_401() {
    use billing_finops_api::authz::PrincipalVerificationError;
    let verifier =
        ConfiguredBearerPrincipalVerifier::new(BEARER, PRINCIPAL, TENANT).expect("verifier");

    // Absent credential.
    let absent = verifier.verify_principal(&CallerCredential {
        authorization: None,
        claimed_principal_id: PRINCIPAL.to_string(),
        claimed_tenant_id: TENANT.to_string(),
    });
    assert_eq!(absent, Err(PrincipalVerificationError::MissingCredential));

    // Forged credential (wrong bearer).
    let forged = verifier.verify_principal(&CallerCredential {
        authorization: Some("Bearer not-the-secret".to_string()),
        claimed_principal_id: PRINCIPAL.to_string(),
        claimed_tenant_id: TENANT.to_string(),
    });
    assert_eq!(forged, Err(PrincipalVerificationError::InvalidCredential));

    // The 401-class boundary error maps to status 401.
    assert_eq!(
        CloudFinopsReportApiError::PrincipalUnverified.finops_report_status_code(),
        401
    );
}

/// VERIFIED CROSS-TENANT → 403, EVEN WITH an authorizer that would otherwise
/// allow (`AllowAll`). This proves the blast-radius binding: a principal verified
/// as tenant B asking for tenant A's report is rejected by the verified-identity
/// cross-check before the PDP can rubber-stamp it. The request body still claims
/// tenant A throughout (so the legacy self-asserted authz would PASS).
#[test]
fn finops_report_api_denies_verified_cross_tenant_report_even_when_pdp_would_allow() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    // Verified as the SAME principal id but a DIFFERENT tenant than the requested
    // report — isolates the cross-TENANT denial (not a principal-id mismatch).
    let verified = verified_principal(PRINCIPAL, "ten_beta");
    let authorizer = AllowAll; // would say yes — proves the cross-check binds first.
    let request = create_request("req-finops-cross-tenant", "idem_finops_cross_tenant");

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("verified cross-tenant report is denied");

    assert_eq!(
        error,
        CloudFinopsReportApiError::VerifiedTenantMismatch {
            verified_tenant_id: "ten_beta".to_string(),
            request_tenant_id: TENANT.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty(), "denied request must not mutate ledger");
}

/// VERIFIED PRINCIPAL SUBSTITUTION → 403. Same tenant, different principal id
/// than the verifier bound.
#[test]
fn finops_report_api_denies_verified_principal_substitution() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal("sp_someone_else", TENANT);
    let authorizer = AllowAll;
    let request = create_request("req-finops-principal-sub", "idem_finops_principal_sub");

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("verified principal substitution is denied");

    assert_eq!(
        error,
        CloudFinopsReportApiError::VerifiedPrincipalMismatch {
            verified_principal_id: "sp_someone_else".to_string(),
            request_principal_id: PRINCIPAL.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty());
}

/// PDP-DENY → 403. Identity verifies and matches, but the PDP denies.
#[test]
fn finops_report_api_maps_pdp_deny_to_403() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = DenyAll;
    let request = create_request("req-finops-pdp-deny", "idem_finops_pdp_deny");

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("PDP deny is 403");

    assert_eq!(
        error,
        CloudFinopsReportApiError::PdpDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty(), "PDP-denied request must not mutate ledger");
}

/// PDP-REFUSE (fault / unavailable) → 403 (fail-closed, not fail-open).
#[test]
fn finops_report_api_maps_pdp_refusal_to_403_fail_closed() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = RefuseAll;
    let request = create_request("req-finops-pdp-refuse", "idem_finops_pdp_refuse");

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("PDP refusal is fail-closed 403");

    assert_eq!(
        error,
        CloudFinopsReportApiError::PdpDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty());
}

/// PLATFORM BLAST-RADIUS (the #815 global-scope CRITICAL): a request targeting
/// the reserved platform-aggregate tenant is presented to the PDP as a Platform
/// resource. A tenant-only authorizer denies it — a tenant-finops-admin cannot
/// exfiltrate platform-wide spend by self-asserting the platform tenant.
#[test]
fn finops_report_api_presents_platform_aggregate_as_platform_resource_to_pdp() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    // Verified principal whose tenant IS the platform tenant (self-asserted) and
    // an authorizer that only allows tenant-scoped reads for that tenant.
    let verified = verified_principal(PRINCIPAL, PLATFORM_AGGREGATE_TENANT_ID);
    let authorizer = TenantOnly {
        tenant_id: PLATFORM_AGGREGATE_TENANT_ID.to_string(),
    };
    let mut request = create_request("req-finops-platform", "idem_finops_platform");
    request.boundary.tenant_id = PLATFORM_AGGREGATE_TENANT_ID.to_string();
    request.principal.tenant_id = PLATFORM_AGGREGATE_TENANT_ID.to_string();
    request.authorization.tenant_id = PLATFORM_AGGREGATE_TENANT_ID.to_string();
    request.body.tenant_id = PLATFORM_AGGREGATE_TENANT_ID.to_string();

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("platform aggregate requires platform-admin, not tenant authority");

    assert_eq!(
        error,
        CloudFinopsReportApiError::PdpDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty());
}

// ── Structural validation (shape gate, unchanged behavior) ───────────────────

#[test]
fn finops_report_api_rejects_path_body_report_drift_before_ledger() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = AllowAll;
    let mut request = create_request("req-finops-report-drift", "idem_finops_report_drift");
    request.body.id = "finr_other".to_string();
    request.path_report_id = REPORT_ID.to_string();

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("path/body report drift is rejected");

    assert_eq!(
        error,
        CloudFinopsReportApiError::ReportIdMismatch {
            path_report_id: REPORT_ID.to_string(),
            body_report_id: "finr_other".to_string()
        }
    );
    assert_eq!(error.finops_report_status_code(), 400);
    assert!(idempotency.is_empty());
}

#[test]
fn finops_report_api_separates_missing_authentication_from_denied_authorization() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);

    // Missing principal id in the request → 401 authentication failure.
    let mut unauthenticated =
        create_request("req-finops-report-authn", "idem_finops_report_authn");
    unauthenticated.principal.principal_id = " ".to_string();
    let authn_error = generate_cloud_finops_report_from_api(
        &verified,
        &AllowAll,
        &mut ledger,
        &mut idempotency,
        unauthenticated,
    )
    .expect_err("missing principal is authentication failure");
    assert_eq!(authn_error, CloudFinopsReportApiError::EmptyPrincipalId);
    assert_eq!(authn_error.finops_report_status_code(), 401);

    // Self-asserted allowed_surfaces NO LONGER grants: the PDP is authoritative.
    // Even with the surface dropped from the caller-supplied authorization, the
    // request is decided by the PDP (DenyAll here) → 403, proving the caller can
    // no longer authorize itself by setting allowed_surfaces.
    let mut denied = create_request("req-finops-report-authz", "idem_finops_report_authz");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.invoice.generate".to_string()];
    let authz_error = generate_cloud_finops_report_from_api(
        &verified,
        &DenyAll,
        &mut ledger,
        &mut idempotency,
        denied,
    )
    .expect_err("authorization is decided by the PDP, not the caller");
    assert_eq!(
        authz_error,
        CloudFinopsReportApiError::PdpDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string()
        }
    );
    assert_eq!(authz_error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty());
}

/// Self-asserted authorization CANNOT grant access: a caller who claims
/// `allowed_surfaces = ["cloud.finops.report"]` is STILL denied by a DenyAll PDP.
/// This is the direct RED test for the gap-fill CRIT.
#[test]
fn finops_report_api_self_asserted_surfaces_do_not_grant_access() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    // Caller self-asserts the exact surface — the forgeable old grant.
    let request = create_request("req-finops-self-grant", "idem_finops_self_grant");
    assert_eq!(
        request.authorization.allowed_surfaces,
        vec![CLOUD_FINOPS_REPORT_SURFACE.to_string()]
    );

    let error = generate_cloud_finops_report_from_api(
        &verified,
        &DenyAll, // server-side decision wins.
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("self-asserted surface does not authorize against a deny PDP");

    assert_eq!(
        error,
        CloudFinopsReportApiError::PdpDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.finops_report_status_code(), 403);
}

#[test]
fn finops_report_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let mut request = create_request(" ", "idem_finops_report_empty_header");
    assert_eq!(
        generate_cloud_finops_report_from_api(
            &verified,
            &AllowAll,
            &mut ledger,
            &mut idempotency,
            request.clone()
        ),
        Err(CloudFinopsReportApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-finops-report-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        generate_cloud_finops_report_from_api(
            &verified,
            &AllowAll,
            &mut ledger,
            &mut idempotency,
            request
        ),
        Err(CloudFinopsReportApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: TENANT.to_string(),
            body_tenant_id: TENANT.to_string()
        })
    );
    assert!(idempotency.is_empty());
}

#[test]
fn finops_report_api_rejects_invalid_axis_and_data_class_labels_before_kernel() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let mut invalid_axis = create_request("req-finops-report-axis", "idem_finops_report_axis");
    invalid_axis.body.axes = vec![CloudFinopsAxisRef {
        value: "cloud-but-not-really".to_string(),
    }];
    assert_eq!(
        generate_cloud_finops_report_from_api(
            &verified,
            &AllowAll,
            &mut ledger,
            &mut idempotency,
            invalid_axis
        ),
        Err(CloudFinopsReportApiError::InvalidAxisLabel {
            axis: "cloud-but-not-really".to_string()
        })
    );

    let mut invalid_class = create_request("req-finops-report-class", "idem_finops_report_class");
    invalid_class.body.data_class = "NOT_A_CLASS".to_string();
    assert_eq!(
        generate_cloud_finops_report_from_api(
            &verified,
            &AllowAll,
            &mut ledger,
            &mut idempotency,
            invalid_class
        ),
        Err(CloudFinopsReportApiError::InvalidDataClassLabel {
            data_class: "NOT_A_CLASS".to_string()
        })
    );
    assert!(idempotency.is_empty());
}

#[test]
fn finops_report_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = AllowAll;
    let mut request = create_request("req-finops-report-idem", "idem_finops_report_reused");
    generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request.clone(),
    )
    .expect("first request records idempotency result");

    request.body.minimum_gross_margin_bps = 7_000;
    let error = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        request,
    )
    .expect_err("same idempotency key with changed body is rejected");

    assert_eq!(
        error,
        CloudFinopsReportApiError::IdempotencyKeyReused {
            idempotency_key: "idem_finops_report_reused".to_string()
        }
    );
    assert_eq!(error.finops_report_status_code(), 422);
    assert_eq!(idempotency.len(), 1);
}

#[test]
fn finops_report_api_maps_kernel_duplicate_report_and_no_data_errors() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let verified = verified_principal(PRINCIPAL, TENANT);
    let authorizer = AllowAll;
    generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        create_request("req-finops-report-first", "idem_finops_report_first"),
    )
    .expect("first report is generated");

    let duplicate = generate_cloud_finops_report_from_api(
        &verified,
        &authorizer,
        &mut ledger,
        &mut idempotency,
        create_request(
            "req-finops-report-duplicate",
            "idem_finops_report_duplicate",
        ),
    )
    .expect_err("duplicate report id maps to conflict");
    assert_eq!(duplicate.finops_report_status_code(), 409);

    let mut empty_ledger = CloudFinopsLedger::default();
    let no_data = generate_cloud_finops_report_from_api(
        &verified,
        &AllowAll,
        &mut empty_ledger,
        &mut CloudFinopsReportGenerateIdempotencyLedger::default(),
        create_request("req-finops-report-no-data", "idem_finops_report_no_data"),
    )
    .expect_err("well-formed report with no allocations is unprocessable");
    assert_eq!(
        no_data,
        CloudFinopsReportApiError::Finops(billing_finops::CloudFinopsError::NoReportData)
    );
    assert_eq!(no_data.finops_report_status_code(), 422);
}
