output "namespace" {
  description = "ArgoCD namespace for this context."
  value       = var.namespace
}

output "server_url" {
  description = "ArgoCD web UI and API URL behind Istio Ambient ingress."
  value       = "https://${var.argocd_host}"
}

output "kubernetes_manifests" {
  description = "Provider-agnostic Kubernetes manifests for ArgoCD."
  value       = local.kubernetes_manifests
}

output "ingress_manifests" {
  description = "Gateway API and Istio authorization manifests for ArgoCD ingress."
  value       = local.ingress_manifests
}

output "application_template_path" {
  description = "Application CRD template path for per-cluster GitOps onboarding."
  value       = "${path.module}/apps/template.yaml"
}

output "cedar_policy_path" {
  description = "Cedar authorization policy path for ArgoCD UI, API, and sync access."
  value       = "${path.module}/cedar/policies.cedar"
}
