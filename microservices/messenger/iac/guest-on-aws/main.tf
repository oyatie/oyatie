# OpenTofu module — messenger µservice — context: guest-on-aws
# Tenant's own AWS account; oyatie runs as guest stack; common for B2B-work
# enterprise customers who already standardized on AWS for their workloads.
# Wave 15A-MESSENGER-FIX authored 2026-05-21
# Per ADR-0131 + ADR-0244 + ADR-0246 MLS + ADR-0251 compliance packs + ADR-0255 BYOK
# + ADR-0328 §D-15 (multi-context deployment) + ADR-0331 tenant_class.
#
# `tenant_class_default = byo-cloud` per /specs/master-plan-sequencing.json deployment_contexts.
# In practice the tenant is always paid because BYOC implies an enterprise contract.

variable "tenant_id" { type = string }
variable "aws_region" {
  type    = string
  default = "us-east-1"
}
variable "vpc_id" { type = string }
variable "private_subnets" { type = list(string) }
variable "eks_cluster_name" { type = string }

variable "tenant_kms_key_id" {
  type        = string
  description = "Tenant-owned KMS key for BYOK envelope encryption per ADR-0255"
}

variable "tenant_class" {
  type        = string
  default     = "paid"
  description = "Per ADR-0331 tenant_class ∈ {demo_trial, paid}. guest-on-aws → paid (BYOC implies enterprise contract)."
  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be one of: demo_trial, paid."
  }
}

variable "paid_billing_components" {
  type    = list(string)
  default = ["per_seat", "per_usage"]
}

variable "audience_mode" {
  type    = string
  default = "B2B-work"
}

variable "mls_e2ee_mode" {
  type        = string
  default     = "tenant_opt_in_with_recovery_key_escrow"
  description = "Per ADR-MSG-001 + memory feedback_mls_rfc_9420_e2ee_personal_messenger"
}

variable "mobile_app_bundle_peers" {
  type    = list(string)
  default = ["mail", "social", "community"]
}

variable "compliance_packs" {
  type    = list(string)
  default = ["soc2", "iso27001"]
}

variable "regulatory_packs" {
  type    = list(string)
  default = ["us", "eu", "kr"]
}

# Provider must be configured with tenant's role-assume credentials per Layer-A bootstrap
provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      "tenant_id"          = var.tenant_id
      "tenant_class"       = var.tenant_class
      "microservice"       = "messenger"
      "data-class"         = "PII_IDENTIFYING"
      "deployment-context" = "guest-on-aws"
      "managed-by"         = "oyatie-opentofu"
      "mobile-app-bundle"  = "messenger+mail+social+community"
    }
  }
}

resource "aws_kms_grant" "messenger_byok_grant" {
  name              = "messenger-byok-grant-${var.tenant_id}"
  key_id            = var.tenant_kms_key_id
  grantee_principal = aws_iam_role.messenger_irsa.arn
  operations        = ["Decrypt", "Encrypt", "GenerateDataKey", "DescribeKey"]
}

resource "aws_iam_role" "messenger_irsa" {
  name = "messenger-irsa-${var.tenant_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = "arn:aws:iam::TENANT_ACCOUNT:oidc-provider/oidc.eks.${var.aws_region}.amazonaws.com/id/PLACEHOLDER" }
      Action    = "sts:AssumeRoleWithWebIdentity"
    }]
  })
}

resource "kubernetes_namespace" "messenger" {
  metadata {
    name = "messenger-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice"       = "messenger"
      "oyatie.io/tenant_id"          = var.tenant_id
      "oyatie.io/tenant-class"       = var.tenant_class
      "oyatie.io/deployment-context" = "guest-on-aws"
      "oyatie.io/data-class"         = "PII_IDENTIFYING"
      "oyatie.io/audience-mode"      = var.audience_mode
      "oyatie.io/mls-e2ee-mode"      = var.mls_e2ee_mode
      "oyatie.io/mobile-app-bundle"  = "messenger+mail+social+community"
    }
  }
}

resource "helm_release" "messenger" {
  name      = "messenger"
  namespace = kubernetes_namespace.messenger.metadata[0].name
  chart     = "${path.module}/../helm/messenger"
  values = [yamlencode({
    image          = { repository = "registry.oyatie.dev/messenger", tag = "1.0.0-wave-15a" }
    serviceAccount = { annotations = { "eks.amazonaws.com/role-arn" = aws_iam_role.messenger_irsa.arn } }
    byok = {
      enabled   = true
      kmsKeyArn = "arn:aws:kms:${var.aws_region}:TENANT_ACCOUNT:key/${var.tenant_kms_key_id}"
    }
    tenantId              = var.tenant_id
    tenantClass           = var.tenant_class
    paidBillingComponents = var.paid_billing_components
    audienceMode          = var.audience_mode
    mlsE2eeMode           = var.mls_e2ee_mode
    mobileAppBundlePeers  = var.mobile_app_bundle_peers
    compliancePacks       = var.compliance_packs
    regulatoryPacks       = var.regulatory_packs
    env = {
      DEFAULT_MLS_CIPHERSUITE  = "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519"
      MLS_KEY_PACKAGE_TTL_DAYS = "7"
      MLS_WELCOME_TTL_DAYS     = "14"
    }
  })]
}

output "messenger_namespace" { value = kubernetes_namespace.messenger.metadata[0].name }
output "tenant_class" { value = var.tenant_class }
output "mls_e2ee_mode" { value = var.mls_e2ee_mode }
