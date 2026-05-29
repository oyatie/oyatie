// OpenTofu module — ED-IS on OCI guest deployment context.
// Authority: ADR-0332 (in flight) | feedback_zero_handroll_opentofu_only_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Provisions: OKE workloads + Autonomous DB + OCI Cache (Valkey) + Streaming +
//             Vault. Tenant onboarding completes with a single `tofu apply`.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 6.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}

variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "compartment_ocid" { type = string }
variable "region" {
  type    = string
  default = "us-ashburn-1"
}
variable "compliance_packs" {
  type    = list(string)
  default = ["HIPAA", "SOC2", "HITRUST", "EMTALA"]
}
variable "image_tag" { type = string }

module "emergency_cluster" {
  source           = "../_shared/oke-cell"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  compartment_ocid = var.compartment_ocid
  region           = var.region
  compliance_packs = var.compliance_packs
}

module "emergency_db" {
  source           = "../_shared/autonomous-db"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  compartment_ocid = var.compartment_ocid
}

module "emergency_valkey" {
  source           = "../_shared/oci-cache"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  compartment_ocid = var.compartment_ocid
}

module "emergency_streaming" {
  source           = "../_shared/oci-streaming"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  compartment_ocid = var.compartment_ocid
  stream_prefix    = "ed"
}

module "emergency_vault" {
  source           = "../_shared/oci-vault"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  compartment_ocid = var.compartment_ocid
  pack             = "HIPAA"
}

output "emergency_endpoint" {
  value = "https://emergency.${var.tenant_id}.${var.cell_id}.oci.oyatie.cloud"
}
