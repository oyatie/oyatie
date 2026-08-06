// cloud-iac µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to iac/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-µservice (and downstream tenant) read scope on apply-state index,
//          drift reports, and provenance records served via iac-registry-rest, ArgoCD UI,
//          and SDK clients.
// Owner:   ops-security + axis-cloud-iac
// Audit:   every deny emits oya_cloud_iac_unauthorized_read_attempt_total
// References: iac-isolation.md §"Invariant ISO-07"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — µservice owner reads own µservice's apply state + drift + provenance
// =============================================================================
//
// A principal bound to MicroserviceOwner::"<microservice-id>" may read any
// ApplyStateRecord / DriftReport / ProvenanceRecord owned by that same µservice.

permit (
  principal in MicroserviceOwner::?p,
  action in [
    Action::"read_apply_state",
    Action::"read_drift_report",
    Action::"read_provenance",
    Action::"read_render_history",
    Action::"read_rollback_history",
    Action::"read_iac_manifests",
    Action::"read_argo_application_status"
  ],
  resource is ApplyStateRecord
)
when {
  principal has microservice_scope &&
  resource has microservice &&
  resource.microservice in principal.microservice_scope
};

// Same permit for DriftReport / ProvenanceRecord
permit (
  principal in MicroserviceOwner::?p,
  action in [Action::"read_drift_report", Action::"read_provenance"],
  resource
)
when {
  principal has microservice_scope &&
  resource has microservice &&
  resource.microservice in principal.microservice_scope
};

// =============================================================================
// PERMIT 2 — tenant operator reads own tenant's slice of own µservice's data
// =============================================================================
//
// Tenants do not directly query cloud-iac REST in normal flow (cloud-iac is internal
// substrate), but tenant-facing dashboards may surface filtered views of apply
// success/failure for the tenant's µservices.

permit (
  principal in TenantOperator::?t,
  action in [Action::"read_apply_state", Action::"read_drift_report"],
  resource
)
when {
  principal has tenant_id &&
  principal has microservice_scope &&
  resource has tenant_id &&
  resource has microservice &&
  principal.tenant_id == resource.tenant_id &&
  resource.microservice in principal.microservice_scope
};

// =============================================================================
// PERMIT 3 — µservice owner reads its own IaC manifest sources (gitops mirror)
// =============================================================================

permit (
  principal in MicroserviceOwner::?p,
  action == Action::"read_iac_manifests",
  resource
)
when {
  principal has microservice_scope &&
  resource has microservice &&
  resource.microservice in principal.microservice_scope &&
  resource has data_class &&
  resource.data_class == "INTERNAL_ONLY"
};

// =============================================================================
// FORBID — explicit cross-µservice deny (defence-in-depth)
// =============================================================================
//
// Cedar's deny-overrides semantics make this the safety net.

forbid (
  principal,
  action in [
    Action::"read_apply_state",
    Action::"read_drift_report",
    Action::"read_provenance",
    Action::"read_render_history",
    Action::"read_rollback_history",
    Action::"read_iac_manifests",
    Action::"read_argo_application_status"
  ],
  resource
)
unless {
  principal has microservice_scope &&
  resource has microservice &&
  resource.microservice in principal.microservice_scope
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant/owner reads
// =============================================================================
//
// SECRET-class data (cluster kubeconfigs, ArgoCD tokens, state encryption keys,
// OpenTofu state-bucket access keys) is out of scope for any read action defined here.
// Access is OpenBao + JIT elevation only.

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — write actions are out of tenant-scope path
// =============================================================================
//
// Authorship of IaC manifests is via git PR (CODEOWNERS + branch-protection enforced);
// Cedar policy at this layer does not authorise direct writes. Apply mutation
// is performed by the iac-applier-worker SPIFFE identity per ci-scope.cedar.

forbid (
  principal in MicroserviceOwner::?p,
  action in [
    Action::"write_apply_state",
    Action::"write_drift_report",
    Action::"write_provenance",
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"mutate_argo_application"
  ],
  resource
);

forbid (
  principal in TenantOperator::?t,
  action in [
    Action::"write_apply_state",
    Action::"write_drift_report",
    Action::"apply_iac",
    Action::"rollback_iac"
  ],
  resource
);

// =============================================================================
// FORBID — defence-in-depth on pack-pinning
// =============================================================================
//
// Reads that would surface data outside the principal's pack scope are refused
// even when µservice-scope matches. This enforces residency at the read layer.

forbid (
  principal,
  action in [
    Action::"read_apply_state",
    Action::"read_drift_report",
    Action::"read_provenance",
    Action::"read_render_history",
    Action::"read_rollback_history"
  ],
  resource
)
when {
  principal has pack_scope &&
  resource has pack &&
  !(resource.pack in principal.pack_scope)
};
