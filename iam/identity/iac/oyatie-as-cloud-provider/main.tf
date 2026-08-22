# OpenTofu module — identity µservice — context: oyatie-as-cloud-provider
# Authority: ADR-0328 §A.6 "oyatie sells IaaS to tenants; provider-control-plane
#            prerequisite; identity is required as provider identity/security
#            service, NOT a cloud adapter"
#            ADR-0131 + ADR-0244 + ADR-0243 + ADR-0329 + ADR-0330 + ADR-0331
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: oyatie itself is the cloud provider. The tenant pays oyatie cloud-billing
#         for compute, storage, network, plus the cell-criticality-tenant_class premium
#         (per ADR-0248 tenant_class 0..tenant_class 4 — preserved cellular vocabulary).
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
  }

  backend "s3" {
    # Backend is oyatie's own cloud-storage (the IaaS storage µservice).
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. oyatie-as-cloud-provider accepts paid only — demo_trial uses iac/oci-guest/always-free/ on the OCI hyperscaler."
  type        = string

  validation {
    condition     = var.tenant_class == "paid"
    error_message = "oyatie-as-cloud-provider accepts tenant_class = paid only."
  }
}

variable "billing_components" {
  description = "Subset of {revenue_share, per_seat, per_usage}. Oyatie cloud provider tenants typically include per_usage for infrastructure metering."
  type        = list(string)
  default     = ["per_usage"]

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

variable "oyatie_cell_id" {
  type        = string
  description = "Oyatie internal cell identifier (e.g., cell-iad-001)."
}

variable "cell_criticality_tier" {
  description = "Cellular criticality classification per ADR-0248 (tenant_class 0..tenant_class 4). PRESERVED vocabulary per ADR-0329 §B2.036 — this is infrastructure-availability, NOT the retired capability-adoption ladder."
  type        = string
  default     = "tenant_class 1"

  validation {
    condition     = contains(["tenant_class 0", "tenant_class 1", "tenant_class 2", "tenant_class 3", "tenant_class 4"], var.cell_criticality_tier)
    error_message = "cell_criticality_tier MUST be a valid ADR-0248 value."
  }
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

variable "compliance_packs" {
  description = "Activated compliance packs per ADR-0251."
  type        = list(string)
  default     = []
}

variable "byok_enabled" {
  description = "BYOK opt-in per ADR-0255 §D-4."
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
      "oyatie.io/microservice"          = "identity"
      "oyatie.io/tenant_id"             = var.tenant_id
      "oyatie.io/tenant-class"          = var.tenant_class
      "oyatie.io/oyatie-cell-id"        = var.oyatie_cell_id
      "oyatie.io/cell-criticality-tenant_class" = var.cell_criticality_tier
      "oyatie.io/data-class"            = "pii-identifying"
      "oyatie.io/deployment-context"    = "oyatie-as-cloud-provider"
      "oyatie.io/billing-emit"          = "true"
    }
    annotations = {
      "oyatie.io/billing-components" = join(",", var.billing_components)
      "oyatie.io/compliance-packs"   = join(",", var.compliance_packs)
    }
  }
}

# -------------------------------------------------------------------------
# Sizing schedule per cell-criticality-tenant_class (preserved ADR-0248 vocabulary).
# These sizes are infrastructure-availability tiers, NOT capability availability.
# -------------------------------------------------------------------------

locals {
  size_by_criticality = {
    "tenant_class 0" = { requests = { cpu = "4", memory = "16Gi" }, limits = { cpu = "16", memory = "64Gi" }, replicas = 12 }
    "tenant_class 1" = { requests = { cpu = "2", memory = "8Gi" }, limits = { cpu = "8", memory = "32Gi" }, replicas = 6 }
    "tenant_class 2" = { requests = { cpu = "1", memory = "4Gi" }, limits = { cpu = "4", memory = "16Gi" }, replicas = 3 }
    "tenant_class 3" = { requests = { cpu = "1", memory = "4Gi" }, limits = { cpu = "2", memory = "8Gi" }, replicas = 2 }
    "tenant_class 4" = { requests = { cpu = "4", memory = "16Gi" }, limits = { cpu = "16", memory = "64Gi" }, replicas = 12 }
  }
}

resource "helm_release" "zitadel" {
  name      = "zitadel"
  namespace = kubernetes_namespace.identity.metadata[0].name
  chart     = "${path.module}/../helm/zitadel"

  values = [
    yamlencode({
      image = {
        repository = "ghcr.io/zitadel/zitadel"
        tag        = "v2.65.0"
      }

      tenantId                  = var.tenant_id
      tenantClass               = var.tenant_class
      billingComponents         = var.billing_components
      compliancePacks           = var.compliance_packs
      oyatieCellId              = var.oyatie_cell_id
      cellCriticalityTier       = var.cell_criticality_tier
      billingEmitToCloudBilling = true

      byok = { enabled = var.byok_enabled }

      replicaCount = local.size_by_criticality[var.cell_criticality_tier].replicas
      resources = {
        requests = local.size_by_criticality[var.cell_criticality_tier].requests
        limits   = local.size_by_criticality[var.cell_criticality_tier].limits
      }

      # Per_usage meter shape per ADR-0331 §D-1.10.
      paidUsageMeters = [
        "identity.oidc.token_issued_per_thousand",
        "identity.webauthn.authentication_per_thousand",
        "identity.scim.operations_per_thousand",
        "identity.step_up.grant_per_thousand",
        "identity.session.active_concurrent_seconds_per_million"
      ]

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
        OYATIE_CELL_CRITICALITY_TIER             = var.cell_criticality_tier
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
output "deployment_context" { value = "oyatie-as-cloud-provider" }
output "cell_criticality_tier" { value = var.cell_criticality_tier }
