// Cedar policy — dual-context authorization (caller + tenant context).
// Authority: ADR-0007 Cedar fragment coverage, ADR-0157 API gateway tier.
//
// Every request to the analytics µservice carries TWO authorization contexts:
//   1. Caller identity (principal) — who is asking.
//   2. Tenant context (resource.tenant_id) — what data are they asking about.
// Both must agree; misalignment is always a forbid.
//
// This policy is the analytics-µservice projection of the canonical
// dual-context pattern used fleet-wide per ADR-0157.

@id("dual-context-caller-tenant-must-match")
forbid (
  principal,
  action,
  resource
) when {
  // Anything that resolves a tenant resource: the caller's tenant must match.
  resource has tenant_id &&
  resource.tenant_id != principal.tenant_id &&
  !(principal in Role::"InternalAdmin") &&
  !(principal in Role::"InternalComplianceOfficer")
};

@id("dual-context-spiffe-bound")
forbid (
  principal,
  action,
  resource
) unless {
  // Every caller must present a valid SPIFFE id from the cell-local mesh.
  principal.spiffe_id != "" &&
  principal.spiffe_trust_domain == context.cell_trust_domain
};

@id("dual-context-residency-bound")
forbid (
  principal,
  action,
  resource
) when {
  // Tenant data may only be queried from a cell that matches the tenant's
  // residency class.
  resource has tenant_id &&
  resource has residency_class &&
  !context.cell_residency_compatible_with_class(resource.residency_class)
};

@id("dual-context-internal-admin-cross-tenant-with-justification")
permit (
  principal in Role::"InternalAdmin",
  action,
  resource
) when {
  // Internal admin reads across tenants ONLY with a recorded justification.
  context.justification_ref != "" &&
  context.audit_event_will_emit == true
};
