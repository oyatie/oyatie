# colo OpenTofu module for the imaging µservice.
# Authority: ADR-0131 + zero-handroll OpenTofu-only.
#
# Use case: co-located hardware adjacent to modalities for sub-second
# image-pull SLO. Common for very-large hospital systems with regional
# imaging center networks.
#
# Substrate: Talos K8s on bare-metal; CIDR aligned with modality VLAN.

terraform {
  required_version = ">= 1.7"
  required_providers {
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id"   { type = string }
variable "modality_vlan_cidr" {
  type = string
  default = "10.50.0.0/16"
}
variable "kubeconfig_path"    { type = string }

provider "kubernetes" {
  config_path = var.kubeconfig_path
}

provider "helm" {
  kubernetes {
    config_path = var.kubeconfig_path
  }
}

resource "kubernetes_namespace" "imaging" {
  metadata {
    name = "imaging-colo-${var.tenant_id}-${var.cell_id}"
    labels = {
      "oyatie.microservice" = "imaging"
      "oyatie.tenant"       = var.tenant_id
      "oyatie.cell"         = var.cell_id
      "oyatie.context"      = "colo"
    }
  }
}

resource "helm_release" "imaging" {
  name       = "imaging"
  namespace  = kubernetes_namespace.imaging.metadata[0].name
  chart      = "../helm/imaging"
  values = [
    yamlencode({
      tenant_id = var.tenant_id
      cell_id   = var.cell_id
      context   = "colo"
      pacs = {
        replicas = 5
        edge_cache_enabled = true
      }
      dimse = {
        replicas = 10
        modality_vlan_cidr = var.modality_vlan_cidr
        tls = true
      }
      dicomweb = {
        replicas = 10
        http3_enabled = true
        edge_cdn_for_thumbnails = true
      }
      vna = {
        local_cache_tb = 50
        upstream_sync_target = "sovereign-cell"
      }
    })
  ]
}

output "namespace" { value = kubernetes_namespace.imaging.metadata[0].name }
