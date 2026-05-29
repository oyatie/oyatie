// Pharmacy microservice — OCI-guest deployment context
// Authority: feedback_zero_handroll_opentofu_only_2026_05_20; ADR-0332; ADR-0254
// OpenTofu only; no manual steps.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    oci        = { source = "oracle/oci", version = ">= 6.0" }
    kubernetes = { source = "hashicorp/kubernetes", version = ">= 2.30" }
    helm       = { source = "hashicorp/helm", version = ">= 2.15" }
  }
}

variable "tenant_id" { type = string }
variable "region" { type = string default = "us-ashburn-1" }
variable "cell_id" { type = string }
variable "rust_image_tag" { type = string }

module "oke_workload" {
  source       = "../../../../iac/modules/oci-oke-workload"
  name         = "pharmacy"
  tenant_id    = var.tenant_id
  cell_id      = var.cell_id
  image_tag    = var.rust_image_tag
  cpu_request  = "500m"
  cpu_limit    = "4"
  mem_request  = "1Gi"
  mem_limit    = "16Gi"
  http3_quic   = true
}

module "autonomous_db" {
  source    = "../../../../iac/modules/oci-autonomous-db"
  name      = "pharmacy"
  tenant_id = var.tenant_id
  workload  = "OLTP"
}

module "streaming" {
  source    = "../../../../iac/modules/oci-streaming"
  name      = "pharmacy"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
}

output "endpoints" {
  value = {
    rest_https3 = module.oke_workload.https3_endpoint
    grpc_h3     = module.oke_workload.grpc_h3_endpoint
    streaming   = module.streaming.stream_url
  }
}
