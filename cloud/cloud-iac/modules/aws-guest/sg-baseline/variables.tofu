variable "tenant_id" {
  type        = string
  description = "Tenant identifier carried into every sg-baseline label."

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{1,62}$", var.tenant_id))
    error_message = "tenant_id must be lowercase kebab-case and 2-63 characters."
  }
}

variable "tenant_class" {
  type        = string
  description = "Tenant class per ADR-0331/ADR-0339."

  validation {
    condition     = contains(["demo_trial", "paid"], var.tenant_class)
    error_message = "tenant_class must be demo_trial or paid."
  }
}

variable "cell_id" {
  type        = string
  description = "Cell identifier for ADR-0248 cellular placement labels."

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{1,62}$", var.cell_id))
    error_message = "cell_id must be lowercase kebab-case and 2-63 characters."
  }
}

variable "service_name" {
  type        = string
  description = "Microservice consuming the shared security-group baseline."

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{1,62}$", var.service_name))
    error_message = "service_name must be lowercase kebab-case and 2-63 characters."
  }
}

variable "compliance_pack" {
  type        = string
  description = "Compliance pack label attached to downstream resources."
  default     = "soc2"
}

variable "allowed_egress_cidrs" {
  type        = list(string)
  description = "Explicit egress allow-list; ingress remains deny-by-default."
  default     = ["10.0.0.0/8"]

  validation {
    condition     = length(var.allowed_egress_cidrs) > 0 && alltrue([for cidr in var.allowed_egress_cidrs : can(cidrnetmask(cidr))])
    error_message = "allowed_egress_cidrs must contain at least one valid CIDR."
  }
}

variable "allowed_egress_tcp_ports" {
  type        = list(number)
  description = "TCP ports allowed for canonical egress."
  default     = [443]

  validation {
    condition     = length(var.allowed_egress_tcp_ports) > 0 && alltrue([for port in var.allowed_egress_tcp_ports : port >= 1 && port <= 65535])
    error_message = "allowed_egress_tcp_ports must be TCP ports in 1..65535."
  }
}

variable "cosign_attestation_digest" {
  type        = string
  description = "sha256 digest of the repo-local cosign attestation metadata fixture."

  validation {
    condition     = can(regex("^sha256:[0-9a-f]{64}$", var.cosign_attestation_digest))
    error_message = "cosign_attestation_digest must be sha256:<64 lowercase hex chars>."
  }
}
