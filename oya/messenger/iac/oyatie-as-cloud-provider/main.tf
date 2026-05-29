# OpenTofu module — messenger µservice — context: oyatie-as-cloud-provider
# oyatie sells IaaS/PaaS to the tenant directly; cloud-* µservices are the
# IaaS surface (NOT AWS/OCI wrappers). The tenant pays oyatie cloud-billing
# for compute + storage + network + messenger-substrate-consumption.
# Wave 15A-MESSENGER-FIX authored 2026-05-21
# Per ADR-0245 substrate-vs-product + memory feedback_multi_context_provider_agnostic_2026_05_20.

variable "tenant_id" { type = string }
variable "oyatie_cell_id" { type = string }
variable "oyatie_cell_certification_levels" {
  type    = list(string)
  default = ["base-soc2-iso27001"]
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

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331; oyatie-as-cloud-provider supports both demo_trial and paid."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be one of: demo_trial, paid."
  }
}

variable "paid_billing_components" {
  type        = list(string)
  default     = ["per_seat", "per_usage"]
  description = "Per ADR-0331 paid.billing_components ⊆ {revenue_share, per_seat, per_usage}. demo_trial sets this to []."
}

variable "audience_mode" {
  type        = string
  default     = "B2C-personal"
  description = "oyatie-as-cloud-provider's hero is the consumer-grade personal messenger surface; switch to B2B-work or oyatie-internal-tenant per contract."
}

variable "mls_e2ee_mode" {
  type        = string
  default     = "enforce"
  description = "B2C personal default-on per memory feedback_mls_rfc_9420_e2ee_personal_messenger."
}

variable "mobile_app_bundle_peers" {
  type    = list(string)
  default = ["mail", "social", "community"]
}

variable "compliance_packs" {
  type    = list(string)
  default = ["soc2"]
}

variable "regulatory_packs" {
  type    = list(string)
  default = ["us", "eu", "kr", "jp", "sg"]
}

provider "kubernetes" {
  host                   = var.k8s_cluster_endpoint
  cluster_ca_certificate = base64decode(var.k8s_ca_cert)
  token                  = var.k8s_token
}

resource "kubernetes_namespace" "messenger" {
  metadata {
    name = "messenger-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "messenger"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/oyatie-cell-id"     = var.oyatie_cell_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/deployment-context" = "oyatie-as-cloud-provider"
      "oyatie.io/data-class"         = "PII_IDENTIFYING"
      "oyatie.io/audience-mode"      = var.audience_mode
      "oyatie.io/mls-e2ee-mode"      = var.mls_e2ee_mode
      "oyatie.io/mobile-app-bundle"  = "messenger+mail+social+community"
      "oyatie.io/billing-emit"       = "true"
    }
  }
}

resource "helm_release" "messenger" {
  name      = "messenger"
  namespace = kubernetes_namespace.messenger.metadata[0].name
  chart     = "${path.module}/../helm/messenger"
  values = [yamlencode({
    image                     = { repository = "registry.oyatie.dev/messenger", tag = "1.0.0-wave-15a" }
    tenantId                  = var.tenant_id
    oyatieCellId              = var.oyatie_cell_id
    tenantClass               = var.tenant_class
    paidBillingComponents     = var.tenant_class == "paid" ? var.paid_billing_components : []
    audienceMode              = var.audience_mode
    mlsE2eeMode               = var.mls_e2ee_mode
    mobileAppBundlePeers      = var.mobile_app_bundle_peers
    billingEmitToCloudBilling = true
    compliancePacks           = var.compliance_packs
    regulatoryPacks           = var.regulatory_packs
    paidBillingComponentsEmitted = [
      "messenger.message.sent_per_million",
      "messenger.channel.active_per_month",
      "messenger.huddle.minute_per_thousand",
      "messenger.attachment.byte_per_gb",
      "messenger.mls.key_package.uploaded_per_thousand",
      "messenger.search.query_per_thousand",
      "messenger.mention.fanout_per_million",
      "messenger.workflow.trigger_per_thousand"
    ]
    demoTrialUsageCaps = var.tenant_class == "demo_trial" ? {
      monthlyActiveUsersCap = 25
      channelsCap           = 10
      huddleMinutesCap      = 120
      attachmentGbCap       = 2
    } : null
    resources = var.tenant_class == "demo_trial" ? {
      requests = { cpu = "0.25", memory = "1Gi" }
      limits   = { cpu = "0.5", memory = "2Gi" }
      } : {
      requests = { cpu = "1", memory = "4Gi" }
      limits   = { cpu = "4", memory = "16Gi" }
    }
    env = {
      DEFAULT_MLS_CIPHERSUITE  = "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519"
      MLS_KEY_PACKAGE_TTL_DAYS = "7"
      MLS_WELCOME_TTL_DAYS     = "14"
    }
  })]
}

output "messenger_namespace" { value = kubernetes_namespace.messenger.metadata[0].name }
output "tenant_class" { value = var.tenant_class }
output "audience_mode" { value = var.audience_mode }
output "mls_e2ee_mode" { value = var.mls_e2ee_mode }
