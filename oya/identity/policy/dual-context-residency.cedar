// identity µservice — dual-context + residency policy
// Authority: ADR-0145 (purpose binding), ADR-0156 (data-class), ADR-0179 (per-pack residency)

// =============================================================================
// Purpose binding (ADR-0145 §"OIDC bearer claims"): every action requires
// purpose claim consistent with the action class.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"AuditExportAll",
    IdentityAction::"ListUsers",
    IdentityAction::"ScimListUsers"
  ],
  resource
) when {
  principal.purpose != "tenant-admin" &&
  principal.purpose != "audit-investigation" &&
  principal.purpose != "compliance-evidence"
};

// =============================================================================
// Data-class binding (ADR-0156): bearer's data_class claim must permit the
// data class being accessed.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"AuditExportAll",
    IdentityAction::"ListUsers"
  ],
  resource
) when {
  !(principal.data_class in [
    "PII_IDENTIFYING",
    "PII_QUASI_IDENTIFYING",
    "AUDIT"
  ])
};

// =============================================================================
// Per-pack residency (ADR-0179): principal's pack must match resource's pack.
// Cross-pack identity replication is forbidden by construction.
// =============================================================================

forbid (
  principal,
  action,
  resource
) when {
  principal.pack != resource.pack
};

// =============================================================================
// US-Healthcare (HIPAA) extra binding: BAA must be in force.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"ListUsers",
    IdentityAction::"ScimListUsers",
    IdentityAction::"AuditExportAll"
  ],
  resource
) when {
  resource.pack == "us-healthcare" &&
  !context.baa_in_force
};

// =============================================================================
// KR PIPA sensitive-info (Art. 23): pack-kr resources require explicit consent
// claim before sensitive-class queries.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"ListUsers",
    IdentityAction::"AuditExportAll"
  ],
  resource
) when {
  resource.pack == "kr" &&
  resource.data_class == "SENSITIVE_PIPA_ART23" &&
  !context.pipa_consent_present
};
