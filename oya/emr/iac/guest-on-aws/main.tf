# OpenTofu module — EMR µservice — context: guest-on-aws
# Tenant's own AWS account; BAA-eligible; common for US healthcare IDNs
# Wave 15M-B authored 2026-05-21

terraform {
  required_version = ">= 1.7"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.65" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.30" }
    helm = { source = "hashicorp/helm", version = "~> 2.13" }
  }
}

variable "tenant_id" { type = string }
variable "aws_region" {
  type = string
  default = "us-east-1"
}
variable "vpc_id" { type = string }
variable "private_subnets" { type = list(string) }
variable "eks_cluster_name" { type = string }
variable "tenant_kms_key_id" {
  type = string
  description = "Tenant-owned KMS key for BYOK envelope encryption"
}
variable "baa_signed_blob_ref" {
  type = string
  description = "S3 ref to signed BAA document"
}

# Provider must be configured with tenant's role-assume credentials per Layer-A bootstrap
provider "aws" {
  region = var.aws_region
  default_tags { tags = {
    "tenant_id"        = var.tenant_id
    "microservice"     = "emr"
    "data-class"       = "phi-protected-health-information"
    "compliance-pack"  = "HIPAA-2024"
    "managed-by"       = "oyatie-opentofu"
  } }
}

resource "aws_kms_grant" "emr_phi_grant" {
  name              = "emr-phi-grant-${var.tenant_id}"
  key_id            = var.tenant_kms_key_id
  grantee_principal = aws_iam_role.emr_irsa.arn
  operations = ["Decrypt", "Encrypt", "GenerateDataKey", "DescribeKey"]
}

resource "aws_iam_role" "emr_irsa" {
  name = "emr-irsa-${var.tenant_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Federated = "arn:aws:iam::TENANT_ACCOUNT:oidc-provider/oidc.eks.${var.aws_region}.amazonaws.com/id/PLACEHOLDER" }
      Action = "sts:AssumeRoleWithWebIdentity"
    }]
  })
}

resource "kubernetes_namespace" "emr" {
  metadata {
    name = "emr-${var.tenant_id}"
    labels = {
      "oyatie.io/microservice" = "emr"
      "oyatie.io/tenant_id"    = var.tenant_id
      "oyatie.io/data-class"   = "phi-protected-health-information"
      "oyatie.io/compliance-pack" = "HIPAA-2024"
    }
  }
}

resource "helm_release" "emr" {
  name       = "emr"
  namespace  = kubernetes_namespace.emr.metadata[0].name
  chart      = "${path.module}/../../helm/emr"
  values = [yamlencode({
    image = { repository = "registry.oyatie.health/emr", tag = "1.0.0-wave-15m-b" }
    serviceAccount = { annotations = { "eks.amazonaws.com/role-arn" = aws_iam_role.emr_irsa.arn } }
    byok = { enabled = true, kmsKeyArn = "arn:aws:kms:${var.aws_region}:TENANT_ACCOUNT:key/${var.tenant_kms_key_id}" }
    tenantId = var.tenant_id
    baaSignedBlobRef = var.baa_signed_blob_ref
    compliancePacksRequired = ["HIPAA-2024"]
  })]
}

output "emr_namespace" { value = kubernetes_namespace.emr.metadata[0].name }
