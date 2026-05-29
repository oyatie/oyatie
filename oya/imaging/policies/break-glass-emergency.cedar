// Cedar policy: Emergency break-glass access for life-threatening situations.
//
// Authority: ADR-0243 + HIPAA 164.512(b) (treatment) + IHE EUA / ATNA audit.
//
// Principal: User with role "clinician".
// Action: imaging::Action::"BreakGlassReadStudy"
// Resource: imaging::Study
//
// Critical invariant: every break-glass access MUST emit an audit-chain
// entry within 1 second AND trigger a post-event review queue entry.

permit (
  principal in imaging::Role::"clinician",
  action == imaging::Action::"BreakGlassReadStudy",
  resource is imaging::Study
) when {
  principal.tenant_id == resource.tenant_id &&
  context.break_glass_reason in [
    "life-threatening-emergency",
    "code-blue",
    "code-stemi",
    "code-stroke",
    "code-trauma",
    "code-sepsis"
  ] &&
  context.attestation_clicked == true &&
  context.audit_chain_emit_committed == true &&
  context.post_event_review_enqueued == true &&
  principal.active &&
  !principal.suspended
};

// Forbid break-glass without explicit attestation + audit emission.
forbid (
  principal,
  action == imaging::Action::"BreakGlassReadStudy",
  resource
) when {
  context.attestation_clicked == false ||
  context.audit_chain_emit_committed == false ||
  context.post_event_review_enqueued == false
};

// Forbid break-glass for demo_trial tenants.
forbid (
  principal,
  action == imaging::Action::"BreakGlassReadStudy",
  resource is imaging::Study
) when {
  resource.tenant_class == imaging::TenantClass::"DEMO_TRIAL"
};

// Forbid break-glass when the actor has had >3 break-glass uses in the past
// 30 days without supervisory review (anti-abuse).
forbid (
  principal,
  action == imaging::Action::"BreakGlassReadStudy",
  resource
) when {
  principal.break_glass_count_last_30d > 3 &&
  principal.supervisor_review_completed == false
};
