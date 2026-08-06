# sovereign-cell OpenTofu module for the imaging µservice.
# Authority: ADR-0131 + ADR-0248 cellular shape + ADR-0251 compliance packs +
# ADR-0254 Cloud Hypervisor + Kata Containers.
#
# MANDATORY for paid PHI tenants. Cell-aware deployment with Cedar-enforced
# isolation per HIPAA / GDPR / KR-MD / EU-MDR packs.

terraform {
  required_version = ">= 1.7"
  required_providers {
    oyatie = { source = "oyatie/oyatie", version = "~> 0.1" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id"   { type = string }
variable "pack_id" {
  type    = string
  description = "Compliance pack: HIPAA-2024 | GDPR | KR-Medical-Devices | EU-MDR | EU-AI-Act | MQSA"
}
variable "region" { type = string }
variable "byok_opt_in" {
  type = bool
  default = false
}

locals {
  pack_constraints = {
    "HIPAA-2024" = {
      region_allow = ["us-east", "us-west", "us-central"]
      audit_retention_days = 2190 # 6 years
    }
    "GDPR" = {
      region_allow = ["eu-west", "eu-central", "eu-north"]
      audit_retention_days = 2190
    }
    "KR-Medical-Devices" = {
      region_allow = ["kr-seoul", "kr-busan"]
      audit_retention_days = 1825 # 5 years
    }
    "EU-MDR" = {
      region_allow = ["eu-west", "eu-central"]
      audit_retention_days = 5475 # 15 years (Class IIa post-market surveillance)
    }
    "EU-AI-Act" = {
      region_allow = ["eu-west", "eu-central"]
      audit_retention_days = 3650
    }
    "MQSA" = {
      region_allow = ["us-east", "us-west", "us-central"]
      audit_retention_days = 3650 # 10 years
    }
  }
  selected = local.pack_constraints[var.pack_id]
}

resource "oyatie_cell" "imaging_sovereign" {
  name      = "imaging-${var.tenant_id}-${var.cell_id}-${var.pack_id}"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  pack_id   = var.pack_id
  region    = var.region
  tier      = "tier-0"
  cellular_shape = "amazon-style-shuffle-sharded"
  cloud_hypervisor = true
  kata_containers  = true
}

resource "oyatie_cloud_data_postgres" "pacs_index" {
  name        = "imaging-pacs-index"
  tenant_id   = var.tenant_id
  cell_id     = oyatie_cell.imaging_sovereign.cell_id
  pack_id     = var.pack_id
  size_gb     = 5000
  replicas    = 5
  cross_az    = true
  cross_cell_within_pack = true
  encryption  = var.byok_opt_in ? "kms-byok" : "kms-default"
  audit_retention_days = local.selected.audit_retention_days
}

resource "oyatie_cloud_storage_bucket" "pacs_blobs" {
  name        = "imaging-pacs-blobs"
  tenant_id   = var.tenant_id
  cell_id     = oyatie_cell.imaging_sovereign.cell_id
  pack_id     = var.pack_id
  durability  = "13-nines"
  erasure_coding = "14+4"
  cross_az    = true
  cross_cell_within_pack = true
  encryption  = var.byok_opt_in ? "kms-byok" : "kms-default"
}

resource "oyatie_cloud_kms_key" "imaging_envelope" {
  name        = "imaging-envelope"
  tenant_id   = var.tenant_id
  cell_id     = oyatie_cell.imaging_sovereign.cell_id
  pack_id     = var.pack_id
  byok        = var.byok_opt_in
  rotation_days = 90
}

resource "oyatie_cloud_iam_cedar_bundle" "imaging" {
  name      = "imaging"
  tenant_id = var.tenant_id
  cell_id   = oyatie_cell.imaging_sovereign.cell_id
  pack_id   = var.pack_id
  policies  = [
    file("${path.module}/../../policies/radiologist-can-read.cedar"),
    file("${path.module}/../../policies/technologist-can-acquire.cedar"),
    file("${path.module}/../../policies/peer-reviewer-can-read-blind.cedar"),
    file("${path.module}/../../policies/ai-model-can-read-deidentified.cedar"),
    file("${path.module}/../../policies/patient-can-view-own.cedar"),
    file("${path.module}/../../policies/hipaa-deny-default.cedar"),
    file("${path.module}/../../policies/dose-monitoring-can-read-aggregate.cedar"),
    file("${path.module}/../../policies/break-glass-emergency.cedar"),
    file("${path.module}/../../policies/external-referring-can-view-shared.cedar"),
  ]
}

resource "oyatie_audit_chain_binding" "imaging" {
  tenant_id = var.tenant_id
  cell_id   = oyatie_cell.imaging_sovereign.cell_id
  pack_id   = var.pack_id
  source_microservice = "imaging"
  retention_days = local.selected.audit_retention_days
  tamper_evident = true
  cross_cell_replicated = true
}

output "sovereign_cell_id" { value = oyatie_cell.imaging_sovereign.cell_id }
output "pack_id"           { value = var.pack_id }
output "audit_retention_days" { value = local.selected.audit_retention_days }
