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
resource "oci_artifacts_container_repository" "foundry_workspace_shell" {
  compartment_id = oci_identity_compartment.foundry.id
  display_name   = "oya-ops-workspace-shell"
  is_public      = false
  is_immutable   = false
  freeform_tags  = local.common_tags
}

resource "oci_artifacts_container_repository" "cloud_kms_adapter_oci" {
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
output "ocir_repo_foundry_workspace_shell" { value = oci_artifacts_container_repository.foundry_workspace_shell.display_name }
output "ocir_repo_cloud_kms_adapter_oci" { value = oci_artifacts_container_repository.cloud_kms_adapter_oci.display_name }
