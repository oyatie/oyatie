---
ip_id: IP-019
microservice: tenancy
bounded_context: dr-pairing
layer: usecase
status: in-progress
related_adrs: [ADR-0244, ADR-0248, ADR-0252, ADR-0263]
---

# IP-019 — DR-pairing controller

> **Delivery note (2026-08-20).** Implemented in tenancy/core/dr-pairing as `tenancy-dr-pairing`, collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: same-jurisdiction pairing, versioned promotion with optimistic concurrency, and auditable failover events. Deferred and named as a gap in the crate's `lib.rs` header:
> cell composition from the sibling cell-assignment crate and Cedar residency evaluation — both modelled locally as ports because a cross-crate path dep rewrites the frozen lockfile. The crate names in the tables below are this plan's original
> proposal, not what shipped.


## A. Problem

`tenancy` owns the tenant-to-cell assignment, but current plans do not define how each tenant receives a disaster-recovery pair or how promotion preserves residency. A generic "fail over to another region" plan is invalid for tenancy: every tenant carries immutable `jurisdiction_code`, RLS state, audit-chain seals, and downstream `TenantContext` dependencies. DR promotion that crosses pack boundaries would create an isolation incident while trying to fix an availability incident.

## B. Approach

Create `oya-tenancy-dr-pairing-usecase` to assign a same-jurisdiction home/DR pair, evaluate promotion eligibility, and emit auditable promotion/restoration events. The usecase composes existing `CellAssignment` data from `tenancy/contracts/openapi/tenancy.yaml`, residency policy from `tenancy/policy/data-residency.cedar`, and SLO signals from tenancy dashboards.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-dr-pairing-usecase/Cargo.toml` | create | Usecase crate. |
| `src/assign_pair.rs` | create | Assign DR pair at tenant activation. |
| `src/promote.rs` | create | Promotion eligibility and command handler. |
| `src/restore.rs` | create | Restore home cell after exercise or incident. |
| `src/ports.rs` | create | `CellAssignmentReadPort`, `ResidencyPolicyPort`, `BurnRateReadPort`, `AuditEmitPort`. |
| `tenancy/capabilities/dr-pair-promote.yaml` | align | Capability points to this usecase. |
| `tenancy/catalog/oya-tenancy-dr-pairing-usecase.yaml` | update/create | Catalog evidence. |

## D. Implementation

1. Read the current `CellAssignment` and candidate cells; require matching `pack` and jurisdiction.
2. Score candidates by health, load percentage, pack support, and distance from the home fault domain.
3. Persist a planned pair with `home_cell_id`, `dr_cell_id`, `jurisdiction_code`, `rpo_seconds`, `rto_seconds`, and `pair_version`.
4. Implement `promote()` so planned exercises require operator approval and automatic promotion requires burn-rate breach plus quorum guard.
5. Use TrueTime/HLC abstractions from ADR-0252 for promotion ordering; dev/test can use deterministic fake time.
6. Evaluate `policy/data-residency.cedar` before any pair assignment or promotion event.
7. Emit `oya.tenancy.dr-pairing-promoted` and `oya.tenancy.dr-pairing-restored` with idempotency key and audit-chain correlation id.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-dr-pairing-usecase --all-features`.
- Same-jurisdiction invariant tested for KR, EU, US-HC, and BR packs.
- Split-brain test refuses promotion unless quorum and current pair version match.
- RPO <= 30s and RTO <= 5min remain capability targets, not claimed measured results until drill evidence lands.
- `tenancy/runbooks/dr-pair-promotion-drill.md` references the command path and rollback evidence.

## F. Evidence

- `tenancy/PRD.md` identifies cell assignment and blast-radius bounding as tenant outcomes.
- `tenancy/contracts/openapi/tenancy.yaml` defines `CellAssignment` with `tenant_id`, `cell_id`, `shard_key`, `pack`, and `cell_health`.
- `tenancy/policy/data-residency.cedar` is the residency policy gate.
- `tenancy/dashboards/dr-pairing-state.json` and `runbooks/dr-pair-promotion-drill.md` are the operational evidence surfaces.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| AWS Aurora Global Database | Regional failover with controlled promotion | Adds a tenancy-specific promotion controller, not just database failover. |
| CockroachDB multi-region | Survive regional failure while preserving locality | Keeps promotion within the tenant's residency pack. |
| Stripe | Regional payment resiliency patterns | Ensures payment tenants can keep trusted `TenantContext` during substrate failover. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-019-dr-pairing-controller.md` matched `openapi`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## DR posture (per ADR-0343)
- Manifest target source: `tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `tenancy/IP-019-dr-pairing-controller.md` matched `SLO, multi-region, payment`; anchors `tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
