# Canonical OpenTofu module — k8s-namespace-bootstrap (Tier B per ADR-0202).
# Namespace + RBAC + default-deny NetworkPolicy + ArgoCD AppProject
# bootstrap. Tier-A ArgoCD lands app manifests inside this namespace
# safely.

terraform {
  required_version = ">= 1.6"

  required_providers {
    kubernetes = {
      source  = "registry.opentofu.org/hashicorp/kubernetes"
      version = ">= 2.30.0"
    }
  }
}

variable "cluster_id" { type = string }
variable "namespace" { type = string }
variable "owner_team" { type = string }
variable "argocd_project_name" { type = string }
variable "tags" {
  type    = map(string)
  default = {}
}

output "namespace_name" { value = var.namespace }
output "service_account_name" { value = "" }
output "argocd_project_name" { value = var.argocd_project_name }
