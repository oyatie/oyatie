// Current authority: specs/microservices/global-trade.json metadata-only PRD; inventory/provenance/planned-only.
// This Cedar fragment is planned/provenance only and is not runtime policy-engine authority.
// Non-claims: no live authorization runtime, no Workflow execution, no runtime audit-chain emission, no cloud deployment.
// Global Trade pack overlay authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: GTS
// Benchmarks: SAP GTS Global Trade Services | Oracle Global Trade Management | Workday supplier-compliance workflow counterpart | NetSuite international tax and trade counterpart | Microsoft Dynamics 365 global trade and export-control counterpart

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
