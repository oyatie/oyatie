// mail µservice — auditor-scope policy fragment
// Cedar v4 schema
//
// Purpose: scope external auditor (SOC 2 / ISO 27001 / GDPR DPA / KR PIPC / HIPAA OCR /
//          KR-FSS / ANPD / etc.) read access during time-boxed engagement window.
//          Auditor sees audit-chain seals, retention ledger, configuration evidence,
//          and tenant-scoped operational metrics. Auditor does NOT see mail content.
// Owner:   ops-security + council-privacy + ops-legal
// Audit:   every auditor read emits mail_auditor_read_total + audit-chain seal-of-audit
// References: threat-model.md T-S-05 + dpia.md R-16
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads audit-chain seals for engagement-scoped tenants
// =============================================================================

permit (
  principal in ExternalAuditor::?a,
  action == Action::"read_audit_chain_seals",
  resource in AuditChainSeal::?s
)
when {
  principal has engagement_id &&
  principal has engagement_tenant_scope &&
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  resource has tenant_id &&
  resource has emitted_at &&
  resource.tenant_id in principal.engagement_tenant_scope &&
  resource.emitted_at >= principal.engagement_window_start &&
  resource.emitted_at <= principal.engagement_window_end
};

// =============================================================================
// PERMIT 2 — auditor reads retention ledger entries (engagement-scoped)
// =============================================================================

permit (
  principal in ExternalAuditor::?a,
  action == Action::"read_retention_ledger",
  resource in RetentionLedgerEntry::?r
)
when {
  principal has engagement_tenant_scope &&
  resource has tenant_id &&
  resource.tenant_id in principal.engagement_tenant_scope
};

// =============================================================================
// PERMIT 3 — auditor reads tenant-scope DKIM/SPF/DMARC config
// =============================================================================

permit (
  principal in ExternalAuditor::?a,
  action in [
    Action::"read_dkim_config",
    Action::"read_mta_sts_policy",
    Action::"read_tls_rpt_endpoint",
    Action::"read_retention_policy",
    Action::"read_legal_holds_metadata"   // hold scope + approval, not content
  ],
  resource in TenantMailConfig::?c
)
when {
  principal has engagement_tenant_scope &&
  resource has tenant_id &&
  resource.tenant_id in principal.engagement_tenant_scope
};

// =============================================================================
// PERMIT 4 — auditor reads continuous-compliance evidence artifacts
// =============================================================================

permit (
  principal in ExternalAuditor::?a,
  action == Action::"read_compliance_evidence",
  resource in ComplianceEvidence::?e
)
when {
  principal has engagement_framework &&
  resource has framework &&
  principal.engagement_framework == resource.framework
};

// =============================================================================
// FORBID — auditor NEVER reads mail content (Professional or Personal)
// =============================================================================
//
// Auditor sees metadata + evidence; never sees mailbox body, headers, attachments,
// or search index plaintext (which doesn't exist anyway per Invariant DCI-05).

forbid (
  principal in ExternalAuditor::?a,
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"read_mime_blob",
    Action::"search",
    Action::"download_ediscovery_export"
  ],
  resource
);

// =============================================================================
// FORBID — auditor reads outside engagement window
// =============================================================================

forbid (
  principal in ExternalAuditor::?a,
  action,
  resource
)
when {
  principal has engagement_window_end &&
  context has now &&
  context.now > principal.engagement_window_end
};

// =============================================================================
// FORBID — auditor reads tenants outside engagement scope
// =============================================================================

forbid (
  principal in ExternalAuditor::?a,
  action,
  resource
)
when {
  principal has engagement_tenant_scope &&
  resource has tenant_id &&
  !(resource.tenant_id in principal.engagement_tenant_scope)
};

// =============================================================================
// FORBID — auditor never sees Personal-pillar resources at all
// =============================================================================

forbid (
  principal in ExternalAuditor::?a,
  action,
  resource
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};
