# OpenTofu module — messenger µservice — context: on-prem
# Customer-owned data center; sovereign-cell common for KR-FSS / EU / public-sector buyers
# who require on-premise control of E2EE messaging keys and audit chain.
# Targets RHEL / Oracle Linux / SUSE / Ubuntu LTS / Debian / Rocky / AlmaLinux /
# CentOS Stream / Flatcar / Photon on bare-metal or vSphere.
# Wave 15A-MESSENGER-FIX authored 2026-05-21

variable "tenant_id" { type = string }
variable "site_id" {
  type        = string
  description = "Customer DC site identifier"
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
  type    = bool
  default = false
}

variable "hsm_endpoint" {
  type        = string
  default     = ""
  description = "Optional HSM endpoint for MLS server-signing-key + welcome-decryption-key escrow (FSS-class sovereign deployments)"
}

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331; on-prem implies paid contract (no on-prem demo_trial)."
  validation {
    condition     = var.tenant_class == "paid"
    error_message = "on-prem context requires tenant_class=paid (no on-prem demo_trial)."
  }
}

variable "paid_billing_components" {
  type    = list(string)
  default = ["per_seat", "per_usage"]
}

variable "audience_mode" {
  type    = string
  default = "B2B-work"
}

variable "mls_e2ee_mode" {
  type        = string
  default     = "enforce"
  description = "On-prem messenger defaults to MLS enforce per sovereign-cell + memory feedback_mls_rfc_9420_e2ee_personal_messenger."
}

variable "mobile_app_bundle_peers" {
  type    = list(string)
  default = ["mail", "social", "community"]
}

variable "compliance_packs" {
  type    = list(string)
  default = ["iso27001", "kr-isms-p"]
}

variable "regulatory_packs" {
  type    = list(string)
  default = ["kr", "eu"]
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
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/site_id"            = var.site_id
      "oyatie.io/deployment-context" = "on-prem"
      "oyatie.io/data-class"         = "PII_IDENTIFYING,AUDIT"
      "oyatie.io/sovereign-cell"     = tostring(var.sovereign_cell)
      "oyatie.io/air-gap-mode"       = tostring(var.air_gap_mode)
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
  values = [yamlencode({
    image = {
      repository = var.air_gap_mode ? "registry.internal.${var.site_id}/messenger" : "registry.oyatie.dev/messenger"
      tag        = "1.0.0-wave-15a"
    }
    tenantId                = var.tenant_id
    tenantClass             = var.tenant_class
    paidBillingComponents   = var.paid_billing_components
    audienceMode            = var.audience_mode
    mlsE2eeMode             = var.mls_e2ee_mode
    mobileAppBundlePeers    = var.mobile_app_bundle_peers
    sovereign               = var.sovereign_cell
    airGapMode              = var.air_gap_mode
    hsmEndpoint             = var.hsm_endpoint
    egressBlocked           = var.air_gap_mode
    compliancePacks         = var.compliance_packs
    regulatoryPacks         = var.regulatory_packs
    multiRegionActiveActive = false
    env = {
      DEFAULT_MLS_CIPHERSUITE  = "MLS_256_DHKEMP384_AES256GCM_SHA384_P384"
      MLS_KEY_PACKAGE_TTL_DAYS = "7"
      MLS_WELCOME_TTL_DAYS     = "14"
    }
  })]
}

output "messenger_namespace" { value = kubernetes_namespace.messenger.metadata[0].name }
output "sovereign_cell" { value = var.sovereign_cell }
output "air_gap_mode" { value = var.air_gap_mode }
output "tenant_class" { value = var.tenant_class }
output "mls_e2ee_mode" { value = var.mls_e2ee_mode }
