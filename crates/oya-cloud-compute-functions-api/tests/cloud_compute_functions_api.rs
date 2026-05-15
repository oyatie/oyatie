// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeRepo, FunctionDeploymentCreate,
    FunctionDeploymentState,
};
use oya_cloud_compute_functions_api::{
    CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE, CloudComputeFunctionsApiAuthorization,
    CloudComputeFunctionsApiBoundaryContext, CloudComputeFunctionsApiError,
    CloudComputeFunctionsApiPrincipal, CloudComputeFunctionsInvokeApiRequest,
    CloudComputeFunctionsInvokeApiStatus, CloudComputeFunctionsInvokeIdempotencyLedger,
    CloudComputeFunctionsInvokeRequest, invoke_cloud_compute_function_from_api,
};
use oya_cloud_resource_domain::FunctionRuntime;
use oya_data_boundary_kernel::DataClass;
use oya_residency_domain::ResidencyClass;

const FUNCTION_ID: &str = "oya:cloud:kr-seoul:ten_kr:function:image-resize";
const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeFunctionsApiBoundaryContext {
    CloudComputeFunctionsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudComputeFunctionsApiPrincipal {
    CloudComputeFunctionsApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudComputeFunctionsApiAuthorization {
    CloudComputeFunctionsApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn function_create() -> FunctionDeploymentCreate {
    FunctionDeploymentCreate {
        resource_id: FUNCTION_ID.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        az: "kr-seoul-a".to_string(),
        cell_id: "cell-kr-seoul-a-001".to_string(),
        runtime: FunctionRuntime::Wasm,
        name: "image-resize".to_string(),
        bundle: format!("function://harbor.kr-seoul.oya/ten_kr/image-resize@sha256:{DIGEST}"),
        cold_start_budget_ms: 750,
        timeout_ms: 30_000,
        memory_mb: 512,
        max_concurrency: 250,
        allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
        residency: ResidencyClass::StrictKr,
        state: FunctionDeploymentState::Deploying,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_100_020,
    }
}

fn seed_active_function(catalog: &mut CloudComputeCatalog) {
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    catalog
        .activate_function(&function.resource_id.value)
        .expect("function activates");
}

fn seed_deploying_function(catalog: &mut CloudComputeCatalog) {
    catalog
        .register_function(function_create())
        .expect("function registers");
}

fn body(invocation_id: &str, payload_data_class: &str) -> CloudComputeFunctionsInvokeRequest {
    CloudComputeFunctionsInvokeRequest {
        invocation_id: invocation_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        function_id: FUNCTION_ID.to_string(),
        region: "kr-seoul".to_string(),
        payload_data_class: payload_data_class.to_string(),
        requested_at_epoch_seconds: 1_700_100_030,
    }
}

fn request(
    request_id: &str,
    idempotency_key: &str,
    invocation_id: &str,
) -> CloudComputeFunctionsInvokeApiRequest {
    CloudComputeFunctionsInvokeApiRequest {
        path_function_id: FUNCTION_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: authorization_for("sp_compute", &[CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE]),
        body: body(invocation_id, "PII_IDENTIFYING"),
    }
}

#[test]
fn api_surface_status_contracts_are_covered() {
    assert_eq!(
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
        "cloud.compute.functions.invoke"
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Accepted.code(), 202);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::BadRequest.code(), 400);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn functions_invoke_api_records_invocation_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-invoke",
        "idem-functions-invoke-001",
        "fninv_001",
    );

    let first = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("authorized function invoke succeeds");
    let second = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-functions-invoke");
    assert_eq!(first.data.invocation_id, "fninv_001");
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.function_id, FUNCTION_ID);
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.payload_data_class, "PII_IDENTIFYING");
    assert_eq!(first.data.cold_start_budget_ms, 750);
    assert_eq!(first.data.accepted_at_epoch_seconds, 1_700_100_030);
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn functions_invoke_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-drift",
        "idem-functions-drift-001",
        "fninv_drift",
    );
    request.body.function_id = "oya:cloud:kr-seoul:ten_kr:function:other".to_string();

    let error = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body function drift is rejected");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::FunctionIdMismatch {
            path_function_id: FUNCTION_ID.to_string(),
            body_function_id: "oya:cloud:kr-seoul:ten_kr:function:other".to_string(),
        }
    );
    assert_eq!(error.invoke_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut missing_principal = request(
        "req-compute-functions-authn",
        "idem-functions-authn-001",
        "fninv_authn",
    );
    missing_principal.principal.principal_id.clear();

    let authn_error =
        invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, missing_principal)
            .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(authn_error, CloudComputeFunctionsApiError::EmptyPrincipalId);
    assert_eq!(authn_error.invoke_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);

    let mut denied = request(
        "req-compute-functions-authz",
        "idem-functions-authz-001",
        "fninv_authz",
    );
    denied.authorization.allowed_surfaces = vec!["cloud.compute.vm.create".to_string()];

    let authz_error = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, denied)
        .expect_err("authorization decision does not allow invoke");

    assert_eq!(
        authz_error,
        CloudComputeFunctionsApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        }
    );
    assert_eq!(authz_error.invoke_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_replays_with_refreshed_authz() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-authz-refresh-1",
        "idem-functions-authz-refresh-001",
        "fninv_authz_refresh",
    );
    let first = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial function invoke succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-functions-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.k8s.cluster.create".to_string(),
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
    ];
    let second = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-idem",
        "idem-functions-idem-001",
        "fninv_idem",
    );
    invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial invoke succeeds");

    let mut drifted = request;
    drifted.body.payload_data_class = "PUBLIC".to_string();
    assert_eq!(
        invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeFunctionsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-functions-idem-001".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_maps_unknown_inactive_and_duplicate_invocations() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let unknown = invoke_cloud_compute_function_from_api(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-functions-unknown",
            "idem-functions-unknown-001",
            "fninv_unknown",
        ),
    )
    .expect_err("unknown function is not found");
    assert_eq!(
        unknown,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::UnknownFunction)
    );
    assert_eq!(unknown.invoke_status_code(), 404);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);

    let mut inactive_catalog = CloudComputeCatalog::default();
    seed_deploying_function(&mut inactive_catalog);
    let mut inactive_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let inactive = invoke_cloud_compute_function_from_api(
        &mut inactive_catalog,
        &mut inactive_ledger,
        request(
            "req-compute-functions-inactive",
            "idem-functions-inactive-001",
            "fninv_inactive_api",
        ),
    )
    .expect_err("deploying functions cannot be invoked");
    assert_eq!(
        inactive,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::FunctionNotActive)
    );
    assert_eq!(inactive.invoke_status_code(), 409);
    assert_eq!(inactive_ledger.len(), 1);
    assert_eq!(inactive_catalog.invocations().count(), 0);

    let mut active_catalog = CloudComputeCatalog::default();
    seed_active_function(&mut active_catalog);
    let mut duplicate_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    invoke_cloud_compute_function_from_api(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-a",
            "idem-functions-duplicate-a",
            "fninv_duplicate",
        ),
    )
    .expect("first invoke succeeds");
    let duplicate = invoke_cloud_compute_function_from_api(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-b",
            "idem-functions-duplicate-b",
            "fninv_duplicate",
        ),
    )
    .expect_err("same invocation id with a new idempotency key conflicts");
    assert_eq!(
        duplicate,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::DuplicateInvocation)
    );
    assert_eq!(duplicate.invoke_status_code(), 409);
    assert_eq!(duplicate_ledger.len(), 2);
    assert_eq!(active_catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_maps_payload_data_class_policy_without_masking() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut denied = request(
        "req-compute-functions-payload-denied",
        "idem-functions-payload-denied-001",
        "fninv_payload_denied",
    );
    denied.body.payload_data_class = "PHI".to_string();

    let error = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, denied)
        .expect_err("payload class must be allowed by deployment policy");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::PayloadDataClassNotAllowed)
    );
    assert_eq!(error.invoke_status_code(), 403);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_unknown_payload_data_class_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-data-class",
        "idem-functions-data-class-001",
        "fninv_data_class",
    );
    request.body.payload_data_class = "NOT_A_DATA_CLASS".to_string();

    let error = invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, request)
        .expect_err("unknown payload data class label is rejected before invocation");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::InvalidPayloadDataClassLabel {
            payload_data_class: "NOT_A_DATA_CLASS".to_string(),
        }
    );
    assert_eq!(error.invoke_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}
