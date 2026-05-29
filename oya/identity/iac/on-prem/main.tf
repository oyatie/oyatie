# OpenTofu module — identity µservice — context: on-prem
# Authority: ADR-0131 + ADR-0244 + ADR-0243 + ADR-0329 + ADR-0330 + ADR-0331
#            ADR-0328 §A.4 "customer-controlled facility, disconnected audit,
#            local IdP/HSM, portable storage"
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: customer-owned facility, customer-owned hardware, customer-owned ops.
#         identity is mandatory in on-prem because regulated tenants must own
#         the principal root locally.
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
    vsphere    = { source = "hashicorp/vsphere", version = "~> 2.8" }
  }

  backend "s3" {
    # MinIO-backed S3 + Postgres lock table per
    # specs/master-plan-sequencing.json:758-765 on-prem state backend.
    # endpoint=https://minio.<facility-id>.local
    # bucket=oyatie-tofu-state, key="identity/<tenant>.tfstate",
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. on-prem accepts paid only. demo_trial customers run on OCI Always Free."
  type        = string

  validation {
    condition     = var.tenant_class == "paid"
    error_message = "on-prem accepts tenant_class = paid only. demo_trial uses iac/oci-guest/always-free/."
  }
}

variable "billing_components" {
  description = "Subset of {revenue_share, per_seat, per_usage} per ADR-0330 §B.2. On-prem contracts typically use per_seat."
  type        = list(string)
  default     = ["per_seat"]

  validation {
    condition = alltrue([
      for c in var.billing_components : contains(["revenue_share", "per_seat", "per_usage"], c)
    ])
    error_message = "billing_components MUST be a subset of {revenue_share, per_seat, per_usage}."
  }
}

variable "tenant_id" {
  type = string

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{2,62}[a-z0-9]$", var.tenant_id))
    error_message = "paid tenant_id MUST NOT carry the demo_ prefix; format per ADR-0244 §D-1."
  }
}

variable "site_id" {
  type        = string
  description = "Customer facility identifier"
}

variable "k8s_cluster_endpoint" { type = string }

variable "k8s_ca_cert" {
  type      = string
  sensitive = true
}

variable "k8s_token" {
  type      = string
  sensitive = true
}

variable "sovereign_cell" {
  type    = bool
  default = true
}

variable "air_gap_mode" {
  type        = bool
  default     = false
  description = "When true, the bundle ships from a local OCI registry mirror and egress is blocked."
}

variable "hsm_endpoint" {
  type        = string
  default     = ""
  description = "On-prem HSM endpoint for issuer signing key custody (Thales / Entrust / SafeNet). When empty, OpenBao FIPS L2 software KMS is used."
}

variable "compliance_packs" {
  description = "Activated compliance packs per ADR-0251."
  type        = list(string)
  default     = []
}

variable "byok_enabled" {
  description = "BYOK opt-in per ADR-0255 §D-4. paid-only by ADR-0330 §B.3.7."
  type        = bool
  default     = false
}

# -------------------------------------------------------------------------
# Provider wiring
# -------------------------------------------------------------------------

provider "kubernetes" {
  host                   = var.k8s_cluster_endpoint
  cluster_ca_certificate = base64decode(var.k8s_ca_cert)
  token                  = var.k8s_token
}

# -------------------------------------------------------------------------
# Resources
# -------------------------------------------------------------------------

resource "kubernetes_namespace" "identity" {
  metadata {
    name = "identity-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "identity"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/site_id"            = var.site_id
      "oyatie.io/data-class"         = "pii-identifying"
      "oyatie.io/deployment-context" = "on-prem"
      "oyatie.io/sovereign-cell"     = tostring(var.sovereign_cell)
      "oyatie.io/air-gap-mode"       = tostring(var.air_gap_mode)
    }
    annotations = {
      "oyatie.io/billing-components" = join(",", var.billing_components)
      "oyatie.io/compliance-packs"   = join(",", var.compliance_packs)
      "oyatie.io/hsm-endpoint"       = var.hsm_endpoint
    }
  }
}

resource "helm_release" "zitadel" {
  name      = "zitadel"
  namespace = kubernetes_namespace.identity.metadata[0].name
  chart     = "${path.module}/../helm/zitadel"

  values = [
    yamlencode({
      image = {
        repository = var.air_gap_mode ? "registry.internal.${var.site_id}/zitadel" : "ghcr.io/zitadel/zitadel"
        tag        = "v2.65.0"
      }

      tenantId          = var.tenant_id
      tenantClass       = var.tenant_class
      billingComponents = var.billing_components
      compliancePacks   = var.compliance_packs

      sovereign     = var.sovereign_cell
      airGapMode    = var.air_gap_mode
      egressBlocked = var.air_gap_mode
      hsmEndpoint   = var.hsm_endpoint

      byok = { enabled = var.byok_enabled }

      resources = {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
        OYATIE_AIR_GAP_MODE                      = tostring(var.air_gap_mode)
      }
    })
  ]
}

# -------------------------------------------------------------------------
# Outputs
# -------------------------------------------------------------------------

output "identity_namespace" { value = kubernetes_namespace.identity.metadata[0].name }
output "tenant_class" { value = var.tenant_class }
output "billing_components" { value = var.billing_components }
output "deployment_context" { value = "on-prem" }
output "sovereign_cell" { value = var.sovereign_cell }
output "air_gap_mode" { value = var.air_gap_mode }
