# Canonical OpenTofu module — secrets-bootstrap (Tier B per ADR-0202).
# OpenBao initial seed + PKI mount creation.

terraform {
  required_version = ">= 1.6"
}

variable "account_id"     { type = string }
variable "region"         { type = string }
variable "bao_namespace"  { type = string, default = "oya" }
variable "tags"           { type = map(string), default = {} }

output "bao_root_token_secret_id" { value = "" }
output "bao_unseal_keys_secret_ids" { value = [] }
output "pki_mount_path" { value = "" }
