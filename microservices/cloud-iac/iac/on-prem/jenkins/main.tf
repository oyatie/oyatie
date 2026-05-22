terraform {
  required_version = ">= 1.6"
}

locals {
  component = "jenkins"

  common_labels = merge({
    "app.kubernetes.io/name"       = local.component
    "app.kubernetes.io/part-of"    = "oya-ci-cd-substrate"
    "app.kubernetes.io/component"  = "ci-controller"
    "app.kubernetes.io/managed-by" = "opentofu"
    "oyatie.com/adr"               = "ADR-0349"
    "oyatie.com/context"           = var.context
    "oyatie.com/tenant-id"         = var.tenant_id
    "oyatie.com/tenant-class"      = var.tenant_class
    "oyatie.com/runtime-tier"      = "1"
  }, var.tags)

  pod_labels = merge(local.common_labels, {
    "istio.io/dataplane-mode"       = "ambient"
    "sidecar.istio.io/inject"       = "false"
    "oyatie.com/pod-runtime"        = "kata-cloud-hypervisor"
    "oyatie.com/license-class"      = "contributor"
    "oyatie.com/license-expression" = "MIT"
  })

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
      name      = "jenkins-controller"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "oyatie.com/openbao-mount" = var.openbao_mount_path
      }
    }
  }

  jcasc_config_manifest = {
    apiVersion = "v1"
    kind       = "ConfigMap"
    metadata = {
      name      = "jenkins-jcasc"
      namespace = var.namespace
      labels    = local.common_labels
    }
    data = {
      "jenkins.yaml" = file("${path.module}/values/jenkins.yaml")
      "plugins.yaml" = file("${path.module}/values/plugins.yaml")
    }
  }

  persistent_volume_claim_manifest = {
    apiVersion = "v1"
    kind       = "PersistentVolumeClaim"
    metadata = {
      name      = "jenkins-home"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      accessModes      = ["ReadWriteOnce"]
      storageClassName = var.storage_class_name
      resources = {
        requests = {
          storage = var.storage_size
        }
      }
    }
  }

  deployment_manifest = {
    apiVersion = "apps/v1"
    kind       = "Deployment"
    metadata = {
      name      = "jenkins"
      namespace = var.namespace
      labels    = local.common_labels
      annotations = {
        "cosign.sigstore.dev/required"  = "true"
        "oyatie.com/image-digest"       = var.jenkins_image_digest
        "oyatie.com/adr-0181-promotion" = "required"
        "oyatie.com/jenkins-controller" = "lts"
        "oyatie.com/jcasc-only"         = "true"
      }
    }
    spec = {
      replicas = var.replicas
      selector = {
        matchLabels = {
          "app.kubernetes.io/name" = local.component
        }
      }
      template = {
        metadata = {
          labels = local.pod_labels
          annotations = {
            "cosign.sigstore.dev/required"  = "true"
            "oyatie.com/runtime-isolation"  = "kata-cloud-hypervisor"
            "oyatie.com/audit-chain-event"  = "jenkins-controller-start"
            "oyatie.com/agent-image-digest" = var.agent_image_digest
          }
        }
        spec = {
          runtimeClassName   = "kata-cloud-hypervisor"
          serviceAccountName = "jenkins-controller"
          securityContext = {
            runAsNonRoot = true
            fsGroup      = 1000
          }
          containers = [{
            name            = "jenkins"
            image           = "${var.jenkins_image}@${var.jenkins_image_digest}"
            imagePullPolicy = "IfNotPresent"
            ports = [
              { name = "http", containerPort = 8080 },
              { name = "agent", containerPort = 50000 }
            ]
            env = [
              { name = "CASC_JENKINS_CONFIG", value = "/var/jenkins_home/casc_configs/jenkins.yaml" },
              { name = "OYA_CI_CONTEXT", value = var.context },
              { name = "OYA_AGENT_IMAGE", value = "${var.agent_image}@${var.agent_image_digest}" },
              { name = "OYA_OIDC_ISSUER", value = var.oidc_issuer }
            ]
            volumeMounts = [
              { name = "jenkins-home", mountPath = "/var/jenkins_home" },
              { name = "jcasc", mountPath = "/var/jenkins_home/casc_configs", readOnly = true }
            ]
            readinessProbe = {
              httpGet             = { path = "/login", port = 8080 }
              initialDelaySeconds = 30
              periodSeconds       = 10
            }
            livenessProbe = {
              httpGet             = { path = "/login", port = 8080 }
              initialDelaySeconds = 90
              periodSeconds       = 20
            }
            resources = {
              requests = { cpu = "1000m", memory = "2Gi" }
              limits   = { cpu = "4000m", memory = "8Gi" }
            }
            securityContext = {
              allowPrivilegeEscalation = false
              readOnlyRootFilesystem   = false
              capabilities             = { drop = ["ALL"] }
            }
          }]
          volumes = [
            { name = "jenkins-home", persistentVolumeClaim = { claimName = "jenkins-home" } },
            { name = "jcasc", configMap = { name = "jenkins-jcasc" } }
          ]
        }
      }
    }
  }

  service_manifest = {
    apiVersion = "v1"
    kind       = "Service"
    metadata = {
      name      = "jenkins"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      type = "ClusterIP"
      selector = {
        "app.kubernetes.io/name" = local.component
      }
      ports = [
        { name = "http", port = 8080, targetPort = 8080 },
        { name = "agent", port = 50000, targetPort = 50000 }
      ]
    }
  }

  network_policy_manifest = {
    apiVersion = "networking.k8s.io/v1"
    kind       = "NetworkPolicy"
    metadata = {
      name      = "jenkins-tenant-boundary"
      namespace = var.namespace
      labels    = local.common_labels
    }
    spec = {
      podSelector = { matchLabels = { "app.kubernetes.io/name" = local.component } }
      policyTypes = ["Ingress", "Egress"]
      ingress     = [{ from = [{ namespaceSelector = { matchLabels = { "oyatie.com/tenant-id" = var.tenant_id } } }] }]
      egress      = [{ to = [{ namespaceSelector = { matchLabels = { "oyatie.com/tenant-id" = var.tenant_id } } }] }]
    }
  }

  kubernetes_manifests = [
    local.namespace_manifest,
    local.service_account_manifest,
    local.jcasc_config_manifest,
    local.persistent_volume_claim_manifest,
    local.deployment_manifest,
    local.service_manifest,
    local.network_policy_manifest
  ]
}
