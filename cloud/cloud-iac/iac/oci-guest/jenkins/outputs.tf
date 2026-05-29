output "namespace" {
  description = "Jenkins namespace for this context."
  value       = var.namespace
}

output "controller_url" {
  description = "Jenkins web UI URL behind Istio Ambient ingress."
  value       = "https://${var.jenkins_host}"
}

output "kubernetes_manifests" {
  description = "Provider-agnostic Kubernetes manifests for Jenkins."
  value       = local.kubernetes_manifests
}

output "ingress_manifests" {
  description = "Gateway API and Istio authorization manifests for Jenkins ingress."
  value       = local.ingress_manifests
}

output "jcasc_files" {
  description = "Jenkins Configuration as Code files consumed by the controller ConfigMap."
  value = {
    jenkins = "${path.module}/values/jenkins.yaml"
    plugins = "${path.module}/values/plugins.yaml"
  }
}

output "cedar_policy_path" {
  description = "Cedar authorization policy path for Jenkins UI and API access."
  value       = "${path.module}/cedar/policies.cedar"
}
