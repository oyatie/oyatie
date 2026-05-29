---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-001
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD (Basic Data: BOM / Routing master)
tenant_class: substrate
persona: production-engineer
---

# IP-001: Domain layer for bom-revision

## A. Intent

The Bill-of-Materials revision is the **single source of truth for product structure** that every downstream PP/SCP/QM step depends on. In SAP S/4HANA the equivalent lives in `PP-BD` (Basic Data) across tables `STKO` (BOM header), `STPO` (BOM item), `MAST` (material-to-BOM allocation), and `STAS` (BOM item status). Every other PP slice — MRP explosion, routing, capacity calendar, production order — reads from a frozen revision of this domain.

This IP implements the **domain layer** (Clean Architecture inner ring) for `bom-revision`: pure value objects, aggregates, domain events, and invariants. NO I/O, NO database driver, NO HTTP. The layer compiles standalone and depends only on `oya-shared-types` and `oya-shared-time` (HLC). The downstream usecase / adapter / surface layers (IP-007, IP-013, IP-014) wire it to outside-world ports.

### A.1 Why the domain layer is its own IP

Hexagonal-clean separation forces the BOM invariants to be expressed in pure types so they are independently verifiable. Concretely: a circular BOM is a **graph-theoretic** invariant (no SCC of size > 1 in the directed component-of graph) and must be checkable without a database. SAP's `CS_BO_GR_INVERSE_LIST` does this in ABAP; we do it in Rust within `oya-production-planning-bom-domain`.

### A.2 Concrete SAP equivalence

| SAP S/4HANA PP-BD entity | Oyatie aggregate root | Concrete delta |
|---|---|---|
| `STKO` BOM header | `BomRevision` aggregate root | Tenant-scoped key `(tenant_id, bom_id, revision_no)`; HLC `effective_from` |
| `STPO` BOM item | `BomPosition` value object inside aggregate | Position lifecycle via state enum, NOT row deletes |
| `MAST` material-to-BOM allocation | `MaterialBomAssignment` row in PP usecase layer (IP-007) | NOT in domain — assignment is an aggregate-external relation |
| `STAS` BOM item status | `BomPosition.lifecycle_state` enum | `draft` / `released` / `obsolete` / `engineering_hold` |
| `MAKT` material description (text) | NOT modelled here | Belongs in `material-master` µservice |
| `STZU` BOM admin data | Folded into aggregate's audit fields | Single audit trail, no separate admin table |

### A.3 Journey leg

In `j101-multi-tier-supply-chain-formation`, BOM revisions feed MRP explosion (IP-016). A misshaped BOM (cycle, missing component, dangling parent) **must fail loudly** at the domain layer, never in production. This IP's invariants are what make the failure loud.

## B. Acceptance criteria

- **AC-1:** `BomRevision::new()` rejects with typed error `BomError::DuplicatePosition` if two `BomPosition` rows share the same `position_no` within the revision.
- **AC-2:** `BomRevision::validate_acyclic()` runs Tarjan's SCC over the position graph (parent_component_id → component_id) and returns `BomError::CircularBom { cycle_path }` if any SCC has size > 1.
- **AC-3:** `BomRevision::release()` transitions `lifecycle_state` from `draft` to `released` only if all positions are in `draft` or `released`; rejects with `BomError::PositionInEngineeringHold` otherwise.
- **AC-4:** `BomRevision::supersede(new_revision_no)` emits a `BomRevisionSupersededEvent` with the prior `revision_no` and HLC; the prior revision is frozen (`lifecycle_state=obsolete`) but never deleted.
- **AC-5:** All public methods are `#[must_use]` and return `Result<T, BomError>`; no `panic!`, no `unwrap()` in production paths.
- **AC-6:** Tenant invariant: `BomRevision::new()` rejects if any `BomPosition.tenant_id` differs from the aggregate's `tenant_id`.
- **AC-7:** Quantity invariant: `BomPosition::quantity_per_assembly` is `Decimal` with scale 6; values < 0 rejected; value = 0 allowed (phantom / reference position).
- **AC-8:** Cedar default-deny is preserved at every public entry point (resource boundary).

## C. Verification

```bash
cargo test -p oya-production-planning-bom-domain -- bom_revision::
cargo test -p oya-production-planning-bom-domain -- circular_bom_tarjan_detects_3_cycle
cargo test -p oya-production-planning-bom-domain -- circular_bom_tarjan_detects_self_loop
cargo test -p oya-production-planning-bom-domain -- duplicate_position_rejected
cargo test -p oya-production-planning-bom-domain -- cross_tenant_position_rejected
cargo test -p oya-production-planning-bom-domain -- supersede_emits_event_and_freezes_prior
cargo test -p oya-production-planning-bom-domain -- release_blocked_by_engineering_hold
cargo bench -p oya-production-planning-bom-domain -- bom_validate_acyclic_breadth_500_depth_8
```

Coverage floor: ≥ 95% line, ≥ 90% branch on `crates/oya-production-planning-bom-domain/`. Mutation-testing (cargo-mutants) survival rate ≤ 5%.

## D. Detailed mechanics

### D-1. Aggregate root types

```rust
// crates/oya-production-planning-bom-domain/src/bom_revision.rs

use oya_shared_time::Hlc;
use oya_shared_types::{TenantId, MaterialId, PrincipalId};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct BomRevision {
    tenant_id: TenantId,
    bom_id: BomId,
    revision_no: RevisionNo,
    material_id: MaterialId,                 // header material
    plant_code: PlantCode,
    bom_usage: BomUsage,                     // production | engineering | costing | ...
    base_quantity: Decimal,                  // per-N-units BOM
    base_uom: UnitOfMeasure,
    lifecycle_state: BomLifecycleState,
    positions: Vec<BomPosition>,             // 0..=10_000
    effective_from: Hlc,
    superseded_at: Option<Hlc>,
    superseded_by: Option<RevisionNo>,
    audit: AuditFields,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BomPosition {
    tenant_id: TenantId,
    position_no: PositionNo,                 // 1..=10_000
    parent_component_id: Option<MaterialId>, // None at level 0
    component_id: MaterialId,
    quantity_per_assembly: Decimal,          // scale 6
    uom: UnitOfMeasure,
    item_category: ItemCategory,             // stock / non-stock / phantom / class / document
    scrap_pct: Decimal,                      // 0..=100
    valid_from: Hlc,
    valid_to: Option<Hlc>,
    lifecycle_state: PositionLifecycleState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BomLifecycleState { Draft, Released, Obsolete, EngineeringHold }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionLifecycleState { Draft, Released, Obsolete, EngineeringHold }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BomUsage { Production, Engineering, Costing, Sales, Plant, Universal }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemCategory { Stock, NonStock, Phantom, Class, Document, Subcontract }
```

### D-2. Domain events (event-sourced projection)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BomRevisionEvent {
    Created { tenant_id: TenantId, bom_id: BomId, revision_no: RevisionNo, occurred_at: Hlc, principal_id: PrincipalId },
    PositionAdded { bom_id: BomId, revision_no: RevisionNo, position_no: PositionNo, component_id: MaterialId, occurred_at: Hlc },
    PositionUpdated { bom_id: BomId, revision_no: RevisionNo, position_no: PositionNo, diff: PositionDiff, occurred_at: Hlc },
    Released { bom_id: BomId, revision_no: RevisionNo, occurred_at: Hlc, principal_id: PrincipalId },
    PlacedOnHold { bom_id: BomId, revision_no: RevisionNo, reason: EngineeringHoldReason, occurred_at: Hlc },
    Superseded { bom_id: BomId, prior_revision_no: RevisionNo, new_revision_no: RevisionNo, occurred_at: Hlc, principal_id: PrincipalId },
}
```

All events carry `tenant_id`, `policy_bundle_version`, `cedar_decision_id`, `correlation_id` at the envelope layer (added by IP-013 adapter).

### D-3. Tarjan SCC for circular-BOM detection

```rust
impl BomRevision {
    pub fn validate_acyclic(&self) -> Result<(), BomError> {
        // Build directed graph: parent -> child
        let mut adj: HashMap<&MaterialId, Vec<&MaterialId>> = HashMap::new();
        for pos in &self.positions {
            if let Some(parent) = &pos.parent_component_id {
                adj.entry(parent).or_default().push(&pos.component_id);
            }
        }
        // Tarjan's algorithm
        let mut tarjan = TarjanScc::new(&adj);
        for scc in tarjan.strongly_connected_components() {
            if scc.len() > 1 {
                return Err(BomError::CircularBom {
                    cycle_path: scc.iter().map(|m| (*m).clone()).collect(),
                });
            }
            // self-loop detection
            if scc.len() == 1 && adj.get(scc[0]).map_or(false, |children| children.contains(&scc[0])) {
                return Err(BomError::SelfReferentialPosition {
                    component_id: scc[0].clone(),
                });
            }
        }
        Ok(())
    }
}
```

### D-4. Typed error model

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BomError {
    #[error("duplicate position_no={position_no} within revision={revision_no}")]
    DuplicatePosition { position_no: PositionNo, revision_no: RevisionNo },

    #[error("circular BOM detected: {cycle_path:?}")]
    CircularBom { cycle_path: Vec<MaterialId> },

    #[error("self-referential position: component_id={component_id} cannot be its own parent")]
    SelfReferentialPosition { component_id: MaterialId },

    #[error("cross-tenant violation: position.tenant_id={position_tenant} != revision.tenant_id={revision_tenant}")]
    CrossTenantViolation { position_tenant: TenantId, revision_tenant: TenantId },

    #[error("quantity must be >= 0; received {received}")]
    NegativeQuantity { received: Decimal },

    #[error("position in engineering hold cannot be released; position_no={position_no}")]
    PositionInEngineeringHold { position_no: PositionNo },

    #[error("revision is obsolete and cannot be modified")]
    ObsoleteRevisionImmutable,

    #[error("supersede target revision_no={new} must be > current={current}")]
    InvalidSupersedeRevisionNo { current: RevisionNo, new: RevisionNo },

    #[error("base_quantity must be > 0; received {received}")]
    InvalidBaseQuantity { received: Decimal },
}
```

### D-5. Invariant matrix

| Invariant | Check site | Error |
|---|---|---|
| Tenant uniformity | `BomRevision::new` | `CrossTenantViolation` |
| Position uniqueness | `BomRevision::new` | `DuplicatePosition` |
| Acyclic graph | `BomRevision::validate_acyclic` | `CircularBom` / `SelfReferentialPosition` |
| Non-negative quantity | `BomPosition::new` | `NegativeQuantity` |
| Released only from draft/released positions | `BomRevision::release` | `PositionInEngineeringHold` |
| Obsolete revisions immutable | every mutator | `ObsoleteRevisionImmutable` |
| Supersede monotonicity | `BomRevision::supersede` | `InvalidSupersedeRevisionNo` |
| Base quantity > 0 | `BomRevision::new` | `InvalidBaseQuantity` |

### D-6. Audit-event class

`EVT-PRODUCTION_PLANNING-BOM_REVISION-IP_ACCEPTED` registered per ADR-0263 audit-class registry; emitted by adapter (IP-013) on every state transition.

### D-7. SLO ownership

The domain layer carries no runtime SLO directly (it is in-process). Its budget is:
- `BomRevision::validate_acyclic` ≤ 2ms P95 for 500-position, depth-8 BOM (benched).
- `BomRevision::new` ≤ 200µs P95 for a 100-position BOM.

These feed the IP-002 MRP-run SLO (`production-planning.mrp-explosion` p95-30s) by leaving ≥99% of budget for I/O.

### D-8. Cross-µservice handoffs (read-only consumers of this aggregate)

| Consumer µservice | Read path | Purpose |
|---|---|---|
| `supply-chain-planning` | Ontology projection `ontology.production_planning.bom_revision` | MRP explosion feeds dependent requirements |
| `quality-management` | Read via gRPC `GetBomRevision` | Inspection plan derivation per component |
| `costing` | Ontology projection (library-first) | Cost roll-up per BOM position |
| `marketplace` | Read-only ontology view (settlement OWNED elsewhere per ADR-0314) | Supplier-facing BOM exposure |

## E. Failure modes & recovery

### E-1. Circular BOM detected on `release()`

**Detection:** `validate_acyclic` returns `BomError::CircularBom`.
**Behaviour:** `release()` returns the error; revision stays in `draft`. No event emitted.
**Recovery:** Production engineer inspects the cycle path (returned in the error), edits the offending position, retries release. Runbook `runbooks/circular-bom-repair.md`.

### E-2. Mid-edit revision becomes obsolete (concurrent supersede)

**Detection:** Mutator on an obsolete revision returns `ObsoleteRevisionImmutable`.
**Behaviour:** Edit rejected at domain layer; usecase layer (IP-007) catches and re-reads the current active revision.
**Recovery:** Optimistic concurrency: editor re-applies changes against the new revision.

### E-3. Tenant context misconfigured at construction

**Detection:** `CrossTenantViolation` raised in `BomRevision::new`.
**Behaviour:** Aggregate never instantiated; no partial state. Caller emits a security audit `EVT-SECURITY-CROSS_TENANT_ATTEMPT`.
**Recovery:** Operator inspects the principal token and policy_bundle_version; tenant isolation rebuilt per ADR-0244.

## F. Migration

- Phase 1: domain crate ships behind `cargo` feature flag `bom_domain_v1`; older callers continue against the legacy adapter (none yet).
- Phase 2: usecase layer (IP-007) wires to this domain; integration tests pass.
- Phase 3: adapter (IP-013) persists via outbox; CDC topic `production-planning.bom-revision.v1` switched on.

Rollback: flag → false; domain crate yanked from workspace.

## G. References

- ADR-0105 (layer enum), ADR-0131 (per-µservice flat layout), ADR-0244 (tenant primitive), ADR-0263 (audit registry), ADR-0294 (Cedar soak), ADR-0315 (SAP parity), ADR-0316 (audit anchoring), ADR-0297 (HLC defaults).
- Tarjan, R. E. (1972). "Depth-first search and linear graph algorithms." SIAM J. Comput. 1(2): 146–160.
- SAP Help: PP-BD master data (`https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/...`, cited for entity mapping only).
- Benchmarks: SAP S/4HANA PP-BD | Oracle Fusion Cloud Manufacturing | Microsoft Dynamics 365 Supply Chain Management | NetSuite Manufacturing | Workday Adaptive Planning manufacturing-capacity counterpart.

## H. Out-of-scope

- Material master (lives in `material-master` µservice).
- Routing operations (IP-004 routing-step).
- Persistence (IP-013 adapter).
- HTTP/gRPC surface (IP-014).
- Cross-µservice handoffs (IP-016/IP-017).

— end IP-001 —
