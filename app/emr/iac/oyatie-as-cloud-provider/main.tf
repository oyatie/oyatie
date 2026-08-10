# OpenTofu module — EMR µservice — context: oyatie-as-cloud-provider
# oyatie sells IaaS to the tenant, hosts EMR. Tenant pays oyatie cloud-billing
# for compute + storage + network + PHI-eligible-cell premium.
# This is the "oyatie is itself a cloud provider" mode.
# Wave 15M-B authored 2026-05-21

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.13" }
  }
}

variable "tenant_id" { type = string }
variable "oyatie_cell_id" { type = string }
variable "oyatie_cell_certification_levels" {
  type = list(string)
  default = ["hipaa-certified"]
}
variable "k8s_cluster_endpoint" { type = string }
variable "k8s_ca_cert" {
  type = string
  sensitive = true
}
variable "k8s_token" {
  type = string
  sensitive = true
}
variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Tenant class for EMR capacity and billing semantics."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
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
      "oyatie.io/microservice"     = "emr"
      "oyatie.io/tenant_id"        = var.tenant_id
      "oyatie.io/oyatie-cell-id"   = var.oyatie_cell_id
      "oyatie.io/tenant-class"     = var.tenant_class
      "oyatie.io/data-class"       = "phi-protected-health-information"
      "oyatie.io/compliance-pack"  = "HIPAA-2024"
      "oyatie.io/billing-emit"     = "true"
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"
  values = [yamlencode({
    image = { repository = "registry.oyatie.health/emr", tag = "1.0.0-wave-15m-b" }
    tenantId                 = var.tenant_id
    oyatieCellId             = var.oyatie_cell_id
    tenantClass              = var.tenant_class
    billingEmitToCloudBilling = true
    compliancePacksRequired  = ["HIPAA-2024"]
    paidBillingComponentsEmitted = [
      "emr.patient.active_per_month",
      "emr.encounter.recorded_per_month",
      "emr.note.signed_per_month",
      "emr.order.entered_per_month",
      "emr.fhir.read_per_thousand",
      "emr.fhir.write_per_thousand",
      "emr.patient_portal.active_user_per_month",
      "emr.documentation_template.usage_per_month",
      "emr.clinical_decision_support.invocation_per_thousand",
      "emr.audit_event.emission_per_million"
    ]
    resources = (
      var.tenant_class == "demo_trial" ? { requests = { cpu = "1", memory = "4Gi" },  limits = { cpu = "2", memory = "8Gi" } } :
      { requests = { cpu = "2", memory = "8Gi" }, limits = { cpu = "8", memory = "32Gi" } }
    )
  })]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
output "tenant_class" { value = var.tenant_class }
