variable "tenancy_ocid" {
  description = "Root tenancy OCID. bitween → ocid1.tenancy.oc1..aaaaaaaafapguslxi54jdloww2rhtlyb7fyhf3tqgjm7xpfiwveuy43ltt3a; bominal-oci → ocid1.tenancy.oc1..aaaaaaaakdaslkhvri7nkvyvgenzlxqaoqt4gevibkegsnluovx3yr5b4lhq."
  type        = string
}

variable "oci_config_profile" {
  description = "Name of the profile in ~/.oci/config the provider should use. Each workspace's tfvars selects its corresponding profile so credentials, region, and tenancy line up. Default matches the legacy single-tenant setup."
  type        = string
  default     = "DEFAULT"
}

variable "region" {
  description = "OCI region (mirrors ~/.oci/config)."
  type        = string
  default     = "ap-chuncheon-1"
}

variable "ssh_authorized_keys" {
  description = "Public keys allowed to SSH the A1 instance(s)."
  type        = list(string)
}

variable "stage0_shape" {
  description = "Desired compute shape for Stage-0 application-shell VM. Defaults to VM.Standard.A2.Flex because A1.Flex Always Free can be out-of-capacity in ap-chuncheon-1; switch to A1.Flex by changing this variable and running OpenTofu through the root Makefile."
  type        = string
  default     = "VM.Standard.A2.Flex"
}

variable "ops_notification_subscriptions" {
  description = "OpenTofu-managed notification subscribers for oyatie-ops-alerts. Keys are stable subscriber ids; endpoints are stored in OpenTofu state, so do not put secrets here."
  type = map(object({
    protocol = string
    endpoint = string
  }))
  default = {}

  validation {
    condition = alltrue([
      for subscriber in values(var.ops_notification_subscriptions) :
      contains(["CUSTOM_HTTPS", "EMAIL", "ORACLE_FUNCTIONS", "PAGERDUTY", "SLACK", "SMS"], subscriber.protocol)
    ])
    error_message = "ops_notification_subscriptions.protocol must be one of CUSTOM_HTTPS, EMAIL, ORACLE_FUNCTIONS, PAGERDUTY, SLACK, SMS."
  }
}

variable "stage0_ocpus" {
  description = "OCPUs for stage-0 instance (Always Free up to 4 across tenancy)."
  type        = number
  default     = 1
}

variable "stage0_memory_gbs" {
  description = "Memory for stage-0 instance (Always Free up to 24 GB across tenancy)."
  type        = number
  default     = 6
}

variable "stage0_image_ocid" {
  description = "Boot image OCID (Canonical-Ubuntu-22.04-aarch64)."
  type        = string
}

variable "stage0_availability_domain" {
  description = "AD name for stage-0 instance."
  type        = string
  default     = "Iyyn:AP-CHUNCHEON-1-AD-1"
}

variable "create_stage0_a1" {
  description = "Create the A1.Flex primary stage-0 instance. Set false while A1 capacity is exhausted; the rest of the infra still applies clean. Re-enable when retrying."
  type        = bool
  default     = false
}

variable "always_free_mode" {
  description = "Restrict the module to Always-Free-eligible resources. When true, NAT Gateway, Service Gateway, and their private-subnet routes are omitted (AF tenancies have a hard limit of 0 for both). Set false for PAYG tenancies to enable the full topology."
  type        = bool
  default     = true
}

variable "create_stage0_aux_e2" {
  description = "Create the 2× E2.1.Micro auxiliary stage-0 instances (bastion + ops). Disable in tenancies that already host E2 instances (Always-Free caps E2.1.Micro at 2 per tenancy) or where the auxiliary fleet shape differs."
  type        = bool
  default     = true
}

# DNS labels are immutable in OCI — changing them forces destroy+recreate of
# the VCN and its dependents. These vars let each workspace match the live
# tenancy's existing labels, so Tofu can manage existing topologies without
# a destructive replacement.
variable "vcn_dns_label" {
  description = "DNS label for the nonprod VCN (immutable in OCI). Default matches the oyatie default. Override in tenancies whose VCN was created with a different label."
  type        = string
  default     = "oyatienpvcn"
}

variable "subnet_public_dns_label" {
  description = "DNS label for the nonprod public subnet (immutable). Override to match the live subnet's label."
  type        = string
  default     = "oyatienpsn"
}

variable "subnet_private_dns_label" {
  description = "DNS label for the nonprod private subnet (immutable). Override to match the live subnet's label."
  type        = string
  default     = "oyatienppvt"
}

variable "budget_monthly_amount_usd" {
  description = "Monthly tenancy-wide budget cap in USD. Default $1 trips the 50% alert on any non-zero spend — suitable for a PAYG-upgraded-but-still-free posture. Raise only if a paid workload is deliberately added."
  type        = number
  default     = 1
}

variable "budget_alert_recipients" {
  description = "Comma-separated email recipients for tenancy budget alerts. Empty disables email delivery (alerts still appear in the OCI console). Example: \"ops@example.com,sec@example.com\"."
  type        = string
  default     = ""
}

variable "enable_compartment_quota" {
  description = "Apply the Always-Free hard-cap quota policy. The current statements use the canonical service-family form (`compute quota standard-a1-core-count`) but OCI's quota DSL accepts only a curated subset of limit names; the exact accepted name varies per region and provider version. Leave disabled until verified against `oci limits quota create` in the target region, otherwise apply will fail at this resource."
  type        = bool
  default     = false
}

variable "enable_container_registry" {
  description = "Provision OCI Container Registry (OCIR) repos. OCIR is documented as Always-Free in most regions, but bominal-oci/us-ashburn-1 returns FREE_TIER_NOT_SUPPORTED for OCIR creates as of 2026-05-18. Leave disabled on AF tenancies; enable after PAYG flip or in tenancies confirmed to permit OCIR on AF."
  type        = bool
  default     = false
}

# ---- Stage-0 A1 instance shape parameters (per tenancy) ----
# hostname_label is immutable in OCI like dns_label, so each tenancy needs to
# match its live instance's existing label or it will force replacement.
variable "stage0_display_name" {
  description = "Display name for the stage-0 A1 instance. Match the live instance's name to avoid an unnecessary update on import."
  type        = string
  default     = "oyatie-stage0-a1"
}

variable "stage0_hostname_label" {
  description = "Hostname label for the stage-0 A1 primary VNIC. Immutable — match the live instance's label to avoid forcing replacement."
  type        = string
  default     = "oyatie-stage0"
}

variable "stage0_use_private_subnet" {
  description = "When true, place the stage-0 A1 instance in the private subnet with no public IP (suitable for Bastion-managed access). When false (default), use the public subnet with an assigned public IP."
  type        = bool
  default     = false
}

# ---- Aux E2 roles + image (per tenancy) ----
variable "stage0_aux_e2_image_ocid" {
  description = "Boot image OCID for the auxiliary E2.1.Micro x86 instances. Region-specific; defaults to the bitween Chuncheon image."
  type        = string
  default     = "ocid1.image.oc1.ap-chuncheon-1.aaaaaaaa7dt7pyhhltpw2lfpgqvfhwy3b3g6jbbzm3vh5ag3masvvd2bo6ia"
}

variable "stage0_aux_e2_roles" {
  description = "Map of aux E2.1.Micro roles to their per-instance attributes. Key becomes part of the resource address; each value specifies the OCI display_name, hostname_label (immutable, must match live for import), and optional per-role availability_domain (falls back to var.stage0_availability_domain when null). The map's keys + count are bounded by the Always-Free 2× E2.1.Micro envelope per tenancy."
  type = map(object({
    display_name        = string
    hostname_label      = string
    availability_domain = optional(string)
  }))
  default = {
    bastion = { display_name = "oyatie-stage0-e2-bastion", hostname_label = "oyatie-e2-bastion" }
    ops     = { display_name = "oyatie-stage0-e2-ops", hostname_label = "oyatie-e2-ops" }
  }
}
