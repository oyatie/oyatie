# oci-guest OpenTofu module for the imaging µservice.
# Authority: ADR-0131 + zero-handroll OpenTofu-only + OCI Always Free
# maximization for demo_trial.
#
# Demo_trial profile: exploits OCI Always Free tier (2× Ampere A1 ARM
# 4 OCPU + 24GB RAM / 2× Autonomous DB / 200GB block / 10GB obj / 10TB
# egress / Vault / LB / Streaming).
#
# Paid profile: Always Free baseline plus paid scale.

terraform {
  required_version = ">= 1.7"
  required_providers {
    oci = { source = "oracle/oci", version = "~> 6.0" }
  }
}

variable "tenant_id"         { type = string }
variable "cell_id"           { type = string }
variable "compartment_ocid"  { type = string }
variable "tenant_class" {
  type = string
  default = "demo_trial"
}
variable "region" {
  type = string
  default = "us-ashburn-1"
}

locals {
  is_always_free = var.tenant_class == "demo_trial"
}

resource "oci_core_vcn" "imaging" {
  compartment_id = var.compartment_ocid
  cidr_block     = "10.43.0.0/16"
  display_name   = "imaging-${var.tenant_id}-${var.cell_id}"
  defined_tags = {
    "oyatie.microservice" = "imaging"
    "oyatie.tenant"       = var.tenant_id
    "oyatie.cell"         = var.cell_id
    "oyatie.context"      = "oci-guest"
    "oyatie.tenant_class" = var.tenant_class
  }
}

# Always Free Ampere A1 ARM instances for demo_trial PACS substrate.
resource "oci_core_instance" "pacs_a1_arm" {
  count               = local.is_always_free ? 2 : 0
  compartment_id      = var.compartment_ocid
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  display_name        = "imaging-pacs-a1-${count.index}"
  shape               = "VM.Standard.A1.Flex"
  shape_config {
    ocpus         = 2 # 4 OCPU total across 2 instances = always-free limit
    memory_in_gbs = 12
  }
  create_vnic_details {
    subnet_id = oci_core_subnet.imaging.id
  }
  source_details {
    source_type = "image"
    source_id   = data.oci_core_images.ol9.images[0].id
  }
}

# Paid profile: scale-out OKE cluster.
resource "oci_containerengine_cluster" "imaging" {
  count          = local.is_always_free ? 0 : 1
  compartment_id = var.compartment_ocid
  name           = "imaging-${var.tenant_id}-${var.cell_id}"
  vcn_id         = oci_core_vcn.imaging.id
  kubernetes_version = "v1.30.1"
  options {
    service_lb_subnet_ids = [oci_core_subnet.lb.id]
  }
}

resource "oci_core_subnet" "imaging" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.imaging.id
  cidr_block     = "10.43.1.0/24"
  display_name   = "imaging-pods"
}

resource "oci_core_subnet" "lb" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.imaging.id
  cidr_block     = "10.43.2.0/24"
  display_name   = "imaging-lb"
}

# Autonomous DB for demo_trial PACS index (Always Free 2× Autonomous DB).
resource "oci_database_autonomous_database" "pacs_index" {
  count          = local.is_always_free ? 1 : 0
  compartment_id = var.compartment_ocid
  cpu_core_count = 1
  data_storage_size_in_tbs = 1
  db_name        = "imagingpacs"
  display_name   = "imaging-pacs-index-always-free"
  admin_password = var.adb_admin_password
  db_workload    = "OLTP"
  is_free_tier   = true
}

# Object Storage bucket for demo_trial DICOM blobs.
resource "oci_objectstorage_bucket" "pacs_blobs" {
  compartment_id = var.compartment_ocid
  name           = "imaging-pacs-blobs-${var.tenant_id}-${var.cell_id}"
  namespace      = data.oci_objectstorage_namespace.ns.namespace
  access_type    = "NoPublicAccess"
}

variable "adb_admin_password" {
  type      = string
  sensitive = true
  default   = ""
}

data "oci_identity_availability_domains" "ads" {
  compartment_id = var.compartment_ocid
}

data "oci_objectstorage_namespace" "ns" {
  compartment_id = var.compartment_ocid
}

data "oci_core_images" "ol9" {
  compartment_id = var.compartment_ocid
  operating_system = "Oracle Linux"
  operating_system_version = "9"
  shape = "VM.Standard.A1.Flex"
  sort_by = "TIMECREATED"
  sort_order = "DESC"
}

output "always_free_active" { value = local.is_always_free }
output "vcn_id" { value = oci_core_vcn.imaging.id }
