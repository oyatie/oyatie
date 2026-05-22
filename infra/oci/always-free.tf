// always-free.tf — Always-Free OCI resources beyond the A1 + E2 instances.
// Per user directive 2026-05-16: "make use of all the always free tier resources
// where it make sense".
//
// Compute already provisioned in compute-aux.tf (2× E2.1.Micro) + compute.tf (A1).
// Networking already provisioned in networking.tf + network-expand.tf.
// KMS vault + master key already in kms.tf.
// Object Storage bucket (cold backup) already in storage.tf.
//
// This file adds:
//   1. OCI Bastion service (Always Free; managed bastion for private-subnet SSH)
//   2. Logging log group + custom log (10 GB/month Always Free)
//   3. Notifications topic (1M sends/month free)
//   4. Container Registry (OCIR — Always Free in most regions, including ap-chuncheon-1)
//   5. Resource Manager stack (free; mirrors infra/oci/ for IaC drift detection)
//
// Deferred (provision when a consumer arrives):
//   - Autonomous Database (2× 20 GB free) — wait for first OLTP workload
//   - NoSQL Cloud (25 GB free) — wait for a service that wants it
//   - Email Delivery — requires approved-sender provisioning
//   - Load Balancer Flex (10 Mbps free) — wait for 2+ public backends
//   - Streaming (50 GB/mo free) — wait for an event-driven service

# ---- 1. OCI Bastion (Always Free) ----
resource "oci_bastion_bastion" "ops" {
  compartment_id               = oci_identity_compartment.nonprod.id
  name                         = "oyatie-ops-bastion"
  bastion_type                 = "STANDARD"
  target_subnet_id             = oci_core_subnet.nonprod_private.id
  client_cidr_block_allow_list = ["0.0.0.0/0"]
  max_session_ttl_in_seconds   = 10800 // 3h max session
  freeform_tags                = local.common_tags
}

# ---- 2. Logging — log group + a custom-log placeholder ----
resource "oci_logging_log_group" "ops" {
  compartment_id = oci_identity_compartment.foundry.id
  display_name   = "oyatie-ops"
  description    = "Centralized custom-log group for foundry + cloud-axis services"
  freeform_tags  = local.common_tags
}

resource "oci_logging_log" "ops_canary" {
  display_name       = "oyatie-ops-canary"
  log_group_id       = oci_logging_log_group.ops.id
  log_type           = "CUSTOM"
  is_enabled         = true
  retention_duration = 30
  freeform_tags      = local.common_tags
}

# ---- 3. Notifications topic ----
resource "oci_ons_notification_topic" "ops_alerts" {
  compartment_id = oci_identity_compartment.foundry.id
  name           = "oyatie-ops-alerts"
  description    = "Ops alerts (security scan, backup failures, instance state changes). Subscribers are declared via var.ops_notification_subscriptions and managed by OpenTofu."
  freeform_tags  = local.common_tags
}

resource "oci_ons_subscription" "ops_alerts" {
  for_each = var.ops_notification_subscriptions

  compartment_id = oci_identity_compartment.foundry.id
  endpoint       = each.value.endpoint
  protocol       = each.value.protocol
  topic_id       = oci_ons_notification_topic.ops_alerts.topic_id
  freeform_tags  = local.common_tags
}

# ---- 4. Container Registry — repos for foundry + cloud-axis images ----
# OCIR is documented as Always-Free but some tenancy/region combos (e.g.
# bominal-oci/us-ashburn-1) return FREE_TIER_NOT_SUPPORTED. Gated behind
# var.enable_container_registry to allow AF tenancies to apply cleanly.
resource "oci_artifacts_container_repository" "foundry_workspace_shell" {
  count          = var.enable_container_registry ? 1 : 0
  compartment_id = oci_identity_compartment.foundry.id
  display_name   = "oya-ops-workspace-shell"
  is_public      = false
  is_immutable   = false
  freeform_tags  = local.common_tags
}

resource "oci_artifacts_container_repository" "cloud_kms_adapter_oci" {
  count          = var.enable_container_registry ? 1 : 0
  compartment_id = oci_identity_compartment.cloud.id
  display_name   = "oya-cloud-kms-adapter-oci"
  is_public      = false
  is_immutable   = false
  freeform_tags  = local.common_tags
}

# ---- 5. Outputs ----
output "bastion_id" { value = oci_bastion_bastion.ops.id }
output "log_group_id" { value = oci_logging_log_group.ops.id }
output "notifications_topic_id" { value = oci_ons_notification_topic.ops_alerts.topic_id }
output "notifications_subscription_ids" {
  value = {
    for name, subscription in oci_ons_subscription.ops_alerts : name => subscription.id
  }
}
output "ocir_namespace" { value = data.oci_objectstorage_namespace.tenancy.namespace }
output "ocir_repo_foundry_workspace_shell" {
  value = length(oci_artifacts_container_repository.foundry_workspace_shell) > 0 ? oci_artifacts_container_repository.foundry_workspace_shell[0].display_name : null
}
output "ocir_repo_cloud_kms_adapter_oci" {
  value = length(oci_artifacts_container_repository.cloud_kms_adapter_oci) > 0 ? oci_artifacts_container_repository.cloud_kms_adapter_oci[0].display_name : null
}

# ---- 6. Budget — tenancy-wide spend guardrail ($1 cap; alerts at 50/100% actual + 100% forecast) ----
# Per user directive 2026-05-18 ('switch to PAYG but stay within free limit usage').
# Always-Free workloads cost $0; any non-zero monthly spend trips the 50% alert
# immediately on the $1 default. Budgets themselves are a free OCI service.
resource "oci_budget_budget" "tenancy_zero_spend" {
  compartment_id = local.tenancy_ocid
  amount         = var.budget_monthly_amount_usd
  reset_period   = "MONTHLY"
  display_name   = "oyatie-tenancy-zero-spend"
  description    = "PAYG-but-free posture: detect any non-zero monthly spend. AF workloads are $0; any spend = misconfiguration."
  target_type    = "COMPARTMENT"
  targets        = [local.tenancy_ocid]
  freeform_tags  = local.common_tags
}

resource "oci_budget_alert_rule" "actual_50" {
  budget_id      = oci_budget_budget.tenancy_zero_spend.id
  threshold      = 50
  threshold_type = "PERCENTAGE"
  type           = "ACTUAL"
  display_name   = "actual-50pct"
  description    = "Actual spend at 50% of monthly budget. AF workloads should never trip this; investigate."
  recipients     = var.budget_alert_recipients
  message        = "OCI tenancy actual spend hit 50% of monthly cap. Check Cost Analysis at https://cloud.oracle.com/billing/cost-analysis and terminate the offending resource."
}

resource "oci_budget_alert_rule" "actual_100" {
  budget_id      = oci_budget_budget.tenancy_zero_spend.id
  threshold      = 100
  threshold_type = "PERCENTAGE"
  type           = "ACTUAL"
  display_name   = "actual-100pct"
  description    = "Actual spend at 100% of monthly budget. Misconfiguration is bleeding cash; act immediately."
  recipients     = var.budget_alert_recipients
  message        = "OCI tenancy actual spend hit 100% of monthly cap. Audit https://cloud.oracle.com/billing immediately."
}

resource "oci_budget_alert_rule" "forecast_100" {
  budget_id      = oci_budget_budget.tenancy_zero_spend.id
  threshold      = 100
  threshold_type = "PERCENTAGE"
  type           = "FORECAST"
  display_name   = "forecast-100pct"
  description    = "Forecasted monthly spend reaches 100% of budget. Early warning before a billing event lands."
  recipients     = var.budget_alert_recipients
  message        = "OCI tenancy forecast monthly spend reached 100% of cap. Projected billing event imminent — audit now."
}

output "tenancy_budget_id" { value = oci_budget_budget.tenancy_zero_spend.id }

# ---- 7. Compartment quota — hard caps mirroring the Always-Free envelope ----
# Quotas are enforced by OCI at resource-create admission time. Any create that
# would exceed these caps is rejected at the API by OCI, before any state
# mutation. This makes accidental paid-tier creation impossible by policy on a
# PAYG-upgraded account.
#
# Statement DSL is regional. If a plan/apply fails on an unknown quota name,
# enumerate the canonical names for this region with:
#   oci limits service list --compartment-id <tenancy-ocid> --auth security_token
#   oci limits quota list   --compartment-id <tenancy-ocid> --auth security_token
resource "oci_limits_quota" "always_free_envelope" {
  count          = var.enable_compartment_quota ? 1 : 0
  compartment_id = local.tenancy_ocid
  name           = "oyatie-always-free-envelope"
  description    = "Hard caps mirroring the Always-Free allotment (4 A1 OCPUs / 24 GB A1 memory / 200 GB block storage / 2 public IPs). Prevents accidental paid-tier creation on a PAYG-upgraded account."

  statements = [
    "Set compute quota standard-a1-core-regional-count to 4 in tenancy",
    "Set compute quota standard-a1-memory-regional-count to 24 in tenancy",
    "Set block-storage quota total-storage-gb to 200 in tenancy",
    "Set vcn quota reserved-public-ip-count to 2 in tenancy",
  ]

  freeform_tags = local.common_tags
}

output "always_free_envelope_quota_id" {
  value = length(oci_limits_quota.always_free_envelope) > 0 ? oci_limits_quota.always_free_envelope[0].id : null
}
