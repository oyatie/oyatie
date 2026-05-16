variable "tenancy_ocid" {
  description = "Root tenancy OCID (bitween)."
  type        = string
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
