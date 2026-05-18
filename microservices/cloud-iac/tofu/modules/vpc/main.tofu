# Canonical OpenTofu module — vpc (Tier B per ADR-0202).

terraform {
  required_version = ">= 1.6"
}

variable "account_id"   { type = string }
variable "region"       { type = string }
variable "cidr_block"   { type = string }
variable "az_count"     { type = number, default = 3 }
variable "tags"         { type = map(string), default = {} }

output "vpc_id"             { value = "" }
output "private_subnet_ids" { value = [] }
output "public_subnet_ids"  { value = [] }
