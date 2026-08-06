# compliance µservice — Terraform module
# Binding: ADR-0254 (K8s + Cloud Hypervisor + Kata)
# Multi-region per `multi-region.md`

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm",       version = "~> 2.13" }
    openbao    = { source = "openbao/openbao",      version = "~> 0.4" }
  }
}

variable "cell_tier" {
  type        = string
  description = "Cell tier (tier-0..tier-3) per ADR-0248"
  default     = "tier-1"
}

variable "regions" {
  type        = list(string)
  description = "Regions to deploy to; honours multi-region.md home/dr pairing"
  default     = ["us-east-1", "eu-central-1", "ap-northeast-2"]
}

variable "compliance_packs" {
  type        = list(string)
  description = "Active compliance packs"
  default     = ["soc2-type-2", "gdpr", "hipaa", "pci-dss"]
}

resource "kubernetes_namespace" "compliance" {
  metadata {
    name = "oya-compliance"
    labels = {
      "oyatie.io/microservice" = "compliance"
      "oyatie.io/cell-tier"    = var.cell_tier
    }
  }
}

resource "helm_release" "evidence_collector" {
  name      = "compliance-evidence-collector"
  namespace = kubernetes_namespace.compliance.metadata[0].name
  chart     = "${path.module}/helm/evidence-collector"
  values = [
    yamlencode({
      compliancePacks = var.compliance_packs
      cellTier        = var.cell_tier
    })
  ]
}
