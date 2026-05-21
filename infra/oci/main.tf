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
  // Reads ~/.oci/config under var.oci_config_profile. Profile carries region +
  // tenancy + session-token credentials. Each workspace's tfvars selects its
  // matching OCI profile: bitween → DEFAULT, bominal-oci → bominal-oci.
  config_file_profile = var.oci_config_profile
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
