# Cloud IaC provider dependency lock root.
#
# This root configuration exists only to materialize provider dependency
# selections and checksums for local review. It intentionally contains no
# provider configuration blocks, resources, data sources, backends, variables,
# outputs, credentials, or plan/apply surface.

terraform {
  required_version = ">= 1.6"

  required_providers {
    aws = {
      source  = "registry.opentofu.org/hashicorp/aws"
      version = ">= 5.0.0"
    }

    cloudflare = {
      source  = "registry.opentofu.org/cloudflare/cloudflare"
      version = ">= 4.0.0"
    }

    kubernetes = {
      source  = "registry.opentofu.org/hashicorp/kubernetes"
      version = ">= 2.30.0"
    }

    oci = {
      source  = "registry.opentofu.org/opentofu/oci"
      version = ">= 6.0.0"
    }

    vault = {
      source  = "registry.opentofu.org/hashicorp/vault"
      version = ">= 4.0.0"
    }
  }
}
