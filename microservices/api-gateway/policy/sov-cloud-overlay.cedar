// api-gateway µservice — sovereign-cloud overlay fragment
// Cedar v4.2 LTS
// Authority: ADR-0244 (tenant scoping) + ADR-0248 (cellular) + ADR-0251 (compliance packs).

// =============================================================================
// FORBID — pack-eu tenant request routed to non-EU cell
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource in CellResource::?r
)
when {
  principal has compliance_packs &&
  principal.compliance_packs.contains("pack-eu") &&
  resource has cell_jurisdiction &&
  !(resource.cell_jurisdiction in ["eu-frankfurt", "eu-ireland"])
};

// =============================================================================
// FORBID — pack-kr tenant routed to non-KR cell
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource in CellResource::?r
)
when {
  principal has compliance_packs &&
  principal.compliance_packs.contains("pack-kr") &&
  resource has cell_jurisdiction &&
  resource.cell_jurisdiction != "ap-seoul"
};

// =============================================================================
// FORBID — pack-cn-pipl-2021 tenant routed outside PRC
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource in CellResource::?r
)
when {
  principal has compliance_packs &&
  principal.compliance_packs.contains("pack-cn-pipl-2021") &&
  resource has cell_jurisdiction &&
  resource.cell_jurisdiction != "cn-shanghai"
};

// =============================================================================
// FORBID — pack-il5/6 tenant on non-IL5/6 cell
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource in CellResource::?r
)
when {
  principal has compliance_packs &&
  (principal.compliance_packs.contains("pack-il5") || principal.compliance_packs.contains("pack-il6")) &&
  resource has cell_tier &&
  !(resource.cell_tier in ["il5-sov", "il6-sov"])
};

// =============================================================================
// FORBID — pack-us-healthcare without BAA
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource
)
when {
  principal has compliance_packs &&
  principal.compliance_packs.contains("pack-us-healthcare") &&
  principal has baa_in_force &&
  principal.baa_in_force == false
};

// =============================================================================
// PERMIT — sov-cell-kr admin operations from KR origin only
// =============================================================================

permit (
  principal in AdminPrincipal::?p,
  action == Action::"Route::admin_write",
  resource in CellResource::?r
)
when {
  resource has cell_jurisdiction &&
  resource.cell_jurisdiction == "ap-seoul" &&
  principal has origin_country &&
  principal.origin_country == "KR"
};
