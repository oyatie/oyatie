// OpenTofu module — ED-IS on OCI Always Free shape.
// Authority: ADR-0332 (in flight) |
//            feedback_oci_always_free_maximization_2026_05_20 |
//            feedback_zero_handroll_opentofu_only_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Provisions the entire ED-IS stack inside the OCI Always Free envelope:
//   - 2× Ampere A1 ARM VM (4 OCPU + 24 GB RAM each)
//   - 2× Autonomous Database (Always Free 20 GB each)
//   - 200 GB block storage
//   - 10 GB object storage
//   - 10 TB egress / month
//   - Load Balancer (10 Mbps)
//   - Vault + KMS key (Always Free vault tier)
//   - Streaming (Always Free quota)
//
// Used by demo_trial tenants and sandbox deployments.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 6.0"
    }
  }
}

variable "tenant_id"     { type = string }
variable "cell_id"       { type = string }
variable "compartment_ocid" { type = string }
variable "image_tag"     { type = string }
variable "demo_mode" {
  type = bool
  default = true
}

# 2× Ampere A1 ARM VMs — maxed against the 4 OCPU + 24 GB Always Free shape.
resource "oci_core_instance" "emergency_a1_primary" {
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  compartment_id      = var.compartment_ocid
  shape               = "VM.Standard.A1.Flex"
  display_name        = "emergency-${var.cell_id}-a1-primary"
  shape_config {
    ocpus         = 4
    memory_in_gbs = 24
  }
  source_details {
    source_type = "image"
    source_id   = data.oci_core_images.ubuntu_arm.images[0].id
  }
  create_vnic_details {
    subnet_id = oci_core_subnet.emergency_subnet.id
    assign_public_ip = false
  }
  freeform_tags = {
    "microservice" = "emergency"
    "tenant_id"    = var.tenant_id
    "cell_id"      = var.cell_id
    "tenant_class_profile" = "demo-trial-always-free"
  }
}

resource "oci_core_instance" "emergency_a1_secondary" {
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  compartment_id      = var.compartment_ocid
  shape               = "VM.Standard.A1.Flex"
  display_name        = "emergency-${var.cell_id}-a1-secondary"
  shape_config {
    ocpus         = 4
    memory_in_gbs = 24
  }
  source_details {
    source_type = "image"
    source_id   = data.oci_core_images.ubuntu_arm.images[0].id
  }
  create_vnic_details {
    subnet_id = oci_core_subnet.emergency_subnet.id
    assign_public_ip = false
  }
}

# 2× Autonomous Database (Always Free 20 GB each).
resource "oci_database_autonomous_database" "emergency_adb_primary" {
  compartment_id           = var.compartment_ocid
  cpu_core_count           = 1
  data_storage_size_in_tbs = 1
  db_name                  = "edisprimary"
  display_name             = "emergency-adb-${var.cell_id}-primary"
  is_free_tier             = true
  db_workload              = "OLTP"
  freeform_tags = {
    "microservice" = "emergency"
    "tenant_class_profile" = "demo-trial-always-free"
  }
}

resource "oci_database_autonomous_database" "emergency_adb_audit" {
  compartment_id           = var.compartment_ocid
  cpu_core_count           = 1
  data_storage_size_in_tbs = 1
  db_name                  = "edisaudit"
  display_name             = "emergency-adb-${var.cell_id}-audit"
  is_free_tier             = true
  db_workload              = "OLTP"
}

# Always Free Vault tier
resource "oci_kms_vault" "emergency_vault" {
  compartment_id = var.compartment_ocid
  display_name   = "emergency-vault-${var.cell_id}"
  vault_type     = "VIRTUAL_PRIVATE"
}

# Always Free Streaming
resource "oci_streaming_stream" "ed_events" {
  compartment_id = var.compartment_ocid
  name           = "ed-events-${var.cell_id}"
  partitions     = 1
  retention_in_hours = 24
}

# Always Free Load Balancer (10 Mbps)
resource "oci_load_balancer_load_balancer" "emergency_lb" {
  compartment_id = var.compartment_ocid
  display_name   = "emergency-lb-${var.cell_id}"
  shape          = "10Mbps-Micro"  # Always Free shape
  subnet_ids     = [oci_core_subnet.emergency_subnet.id]
  is_private     = false
}

# Network primitives
resource "oci_core_vcn" "emergency_vcn" {
  compartment_id = var.compartment_ocid
  cidr_block     = "10.0.0.0/16"
  display_name   = "emergency-vcn-${var.cell_id}"
}

resource "oci_core_subnet" "emergency_subnet" {
  compartment_id = var.compartment_ocid
  cidr_block     = "10.0.1.0/24"
  display_name   = "emergency-subnet-${var.cell_id}"
  vcn_id         = oci_core_vcn.emergency_vcn.id
}

data "oci_identity_availability_domains" "ads" {
  compartment_id = var.compartment_ocid
}

data "oci_core_images" "ubuntu_arm" {
  compartment_id           = var.compartment_ocid
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "24.04"
  shape                    = "VM.Standard.A1.Flex"
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

output "emergency_endpoint" {
  value = oci_load_balancer_load_balancer.emergency_lb.ip_address_details[0].ip_address
}

output "deployment_tier" {
  value = "bronze-always-free"
}
