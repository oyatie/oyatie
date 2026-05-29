// Warehouse pack overlay authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: EWM
// Benchmarks: SAP EWM Extended Warehouse Management | Oracle Fusion Warehouse Management | Workday inventory-operations counterpart | NetSuite WMS | Microsoft Dynamics 365 Warehouse Management

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
