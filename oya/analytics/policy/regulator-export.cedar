// Cedar policy — regulator-export evidence pack authorization.
// Authority: ADR-0007 Cedar, ADR-0038 DSR cascade, ADR-0003 audit chain.
//
// Regulator-export is a high-privilege operation: it produces a cosign-signed
// pack containing tenant audit-log data over an extended window. Only a
// compliance officer for the tenant may initiate it; every initiation emits
// an audit-chain event with cosign-attestable signature.

@id("regulator-export-compliance-officer-for-tenant")
permit (
  principal in Role::"ComplianceOfficer",
  action == Action::"RegulatorExport",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};

@id("regulator-export-internal-compliance-for-any-tenant")
permit (
  principal in Role::"InternalComplianceOfficer",
  action == Action::"RegulatorExport",
  resource
) when {
  // Internal compliance officer (Oyatie-side) can export any tenant
  // ONLY when a court order or regulator request is recorded.
  context.justification_kind in ["court_order", "regulator_request"] &&
  context.justification_ref != ""
};

@id("regulator-export-emits-audit-event")
// This policy is enforced at the handler layer: it is non-blocking but
// recorded; if the audit event fails to emit, the operation aborts.
forbid (
  principal,
  action == Action::"RegulatorExport",
  resource
) unless {
  context.audit_event_will_emit == true
};

@id("regulator-export-no-cross-tenant-without-role")
forbid (
  principal,
  action == Action::"RegulatorExport",
  resource
) unless {
  principal.tenant_id == resource.tenant_id ||
  principal in Role::"InternalComplianceOfficer"
};
