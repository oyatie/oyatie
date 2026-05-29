# OpenTofu module: performance-management deployment in customer OCI tenancy.
# Context: oci-guest (paid tier). For demo_trial use `always-free/` sub-module.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    oci = {
      source  = "opentofu/oci"
      version = ">= 6.0.0"
    }
    helm = {
      source  = "opentofu/helm"
      version = ">= 2.13.0"
    }
  }
}

variable "tenant_id" { type = string }
variable "tenant_class" {
  type = string
  default = "paid"
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}
variable "oci_region" {
  type    = string
  default = "us-ashburn-1"
}
variable "compartment_ocid" { type = string }

resource "oci_containerengine_cluster" "performance_management" {
  compartment_id     = var.compartment_ocid
  name               = "perf-mgmt-${var.tenant_id}"
  kubernetes_version = "v1.30.0"
  vcn_id             = var.vcn_id
}

variable "vcn_id" { type = string }

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
    value = "oci-guest"
  }
}

resource "oya_billing_binding" "performance_management_settlement" {
  billing_component_id   = "bc-performance-management"
  service_name           = "performance-management"
  tenant_id              = var.tenant_id
  tenant_class           = var.tenant_class
  context                = "oci-guest"
  marketplace_settlement = "DealSet"
}
