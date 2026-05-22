// Customer Relationship Management pack overlay authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: CRM
// Benchmarks: SAP CRM Customer Relationship Management | Oracle Fusion Sales and Service | Workday customer and service-workflow counterpart | NetSuite CRM | Microsoft Dynamics 365 Customer Engagement

forbid (principal, action, resource);

permit (principal, action, resource)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has compliance_packs &&
  context has requested_pack &&
  context.requested_pack in context.compliance_packs &&
  context has pack_overlay_signature &&
  context.pack_overlay_signature != ""
};
