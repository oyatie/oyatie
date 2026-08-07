terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm",       version = "~> 2.13" }
  }
}

variable "regions" {
  type    = list(string)
  default = ["us-east-1", "eu-central-1", "ap-northeast-2"]
}

variable "compliance_packs" {
  type    = list(string)
  default = ["generic", "kr", "eu", "us-healthcare", "us-financial"]
}

resource "kubernetes_namespace" "finops_portal" {
  metadata {
    name = "oya-finops-portal"
    labels = {
      "oyatie.io/microservice" = "finops-portal"
      "oyatie.io/cell-tier"    = "tier-1"
    }
  }
}

resource "helm_release" "finops_portal" {
  for_each  = toset(var.regions)
  name      = "finops-portal-${each.value}"
  namespace = kubernetes_namespace.finops_portal.metadata[0].name
  chart     = "${path.module}/helm/finops-portal"
  values = [
    yamlencode({
      region          = each.value
      compliancePacks = var.compliance_packs
    })
  ]
}
