# OpenTofu module — identity µservice — context: colo
# Authority: ADR-0131 + ADR-0244 + ADR-0243 + ADR-0329 + ADR-0330 + ADR-0331
#            ADR-0328 §A.5 "sovereign cell, dedicated hardware, regulated
#            low-latency, facility-owned operation, Cilium/BGP/MetalLB,
#            facility telemetry, HSM custody"
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: customer-owned hardware in a colo facility (Equinix / Digital Realty /
#         OVH / Telehouse / KT Cloud / etc.) where oyatie provides operational
#         software and the tenant pays for facility power, space, hardware.
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
  }

  backend "s3" {
    # MinIO + Postgres lock table per spec.
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. colo accepts paid only."
  type        = string

  validation {
    condition     = var.tenant_class == "paid"
    error_message = "colo accepts tenant_class = paid only. demo_trial uses iac/oci-guest/always-free/."
  }
}

variable "billing_components" {
  description = "Subset of {revenue_share, per_seat, per_usage}."
  type        = list(string)
  default     = ["per_seat"]
}

variable "tenant_id" {
  type = string

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{2,62}[a-z0-9]$", var.tenant_id))
    error_message = "paid tenant_id MUST NOT carry the demo_ prefix; format per ADR-0244 §D-1."
  }
}

variable "colo_provider" {
  type        = string
  description = "Equinix / Digital Realty / OVH / Telehouse / KT Cloud / Hetzner / etc."
}

variable "colo_region" { type = string }

variable "sovereign_jurisdiction" {
  type        = string
  description = "EU / KR / KSA / JP / AE / etc."
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
  description = "Activated compliance packs per ADR-0251. colo cells frequently activate sovereign packs."
  type        = list(string)
  default     = []
}

variable "byok_enabled" {
  description = "BYOK opt-in per ADR-0255 §D-4. paid-only by ADR-0330 §B.3.7."
  type        = bool
  default     = false
}

variable "metallb_pool_cidr" {
  type        = string
  default     = ""
  description = "MetalLB / BGP-announced CIDR for facility-owned ingress."
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
      "oyatie.io/microservice"           = "identity"
      "oyatie.io/tenant_id"              = var.tenant_id
      "oyatie.io/tenant-class"           = var.tenant_class
      "oyatie.io/colo-provider"          = var.colo_provider
      "oyatie.io/colo-region"            = var.colo_region
      "oyatie.io/sovereign-jurisdiction" = var.sovereign_jurisdiction
      "oyatie.io/data-class"             = "pii-identifying"
      "oyatie.io/deployment-context"     = "colo"
    }
    annotations = {
      "oyatie.io/billing-components" = join(",", var.billing_components)
      "oyatie.io/compliance-packs"   = join(",", var.compliance_packs)
      "oyatie.io/metallb-pool-cidr"  = var.metallb_pool_cidr
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
        repository = "ghcr.io/zitadel/zitadel"
        tag        = "v2.65.0"
      }

      tenantId              = var.tenant_id
      tenantClass           = var.tenant_class
      billingComponents     = var.billing_components
      compliancePacks       = var.compliance_packs
      sovereignJurisdiction = var.sovereign_jurisdiction
      coloProvider          = var.colo_provider
      coloRegion            = var.colo_region

      byok = { enabled = var.byok_enabled }

      resources = {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
        OYATIE_SOVEREIGN_JURISDICTION            = var.sovereign_jurisdiction
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
output "deployment_context" { value = "colo" }
output "sovereign_jurisdiction" { value = var.sovereign_jurisdiction }
