// Cedar policy — analytics µservice dashboard authorization.
// Authority: ADR-0007 Cedar fragment coverage, ADR-0193.

@id("analytics-dashboard-same-tenant")
permit (
  principal,
  action == Action::"ViewDashboard",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};

@id("analytics-audit-log-same-tenant")
permit (
  principal,
  action == Action::"QueryAuditLog",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};

@id("analytics-regulator-export-compliance-officer-only")
permit (
  principal in Role::"ComplianceOfficer",
  action == Action::"RegulatorExport",
  resource
) when {
  principal.tenant_id == resource.tenant_id ||
  principal.tenant_id == "internal_compliance"
};

// Default-deny for cross-tenant.
forbid (
  principal,
  action,
  resource
) unless {
  principal.tenant_id == resource.tenant_id ||
  principal in Role::"InternalAdmin"
};
