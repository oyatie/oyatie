variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "pack_overlay" {
  type    = list(string)
  default = ["HIPAA-2024", "CLIA", "CAP"]
}

locals {
  diagnostics_colo_profile = {
    tenant_id    = var.tenant_id
    cell_id      = var.cell_id
    service_line = "lab-pathology"
    storage      = "tenant-scoped-evidence-volume"
    packs        = var.pack_overlay
  }
}

output "diagnostics_colo_profile" {
  value = local.diagnostics_colo_profile
}
