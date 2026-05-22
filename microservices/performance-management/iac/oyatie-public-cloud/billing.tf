# Billing seam: bind to ADR-0314 DealSet marketplace settlement.
# Closes audit Finding 6.6.A (P0).
resource "oya_billing_binding" "performance_management_settlement" {
  billing_component_id = "bc-performance-management"
  service_name         = "performance-management"
  tenant_id            = var.tenant_id
  tenant_class         = var.tenant_class
  context              = "oyatie-public-cloud"
  marketplace_settlement = "DealSet"
}
