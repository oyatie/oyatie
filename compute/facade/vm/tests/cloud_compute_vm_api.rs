// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use compute_domain::{CloudComputeCatalog, CloudComputeError};
use compute_vm_api::{
    CLOUD_COMPUTE_VM_CREATE_SURFACE, CloudComputeVmApiAuthorization,
    CloudComputeVmApiAuthorizationProof, CloudComputeVmApiBoundaryContext, CloudComputeVmApiError,
    CloudComputeVmApiPrincipal, CloudComputeVmCreateApiRequest, CloudComputeVmCreateApiStatus,
    CloudComputeVmCreateIdempotencyLedger, CloudComputeVmCreateRequest,
    CloudComputeVmCreateSuccessResponse, CloudComputeVmFlavorSpec, CloudComputeVmIamRoleRef,
    CloudComputeVmQuotaEnvelope, CloudComputeVmSecurityGroupRef,
    CloudComputeVmTrustedAuthorizationVerifier, create_cloud_compute_vm_from_api,
    create_cloud_compute_vm_from_api_with_verifier,
};

const INSTANCE_ID: &str = "oyatie:cloud:region-home:ten_alpha:instance:app-1";
const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VERIFIER_EVALUATION_EPOCH_SECONDS: u64 = 1_700_099_500;

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudComputeVmApiBoundaryContext {
    CloudComputeVmApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudComputeVmApiPrincipal {
    CloudComputeVmApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudComputeVmApiAuthorization {
    let decision_id = format!("authz_decision_{principal_id}");
    CloudComputeVmApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.clone(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
        proof: Some(authorization_proof_for(
            principal_id,
            CLOUD_COMPUTE_VM_CREATE_SURFACE,
            &decision_id,
        )),
    }
}
fn authorization_proof_for(
    principal_id: &str,
    surface: &str,
    decision_id: &str,
) -> CloudComputeVmApiAuthorizationProof {
    CloudComputeVmApiAuthorizationProof {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        surface: surface.to_string(),
        decision_id: decision_id.to_string(),
        verified: true,
        issued_at_epoch_seconds: 1_700_099_000,
        expires_at_epoch_seconds: 1_700_100_000,
    }
}

fn trusted_verifier_for(
    request: &CloudComputeVmCreateApiRequest,
) -> CloudComputeVmTrustedAuthorizationVerifier {
    CloudComputeVmTrustedAuthorizationVerifier::new(VERIFIER_EVALUATION_EPOCH_SECONDS)
        .with_trusted_proof(authorization_proof_for(
            &request.principal.principal_id,
            CLOUD_COMPUTE_VM_CREATE_SURFACE,
            &request.authorization.decision_id,
        ))
}

fn trusted_verifier_with(
    proof: CloudComputeVmApiAuthorizationProof,
) -> CloudComputeVmTrustedAuthorizationVerifier {
    CloudComputeVmTrustedAuthorizationVerifier::new(VERIFIER_EVALUATION_EPOCH_SECONDS)
        .with_trusted_proof(proof)
}

fn create_vm_with_trusted_verifier(
    catalog: &mut CloudComputeCatalog,
    ledger: &mut CloudComputeVmCreateIdempotencyLedger,
    request: CloudComputeVmCreateApiRequest,
) -> Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError> {
    let verifier = trusted_verifier_for(&request);
    create_cloud_compute_vm_from_api_with_verifier(catalog, ledger, request, &verifier)
}

fn authorization_denied() -> CloudComputeVmApiError {
    CloudComputeVmApiError::AuthorizationDenied {
        surface: CLOUD_COMPUTE_VM_CREATE_SURFACE.to_string(),
    }
}

fn quota() -> CloudComputeVmQuotaEnvelope {
    CloudComputeVmQuotaEnvelope {
        vcpu_limit: 128,
        memory_gb_limit: 512,
        gpu_limit: 8,
        local_ssd_gb_limit: 4_096,
        current_vcpu: 4,
        current_memory_gb: 16,
        current_gpu: 0,
        current_local_ssd_gb: 100,
    }
}

fn flavor() -> CloudComputeVmFlavorSpec {
    CloudComputeVmFlavorSpec {
        class: "general_purpose".to_string(),
        vcpu: 4,
        memory_gb: 16,
        gpu_count: 0,
        local_ssd_gb: 100,
    }
}

fn body(resource_id: &str) -> CloudComputeVmCreateRequest {
    CloudComputeVmCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        az: "region-home-a".to_string(),
        cell_id: "cell-region-home-a-001".to_string(),
        flavor: flavor(),
        image: format!("oci://harbor.region-home.oyatie.io/ten_alpha/app@sha256:{DIGEST}"),
        key_pair: Some("key_prod".to_string()),
        vpc_id: "oyatie:cloud:region-home:ten_alpha:vpc:prod".to_string(),
        subnet_id: "oyatie:cloud:region-home:ten_alpha:subnet:prod-a".to_string(),
        security_groups: vec![
            CloudComputeVmSecurityGroupRef {
                value: "sg_web".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-home".to_string(),
                vpc_id: "oyatie:cloud:region-home:ten_alpha:vpc:prod".to_string(),
            },
            CloudComputeVmSecurityGroupRef {
                value: "sg_app".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-home".to_string(),
                vpc_id: "oyatie:cloud:region-home:ten_alpha:vpc:prod".to_string(),
            },
        ],
        iam_role: Some(CloudComputeVmIamRoleRef {
            value: "role_app".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-home".to_string(),
            vpc_id: "oyatie:cloud:region-home:ten_alpha:vpc:prod".to_string(),
        }),
        user_data_uri: Some("userdata/ten_alpha/app-1/cloud-init.yaml".to_string()),
        quota: quota(),
        residency: "strict_home_region".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_100_000,
    }
}

fn request(request_id: &str, idempotency_key: &str) -> CloudComputeVmCreateApiRequest {
    CloudComputeVmCreateApiRequest {
        path_instance_id: INSTANCE_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: authorization_for("sp_compute", &[CLOUD_COMPUTE_VM_CREATE_SURFACE]),
        body: body(INSTANCE_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_COMPUTE_VM_CREATE_SURFACE, "cloud.compute.vm.create");
    assert_eq!(CloudComputeVmCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudComputeVmCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudComputeVmCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudComputeVmCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeVmCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeVmCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeVmCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn vm_create_api_creates_instance_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-create", "idem-compute-vm-create");

    let first = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("authorized VM create succeeds");
    let second = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-vm-create");
    assert_eq!(first.data.resource_id, INSTANCE_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.az, "region-home-a");
    assert_eq!(first.data.cell_id, "cell-region-home-a-001");
    assert_eq!(first.data.flavor.class, "general_purpose");
    assert_eq!(first.data.flavor.vcpu, 4);
    assert_eq!(first.data.image_kind, "oci");
    assert_eq!(first.data.residency, "strict_home_region");
    assert_eq!(first.data.state, "pending");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn vm_create_api_uses_trusted_verifier_and_ignores_caller_supplied_proof_fields() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request(
        "req-compute-vm-ignore-caller-proof",
        "idem-compute-vm-ignore-caller-proof",
    );
    request.authorization.tenant_id = "ten_forged".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.allowed_surfaces = vec!["cloud.compute.k8s.cluster.create".to_string()];
    let mut forged_proof = authorization_proof_for(
        "sp_attacker",
        "cloud.compute.k8s.cluster.create",
        "authz_decision_attacker",
    );
    forged_proof.tenant_id = "ten_other".to_string();
    forged_proof.expires_at_epoch_seconds = forged_proof.issued_at_epoch_seconds;
    request.authorization.proof = Some(forged_proof);

    let response = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("trusted verifier state authorizes independently of caller proof fields");

    assert_eq!(response.data.resource_id, INSTANCE_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-drift", "idem-compute-vm-drift");
    request.body.resource_id = "oyatie:cloud:region-home:ten_alpha:instance:other".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("path/body instance drift is rejected");

    assert_eq!(
        error,
        CloudComputeVmApiError::InstanceIdMismatch {
            path_instance_id: INSTANCE_ID.to_string(),
            body_resource_id: "oyatie:cloud:region-home:ten_alpha:instance:other".to_string(),
        }
    );
    assert_eq!(error.vm_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut empty_request = request(" ", "idem-compute-vm-empty-header");
    assert_eq!(
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, empty_request.clone()),
        Err(CloudComputeVmApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-compute-vm-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, empty_request)
        .expect_err("tenant drift is rejected before idempotency ledger write");

    assert_eq!(error.vm_create_status_code(), 403);
    assert!(matches!(
        error,
        CloudComputeVmApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_trusted_verifier_surface_mismatch_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-authz", "idem-compute-vm-authz");
    let verifier = trusted_verifier_with(authorization_proof_for(
        "sp_compute",
        "cloud.compute.k8s.cluster.create",
        &request.authorization.decision_id,
    ));

    let error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect_err("trusted verifier decision does not allow VM create");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_legacy_entrypoint_fails_closed_without_verifier_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-vm-missing-verifier",
        "idem-compute-vm-missing-verifier",
    );

    let error = create_cloud_compute_vm_from_api(&mut catalog, &mut ledger, request)
        .expect_err("legacy VM create entrypoint must not trust caller-supplied proof");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_trusted_verifier_tenant_principal_and_decision_mismatch() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();

    let tenant_request = request(
        "req-compute-vm-tenant-authz",
        "idem-compute-vm-tenant-authz",
    );
    let mut tenant_proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &tenant_request.authorization.decision_id,
    );
    tenant_proof.tenant_id = "ten_other".to_string();
    let tenant_verifier = trusted_verifier_with(tenant_proof);
    let tenant_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        tenant_request,
        &tenant_verifier,
    )
    .expect_err("trusted verifier proof bound to another tenant is rejected");
    assert_eq!(tenant_error, authorization_denied());

    let principal_request = request(
        "req-compute-vm-principal-authz",
        "idem-compute-vm-principal-authz",
    );
    let principal_verifier = trusted_verifier_with(authorization_proof_for(
        "sp_other",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &principal_request.authorization.decision_id,
    ));
    let principal_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        principal_request,
        &principal_verifier,
    )
    .expect_err("trusted verifier proof bound to another principal is rejected");
    assert_eq!(principal_error, authorization_denied());

    let decision_request = request(
        "req-compute-vm-decision-authz",
        "idem-compute-vm-decision-authz",
    );
    let decision_verifier = trusted_verifier_with(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        "authz_decision_other",
    ));
    let decision_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        decision_request,
        &decision_verifier,
    )
    .expect_err("trusted verifier state is keyed by the requested decision id");
    assert_eq!(decision_error, authorization_denied());

    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_unverified_or_expired_trusted_verifier_proof_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();

    let unverified_request = request(
        "req-compute-vm-unverified-authz",
        "idem-compute-vm-unverified-authz",
    );
    let mut unverified_proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &unverified_request.authorization.decision_id,
    );
    unverified_proof.verified = false;
    let unverified_verifier = trusted_verifier_with(unverified_proof);
    let unverified_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        unverified_request,
        &unverified_verifier,
    )
    .expect_err("unverified trusted verifier proof is rejected");
    assert_eq!(unverified_error, authorization_denied());
    assert_eq!(unverified_error.vm_create_status_code(), 403);
    let request = request(
        "req-compute-vm-expired-authz",
        "idem-compute-vm-expired-authz",
    );
    let mut proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &request.authorization.decision_id,
    );
    proof.expires_at_epoch_seconds = VERIFIER_EVALUATION_EPOCH_SECONDS;
    let verifier = trusted_verifier_with(proof);

    let error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect_err("expired trusted verifier proof is rejected");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-authn", "idem-compute-vm-authn");
    request.principal.principal_id = " ".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(error, CloudComputeVmApiError::EmptyPrincipalId);
    assert_eq!(error.vm_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_replays_with_refreshed_authz_and_reordered_security_groups() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-vm-authz-refresh-1",
        "idem-compute-vm-authz-refresh",
    );
    let first = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial VM create succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-vm-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &retry.authorization.decision_id,
    ));
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.k8s.cluster.create".to_string(),
        CLOUD_COMPUTE_VM_CREATE_SURFACE.to_string(),
    ];
    retry.body.security_groups.reverse();
    let second = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_rejects_foreign_security_group_and_iam_role_proofs_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut group_request = request("req-compute-vm-sg-proof", "idem-compute-vm-sg-proof");
    group_request.body.security_groups[0].tenant_id = "ten_other".to_string();

    let group_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, group_request)
        .expect_err("security group proof must match tenant boundary");

    assert!(matches!(
        group_error,
        CloudComputeVmApiError::SecurityGroupBindingMismatch { .. }
    ));
    assert_eq!(group_error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);

    let mut role_request = request("req-compute-vm-role-proof", "idem-compute-vm-role-proof");
    role_request
        .body
        .iam_role
        .as_mut()
        .expect("role ref exists")
        .vpc_id = "oyatie:cloud:region-home:ten_other:vpc:foreign".to_string();
    let role_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, role_request)
        .expect_err("IAM role proof must match VPC boundary");

    assert!(matches!(
        role_error,
        CloudComputeVmApiError::IamRoleBindingMismatch { .. }
    ));
    assert_eq!(role_error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-idem", "idem-compute-vm-idem");
    create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial VM create succeeds");

    let mut drifted = request;
    drifted.body.flavor.memory_gb = 32;
    assert_eq!(
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeVmApiError::IdempotencyKeyReused {
            idempotency_key: "idem-compute-vm-idem".to_string(),
        })
    );
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_maps_duplicate_instance_to_conflict() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-compute-vm-dup-1", "idem-compute-vm-dup-1"),
    )
    .expect("first VM create succeeds");

    let error = create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-compute-vm-dup-2", "idem-compute-vm-dup-2"),
    )
    .expect_err("same instance id through new idempotency key is a conflict");

    assert_eq!(
        error,
        CloudComputeVmApiError::Compute(CloudComputeError::DuplicateInstance)
    );
    assert_eq!(error.vm_create_status_code(), 409);
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_maps_quota_residency_and_invalid_image_without_masking() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut quota_request = request("req-compute-vm-quota", "idem-compute-vm-quota");
    quota_request.body.quota.vcpu_limit = 6;
    let quota_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, quota_request)
        .expect_err("tenant quota is enforced");
    assert_eq!(
        quota_error,
        CloudComputeVmApiError::Compute(CloudComputeError::QuotaExceeded)
    );
    assert_eq!(quota_error.vm_create_status_code(), 403);

    let mut residency_request = request("req-compute-vm-residency", "idem-compute-vm-residency");
    residency_request.body.region = "failover-region".to_string();
    residency_request.body.az = "failover-region-a".to_string();
    residency_request.body.cell_id = "cell-failover-region-a-001".to_string();
    residency_request.body.resource_id =
        "oyatie:cloud:failover-region:ten_alpha:instance:app-1".to_string();
    residency_request.path_instance_id = residency_request.body.resource_id.clone();
    residency_request.body.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    residency_request.body.subnet_id =
        "oyatie:cloud:failover-region:ten_alpha:subnet:prod-a".to_string();
    for group in &mut residency_request.body.security_groups {
        group.region = "failover-region".to_string();
        group.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    }
    if let Some(role) = &mut residency_request.body.iam_role {
        role.region = "failover-region".to_string();
        role.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    }
    let residency_error =
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, residency_request)
            .expect_err("strict home-region residency denies US VM placement");
    assert_eq!(
        residency_error,
        CloudComputeVmApiError::Compute(CloudComputeError::ResidencyRegionMismatch)
    );
    assert_eq!(residency_error.vm_create_status_code(), 403);

    let mut image_request = request("req-compute-vm-image", "idem-compute-vm-image");
    image_request.body.image = "oci://harbor.region-home.oyatie.io/ten_alpha/app:latest".to_string();
    let image_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, image_request)
        .expect_err("image refs must be digest pinned");
    assert_eq!(
        image_error,
        CloudComputeVmApiError::Compute(CloudComputeError::InvalidImageRef)
    );
    assert_eq!(image_error.vm_create_status_code(), 400);
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_unknown_data_class_label_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-class", "idem-compute-vm-class");
    request.body.data_class = "SECRET".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("operational markers are not VM API data classes");

    assert_eq!(
        error,
        CloudComputeVmApiError::InvalidDataClassLabel {
            data_class: "SECRET".to_string(),
        }
    );
    assert_eq!(error.vm_create_status_code(), 400);
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_idempotency_ledger_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::with_max_entries(1);

    create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-vm-bound-1", "idem-vm-bound-1"),
    )
    .expect("first create succeeds");
    let mut second = request("req-vm-bound-2", "idem-vm-bound-2");
    second.path_instance_id = "oyatie:cloud:region-home:ten_alpha:instance:app-2".to_string();
    second.body.resource_id = second.path_instance_id.clone();
    create_vm_with_trusted_verifier(&mut catalog, &mut ledger, second)
        .expect("second create succeeds");

    assert_eq!(ledger.len(), 1);

    let replay_error = create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-vm-bound-replay", "idem-vm-bound-1"),
    )
    .expect_err(
        "evicted idempotency key is no longer replayable and reaches duplicate resource guard",
    );

    assert_eq!(
        replay_error,
        CloudComputeVmApiError::Compute(CloudComputeError::DuplicateInstance)
    );
}
