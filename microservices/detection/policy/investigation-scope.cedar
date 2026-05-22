// Detection Cedar policy: investigation-scope
// Binding ADRs: ADR-0243, ADR-0307, ADR-0310, ADR-0263.

forbid (principal, action, resource)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in Detection::Role::"investigator",
  action == Detection::Action::"investigation-bridge",
  resource in Detection::Resource::"caseEvidence"
)
when {
  principal.tenant_id == resource.tenant_id &&
  context has trace_id &&
  context has audit_id &&
  context.purpose == "case-management"
};
