variable "cell_id" {
  type        = string
  description = "Cell identifier per ADR-0248 cellular architecture"
}

variable "strimzi_version" {
  type    = string
  default = "0.42.0"
}

variable "postgres_chart_version" {
  type    = string
  default = "13.4.0"
}

variable "app_chart_version" {
  type        = string
  description = "cloud-billing Helm chart version"
}

variable "app_image_tag" {
  type        = string
  description = "cloud-billing container image tag"
}

variable "audit_chain_endpoint" {
  type        = string
  description = "audit-chain gRPC endpoint"
}

variable "cloud_kms_endpoint" {
  type        = string
  description = "cloud-kms gRPC endpoint"
}

variable "cloud_billing_tax_endpoint" {
  type        = string
  description = "cloud-billing-tax gRPC endpoint"
}

variable "payments_endpoint" {
  type        = string
  description = "payments gRPC endpoint"
}

variable "otlp_endpoint" {
  type        = string
  description = "OpenTelemetry OTLP endpoint"
}
