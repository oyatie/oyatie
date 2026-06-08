# Per-pack RBAC + IAM scaffold for foundry-providers µservice.
# Pinned to OCI / OKE; OpenBao policy binding referenced from cloud-secrets µservice.
# References: ADR-0117, ADR-0131, microservices/intelligence/policy/credential-isolation.md (CI-INV-05).

terraform {
  required_version = ">= 1.7"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = ">= 5.30"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = ">= 2.30"
    }
    openbao = {
      source  = "openbao/openbao"
      version = ">= 1.4"
    }
  }
}

variable "pack" {
  description = "Residency pack code (e.g., kr, eu, us, us-healthcare)"
  type        = string
}

variable "compartment_ocid" {
  description = "OCI compartment OCID for this pack"
  type        = string
}

# Kubernetes ServiceAccount per adapter family (spiffe-bound).
resource "kubernetes_service_account" "router_rest" {
  metadata {
    name      = "oya-intelligence-providers-router-rest"
    namespace = "oya-intelligence-providers-${var.pack}"
    labels = {
      "app.kubernetes.io/name" = "oya-intelligence-providers-router-rest"
      "spiffe.io/identity"     = "spiffe://oyatie.dev/foundry-providers/router-rest/pack/${var.pack}"
    }
  }
}

resource "kubernetes_service_account" "adapter_anthropic_api" {
  metadata {
    name      = "oya-intelligence-providers-adapter-anthropic-api"
    namespace = "oya-intelligence-providers-${var.pack}"
    labels = {
      "app.kubernetes.io/name" = "oya-intelligence-providers-adapter-anthropic-api"
      "spiffe.io/identity"     = "spiffe://oyatie.dev/foundry-providers/adapter-anthropic-api/pack/${var.pack}"
    }
  }
}
# (similar SA blocks for the other 7 adapters; elided for brevity)

# OpenBao policy binding: ONLY read on providers/* for the pack+tenant pair.
# Per credential-isolation.md CI-INV-05.
resource "openbao_policy" "foundry_providers_adapter_anthropic_api" {
  name   = "oya-intelligence-providers-adapter-anthropic-api-${var.pack}"
  policy = <<-HCL
    # Read-only on Anthropic API credentials for the pack.
    path "secret/data/${var.pack}/+/providers/anthropic/+" {
      capabilities = ["read"]
    }

    # Explicit deny on any non-read action.
    path "secret/+" {
      capabilities = ["deny"]
    }
  HCL
}
# (similar policy blocks for the other adapters; elided for brevity)

# KMS keyring for per-pack signing keys.
resource "oci_kms_key" "router_signing_key" {
  compartment_id = var.compartment_ocid
  display_name   = "oya-intelligence-providers-router-signing-${var.pack}"
  key_shape {
    algorithm = "RSA"
    length    = 4096
  }
}

# Output ServiceAccount references for the Helm chart to consume.
output "router_rest_sa_name" {
  value = kubernetes_service_account.router_rest.metadata[0].name
}

output "adapter_anthropic_api_sa_name" {
  value = kubernetes_service_account.adapter_anthropic_api.metadata[0].name
}

output "openbao_policy_anthropic_api" {
  value = openbao_policy.foundry_providers_adapter_anthropic_api.name
}
