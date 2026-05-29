// Pharmacy microservice — Oyatie-as-cloud-provider deployment context
// Authority: feedback_multi_context_provider_agnostic_2026_05_20; ADR-0248 (cellular); ADR-0254 (K8s + Cloud Hypervisor)

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = ">= 2.30" }
    helm       = { source = "hashicorp/helm", version = ">= 2.15" }
    oyatie     = { source = "oyatie/oyatie", version = ">= 0.1" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "rust_image_tag" { type = string }
variable "compliance_packs" {
  type    = list(string)
  default = ["hipaa", "dea-controlled-substance", "dscsa", "usp-797", "usp-800", "340b", "ncpdp-script", "surescripts"]
}

module "oyatie_cell" {
  source            = "../../../../iac/modules/oyatie-cell-workload"
  name              = "pharmacy"
  tenant_id         = var.tenant_id
  cell_id           = var.cell_id
  image_tag         = var.rust_image_tag
  cpu_limit         = "4"
  mem_limit         = "16Gi"
  http3_quic        = true
  cloud_hypervisor  = true // microVM via Kata pods (per ADR-0254)
  confidential_compute_modes = ["sev-snp", "tdx", "arm-cca"]
  compliance_packs  = var.compliance_packs
}

output "endpoints" {
  value = module.oyatie_cell.endpoints
}
