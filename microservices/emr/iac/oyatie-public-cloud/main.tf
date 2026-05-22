# OpenTofu module — EMR µservice — context: oyatie-public-cloud
# Per ADR-0131 + ADR-0244 + ADR-0251 + memory feedback_zero_handroll_opentofu_only
# Wave 15M-B authored 2026-05-21
# Targets oyatie's hosted multi-tenant cells (tier-0 + tier-1)

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
    kustomization = { source = "kbst/kustomization", version = "~> 0.9" }
  }
}

variable "cell_id" {
  type = string
  description = "Target cell identifier (e.g., cell-us-east-1-tier-0-001)"
}

variable "cell_certification_levels" {
  type    = list(string)
  default = ["hipaa-certified"]
  description = "Cell's certification level set per ADR-0251 §D-4. emr requires hipaa-certified or hipaa-pci-certified or healthcare-sovereign."
}

variable "tenant_ids" {
  type        = list(string)
  description = "Tenant IDs eligible for this cell"
}

variable "replicas_per_az" {
  type    = number
  default = 12
}

variable "azs" {
  type    = list(string)
  default = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

resource "kubernetes_namespace" "emr" {
  metadata {
    name = "emr-${var.cell_id}"
    labels = {
      "oyatie.io/microservice" = "emr"
      "oyatie.io/cell-id"      = var.cell_id
      "oyatie.io/data-class"   = "phi-protected-health-information"
      "oyatie.io/compliance-pack" = "HIPAA-2024"
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"

  values = [
    yamlencode({
      replicaCount = var.replicas_per_az * length(var.azs)
      image = {
        repository = "registry.oyatie.health/emr"
        tag        = "1.0.0-wave-15m-b"
        pullPolicy = "IfNotPresent"
      }
      tolerations  = [{ key = "phi-eligible", operator = "Equal", value = "true", effect = "NoSchedule" }]
      affinity = {
        podAntiAffinity = { preferredDuringSchedulingIgnoredDuringExecution = [
          { weight = 100, podAffinityTerm = { labelSelector = { matchLabels = { app = "emr" } }, topologyKey = "topology.kubernetes.io/zone" } }
        ] }
      }
      podDisruptionBudget = { minAvailable = "60%" }
      resources = {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }
      autoscaling = { enabled = true, minReplicas = 12, maxReplicas = 72, targetCPUUtilizationPercentage = 70 }
      tenantIds = var.tenant_ids
      certificationLevels = var.cell_certification_levels
      compliancePacksRequired = ["HIPAA-2024"]
      env = {
        DEFAULT_FHIR_VERSION = "R5"
        CHART_OPEN_SNAPSHOT_TTL_SECONDS = "60"
        CDS_HOOKS_DEADLINE_MS = "500"
        EPCS_2FA_REQUIRED_FOR_SCHEDULES = "II"
        PDMP_REQUIRED_FOR_SCHEDULES = "II,III,IV"
      }
    })
  ]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
output "emr_helm_release" { value = helm_release.emr.name }
