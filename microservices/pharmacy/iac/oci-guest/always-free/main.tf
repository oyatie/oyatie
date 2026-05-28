// Pharmacy microservice — OCI Always Free deployment context
// Authority: feedback_oci_always_free_maximization_2026_05_20; ADR-0332
// Targets: demo / sandbox / trial / dev tenants on OCI Always Free tier.
//   - 2 × Ampere A1 ARM (4 OCPU + 24 GB RAM each)
//   - 2 × Autonomous DB Always Free
//   - 200 GB block + 10 GB object storage
//   - 10 TB egress / month
//   - OCI Vault, Load Balancer, Streaming included

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    oci = { source = "oracle/oci", version = ">= 6.0" }
  }
}

variable "tenant_id" { type = string }
variable "compartment_ocid" { type = string }
variable "pharmacy_oltp_admin_password" {
  type        = string
  sensitive   = true
  description = "Autonomous Database admin password supplied from OCI Vault/OpenBao or an operator secret store; never derive it from tenant metadata."

  validation {
    condition = (
      length(var.pharmacy_oltp_admin_password) >= 12 &&
      length(var.pharmacy_oltp_admin_password) <= 30 &&
      can(regex("[[:upper:]]", var.pharmacy_oltp_admin_password)) &&
      can(regex("[[:lower:]]", var.pharmacy_oltp_admin_password)) &&
      can(regex("[[:digit:]]", var.pharmacy_oltp_admin_password)) &&
      can(regex("[^[:alnum:]]", var.pharmacy_oltp_admin_password))
    )
    error_message = "pharmacy_oltp_admin_password must be 12-30 characters and include upper, lower, digit, and special characters."
  }
}

resource "oci_core_instance" "pharmacy_arm_node" {
  count               = 2
  availability_domain = "AD-1"
  compartment_id      = var.compartment_ocid
  shape               = "VM.Standard.A1.Flex"
  shape_config {
    ocpus         = 4
    memory_in_gbs = 24
  }
  display_name = "pharmacy-${var.tenant_id}-arm-${count.index}"
  freeform_tags = {
    microservice = "pharmacy"
    tier         = "always-free"
    tenant       = var.tenant_id
  }
}

resource "oci_database_autonomous_database" "pharmacy_oltp" {
  compartment_id           = var.compartment_ocid
  db_name                  = "pharmacy${substr(var.tenant_id, 0, 6)}"
  display_name             = "pharmacy-${var.tenant_id}-oltp"
  is_free_tier             = true
  db_workload              = "OLTP"
  cpu_core_count           = 1
  data_storage_size_in_tbs = 1
  admin_password           = var.pharmacy_oltp_admin_password
}

resource "oci_streaming_stream" "pharmacy_events" {
  compartment_id     = var.compartment_ocid
  name               = "pharmacy-${var.tenant_id}-events"
  partitions         = 1
  retention_in_hours = 24
}

output "always_free_summary" {
  value = {
    arm_nodes = oci_core_instance.pharmacy_arm_node[*].display_name
    oltp_db   = oci_database_autonomous_database.pharmacy_oltp.display_name
    stream    = oci_streaming_stream.pharmacy_events.name
    tier      = "always-free"
    tenant    = var.tenant_id
    note      = "Maximizes OCI Always Free for demo/sandbox/trial/dev tenants per feedback_oci_always_free_maximization_2026_05_20"
  }
}
