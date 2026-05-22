// Pharmacy microservice — sovereign deployment context (CSAP / IL5 / C5 / CCCS)
// Authority: ADR-0332; ADR-0250 (build-ahead of certification); ADR-0251 (compliance packs)
// Air-gapped substrate with offline FDB/Multum knowledge package distribution.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = ">= 2.30" }
    helm       = { source = "hashicorp/helm", version = ">= 2.15" }
  }
}

variable "tenant_id" { type = string }
variable "sovereign_realm" {
  type        = string
  description = "Sovereign realm slug: csap-kr | il5-us | c5-de | cccs-ca | etc."
}
variable "cell_id" { type = string }
variable "rust_image_tag" { type = string }
variable "offline_knowledge_package_path" { type = string }

module "sovereign_workload" {
  source                          = "../../../../iac/modules/sovereign-workload"
  name                            = "pharmacy"
  tenant_id                       = var.tenant_id
  sovereign_realm                 = var.sovereign_realm
  cell_id                         = var.cell_id
  image_tag                       = var.rust_image_tag
  air_gapped                      = true
  offline_knowledge_package_path  = var.offline_knowledge_package_path
  cilium_l4                       = true
  ambient_mesh                    = true
  http3_quic                      = true
  enforce_local_kms               = true
  enforce_local_secrets_substrate = true
  attest_supply_chain             = true
}

output "endpoints" {
  value = module.sovereign_workload.endpoints
}
