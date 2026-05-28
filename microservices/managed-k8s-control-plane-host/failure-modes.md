# Failure modes — `oya-managed-k8s-control-plane-host`

ADR-0083 Tier-3: every fallible path returns a typed error; the request path
never panics. The `ProvisioningError` enum is the single typed failure surface.

| Failure | Trigger | Typed surface | HTTP | Disposition |
|---------|---------|---------------|------|-------------|
| Malformed cluster ref | empty `tenant_id`/`cluster_name` | `ProvisioningError::InvalidClusterRef` | 400 | Fail-closed; reject before any backend touch. |
| Unknown tier / datastore slug | bad enum string in request body | `InvalidClusterRef` (parse fail) | 400 | Fail-closed; kernel `parse` returns `None`. |
| Control plane not found | `status`/`teardown` of an unknown handle | `ProvisioningError::NotFound` | 404 | Default-deny read; teardown is idempotent (unknown handle = no-op success, not 404, in the in-memory adapter). |
| Illegal lifecycle transition | adapter attempts an out-of-graph status move | `ProvisioningError::IllegalTransition` | 409 | Kernel `transition` rejects; the in-memory adapter can never persist an illegal status. |
| Management cluster unreachable | CAPI adapter cannot reach the mgmt API server | `ProvisioningError::Backend` | 502 | Fail-closed; no unbounded retry on the provision path (circuit-breaker posture). |
| Live reconcile not built | CAPI adapter `provision`/`status`/`teardown` | `ProvisioningError::Unimplemented(KamajiProviderLiveIntegration)` | 501 | HONEST-DEFERRED; never a fake success. |
| Boot without mgmt kubeconfig | `$OYA_MGMT_KUBECONFIG` absent/empty | `BootError::MissingMgmtKubeconfig` | (process exits non-zero) | Fail-closed; never falls back to in-memory in production. |
| Mutex poisoned (in-memory adapter) | a panic while holding the records lock (test only) | `ProvisioningError::Backend` | 502 | Defensive; the in-memory adapter does no panicking work under the lock. |

## Degradation

- The hosted tier concentrates blast radius in the management cluster (ADR-0376):
  a management-cluster outage degrades ALL hosted control planes. Mitigation:
  management-cluster HA (hard prerequisite) + the dedicated tier as the
  no-shared-substrate escape hatch.
- A single tenant's control plane failure is isolated to that tenant
  (per-tenant `TenantControlPlane` + datastore); it does not cascade to peers.
