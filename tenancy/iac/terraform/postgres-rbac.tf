# Postgres RBAC managed via Terraform per tenancy/policy/rls-isolation.md
# Invariant RLS-04: app role MUST NOT have bypassrls; only the JIT-elevated tenancy-admin-jit role can.
# UI/CLI mutation outside Terraform is forbidden by Postgres `pg_hba.conf` + per-role lockdown.
# This Terraform is the source-of-truth for Postgres roles + RLS-bypass policy.

terraform {
  required_providers {
    postgresql = {
      source  = "cyrilgdn/postgresql"
      version = "~> 1.22"
    }
    openbao = {
      source  = "openbao/openbao"
      version = "~> 0.1"
    }
  }
}

provider "postgresql" {
  host     = var.postgres_host
  port     = 5432
  database = "tenancy"
  username = "postgres"  # only Terraform runs as postgres; runtime apps run as tenancy_app
  password = var.postgres_admin_password  # OpenBao-issued JIT
  sslmode  = "require"
  superuser = true
}

# =====================================================================
# Runtime app role — used by tenancy crates + every workload µservice's adapter.
# CRITICAL: bypassrls = false (enforced; cannot be changed via UI).
# =====================================================================
resource "postgresql_role" "tenancy_app" {
  name                = "tenancy_app"
  login               = true
  password            = data.openbao_kv_v2_secret.tenancy_app_password.data["password"]
  bypass_row_level_security = false        # ENFORCED per policy/rls-isolation.md Invariant RLS-04
  superuser           = false
  create_role         = false
  create_database     = false
  inherit             = true
  replication         = false
  connection_limit    = 500                # per capacity-model.md max connections / pod × pods
  encrypted_password  = true
  search_path         = ["tenancy", "public"]
}

# =====================================================================
# JIT-elevated admin role — used ONLY for DDL changes (RLS policy install).
# bypassrls = true, but role login is via OpenBao 2-person-rule JIT (≤1h TTL).
# Used by IP-006 RLS generator + emergency interventions.
# =====================================================================
resource "postgresql_role" "tenancy_admin_jit" {
  name                = "tenancy_admin_jit"
  login               = true
  password            = data.openbao_kv_v2_secret.tenancy_admin_jit_password.data["password"]
  bypass_row_level_security = true         # ONLY this role can bypass RLS
  superuser           = false              # NOT a superuser
  create_role         = false
  create_database     = false
  inherit             = true
  replication         = false
  valid_until         = "2026-05-18 00:00:00 UTC"  # JIT TTL; OpenBao re-issues on grant
}

# =====================================================================
# Auditor role — read-only via folder-scoped grants (issued JIT per Cedar auditor-scope).
# =====================================================================
resource "postgresql_role" "tenancy_auditor_jit" {
  name                = "tenancy_auditor_jit"
  login               = true
  password            = data.openbao_kv_v2_secret.auditor_jit_password.data["password"]
  bypass_row_level_security = false
  superuser           = false
  create_role         = false
  create_database     = false
  inherit             = true
  valid_until         = "2026-05-17 04:00:00 UTC"  # JIT TTL ≤ 4h
}

# =====================================================================
# Replication role — used by Patroni for sync + async replicas.
# =====================================================================
resource "postgresql_role" "replication" {
  name                = "replication"
  login               = true
  password            = data.openbao_kv_v2_secret.replication_password.data["password"]
  replication         = true
  bypass_row_level_security = false
  superuser           = false
}

# =====================================================================
# Grants: tenancy_app gets SELECT/INSERT/UPDATE/DELETE on tenancy schema.
# =====================================================================
resource "postgresql_grant" "tenancy_app_dml" {
  database    = "tenancy"
  role        = postgresql_role.tenancy_app.name
  schema      = "tenancy"
  object_type = "table"
  privileges  = ["SELECT", "INSERT", "UPDATE", "DELETE"]
}

# Auditor: SELECT only, on the tables exposed to audit (per Cedar auditor-scope).
resource "postgresql_grant" "auditor_select" {
  database    = "tenancy"
  role        = postgresql_role.tenancy_auditor_jit.name
  schema      = "tenancy"
  object_type = "table"
  privileges  = ["SELECT"]
}

# =====================================================================
# OpenBao secret references (sourced; never embedded in this file).
# =====================================================================
data "openbao_kv_v2_secret" "tenancy_app_password" {
  mount_path = "secret"
  name       = "tenancy/postgres/tenancy_app"
}

data "openbao_kv_v2_secret" "tenancy_admin_jit_password" {
  mount_path = "secret"
  name       = "tenancy/postgres/tenancy_admin_jit"
}

data "openbao_kv_v2_secret" "auditor_jit_password" {
  mount_path = "secret"
  name       = "tenancy/postgres/auditor_jit"
}

data "openbao_kv_v2_secret" "replication_password" {
  mount_path = "secret"
  name       = "tenancy/postgres/replication"
}

# =====================================================================
# Variables
# =====================================================================
variable "postgres_host" {
  type     = string
  description = "Postgres primary endpoint (per-pack)"
}

variable "postgres_admin_password" {
  type      = string
  sensitive = true
  description = "JIT-issued via OpenBao for Terraform runs only"
}
