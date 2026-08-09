// cloud-iac µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA / SLSA-L3 / etc.)
//          read time-boxed, µservice-scoped subsets of the apply-state index, drift reports,
//          provenance records, and audit-chain events when an engagement is active.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05, T-L-05; iac-isolation.md §"Audit Trail"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1a — auditor reads µservice-scoped apply state within engagement window
// =============================================================================
//
// Auditor principals are issued JIT tokens by OpenBao at engagement start.
// The token carries:
//   - engagement_id        unique per audit cycle
//   - scoped_microservices list of microservice_ids in scope
//   - scoped_packs         list of pack_ids in scope
//   - valid_from / valid_to  ISO8601 timestamps
//   - audit_framework      one of [soc2, iso27001, hipaa, gdpr-dpa, kr-pipc, slsa-l3, ...]

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_apply_state",
    Action::"read_argo_application_status"
  ],
  resource is ApplyStateRecord
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 1b — auditor reads µservice-scoped drift reports within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_drift_report",
  resource is DriftReport
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 1c — auditor reads provenance and attestation evidence within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_provenance",
    Action::"read_slsa_attestation",
    Action::"read_cosign_verification_log"
  ],
  resource is ProvenanceRecord
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 1d — auditor reads render and rollback histories within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_render_history",
  resource is RenderHistory
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

permit (
  principal in Auditor::?a,
  action == Action::"read_rollback_history",
  resource is RollbackHistory
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 1e — auditor reads IaC manifests and audit-chain records within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_iac_manifests",
  resource is IacManifest
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

permit (
  principal in Auditor::?a,
  action == Action::"read_audit_chain",
  resource is AuditChainRecord
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has microservice &&
  principal has scoped_microservices &&
  resource.microservice in principal.scoped_microservices &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 2 — auditor reads policy + compliance artifacts (non-tenant-scoped)
// =============================================================================
//
// Threat-model, DPIA, compliance, runbooks, ADRs, PRDs, Cedar fragments themselves.

permit (
  principal in Auditor::?a,
  action == Action::"read_artifact",
  resource in PolicyArtifact::?r
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has artifact_type &&
  resource.artifact_type in [
    "threat-model",
    "dpia",
    "compliance",
    "iac-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "adr",
    "prd",
    "phase-spec",
    "implementation-plan",
    "capacity-model",
    "cost-budget",
    "multi-region",
    "failure-modes",
    "incident-response",
    "backfill-replay",
    "sdk-plan",
    "competitor-parity-matrix"
  ]
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"write_apply_state",
    Action::"write_drift_report",
    Action::"write_provenance",
    Action::"write_artifact",
    Action::"delete_apply_state",
    Action::"delete_drift_report",
    Action::"delete_provenance",
    Action::"delete_artifact",
    Action::"mutate_argo_application",
    Action::"rotate_secret"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read outside engagement window (defence-in-depth)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped_microservices
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_apply_state",
    Action::"read_drift_report",
    Action::"read_provenance",
    Action::"read_render_history",
    Action::"read_rollback_history",
    Action::"read_iac_manifests",
    Action::"read_audit_chain",
    Action::"read_slsa_attestation"
  ],
  resource
)
when {
  resource has microservice &&
  principal has scoped_microservices &&
  !(resource.microservice in principal.scoped_microservices)
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped_packs (residency-bounded audit)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has pack &&
  principal has scoped_packs &&
  !(resource.pack in principal.scoped_packs)
};

// =============================================================================
// FORBID — auditor cannot access SECRET-class data
// =============================================================================
//
// Auditors do not read raw kubeconfigs, signing keys, ArgoCD admin tokens,
// or OpenTofu state encryption keys. Audit evidence is derived facts (audit-chain
// records, provenance attestations), not raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
