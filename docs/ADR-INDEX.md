---
purpose: Live ADR index for the 18 apex files in decisions/.
doc_status: published
status: current
---

# Oyatie — ADR Index

**Current.** Hand-maintained index of the live apex set. Historical note: an older projection claimed generation by retired `oya doc adr-index`; that CLI is not merge authority.

**Authoritative protocol:** [`AGENTS.md`](AGENTS.md) + [`specs/markdown-retirement-policy.json`](../specs/markdown-retirement-policy.json). [`DOC-CATALOG.md`](DOC-CATALOG.md) is a tombstone.

**Machine-readable mirror:** [`machine-readable/decisions.json`](machine-readable/decisions.json) (producer-owned; do not hand-edit if generated).

## At-a-glance

- **Total live ADRs:** 18
- **Numbering:** contiguous ADR-0700..ADR-0717 (gap-free)
- **Next ADR number:** 0718
- **Status counts:** Accepted 13, Proposed 5
- **Legacy retirement:** [`ADR-LEGACY-REGRESSION-MAPPING.md`](ADR-LEGACY-REGRESSION-MAPPING.md) and [`adr-archive/`](adr-archive/).

Do not collapse citations onto ADR-0709. Read the topic apex (especially 0700, 0701, 0705, 0716).

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
| ADR-0711 | Accepted | Swarm Delivery Law — integration branch topology and command discipline | council-architecture | [`ADR-0711-swarm-delivery-law-integ-branch-topology.md`](decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md) |
| ADR-0712 | Proposed | Node kernel + pool matrix — Linux primary; Asterinas soak until A1 | council-architecture | [`ADR-0712-node-kernel-pool-matrix.md`](decisions/ADR-0712-node-kernel-pool-matrix.md) |
| ADR-0713 | Proposed | Node Substrate Architecture — PID1 stub + restartable supervisor; severable Accept | council-architecture | [`ADR-0713-node-substrate-architecture.md`](decisions/ADR-0713-node-substrate-architecture.md) |
| ADR-0714 | Proposed | Isolation-property RuntimeClass names with orthogonal placement axis | council-architecture | [`ADR-0714-isolation-property-runtime-tier-names.md`](decisions/ADR-0714-isolation-property-runtime-tier-names.md) |
| ADR-0715 | Proposed | F1 Admission package — ADR-0710 Accept/Reject blocked on D-8 | council-architecture | [`ADR-0715-f1-admission-adr-0710-d8-gate.md`](decisions/ADR-0715-f1-admission-adr-0710-d8-gate.md) |
| ADR-0716 | Accepted | Cargo is the CI merge path; buck2 is local hermeticity plus a weekly smoke | council-architecture | [`ADR-0716-cargo-merge-path-buck2-local-hermeticity.md`](decisions/ADR-0716-cargo-merge-path-buck2-local-hermeticity.md) |
| ADR-0717 | Accepted | Corpus-budget sprawl ratchet | council-architecture | [`ADR-0717-corpus-budget-shrink-only-ratchet.md`](decisions/ADR-0717-corpus-budget-shrink-only-ratchet.md) |

## Update protocol

- Live: [`AGENTS.md`](AGENTS.md) Done-Definition (same-wave load-bearing co-change).
- New ADRs land via [`templates/adr-template.md`](../templates/adr-template.md) as 0718+ unless filling a reserved number.
- Per-ADR amendments keep the original number and cite the amending PR.
- Supersession is recorded in the per-ADR header and mirrored here.
