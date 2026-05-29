# OpenTofu module: performance-management on-prem deployment.
# Target OSes per supported_oses.json: Talos / RHEL / SUSE / Ubuntu LTS / Debian / etc.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    kubernetes = {
      source  = "opentofu/kubernetes"
      version = ">= 2.30.0"
    }
    helm = {
      source  = "opentofu/helm"
      version = ">= 2.13.0"
    }
  }
}

variable "tenant_id" { type = string }
variable "tenant_class" {
  type    = string
  default = "paid"
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}
variable "kubeconfig_path" { type = string }

provider "kubernetes" {
  config_path = var.kubeconfig_path
}

provider "helm" {
  kubernetes {
    config_path = var.kubeconfig_path
  }
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
    name  = "context"
    value = "on-prem"
  }
}

resource "oya_billing_binding" "performance_management_settlement" {
  billing_component_id   = "bc-performance-management"
  service_name           = "performance-management"
  tenant_id              = var.tenant_id
  tenant_class           = var.tenant_class
  context                = "on-prem"
  marketplace_settlement = "DealSet"
}
