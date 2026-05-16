// Root module for OCI infra. All non-Foundry services declare desired state here.
// Hand-rolled `oci` CLI usage is forbidden; every resource is declarative.

terraform {
  required_version = ">= 1.12.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 6.0"
    }
  }
  backend "local" {
    path = "terraform.tfstate"
  }
}

provider "oci" {
  // Reads ~/.oci/config DEFAULT profile (already authed: tenancy bitween, region ap-chuncheon-1).
  config_file_profile = "DEFAULT"
}

locals {
  tenancy_ocid = var.tenancy_ocid
  region       = var.region
  common_tags = {
    "managed-by"  = "opentofu"
    "git-branch"  = "dev"
    "provisioner" = "oyatie-infra"
  }
}
