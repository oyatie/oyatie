# OpenTofu module — identity µservice — context: oci-guest / always-free
# Authority: ADR-0329 §B2.026 reword of "OCI Always Free demo_trial = Always Free"
#            ADR-0330 §B.3.2 demo_trial defaults to OCI Always Free
#            ADR-0331 §D-8.2..D-8.5 Always-Free module validates tenant_class = demo_trial
#            memory feedback_oci_always_free_maximization_2026_05_20
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: oyatie demo_trial tenants only, hosted on OCI's perpetual Always Free
#         envelope (2× Ampere A1 = 4 OCPU + 24 GB RAM, 200 GB block, 2 Autonomous
#         Databases × 20 GB, 10 TB egress/month, 1 LB, Vault, Streaming,
#         Functions, API Gateway, WAF, Bastion).
# Note: This module REFUSES tenant_class = paid. paid tenants use
#       iac/guest-on-oci/ regardless of cost preference.
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    oci        = { source = "oracle/oci", version = "~> 6.0" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. iac/oci-guest/always-free/ accepts demo_trial only per ADR-0331 §D-8.4."
  type        = string

  validation {
    condition     = var.tenant_class == "demo_trial"
    error_message = "iac/oci-guest/always-free/ MUST be applied only for tenant_class = demo_trial per ADR-0331 §D-8.4. Paid tenants use iac/guest-on-oci/."
  }
}

variable "billing_components" {
  description = "Must be empty for demo_trial per ADR-0330 §B.2.1."
  type        = list(string)
  default     = []

  validation {
    condition     = length(var.billing_components) == 0
    error_message = "demo_trial billing_components MUST be empty per ADR-0330 §B.2.1."
  }
}

variable "tenant_id" {
  description = "Tenant identifier per ADR-0244 §D-1. demo_trial tenants carry the demo_ prefix per ADR-0330 §B.3.13."
  type        = string

  validation {
    condition     = can(regex("^demo_[a-z0-9][a-z0-9-]{2,57}[a-z0-9]$", var.tenant_id))
    error_message = "demo_trial tenant_id MUST start with demo_ prefix per ADR-0330 §B.3.13."
  }
}

variable "oci_compartment_id" { type = string }

variable "oci_region" {
  type        = string
  default     = "us-ashburn-1"
  description = "OCI region. Always Free home region is fixed once chosen by the tenant — cannot be changed."
}

variable "trial_expires_at" {
  type        = string
  description = "ISO-8601 UTC timestamp when the demo_trial window closes per ADR-0330 §B.3.4 (default 30 days from creation)."
}

variable "demo_trial_caps" {
  description = "Per-microservice demo_trial caps per ADR-0331 §D-5. Identity caps below are bespoke."
  type = object({
    max_users_per_tenant      = number
    max_passkeys_per_user     = number
    max_active_sessions       = number
    scim_ops_per_minute       = number
    oidc_token_issues_per_day = number
  })
  default = {
    max_users_per_tenant      = 25
    max_passkeys_per_user     = 5
    max_active_sessions       = 50
    scim_ops_per_minute       = 30
    oidc_token_issues_per_day = 5000
  }
}

# -------------------------------------------------------------------------
# Always Free envelope: hard caps from OCI documentation, enforced as
# OpenTofu preconditions so an accidental tenant_class flip cannot exceed
# the free envelope.
# -------------------------------------------------------------------------

locals {
  always_free_envelope = {
    a1_ocpu_total           = 4 # 2× Ampere A1
    a1_memory_gib_total     = 24
    block_storage_gib_total = 200
    autonomous_db_instances = 2
    autonomous_db_gib_each  = 20
    egress_gib_per_month    = 10240
    load_balancers          = 1
  }
}

# -------------------------------------------------------------------------
# Resources sized to fit the Always Free envelope.
# -------------------------------------------------------------------------

resource "kubernetes_namespace" "identity" {
  metadata {
    name = "identity-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "identity"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = "demo_trial"
      "oyatie.io/data-class"         = "pii-identifying"
      "oyatie.io/deployment-context" = "guest-on-oci"
      "oyatie.io/oci-profile"        = "always-free"
    }
    annotations = {
      "oyatie.io/trial-expires-at" = var.trial_expires_at
      "oyatie.io/cap-shape"        = jsonencode(var.demo_trial_caps)
    }
  }
}

resource "helm_release" "zitadel_always_free" {
  name      = "zitadel"
  namespace = kubernetes_namespace.identity.metadata[0].name
  chart     = "${path.module}/../../helm/zitadel"

  values = [
    yamlencode({
      image = {
        repository = "ghcr.io/zitadel/zitadel"
        tag        = "v2.65.0"
      }

      # Single replica fits Always Free envelope.
      # The unavailability risk is acceptable because demo_trial SLO is best-effort
      # per ADR-0330 §B.3.5.
      replicaCount = 1

      resources = {
        requests = { cpu = "500m", memory = "1.5Gi" }
        limits   = { cpu = "1500m", memory = "3Gi" }
      }

      autoscaling = { enabled = false }

      tenantId          = var.tenant_id
      tenantClass       = "demo_trial"
      billingComponents = []
      compliancePacks   = []                  # demo_trial cannot activate packs per ADR-0330 §B.3.6
      byok              = { enabled = false } # demo_trial cannot opt into BYOK per ADR-0330 §B.3.7

      caps = var.demo_trial_caps

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
        OYATIE_DEMO_TRIAL_EXPIRES_AT             = var.trial_expires_at
        OYATIE_OCI_ALWAYS_FREE_PROFILE           = "true"
      }
    })
  ]
}

# -------------------------------------------------------------------------
# Outputs
# -------------------------------------------------------------------------

output "identity_namespace" { value = kubernetes_namespace.identity.metadata[0].name }
output "tenant_class" { value = "demo_trial" }
output "deployment_context" { value = "guest-on-oci" }
output "oci_profile" { value = "always-free" }
output "trial_expires_at" { value = var.trial_expires_at }
output "always_free_envelope" { value = local.always_free_envelope }
