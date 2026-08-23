# Terraform module — feature-flags
# Binding ADR: ADR-0159 + ADR-0248 (cellular architecture) + ADR-0254 (K8s)
# Provisions: Postgres (Patroni + Citus), ClickHouse, Kafka topics,
#             OpenBao policies, DNS HTTPS RR for ECH, per-cell deployment

terraform {
  required_version = ">= 1.7"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = ">= 2.25"
    }
    helm = {
      source  = "hashicorp/helm"
      version = ">= 2.12"
    }
    openbao = {
      source  = "openbao/openbao"
      version = ">= 0.1"
    }
    clickhouse = {
      source  = "ClickHouse/clickhouse"
      version = ">= 0.7"
    }
  }
}

variable "cell_id" {
  description = "Cell ID for this deployment (e.g., us-east-cell-1)"
  type        = string
}

variable "cell_tier" {
  description = "Cell tier per ADR-0248"
  type        = number
  default     = 2
}

variable "sovereign_pack" {
  description = "Sovereign compliance pack (gdpr-eu, kr-isms-p, fedramp-high, or empty)"
  type        = string
  default     = ""
}

variable "postgres_replica_count" {
  description = "Number of Postgres replicas (1 primary + N standbys)"
  type        = number
  default     = 3
}

variable "eval_replica_count" {
  description = "Initial number of flag evaluator replicas"
  type        = number
  default     = 4
}

# ─────────────────────────────────────────────────────────────────────────────
# Kubernetes namespace
# ─────────────────────────────────────────────────────────────────────────────
resource "kubernetes_namespace" "feature_flags" {
  metadata {
    name = "feature-flags"
    labels = {
      microservice    = "feature-flags"
      cell_id         = var.cell_id
      cell_tier       = tostring(var.cell_tier)
      sovereign_pack  = var.sovereign_pack
      "binding-adr"   = "ADR-0159"
    }
  }
}

# ─────────────────────────────────────────────────────────────────────────────
# Helm release: feature-flags evaluator
# ─────────────────────────────────────────────────────────────────────────────
resource "helm_release" "feature_flags_evaluator" {
  name       = "feature-flags-evaluator"
  namespace  = kubernetes_namespace.feature_flags.metadata[0].name
  chart      = "../helm/feature-flags"
  version    = "1.0.0"

  values = [
    file("${path.module}/../helm-values.yaml"),
  ]

  set {
    name  = "cell.id"
    value = var.cell_id
  }

  set {
    name  = "replicaCount"
    value = tostring(var.eval_replica_count)
  }

  set {
    name  = "global.sovereignPack"
    value = var.sovereign_pack
  }
}

# ─────────────────────────────────────────────────────────────────────────────
# OpenBao policy binding
# ─────────────────────────────────────────────────────────────────────────────
resource "openbao_policy" "feature_flags" {
  name   = "feature-flags-${var.cell_id}"
  policy = file("${path.module}/../openbao-policy.hcl")
}

resource "openbao_auth_backend" "kubernetes" {
  type = "kubernetes"
  path = "kubernetes-${var.cell_id}"
}

resource "openbao_kubernetes_auth_backend_role" "feature_flags_evaluator" {
  backend                          = openbao_auth_backend.kubernetes.path
  role_name                        = "feature-flags-evaluator"
  bound_service_account_names      = ["feature-flags-evaluator"]
  bound_service_account_namespaces = ["feature-flags"]
  token_policies                   = [openbao_policy.feature_flags.name]
  token_ttl                        = 60   # ≤60s per ADR-0296
  token_max_ttl                    = 60
}

# ─────────────────────────────────────────────────────────────────────────────
# Kafka topics (kill-switch broadcast + event emission)
# ─────────────────────────────────────────────────────────────────────────────
resource "kubernetes_manifest" "kafka_topic_killswitch" {
  manifest = {
    apiVersion = "kafka.strimzi.io/v1beta2"
    kind       = "KafkaTopic"
    metadata = {
      name      = "oyatie.feature-flags.killswitch-engaged"
      namespace = "kafka"
      labels = {
        "strimzi.io/cluster" = "oyatie-kafka"
        "binding-adr"        = "ADR-0159"
      }
    }
    spec = {
      partitions = 50   # One per cell; kill-switch fan-out
      replicas   = 3
      config = {
        "retention.ms"       = "604800000"  # 7 days
        "cleanup.policy"     = "delete"
        "compression.type"   = "lz4"
        "min.insync.replicas" = "2"
      }
    }
  }
}

resource "kubernetes_manifest" "kafka_topic_flag_state" {
  manifest = {
    apiVersion = "kafka.strimzi.io/v1beta2"
    kind       = "KafkaTopic"
    metadata = {
      name      = "oyatie.feature-flags.flag-state-changed"
      namespace = "kafka"
    }
    spec = {
      partitions = 100
      replicas   = 3
      config = {
        "retention.ms"  = "2592000000"  # 30 days
        "cleanup.policy" = "delete"
      }
    }
  }
}

# ─────────────────────────────────────────────────────────────────────────────
# ClickHouse database for experiment metric attribution
# ─────────────────────────────────────────────────────────────────────────────
resource "clickhouse_database" "feature_flags_experiments" {
  name    = "feature_flags"
  cluster = "oyatie-clickhouse-${var.cell_id}"
}

resource "clickhouse_table" "audit_events" {
  database = clickhouse_database.feature_flags_experiments.name
  cluster  = "oyatie-clickhouse-${var.cell_id}"
  name     = "feature_flags_audit_events"

  engine = {
    name = "ReplicatedMergeTree"
    parameters = [
      "/clickhouse/tables/{shard}/feature_flags_audit_events",
      "{replica}"
    ]
  }

  order_by    = ["tenant_id", "timestamp", "event_class"]
  partition_by = ["toYYYYMM(timestamp)"]

  columns = [
    { name = "event_id", type = "UUID" },
    { name = "event_class", type = "LowCardinality(String)" },
    { name = "flag_key", type = "String" },
    { name = "tenant_id", type = "String" },
    { name = "timestamp", type = "DateTime64(3, 'UTC')" },
    { name = "hlc_timestamp", type = "String" },
    { name = "actor_principal_id", type = "String" },
    { name = "audit_chain_id", type = "UUID" },
    { name = "audit_sealed", type = "Bool" },
    { name = "replayed", type = "Bool" },
    { name = "payload_json", type = "String" }
  ]

  settings = {
    index_granularity = 8192
  }
}

# ─────────────────────────────────────────────────────────────────────────────
# Outputs
# ─────────────────────────────────────────────────────────────────────────────
output "namespace" {
  value = kubernetes_namespace.feature_flags.metadata[0].name
}

output "openbao_policy_name" {
  value = openbao_policy.feature_flags.name
}
