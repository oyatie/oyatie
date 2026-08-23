use compute_domain::{
    CloudComputeError, ComputeTenantCellGuardrail, ComputeTenantCellGuardrailCreate,
    ComputeWorkloadIsolation,
};

fn guardrail_create() -> ComputeTenantCellGuardrailCreate {
    ComputeTenantCellGuardrailCreate {
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        primary_cell_id: "cell-region-alpha-a-001".to_string(),
        vm_instance_id: "oyatie:cloud:region-alpha:ten_alpha:instance:app-1".to_string(),
        vm_iam_role: "role_app".to_string(),
        vm_runtime_isolation: ComputeWorkloadIsolation::HardwareVirtualizedVm,
        k8s_cluster_id: "oyatie:cloud:region-alpha:ten_alpha:k8s:prod".to_string(),
        k8s_service_account_ref: "ksa/ten_alpha/cell-region-alpha-a-001/workload-runtime"
            .to_string(),
        k8s_private_control_plane: true,
        k8s_pod_security_restricted: true,
        k8s_topology_spread_required: true,
        k8s_runtime_isolation: ComputeWorkloadIsolation::KataMicroVm,
        function_id: "oyatie:cloud:region-alpha:ten_alpha:function:image-resize".to_string(),
        function_service_account_ref:
            "function-sa/ten_alpha/cell-region-alpha-a-001/image-resize-runtime".to_string(),
        function_runtime_isolation: ComputeWorkloadIsolation::FirecrackerMicroVm,
        audit_evidence_ref: "evidence/compute/audit/ten_alpha/cell-region-alpha-a-001/guardrail"
            .to_string(),
        scheduling_evidence_ref:
            "evidence/compute/scheduling/ten_alpha/cell-region-alpha-a-001/topology-spread"
                .to_string(),
        identity_evidence_refs: vec![
            "evidence/compute/identity/ten_alpha/vm-role-app".to_string(),
            "evidence/compute/identity/ten_alpha/k8s-workload-runtime".to_string(),
            "evidence/compute/identity/ten_alpha/function-image-resize".to_string(),
        ],
    }
}

#[test]
fn admits_vm_k8s_and_function_guardrail_metadata_only() {
    let guardrail =
        ComputeTenantCellGuardrail::new(guardrail_create()).expect("compute guardrail is valid");

    assert_eq!(guardrail.tenant_id.value, "ten_alpha");
    assert_eq!(guardrail.region.value.value, "region-alpha");
    assert_eq!(
        guardrail.vm_runtime_isolation.value,
        ComputeWorkloadIsolation::HardwareVirtualizedVm
    );
    assert_eq!(
        guardrail.k8s_runtime_isolation.value,
        ComputeWorkloadIsolation::KataMicroVm
    );
    assert_eq!(
        guardrail.function_runtime_isolation.value,
        ComputeWorkloadIsolation::FirecrackerMicroVm
    );
    assert!(guardrail.k8s_private_control_plane.value);
    assert!(guardrail.k8s_pod_security_restricted.value);
    assert!(guardrail.k8s_topology_spread_required.value);
    assert_eq!(guardrail.identity_evidence_refs.value.len(), 3);
}

#[test]
fn rejects_public_control_plane_unrestricted_pods_and_missing_spread_policy() {
    let public_control_plane = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_private_control_plane: false,
        ..guardrail_create()
    })
    .expect_err("Kubernetes tenant workloads require private control plane evidence");
    assert_eq!(
        public_control_plane,
        CloudComputeError::InvalidRuntimeIsolationPolicy
    );

    let unrestricted_pods = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_pod_security_restricted: false,
        ..guardrail_create()
    })
    .expect_err("Kubernetes tenant workloads require restricted pod security evidence");
    assert_eq!(
        unrestricted_pods,
        CloudComputeError::InvalidRuntimeIsolationPolicy
    );

    let missing_spread = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_topology_spread_required: false,
        ..guardrail_create()
    })
    .expect_err("Kubernetes tenant workloads require topology spread evidence");
    assert_eq!(missing_spread, CloudComputeError::InvalidSchedulingPolicy);
}

#[test]
fn rejects_weak_runtime_isolation_for_tenant_workloads() {
    let weak_vm = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        vm_runtime_isolation: ComputeWorkloadIsolation::HardenedNode,
        ..guardrail_create()
    })
    .expect_err("VM guardrail cannot rely on a hardened shared node only");
    assert_eq!(weak_vm, CloudComputeError::InvalidRuntimeIsolationPolicy);

    let weak_k8s = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_runtime_isolation: ComputeWorkloadIsolation::SharedHost,
        ..guardrail_create()
    })
    .expect_err("Kubernetes guardrail cannot run tenant pods on shared hosts");
    assert_eq!(weak_k8s, CloudComputeError::InvalidRuntimeIsolationPolicy);
}

#[test]
fn rejects_identity_refs_outside_tenant_cell_or_invalid_role() {
    let tenant_drift = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_service_account_ref: "ksa/ten_other/cell-region-alpha-a-001/workload-runtime"
            .to_string(),
        ..guardrail_create()
    })
    .expect_err("Kubernetes service account must remain tenant scoped");
    assert_eq!(
        tenant_drift,
        CloudComputeError::InvalidWorkloadIdentityPolicy
    );

    let cell_drift = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        function_service_account_ref:
            "function-sa/ten_alpha/cell-region-alpha-b-001/image-resize-runtime".to_string(),
        ..guardrail_create()
    })
    .expect_err("function service account must remain cell scoped");
    assert_eq!(cell_drift, CloudComputeError::InvalidWorkloadIdentityPolicy);

    let invalid_role = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        vm_iam_role: "app-role".to_string(),
        ..guardrail_create()
    })
    .expect_err("VM identity must be an IAM role reference, not free text");
    assert_eq!(
        invalid_role,
        CloudComputeError::InvalidWorkloadIdentityPolicy
    );
}

#[test]
fn rejects_secret_like_evidence_refs() {
    let audit_secret = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        audit_evidence_ref: "evidence/compute/audit/ten_alpha/password=supersecret".to_string(),
        ..guardrail_create()
    })
    .expect_err("audit evidence refs must not carry credentials");
    assert_eq!(audit_secret, CloudComputeError::InvalidAuditEvidenceRef);

    let identity_secret = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        identity_evidence_refs: vec![
            "evidence/compute/identity/ten_alpha/role-app".to_string(),
            "evidence/compute/identity/ten_alpha/private_key.pem".to_string(),
        ],
        ..guardrail_create()
    })
    .expect_err("identity evidence refs must be references only");
    assert_eq!(
        identity_secret,
        CloudComputeError::InvalidWorkloadIdentityPolicy
    );
}

#[test]
fn rejects_cross_tenant_or_cross_region_compute_resource_drift() {
    let foreign_vm = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        vm_instance_id: "oyatie:cloud:region-alpha:ten_other:instance:app-1".to_string(),
        ..guardrail_create()
    })
    .expect_err("VM instance id must match guardrail tenant");
    assert_eq!(foreign_vm, CloudComputeError::ResourceTenantMismatch);

    let foreign_k8s = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        k8s_cluster_id: "oyatie:cloud:region-beta:ten_alpha:k8s:prod".to_string(),
        ..guardrail_create()
    })
    .expect_err("Kubernetes cluster id must match guardrail region");
    assert_eq!(foreign_k8s, CloudComputeError::ResourceRegionMismatch);

    let wrong_kind = ComputeTenantCellGuardrail::new(ComputeTenantCellGuardrailCreate {
        function_id: "oyatie:cloud:region-alpha:ten_alpha:instance:not-a-function".to_string(),
        ..guardrail_create()
    })
    .expect_err("function guardrail resource must be a function id");
    assert_eq!(wrong_kind, CloudComputeError::ResourceKindMismatch);
}
