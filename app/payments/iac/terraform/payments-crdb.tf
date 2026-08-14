# Terraform: payments CockroachDB cluster + schema
# ADR-0244: tenant_id on every row; ADR-0248: cellular architecture Tier-1 control plane
# ADR-0252: TrueTime opt-in for settlement BC via CRDB multi-region table
# ADR-0296: credentials via OpenBao (NOT Terraform state)

terraform {
  required_version = ">= 1.8"
  required_providers {
    cockroach = {
      source  = "cockroachdb/cockroach"
      version = "~> 1.5"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.30"
    }
  }
  backend "gcs" {
    bucket = "oyatie-terraform-state"
    # Stable state prefix: the pre-existing remote state lives under
    # microservices/payments/crdb; changing the prefix would select a new,
    # empty remote state and re-create the cluster/db/user/ExternalSecret as
    # unmanaged. Migrate atomically before ever switching the prefix.
    prefix = "microservices/payments/crdb"
  }
}

# ─── CRDB Cluster (multi-region, Tier-1 control plane) ───────────────────────

resource "cockroach_cluster" "payments" {
  name           = "payments"
  cloud_provider = "GCP"
  plan           = "DEDICATED"
  serverless     = false

  dedicated = {
    storage_gib       = 500
    num_virtual_cpus  = 8
  }

  regions = [
    { name = "us-east1",         node_count = 3 },
    { name = "europe-west1",     node_count = 3 },
    { name = "asia-northeast3",  node_count = 3 },  # KR (Seoul)
  ]

  # TrueTime-equivalent: CRDB global tables for settlement BC
  # PCI isolation: dedicated cluster, not shared with other µservices
}

# ─── Database ─────────────────────────────────────────────────────────────────

resource "cockroach_database" "payments" {
  name       = "payments"
  cluster_id = cockroach_cluster.payments.id
}

# ─── SQL User (app principal) ─────────────────────────────────────────────────
# Credential is written to OpenBao post-provisioning — NOT stored in TF state.

resource "cockroach_sql_user" "payments_app" {
  name       = "payments_app"
  cluster_id = cockroach_cluster.payments.id
  # password intentionally omitted; rotated via OpenBao dynamic credentials
}

# ─── Kubernetes ExternalSecret binding ───────────────────────────────────────

resource "kubernetes_manifest" "payments_crdb_external_secret" {
  manifest = {
    apiVersion = "external-secrets.io/v1beta1"
    kind       = "ExternalSecret"
    metadata = {
      name      = "payments-crdb-credentials"
      namespace = "payments"
    }
    spec = {
      refreshInterval = "1m"
      secretStoreRef = {
        name = "openbao-cluster-store"
        kind = "ClusterSecretStore"
      }
      target = {
        name = "payments-crdb-credentials"
        template = {
          data = {
            DATABASE_URL = "{{ .url }}"
          }
        }
      }
      data = [{
        secretKey = "url"
        remoteRef = {
          # Key is relative to the store's configured KV-v2 mount (path="secret");
          # ESO performs the mount/data translation itself, so no secret/data/ prefix.
          key      = "payments/crdb"
          property = "url"
        }
      }]
    }
  }
}

# ─── Outputs (non-sensitive only) ────────────────────────────────────────────

output "payments_crdb_cluster_id" {
  value       = cockroach_cluster.payments.id
  description = "CRDB cluster ID for payments µservice"
}

output "payments_crdb_regions" {
  value       = cockroach_cluster.payments.regions
  description = "Active regions for payments CRDB cluster"
}
