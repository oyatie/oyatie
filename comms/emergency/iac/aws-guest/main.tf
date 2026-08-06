// OpenTofu module — ED-IS on AWS guest deployment context.
// Authority: ADR-0332 (in flight) | feedback_zero_handroll_opentofu_only_2026_05_20
// Owner: emergency-medicine-platform-engineer
//
// Provisions: EKS workloads + Aurora Postgres + ElastiCache for Valkey +
//             MSK (or NATS-on-EKS) + KMS keys + IAM roles. Tenant onboarding
//             completes with a single `tofu apply`.

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.50"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}

variable "tenant_id" {
  description = "Reserved-namespace-aware tenant id (e.g., oyatie or customer-corp-001)."
  type        = string
}

variable "cell_id" {
  description = "Cell identifier per ADR-0248 cellular architecture."
  type        = string
}

variable "region" {
  description = "AWS region."
  type        = string
  default     = "us-east-1"
}

variable "compliance_packs" {
  description = "Compliance packs to enable for this deployment."
  type        = list(string)
  default     = ["HIPAA", "SOC2", "HITRUST", "EMTALA"]
}

variable "image_tag" {
  description = "Container image tag for the emergency µservice."
  type        = string
}

module "emergency_cluster" {
  source           = "../_shared/eks-cell"
  tenant_id        = var.tenant_id
  cell_id          = var.cell_id
  region           = var.region
  compliance_packs = var.compliance_packs
}

module "emergency_db" {
  source    = "../_shared/aurora-postgres"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  db_name   = "emergency"
}

module "emergency_valkey" {
  source    = "../_shared/elasticache-valkey"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
}

module "emergency_nats" {
  source        = "../_shared/nats-on-eks"
  tenant_id     = var.tenant_id
  cell_id       = var.cell_id
  stream_prefix = "ed"
}

module "emergency_kms" {
  source    = "../_shared/kms-envelope-key"
  tenant_id = var.tenant_id
  cell_id   = var.cell_id
  pack      = "HIPAA"
}

resource "kubernetes_deployment_v1" "emergency_api_rest" {
  metadata {
    name      = "emergency-api-rest"
    namespace = module.emergency_cluster.namespace
    labels = {
      microservice = "emergency"
      tenant_id    = var.tenant_id
      cell_id      = var.cell_id
    }
  }
  spec {
    replicas = 3
    selector { match_labels = { app = "emergency-api-rest" } }
    template {
      metadata { labels = { app = "emergency-api-rest" } }
      spec {
        container {
          name  = "emergency-api-rest"
          image = "oyatie/emergency:${var.image_tag}"
          port { container_port = 8443 }
          env {
            name  = "OYA_TENANT_ID"
            value = var.tenant_id
          }
          env {
            name  = "OYA_CELL_ID"
            value = var.cell_id
          }
          env {
            name  = "OYA_DB_URL"
            value = module.emergency_db.url
          }
          env {
            name  = "OYA_VALKEY_URL"
            value = module.emergency_valkey.url
          }
          env {
            name  = "OYA_NATS_URL"
            value = module.emergency_nats.url
          }
          env {
            name  = "OYA_KMS_KEY_ARN"
            value = module.emergency_kms.key_arn
          }
        }
      }
    }
  }
}

output "emergency_endpoint" {
  value = "https://emergency.${var.tenant_id}.${var.cell_id}.aws.oyatie.cloud"
}
