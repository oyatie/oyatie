---
id: ADR-0607
title: "Fail-closed Cedar authz on the managed-K8s control-plane facades (cluster-lifecycle / control-plane-host / tenant-quota)"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-28
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0243, ADR-0562, ADR-0566, ADR-0573]
amends: []
related: [ADR-0243, ADR-0376, ADR-0566, ADR-0573]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0607: Fail-closed Cedar authz on the managed-K8s control-plane facades

## Status

**Proposed - 2026-06-28 (authored for founder sign-off; door: one-way).**

## Context

GH #979 (parts 1+2): the managed-K8s control-plane facades trusted forgeable in-band
authority — an AUTH-005 (ADR-0573) instance on the cluster-admission and quota control plane.

- `cluster-lifecycle-app` compared an `x-oya-tenant-id` HEADER against the request body's
  tenant id (header==body, both caller-supplied) — a forged header obtained cross-tenant
  cluster admission.
- `tenant-quota-app` carried a Cedar policy that was authored but never consulted — quota
  mutation was effectively unauthorized.
- `control-plane-host-app` admin routes lacked a verified-principal authz seam.

A one-time fix does not prevent recurrence; the cross-tenant-trust class is closed in code here
and the surfaces are removed from the authz-coverage baseline (ADR-0566) so a regression
re-reddens the gate.

## Decision

Make all three facades fail-closed against a SERVER-VERIFIED principal and a consulted Cedar PDP,
mirroring the merged `tenant-quota-adapter-cedar` (ADR-0243, Cedar as the universal gate) and the
clean-arch ports/adapters layering (ADR-0131):

- A `VerifiedCaller` is bound from a constant-time bearer check; the `x-oya-tenant-id` header
  compare is deleted — identity is never caller-asserted. authn runs before body parse
  (`FromRequestParts`).
- A Cedar PDP `ensure_authorized(principal, action, tenant)` is consulted in EVERY handler,
  fail-closed: default-deny, a PDP fault (`catch_unwind`) maps to Refused → 403, an empty bearer
  refuses at boot.
- A new adapter crate, `k8s/adapters/cluster-lifecycle-adapter-cedar`, holds the cluster-lifecycle
  Cedar decision: two explicit permits only (tenant-admin `SameTenantAsPrincipal` + platform
  operator), Cedar default-deny otherwise, a defense-in-depth cross-tenant guard,
  `#![forbid(unsafe_code)]`. It reuses `iam-identity-workload-authz-cedar` and is mapped to the
  `k8s` capability (ADR-0562) under the top-level `k8s/OWNERS`.

The per-facade authz seam stays a module (`src/authz.rs`) in each facade crate (the registry
generates KEEP from `git ls-files × OWNERS`); the new adapter crate is wired into each facade via a
path dependency, and its catalog record is filed under `managed-k8s-cluster-lifecycle`.

## Governed surfaces

The exact tracked paths this decision introduces and governs (born-accounting justification per
ADR-0568; one verbatim repo-relative path per line):

```
k8s/adapters/cluster-lifecycle-adapter-cedar/BUCK
k8s/adapters/cluster-lifecycle-adapter-cedar/Cargo.toml
k8s/adapters/cluster-lifecycle-adapter-cedar/src/lib.rs
k8s/facade/cluster-lifecycle-app/src/authz.rs
k8s/facade/control-plane-host-app/src/authz.rs
k8s/facade/tenant-quota-app/src/authz.rs
registry/catalog/k8s-cluster-lifecycle-adapter-cedar.yaml
```

## Consequences

- The cross-tenant / header-trust admission class is structurally closed on the managed-K8s
  control plane; the three surfaces leave the authz-coverage baseline (pure shrink).
- The new Cedar adapter is owned (`k8s/OWNERS`), reachable (facade path-deps + BUCK), justified
  (this ADR), and catalog-registered — clearing total-accounting.
- Future managed-K8s control-plane facades follow the same `VerifiedCaller` + consulted-Cedar-PDP
  pattern; a new unauthenticated admission surface fails closed.
