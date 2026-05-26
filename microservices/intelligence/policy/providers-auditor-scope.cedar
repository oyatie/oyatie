// foundry-providers µservice — auditor read scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: grant audit principals JIT read scope on ProviderInvoked events
//          and provider configurations during scheduled audits.
// Owner:   ops-compliance + ops-security
// Audit:   every auditor read emits AuditRead event; deny emits
//          oya_foundry_providers_auditor_unauthorized_read_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads scoped invocation events
// =============================================================================
//
// JIT-elevated auditor principal (per ADR-0028 audit-chain posture) may read
// invocation events for a tenant + pack + time-window combination during an
// active audit ticket.

permit (
  principal in Auditor::?p,
  action in [
    Action::"audit_read_invocation",
    Action::"audit_read_provider_config",
    Action::"audit_read_credential_audit_log",
    Action::"audit_read_router_decision_history"
  ],
  resource in AuditableEvent::?r
)
when {
  principal has audit_ticket_id &&
  principal has audit_ticket_active &&
  principal.audit_ticket_active == true &&
  principal has audit_scope_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.audit_scope_tenants &&
  principal has audit_scope_packs &&
  resource has pack &&
  resource.pack in principal.audit_scope_packs &&
  principal has audit_window_start &&
  principal has audit_window_end &&
  resource has occurred_at &&
  resource.occurred_at >= principal.audit_window_start &&
  resource.occurred_at <= principal.audit_window_end
};

// =============================================================================
// PERMIT 2 — auditor reads aggregate (non-tenant) substrate health
// =============================================================================

permit (
  principal in Auditor::?p,
  action in [
    Action::"audit_read_substrate_health",
    Action::"audit_read_self_slo"
  ],
  resource in AuditableEvent::?r
)
when {
  principal has audit_ticket_active &&
  principal.audit_ticket_active == true &&
  resource has data_class &&
  resource.data_class == "INTERNAL_ONLY"
};

// =============================================================================
// FORBID — never expose credential bytes to auditor
// =============================================================================
//
// Per credential-isolation.md CI-INV-03: audit records carry only credential_ref +
// lease_id; raw credential bytes are not present in any event.

forbid (
  principal,
  action,
  resource in AuditableEvent::?r
)
when {
  resource has data_class &&
  resource.data_class == "AUTH_CREDENTIAL"
};

// =============================================================================
// FORBID — auditor reads outside the audit ticket window
// =============================================================================

forbid (
  principal in Auditor::?p,
  action,
  resource
)
unless {
  principal has audit_ticket_active &&
  principal.audit_ticket_active == true
};
