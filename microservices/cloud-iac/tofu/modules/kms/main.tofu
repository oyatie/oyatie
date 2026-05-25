# Canonical OpenTofu module — kms (Tier B per ADR-0202).

terraform {
  required_version = ">= 1.6"

  required_providers {
    aws = {
      source  = "registry.opentofu.org/hashicorp/aws"
      version = ">= 5.0.0"
    }
    oci = {
      source  = "registry.opentofu.org/opentofu/oci"
      version = ">= 6.0.0"
    }
  }
}

variable "account_id" { type = string }
variable "region" { type = string }
variable "key_purpose" { type = string } # e.g. "tenant-data", "audit-chain"
variable "rotation_days" {
  type    = number
  default = 365
}
variable "tags" {
  type    = map(string)
  default = {}
}

output "key_arn" { value = "" }
output "key_id" { value = "" }
output "key_alias" { value = "" }
