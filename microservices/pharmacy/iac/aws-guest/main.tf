// Pharmacy microservice — AWS-guest deployment context
// Authority: feedback_zero_handroll_opentofu_only_2026_05_20; ADR-0332; ADR-0254
// OpenTofu only; no Terraform; no manual steps.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    aws        = { source = "hashicorp/aws", version = ">= 5.50" }
    kubernetes = { source = "hashicorp/kubernetes", version = ">= 2.30" }
    helm       = { source = "hashicorp/helm", version = ">= 2.15" }
  }
}

variable "tenant_id" { type = string }
variable "region" { type = string default = "us-east-1" }
variable "cell_id" { type = string }
variable "rust_image_tag" { type = string }

module "eks_workload" {
  source       = "../../../../iac/modules/aws-eks-workload"
  name         = "pharmacy"
  tenant_id    = var.tenant_id
  cell_id      = var.cell_id
  image_tag    = var.rust_image_tag
  cpu_request  = "500m"
  cpu_limit    = "4"
  mem_request  = "1Gi"
  mem_limit    = "16Gi"
  http3_quic   = true
  cilium_l4    = true
  ambient_mesh = true
}

module "rds_postgres_citus" {
  source     = "../../../../iac/modules/aws-rds-citus"
  name       = "pharmacy"
  tenant_id  = var.tenant_id
  cell_id    = var.cell_id
  engine_pin = "16.2"
}

module "msk_pulsar" {
  source    = "../../../../iac/modules/aws-msk-pulsar"
  name      = "pharmacy"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  pin       = "3.2"
}

module "openbao_secret_bindings" {
  source     = "../../../../iac/modules/openbao-bindings"
  name       = "pharmacy"
  tenant_id  = var.tenant_id
  paths      = [
    "secret/pharmacy/surescripts-mtls-cert",
    "secret/pharmacy/surescripts-mtls-key",
    "secret/pharmacy/pbm-ncpdp-issuer-credential",
    "secret/pharmacy/fdb-api-key",
    "secret/pharmacy/multum-api-key",
    "secret/pharmacy/medi-span-api-key",
    "secret/pharmacy/epcs-kms-key-binding",
    "secret/pharmacy/cabinet-pyxis-api-key",
    "secret/pharmacy/cabinet-omnicell-api-key",
    "secret/pharmacy/pump-alaris-drug-library-signing-key",
    "secret/pharmacy/dscsa-epcis-partner-mtls"
  ]
}

output "endpoints" {
  value = {
    rest_https3 = module.eks_workload.https3_endpoint
    grpc_h3     = module.eks_workload.grpc_h3_endpoint
    pulsar      = module.msk_pulsar.broker_url
  }
}
