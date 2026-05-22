output "service_endpoint" {
  description = "cloud-billing service endpoint (gRPC-over-HTTP/3 per ADR-0253)"
  value       = "cloud-billing.${kubernetes_namespace.cloud_billing.metadata[0].name}.svc.cluster.local:50051"
}

output "observability_export" {
  description = "OpenTelemetry export endpoint"
  value       = var.otlp_endpoint
}

output "billing_meter_ids" {
  description = "Per-µservice meter unit identifiers consumed by cloud-billing"
  value = [
    "llm_input_tokens",
    "llm_output_tokens",
    "gpu_seconds",
    "workflow_executions",
    "gb_stored",
    "gb_egress",
    "api_calls",
    "vcpu_hour",
    "memory_gb_hour",
    "pod_minute",
    "invocation_count",
    "gb_seconds",
    "vector_search_queries",
    "requests"
  ]
}

output "iam_bindings" {
  description = "Cedar permits + cloud-iam role bindings"
  value = {
    permits = [
      "cap.cloud.billing.read_tenant_class",
      "cap.cloud.billing.convert_tenant",
      "cap.cloud.billing.mutate_billing_components",
      "cap.cloud.billing.emit_usage_event",
      "cap.cloud.billing.issue_invoice",
      "cap.cloud.billing.void_invoice",
      "cap.cloud.billing.issue_credit_memo",
      "cap.cloud.billing.purchase_reservation",
      "cap.cloud.billing.convert_reservation",
      "cap.cloud.billing.compute_settlement",
      "cap.cloud.billing.initiate_payout"
    ]
  }
}

output "state_backend_ref" {
  description = "Postgres logical-replication backing store"
  value       = "postgres://${helm_release.postgres_primary.name}.${kubernetes_namespace.cloud_billing.metadata[0].name}.svc.cluster.local:5432/cloud_billing"
}

output "module_attestation_ref" {
  description = "sigstore + cosign attestation reference per ADR-0039"
  value       = data.cosign_verify.module_attestation.image
}
