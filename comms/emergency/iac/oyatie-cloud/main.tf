// OpenTofu module — ED-IS on oyatie's own cloud (oyatie-as-cloud-provider).
// Authority: ADR-0332 (in flight) | ADR-0254 (Kubernetes + Cloud Hypervisor) |
//            feedback_multi_context_provider_agnostic_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Provisions ED-IS on Oyatie's own IaaS substrate (Cloud Hypervisor + Kata
// pods on Talos). This is the "Oyatie as cloud provider" context — Oyatie
// is selling raw compute and ED-IS runs on top of it.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    oyatie = {
      source  = "oyatie/oyatie-cloud"
      version = "~> 0.1"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}

variable "tenant_id" { type = string }
variable "cell_id"   { type = string }
variable "region" {
  type = string
  default = "oyacloud-us-east-1"
}
variable "image_tag" { type = string }
variable "compliance_packs" {
  type    = list(string)
  default = ["HIPAA", "SOC2", "HITRUST", "EMTALA"]
}
variable "kata_pod_runtime" {
  type = bool
  default = true
}

resource "oyatie_cell" "emergency_cell" {
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  region    = var.region
  tier      = "tier-2-single-tenant"
  hypervisor = "cloud-hypervisor"
  pod_runtime = var.kata_pod_runtime ? "kata" : "runc"
  compliance_packs = var.compliance_packs
}

resource "oyatie_workload" "emergency_workload" {
  cell_id   = oyatie_cell.emergency_cell.cell_id
  workload  = "emergency"
  image_tag = var.image_tag
  replicas  = 3
}

output "emergency_endpoint" {
  value = oyatie_workload.emergency_workload.public_endpoint
}

output "cell_tier" {
  value = oyatie_cell.emergency_cell.tier
}
