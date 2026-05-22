locals {
  gateway_manifest = {
    apiVersion = "gateway.networking.k8s.io/v1"
    kind       = "Gateway"
    metadata = {
      name      = "jenkins-gateway"
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
        hostname = var.jenkins_host
        port     = 443
        protocol = "HTTPS"
        tls = {
          mode            = "Terminate"
          certificateRefs = [{ name = "jenkins-gateway-tls" }]
        }
      }]
    }
  }

  http_route_manifest = {
    apiVersion = "gateway.networking.k8s.io/v1"
    kind       = "HTTPRoute"
    metadata = {
      name      = "jenkins-route"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "oyatie.com/ambient-waypoint-required" = "true"
      }
    }
    spec = {
      parentRefs = [{ name = "jenkins-gateway" }]
      hostnames  = [var.jenkins_host]
      rules = [{
        backendRefs = [{ name = "jenkins", port = 8080 }]
      }]
    }
  }

  authorization_policy_manifest = {
    apiVersion = "security.istio.io/v1beta1"
    kind       = "AuthorizationPolicy"
    metadata = {
      name      = "jenkins-tenant-access"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      selector = { matchLabels = { "app.kubernetes.io/name" = local.component } }
      action   = "ALLOW"
      rules = [{
        from = [{ source = { namespaces = [var.namespace] } }]
        to   = [{ operation = { ports = ["8080"] } }]
      }]
    }
  }

  ingress_manifests = [
    local.gateway_manifest,
    local.http_route_manifest,
    local.authorization_policy_manifest
  ]
}
