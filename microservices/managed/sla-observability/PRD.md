# Managed K8s SLA Observability PRD

## Problem
Managed Kubernetes tenants need deterministic, tenant-scoped SLA summaries for control-plane availability, provisioning latency, and error-budget burn without coupling this bounded context to live Prometheus or Kubernetes clients.

## Scope
- Consume the settled `oya-managed-k8s-control-plane-host-api` status/provisioning seam.
- Accept test/in-memory status snapshots for local verification.
- Compute golden-signal style DTOs: current availability state, observed availability basis points, provisioning-latency SLO state, and error-budget burn.
- Fail closed for malformed or unknown clusters with typed errors.

## Non-goals
- No live Prometheus scraping.
- No direct Kubernetes/kube-rs dependency.
- No quota-policy decisions; quota surfaces remain owned by tenant-quota.
