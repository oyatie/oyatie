# OpenTofu module — messenger µservice — context: colo
# Customer-rented or owned hardware in colocation facility (OVH / Equinix /
# Hetzner / Telehouse / Digital Realty / KINX / NaverCloud / etc.) under EU GDPR
# Article 9 + Schrems II sovereignty preferences or KR data-sovereignty rules.
# Wave 15A-MESSENGER-FIX authored 2026-05-21

variable "tenant_id" { type = string }
variable "colo_provider" {
  type        = string
  description = "OVH / Equinix / Hetzner / Telehouse / Digital Realty / KINX / NaverCloud / etc."
}
variable "colo_region" { type = string }
variable "k8s_cluster_endpoint" { type = string }
variable "k8s_ca_cert" {
  type      = string
  sensitive = true
}
variable "k8s_token" {
  type      = string
  sensitive = true
}
variable "sovereign_jurisdiction" {
  type        = string
  description = "EU / KR / KSA / JP / SG / AE / etc."
}

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331; colo implies paid contract."
  validation {
    condition     = var.tenant_class == "paid"
    error_message = "colo context requires tenant_class=paid."
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
  type    = string
  default = "enforce"
}

variable "mobile_app_bundle_peers" {
  type    = list(string)
  default = ["mail", "social", "community"]
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
      "oyatie.io/microservice"           = "messenger"
      "oyatie.io/tenant_id"              = var.tenant_id
      "oyatie.io/tenant-class"           = var.tenant_class
      "oyatie.io/colo-provider"          = var.colo_provider
      "oyatie.io/colo-region"            = var.colo_region
      "oyatie.io/sovereign-jurisdiction" = var.sovereign_jurisdiction
      "oyatie.io/deployment-context"     = "colo"
      "oyatie.io/data-class"             = "PII_IDENTIFYING"
      "oyatie.io/audience-mode"          = var.audience_mode
      "oyatie.io/mls-e2ee-mode"          = var.mls_e2ee_mode
      "oyatie.io/mobile-app-bundle"      = "messenger+mail+social+community"
    }
  }
}

resource "helm_release" "messenger" {
  name      = "messenger"
  namespace = kubernetes_namespace.messenger.metadata[0].name
  chart     = "${path.module}/../helm/messenger"
  values = [yamlencode({
    image                 = { repository = "registry.oyatie.dev/messenger", tag = "1.0.0-wave-15a" }
    tenantId              = var.tenant_id
    tenantClass           = var.tenant_class
    paidBillingComponents = var.paid_billing_components
    audienceMode          = var.audience_mode
    mlsE2eeMode           = var.mls_e2ee_mode
    mobileAppBundlePeers  = var.mobile_app_bundle_peers
    sovereignJurisdiction = var.sovereign_jurisdiction
    coloProvider          = var.colo_provider
    coloRegion            = var.colo_region
    compliancePacks = (
      var.sovereign_jurisdiction == "EU" ? ["gdpr", "iso27001", "NIS2"] :
      var.sovereign_jurisdiction == "KR" ? ["kr-pipa", "kr-isms-p", "iso27001"] :
      var.sovereign_jurisdiction == "KSA" ? ["ksa-pdpl", "ksa-csap", "iso27001"] :
      ["iso27001"]
    )
    env = {
      DEFAULT_MLS_CIPHERSUITE  = "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519"
      MLS_KEY_PACKAGE_TTL_DAYS = "7"
      MLS_WELCOME_TTL_DAYS     = "14"
    }
  })]
}

output "messenger_namespace" { value = kubernetes_namespace.messenger.metadata[0].name }
output "sovereign_jurisdiction" { value = var.sovereign_jurisdiction }
output "tenant_class" { value = var.tenant_class }
output "mls_e2ee_mode" { value = var.mls_e2ee_mode }
