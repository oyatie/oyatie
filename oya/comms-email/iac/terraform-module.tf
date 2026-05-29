# comms-email — Terraform module per ADR-0254

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm",       version = "~> 2.13" }
    openbao    = { source = "openbao/openbao",      version = "~> 0.4" }
  }
}

variable "regions" {
  type    = list(string)
  default = ["us-east-1", "us-west-2", "eu-central-1", "ap-northeast-2"]
}

variable "provider_default_per_region" {
  type = map(string)
  default = {
    "us-east-1"      = "ses"
    "us-west-2"      = "ses"
    "eu-central-1"   = "postal-eu"
    "ap-northeast-2" = "postal-kr"
  }
}

resource "kubernetes_namespace" "comms_email" {
  metadata {
    name = "oya-comms-email"
    labels = {
      "oyatie.io/microservice" = "comms-email"
      "oyatie.io/cell-tier"    = "tier-1"
    }
  }
}

resource "helm_release" "outbound" {
  for_each = toset(var.regions)
  name      = "comms-email-${each.value}"
  namespace = kubernetes_namespace.comms_email.metadata[0].name
  chart     = "${path.module}/helm/postal"
  values = [
    yamlencode({
      region         = each.value
      defaultProvider = var.provider_default_per_region[each.value]
    })
  ]
}
