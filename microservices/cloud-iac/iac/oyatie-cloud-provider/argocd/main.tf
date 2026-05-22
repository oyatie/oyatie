terraform {
  required_version = ">= 1.6"
}

locals {
  component = "argocd"

  common_labels = merge({
    "app.kubernetes.io/name"        = local.component
    "app.kubernetes.io/part-of"     = "oya-ci-cd-substrate"
    "app.kubernetes.io/component"   = "gitops-controller"
    "app.kubernetes.io/managed-by"  = "opentofu"
    "oyatie.com/adr"                = "ADR-0349"
    "oyatie.com/context"            = var.context
    "oyatie.com/tenant-id"          = var.tenant_id
    "oyatie.com/tenant-class"       = var.tenant_class
    "oyatie.com/license-class"      = "contributor"
    "oyatie.com/license-expression" = "Apache-2.0"
  }, var.tags)

  namespace_manifest = {
    apiVersion = "v1"
    kind       = "Namespace"
    metadata = {
      name = var.namespace
      labels = merge(local.common_labels, {
        "istio.io/dataplane-mode" = "ambient"
      })
    }
  }

  service_account_manifest = {
    apiVersion = "v1"
    kind       = "ServiceAccount"
    metadata = {
      name      = "argocd-server"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "oyatie.com/openbao-mount" = var.openbao_mount_path
      }
    }
  }

  deployment_manifest = {
    apiVersion = "apps/v1"
    kind       = "Deployment"
    metadata = {
      name      = "argocd-server"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "cosign.sigstore.dev/required"  = "true"
        "oyatie.com/image-digest"       = var.argocd_image_digest
        "oyatie.com/adr-0181-promotion" = "required"
      }
    }
    spec = {
      replicas = 2
      selector = { matchLabels = { "app.kubernetes.io/name" = local.component } }
      template = {
        metadata = {
          labels = merge(local.common_labels, {
            "istio.io/dataplane-mode" = "ambient"
          })
          annotations = {
            "cosign.sigstore.dev/required" = "true"
            "oyatie.com/audit-chain-event" = "argocd-controller-start"
          }
        }
        spec = {
          runtimeClassName   = "kata-cloud-hypervisor"
          serviceAccountName = "argocd-server"
          securityContext = {
            runAsNonRoot = true
            fsGroup      = 999
          }
          containers = [{
            name            = "argocd-server"
            image           = "${var.argocd_image}@${var.argocd_image_digest}"
            imagePullPolicy = "IfNotPresent"
            args            = ["argocd-server", "--insecure=false"]
            ports = [
              { name = "https", containerPort = 8080 },
              { name = "grpc", containerPort = 8083 }
            ]
            env = [
              { name = "OYA_CD_CONTEXT", value = var.context },
              { name = "OYA_REPO_URL", value = var.repo_url },
              { name = "OYA_TARGET_REVISION", value = var.target_revision }
            ]
            resources = {
              requests = { cpu = "500m", memory = "1Gi" }
              limits   = { cpu = "2000m", memory = "4Gi" }
            }
            securityContext = {
              allowPrivilegeEscalation = false
              readOnlyRootFilesystem   = true
              capabilities             = { drop = ["ALL"] }
            }
          }]
        }
      }
    }
  }

  service_manifest = {
    apiVersion = "v1"
    kind       = "Service"
    metadata = {
      name      = "argocd-server"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      type     = "ClusterIP"
      selector = { "app.kubernetes.io/name" = local.component }
      ports = [
        { name = "https", port = 443, targetPort = 8080 },
        { name = "grpc", port = 8083, targetPort = 8083 }
      ]
    }
  }

  project_manifest = {
    apiVersion = "argoproj.io/v1alpha1"
    kind       = "AppProject"
    metadata = {
      name      = "oya-oyatie-cloud-provider-tenants"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      description  = "Tenant-isolated ArgoCD project for oyatie-cloud-provider."
      sourceRepos  = [var.repo_url]
      destinations = [{ namespace = "*", server = "https://kubernetes.default.svc" }]
      clusterResourceWhitelist = [
        { group = "", kind = "Namespace" },
        { group = "gateway.networking.k8s.io", kind = "HTTPRoute" },
        { group = "security.istio.io", kind = "AuthorizationPolicy" }
      ]
    }
  }

  cosign_policy_config_manifest = {
    apiVersion = "v1"
    kind       = "ConfigMap"
    metadata = {
      name      = "argocd-cosign-policy"
      namespace = var.namespace
      labels    = local.common_labels
    }
    data = {
      "policy.yaml" = <<-YAML
      imagePromotion:
        adr: ADR-0181
        requireCosignVerification: true
        failOpen: false
        auditChainEvent: argocd-image-signature-verified
      YAML
    }
  }

  kubernetes_manifests = [
    local.namespace_manifest,
    local.service_account_manifest,
    local.deployment_manifest,
    local.service_manifest,
    local.project_manifest,
    local.cosign_policy_config_manifest
  ]
}
