# Managed K8s Cluster Lifecycle — PRD

## Purpose

`managed-k8s-cluster-lifecycle` is the dogfood-first admission and lifecycle orchestration surface for Oyatie-managed Kubernetes clusters. It accepts gateway-authenticated cluster lifecycle requests, checks tenant quota before backend actuation, and calls the managed-k8s control-plane host only when admission and quota policy allow it.

## Source authority and claim ceiling

- Source authority for this service lives under `cloud/managed-k8s-cluster-lifecycle/**`.
- ADR-0376 defines the managed-Kubernetes product split; this PRD scopes only the cluster-lifecycle service.
- Current shipped/local authority is a deterministic foundation for create admission (`POST /clusters`) plus `/healthz`; update, scale, delete, durable operation-ledger records, and reconciliation semantics are design/build targets for follow-on cards.
- This PRD does not claim live Kamaji, Talos, Cluster API, Kubernetes, cloud-provider, public SLA, billing, DPIA, external GA, production-readiness, or measured SLO evidence.

## Users and outcomes

- Tenant and platform dogfood callers can request a managed Kubernetes cluster through a gateway-injected tenant principal.
- Platform operators can rely on fail-closed admission semantics: quota deny/not-found/persistence errors do not invoke provisioning.
- Follow-on build work has a service-specific authority document for lifecycle behavior instead of copied tenant-quota requirements.

## In-scope behavior

- Validate cluster create request shape, desired tier, tenant principal, and resource request dimensions.
- Reject caller-supplied or mismatched tenant context; `x-oya-tenant-id` is trusted only when injected by the upstream gateway.
- Call `managed-k8s-tenant-quota` through `QuotaDecisionPort` before provisioning.
- Call `managed-k8s-control-plane-host` through `ControlPlaneProvisioning::provision` only after quota allow.
- Preserve honest deferred boundaries for update/scale/delete until operation-ledger and provider-port support exist.

## Acceptance criteria

1. Create admission validates `tenant_id`, `cluster_name`, desired tier, and requested node/vCPU/RAM dimensions before any backend actuation.
2. Missing tenant principal, tenant mismatch, malformed request, quota deny, quota not-found, and quota persistence/unavailable errors fail closed and do not call `ControlPlaneProvisioning`.
3. Quota allow maps the cluster request to a control-plane provisioning request and records/returns the control-plane handle produced by the current port.
4. Update, scale, and delete must not claim live provider success until a follow-on operation-ledger build adds deterministic state, idempotency, rollback/hold, and reconciliation evidence.
5. Observability/SLO language remains target or local deterministic evidence only until measured ingestion and SLO-gated promotion evidence exists.
6. No tenant-quota runtime implementation, quota RBAC administration surface, or billing/DPIA/public-GA claim is owned by this service PRD.

## Non-goals

- Implementing or changing tenant-quota runtime storage/RBAC APIs.
- Performing live Kubernetes, Cluster API, Kamaji, Talos, or cloud-provider actions.
- Claiming production readiness, external availability, public SLA, billing readiness, DPIA completion, or measured SLO compliance.
- Hand-editing generated artifacts.
