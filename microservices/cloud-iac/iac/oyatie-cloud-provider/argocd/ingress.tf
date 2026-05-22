locals {
  gateway_manifest = {
    apiVersion = "gateway.networking.k8s.io/v1"
    kind       = "Gateway"
    metadata = {
      name      = "argocd-gateway"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "oyatie.com/mesh-mode" = "istio-ambient"
        "oyatie.com/adr"       = "ADR-0044"
      }
    }
    spec = {
      gatewayClassName = "istio"
      listeners = [{
        name     = "https"
        hostname = var.argocd_host
        port     = 443
        protocol = "HTTPS"
        tls = {
          mode            = "Terminate"
          certificateRefs = [{ name = "argocd-gateway-tls" }]
        }
      }]
    }
  }

  http_route_manifest = {
    apiVersion = "gateway.networking.k8s.io/v1"
    kind       = "HTTPRoute"
    metadata = {
      name      = "argocd-route"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "oyatie.com/ambient-waypoint-required" = "true"
      }
    }
    spec = {
      parentRefs = [{ name = "argocd-gateway" }]
      hostnames  = [var.argocd_host]
      rules = [{
        backendRefs = [{ name = "argocd-server", port = 443 }]
      }]
    }
  }

  authorization_policy_manifest = {
    apiVersion = "security.istio.io/v1beta1"
    kind       = "AuthorizationPolicy"
    metadata = {
      name      = "argocd-tenant-access"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      selector = { matchLabels = { "app.kubernetes.io/name" = local.component } }
      action   = "ALLOW"
      rules = [{
        from = [{ source = { namespaces = [var.namespace] } }]
        to   = [{ operation = { ports = ["8080", "8083"] } }]
      }]
    }
  }

  ingress_manifests = [
    local.gateway_manifest,
    local.http_route_manifest,
    local.authorization_policy_manifest
  ]
}
