# Canonical OpenTofu module — secrets-bootstrap (Tier B per ADR-0202).
# OpenBao initial seed + PKI mount creation.

terraform {
  required_version = ">= 1.6"

  required_providers {
    vault = {
      source  = "registry.opentofu.org/hashicorp/vault"
      version = ">= 4.0.0"
    }
    kubernetes = {
      source  = "registry.opentofu.org/hashicorp/kubernetes"
      version = ">= 2.30.0"
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

variable "bao_namespace" {
  type    = string
  default = "oya"
  validation {
    condition     = trimspace(var.bao_namespace) != ""
    error_message = "bao_namespace must not be empty"
  }
}
variable "tags" {
  type    = map(string)
  default = {}
}

output "bao_root_token_secret_id" { value = "" }
output "bao_unseal_keys_secret_ids" { value = [] }
output "pki_mount_path" { value = "" }
