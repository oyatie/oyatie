// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use oya_cloud_billing_domain::Money;
use oya_cloud_finops_api::{
    CLOUD_FINOPS_REPORT_SURFACE, CloudFinopsApiAuthorization, CloudFinopsApiBoundaryContext,
    CloudFinopsApiPrincipal, CloudFinopsAxisRef, CloudFinopsMoneyRecord, CloudFinopsPeriodRequest,
    CloudFinopsReportAnomalyPolicyRequest, CloudFinopsReportApiError, CloudFinopsReportApiRequest,
    CloudFinopsReportApiStatus, CloudFinopsReportGenerateIdempotencyLedger,
    CloudFinopsReportGenerateRequest, generate_cloud_finops_report_from_api,
};
use oya_cloud_finops_domain::{
    AxisBudgetCreate, CloudFinopsLedger, CostAllocationCreate, FinopsPeriod, RateCardLineCreate,
    STABLE_GROSS_MARGIN_TARGET_BPS, UnitRate,
};
use oya_data_boundary_kernel::DataClass;
use oya_metering_domain::{
    AxisId, MeterEvent, MeterEventCreate, MeterUnit, MeterUnitKind, PlaneTag,
};

const REPORT_ID: &str = "finr_kr_month";
const TENANT: &str = "ten_alpha";
const REGION: &str = "kr-seoul";
const RESOURCE: &str = "oya:cloud:kr-seoul:ten_alpha:instance:vm-a";
const RATE_CARD: &str = "rate/kr-standard";

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
        data_class: DataClass::FinancialKrCredit,
    }
}

fn allocation(id: &str, event: MeterEvent) -> CostAllocationCreate {
    CostAllocationCreate {
        id: id.to_string(),
        region: REGION.to_string(),
        resource_id: RESOURCE.to_string(),
        rate_card_ref: RATE_CARD.to_string(),
        meter_event: event,
        data_class: DataClass::FinancialKrCredit,
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
            data_class: DataClass::FinancialKrCredit,
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
        data_class: "FINANCIAL_KR_신용정보".to_string(),
    }
}

fn create_request(request_id: &str, idempotency_key: &str) -> CloudFinopsReportApiRequest {
    CloudFinopsReportApiRequest {
        path_report_id: REPORT_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_finops_admin"),
        authorization: authorization_for("sp_finops_admin", &[CLOUD_FINOPS_REPORT_SURFACE]),
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

#[test]
fn finops_report_api_generates_report_once_and_replays_same_idempotent_result() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let request = create_request("req-finops-report-create", "idem_finops_report_create");

    let first =
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request.clone())
            .expect("authorized report generation succeeds");
    let second = generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request)
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
    assert_eq!(first.data.data_class, "FINANCIAL_KR");
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

#[test]
fn finops_report_api_rejects_path_body_report_drift_before_ledger() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let mut request = create_request("req-finops-report-drift", "idem_finops_report_drift");
    request.body.id = "finr_other".to_string();

    let error = generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request)
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
    let mut unauthenticated = create_request("req-finops-report-authn", "idem_finops_report_authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error =
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, unauthenticated)
            .expect_err("missing principal is authentication failure");
    assert_eq!(authn_error, CloudFinopsReportApiError::EmptyPrincipalId);
    assert_eq!(authn_error.finops_report_status_code(), 401);

    let mut denied = create_request("req-finops-report-authz", "idem_finops_report_authz");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.invoice.generate".to_string()];
    let authz_error = generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, denied)
        .expect_err("authorization decision excludes finops report");
    assert_eq!(
        authz_error,
        CloudFinopsReportApiError::AuthorizationDenied {
            surface: CLOUD_FINOPS_REPORT_SURFACE.to_string()
        }
    );
    assert_eq!(authz_error.finops_report_status_code(), 403);
    assert!(idempotency.is_empty());
}

#[test]
fn finops_report_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut ledger = ledger_with_report_data();
    let mut idempotency = CloudFinopsReportGenerateIdempotencyLedger::default();
    let mut request = create_request(" ", "idem_finops_report_empty_header");
    assert_eq!(
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request.clone()),
        Err(CloudFinopsReportApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-finops-report-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request),
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
    let mut invalid_axis = create_request("req-finops-report-axis", "idem_finops_report_axis");
    invalid_axis.body.axes = vec![CloudFinopsAxisRef {
        value: "cloud-but-not-really".to_string(),
    }];
    assert_eq!(
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, invalid_axis),
        Err(CloudFinopsReportApiError::InvalidAxisLabel {
            axis: "cloud-but-not-really".to_string()
        })
    );

    let mut invalid_class = create_request("req-finops-report-class", "idem_finops_report_class");
    invalid_class.body.data_class = "NOT_A_CLASS".to_string();
    assert_eq!(
        generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, invalid_class),
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
    let mut request = create_request("req-finops-report-idem", "idem_finops_report_reused");
    generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request.clone())
        .expect("first request records idempotency result");

    request.body.minimum_gross_margin_bps = 7_000;
    let error = generate_cloud_finops_report_from_api(&mut ledger, &mut idempotency, request)
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
    generate_cloud_finops_report_from_api(
        &mut ledger,
        &mut idempotency,
        create_request("req-finops-report-first", "idem_finops_report_first"),
    )
    .expect("first report is generated");

    let duplicate = generate_cloud_finops_report_from_api(
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
        &mut empty_ledger,
        &mut CloudFinopsReportGenerateIdempotencyLedger::default(),
        create_request("req-finops-report-no-data", "idem_finops_report_no_data"),
    )
    .expect_err("well-formed report with no allocations is unprocessable");
    assert_eq!(
        no_data,
        CloudFinopsReportApiError::Finops(oya_cloud_finops_domain::CloudFinopsError::NoReportData)
    );
    assert_eq!(no_data.finops_report_status_code(), 422);
}
