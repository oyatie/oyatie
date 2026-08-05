# Managed K8s Cluster Lifecycle — Threat Model

## ADR-0376 threat model

### Tenant principal spoofing
**Risk**: A caller supplies a tenant id that differs from the gateway-authenticated
tenant principal.
**Mitigation**: Cluster-lifecycle accepts tenant identity only from the gateway
principal and rejects missing, malformed, or mismatched tenant context before any
dependency call.

### Quota bypass
**Risk**: A tenant requests more clusters, nodes, vCPU, or RAM than their quota
allows and reaches provisioning anyway.
**Mitigation**: Cluster-lifecycle calls `managed-k8s-tenant-quota` through
`QuotaDecisionPort` before provisioning. Deny, not-found, and unavailable quota
results are fail-closed admission outcomes and do not invoke control-plane-host.

### Dependency fail-open
**Risk**: A quota dependency timeout or persistence error is treated as allow.
**Mitigation**: Quota dependency errors are mapped to hard admission failures.
Manual operator recovery must restore the quota dependency or retry later; it must
not bypass quota and call provisioning directly.

### Premature control-plane invocation
**Risk**: Cluster-lifecycle invokes `managed-k8s-control-plane-host` before request
validation and quota admission are complete.
**Mitigation**: The lifecycle port order is validation → quota decision →
control-plane provisioning. Tests and runbooks must preserve this order.

### Cross-tenant cluster handle exposure
**Risk**: A create response leaks another tenant's cluster handle or provisioning
state.
**Mitigation**: The provisioning request is constructed from the gateway tenant
principal and the validated cluster request after quota allow. Cluster-lifecycle
does not expose tenant-quota read/write/admin APIs or quota records.
