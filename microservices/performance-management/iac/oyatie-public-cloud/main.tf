# OpenTofu module: performance-management deployment for oyatie-public-cloud context.
# Closes audit Findings 2.5.A, 2.5.B, 6.1.A, 7.1.A (P0).
# Engine: OpenTofu only (Terraform forbidden per ADR-0328 §D-20.30).
# Authority: ADR-0329 multi-context deployment doctrine.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    helm = {
      source  = "opentofu/helm"
      version = ">= 2.13.0"
    }
    kubernetes = {
      source  = "opentofu/kubernetes"
      version = ">= 2.30.0"
    }
  }
}

variable "tenant_id" {
  type        = string
  description = "Tenant UUID per ADR-0244 universal tenant scoping."
}

variable "tenant_class" {
  type        = string
  description = "Tenant class enum per ADR-0331."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}

variable "cell_tier" {
  type    = string
  default = "T1"
}

resource "helm_release" "performance_management" {
  name      = "performance-management"
  namespace = "perf-mgmt-${var.tenant_id}"
  chart     = "${path.module}/../charts/performance-management"
  values = [
    file("${path.module}/../helm-values.yaml")
  ]
  set {
    name  = "tenant.id"
    value = var.tenant_id
  }
  set {
    name  = "tenant.class"
    value = var.tenant_class
  }
  set {
    name  = "cell.tier"
    value = var.cell_tier
  }
}

output "service_url" {
  value = "https://${var.tenant_id}.perf.oyatie.dev"
}
