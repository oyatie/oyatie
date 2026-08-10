# OpenTofu module — EMR µservice — context: colo
# Customer-owned hardware in customer-owned data center, but oyatie manages
# the operational software. Common for EU healthcare networks under EU GDPR
# Article 9 sovereignty preferences.
# Wave 15M-B authored 2026-05-21

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.13" }
  }
}

variable "tenant_id" { type = string }
variable "colo_provider" {
  type = string
  description = "OVH / Equinix / Hetzner / Telehouse / Digital Realty / etc."
}
variable "colo_region" { type = string }
variable "k8s_cluster_endpoint" { type = string }
variable "k8s_ca_cert" {
  type = string
  sensitive = true
}
variable "k8s_token" {
  type = string
  sensitive = true
}
variable "sovereign_jurisdiction" {
  type = string
  description = "EU / KR / KSA / JP / etc."
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
      "oyatie.io/microservice"            = "emr"
      "oyatie.io/tenant_id"               = var.tenant_id
      "oyatie.io/colo-provider"           = var.colo_provider
      "oyatie.io/colo-region"             = var.colo_region
      "oyatie.io/sovereign-jurisdiction"  = var.sovereign_jurisdiction
      "oyatie.io/data-class"              = "phi-protected-health-information"
      "oyatie.io/compliance-pack"         = "HIPAA-2024"
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"
  values = [yamlencode({
    image = { repository = "registry.oyatie.health/emr", tag = "1.0.0-wave-15m-b" }
    tenantId             = var.tenant_id
    sovereignJurisdiction = var.sovereign_jurisdiction
    coloProvider         = var.colo_provider
    coloRegion           = var.colo_region
    compliancePacksRequired = (
      var.sovereign_jurisdiction == "EU"
        ? ["HIPAA-2024", "EU-GDPR-2018-baseline"]
        : (var.sovereign_jurisdiction == "KR"
          ? ["HIPAA-2024", "KR-PIPA-2023-amendment", "KR-MEDICAL-LAW-2024"]
          : ["HIPAA-2024"])
    )
  })]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
output "sovereign_jurisdiction" { value = var.sovereign_jurisdiction }
