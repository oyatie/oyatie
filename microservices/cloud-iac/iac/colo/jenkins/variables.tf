variable "context" {
  description = "Deployment context for this Jenkins module instance."
  type        = string
  default     = "colo"
}

variable "tenant_id" {
  description = "Tenant or platform partition allowed to use this Jenkins controller."
  type        = string
  default     = "platform"
}

variable "tenant_class" {
  description = "Tenant isolation class surfaced into labels, Cedar, and audit evidence."
  type        = string
  default     = "contributor"
}

variable "cluster_id" {
  description = "Cluster identity that hosts the Jenkins controller."
  type        = string
  default     = "colo"
}

variable "namespace" {
  description = "Kubernetes namespace for Jenkins."
  type        = string
  default     = "oya-ci-jenkins"
}

variable "jenkins_host" {
  description = "External host routed by Gateway API / Istio Ambient ingress."
  type        = string
  default     = "jenkins.colo.oyatie.internal"
}

variable "jenkins_image" {
  description = "Jenkins LTS controller image. ADR-0349 forbids Jenkins X / Tekton replacement."
  type        = string
  default     = "jenkins/jenkins:lts-jdk17"
}

variable "jenkins_image_digest" {
  description = "Cosign-verified Jenkins controller image digest promoted by ADR-0181."
  type        = string
  default     = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
}

variable "agent_image" {
  description = "Jenkins Kubernetes agent image used for Rust CI lanes."
  type        = string
  default     = "registry.oyatie.dev/ci/rust:stable"
}

variable "agent_image_digest" {
  description = "Cosign-verified Jenkins agent image digest promoted by ADR-0181."
  type        = string
  default     = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
}

variable "storage_class_name" {
  description = "Persistent volume storage class for Jenkins home."
  type        = string
  default     = "oyatie-retained"
}

variable "storage_size" {
  description = "Persistent volume size for Jenkins home."
  type        = string
  default     = "100Gi"
}

variable "replicas" {
  description = "Jenkins controller replicas. Jenkins remains singleton unless an HA controller strategy is approved."
  type        = number
  default     = 1
}

variable "oidc_issuer" {
  description = "OIDC issuer used by Jenkins web UI and API authentication."
  type        = string
  default     = "https://identity.oyatie.internal"
}

variable "openbao_mount_path" {
  description = "OpenBao mount path for Jenkins runtime secrets. Secret values are not stored in this module."
  type        = string
  default     = "secret/data/ci/jenkins"
}

variable "airgap_mirror_registry" {
  description = "Optional air-gap image mirror registry for offline contexts."
  type        = string
  default     = null
}

variable "tags" {
  description = "Additional governance tags emitted into module outputs."
  type        = map(string)
  default     = {}
}
