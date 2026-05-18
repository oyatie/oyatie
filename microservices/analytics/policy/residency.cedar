// Cedar policy — residency enforcement.
// Authority: ADR-0049 cross-region replication and residency, ADR-0010 regional packs.
//
// Analytics data residency classes:
//   StrictKR        — only kr-* cells
//   StrictEU        — only eu-* cells
//   KrWithUsFailover — primary kr-*, DR-failover to us-*
//   Global          — any cell per cell-strategy

@id("residency-strict-kr-only-kr-cells")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"StrictKR" &&
  !context.cell.startsWith("kr-")
};

@id("residency-strict-eu-only-eu-cells")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"StrictEU" &&
  !context.cell.startsWith("eu-")
};

@id("residency-kr-with-us-failover-only-on-dr")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"KrWithUsFailover" &&
  context.cell.startsWith("us-") &&
  context.dr_failover_active != true
};

@id("residency-no-cross-cell-federation-from-tenant")
forbid (
  principal,
  action == Action::"CrossCellQuery",
  resource
) unless {
  principal in Role::"InternalAdmin"
};

@id("residency-ksa-only-ksa-cells")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"StrictKSA" &&
  !context.cell.startsWith("ksa-")
};

@id("residency-uae-only-uae-cells")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"StrictUAE" &&
  !context.cell.startsWith("uae-")
};

@id("residency-us-healthcare-only-us-healthcare-cells")
forbid (
  principal,
  action,
  resource
) when {
  resource has residency_class &&
  resource.residency_class == ResidencyClass::"UsHealthcareHipaa" &&
  !context.cell.startsWith("us-healthcare-")
};
