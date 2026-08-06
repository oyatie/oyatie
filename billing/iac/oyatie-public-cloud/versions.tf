# cloud-billing OpenTofu module for the oyatie-public-cloud deployment context
#
# Per ADR-0328 §D-16: every module declares OpenTofu + provider version pins.
# Per ADR-0039: every module is sigstore + cosign signed.
# Per ADR-0218: tenant-bound deployment context is oyatie-public-cloud (SaaS default).

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.13"
    }
    postgresql = {
      source  = "cyrilgdn/postgresql"
      version = "~> 1.23"
    }
    cosign = {
      source  = "chainguard-dev/cosign"
      version = "~> 0.5"
    }
  }
}
