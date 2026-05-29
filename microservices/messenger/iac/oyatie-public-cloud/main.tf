# OpenTofu module — messenger µservice — context: oyatie-public-cloud
# Per ADR-0131 per-microservice flat layout + ADR-0244 tenant scoping + ADR-0251 compliance packs
# + ADR-0246 MLS RFC 9420 + memory feedback_zero_handroll_opentofu_only + ADR-0328 §D-15..§D-16
# Wave 15A-MESSENGER-FIX authored 2026-05-21
# Targets oyatie's hosted multi-tenant cells (tier-0 messaging fleet + tier-1 huddle SFU)
#
# This is the canonical deployment context — Oyatie-operated managed cloud cells.
# `tenant_class_default = paid` per /specs/master-plan-sequencing.json deployment_contexts.
# Hosts both the B2C personal messenger and B2B work messenger surfaces.
#
# Sister grafana-rbac.tf lives alongside this main.tf for the grafana folder + role binding.

variable "cell_id" {
  type        = string
  description = "Target cell identifier (e.g., cell-us-east-1-tier-0-msgr-001)"
}

variable "cell_certification_levels" {
  type        = list(string)
  default     = ["base-soc2-iso27001"]
  description = "Cell certification set per ADR-0251 §D-4; B2B work mode may require kr-isms-p or hipaa or eu-ai-act."
}

variable "tenant_ids" {
  type        = list(string)
  description = "Tenant IDs eligible for this cell"
}

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331 + memory feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20: tenant_class ∈ {demo_trial, paid}. oyatie-public-cloud defaults to paid."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be one of: demo_trial, paid (ADR-0331)."
  }
}

variable "paid_billing_components" {
  type        = list(string)
  default     = ["per_seat", "per_usage"]
  description = "Per ADR-0331 paid.billing_components ⊆ {revenue_share, per_seat, per_usage}; tenant_class=demo_trial sets this to []."
}

variable "audience_mode" {
  type        = string
  default     = "B2B-work"
  description = "Per PRD §2.1 audience mode: B2C-personal | B2B-work | oyatie-internal-tenant. Personal-mode defaults MLS enforce; work-mode is per-tenant opt-in via compliance pack."
}

variable "mls_e2ee_mode" {
  type        = string
  default     = "tenant_opt_in_with_recovery_key_escrow"
  description = "Per ADR-MSG-001 + memory feedback_mls_rfc_9420_e2ee_personal_messenger: enforce (B2C default-on) | tenant_opt_in_with_recovery_key_escrow (B2B) | disabled (only for non-PII demo_trial tenants)."
  validation {
    condition     = contains(["enforce", "tenant_opt_in_with_recovery_key_escrow", "disabled"], var.mls_e2ee_mode)
    error_message = "mls_e2ee_mode must be one of: enforce, tenant_opt_in_with_recovery_key_escrow, disabled."
  }
}

variable "mobile_app_bundle_peers" {
  type        = list(string)
  default     = ["mail", "social", "community"]
  description = "Per memory feedback_cell_standalone_network_merges_community_2026_05_21 the mobile-app-bundle ships messenger + mail + social + community as four panes of one binary; backend µservices remain canonical-separate per ADR-0145 + ADR-0064."
}

variable "compliance_packs" {
  type        = list(string)
  default     = ["soc2", "iso27001", "gdpr"]
  description = "Compliance packs activated for this cell per ADR-0251. demo_trial tenants cannot activate any pack."
}

variable "regulatory_packs" {
  type    = list(string)
  default = ["kr", "eu", "us", "us-healthcare", "jp", "sg", "au", "in", "br", "ae", "ksa"]
}

variable "replicas_per_az" {
  type    = number
  default = 18
}

variable "azs" {
  type    = list(string)
  default = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "huddle_sfu_replicas" {
  type        = number
  default     = 6
  description = "LiveKit SFU replica count per AZ (huddles BC per ADR-MSGR-0001)"
}

resource "kubernetes_namespace" "messenger" {
  metadata {
    name = "messenger-${var.cell_id}"
    labels = {
      "oyatie.io/microservice"       = "messenger"
      "oyatie.io/cell-id"            = var.cell_id
      "oyatie.io/deployment-context" = "oyatie-public-cloud"
      "oyatie.io/data-class"         = "PII_IDENTIFYING,AUTHENTICATION,AUDIT"
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/audience-mode"      = var.audience_mode
      "oyatie.io/mls-e2ee-mode"      = var.mls_e2ee_mode
      "oyatie.io/mobile-app-bundle"  = "messenger+mail+social+community"
    }
  }
}

resource "helm_release" "messenger" {
  name      = "messenger"
  namespace = kubernetes_namespace.messenger.metadata[0].name
  chart     = "${path.module}/../helm/messenger"

  values = [
    yamlencode({
      replicaCount = var.replicas_per_az * length(var.azs)
      image = {
        repository = "registry.oyatie.dev/messenger"
        tag        = "1.0.0-wave-15a"
        pullPolicy = "IfNotPresent"
      }
      tolerations = []
      affinity = {
        podAntiAffinity = {
          preferredDuringSchedulingIgnoredDuringExecution = [{
            weight = 100
            podAffinityTerm = {
              labelSelector = { matchLabels = { app = "messenger" } }
              topologyKey   = "topology.kubernetes.io/zone"
            }
          }]
        }
      }
      podDisruptionBudget = { minAvailable = "60%" }
      resources = {
        requests = { cpu = "1", memory = "4Gi" }
        limits   = { cpu = "4", memory = "16Gi" }
      }
      autoscaling = {
        enabled                        = true
        minReplicas                    = 18
        maxReplicas                    = 108
        targetCPUUtilizationPercentage = 65
      }
      tenantIds             = var.tenant_ids
      tenantClass           = var.tenant_class
      paidBillingComponents = var.paid_billing_components
      audienceMode          = var.audience_mode
      mlsE2eeMode           = var.mls_e2ee_mode
      mobileAppBundlePeers  = var.mobile_app_bundle_peers
      certificationLevels   = var.cell_certification_levels
      compliancePacks       = var.compliance_packs
      regulatoryPacks       = var.regulatory_packs

      env = {
        DEFAULT_MLS_CIPHERSUITE              = "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519"
        MLS_KEY_PACKAGE_TTL_DAYS             = "7"
        MLS_WELCOME_TTL_DAYS                 = "14"
        WS_GATEWAY_FANOUT_DEADLINE_MS        = "100"
        MENTION_FANOUT_DEADLINE_MS           = "250"
        PRESENCE_PROPAGATION_BUDGET_MS       = "200"
        ATTACHMENT_SCAN_FRESHNESS_BUDGET_SEC = "60"
        BUNDLE_PEER_GRPC_TIMEOUT_MS          = "1500"
      }

      huddleSfu = {
        replicaCount = var.huddle_sfu_replicas * length(var.azs)
        resources = {
          requests = { cpu = "2", memory = "8Gi" }
          limits   = { cpu = "8", memory = "32Gi" }
        }
      }
    })
  ]
}

output "messenger_namespace" { value = kubernetes_namespace.messenger.metadata[0].name }
output "messenger_helm_release" { value = helm_release.messenger.name }
output "tenant_class" { value = var.tenant_class }
output "mls_e2ee_mode" { value = var.mls_e2ee_mode }
output "mobile_app_bundle_peers" { value = var.mobile_app_bundle_peers }
