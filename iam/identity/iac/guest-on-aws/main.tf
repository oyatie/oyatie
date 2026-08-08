# OpenTofu module — identity µservice — context: guest-on-aws
# Authority: ADR-0131 + ADR-0244 + ADR-0243 + ADR-0329 + ADR-0330 + ADR-0331
#            ADR-0328 §D-15 six canonical deployment contexts
#            ADR-0064 canonical-base + localization
# Wave: 15A-IDENTITY-FIX authored 2026-05-21
# Target: tenant's own AWS account; tenant chose AWS guest deployment.
# Note: AWS IAM is NEVER the app authority — oyatie identity is the authority
#       per ADR-0244. AWS primitives only back the kubernetes / secrets layer.
# Engine: OpenTofu only — NOT HashiCorp Terraform

terraform {
  required_version = ">= 1.7"
  required_providers {
    aws        = { source = "hashicorp/aws", version = "~> 5.65" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm       = { source = "hashicorp/helm", version = "~> 2.13" }
  }

  backend "s3" {
    # Concrete backend wired per-tenant by the cloud-iac bootstrap:
    # bucket=<tenant>-tofu-state, key="identity/<tenant>.tfstate",
    # dynamodb_table=<tenant>-tofu-locks, encrypt=true, kms_key_id=<tenant KMS>.
  }
}

# -------------------------------------------------------------------------
# Variables
# -------------------------------------------------------------------------

variable "tenant_class" {
  description = "Tenant class per ADR-0330. demo_trial guests on AWS pay $0; paid guests bill via the tenant's contracted billing_components."
  type        = string

  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class MUST be one of {demo_trial, paid} per ADR-0330 §B.1.1."
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
  description = "Tenant identifier per ADR-0244 §D-1."
  type        = string

  validation {
    condition     = can(regex("^(demo_)?[a-z0-9][a-z0-9-]{2,62}[a-z0-9]$", var.tenant_id))
    error_message = "tenant_id MUST conform to ADR-0244 §D-1 format."
  }
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

variable "vpc_id" { type = string }
variable "private_subnets" { type = list(string) }
variable "eks_cluster_name" { type = string }

variable "tenant_kms_key_id" {
  type        = string
  description = "Tenant-owned KMS key for BYOK envelope encryption (ADR-0255 §D-4; paid-only)."
  default     = ""
}

variable "compliance_packs" {
  description = "Activated compliance packs per ADR-0251. paid only (ADR-0330 §B.3.6)."
  type        = list(string)
  default     = []
}

variable "byok_enabled" {
  description = "BYOK opt-in per ADR-0255 §D-4. Requires tenant_class = paid per ADR-0330 §B.3.7."
  type        = bool
  default     = false
}

# -------------------------------------------------------------------------
# Cross-bindings: tenant_class = paid is required for BYOK and compliance packs.
# -------------------------------------------------------------------------

resource "terraform_data" "tenant_class_gate_checks" {
  lifecycle {
    precondition {
      condition     = var.tenant_class == "paid" || length(var.compliance_packs) == 0
      error_message = "ADR-0330 §B.3.6: demo_trial tenants MUST NOT activate compliance packs."
    }
    precondition {
      condition     = var.tenant_class == "paid" || !var.byok_enabled
      error_message = "ADR-0330 §B.3.7 + ADR-0255 §D-4: BYOK requires tenant_class = paid."
    }
    precondition {
      condition     = !var.byok_enabled || length(var.tenant_kms_key_id) > 0
      error_message = "BYOK enabled but tenant_kms_key_id is empty — refusing to provision an insecure default."
    }
  }
}

# -------------------------------------------------------------------------
# Provider
# -------------------------------------------------------------------------

provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      "tenant_id"          = var.tenant_id
      "tenant_class"       = var.tenant_class
      "microservice"       = "identity"
      "data-class"         = "pii-identifying"
      "deployment-context" = "guest-on-aws"
      "managed-by"         = "oyatie-opentofu"
    }
  }
}

# -------------------------------------------------------------------------
# IRSA: the identity µservice pod assumes a role with the
# minimum permissions needed to read tenant-scoped KMS keys.
# -------------------------------------------------------------------------

resource "aws_iam_role" "identity_irsa" {
  name = "oyatie-identity-irsa-${var.tenant_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = "arn:aws:iam::TENANT_ACCOUNT:oidc-provider/oidc.eks.${var.aws_region}.amazonaws.com/id/PLACEHOLDER" }
      Action    = "sts:AssumeRoleWithWebIdentity"
    }]
  })
}

resource "aws_kms_grant" "identity_byok_grant" {
  count             = var.byok_enabled ? 1 : 0
  name              = "identity-byok-${var.tenant_id}"
  key_id            = var.tenant_kms_key_id
  grantee_principal = aws_iam_role.identity_irsa.arn
  operations        = ["Decrypt", "Encrypt", "GenerateDataKey", "DescribeKey"]
}

# -------------------------------------------------------------------------
# Kubernetes namespace + Zitadel helm release
# -------------------------------------------------------------------------

resource "kubernetes_namespace" "identity" {
  metadata {
    name = "identity-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "identity"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/data-class"         = "pii-identifying"
      "oyatie.io/deployment-context" = "guest-on-aws"
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
      image = {
        repository = "ghcr.io/zitadel/zitadel"
        tag        = "v2.65.0"
      }
      serviceAccount = {
        annotations = {
          "eks.amazonaws.com/role-arn" = aws_iam_role.identity_irsa.arn
        }
      }
      tenantId          = var.tenant_id
      tenantClass       = var.tenant_class
      billingComponents = var.billing_components
      compliancePacks   = var.compliance_packs

      byok = {
        enabled   = var.byok_enabled
        kmsKeyArn = var.byok_enabled ? "arn:aws:kms:${var.aws_region}:TENANT_ACCOUNT:key/${var.tenant_kms_key_id}" : ""
      }

      resources = var.tenant_class == "demo_trial" ? {
        requests = { cpu = "0.5", memory = "2Gi" }
        limits   = { cpu = "1", memory = "4Gi" }
        } : {
        requests = { cpu = "2", memory = "8Gi" }
        limits   = { cpu = "8", memory = "32Gi" }
      }

      env = {
        OYATIE_TENANT_CLASS_CLAIM_EMISSION       = "true"
        OYATIE_TENANT_CLASS_PRINCIPAL_CLAIM_NAME = "tenant_class"
        OYATIE_BILLING_COMPONENTS_CLAIM_NAME     = "billing_components"
        OYATIE_AWS_PRIMITIVE_AUTHORITY_DISABLED  = "true"
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
output "deployment_context" { value = "guest-on-aws" }
output "byok_enabled" { value = var.byok_enabled }
