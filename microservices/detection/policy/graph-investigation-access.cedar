// Detection Cedar policy: graph-investigation-access
// Binding ADRs: ADR-0243, ADR-0307, ADR-0308, ADR-0309, ADR-0310, ADR-0263.
// Lifecycle: Proposed -> Soaking -> Active -> Sunset per ADR-0294.
// Scope: tenant-scoped detection substrate action; default deny unless a permit below applies.

forbid (
  principal,
  action,
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in Detection::Role::"operator",
  action in Detection::Action::"graph-investigation-access",
  resource in Detection::Resource::"tenantScoped"
)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.compliance_pack in ["pack-us", "pack-eu-ai-act", "pack-kr"] &&
  context has trace_id &&
  context has audit_id
};

permit (
  principal in Detection::Role::"auditor",
  action == Detection::Action::"readAuditEvidence",
  resource in Detection::Resource::"caseEvidence"
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.purpose == "regulator-response" &&
  context.audit_event_class == "InvestigationPIIAccessed"
};
