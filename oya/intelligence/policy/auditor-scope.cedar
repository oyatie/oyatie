// intelligence µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: time-boxed read access to per-call audit records for external auditors.
// Owner:   ops-security + council-privacy + ops-compliance
// References: observability/policy/auditor-scope.cedar (parallel pattern).

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — auditor reads tenant-scoped audit-tap records within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_audit_tap_record",
    Action::"read_dispatch_envelope_meta",
    Action::"read_refusal_decision",
    Action::"read_routing_decision",
    Action::"read_cost_record",
    Action::"read_artifact"
  ],
  resource
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT — auditor reads pack-level policy + DPIA + threat-model artifacts
// =============================================================================

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
    "threat-model", "dpia", "compliance", "tenant-isolation", "data-residency",
    "policy-cedar", "runbook", "adr", "prd", "architecture", "manifest"
  ]
};

// =============================================================================
// FORBID — auditor cannot write
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"dispatch",
    Action::"write_audit_tap_record",
    Action::"write_artifact",
    Action::"delete_audit_tap_record",
    Action::"rotate_secret"
  ],
  resource
);

// =============================================================================
// FORBID — auditor outside engagement window
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
// FORBID — auditor cannot access SECRET-class data
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
