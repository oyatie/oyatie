---
id: ADR-0158
status: Superseded
deciders: council-architecture, axis-tenancy, axis-cloud-k8s, axis-data-class, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-708]
amended_by: [ADR-0343]
related: [ADR-0009, ADR-0028, ADR-0049, ADR-0114, ADR-0121, ADR-0128, ADR-0142, ADR-0148, ADR-0157]
related_specs:
  - /specs/multi-region-disposition-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0158 — Per-µservice Multi-Region Disposition (active-active / active-passive / single-region), Sovereign Tenant Region Pinning, Global Control-Plane

## Status

Accepted (2026-05-18). Evolves ADR-0049 (cross-region residency) by declaring a uniform per-µservice multi-region disposition surface. Every µservice declares its multi-region disposition at PRD time; every tenant inherits a region-routing policy from its residency class and per-pack overlay; the global control plane (per Google Spanner / Stripe global routing pattern) routes tenant requests; the data plane is regional per the µservice's declared disposition.

## Context

ADR-0049 fixed *residency class* (StrictKr / KrWithUsFailover / Global / PerPack) for *tenant data*. It did NOT fix:

1. The **per-µservice disposition** ("can this µservice run active-active across regions, or is it inherently single-master?"). Some µservices (audit-chain — append-only Merkle log) are naturally active-active replicable. Some (workflow-engine — state machine with strong consistency requirements) are single-master per cell. Some (foundry — stateful GPU pool) are single-region pinned to where the GPU pool lives.
2. The **sovereign-tenant region-pin contract** ("a `pack-ksa` tenant lives only in the KSA cell, period"). ADR-0049 implied this; ADR-0158 makes it explicit and enforceable.
3. The **global control plane** ("when a tenant in `pack-kr` calls `api.oyatie.com`, which cell handles it?"). The hyperscaler precedent (Google Spanner's `Universe` topology, Stripe's tenant-tag routing, Cloudflare's anycast + steering) is uniform: control-plane state is global, data plane is regional.

Without an explicit per-µservice disposition declaration, every µservice is implicitly "single-region until proven otherwise". That penalizes µservices that ARE naturally active-active (audit-chain, ontology read replicas) and hides per-µservice DR posture. Without an explicit sovereign-pin contract, the api-gateway tier (ADR-0157) cannot enforce per-pack residency at edge.

## Decision

Every oyatie µservice declares one of three multi-region dispositions in its `manifest.json` under `multi_region_disposition`:

- **`active_active`** — multiple write-able regions; data converges via the µservice's chosen consistency model (CRDT per ADR-0142, or quorum-write per Spanner-class semantics, or append-only-merge for audit-chain).
- **`active_passive`** — one primary region; one or more warm-standby regions; failover RPO + RTO declared in the µservice's `multi-region.md`. The canonical use-case.
- **`single_region`** — pinned to one region; no cross-region presence; failover is intra-region only.

The disposition is a **first-class manifest field**. CI gate `cloud-ci/Rust gate packet multi-region-disposition` refuses merge if (a) the manifest declares a disposition not matching the µservice's actual deployment shape, or (b) a sovereign tenant routes to a cell outside its allowed-region set.

### Sovereign-tenant region-pin contract

Per-pack tenant residency overrides per-µservice disposition. A `pack-ksa` tenant in a `strict_ksa` residency class is pinned to the KSA cell regardless of the µservice disposition:

- If the µservice is `active_active` globally → for this tenant, only the KSA replica is used. Other regions reject the tenant.
- If the µservice is `active_passive` → only the KSA cell handles the tenant; cross-region failover is *intra-KSA* only.
- If the µservice is `single_region` and the single region is NOT KSA → that µservice is **unavailable** to KSA-pinned tenants. The PRD must declare this explicitly (e.g. "shorts µservice is single-region US; not offered to KSA-pinned tenants at GA").

The api-gateway tier (ADR-0157) enforces the region-pin at edge by rejecting requests whose JWT `tenant_id` resolves to a region the local cell is not allowed to serve.

### Global control plane

A dedicated **global control-plane** handles tenant-to-cell routing decisions:

- **Tenant registry** — global directory of (tenant_id → home_region + allowed_regions + residency_class). Source of truth in the tenancy µservice's global table, replicated via Patroni cross-region async replication (eventually consistent, ~5 sec lag), with anycast DNS pointing the api-gateway tier at the nearest replica.
- **Routing decision** — the api-gateway tier looks up the tenant in the local tenant-registry replica. If the request is for a tenant whose home is a different region, the gateway either (a) for `active_active` µservices, serves locally if allowed, (b) for `active_passive` / `single_region` µservices, returns 421 Misdirected Request with a Location header pointing the client at the correct regional endpoint.
- **No cross-region database transactions.** Data plane is strictly regional. Cross-region consistency is handled per-µservice: CRDT-merge (ADR-0142) for active-active, async-replication-warm-standby for active-passive, none for single-region.

### Standard topology per pack

| Pack | Primary region(s) | Disposition default | Sovereign-pin |
|---|---|---|---|
| `pack-kr` (KR strict) | KR-Seoul1 + KR-Chuncheon | `active_passive` (Seoul primary, Chuncheon warm-standby) | yes, intra-KR only |
| `pack-kr-fintech` | KR-Seoul1 + KR-Chuncheon | `active_passive` | yes, intra-KR only |
| `pack-eu` (EU sovereign) | EU-Frankfurt1 + EU-Dublin | `active_active` (Frankfurt + Dublin) | yes, intra-EU only |
| `pack-us` | US-Virginia + US-Oregon | `active_active` | no |
| `pack-jp` | JP-Tokyo + JP-Osaka | `active_passive` | yes, intra-JP only |
| `pack-ksa` (KSA sovereign) | KSA-Riyadh + KSA-Jeddah | `active_passive` | yes, intra-KSA only |
| `pack-uae` | UAE-Dubai | `single_region` (no UAE secondary at GA) | yes, intra-UAE only |
| `pack-global` (multi-tenant SaaS) | US-Virginia + EU-Frankfurt1 + APAC-Singapore | `active_active` | no |

### Per-µservice declarations (baseline at GA)

| µservice | Disposition | Rationale |
|---|---|---|
| `tenancy` | `active_active` (global tenant registry replicated) | Tenant lookup must be cheap from every cell. |
| `audit-chain` | `active_active` (append-only Merkle merge) | Append-only is naturally mergeable. |
| `cell` | `active_passive` (cell control-plane per region) | Cell-local control plane; cross-cell failover via global anycast. |
| `api-gateway` | `active_active` per-cell | Edge tier exists per-cell. |
| `cloud-k8s` | `active_passive` per-cell | Cluster control-plane is per-cell. |
| `cloud-secrets` | `single_region` per-cell | Secret material does not cross region. |
| `cloud-iac` | `active_passive` | IaC registry replicates; IaC apply is regional. |
| `governance` | `active_active` (policy is global) | Policy distributes to every cell. |
| `observability` | `active_active` per-region | Each region keeps its own observability stack; aggregator queries fan-out. |
| `ontology` | `active_active` (CRDT-backed per ADR-0142) | Read-heavy; CRDT-merge solves convergence. |
| `workflow-engine` | `active_passive` per-cell | State machine with per-cell strong consistency. |
| `workflow-studio` | `active_active` | Stateless control-plane µservice. |
| `foundry` | `single_region` per-cell | GPU pool pinned to region. |
| `mail` / `calendar` / `drive` / `notes` / `tasks` / `forms` | `active_passive` per-cell | User-data µservices; per-cell strong-consistency primary. |
| `connector` / `meet` / `messenger` | `active_active` per-cell | Real-time messaging; per-cell active; cross-cell federation. |
| `sites` / `shorts` / `social` / `community` | `active_active` | Read-heavy content µservices. |
| `recordings` / `sheets` / `slides` / `translate` / `anonymous` / `network` | `active_passive` | Standard SaaS tier. |
| `application` | `active_active` | Stateless app shell. |

Each µservice's `multi-region.md` MUST contain the disposition statement, the rationale, and (for `active_passive`) the RPO + RTO numbers.

## Alternatives considered

### Alternative A — Implicit single-region; opt-in to multi-region per µservice

- **Pros:** lowest startup cost; defer multi-region work until proven need.
- **Cons:** sovereign-tenant routing becomes per-µservice ad-hoc; ADR-0049 residency invariant cannot be enforced uniformly; the api-gateway tier (ADR-0157) cannot route without a per-µservice disposition table.
- **Rejected because:** ADR-0049 + ADR-0157 already mandate per-cell routing; implicit single-region violates these.

### Alternative B — Mandatory active-active everywhere

- **Pros:** maximum availability; simplest disposition matrix.
- **Cons:** active-active forces every µservice to adopt a CRDT or quorum-write model, which is infeasible for state-machine µservices (workflow-engine) and pinned-GPU µservices (foundry). Cross-region consistency overhead is per-µservice unbounded.
- **Rejected because:** active-active is correct only when the data shape supports it. Forcing it everywhere is engineering malpractice.

### Alternative C — Per-µservice disposition declaration with sovereign-pin overlay (this ADR)

- **Pros:** explicit; enforceable; matches hyperscaler precedent (AWS multi-region services declare their own disposition); sovereign-pin is a clean overlay on top of disposition; the api-gateway tier has a uniform routing table.
- **Cons:** every µservice's PRD must declare disposition (new doc requirement); CI lane needed to enforce.
- **Accepted.**

### Alternative D — Global Spanner-class strongly-consistent everywhere

- **Pros:** simplest model — every µservice gets global strong consistency for free.
- **Cons:** Spanner-class consistency adds ~50 ms write latency for cross-region quorum; not appropriate for low-latency UI tiers; Spanner is GCP-specific (or CockroachDB on-prem); ADR-0121 portability invariant disallows GCP-only.
- **Rejected because:** wrong cost model for most µservices; portability invariant.

## Consequences

### Positive

1. **Per-µservice disposition is an auditable artifact.** Every µservice declares its disposition in `manifest.json`; CI gate validates; sovereign-tenant routing becomes uniform across the fleet.
2. **Sovereign-tenant pinning structurally enforced.** A KSA-pinned tenant cannot accidentally land in a US cell because the api-gateway rejects mismatches at edge.
3. **DR posture explicit per µservice.** RPO + RTO declared in `multi-region.md`; SOC 2 A1.x + ISO 22301 (business continuity) evidence rolls up per µservice.
4. **Latency budget honest per disposition.** Active-active µservices declare their convergence cost; active-passive declare their failover cost; single-region declare their failover-is-intra-region. The cross-µservice latency budget (ADR-0145) honest by construction.
5. **Cross-region database transactions forbidden by design.** No accidental cross-region writes; cross-region consistency is per-µservice's explicit model (CRDT, async-replication, none).

### Negative

1. **More PRD content required.** Every µservice's `multi-region.md` must declare the disposition + rationale + RPO + RTO.
2. **Single-region µservices unavailable to some sovereign tenants.** E.g. `pack-ksa` tenants cannot use the `shorts` µservice at GA. PRD must declare; tenant must accept at onboarding.
3. **Active-active operational complexity.** Every active-active µservice owns the convergence story; CRDT µservices (ontology) carry ADR-0142 trait cost.
4. **Failover-drill cost per cell-pair.** Each active-passive pair (KR-Seoul1 ↔ KR-Chuncheon, KSA-Riyadh ↔ KSA-Jeddah) requires quarterly DR drill.

### Operational

1. CI lane `cloud-ci/Rust gate packet multi-region-disposition` reads every µservice's `manifest.json#multi_region_disposition` + `multi-region.md` and refuses merge on mismatch.
2. CI lane `cloud-ci/Rust gate packet sovereign-tenant-pin` reads the tenant-registry test fixtures and verifies the api-gateway route table rejects mismatched cells.
3. Each µservice updates `multi-region.md` with the disposition statement (companion to this ADR).
4. Global control plane lives in `microservices/tenancy/` (tenant-registry replication) + `microservices/api-gateway/` (routing decision).
5. RPO + RTO numbers feed the SLO-gated promotion ADR-0139.

## References

- Google Cloud Spanner — multi-region topology and external consistency (https://cloud.google.com/spanner/docs/instance-configurations).
- Stripe tenant routing — Stripe engineering blog "Online migrations at scale" (2017) describes the tenant-tag routing model.
- AWS Multi-Region Application Architecture — https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/welcome.html
- Azure Cosmos DB multi-region writes — https://learn.microsoft.com/azure/cosmos-db/distribute-data-globally
- Cloudflare Anycast routing — https://www.cloudflare.com/learning/cdn/glossary/anycast-network/
- ISO 22301 Business Continuity Management.
- SOC 2 CC A1.1 / A1.2 availability criteria.
- ADR-0009 — cell-architecture (per-cell, per-region).
- ADR-0028 — cloud microservice architecture.
- ADR-0049 — cross-region residency (sovereign pin overlay).
- ADR-0114 — canary + rollback (multi-region rollback).
- ADR-0121 — onprem K8s stack (portability invariant).
- ADR-0142 — CRDT portability trait (active-active convergence).
- ADR-0148 — Istio service mesh (multi-primary topology).
- ADR-0157 — api-gateway tier (north-south routing).
