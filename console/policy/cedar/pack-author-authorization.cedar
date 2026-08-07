// ops-dashboard-control-center — pack-author-authorization policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate compliance-pack authoring actions behind hardware-key step-up.
//          Pack authors can propose, review, and publish Cedar fragments + pack overlays.
//          Publishing requires hardware key (T3); proposal requires TOTP/passkey (T2).
//
// Owner:   ops-sre-reliability + council-compliance
// Binding ADR: ADR-0243, ADR-0251 (compliance packs), ADR-0294 (soak window)
// References: ARCHITECTURE.md §cedar-gates, compliance.md §pack-overlay-roster
// Soak window: ≥60s per ADR-0294

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — pack author read + propose (T2 step-up)
// =============================================================================

permit (
  principal in PackAuthor::?p,
  action in [
    Action::"pack_overlay_read",
    Action::"pack_overlay_propose",
    Action::"cedar_fragment_draft",
    Action::"pack_compliance_check_read"
  ],
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_TOTP_OR_PASSKEY", "STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  (context.now - principal.step_up_completed_at) < 14400 &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT — pack author publish (T3 hardware key, soak window verified)
// =============================================================================

permit (
  principal in PackAuthor::?p,
  action == Action::"pack_author_publish",
  resource in PackOverlay::?r
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  (context.now - principal.step_up_completed_at) < 3600 &&
  resource has soak_elapsed_seconds &&
  resource.soak_elapsed_seconds >= 60 &&
  resource has review_approved &&
  resource.review_approved == true &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT — pack author quarantine (suspend a pack overlay pending review)
// =============================================================================

permit (
  principal in PackAuthor::?p,
  action == Action::"pack_overlay_quarantine",
  resource in PackOverlay::?r
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true &&
  resource has quarantine_reason &&
  resource.quarantine_reason != ""
};

// =============================================================================
// FORBID — pack publish without soak window
// =============================================================================

forbid (
  principal,
  action == Action::"pack_author_publish",
  resource in PackOverlay::?r
)
when {
  !(resource has soak_elapsed_seconds) ||
  resource.soak_elapsed_seconds < 60
};

// =============================================================================
// FORBID — pack publish without review approval
// =============================================================================

forbid (
  principal,
  action == Action::"pack_author_publish",
  resource in PackOverlay::?r
)
when {
  !(resource has review_approved) ||
  resource.review_approved == false
};
