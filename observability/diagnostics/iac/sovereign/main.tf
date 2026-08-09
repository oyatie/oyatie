variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "pack_overlay" {
  type    = list(string)
  default = ["HIPAA-2024", "EU-IVDR", "KR-IVD", "CLIA", "CAP"]
}

locals {
  diagnostics_sovereign_controls = {
    tenant_id              = var.tenant_id
    cell_id                = var.cell_id
    home_cell_required     = true
    cross_cell_phi_default = "deny"
    service_line           = "lab-pathology"
    packs                  = var.pack_overlay
  }
}

output "diagnostics_sovereign_controls" {
  value = local.diagnostics_sovereign_controls
}
