// cloud-secrets µservice — external auditor scope policy fragment
// Cedar v4; conforms to schema.cedarschema
//
// Purpose: bound external auditor (SOC 2 / ISO 27001 / PCI-DSS QSA / HIPAA /
//          PIPC) access during a time-boxed audit window.
//
// Auditors read audit-chain events + policy + IaC + threat-model evidence;
// they NEVER read raw secrets or HSM-resident material.
//
// Owner:   ops-security + ops-legal + axis-governance
// References: threat-model.md "Actors" §"External auditor"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — auditor reads audit-chain export within their authorised window
// =============================================================================

permit (
  principal in ExternalAuditor::?p,
  action == Action::"read_audit_chain_export",
  resource in AuditChainEvent::?e
)
when {
  principal has audit_window_start &&
  principal has audit_window_end &&
  e has occurred_at &&
  principal.audit_window_start <= e.occurred_at &&
  e.occurred_at <= principal.audit_window_end &&
  principal has authorised_pack &&
  resource has pack &&
  principal.authorised_pack == resource.pack
};

// =============================================================================
// PERMIT 2 — auditor reads policy + IaC + threat-model + DPIA
// =============================================================================

permit (
  principal in ExternalAuditor::?p,
  action in [
    Action::"read_policy_fragment",
    Action::"read_iac_manifest",
    Action::"read_threat_model",
    Action::"read_dpia",
    Action::"read_runbook",
    Action::"read_rotation_policy",
    Action::"read_attestation_report",
    Action::"read_compliance_evidence"
  ],
  resource in Repo::?r
)
when {
  resource has microservice &&
  resource.microservice == "cloud-secrets"
};

// =============================================================================
// PERMIT 3 — auditor reads salted-hash → pack mapping for residency verification
// =============================================================================
//
// Auditors verify that tenants are pack-pinned correctly. They see the
// salted-hash tenant_id + assigned pack, NEVER the raw tenant_id mapping.

permit (
  principal in ExternalAuditor::?p,
  action == Action::"read_pack_assignment",
  resource in PackAssignment::?r
)
when {
  resource has data_class &&
  resource.data_class != "SECRET"
};

// =============================================================================
// FORBID — auditor never reads raw secrets
// =============================================================================

forbid (
  principal in ExternalAuditor::?p,
  action == Action::"resolve_secret_reference",
  resource
);

// =============================================================================
// FORBID — auditor never accesses HSM operations
// =============================================================================

forbid (
  principal in ExternalAuditor::?p,
  action in [
    Action::"hsm_sign",
    Action::"hsm_attest",
    Action::"hsm_generate_kek",
    Action::"hsm_rotate_kek",
    Action::"hsm_export_partition_state"
  ],
  resource
);

// =============================================================================
// FORBID — auditor never reads outside their authorised window or pack
// =============================================================================

forbid (
  principal in ExternalAuditor::?p,
  action,
  resource
)
when {
  principal has audit_window_start &&
  principal has audit_window_end &&
  resource has occurred_at &&
  (resource.occurred_at < principal.audit_window_start ||
   resource.occurred_at > principal.audit_window_end)
};

forbid (
  principal in ExternalAuditor::?p,
  action,
  resource
)
when {
  principal has authorised_pack &&
  resource has pack &&
  principal.authorised_pack != resource.pack
};

// =============================================================================
// FORBID — auditor token expiry (defence-in-depth)
// =============================================================================
//
// JIT token expiry is enforced at the IdP; this fragment is belt+suspenders.

forbid (
  principal in ExternalAuditor::?p,
  action,
  resource
)
when {
  principal has token_expires_at &&
  principal has now &&
  principal.token_expires_at <= principal.now
};
