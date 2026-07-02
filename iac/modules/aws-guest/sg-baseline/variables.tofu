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

  validation {
    condition     = !contains(var.allowed_egress_cidrs, "0.0.0.0/0") && !contains(var.allowed_egress_cidrs, "::/0")
    error_message = "allowed_egress_cidrs must not include the open internet (0.0.0.0/0 or ::/0); sg-baseline is deny-by-default with an explicit egress allow-list."
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
