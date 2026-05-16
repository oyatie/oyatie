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
  description = "Bootstrap compute shape for Stage-0 application-shell VM. Defaults to VM.Standard.A2.Flex because A1.Flex Always Free is out-of-capacity at launch in ap-chuncheon-1; resize to A1.Flex after the instance is RUNNING."
  type        = string
  default     = "VM.Standard.A2.Flex"
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

variable "cloudflare_domain" {
  description = "Apex domain that owns DNS / Cloudflare-managed subdomains. Matches infra/cloudflare/variables.tf's cloudflare_domain."
  type        = string
  default     = "oyatie.com"
}
