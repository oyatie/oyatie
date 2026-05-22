# OpenTofu constraints for messenger on-prem deployment context.

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.13"
    }
    vsphere = {
      source  = "hashicorp/vsphere"
      version = "~> 2.8"
    }
  }
}
