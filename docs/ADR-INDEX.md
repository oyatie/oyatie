---
purpose: Generated ADR index and machine-readable mirror pointer for ADR freshness, numbering, owner, status, and supersession review.
doc_status: published
---

# Oyatie — ADR Index

> **Generated:** from [`decisions/`](decisions/) by `oya doc adr-index`. Do not hand-edit generated rows.
> **Authoritative:** `crew-adr-promotion` owns freshness per [DOC-CATALOG.md `doc.adr_index`](DOC-CATALOG.md).
> **Machine-readable mirror:** [`machine-readable/decisions.json`](machine-readable/decisions.json).

## At-a-glance

- **Total ADRs:** 16
- **Numbering:** contiguous ADR-0700..ADR-0715 (gap-free)
- **Next ADR number:** 0716
- **Status counts:** Accepted 10, Proposed 6
- **Legacy retirement:** see [`ADR-LEGACY-REGRESSION-MAPPING.md`](ADR-LEGACY-REGRESSION-MAPPING.md).

## Full table (one row per ADR, sorted by ADR number)

| ADR | Status | Title | Owner | File |
|---|---|---|---|---|
| ADR-0700 | Accepted | Live CI admission, build hermeticity, and runner substrate | council-architecture | [`ADR-0700-ci-admission-live-apex.md`](decisions/ADR-0700-ci-admission-live-apex.md) |
| ADR-0701 | Accepted | Live monorepo capability layout, faces, and reorg doctrine | council-architecture | [`ADR-0701-monorepo-capability-live-apex.md`](decisions/ADR-0701-monorepo-capability-live-apex.md) |
| ADR-0702 | Accepted | Live identity, tenancy, authz, secrets, and control-plane fail-closed posture | council-architecture | [`ADR-0702-identity-authz-live-apex.md`](decisions/ADR-0702-identity-authz-live-apex.md) |
| ADR-0703 | Accepted | Live CAS/cache policy (activation-gated RE remains fail-closed) | council-architecture | [`ADR-0703-cas-cache-live-apex.md`](decisions/ADR-0703-cas-cache-live-apex.md) |
| ADR-0704 | Accepted | Live Kubernetes Go→Rust port engine and owned-kernel interfaces | council-architecture | [`ADR-0704-k8s-port-live-apex.md`](decisions/ADR-0704-k8s-port-live-apex.md) |
| ADR-0705 | Accepted | Live product protocols, APIs, and communications plane | council-architecture | [`ADR-0705-product-protocol-live-apex.md`](decisions/ADR-0705-product-protocol-live-apex.md) |
| ADR-0706 | Accepted | Live observability, SLO, and progressive-delivery telemetry | council-architecture | [`ADR-0706-observability-live-apex.md`](decisions/ADR-0706-observability-live-apex.md) |
| ADR-0707 | Accepted | Live trust, safety, and resilience substrate doctrines | council-architecture | [`ADR-0707-trust-safety-live-apex.md`](decisions/ADR-0707-trust-safety-live-apex.md) |
| ADR-0708 | Accepted | Live platform foundations: cells, residency, workflow, plugins, search | council-architecture | [`ADR-0708-platform-foundations-live-apex.md`](decisions/ADR-0708-platform-foundations-live-apex.md) |
| ADR-0709 | Accepted | Live general architecture and remaining accepted doctrine | council-architecture | [`ADR-0709-general-live-apex.md`](decisions/ADR-0709-general-live-apex.md) |
| ADR-0710 | Proposed | Kubernetes admission substrate is the API server: VAP/CEL + PSA, no policy webhook | council-architecture | [`ADR-0710-kubernetes-admission-substrate-is-the-api-server.md`](decisions/ADR-0710-kubernetes-admission-substrate-is-the-api-server.md) |
| ADR-0711 | Proposed | Node kernel + pool matrix: Asterinas shared-kernel pools with co-selected stripped-Linux KVM SKU | council-architecture | [`ADR-0711-node-kernel-pool-matrix.md`](decisions/ADR-0711-node-kernel-pool-matrix.md) |
| ADR-0712 | Proposed | Owned node runtime — supervisor libraries, no manager daemon | council-architecture | [`ADR-0712-owned-node-runtime-no-manager-daemon.md`](decisions/ADR-0712-owned-node-runtime-no-manager-daemon.md) |
| ADR-0713 | Proposed | Runtime tier rename to isolation-property names with orthogonal placement axis | council-architecture | [`ADR-0713-isolation-property-runtime-tier-names.md`](decisions/ADR-0713-isolation-property-runtime-tier-names.md) |
| ADR-0714 | Proposed | ADR-0710 Accept/Reject is blocked on D-8 workload-boundary evidence | council-architecture | [`ADR-0714-adr-0710-accept-blocked-on-d8.md`](decisions/ADR-0714-adr-0710-accept-blocked-on-d8.md) |
| ADR-0715 | Proposed | OS-layer retirement — node supervisor as OS; apex noun amend proposal | council-architecture | [`ADR-0715-os-layer-retirement-supervisor-as-os.md`](decisions/ADR-0715-os-layer-retirement-supervisor-as-os.md) |

## Update protocol

- Per-event + monthly per `doc.adr_index` row in [`DOC-CATALOG.md`](DOC-CATALOG.md).
- New ADRs land via [`templates/adr-template.md`](templates/adr-template.md) and use the next available number (0716), unless an explicit reserved-number ADR is being filled.
- Per-ADR amendments preserve the original ADR number; the amended ADR cites its original date and links to the amending PR.
- Supersession is recorded in the per-ADR header and mirrored here on regeneration.

## Sources scanned

- `decisions/` directory listing — 16 ADR files (sorted ascending)
- [`machine-readable/decisions.json`](machine-readable/decisions.json) — generated machine mirror
- [`DOC-CATALOG.md`](DOC-CATALOG.md) — owner / cadence / dependent docs / validation checks
