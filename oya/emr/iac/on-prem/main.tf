# OpenTofu module — EMR µservice — context: on-prem
# Customer-owned data center; sovereign-cell common for healthcare
# Targets RHEL / Oracle Linux / SUSE / Ubuntu / Photon-OS on bare-metal or vSphere
# Wave 15M-B authored 2026-05-21

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.13" }
    vsphere = { source = "hashicorp/vsphere", version = "~> 2.8" }
  }
}

variable "tenant_id" { type = string }
variable "site_id" {
  type = string
  description = "Customer DC site identifier"
}
variable "k8s_cluster_endpoint" { type = string }
variable "k8s_ca_cert" {
  type = string
  sensitive = true
}
variable "k8s_token" {
  type = string
  sensitive = true
}
variable "sovereign_cell" {
  type = bool
  default = true
}
variable "air_gap_mode" {
  type = bool
  default = false
}
variable "hsm_endpoint" {
  type = string
  default = ""
}

provider "kubernetes" {
  host                   = var.k8s_cluster_endpoint
  cluster_ca_certificate = base64decode(var.k8s_ca_cert)
  token                  = var.k8s_token
}

resource "kubernetes_namespace" "emr" {
  metadata {
    name = "emr-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"     = "emr"
      "oyatie.io/tenant_id"        = var.tenant_id
      "oyatie.io/site_id"          = var.site_id
      "oyatie.io/data-class"       = "phi-protected-health-information"
      "oyatie.io/compliance-pack"  = "HIPAA-2024"
      "oyatie.io/sovereign-cell"   = tostring(var.sovereign_cell)
      "oyatie.io/air-gap-mode"     = tostring(var.air_gap_mode)
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"
  values = [yamlencode({
    image = {
      repository = var.air_gap_mode ? "registry.internal.${var.site_id}/emr" : "registry.oyatie.health/emr"
      tag        = "1.0.0-wave-15m-b"
    }
    tenantId      = var.tenant_id
    sovereign     = var.sovereign_cell
    airGapMode    = var.air_gap_mode
    hsmEndpoint   = var.hsm_endpoint
    egressBlocked = var.air_gap_mode
    compliancePacksRequired = ["HIPAA-2024"]
    multiRegionActiveActive = false
  })]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
output "sovereign_cell" { value = var.sovereign_cell }
output "air_gap_mode" { value = var.air_gap_mode }
