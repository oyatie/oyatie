# oyatie-cloud-provider OpenTofu module for the imaging µservice.
# Authority: ADR-0131 + Oyatie-as-cloud-provider (cloud-* µservices ARE
# Oyatie's own IaaS surface).
#
# Use case: hosted radiology service bureau / teleradiology night-hawks.
# Substrate: oyatie cloud-compute + cloud-storage + cloud-data + cloud-kms.

terraform {
  required_version = ">= 1.7"
  required_providers {
    oyatie = { source = "oyatie/oyatie", version = "~> 0.1" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id"   { type = string }
variable "pack_id" {
  type = string
  default = "HIPAA-2024"
}

resource "oyatie_cloud_data_postgres" "pacs_index" {
  name        = "imaging-pacs-index"
  tenant_id   = var.tenant_id
  cell_id     = var.cell_id
  size_gb     = 500
  replicas    = 3
  cross_az    = true
  pack_id     = var.pack_id
  encryption  = "kms-byok-opt-in"
}

resource "oyatie_cloud_storage_bucket" "pacs_blobs" {
  name        = "imaging-pacs-blobs"
  tenant_id   = var.tenant_id
  cell_id     = var.cell_id
  durability  = "13-nines"
  erasure_coding = "14+4"
  encryption  = "kms-byok-opt-in"
  pack_id     = var.pack_id
}

resource "oyatie_cloud_kms_key" "imaging_envelope" {
  name        = "imaging-envelope"
  tenant_id   = var.tenant_id
  cell_id     = var.cell_id
  byok        = false
  rotation_days = 90
}

resource "oyatie_cloud_compute_k8s_workload" "dimse_listener" {
  name      = "imaging-dimse-listener"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  replicas  = 5
  image     = "oyatie/oya-imaging-dimse-api:0.1.0-wave-15m-g"
  ports = [
    { name = "dimse", port = 11112, protocol = "tcp", tls = true }
  ]
  resources = {
    requests = { cpu = "2", memory = "8Gi" }
    limits   = { cpu = "8", memory = "32Gi" }
  }
}

resource "oyatie_cloud_compute_k8s_workload" "dicomweb_api" {
  name      = "imaging-dicomweb-api"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  replicas  = 5
  image     = "oyatie/oya-imaging-dicomweb-api:0.1.0-wave-15m-g"
  ports = [
    { name = "http3", port = 443, protocol = "quic" }
  ]
}

output "pacs_postgres_endpoint" { value = oyatie_cloud_data_postgres.pacs_index.endpoint }
output "pacs_blobs_endpoint"    { value = oyatie_cloud_storage_bucket.pacs_blobs.endpoint }
