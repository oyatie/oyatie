// OpenTofu module — ED-IS on-premise deployment.
// Authority: ADR-0332 (in flight) | feedback_zero_handroll_opentofu_only_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Targets customer-controlled on-prem hospital infrastructure.
// Backends: Talos / KubeVirt / vSphere + Postgres operator + Valkey operator +
// NATS cluster + customer KMS / HSM.

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

variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "kubernetes_host" { type = string }
variable "kubeconfig_path" { type = string }
variable "image_tag" { type = string }
variable "compliance_packs" {
  type    = list(string)
  default = ["HIPAA", "SOC2", "HITRUST", "EMTALA", "TJC", "ACS-Trauma-Verification"]
}
variable "customer_kms_endpoint" { type = string }
variable "byok_required" {
  type    = bool
  default = true
}

provider "kubernetes" {
  config_path = var.kubeconfig_path
}

provider "helm" {
  kubernetes {
    config_path = var.kubeconfig_path
  }
}

module "postgres_operator" {
  source    = "../_shared/on-prem-postgres-operator"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
}

module "valkey_operator" {
  source    = "../_shared/on-prem-valkey-operator"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
}

module "nats_cluster" {
  source        = "../_shared/on-prem-nats-cluster"
  tenant_id     = var.tenant_id
  cell_id       = var.cell_id
  stream_prefix = "ed"
}

resource "kubernetes_namespace_v1" "emergency_ns" {
  metadata {
    name = "emergency-${var.tenant_id}-${var.cell_id}"
    labels = {
      microservice       = "emergency"
      tenant_id          = var.tenant_id
      cell_id            = var.cell_id
      deployment_context = "on-prem"
    }
  }
}

resource "helm_release" "emergency_chart" {
  name      = "emergency"
  chart     = "../_charts/emergency"
  namespace = kubernetes_namespace_v1.emergency_ns.metadata[0].name
  values = [
    yamlencode({
      image                 = { tag = var.image_tag }
      tenant_id             = var.tenant_id
      cell_id               = var.cell_id
      compliance_packs      = var.compliance_packs
      byok_required         = var.byok_required
      customer_kms_endpoint = var.customer_kms_endpoint
      db                    = { url = module.postgres_operator.url }
      valkey                = { url = module.valkey_operator.url }
      nats                  = { url = module.nats_cluster.url }
    })
  ]
}

output "emergency_endpoint" {
  value = "https://emergency.${var.tenant_id}.${var.cell_id}.onprem.local"
}
