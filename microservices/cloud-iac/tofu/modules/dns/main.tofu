# Canonical OpenTofu module — dns (Tier B per ADR-0202).
# Publishes SPF / DKIM / DMARC records consumed by ADR-0201
# email comms substrate.

terraform {
  required_version = ">= 1.6"
}

variable "account_id" { type = string }
variable "zones"      { type = list(string) }
variable "records"    {
  type = list(object({
    zone  = string
    name  = string
    type  = string
    ttl   = number
    value = string
  }))
  default = []
}

output "zone_ids" { value = {} }
