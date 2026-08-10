# OpenTofu module — EMR µservice — context: guest-on-oci
# Tenant's own OCI account; OKE cluster; BAA-eligible
# Per memory feedback_oci_always_free_maximization — demo_trial tenants may use Always Free
# Wave 15M-B authored 2026-05-21

terraform {
  required_version = ">= 1.7"
  required_providers {
    oci = { source = "oracle/oci", version = "~> 6.0" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.13" }
  }
}

variable "tenant_id" { type = string }
variable "oci_compartment_id" { type = string }
variable "oci_region" {
  type = string
  default = "us-ashburn-1"
}
variable "oke_cluster_id" { type = string }
variable "tenant_vault_id" { type = string }
variable "tenant_kms_key_id" { type = string }
variable "always_free_eligible" {
  type = bool
  default = false
}

resource "oci_identity_policy" "emr_iam" {
  compartment_id = var.oci_compartment_id
  name           = "emr-iam-${var.tenant_id}"
  description    = "EMR µservice permissions for tenant ${var.tenant_id}"
  statements = [
    "allow dynamic-group emr-${var.tenant_id} to use vaults in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group emr-${var.tenant_id} to use keys in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group emr-${var.tenant_id} to read objectstorage-namespaces in compartment id ${var.oci_compartment_id}"
  ]
}

resource "kubernetes_namespace" "emr" {
  metadata {
    name = "emr-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice" = "emr"
      "oyatie.io/tenant_id"    = var.tenant_id
      "oyatie.io/data-class"   = "phi-protected-health-information"
      "oyatie.io/compliance-pack" = "HIPAA-2024"
      "oyatie.io/always-free"  = tostring(var.always_free_eligible)
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"
  values = [yamlencode({
    image = { repository = "registry.oyatie.health/emr", tag = "1.0.0-wave-15m-b" }
    tenantId = var.tenant_id
    byok = { enabled = true, vaultId = var.tenant_vault_id, kmsKeyId = var.tenant_kms_key_id }
    alwaysFreeEligible = var.always_free_eligible
    compliancePacksRequired = ["HIPAA-2024"]
    resources = var.always_free_eligible ? {
      requests = { cpu = "0.5", memory = "2Gi" }
      limits   = { cpu = "1", memory = "4Gi" }
    } : {
      requests = { cpu = "2", memory = "8Gi" }
      limits   = { cpu = "8", memory = "32Gi" }
    }
  })]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
output "always_free_eligible" { value = var.always_free_eligible }
