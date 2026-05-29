# Terraform-managed Grafana folder + role definitions for the messenger µservice
# Mirrors the observability/iac/terraform/grafana-rbac.tf pattern.
# Per `policy/tenant-scope.cedar` + `policy/auditor-scope.cedar`.

provider "grafana" {
  url  = var.grafana_url
  auth = var.grafana_admin_token # populated via OpenBao SecretReference
}

variable "grafana_url" {
  type = string
}

variable "grafana_admin_token" {
  type      = string
  sensitive = true
}

resource "grafana_folder" "messenger" {
  title = "messenger"
  uid   = "messenger"
}

resource "grafana_folder" "messenger_per_pack" {
  for_each          = toset(["kr", "eu", "us", "us-hc", "jp", "sg", "au", "in", "br", "ae", "ksa"])
  title             = "messenger / pack-${each.key}"
  uid               = "messenger-${each.key}"
  parent_folder_uid = grafana_folder.messenger.uid
}

# Roles
resource "grafana_role" "messenger_tenant_operator" {
  name        = "messenger:tenant-operator"
  description = "Read messenger dashboards scoped to single tenant"
  uid         = "messenger-tenant-operator"
  global      = false
  group       = "messenger"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:messenger"
  }
}

resource "grafana_role" "messenger_sre" {
  name        = "messenger:sre"
  description = "Cross-tenant SRE view; behind admin-RBAC + Cedar"
  uid         = "messenger-sre"
  global      = false
  group       = "messenger"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:messenger"
  }
  permissions {
    action = "alert.rules:read"
    scope  = "folders:uid:messenger"
  }
}

resource "grafana_role" "messenger_auditor" {
  name        = "messenger:auditor"
  description = "Time-boxed engagement-scoped read; per policy/auditor-scope.cedar"
  uid         = "messenger-auditor"
  global      = false
  group       = "messenger"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:messenger"
  }
}
