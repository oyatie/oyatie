# Grafana RBAC managed via Terraform per microservices/observability/policy/tenant-isolation.md FM-06.
# UI-editing forbidden by grafana.ini in helm/grafana/values.yaml.
# This Terraform is the source-of-truth for Grafana orgs + folders + role-grants.

terraform {
  required_providers {
    grafana = {
      source  = "grafana/grafana"
      version = "~> 3.10"
    }
  }
}

provider "grafana" {
  url  = "https://grafana-${var.pack}.oyatie.dev"
  auth = "${var.grafana_admin_token}"  # injected via OpenBao
}

# Per-tenant Grafana org (1 org per tenant; isolation boundary)
resource "grafana_organization" "tenant" {
  for_each = var.tenants
  name     = each.value.hashed_tenant_id
}

# Per-tenant folder; readonly for tenant operators
resource "grafana_folder" "tenant_dashboards" {
  for_each = var.tenants
  title    = "tenant-${each.value.hashed_tenant_id}"
  org_id   = grafana_organization.tenant[each.key].org_id
}

# Tenant-operator role: read-only on own folder; cannot pivot to other orgs
resource "grafana_role" "tenant_operator" {
  for_each   = var.tenants
  name       = "tenant-operator-${each.value.hashed_tenant_id}"
  uid        = "tenant-operator-${each.value.hashed_tenant_id}"
  version    = 1
  global     = false
  group      = "tenant"
  org_id     = grafana_organization.tenant[each.key].org_id
  permissions {
    action = "folders:read"
    scope  = "folders:uid:${grafana_folder.tenant_dashboards[each.key].uid}"
  }
}

# Internal-operator role (ops-sre-reliability): cross-tenant readonly with JIT only
resource "grafana_role" "internal_operator_jit" {
  name    = "internal-operator-jit"
  uid     = "internal-operator-jit"
  version = 1
  global  = true
  group   = "internal"
  permissions {
    action = "datasources:query"
    scope  = "datasources:*"
  }
}

# Auditor role: time-boxed JIT per policy/auditor-scope.cedar
resource "grafana_role" "auditor_jit" {
  name    = "auditor-jit"
  uid     = "auditor-jit"
  version = 1
  global  = true
  group   = "auditor"
  permissions {
    action = "folders:read"
    scope  = "folders:uid:*"  # filtered to scoped_tenants at the OIDC layer
  }
}

variable "tenants" {
  type = map(object({
    hashed_tenant_id = string
    pack             = string
  }))
  default = {}
}

variable "pack" { type = string }
variable "grafana_admin_token" {
  type = string
  sensitive = true
}
