# OpenTofu module — identity µservice — context: oyatie-public-cloud
# Authority: ADR-0131 per-microservice flat layout
#            ADR-0244 tenant as universal scoping primitive
#            ADR-0243 Cedar as universal gate (tenant_class principal claim)
#            ADR-0329 tenant_class system retired (no demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating)
#            ADR-0330 tenant_class (demo_trial | paid) + billing_components
#            ADR-0331 cross-µservice tenant_class adoption template
#            ADR-0328 §D-15 six canonical deployment contexts
#            memory feedback_zero_handroll_opentofu_only_2026_05_20
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: oyatie's hosted multi-tenant public cloud cells (criticality tenant_class 0/1)
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes    = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm          = { source = "hashicorp/helm", version = "~> 2.13" }
    kustomization = { source = "kbst/kustomization", version = "~> 0.9" }
  }

  backend "s3" {
    # Inherited from cloud-iac convention; concrete backend wired per-cell:
    # bucket=oyatie-tofu-state-<region>, key="identity/<cell_id>.tfstate",
    # dynamodb_table=oyatie-tofu-locks-<region>, encrypt=true, kms_key_id=<cell KMS>.
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. Public-cloud cells host both demo_trial (capped) and paid tenants on the same substrate; this module is the canonical entry for paid public-cloud workloads."
  type        = string

  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class MUST be one of {demo_trial, paid} per ADR-0330 §B.1.1."
  }
}

variable "billing_components" {
  description = "Subset of {revenue_share, per_seat, per_usage} per ADR-0330 §B.2. Empty when tenant_class == demo_trial."
  type        = list(string)
  default     = []

  validation {
    condition = alltrue([
      for c in var.billing_components : contains(["revenue_share", "per_seat", "per_usage"], c)
    ])
    error_message = "billing_components MUST be a subset of {revenue_share, per_seat, per_usage} per ADR-0330 §B.2.2."
  }
}

variable "cell_id" {
  description = "Target cell identifier (e.g., cell-us-east-1-tenant_class-0-001) per ADR-0248 cellular criticality."
  type        = string
}

variable "cell_criticality_tier" {
  description = "Cellular criticality classification per ADR-0248 (tenant_class 0..tenant_class 4). Preserved through ADR-0329 retirement — this is infrastructure-availability, NOT capability-adoption vocabulary."
  type        = string
  default     = "tenant_class 1"

  validation {
    condition     = contains(["tenant_class 0", "tenant_class 1", "tenant_class 2", "tenant_class 3", "tenant_class 4"], var.cell_criticality_tier)
    error_message = "cell_criticality_tier MUST be a valid ADR-0248 cellular criticality value."
  }
}

variable "tenant_ids" {
  description = "Tenant IDs eligible for this cell. Identity is a T0 substrate so all tenants are eligible by default."
  type        = list(string)
  default     = []
}

variable "azs" {
  type        = list(string)
  description = "Availability zones for the public-cloud cell."
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "replicas_per_az" {
  type        = number
  default     = 4
  description = "OIDC issuer replicas per AZ. 4x3 = 12 baseline survives single-AZ loss with 2x headroom."
}

variable "zitadel_image_tag" {
  type        = string
  default     = "v2.65.0"
  description = "Zitadel pinned per microservices/identity/manifest.json lts_pins.zitadel."
}

variable "compliance_packs" {
  description = "Activated compliance packs per ADR-0251. demo_trial cells reject all packs (ADR-0330 §B.3.6); paid cells may activate any pack the cell is certified for."
  type        = list(string)
  default     = []
}

# -------------------------------------------------------------------------
# Cross-binding: compliance pack activation requires tenant_class = paid.
# -------------------------------------------------------------------------

resource "terraform_data" "tenant_class_compliance_check" {
  lifecycle {
    precondition {
      condition     = var.tenant_class == "paid" || length(var.compliance_packs) == 0
      error_message = "ADR-0330 §B.3.6 + ADR-0251: demo_trial tenants MUST NOT activate compliance packs."
    }
  }
}

# -------------------------------------------------------------------------
# Resources
# -------------------------------------------------------------------------

resource "kubernetes_namespace" "identity" {
  metadata {
    name = "identity-${var.cell_id}"
    labels = {
      "oyatie.io/microservice"          = "identity"
      "oyatie.io/cell-id"               = var.cell_id
      "oyatie.io/cell-criticality-tenant_class" = var.cell_criticality_tier
      "oyatie.io/data-class"            = "pii-identifying"
      "oyatie.io/tenant-class"          = var.tenant_class
      "oyatie.io/deployment-context"    = "oyatie-public-cloud"
      "oyatie.io/canonical-base"        = "true"
    }
    annotations = {
      "oyatie.io/billing-components" = join(",", var.billing_components)
      "oyatie.io/compliance-packs"   = join(",", var.compliance_packs)
    }
  }
}

resource "helm_release" "zitadel" {
  name      = "zitadel"
  namespace = kubernetes_namespace.identity.metadata[0].name
  chart     = "${path.module}/../helm/zitadel"

  values = [
    yamlencode({
      replicaCount = var.replicas_per_az * length(var.azs)
      image = {
        repository = "ghcr.io/zitadel/zitadel"
        tag        = var.zitadel_image_tag
        pullPolicy = "IfNotPresent"
      }
      podDisruptionBudget = { minAvailable = "60%" }
      affinity = {
        podAntiAffinity = {
          preferredDuringSchedulingIgnoredDuringExecution = [{
            weight = 100
            podAffinityTerm = {
              labelSelector = { matchLabels = { app = "zitadel" } }
              topologyKey   = "topology.kubernetes.io/zone"
            }
          }]
        }
      }
      resources = {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }
      autoscaling = {
        enabled                        = true
        minReplicas                    = var.replicas_per_az * length(var.azs)
        maxReplicas                    = var.replicas_per_az * length(var.azs) * 4
        targetCPUUtilizationPercentage = 70
      }

      tenantClass         = var.tenant_class
      billingComponents   = var.billing_components
      cellId              = var.cell_id
      cellCriticalityTier = var.cell_criticality_tier
      compliancePacks     = var.compliance_packs

      tenantIds = var.tenant_ids

      # Hot-path posture (per chat-history anchor: per-cell auth challenge/session state).
      sessionStateStorage          = "per-cell-postgres"
      jwksRotationDays             = 90
      jwksEmergencyRotationMinutes = 15

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION          = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME    = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME        = "billing_components"
        OYATIE_TIER_VOCABULARY_RETIRED_PER_ADR_0329 = "true"
      }
    })
  ]
}

# -------------------------------------------------------------------------
# Outputs
# -------------------------------------------------------------------------

output "identity_namespace" {
  value = kubernetes_namespace.identity.metadata[0].name
}

output "tenant_class" {
  value = var.tenant_class
}

output "billing_components" {
  value = var.billing_components
}

output "deployment_context" {
  value = "oyatie-public-cloud"
}
