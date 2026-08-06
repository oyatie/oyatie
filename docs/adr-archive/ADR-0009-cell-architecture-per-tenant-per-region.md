---
id: ADR-0009
status: Superseded
superseded_by: [ADR-0700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0009: Cell architecture — per-tenant per-region blast-radius cells with cell-routing primitives at edge / mesh / store / event

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `cloud`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0010

---

## Context

The PRD declares horizontal-scale-end-to-end and cell-isolation-evidence as foundation invariants. Without explicit cell architecture, the flat-catalog cohesion claim degrades in two predictable ways: (a) a single tenant's runaway query, audit emission, or capability invocation starves other tenants in the same shared compute pool; (b) regulator-required isolation evidence (KR CSAP, K-ISMS-P; EU GAIA-X; US FedRAMP; SOC 2 CC7) cannot be produced because the deployment topology has no isolation primitive smaller than "region." LEDG-010 captures the prior single-cluster OCI posture as a contradiction with the AWS-class cloud claim; this ADR closes it.

The cell concept is well established in industry (AWS cell-based architecture, Google's per-locale shards, Azure scale units) and provides the right size — large enough for operational efficiency, small enough that a noisy-neighbor incident is bounded to a known blast radius. Oyatie's flat-catalog posture intensifies the requirement: a cell needs to absorb cross-microservice traffic for the tenants it hosts (workflow + connect + capability + cloud + search + ads) and must isolate audit-chain shards (ADR-0003), eventing partitions (ADR-0005), Ontology row-level boundaries (ADR-0006), and HSM partitions (KCminimum-shippable-tier, FIPS 140-3) cleanly.

---

## Decision

We adopt **cells as the primary blast-radius isolation primitive**, sized per-tenant per-region with five cell tiers, cell-routing primitives at edge / mesh / store / event layers, and per-cell HSM partitions. Cell-isolation evidence is collected quarterly per regulatory pack.

### Cell sizing tiers

| Tier | Reads as | Tenant count | Use case |
|---|---|---|---|
| `Dedicated` | One tenant per cell | 1 | Sovereign / large enterprise; per-pack regulator demand (KR-Government, KR-Healthcare-Tier-1) |
| `Shared-small` | Up to ~10 tenants | ≤ 10 | Mid-market; per-tenant blast-radius bound |
| `Shared-medium` | Up to ~100 tenants | ≤ 100 | SMB / startup tier |
| `Shared-large` | Up to ~1000 tenants | ≤ 1000 | Free / prosumer / Workspace-Personal |
| `Foundry-runtime` | Foundry agent execution; per-agent worktree isolation | per-agent | Agent invocation cells (ADR-0007 runtime) |
| `Public-corpus` | Search public-web ingestion; no tenant data | — | search microservice public corpus only |

Tier promotion (e.g. `Shared-small` → `Dedicated`) is a council-approved migration with an evidence record; downgrade is forbidden mid-tenancy (would cross blast-radius posture).

### Cell entity

```rust
// crates/oya-tenancy-cell-kernel
pub struct CellId(pub uuid::Uuid);

pub struct Cell {
    pub id: CellId,
    pub region: RegionCode,           // KR-Seoul1, JP-Tokyo, US-IAD, EU-FRA, ...
    pub az: AvailabilityZone,         // intra-region failure domain
    pub tier: CellTier,
    pub tenants: BTreeSet<TenantId>,
    pub hsm_partition: HsmPartitionRef,  // per-cell KCminimum-shippable-tier / FIPS 140-3 partition
    pub broker_pool: BrokerPoolRef,      // per-cell or per-region pool with cell-keyed partitions (ADR-0005)
    pub audit_shard: AuditShardRef,      // ADR-0003
    pub ontology_shard: OntologyShardRef,  // ADR-0006 Citus shard
    pub plane_affinity: BTreeSet<Plane>, // ADR-0004; data-plane is cell-local, control + analytics are global per region
    pub residency: ResidencyClass,
}
```

### Cell-routing primitives (four layers)

1. **Edge routing.** The edge gateway (Envoy per ADR-0013 ancestry) reads the request's `Tenant-Id` claim from the JWT, looks up the tenant's cell binding, and steers the request to the cell's regional ingress. Cross-region routing requires `residency.cross_region_replicated == true` AND a per-call audit record.
2. **Mesh routing.** Istio Ambient (per ADR-0044 ancestry) carries the `cell_id` as a request header; sidecars route across services within the cell and reject cross-cell calls unless explicitly declared in the catalog as a `cross_cell_contract`.
3. **Store routing.** Ontology (ADR-0006) reads the `cell_id` from the session context and binds Postgres RLS + Citus shard to the cell. Per-cell HSM partition unwraps DEKs only for the cell's tenants.
4. **Event routing.** Eventing backbone (ADR-0005) partition key is `(tenant_shard, cell_id)`. Per-cell consumers subscribe with cell-prefixed group ids; cross-cell event consumption is explicit catalog declaration.

### Per-cell HSM partition

Every cell has its own KCminimum-shippable-tier-validated (KR), FIPS 140-3 (US/EU), or per-region equivalent HSM partition. Per-record DEK lives in the cell's HSM; KEK is per-cell. DSR-driven shred destroys the DEK in the cell's HSM, fulfilling cryptographic-erasure requirements (ADR-0008 §8). HSM lead time is 6–9 months for KR CSAP / KCminimum-shippable-tier; cell architecture must factor this into capacity planning.

### Cell-isolation evidence (quarterly)

Per regulatory pack, per cell, per quarter:

- **Network isolation evidence** — VPC + SG + service-mesh policies prove no cross-cell ingress / egress beyond declared contracts.
- **Storage isolation evidence** — RLS coverage report; per-cell Citus shard residency proof.
- **Crypto isolation evidence** — per-cell HSM partition audit; DEK shred test against synthetic record.
- **Compute isolation evidence** — per-cell scheduler quotas; noisy-neighbor injection test demonstrates p99 within bound.
- **Audit isolation evidence** — chain shard verification (ADR-0003).

Evidence emits to the trust portal per ADR-0003.

### Boundary

- Applies to: every tenant data plane, every Foundry agent invocation, every cross-microservice call carrying tenant scope.
- Does not apply to: control-plane management (region-global by design per ADR-0004); analytics-plane projections (region-global with per-cell-tagged rows); public-corpus search ingestion (its own `Public-corpus` tier).

### CELL-002 spec-only validation registration

CELL-002 registers a spec-only service-plan and fixture contract for the six-input cell-promotion gate, the ADR-0351 cell-lifecycle/cell-rebalancer split, sharding-automation manifest checks, and the rollback audit-row shape. These paths are accounting/validation surfaces only and do not authorize runtime cell routing, tenant migration, autosharding, auto-rebalance, audit-chain writes, provider/Kubernetes calls, or live evidence collection. Validation is owned by the cloud-ci Rust gate `//ci/facade/contract-slice-conformance`:

- `specs/cell-002-promotion-automation-contract.json`
- `specs/fixtures/cell-002-promotion-automation/rollback-audit-row.json`

### CELL-001R spec-only validation registration

CELL-001R registers a spec-only manifest contract and cloud-ci validation surface for this ADR's cell tier and quarterly evidence concepts. These paths are accounting/validation surfaces only and do not authorize runtime cell routing, provider APIs, Kubernetes/Argo calls, tenant migration, autosharding, auto-rebalance, failover, or live evidence collection:

- `specs/cell-topology-manifest-contract.json`
- `specs/fixtures/cell-topology-manifest/tenancy-kr-strict.json`
- `ci/facade/topology-manifest-contract/BUCK`
- `ci/facade/topology-manifest-contract/Cargo.toml`
- `ci/facade/topology-manifest-contract/src/lib.rs`
- `ci/facade/topology-manifest-contract/tests/cell_topology_manifest_contract.rs`

---

## Consequences

### Positive

- Closes LEDG-010 (single-cluster posture) at the architectural level.
- Per-tenant blast-radius bound ≤ cell tier capacity; noisy-neighbor incidents do not cross cell boundary.
- Per-cell HSM partition + DEK shred cleanly satisfies cryptographic-erasure requirements across PIPA / GDPR / HIPAA / PCI.
- Cell-isolation evidence collected continuously is auditor-defensible without bespoke per-audit work.
- Cells map cleanly to Kafka partition keys (ADR-0005), Citus shards (ADR-0006), audit shards (ADR-0003), and STS scope (ADR-0002).

### Negative

- Per-cell capacity planning is a real ops task — under-provisioned cells stall; over-provisioned cells waste money. Mitigation: per-cell capacity-planning runbook + auto-scaling within tier cap.
- Per-cell HSM partition cost is non-trivial; `Dedicated` tier amortizes badly for low-volume sovereign tenants. Mitigation: per-tenant HSM partition pricing transparency; tier-based recommendations.
- Cross-cell migration (tier promotion) is a heavy operation; require an explicit migration runbook + customer comms.

### Operational

- On-call: per-cell SLO + per-cell-tier rollup; `EVT-CELL-CAPACITY-EXCEEDED` and `EVT-CELL-TIER-PROMOTION-DUE` alerts.
- Runbooks: `runbooks/cell-provision.md`, `runbooks/cell-tier-promotion.md`, `runbooks/cell-isolation-evidence-quarterly.md`, `runbooks/per-cell-hsm-rotation.md`, `runbooks/cell-failover-intra-region.md`.
- CI: `oya-governance-cell-routing` (every data-plane crate declares cell affinity), `oya-governance-cross-cell-call` (any cross-cell call has an explicit catalog contract).
- Capacity: per-cell-tier dashboards in the FinOps surface (per ADR-0019 cadence).

---

## Alternatives considered

### Alternative A — Region-only isolation (no cells)

- **Pros:** simpler.
- **Cons:** noisy-neighbor blast radius = region; LEDG-010.
- **Rejected because:** sovereignty + PRD §6 horizontal-scale invariant.

### Alternative B — Per-tenant dedicated infra everywhere

- **Pros:** maximal isolation.
- **Cons:** unit economics destroy SMB / Workspace-Personal pricing; HSM cost alone forbids it.
- **Rejected because:** financial.

### Alternative C — Compute-only cells (storage shared per region)

- **Pros:** simpler storage tier.
- **Cons:** RLS-only isolation has demonstrated regressions (LEDG-009); per-cell HSM partition eliminates the entire class.
- **Rejected because:** crypto isolation evidence is a regulator-demanded artifact.

---

## Open questions

1. **Q1.** Public-corpus-tier cell — does it need an HSM at all (no tenant data)? Default: NO HSM; per-record signing only. → owner: `axis-search`.
2. **Q2.** Foundry-runtime-cell sizing — one cell per tenant or one cell per agent run? Default: one cell per tenant; per-run worktree isolation inside. → ADR-0007.
3. **Q3.** Cross-cell read-only projection (e.g. cross-cell search) — does this require explicit catalog declaration? Default: YES; treated as cross-microservice-contract. → ADR-0011.
4. **Q4.** Cell-tier downgrade path — strictly forbidden, or council-approved with evidence? Default: forbidden mid-tenancy; only via tenant offboarding + re-onboarding to lower tier. → owner: `council-architecture`.
5. **Q5.** Per-region cell cap before sub-regional split — what's the practical cap on cell count per region before mesh routing becomes lossy? Default: 5000 cells per region; revisit. → owner: `cloud`.

---

## References

- `docs/DESIGN.md` §9 (horizontal scalability primitives — cell routing)
- `docs/PRD.md` §6 constraint 3 (tenancy isolation under formal proof), constraint 6 (horizontal scaling end-to-end)
- `docs/COMPLIANCE-MATRIX.md` (KR CSAP, K-ISMS-P, KCminimum-shippable-tier HSM; EU GAIA-X; US FedRAMP; SOC 2 CC7)
- `docs/CONTRADICTION-LEDGER.md` LEDG-010 (single-cluster posture)
- `docs/GLOSSARY.md` §1 ("Cell architecture", "Shuffle sharding") — AWS canon
- ADR-0001 (cohesion), ADR-0002 (Tenant.region_binding), ADR-0003 (audit shards), ADR-0005 (per-cell partition keys), ADR-0006 (Citus shard), ADR-0010 (regional pack residency contract)
