# Managed K8s SLA Observability — PRD

## Purpose

Tenant-scoped SLA observation for managed Kubernetes control planes. The service
normalizes control-plane status snapshots, computes deterministic availability,
provisioning-latency, and error-budget summaries, and exposes the narrow read
and evidence contracts needed by ADR-0376 follow-on lanes.

## Source Authority and Claim Ceiling

- Source home: `cloud/managed-k8s-sla-observability/**`.
- Deterministic authority today: pure kernel summaries, the
  `SlaObservabilityPort`, in-memory adapter, and app-level ingestion from the
  `ControlPlaneProvisioning::status` seam.
- Target-only authority today: live Kubernetes/Prometheus collectors, measured
  production SLO evidence, tenant-visible public SLA, and production-readiness
  claims. Those require follow-on Build/Review evidence before being treated as
  implemented.
- Admission/plan gating signals from sibling services can be referenced as
  context, but this service does not implement those sibling services.

## Acceptance Criteria

- Tenant cluster reads return deterministic SLA summaries for an authenticated
  `(tenant_id, cluster_name)` scope and fail closed for cross-tenant access.
- Availability uses `healthy_status_samples / total_status_samples` from
  normalized SLA observation snapshots and aligns with
  `cloud/managed-k8s-sla-observability/slos/managed-cluster-availability.openslo.yaml`.
- Provisioning latency uses `provisioning_latency_millis` from first accepted
  provision request to first durable active/serving proof and aligns with
  `cloud/managed-k8s-sla-observability/slos/provisioning-latency.openslo.yaml`.
- Missing, stale, or disagreeing live evidence never synthesizes a green SLA
  summary; it produces a no-data/degraded/hold outcome for downstream policy.
- Placeholder REST/event/gRPC contracts remain honest non-claims while naming
  future read, evidence, ingestion, stale-evidence, and burn-rate verdict
  surfaces for the implementation lane.
