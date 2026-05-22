// api-gateway µservice — auditor-scope fragment
// Cedar v4.2 LTS
// Authority: ADR-0263 (observability emission) + ADR-0028 (audit chain).

// =============================================================================
// PERMIT — auditor reads audit events for their authorized scope
// =============================================================================

permit (
  principal in AuditorPrincipal::?p,
  action in [
    Action::"AuditEvent::read",
    Action::"AuditEvent::list",
    Action::"AuditEvent::export"
  ],
  resource in AuditEvent::?r
)
when {
  principal has tenant_scope &&
  resource has tenant_id &&
  principal.tenant_scope.contains(resource.tenant_id) &&
  principal has auditor_class &&
  principal.auditor_class in ["internal", "external-audit-firm", "regulator"]
};

// =============================================================================
// PERMIT — regulator audit (PIPC, EDPB, CAC, HHS, FedRAMP PMO)
// =============================================================================

permit (
  principal in AuditorPrincipal::?p,
  action == Action::"AuditEvent::regulator_export",
  resource in AuditEvent::?r
)
when {
  principal has auditor_class &&
  principal.auditor_class == "regulator" &&
  principal has regulator_authority &&
  principal.regulator_authority in [
    "kr-pipc",
    "eu-edpb",
    "cn-cac",
    "us-hhs",
    "us-fedramp-pmo",
    "us-fcc",
    "us-ftc",
    "in-mciti"
  ] &&
  resource has compliance_pack &&
  resource.compliance_pack matches principal.regulator_authority
};

// =============================================================================
// FORBID — auditor writes audit events
// =============================================================================
// Auditors are read-only by design.

forbid (
  principal in AuditorPrincipal::?p,
  action in [
    Action::"AuditEvent::create",
    Action::"AuditEvent::update",
    Action::"AuditEvent::delete"
  ],
  resource
);

// =============================================================================
// FORBID — auditor reads beyond their tenant scope
// =============================================================================

forbid (
  principal in AuditorPrincipal::?p,
  action,
  resource in AuditEvent::?r
)
unless {
  principal has tenant_scope &&
  resource has tenant_id &&
  principal.tenant_scope.contains(resource.tenant_id)
};
