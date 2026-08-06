// OpenTofu module — ED-IS colocation deployment.
// Authority: ADR-0332 (in flight) | feedback_zero_handroll_opentofu_only_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Variant of on-prem for customer colocation hardware. Differs in network
// fabric assumptions (provider-supplied L2/L3) and hardware bonding.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.13"
    }
  }
}

variable "tenant_id"     { type = string }
variable "cell_id"       { type = string }
variable "kubeconfig_path" { type = string }
variable "image_tag"     { type = string }
variable "colo_provider" {
  type = string
  default = "equinix"
}
variable "compliance_packs" {
  type    = list(string)
  default = ["HIPAA", "SOC2", "HITRUST", "EMTALA"]
}

provider "kubernetes" {
  config_path = var.kubeconfig_path
}

provider "helm" {
  kubernetes {
    config_path = var.kubeconfig_path
  }
}

resource "kubernetes_namespace_v1" "emergency_ns" {
  metadata {
    name = "emergency-${var.tenant_id}-${var.cell_id}"
    labels = {
      microservice = "emergency"
      tenant_id    = var.tenant_id
      cell_id      = var.cell_id
      deployment_context = "colo"
      colo_provider = var.colo_provider
    }
  }
}

resource "helm_release" "emergency_chart" {
  name      = "emergency"
  chart     = "../_charts/emergency"
  namespace = kubernetes_namespace_v1.emergency_ns.metadata[0].name
  values = [
    yamlencode({
      image = { tag = var.image_tag }
      tenant_id = var.tenant_id
      cell_id   = var.cell_id
      compliance_packs = var.compliance_packs
      colo_provider = var.colo_provider
    })
  ]
}

output "emergency_endpoint" {
  value = "https://emergency.${var.tenant_id}.${var.cell_id}.colo.${var.colo_provider}.oyatie.cloud"
}
