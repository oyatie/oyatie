# Canonical OpenTofu module — dns (Tier B per ADR-0202).
# Publishes SPF / DKIM / DMARC records consumed by ADR-0201
# email comms substrate.

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
    cloudflare = {
      source  = "registry.opentofu.org/cloudflare/cloudflare"
      version = ">= 4.0.0"
    }
  }
}

variable "account_id" { type = string }
variable "zones" { type = list(string) }
variable "records" {
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
