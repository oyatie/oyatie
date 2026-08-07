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

variable "account_id" {
  type = string
  validation {
    condition     = trimspace(var.account_id) != ""
    error_message = "account_id must not be empty"
  }
}

variable "region" {
  type = string
  validation {
    condition     = trimspace(var.region) != ""
    error_message = "region must not be empty"
  }
}

variable "key_purpose" {
  type = string # e.g. "tenant-data", "audit-chain"
  validation {
    condition     = trimspace(var.key_purpose) != ""
    error_message = "key_purpose must not be empty"
  }
}
variable "rotation_days" {
  type    = number
  default = 365
  validation {
    condition     = var.rotation_days > 0
    error_message = "rotation_days must be greater than zero"
  }
}
variable "tags" {
  type    = map(string)
  default = {}
}

output "key_arn" { value = "" }
output "key_id" { value = "" }
output "key_alias" { value = "" }
