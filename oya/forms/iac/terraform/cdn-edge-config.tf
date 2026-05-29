# OCI CDN per-pack edge configuration for forms.
# Per ADR-0131 per-microservice flat layout + multi-region.md DR posture.
# References microservices/forms/policy/data-residency.md for pack list + residency rules.

terraform {
  required_version = ">= 1.6.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = ">= 5.0"
    }
  }
}

variable "packs" {
  description = "List of activated packs for forms CDN edge configuration"
  type = list(object({
    name       = string
    region     = string
    dr_region  = optional(string, "")
    hipaa      = bool
    activated  = bool
  }))
  default = [
    {name = "pack-kr",            region = "ap-seoul-1",     hipaa = false, activated = true,  dr_region = ""},
    {name = "pack-eu",            region = "eu-frankfurt-1", hipaa = false, activated = false, dr_region = "eu-amsterdam-1"},
    {name = "pack-us",            region = "us-ashburn-1",   hipaa = false, activated = false, dr_region = "us-phoenix-1"},
    {name = "pack-us-healthcare", region = "us-ashburn-1",   hipaa = true,  activated = false, dr_region = "us-phoenix-1"},
    {name = "pack-jp",            region = "ap-tokyo-1",     hipaa = false, activated = false, dr_region = ""},
    {name = "pack-sg",            region = "ap-singapore-1", hipaa = false, activated = false, dr_region = ""},
    {name = "pack-au",            region = "ap-sydney-1",    hipaa = false, activated = false, dr_region = "ap-melbourne-1"},
    {name = "pack-in",            region = "ap-hyderabad-1", hipaa = false, activated = false, dr_region = "ap-mumbai-1"},
    {name = "pack-br",            region = "sa-saopaulo-1",  hipaa = false, activated = false, dr_region = "sa-vinhedo-1"},
    {name = "pack-ae",            region = "me-abudhabi-1",  hipaa = false, activated = false, dr_region = "me-dubai-1"},
    {name = "pack-ksa",           region = "me-jeddah-1",    hipaa = false, activated = false, dr_region = "me-riyadh-1"},
  ]
}

resource "oci_waas_waas_policy" "forms_pack_waf" {
  for_each       = { for p in var.packs : p.name => p if p.activated }
  compartment_id = "ocid1.tenancy.oc1..forms-compartment"
  domain         = "forms-${each.value.name}.oyatie.dev"
  display_name   = "forms-waf-${each.value.name}"

  origins {
    label    = "forms-rest-${each.value.name}"
    uri      = "form-rest.forms.svc.${each.value.region}.oyatie.dev"
    http_port = 8080
  }

  policy_config {
    certificate_id    = "ocid1.certificate.oc1..forms-${each.value.name}-cert"
    cipher_group      = "default"
    client_address_header = "X-Real-IP"
  }
}
