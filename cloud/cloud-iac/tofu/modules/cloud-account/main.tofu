# Canonical OpenTofu module — cloud-account (Tier B per ADR-0202).
# Root-of-trust cloud account + organization scaffolding.

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

variable "account_name" {
  type        = string
  description = "Logical account name (e.g. 'oya-prod-us-east-1')."
}

variable "billing_email" {
  type        = string
  description = "Billing contact for the account."
}

variable "tags" {
  type    = map(string)
  default = {}
}

# Real provider resources land at wire time. The module surface
# (variables + outputs) is the canonical contract; the parent
# wires the provider-specific resource declarations.

output "account_id" {
  description = "Cloud-provider account identifier."
  value       = "" # wired at provider integration
}

output "account_alias" {
  description = "Cloud-provider account alias."
  value       = var.account_name
}
