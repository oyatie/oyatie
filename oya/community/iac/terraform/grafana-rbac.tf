# Terraform-managed Grafana folder + role definitions for the community µservice.
# Mirrors the observability/iac/terraform/grafana-rbac.tf and messenger/iac/terraform/grafana-rbac.tf patterns.
# Per `policy/tenant-scope.cedar` + `policy/auditor-scope.cedar`.

terraform {
  required_providers {
    grafana = {
      source  = "grafana/grafana"
      version = "~> 3.0"
    }
  }
}

provider "grafana" {
  url  = var.grafana_url
  auth = var.grafana_admin_token  # populated via OpenBao SecretReference
}

variable "grafana_url" {
  type = string
}

variable "grafana_admin_token" {
  type      = string
  sensitive = true
}

resource "grafana_folder" "community" {
  title = "community"
  uid   = "community"
}

resource "grafana_folder" "community_per_pack" {
  for_each          = toset(["kr", "eu", "us", "us-hc", "jp", "sg", "au", "in", "br", "ae", "ksa"])
  title             = "community / pack-${each.key}"
  uid               = "community-${each.key}"
  parent_folder_uid = grafana_folder.community.uid
}

# Roles
resource "grafana_role" "community_tenant_operator" {
  name        = "community:tenant-operator"
  description = "Read community dashboards scoped to single tenant"
  uid         = "community-tenant-operator"
  global      = false
  group       = "community"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:community"
  }
}

resource "grafana_role" "community_sre" {
  name        = "community:sre"
  description = "Cross-tenant SRE view; behind admin-RBAC + Cedar"
  uid         = "community-sre"
  global      = false
  group       = "community"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:community"
  }
  permissions {
    action = "alert.rules:read"
    scope  = "folders:uid:community"
  }
}

resource "grafana_role" "community_moderation_lead" {
  name        = "community:moderation-lead"
  description = "Moderation-lane dashboards (queue depth, classifier signal, appeal flow) per ADR-COMM-0001"
  uid         = "community-moderation-lead"
  global      = false
  group       = "community"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:community"
  }
}

resource "grafana_role" "community_auditor" {
  name        = "community:auditor"
  description = "Time-boxed engagement-scoped read; per policy/auditor-scope.cedar"
  uid         = "community-auditor"
  global      = false
  group       = "community"
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:community"
  }
}
