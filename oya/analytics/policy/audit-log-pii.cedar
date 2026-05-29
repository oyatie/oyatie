// Cedar policy — audit-log PII column filtering.
// Authority: ADR-0007 Cedar, ADR-0008 DUBO, ADR-0156 PII registry, ADR-0003 audit chain.
//
// PII-tagged columns in the audit-log surface are filtered unless the principal
// holds an explicit DUBO grant for the PII data class. This is the per-column
// defense-in-depth layer behind the per-tenant database isolation.

@id("audit-log-pii-deny-by-default")
forbid (
  principal,
  action == Action::"QueryAuditLog",
  resource
) when {
  resource.column_data_class in [DataClass::"PII", DataClass::"PHI", DataClass::"PCI"] &&
  !principal.dubo_grants.contains(resource.column_data_class)
};

@id("audit-log-pii-permit-with-grant")
permit (
  principal,
  action == Action::"QueryAuditLog",
  resource
) when {
  resource.column_data_class == DataClass::"PII" &&
  principal.dubo_grants.contains(DataClass::"PII") &&
  principal.tenant_id == resource.tenant_id
};

@id("audit-log-pii-internal-compliance-officer")
permit (
  principal in Role::"ComplianceOfficer",
  action == Action::"QueryAuditLog",
  resource
) when {
  // Compliance officer can read PII on regulator-export path only.
  resource.column_data_class == DataClass::"PII" &&
  context.purpose == "regulator_export" &&
  context.audit_event_id != ""
};

@id("audit-log-pii-no-cross-tenant")
forbid (
  principal,
  action == Action::"QueryAuditLog",
  resource
) unless {
  principal.tenant_id == resource.tenant_id ||
  principal in Role::"InternalAdmin"
};
