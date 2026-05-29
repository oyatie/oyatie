---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-004
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD-RTG (Routings, transactions CA01/CA02/CA03)
tenant_class: substrate
persona: process-engineer
---

# IP-004: Domain layer for routing-step

## A. Intent

A routing defines the **operations sequence and resource needs** to manufacture a finished good: each operation references a work-center, has setup/processing/teardown times, may consume PRTs (Production Resources/Tools), and emits a measurable output. In SAP S/4HANA the routing lives in tables `PLKO` (routing header), `PLPO` (operation), `PLAS` (sequence), `PLFL` (sequence link), and `PLMK` (inspection characteristics). Transactions: `CA01` (create), `CA02` (change), `CA03` (display).

This IP implements the **domain layer** for `routing-step`: pure aggregate `Routing`, value object `RoutingStep` (= operation), sequence linker, and routing-network validator (no parallel sequences with broken joins). No I/O.

### A.1 SAP equivalence delta

| SAP entity | Oyatie aggregate / value object |
|---|---|
| `PLKO` routing header | `Routing` aggregate root |
| `PLPO` operation | `RoutingStep` value object |
| `PLAS` sequence | `OperationSequence` (linked-list inside aggregate) |
| `PLFL` sequence link (parallel branching) | `BranchLink` value object |
| `PLMK` inspection chars | NOT here — owned by `quality-management.inspection-plan` |
| `PLPH` super-routing reference | `ReferenceRouting` value object (link to another routing) |
| Setup / Processing / Teardown / Wait | `TimeBlock` enum-tagged: `Setup`, `Processing { per_unit, base_qty }`, `Teardown`, `Wait` |

### A.2 Journey leg

`j101`: after MRP explosion creates planned production orders, the routing drives **operation-level scheduling** (IP-020) on capacity calendar (IP-003).

## B. Acceptance criteria

- **AC-1:** `Routing::new(steps)` rejects with `RoutingError::SequenceGap` if any step's `predecessor` references a step_no not present.
- **AC-2:** `Routing::validate_acyclic_network()` finds cycles via DFS over the predecessor graph and returns `RoutingError::CircularSequence { cycle_path }`.
- **AC-3:** Parallel sequences: branching/joining is supported; a `BranchJoin` step lists all incoming `step_no`s and is reachable from all of them.
- **AC-4:** Time blocks: `Processing { per_unit, base_qty }` — total processing = `per_unit * order_qty + base_qty`; `Setup` independent of qty; `Teardown` independent; `Wait` is queue-time only.
- **AC-5:** Tenant invariant: every step's `work_center_id` resolves within the same tenant.
- **AC-6:** Lifecycle: `Routing::release()` only from `draft`; once `released` the routing is immutable; `supersede()` creates a new revision.
- **AC-7:** PRT (tool) reservation: `RoutingStep::prt_requirements` is a `Vec<PrtRef>`; conflicts checked in the usecase layer (IP-010).
- **AC-8:** Cedar default-deny preserved on every public method entry.

## C. Verification

```bash
cargo test -p oya-production-planning-routing-domain -- routing_simple_linear
cargo test -p oya-production-planning-routing-domain -- routing_parallel_branch_join
cargo test -p oya-production-planning-routing-domain -- sequence_gap_rejected
cargo test -p oya-production-planning-routing-domain -- circular_sequence_detected
cargo test -p oya-production-planning-routing-domain -- branch_join_unreachable_predecessor_rejected
cargo test -p oya-production-planning-routing-domain -- released_routing_immutable
cargo test -p oya-production-planning-routing-domain -- supersede_emits_event
cargo test -p oya-production-planning-routing-domain -- processing_time_formula
cargo test -p oya-production-planning-routing-domain -- cross_tenant_workcenter_rejected
cargo bench -p oya-production-planning-routing-domain -- validate_network_steps_300
```

Coverage ≥ 95% line, ≥ 90% branch.

## D. Detailed mechanics

### D-1. Aggregate root

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Routing {
    tenant_id: TenantId,
    routing_id: RoutingId,
    revision_no: RevisionNo,
    material_id: MaterialId,
    plant_code: PlantCode,
    usage: RoutingUsage,                  // production | rework | repair | reference
    steps: Vec<RoutingStep>,
    lifecycle_state: RoutingLifecycleState,
    effective_from: Hlc,
    superseded_at: Option<Hlc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutingStep {
    step_no: StepNo,                      // 0010, 0020, ...
    predecessor: Option<Vec<StepNo>>,     // None at start; >1 = join
    work_center_id: WorkCenterId,
    control_key: ControlKey,              // routing-step "control key" PP01/PP02/etc.
    setup_time: TimeBlock,
    processing_time: TimeBlock,           // Processing { per_unit, base_qty, base_uom }
    teardown_time: TimeBlock,
    wait_time: TimeBlock,
    base_quantity: Decimal,               // referenced in Processing.base_qty
    base_uom: UnitOfMeasure,
    yield_pct: Decimal,                   // 0..=100 expected yield
    prt_requirements: Vec<PrtRef>,
    description: ShortText,               // 40-char operation description
    skill_qualifications: Vec<SkillId>,   // operator qualifications required
    standard_value_keys: Vec<StandardValueKey>, // e.g., labor, setup, machine
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeBlock {
    Setup(Duration),
    Processing { per_unit: Duration, base_qty: Decimal, base_uom: UnitOfMeasure },
    Teardown(Duration),
    Wait(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutingLifecycleState { Draft, Released, Obsolete }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutingUsage { Production, Rework, Repair, Reference }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlKey {
    Pp01ProcessNoCostInfo, Pp02StandardOperation, Pp03ExternalProcessing,
    Pp99CustomerDefined,
}
```

### D-2. Sequence network validator (DFS for cycles)

```rust
impl Routing {
    pub fn validate_acyclic_network(&self) -> Result<(), RoutingError> {
        let mut adj: HashMap<StepNo, Vec<StepNo>> = HashMap::new();
        for step in &self.steps {
            for pred in step.predecessor.as_deref().unwrap_or(&[]) {
                adj.entry(*pred).or_default().push(step.step_no);
            }
        }
        let mut visited = HashSet::new();
        let mut on_stack = HashSet::new();
        for &start in adj.keys() {
            if !visited.contains(&start) {
                let mut stack = vec![(start, vec![start])];
                while let Some((node, path)) = stack.pop() {
                    on_stack.insert(node);
                    visited.insert(node);
                    for &next in adj.get(&node).unwrap_or(&vec![]) {
                        if on_stack.contains(&next) {
                            let mut cycle = path.clone();
                            cycle.push(next);
                            return Err(RoutingError::CircularSequence { cycle_path: cycle });
                        }
                        if !visited.contains(&next) {
                            let mut new_path = path.clone();
                            new_path.push(next);
                            stack.push((next, new_path));
                        }
                    }
                    on_stack.remove(&node);
                }
            }
        }
        Ok(())
    }

    pub fn total_time(&self, order_qty: Decimal) -> Result<Duration, RoutingError> {
        let mut total = Duration::ZERO;
        for step in &self.steps {
            total = total
                .checked_add(step.setup_time.duration())
                .and_then(|d| d.checked_add(step.processing_time.duration_for_qty(order_qty)))
                .and_then(|d| d.checked_add(step.teardown_time.duration()))
                .and_then(|d| d.checked_add(step.wait_time.duration()))
                .ok_or(RoutingError::DurationOverflow)?;
        }
        Ok(total)
    }
}
```

### D-3. Processing-time formula

```rust
impl TimeBlock {
    pub fn duration_for_qty(&self, order_qty: Decimal) -> Duration {
        match self {
            TimeBlock::Processing { per_unit, base_qty, .. } => {
                // total = per_unit * (order_qty / base_qty) — matches SAP CA01 formula
                let scale = order_qty / base_qty.max(Decimal::ONE);
                per_unit.mul_decimal(scale)
            }
            TimeBlock::Setup(d) | TimeBlock::Teardown(d) | TimeBlock::Wait(d) => *d,
        }
    }
}
```

### D-4. Typed errors

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RoutingError {
    #[error("sequence gap: step {step} references missing predecessor {predecessor}")]
    SequenceGap { step: StepNo, predecessor: StepNo },
    #[error("circular sequence: {cycle_path:?}")]
    CircularSequence { cycle_path: Vec<StepNo> },
    #[error("branch-join step {step} predecessor list contains unreachable step")]
    UnreachableBranchPredecessor { step: StepNo },
    #[error("released routing is immutable")]
    ReleasedImmutable,
    #[error("cross-tenant work-center ref: step {step}")]
    CrossTenantWorkCenter { step: StepNo },
    #[error("duration overflow at step {step}")]
    DurationOverflow,
    #[error("yield_pct outside [0,100]: step={step} value={value}")]
    InvalidYield { step: StepNo, value: Decimal },
    #[error("base quantity <= 0 at step {step}")]
    InvalidBaseQty { step: StepNo },
}
```

### D-5. Lifecycle transitions

```text
Draft --release()--> Released --supersede(new_rev)--> Obsolete
                                  ^
                                  |
                                  +-- new Routing aggregate created with new revision_no
```

### D-6. Audit-event class

`EVT-PRODUCTION_PLANNING-ROUTING_STEP-IP_ACCEPTED` per ADR-0263.

### D-7. SLO contribution

In-process: `validate_acyclic_network` over 300-step routing ≤ 3ms P95. `total_time(order_qty)` ≤ 100µs.

### D-8. Cross-µservice consumers

| Consumer | Mode | Purpose |
|---|---|---|
| `quality-management` | ontology projection | inspection-plan derives inspection points from routing |
| `plant-maintenance` | gRPC `GetRouting` | tool-and-equipment maintenance windows align with rework routings |
| `production-planning.production-order` (IP-005) | in-process | order references the released routing |
| `warehouse` | ontology | staging timing per operation |
| `costing` | ontology | std-cost roll-up per operation |

## E. Failure modes & recovery

### E-1. Sequence gap on construction
**Detection:** `RoutingError::SequenceGap`.
**Behaviour:** Aggregate never instantiated.
**Recovery:** Process engineer adds missing predecessor or removes orphan; runbook `runbooks/routing-sequence-gap.md`.

### E-2. Circular sequence (rework loop misconfigured)
**Detection:** `RoutingError::CircularSequence`.
**Behaviour:** Validator returns the offending cycle.
**Recovery:** Engineer marks the loop as a `RoutingUsage::Rework` reference routing instead of inline.

### E-3. Released routing edit attempt
**Detection:** `RoutingError::ReleasedImmutable`.
**Behaviour:** Mutator rejects.
**Recovery:** Engineer creates a new revision via `supersede`.

### E-4. Yield % out of range
**Detection:** `RoutingError::InvalidYield`.
**Behaviour:** Construction rejects.
**Recovery:** Process engineer reviews historical yield data and re-enters within [0,100].

## F. Migration

Phase 1: domain layer.
Phase 2 (IP-010): usecase wiring.
Phase 3 (IP-013): adapter + outbox.
Phase 4 (IP-020 finite scheduling): routing × capacity → schedule.

Rollback: feature flag `production_planning_routing_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0297, ADR-0315.
- SAP Help: PP-BD Routing (`CA01`/`CA02`, tables `PLKO`/`PLPO`/`PLAS`/`PLFL`).
- Benchmarks: SAP S/4HANA Routing | Oracle Fusion Manufacturing Routings | Siemens Opcenter | Microsoft Dynamics 365 SCM Routes.

## H. Out-of-scope

- Inspection characteristics (owned by `quality-management.inspection-plan`).
- PRT availability (IP-010 + IP-021).
- Persistence (IP-013).

— end IP-004 —
