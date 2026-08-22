# OpenTofu module — identity µservice — context: guest-on-oci
# Authority: ADR-0131 + ADR-0244 + ADR-0243 + ADR-0329 + ADR-0330 + ADR-0331
#            memory feedback_oci_always_free_maximization_2026_05_20
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: tenant's own OCI tenancy with OKE.
# Note: This is the *paid* OCI guest variant. demo_trial tenants are routed to
#       the iac/oci-guest/always-free/ sibling module per ADR-0329 §D-10 and
#       ADR-0330 §B.3.2.
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    oci        = { source = "oracle/oci", version = "~> 6.0" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
  }

  backend "s3" {
    # OCI Object Storage S3-compatible endpoint:
    # endpoint=https://<namespace>.compat.objectstorage.<region>.oraclecloud.com
    # bucket=<tenant>-tofu-state, key="identity/<tenant>.tfstate",
    # Lock table: Autonomous DB via the cloud-iac OCI lock-table module.
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. This module accepts paid only. demo_trial routes to iac/oci-guest/always-free/."
  type        = string

  validation {
    condition     = var.tenant_class == "paid"
    error_message = "iac/guest-on-oci/ accepts tenant_class = paid only. demo_trial MUST use iac/oci-guest/always-free/ per ADR-0330 §B.3.2."
  }
}

variable "billing_components" {
  description = "Subset of {revenue_share, per_seat, per_usage} per ADR-0330 §B.2."
  type        = list(string)
  default     = []

  validation {
    condition = alltrue([
      for c in var.billing_components : contains(["revenue_share", "per_seat", "per_usage"], c)
    ])
    error_message = "billing_components MUST be a subset of {revenue_share, per_seat, per_usage}."
  }
}

variable "tenant_id" {
  type = string

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{2,62}[a-z0-9]$", var.tenant_id))
    error_message = "paid tenant_id MUST NOT carry the demo_ prefix; format per ADR-0244 §D-1."
  }
}

variable "oci_compartment_id" { type = string }
variable "oke_cluster_id" { type = string }

variable "oci_region" {
  type    = string
  default = "us-ashburn-1"
}

variable "tenant_vault_id" {
  type        = string
  description = "OCI Vault OCID for the tenant. Required when byok_enabled."
  default     = ""
}

variable "tenant_kms_key_id" {
  type        = string
  description = "OCI Vault Key OCID for BYOK envelope encryption."
  default     = ""
}

variable "byok_enabled" {
  description = "BYOK opt-in per ADR-0255 §D-4. paid-only by ADR-0330 §B.3.7."
  type        = bool
  default     = false
}

variable "compliance_packs" {
  description = "Activated compliance packs per ADR-0251."
  type        = list(string)
  default     = []
}

# -------------------------------------------------------------------------
# Gates
# -------------------------------------------------------------------------

resource "terraform_data" "tenant_class_gate_checks" {
  lifecycle {
    precondition {
      condition     = !var.byok_enabled || (length(var.tenant_vault_id) > 0 && length(var.tenant_kms_key_id) > 0)
      error_message = "BYOK enabled but tenant_vault_id/tenant_kms_key_id are empty — refusing to provision."
    }
  }
}

# -------------------------------------------------------------------------
# Resources
# -------------------------------------------------------------------------

resource "oci_identity_policy" "identity_iam" {
  compartment_id = var.oci_compartment_id
  name           = "oyatie-identity-iam-${var.tenant_id}"
  description    = "identity µservice OCI policy for tenant ${var.tenant_id} (paid)"
  statements = [
    "allow dynamic-group identity-${var.tenant_id} to use vaults in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group identity-${var.tenant_id} to use keys in compartment id ${var.oci_compartment_id}",
    "allow dynamic-group identity-${var.tenant_id} to read objectstorage-namespaces in compartment id ${var.oci_compartment_id}"
  ]
}

resource "kubernetes_namespace" "identity" {
  metadata {
    name = "identity-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "identity"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/data-class"         = "pii-identifying"
      "oyatie.io/deployment-context" = "guest-on-oci"
    }
    annotations = {
      "oyatie.io/billing-components" = join(",", var.billing_components)
      "oyatie.io/compliance-packs"   = join(",", var.compliance_packs)
      "oyatie.io/byok-enabled"       = tostring(var.byok_enabled)
    }
  }
}

resource "helm_release" "zitadel" {
  name      = "zitadel"
  namespace = kubernetes_namespace.identity.metadata[0].name
  chart     = "${path.module}/../helm/zitadel"

  values = [
    yamlencode({
      image             = { repository = "ghcr.io/zitadel/zitadel", tag = "v2.65.0" }
      tenantId          = var.tenant_id
      tenantClass       = var.tenant_class
      billingComponents = var.billing_components
      compliancePacks   = var.compliance_packs

      byok = {
        enabled  = var.byok_enabled
        vaultId  = var.tenant_vault_id
        kmsKeyId = var.tenant_kms_key_id
      }

      resources = {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
      }
    })
  ]
}

# -------------------------------------------------------------------------
# Outputs
# -------------------------------------------------------------------------

output "identity_namespace" { value = kubernetes_namespace.identity.metadata[0].name }
output "tenant_class" { value = var.tenant_class }
output "billing_components" { value = var.billing_components }
output "deployment_context" { value = "guest-on-oci" }
output "byok_enabled" { value = var.byok_enabled }
