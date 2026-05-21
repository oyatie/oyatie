# OpenTofu module: performance-management deployment in customer AWS VPC, Oyatie-operated.
# Context: guest-on-aws.

terraform {
  required_version = ">= 1.8.0"
  required_providers {
    aws = {
      source  = "opentofu/aws"
      version = ">= 5.50.0"
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
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}
variable "aws_region" {
  type    = string
  default = "us-east-1"
}
variable "vpc_id" { type = string }
variable "eks_cluster_name" { type = string }

resource "aws_eks_addon" "performance_management" {
  cluster_name = var.eks_cluster_name
  addon_name   = "performance-management"
  tags = {
    tenant_id    = var.tenant_id
    tenant_class = var.tenant_class
    context      = "guest-on-aws"
  }
}

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
    value = "guest-on-aws"
  }
}

resource "oya_billing_binding" "performance_management_settlement" {
  billing_component_id   = "bc-performance-management"
  service_name           = "performance-management"
  tenant_id              = var.tenant_id
  tenant_class           = var.tenant_class
  context                = "guest-on-aws"
  marketplace_settlement = "DealSet"
}
