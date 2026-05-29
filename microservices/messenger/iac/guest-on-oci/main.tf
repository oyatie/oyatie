# OpenTofu module — messenger µservice — context: guest-on-oci
# Tenant's own OCI tenancy; OKE cluster; demo_trial tenants may use Always Free.
# Per memory feedback_oci_always_free_maximization_2026_05_20:
#   - demo_trial / sandbox / dev tenants → Always Free shape (2× Ampere A1 ARM 4 OCPU+24GB)
#   - paid tenants → standard OCI shapes
# Wave 15A-MESSENGER-FIX authored 2026-05-21

variable "tenant_id" { type = string }
variable "oci_compartment_id" { type = string }
variable "oci_region" {
  type    = string
  default = "us-ashburn-1"
}
variable "oke_cluster_id" { type = string }
variable "tenant_vault_id" { type = string }
variable "tenant_kms_key_id" { type = string }

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331 tenant_class ∈ {demo_trial, paid}. demo_trial activates Always Free shape."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be one of: demo_trial, paid."
  }
}

variable "always_free_eligible" {
  type        = bool
  default     = false
  description = "If true AND tenant_class=demo_trial → run on Always Free shape. Per memory feedback_oci_always_free_maximization."
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
  default = "tenant_opt_in_with_recovery_key_escrow"
}

variable "mobile_app_bundle_peers" {
  type    = list(string)
  default = ["mail", "social", "community"]
}

variable "compliance_packs" {
  type    = list(string)
  default = ["soc2", "iso27001"]
}

variable "regulatory_packs" {
  type    = list(string)
  default = ["us", "kr", "jp"]
}

resource "oci_identity_policy" "messenger_iam" {
  compartment_id = var.oci_compartment_id
  name           = "messenger-iam-${var.tenant_id}"
  description    = "messenger µservice permissions for tenant ${var.tenant_id}"
  statements = [
    "allow dynamic-group messenger-${var.tenant_id} to use vaults in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group messenger-${var.tenant_id} to use keys in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group messenger-${var.tenant_id} to read objectstorage-namespaces in compartment id ${var.oci_compartment_id}"
  ]
}

resource "kubernetes_namespace" "messenger" {
  metadata {
    name = "messenger-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "messenger"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/deployment-context" = "guest-on-oci"
      "oyatie.io/data-class"         = "PII_IDENTIFYING"
      "oyatie.io/always-free"        = tostring(var.always_free_eligible)
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
    image                 = { repository = "registry.oyatie.dev/messenger", tag = "1.0.0-wave-15a" }
    tenantId              = var.tenant_id
    tenantClass           = var.tenant_class
    paidBillingComponents = var.paid_billing_components
    audienceMode          = var.audience_mode
    mlsE2eeMode           = var.mls_e2ee_mode
    mobileAppBundlePeers  = var.mobile_app_bundle_peers
    byok                  = { enabled = true, vaultId = var.tenant_vault_id, kmsKeyId = var.tenant_kms_key_id }
    alwaysFreeEligible    = var.always_free_eligible
    compliancePacks       = var.compliance_packs
    regulatoryPacks       = var.regulatory_packs
    resources = var.always_free_eligible ? {
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
output "always_free_eligible" { value = var.always_free_eligible }
output "tenant_class" { value = var.tenant_class }
