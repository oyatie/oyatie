# api-gateway Terraform module
# Per ADR-0157 + ADR-0254. Per-cell deploy unit.
terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.12" }
    cloudflare = { source = "cloudflare/cloudflare", version = "~> 4.30" }
  }
}

variable "cell_id" { type = string }
variable "cell_jurisdiction" { type = string }
variable "compliance_packs" { type = list(string) }
variable "envoy_image" { type = string; default = "envoyproxy/envoy:v1.32-distroless" }
variable "replicas_min" { type = number; default = 4 }
variable "replicas_max" { type = number; default = 32 }

resource "kubernetes_namespace" "api_gateway" {
  metadata { name = "api-gateway" }
}

resource "helm_release" "api_gateway" {
  name = "api-gateway"
  chart = "${path.module}/../helm/api-gateway"
  namespace = kubernetes_namespace.api_gateway.metadata[0].name
  values = [
    yamlencode({
      cell_id = var.cell_id
      cell_jurisdiction = var.cell_jurisdiction
      compliance_packs = var.compliance_packs
      envoy = { image = var.envoy_image; replicas = { min = var.replicas_min, max = var.replicas_max } }
    })
  ]
}

resource "cloudflare_zone_settings_override" "api_gateway" {
  zone_id = var.cloudflare_zone_id
  settings { tls_1_3 = "on"; min_tls_version = "1.3"; http3 = "on"; opportunistic_encryption = "on"; security_level = "medium"; brotli = "on" }
}

output "envoy_service_name" { value = "api-gateway-envoy.${kubernetes_namespace.api_gateway.metadata[0].name}.svc.cluster.local" }
