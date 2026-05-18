// application µservice — auditor scope policy fragment
// Cedar v4
//
// Purpose: govern external auditor (SOC2 / ISO / GDPR DPA / KR PIPC / HIPAA OCR)
//          read access to Application Shell audit + policy + evidence surfaces.
// Owner:   council-privacy + ops-security
// References: route-isolation.md; threat-model.md (external auditor actor)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads audit log (time-boxed JIT token; watermarked)
// =============================================================================
//
// Auditors receive a time-boxed JIT token via OpenBao at engagement start.
// All reads are watermarked (auditor_id + timestamp injected into rendered view).

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_audit",
    Action::"read_evidence",
    Action::"read_policy_fragment",
    Action::"read_route_registry",
    Action::"read_module_manifest"
  ],
  resource
)
when {
  principal has jit_token &&
  principal.jit_token.valid_until > context.now &&
  principal has tenant_scope &&
  resource has tenant_id &&
  (principal.tenant_scope == "all" ||
   resource.tenant_id in principal.tenant_scope) &&
  resource has data_class &&
  resource.data_class != "SECRET"
};

// =============================================================================
// PERMIT 2 — auditor exports audit log slice (watermarked)
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"export_audit",
  resource in AuditSlice::?s
)
when {
  principal has jit_token &&
  principal.jit_token.valid_until > context.now &&
  principal has export_scope &&
  resource has tenant_id &&
  resource.tenant_id in principal.export_scope &&
  resource has data_class &&
  (resource.data_class == "AUDIT" ||
   resource.data_class == "INTERNAL_ONLY")
};

// =============================================================================
// FORBID — auditor never writes
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_session",
    Action::"publish_module_manifest",
    Action::"render_route",
    Action::"load_module",
    Action::"purge_cdn"
  ],
  resource
);

// =============================================================================
// FORBID — auditor token expiry hard cap
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  !(principal has jit_token) ||
  principal.jit_token.valid_until <= context.now
};

// =============================================================================
// FORBID — auditor never reads SECRET-class
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
