# on-prem OpenTofu module for the imaging µservice.
# Authority: ADR-0131 + zero-handroll OpenTofu-only.
#
# Most-common deployment for hospital PACS. Customer-controlled hardware in
# their data center. Talos / RHEL 9 / SUSE / Photon (VMware) supported.
#
# Substrate: K8s on Talos for greenfield; helm-chart for brownfield
# RHEL-OpenShift / SUSE-Rancher / VMware-vSphere-with-Tanzu.

terraform {
  required_version = ">= 1.7"
  required_providers {
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
  }
}

variable "tenant_id"              { type = string }
variable "cell_id"                { type = string }
variable "kubeconfig_path"        { type = string }
variable "talos_or_openshift_or_rancher" {
  type = string
  default = "talos"
}
variable "modalities" {
  type = list(object({
    name        = string
    ae_title    = string
    host        = string
    port        = number
    sop_classes = list(string)
  }))
  default = []
}

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
    name = "imaging-${var.tenant_id}-${var.cell_id}"
    labels = {
      "oyatie.microservice" = "imaging"
      "oyatie.tenant"       = var.tenant_id
      "oyatie.cell"         = var.cell_id
      "oyatie.context"      = "on-prem"
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
      context   = "on-prem"
      pacs = {
        replicas = 3
        resources = {
          requests = { cpu = "2", memory = "8Gi" }
          limits   = { cpu = "8", memory = "32Gi" }
        }
      }
      dimse = {
        replicas = 5
        ae_title_pairings = var.modalities
        port = 11112
        tls = true
      }
      dicomweb = {
        replicas = 5
        http3_enabled = true
      }
      vna = {
        durability_nines = 13
        erasure_coding = "14+4"
      }
      ai_marketplace = {
        enabled = true
        vendors = []
      }
      compliance_packs = ["HIPAA-2024", "MQSA", "ACR-Accreditation"]
    })
  ]
}

resource "kubernetes_config_map" "modality_registry" {
  metadata {
    name      = "modality-registry"
    namespace = kubernetes_namespace.imaging.metadata[0].name
  }
  data = {
    modalities = jsonencode(var.modalities)
  }
}

output "namespace" { value = kubernetes_namespace.imaging.metadata[0].name }
