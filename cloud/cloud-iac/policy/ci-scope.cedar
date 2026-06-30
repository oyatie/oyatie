// cloud-iac µservice — CI / applier-worker scope policy fragment
// Cedar v4
//
// Purpose: scope the iac-applier-worker, iac-rollback-worker, iac-validator-worker,
//          iac-renderer-worker, iac-registry-worker SPIFFE identities to their
//          respective mutation paths; forbid cross-µservice mutation.
// Owner:   axis-platform + axis-cloud-iac + ops-security
// References: iac-isolation.md §"Invariants ISO-01..ISO-06", §"Reserved scopes"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — iac-renderer-worker writes RenderCompleted events
// =============================================================================

permit (
  principal == Service::"iac-renderer-worker",
  action in [
    Action::"emit_render_completed",
    Action::"emit_render_failed",
    Action::"write_render_artifact_digest"
  ],
  resource is RenderEvent
);

// =============================================================================
// PERMIT 2 — iac-validator-worker writes ValidationVerdict + DriftReport
// =============================================================================

permit (
  principal == Service::"iac-validator-worker",
  action in [
    Action::"emit_validation_verdict",
    Action::"emit_drift_detected",
    Action::"write_drift_report"
  ],
  resource is ValidationEvent
);

// =============================================================================
// PERMIT 3 — iac-applier-worker applies within declared µservice apply-scope
// =============================================================================

permit (
  principal == Service::"iac-applier-worker",
  action in [
    Action::"apply_iac",
    Action::"emit_apply_started",
    Action::"emit_apply_completed",
    Action::"emit_apply_failed",
    Action::"write_apply_state",
    Action::"mutate_argo_application"
  ],
  resource is ApplyJob
)
when {
  resource has microservice &&
  resource has apply_scope &&
  resource.apply_scope.microservice == resource.microservice
};

// =============================================================================
// PERMIT 4 — iac-rollback-worker reverts within declared µservice scope
// =============================================================================

permit (
  principal == Service::"iac-rollback-worker",
  action in [
    Action::"rollback_iac",
    Action::"emit_apply_rolled_back",
    Action::"write_apply_state"
  ],
  resource is ApplyJob
)
when {
  resource has microservice &&
  resource has apply_scope &&
  resource.apply_scope.microservice == resource.microservice
};

// =============================================================================
// PERMIT 5 — iac-registry-worker maintains the iac-state-index + provenance store
// =============================================================================

permit (
  principal == Service::"iac-registry-worker",
  action in [
    Action::"write_apply_state",
    Action::"write_provenance",
    Action::"write_chart_record",
    Action::"write_module_record",
    Action::"write_overlay_record",
    Action::"emit_microservice_registered"
  ],
  resource is IacRegistryRecord
);

// =============================================================================
// FORBID — workers cannot mutate outside their own action scope
// =============================================================================

forbid (
  principal == Service::"iac-renderer-worker",
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"write_apply_state",
    Action::"mutate_argo_application"
  ],
  resource
);

forbid (
  principal == Service::"iac-validator-worker",
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"write_apply_state",
    Action::"mutate_argo_application"
  ],
  resource
);

forbid (
  principal == Service::"iac-applier-worker",
  action in [
    Action::"rollback_iac",
    Action::"write_provenance",
    Action::"emit_render_completed"
  ],
  resource
);

forbid (
  principal == Service::"iac-rollback-worker",
  action in [
    Action::"apply_iac",
    Action::"emit_render_completed",
    Action::"write_provenance"
  ],
  resource
);

forbid (
  principal == Service::"iac-registry-worker",
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"mutate_argo_application"
  ],
  resource
);

// =============================================================================
// FORBID — cross-µservice apply (ISO-03)
// =============================================================================
//
// Applies that reference resources outside the declared µservice apply-scope
// are refused at this layer. Cluster-RBAC + ArgoCD admission webhook are the
// other lines of defence (defence-in-depth).

forbid (
  principal in [
    Service::"iac-applier-worker",
    Service::"iac-rollback-worker"
  ],
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"emit_apply_started",
    Action::"emit_apply_completed",
    Action::"emit_apply_failed",
    Action::"emit_apply_rolled_back",
    Action::"write_apply_state",
    Action::"mutate_argo_application"
  ],
  resource is ApplyJob
)
when {
  resource has apply_scope &&
  resource has target_resources &&
  exists r in resource.target_resources {
    r has microservice &&
    r.microservice != resource.apply_scope.microservice
  }
};

// =============================================================================
// FORBID — reserved-scope (system:*) write from non-authorised SPIFFE
// =============================================================================
//
// system:cloud-iac, system:cloud-k8s, system:cloud-secrets, system:observability
// reserved scopes may only be applied by their owning µservice's SPIFFE identity.

forbid (
  principal,
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"emit_apply_started",
    Action::"emit_apply_completed",
    Action::"emit_apply_failed",
    Action::"emit_apply_rolled_back",
    Action::"write_apply_state",
    Action::"mutate_argo_application"
  ],
  resource is ApplyJob
)
when {
  resource has apply_scope &&
  resource.apply_scope.microservice like "system:*" &&
  !(
    (principal == Service::"iac-applier-worker" && resource.apply_scope.microservice == "system:cloud-iac")
    || (principal == Service::"iac-rollback-worker" && resource.apply_scope.microservice == "system:cloud-iac")
    || (principal == Service::"cloud-k8s-applier" && resource.apply_scope.microservice == "system:cloud-k8s")
    || (principal == Service::"cloud-secrets-applier" && resource.apply_scope.microservice == "system:cloud-secrets")
    || (principal == Service::"observability-applier" && resource.apply_scope.microservice == "system:observability")
  )
};

// =============================================================================
// FORBID — workers cannot read customer-tenant µservice SECRETs
// =============================================================================
//
// Worker access to SECRET-class material is via OpenBao SecretReference at apply
// time only; not via this Cedar policy.

forbid (
  principal in [
    Service::"iac-renderer-worker",
    Service::"iac-validator-worker",
    Service::"iac-applier-worker",
    Service::"iac-rollback-worker",
    Service::"iac-registry-worker"
  ],
  action in [
    Action::"read_secret_value",
    Action::"materialise_secret"
  ],
  resource
);
