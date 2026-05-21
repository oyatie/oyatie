# cloud-billing OpenTofu module for the oyatie-public-cloud deployment context

resource "kubernetes_namespace" "cloud_billing" {
  metadata {
    name = "cloud-billing"
    labels = {
      "oyatie.io/microservice" = "cloud-billing"
      "oyatie.io/cell-id"      = var.cell_id
      "oyatie.io/deployment-context" = "oyatie-public-cloud"
    }
  }
}

resource "helm_release" "strimzi_kafka" {
  name       = "metering-bus"
  namespace  = kubernetes_namespace.cloud_billing.metadata[0].name
  repository = "oci://oyatie-registry.internal.oyatie.dev/charts"
  chart      = "strimzi-kafka"
  version    = var.strimzi_version

  set {
    name  = "kafka.replicas"
    value = "5"
  }
  set {
    name  = "kafka.minInSyncReplicas"
    value = "3"
  }
  set {
    name  = "kafka.partitions"
    value = "256"
  }
}

resource "helm_release" "postgres_primary" {
  name       = "billing-postgres-primary"
  namespace  = kubernetes_namespace.cloud_billing.metadata[0].name
  repository = "oci://oyatie-registry.internal.oyatie.dev/charts"
  chart      = "postgres-ha"
  version    = var.postgres_chart_version

  set {
    name  = "primary.image.tag"
    value = "16.3"
  }
  set {
    name  = "logicalReplication.enabled"
    value = "true"
  }
}

resource "helm_release" "cloud_billing_app" {
  name       = "cloud-billing"
  namespace  = kubernetes_namespace.cloud_billing.metadata[0].name
  repository = "oci://oyatie-registry.internal.oyatie.dev/charts"
  chart      = "cloud-billing"
  version    = var.app_chart_version

  set {
    name  = "image.repository"
    value = "oyatie-registry.internal.oyatie.dev/cloud-billing"
  }
  set {
    name  = "image.tag"
    value = var.app_image_tag
  }
  set {
    name  = "tenantClass.enforceEnumClosed"
    value = "true"
  }
  set {
    name  = "billingComponents.enforceSubsetClosed"
    value = "true"
  }
  set {
    name  = "auditChain.endpoint"
    value = var.audit_chain_endpoint
  }
  set {
    name  = "cloudKms.endpoint"
    value = var.cloud_kms_endpoint
  }
  set {
    name  = "cloudBillingTax.endpoint"
    value = var.cloud_billing_tax_endpoint
  }
  set {
    name  = "payments.endpoint"
    value = var.payments_endpoint
  }
  set {
    name  = "observability.otlpEndpoint"
    value = var.otlp_endpoint
  }
  set {
    name  = "cell.id"
    value = var.cell_id
  }
  set {
    name  = "deploymentContext"
    value = "oyatie-public-cloud"
  }
}

data "cosign_verify" "module_attestation" {
  image = "oyatie-registry.internal.oyatie.dev/cloud-billing-iac:${var.app_chart_version}"
}
