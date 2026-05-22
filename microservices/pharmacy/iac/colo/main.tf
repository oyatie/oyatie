// Pharmacy microservice — colo deployment context
// Authority: feedback_zero_handroll_opentofu_only_2026_05_20; ADR-0332; ADR-0254

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = ">= 2.30" }
    helm       = { source = "hashicorp/helm", version = ">= 2.15" }
    talos      = { source = "siderolabs/talos", version = ">= 0.6" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "rust_image_tag" { type = string }

module "talos_workload" {
  source       = "../../../../iac/modules/talos-workload"
  name         = "pharmacy"
  tenant_id    = var.tenant_id
  cell_id      = var.cell_id
  image_tag    = var.rust_image_tag
  cpu_limit    = "4"
  mem_limit    = "16Gi"
  http3_quic   = true
  cilium_l4    = true
  ambient_mesh = true
}

output "endpoints" {
  value = module.talos_workload.endpoints
}
