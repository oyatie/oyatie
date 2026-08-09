# Allowed template-marketplace publisher set per pack (per ADR-FORMS-0001 §"Templates").
# References microservices/forms/threat-model.md §"T-S-04" (publisher key compromise).

variable "template_publishers_per_pack" {
  description = "Allowed publisher key references per pack"
  type        = map(list(string))
  default = {
    "pack-kr"            = ["pub:oyatie-official", "pub:kr-gov-templates", "pub:kr-fss-templates"]
    "pack-eu"            = ["pub:oyatie-official", "pub:eu-edpb-templates", "pub:eu-gov-templates"]
    "pack-us"            = ["pub:oyatie-official", "pub:us-state-templates"]
    "pack-us-healthcare" = ["pub:oyatie-official", "pub:hipaa-baa-templates", "pub:us-hhs-templates"]
    "pack-jp"            = ["pub:oyatie-official", "pub:jp-gov-templates"]
    "pack-sg"            = ["pub:oyatie-official", "pub:sg-pdpa-templates"]
    "pack-au"            = ["pub:oyatie-official", "pub:au-oaic-templates"]
    "pack-in"            = ["pub:oyatie-official", "pub:in-dpb-templates"]
    "pack-br"            = ["pub:oyatie-official", "pub:br-anpd-templates"]
    "pack-ae"            = ["pub:oyatie-official", "pub:ae-pdpl-templates"]
    "pack-ksa"           = ["pub:oyatie-official", "pub:ksa-nca-templates"]
  }
}

# Top-10 enterprise publishers require multi-sig (2-of-3) keys.
variable "multisig_required_publishers" {
  description = "Publishers that require multi-sig template-marketplace signing"
  type        = list(string)
  default     = ["pub:oyatie-official"]
}

output "template_publishers_per_pack" {
  value = var.template_publishers_per_pack
}
