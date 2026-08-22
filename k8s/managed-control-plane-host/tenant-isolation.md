# Tenant isolation — `managed-k8s-control-plane-host`

**Authority:** ADR-0376, ADR-0009 (cells), ADR-0148 (mesh), ADR-0147/0338 (Kata +
Cloud Hypervisor for untrusted workloads). non_claim: the LIVE isolation
enforcement (NetworkPolicy, Kamaji datastore separation, ztunnel posture) lands
with `kamaji-provider-live-integration`; this lane fixes the isolation MODEL.

## Isolation model

The control-plane-host concern is the enforcement point for the central
multi-tenancy property: **a tenant reaches its own API server and nothing else.**

### Hosted tier (Kamaji)

- Each tenant control plane is a distinct `TenantControlPlane` in its OWN
  namespace inside the management cluster, with its OWN datastore
  (`DatastoreClass::EtcdPerTenant` = physical etcd separation;
  `PooledRelational` = per-tenant logical separation in a shared relational
  backend).
- One-way reach: the management cluster reconciles the tenant control plane; the
  tenant control plane must NEVER reach the management control plane / etcd or a
  peer tenant's control plane. Enforced (at live integration) by Cilium L3/L4
  NetworkPolicy deny + Istio Ambient ztunnel mTLS (ADR-0148).
- Tenant-untrusted workloads run in Kata + Cloud Hypervisor pools (ADR-0147/0338)
  — but that is the WORKER concern, downstream of this service.

### Dedicated tier (Talos spoke)

- Full physical isolation: the tenant gets its own etcd + control-plane nodes
  (ADR-0375 spoke). No shared substrate; the strongest isolation + sovereign
  story. Cross-tenant reach is impossible by construction (separate clusters).

## Authorization isolation

- Cedar default-deny + explicit `forbid(principal in Role::"tenant", ...)` on the
  host concern: a tenant principal can NEVER provision/observe/teardown a control
  plane directly. Tenants act only transitively via the cluster-lifecycle service
  identity (`cedar/policies.cedar`).
- The `ClusterRef` is the tenant-scoping key on every port call; the in-memory
  adapter keys all state on `(tier, tenant_id, cluster_name)` so no cross-tenant
  handle collision is possible.

## Cross-tenant verification

The cross-tenant-access-fuzz discipline (foundation-level) asserts that a token
for tenant B cannot act on tenant A's resources. This microservice's port keeps
`tenant_id` on every request/response so the (live) adapter and the consuming
cluster-lifecycle service can enforce the same property end-to-end.
