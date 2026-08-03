// Cedar RBAC policy sketch for managed-K8s SLA observability.
// ADR-0376 / ADR-0007 / ADR-0183.
// Claim ceiling: reference guardrails for tenant-scoped SLA summary/evidence
// access. Runtime Cedar authorization is follow-on work; this file is not
// measured production evidence and does not authorize provider or Kubernetes
// actions by itself.

// Tenant viewers and admins may read their own cluster SLA summary only.
@id("sla-summary-read-tenant")
permit (
  principal is Workload,
  action == Action::"slaSummary:Read",
  resource is SlaSummary
)
when {
  principal.scopes.contains("sla:summary:read") &&
  principal.tenant_id == resource.tenant_id
};

// Tenant viewers and admins may read evidence handles for their own cluster only.
@id("sla-evidence-read-tenant")
permit (
  principal is Workload,
  action == Action::"slaEvidence:Read",
  resource is SlaEvidenceHandle
)
when {
  principal.scopes.contains("sla:evidence:read") &&
  principal.tenant_id == resource.tenant_id
};

// Reviewed collector identities may ingest normalized observations for their
// assigned tenant. They may not synthesize success for stale or missing samples.
@id("sla-observation-ingest-collector")
permit (
  principal is Workload,
  action == Action::"slaObservation:Ingest",
  resource is SlaObservation
)
when {
  principal.scopes.contains("sla:observation:ingest") &&
  principal.tenant_id == resource.tenant_id &&
  resource.source_type in ["control-plane-status", "prometheus-snapshot"]
};
