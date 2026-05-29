// recordings µservice — legal-hold policy fragment
// Cedar v4.2
//
// Purpose: enforce the load-bearing 100 %-correctness invariant for legal hold.
//          Defence-in-depth: kernel-layer pessimistic lock + Cedar refusal.
// Owner:   ops-compliance + axis-recordings + council-privacy

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — engaging or releasing a hold requires compliance_officer + four-eyes
// =============================================================================

permit (
  principal,
  action in [Action::"engage_legal_hold", Action::"release_legal_hold"],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has roles &&
  "compliance_officer" in principal.roles &&
  // Four-eyes pair must be present in context
  context has paired_approver_ref &&
  context.paired_approver_ref != principal.user_ref &&
  context has paired_approver_roles &&
  "compliance_officer" in context.paired_approver_roles &&
  // Court order reference must be present for engage
  (
    action == Action::"release_legal_hold" ||
    (
      context has court_order_ref &&
      context.court_order_ref != ""
    )
  )
};

// =============================================================================
// FORBID — retention purge against a recording with legal_hold_engaged == true
// =============================================================================
// This is the **load-bearing** policy. Any breach is a Sev-1 audit event.

forbid (
  principal,
  action in [
    Action::"delete_recording",
    Action::"purge_retention",
    Action::"kms_shred"
  ],
  resource
)
when {
  resource has legal_hold_engaged &&
  resource.legal_hold_engaged == true
};

// =============================================================================
// FORBID — modifying a held recording's retention policy
// =============================================================================

forbid (
  principal,
  action in [
    Action::"update_retention_policy",
    Action::"reduce_retention_floor"
  ],
  resource
)
when {
  resource has legal_hold_engaged &&
  resource.legal_hold_engaged == true
};

// =============================================================================
// FORBID — redaction overlay compensation (un-redact) on held recordings
// without paired approver
// =============================================================================

forbid (
  principal,
  action == Action::"compensate_redaction",
  resource
)
when {
  resource has legal_hold_engaged &&
  resource.legal_hold_engaged == true &&
  (
    !(context has paired_approver_ref) ||
    context.paired_approver_ref == principal.user_ref
  )
};
