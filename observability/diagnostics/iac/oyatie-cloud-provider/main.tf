variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "pack_overlay" {
  type    = list(string)
  default = ["HIPAA-2024", "CLIA", "CAP", "ISO-15189"]
}

locals {
  diagnostics_provider_profile = {
    tenant_id          = var.tenant_id
    cell_id            = var.cell_id
    service_line       = "lab-pathology"
    evidence_residency = "home-cell"
    packs              = var.pack_overlay
  }
}

output "diagnostics_provider_profile" {
  value = local.diagnostics_provider_profile
}
