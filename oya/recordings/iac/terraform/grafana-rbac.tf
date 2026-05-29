terraform {
  required_providers {
    grafana = {
      source  = "grafana/grafana"
      version = "~> 3.0"
    }
  }
}

resource "grafana_folder" "recordings" {
  title = "recordings"
}

resource "grafana_role" "recordings_operator" {
  name        = "recordings-operator"
  description = "axis-recordings SRE on-call"
  permissions {
    action = "folders:read"
    scope  = "folders:uid:${grafana_folder.recordings.uid}"
  }
  permissions {
    action = "dashboards:read"
    scope  = "folders:uid:${grafana_folder.recordings.uid}"
  }
}

resource "grafana_role" "recordings_compliance_auditor" {
  name        = "recordings-compliance-auditor"
  description = "ops-compliance + external auditors read-only access to retention + legal-hold dashboards"
  permissions {
    action = "dashboards:read"
    scope  = "dashboards:uid:rec-retention-and-legal-hold"
  }
}

# Per-pack data source bindings (one per pack)
locals {
  packs = ["pack-kr", "pack-eu", "pack-us", "pack-us-healthcare", "pack-us-financial",
           "pack-jp", "pack-sg", "pack-au", "pack-in", "pack-br", "pack-ae", "pack-ksa"]
}

resource "grafana_data_source" "mimir_per_pack" {
  for_each = toset(local.packs)
  name     = "mimir-${each.value}"
  type     = "prometheus"
  url      = "https://mimir.${each.value}.internal/prometheus"
  json_data_encoded = jsonencode({
    httpHeaderName1 = "X-Scope-OrgID"
  })
  secure_json_data_encoded = jsonencode({
    httpHeaderValue1 = "$${openbao:secret/observability/${each.value}/mimir-token}"
  })
}
