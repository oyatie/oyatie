variable "context" {
  description = "Deployment context for this ArgoCD module instance."
  type        = string
  default     = "oyatie-cloud-provider"
}

variable "tenant_id" {
  description = "Tenant or platform partition allowed to use this ArgoCD instance."
  type        = string
  default     = "platform"
}

variable "tenant_class" {
  description = "Tenant isolation class surfaced into labels, Cedar, and audit evidence."
  type        = string
  default     = "contributor"
}

variable "cluster_id" {
  description = "Cluster identity that hosts ArgoCD."
  type        = string
  default     = "oyatie-cloud-provider"
}

variable "namespace" {
  description = "Kubernetes namespace for ArgoCD."
  type        = string
  default     = "oya-cd-argocd"
}

variable "argocd_host" {
  description = "External host routed by Gateway API / Istio Ambient ingress."
  type        = string
  default     = "argocd.oyatie-cloud-provider.oyatie.internal"
}

variable "argocd_image" {
  description = "ArgoCD image. ADR-0349 selects ArgoCD and rejects Flux CD."
  type        = string
  default     = "quay.io/argoproj/argocd:v2.11.7"
}

variable "argocd_image_digest" {
  description = "Cosign-verified ArgoCD image digest promoted by ADR-0181."
  type        = string
  nullable    = false
  validation {
    condition     = can(regex("^sha256:[0-9a-f]{64}$", var.argocd_image_digest)) && !can(regex("^sha256:0+$", var.argocd_image_digest))
    error_message = "argocd_image_digest must be a real non-zero sha256 digest promoted by ADR-0181."
  }
}

variable "repo_url" {
  description = "Git repository watched by ArgoCD Applications."
  type        = string
  default     = "https://git.oyatie.internal/oyatie/oyatie.git"
}

variable "target_revision" {
  description = "Git revision tracked by default Application templates."
  type        = string
  default     = "dev"
}

variable "repo_credentials_secret_name" {
  description = "Secret reference containing Git credentials. Secret values are not stored in this module."
  type        = string
  default     = "argocd-repo-credentials"
}

variable "openbao_mount_path" {
  description = "OpenBao mount path for ArgoCD runtime secrets. Secret values are not stored in this module."
  type        = string
  default     = "secret/data/cd/argocd"
}

variable "tags" {
  description = "Additional governance tags emitted into module outputs."
  type        = map(string)
  default     = {}
}
