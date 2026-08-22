# Threat model — `managed-k8s-control-plane-host`

**Authority:** ADR-0376. Maturity: design-spec (the live reconcile is
honest-deferred; the threat model frames the surface the live integration must
satisfy).

## Core risk (THE risk)

**Tenant escape from a hosted control plane to the management cluster or a
sibling tenant's control plane.** The hosted (Kamaji) tier runs tenant control
planes as pods inside Oyatie's shared MANAGEMENT cluster. A tenant must reach its
own API server and NOTHING ELSE — not the management cluster's control plane, not
the management cluster's etcd, and not any peer tenant's `TenantControlPlane` or
datastore. This is the deciding security property of the hosted-tier economic
model and the central threat this microservice exists to contain.

## Trust boundaries

1. **Tenant ↔ tenant control plane** — the tenant's only sanctioned reach. The
   hosted control plane's API server is the boundary; the tenant never sees the
   management-cluster substrate beneath it.
2. **Tenant control plane ↔ management cluster** — ONE-WAY: the management cluster
   reconciles the tenant control plane; the tenant control plane must never reach
   back into the management control plane / etcd. Kamaji's per-tenant datastore +
   namespace isolation + the mesh layering (ADR-0148) enforce this; the live
   integration must verify it (NetworkPolicy / Cilium L3-L4 deny + ztunnel).
3. **Operator ↔ this service** — only platform / control-plane-operator
   principals may provision/teardown (Cedar default-deny for tenants;
   `cedar/policies.cedar`).

## Threats + mitigations

| # | Threat | Mitigation |
|---|--------|------------|
| T1 | Tenant pod in a hosted control plane reaches the management API server / etcd | Per-tenant namespace + datastore isolation (Kamaji); Cilium L3/L4 deny + Istio Ambient ztunnel mTLS (ADR-0148); **live-integration must add an explicit NetworkPolicy fitness check.** |
| T2 | Tenant reaches a SIBLING tenant's control plane or datastore | Per-tenant `DatastoreClass::EtcdPerTenant` gives physical datastore separation; pooled-relational uses per-tenant logical separation enforced by Kamaji; cross-tenant deny verified by the cross-tenant-access-fuzz discipline. |
| T3 | A tenant principal provisions/tears down a control plane directly | Cedar `forbid(principal in Role::"tenant", ...)` — belt-and-suspenders over default-deny; tenants act only transitively via the cluster-lifecycle service identity. |
| T4 | This service is pointed at a TENANT cluster kubeconfig (boundary violation) | Operational boundary: the service reads ONLY the management kubeconfig (`$OYATIE_MGMT_KUBECONFIG`), fail-closed; it never holds a tenant-cluster kubeconfig (`operational-boundaries.md`). |
| T5 | A provision/teardown happens without an audit trail | Cedar `forbid` when `context.audit_chain_emit != true`; seal events on every state-changing decision (`audit-evidence-emission.md`). |
| T6 | Silent fake success masks an unbuilt reconcile (supply-chain/trust) | Honest-claims: the CAPI adapter returns a typed `Unimplemented` (HTTP 501), never `Ok(...)`; tracked in placeholder-debt. |
| T7 | Compromised management cluster = total blast radius (hosted-tier concentration) | ADR-0376 known consequence: management-cluster HA + hardening is a hard prerequisite for hosted-tier density; dedicated tier remains the sovereign escape hatch with no shared substrate. |

## Residual / deferred

The LIVE enforcement of T1/T2 (NetworkPolicy, Kamaji datastore isolation config,
ztunnel posture) lands with `kamaji-provider-live-integration`. Until then the
adapter performs NO reconcile (no live attack surface exists yet); the in-memory
adapter has no network surface.
