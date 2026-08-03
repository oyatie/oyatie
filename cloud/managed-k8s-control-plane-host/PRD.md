# PRD — `oya-managed-k8s-control-plane-host`

**Status:** wave-3 design-spec + live adapter foundation (ADR-0376 lane).
Maturity is **not runtime GA**: hosted Kamaji dynamic-object create/read/delete
and dedicated Talos reference resolution exist behind the management-cluster
boundary; sandbox/live cluster proof, billing, public SLA, and external GA remain
separate gates.

**Owner:** council-architecture + ops-platform
**Authority:** ADR-0376 (managed-Kubernetes product surface), ADR-0375 (Talos +
CAPI + Argo CD substrate), ADR-0148 (Cilium L3/L4 + Istio Ambient L7), ADR-0092
(dependency seam), ADR-0083 (error-handling tier), ADR-0131/0132 (flat
single-concern layout).

## 1. Problem

ADR-0376 establishes Oyatie's managed-Kubernetes product as a **two-tier**
offering on the ADR-0375 substrate. The tenant picks the tier; the default is
hosted:

- **Hosted (DEFAULT)** — the tenant control plane runs as pods inside Oyatie's
  shared MANAGEMENT cluster via **Kamaji** (a `TenantControlPlane` on
  `kamaji.clastix.io/v1alpha1` + a per-tenant datastore). Dense, provisions in
  seconds, collapses the ~$73/tenant/month standing dedicated-control-plane tax.
- **Dedicated (PREMIUM)** — a full per-tenant **Talos spoke** (its own etcd + 3
  control-plane nodes), the ADR-0375 spoke promoted to a product SKU. For
  sovereign / air-gapped / strongest-isolation tenants.

This microservice owns the **control-plane-host concern**: provisioning,
observing, and tearing down a tenant control plane across BOTH tiers, plus the
management-cluster multi-tenancy hardening posture (a tenant must NEVER reach the
management cluster or a peer tenant's control plane).

It is ONE of the four flat single-concern microservices ADR-0376 names; the
others (`oya-managed-k8s-cluster-lifecycle`, `oya-managed-k8s-tenant-quota`,
`oya-managed-k8s-sla-observability`) land in their own lanes and CONSUME the
`ControlPlaneProvisioning` port this microservice owns.

## 2. Scope (this lane)

In-scope and BUILT:

- A pure kernel (`-kernel`): the `ControlPlaneTier` { `HostedKamaji`,
  `DedicatedTalosSpoke` } enum, the provisioning status state machine
  (`requested → datastore_bound (hosted) | media_formed (dedicated) →
  provisioning → endpoint_ready → active → draining → deleted`, plus `failed`),
  and the `DatastoreClass` { `EtcdPerTenant`, `PooledRelational` } taxonomy. No
  CRD-field knowledge; abstract status/tier/datastore only.
- The shared `ControlPlaneProvisioning` port + DTOs (`-api`): `provision(request)
  -> ControlPlaneRef`, `status(control_plane_ref)`, `teardown(control_plane_ref)`.
- A kube-rs CAPI adapter (`-adapter-capi`) that holds the management-cluster
  client + the Kamaji `TenantControlPlane` dynamic descriptor and performs
  hosted-tier DynamicObject create/read/delete. Dedicated tier resolves an opaque
  Talos/CAPI reference and does not create a hosted Kamaji object.
- A deterministic in-memory adapter (`-adapter-inmemory`) for tests + bring-up.
- A composition root (`-app`): axum admin/status API + fail-closed `[[bin]]` main.

Explicitly OUT of scope (deferred to named follow-ons, NOT designed here per
ADR-0376): billing/metering of managed clusters, the public SLA contract, the
DPIA for hosting tenant control planes, and external multi-tenant GA — all owned
by a future `oya-managed-k8s-commercial-ga` ADR.

## 3. Dogfood-first (ADR-0376)

The build target is the milestone where Oyatie provisions its OWN clusters as
**tenant-zero** (`oyatie-dogfood-tenancy`) through the same hosted/dedicated path
that will serve external tenants — **no internal bypass**. Tenant-zero is an
ordinary `ClusterRef.tenant_id`; there is no privileged identity in the port.

## 4. The `ControlPlaneProvisioning` port (the shared seam)

```rust
trait ControlPlaneProvisioning: Send + Sync {
    fn provision<'a>(&'a self, request: &'a ProvisionRequest)
        -> BoxFuture<'a, Result<ControlPlaneRef, ProvisioningError>>;
    fn status<'a>(&'a self, control_plane_ref: &'a ControlPlaneRef)
        -> BoxFuture<'a, Result<ControlPlaneStatusReport, ProvisioningError>>;
    fn teardown<'a>(&'a self, control_plane_ref: &'a ControlPlaneRef)
        -> BoxFuture<'a, Result<(), ProvisioningError>>;
}
```

`ProvisionRequest` = `{ cluster_ref, tier, datastore_class }`. The port is
object-safe so the composition root holds `Arc<dyn ControlPlaneProvisioning>` and
swaps adapters without a generic blast radius — this is the seam
`oya-managed-k8s-cluster-lifecycle` and `oya-managed-k8s-sla-observability` will
consume.

## 5. Operational boundary (hard invariant)

This service runs in (and talks ONLY to) Oyatie's MANAGEMENT cluster. It NEVER
runs tenant workloads and NEVER holds a tenant-cluster kubeconfig. The production
boot path is fail-closed: it reads the management kubeconfig path from
`$OYA_MGMT_KUBECONFIG` and refuses to start if absent (it never silently falls
back to the in-memory fake). See `operational-boundaries.md`.

## 6. Authorization (Cedar)

Only the platform / control-plane-operator principals may `provision` /
`teardown`; tenants are default-deny on the host concern (a tenant manages its
cluster as a resource via the cluster-lifecycle API, never the control-plane host
directly). See `cedar/policies.cedar`.

## 7. Live Kamaji/Talos reconciliation boundary

The `-adapter-capi` crate uses kube-rs dynamic APIs to create/read/delete the
hosted Kamaji `TenantControlPlane` GVK pinned in `registry/lts-pins.yaml`:
`kamaji.clastix.io/v1alpha1`, kind `TenantControlPlane`, plural
`tenantcontrolplanes`. It maps provider conditions into kernel lifecycle statuses
without synthesizing endpoints from names, refuses foreign objects that lack the
Oyatie owner label, and exposes a rollback switch that blocks new live provisions
while preserving status/teardown for existing objects. Dedicated Talos provisions
record an opaque ADR-0375 CAPI reference path only; this service never stores
tenant kubeconfig material.

## 8. Acceptance criteria

See `implementation-ready-acceptance-criteria.md`. Summary: kernel state machine
rejects illegal transitions; the in-memory adapter drives both tiers to `active`
and tears down to `deleted`; the live CAPI adapter builds the LTS-pinned Kamaji
dynamic object, maps provider status, guards teardown ownership, records the
dedicated Talos reference path, and honors the rollback switch; the production
`[[bin]]` fails closed without `$OYA_MGMT_KUBECONFIG`.
