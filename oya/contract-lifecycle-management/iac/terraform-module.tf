terraform {
  required_version = ">= 1.8.0"
}

variable "tenant_id" { type = string }
variable "cell_tier" { type = string }

resource "oya_service_module" "contract_lifecycle_management" {
  service_name = "contract-lifecycle-management"
  transport_profile = "h3-h2-h1-strict-tls13-ech-pqc"
  marketplace_settlement = "DealSet"
  binding_adrs = ["ADR-0105","ADR-0131","ADR-0242","ADR-0243","ADR-0244","ADR-0246","ADR-0253-amendment","ADR-0257","ADR-0258","ADR-0263","ADR-0294","ADR-0296","ADR-0297","ADR-0314","ADR-0321"]
}
