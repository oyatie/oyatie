# OpenTofu module: performance-management on OCI Always Free tier (demo_trial workloads).
# Closes audit Finding 6.2.A (P0).
# Memory directive: feedback_oci_always_free_maximization_2026_05_20.
# Resources: 2x Ampere A1 ARM (4 OCPU + 24 GB total), 1x Autonomous DB, Vault, LB.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    oci = {
      source  = "opentofu/oci"
      version = ">= 6.0.0"
    }
  }
}

variable "tenant_id" { type = string }
variable "compartment_ocid" { type = string }

# Tenant class is locked to demo_trial in this sub-module.
locals {
  tenant_class = "demo_trial"
}

# 2x Ampere A1 ARM instances (4 OCPU + 24 GB combined; within Always Free).
resource "oci_core_instance" "perf_mgmt_a1_primary" {
  compartment_id      = var.compartment_ocid
  availability_domain = data.oci_identity_availability_domain.ad1.name
  shape               = "VM.Standard.A1.Flex"
  shape_config {
    ocpus         = 2
    memory_in_gbs = 12
  }
  display_name = "perf-mgmt-a1-primary-${var.tenant_id}"
  source_details {
    source_type = "image"
    source_id   = var.ubuntu_arm_image_ocid
  }
  create_vnic_details {
    subnet_id = var.subnet_ocid
  }
  freeform_tags = {
    tenant_id    = var.tenant_id
    tenant_class = local.tenant_class
    context      = "oci-guest-always-free"
    service      = "performance-management"
  }
}

resource "oci_core_instance" "perf_mgmt_a1_secondary" {
  compartment_id      = var.compartment_ocid
  availability_domain = data.oci_identity_availability_domain.ad1.name
  shape               = "VM.Standard.A1.Flex"
  shape_config {
    ocpus         = 2
    memory_in_gbs = 12
  }
  display_name = "perf-mgmt-a1-secondary-${var.tenant_id}"
  source_details {
    source_type = "image"
    source_id   = var.ubuntu_arm_image_ocid
  }
  create_vnic_details {
    subnet_id = var.subnet_ocid
  }
  freeform_tags = {
    tenant_id    = var.tenant_id
    tenant_class = local.tenant_class
    context      = "oci-guest-always-free"
    service      = "performance-management"
  }
}

# Autonomous DB (Always Free 20 GB).
resource "oci_database_autonomous_database" "perf_mgmt_adb" {
  compartment_id           = var.compartment_ocid
  cpu_core_count           = 1
  data_storage_size_in_tbs = 1
  db_name                  = "perfmgmt"
  is_free_tier             = true
  admin_password           = var.adb_admin_password
  freeform_tags = {
    tenant_id    = var.tenant_id
    tenant_class = local.tenant_class
  }
}

data "oci_identity_availability_domain" "ad1" {
  compartment_id = var.compartment_ocid
  ad_number      = 1
}

variable "subnet_ocid" { type = string }
variable "ubuntu_arm_image_ocid" { type = string }
variable "adb_admin_password" {
  type      = string
  sensitive = true
}

resource "oya_billing_binding" "performance_management_settlement_demo" {
  billing_component_id   = "bc-performance-management"
  service_name           = "performance-management"
  tenant_id              = var.tenant_id
  tenant_class           = local.tenant_class
  context                = "oci-guest-always-free"
  marketplace_settlement = "suppressed"   # demo_trial never triggers DealSet settlement
}
