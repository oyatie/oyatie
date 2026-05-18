# Canonical OpenTofu module — kms (Tier B per ADR-0202).

terraform {
  required_version = ">= 1.6"
}

variable "account_id"     { type = string }
variable "region"         { type = string }
variable "key_purpose"    { type = string }   # e.g. "tenant-data", "audit-chain"
variable "rotation_days"  { type = number, default = 365 }
variable "tags"           { type = map(string), default = {} }

output "key_arn"   { value = "" }
output "key_id"    { value = "" }
output "key_alias" { value = "" }
