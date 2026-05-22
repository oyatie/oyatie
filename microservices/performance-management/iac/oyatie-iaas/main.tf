# OpenTofu module: performance-management on Oyatie's own IaaS (cloud-* substrate).
# Per ADR-0254 K8s everywhere + Cloud Hypervisor + Kata pods.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    helm = {
      source  = "opentofu/helm"
      version = ">= 2.13.0"
    }
    oya_cloud = {
      source  = "oyatie/oya-cloud"
      version = ">= 1.0.0"
    }
  }
}

variable "tenant_id" { type = string }
variable "tenant_class" {
  type = string
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}
variable "oya_region" {
  type    = string
  default = "us-east-1-oya"
}

resource "oya_cloud_kata_pod" "performance_management_pod" {
  service_name = "performance-management"
  tenant_id    = var.tenant_id
  tenant_class = var.tenant_class
  region       = var.oya_region
  runtime      = "kata-containers"
  hypervisor   = "cloud-hypervisor"
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
    value = "oyatie-iaas"
  }
}

resource "oya_billing_binding" "performance_management_settlement" {
  billing_component_id   = "bc-performance-management"
  service_name           = "performance-management"
  tenant_id              = var.tenant_id
  tenant_class           = var.tenant_class
  context                = "oyatie-iaas"
  marketplace_settlement = "DealSet"
}
