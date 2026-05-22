---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-021
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-CRP (Capacity Requirements Planning) + PP-DS (Production Planning and Detailed Scheduling) finite-scheduling layer — transactions CM27/CM28 (capacity leveling), /SAPAPO/CDPSC (DS planning board)
tenant_class: substrate
persona: detailed-scheduler + shop-floor-supervisor
---

# IP-021: Capacity leveling with finite scheduling (forward + backward + bottleneck-anchor strategies)

## A. Intent

Implements **finite-capacity scheduling** for production-planning. Distinct from infinite scheduling (which lets operations overlap regardless of capacity) and from MRP's planning-direction-driven scheduling, finite scheduling **respects the actual capacity envelope** of each work center across the time horizon, producing schedules where no two operations compete for the same finite resource. SAP equivalents: `CM27`/`CM28` (interactive capacity leveling), `/SAPAPO/CDPSC` (APO/PP-DS detailed scheduling planning board); Oracle Fusion equivalent: Production Scheduling Cloud Service; Dynamics 365 SCM: master planning + Planning Optimization scheduling; Siemens Opcenter APS (Advanced Planning & Scheduling); Dassault DELMIA Quintiq; PlanetTogether APS.

### A.1 Three canonical strategies

1. **Forward scheduling** — start from earliest possible date (today or order's earliest-start); operations land sequentially after their predecessors. Used when the planner wants to ASAP completion.
2. **Backward scheduling** — start from required finish date; operations land in reverse from finish back to start. Used when the planner has a fixed delivery commitment.
3. **Bottleneck-anchor scheduling** — identify the bottleneck work center (the one with highest utilization vs capacity); anchor its operations first onto its calendar; schedule non-bottleneck operations relative to bottleneck anchor (DBR — Drum-Buffer-Rope per TOC-author's Theory of Constraints).

Strategy choice is per-call AND per-product-family (configurable default).

### A.2 Why finite scheduling is non-trivial

1. **NP-hard in general** — finite scheduling is reducible to job-shop-scheduling problem (JSSP), NP-hard. Real implementations use heuristics + constraint-propagation.
2. **Constraint catalogue** — calendar working hours, work-center availability, setup-time between sequential operations on same WC, alternate-resource fallback, sequence-dependent setups (per ADR-0263 sequence registry pattern).
3. **Re-scheduling cascades** — any change (new order, calendar exception, confirmation delay) may require re-scheduling 1000s of operations. Algorithm must support **partial re-schedule** with stability heuristics (minimize moves vs prior schedule).
4. **Constraint propagation** — using arc-consistency (AC-3 algorithm) over interval variables: each operation has `(start, finish)` with domain constrained by WC calendar.
5. **Cedar gate on commit** — schedule preview is freely computable; **committing** to the order-system requires Cedar permit since it triggers downstream re-reservation cascades.

## B. Acceptance criteria

- **AC-1:** `LevelCapacityUseCase::execute(strategy, scope, weights)` returns a `ScheduleProposal` (preview) — no DB write to order reservations.
- **AC-2:** `CommitScheduleUseCase::execute(proposal_id)` Cedar-gated; commits proposal to order reservations + emits envelopes.
- **AC-3:** Three strategies: `Strategy::Forward`, `Strategy::Backward`, `Strategy::BottleneckAnchor` — all return same `ScheduleProposal` shape.
- **AC-4:** Bottleneck detection: WC with highest `requested_capacity / available_capacity` over horizon.
- **AC-5:** Setup-time constraints honored: `setup_minutes(predecessor_op, successor_op, work_center)` consulted from sequence-dependent setup matrix.
- **AC-6:** Partial re-schedule: `scope = SchedulingScope::PartialFrom(hlc)` only rescheduled operations starting at-or-after hlc; prior schedule preserved.
- **AC-7:** Schedule stability: minimize sum of (|new_start − old_start|) when partial re-schedule applied.
- **AC-8:** `ScheduleProposal` includes per-op `(operation_id, work_center_id, start_ts, finish_ts, alternate_used)` AND `unschedulable: Vec<OperationId>` (any not placed).
- **AC-9:** Audit emission per ADR-0263; security audit on commit deny.
- **AC-10:** Cross-tenant defence-in-depth on all loads.

## C. Verification

```bash
cargo test -p oya-production-planning-scheduling-usecase -- forward_strategy_happy_path
cargo test -p oya-production-planning-scheduling-usecase -- backward_strategy_meets_finish_date
cargo test -p oya-production-planning-scheduling-usecase -- bottleneck_anchor_drbr_pattern
cargo test -p oya-production-planning-scheduling-usecase -- setup_time_matrix_respected
cargo test -p oya-production-planning-scheduling-usecase -- partial_reschedule_stability
cargo test -p oya-production-planning-scheduling-usecase -- alternate_resource_fallback
cargo test -p oya-production-planning-scheduling-usecase -- unschedulable_operations_reported
cargo test -p oya-production-planning-scheduling-usecase -- commit_cedar_permit_required
cargo test -p oya-production-planning-scheduling-usecase -- commit_emits_reservation_envelopes
cargo test -p oya-production-planning-scheduling-usecase -- cross_tenant_load_rejected
cargo test -p oya-production-planning-scheduling-usecase -- ac3_propagation_terminates
```

## D. Detailed mechanics

### D-1. Data model

```sql
CREATE TABLE production_planning.schedule_proposal (
    tenant_id       TEXT NOT NULL,
    proposal_id     UUID NOT NULL,
    strategy        TEXT NOT NULL CHECK (strategy IN ('forward','backward','bottleneck_anchor')),
    horizon_from    TIMESTAMPTZ NOT NULL,
    horizon_to      TIMESTAMPTZ NOT NULL,
    bottleneck_wc   TEXT,
    placements      JSONB NOT NULL,    -- Vec<{operation_id, wc_id, start, finish, alternate_used}>
    unschedulable   JSONB NOT NULL,    -- Vec<operation_id>
    objective_value NUMERIC(18,4),
    authored_by     TEXT NOT NULL,
    state           TEXT NOT NULL CHECK (state IN ('preview','committed','superseded','expired')),
    expires_at      TIMESTAMPTZ NOT NULL,    -- proposal TTL
    hlc             TEXT NOT NULL,
    decision_id     UUID NOT NULL,
    PRIMARY KEY (tenant_id, proposal_id)
) PARTITION BY HASH (tenant_id);

CREATE TABLE production_planning.setup_time_matrix (
    tenant_id      TEXT NOT NULL,
    work_center_id TEXT NOT NULL,
    predecessor_class TEXT NOT NULL,
    successor_class TEXT NOT NULL,
    setup_minutes  INTEGER NOT NULL,
    PRIMARY KEY (tenant_id, work_center_id, predecessor_class, successor_class)
) PARTITION BY HASH (tenant_id);
```

### D-2. Rust types

```rust
#[derive(Debug, Clone)]
pub enum Strategy { Forward, Backward, BottleneckAnchor }

#[derive(Debug, Clone)]
pub enum SchedulingScope {
    Horizon { from: DateTime<Utc>, to: DateTime<Utc> },
    PartialFrom { hlc: Hlc },
    Order { order_id: OrderId },
}

#[derive(Debug, Clone)]
pub struct ScheduleProposal {
    pub tenant_id: TenantId, pub proposal_id: Uuid,
    pub strategy: Strategy, pub bottleneck_wc: Option<WorkCenterId>,
    pub placements: Vec<Placement>,
    pub unschedulable: Vec<OperationId>,
    pub objective_value: Decimal,
    pub state: ProposalState, pub expires_at: DateTime<Utc>,
    pub hlc: Hlc, pub decision_id: DecisionId,
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub operation_id: OperationId, pub order_id: OrderId,
    pub work_center_id: WorkCenterId, pub start_ts: DateTime<Utc>,
    pub finish_ts: DateTime<Utc>, pub alternate_used: bool,
    pub prior_start_ts: Option<DateTime<Utc>>,
    pub setup_minutes: i32,
}
```

### D-3. Forward scheduling algorithm sketch (priority-rule heuristic + AC-3 propagation)

```rust
pub fn schedule_forward(input: ForwardInput) -> ScheduleProposal {
    let mut placements = Vec::new();
    let mut unsched = Vec::new();
    let mut wc_timeline: HashMap<WorkCenterId, Timeline> = build_wc_timelines(&input.work_centers);
    // priority rule: earliest-due-date first, ties broken by shortest-processing-time
    let mut ops = input.operations.clone();
    ops.sort_by_key(|o| (o.due_date, o.processing_time));
    for op in ops {
        let predecessor_finish = placements.iter()
            .filter(|p| p.order_id == op.order_id && op.predecessors.contains(&p.operation_id))
            .map(|p| p.finish_ts)
            .max()
            .unwrap_or(input.horizon_from);
        let wc = op.preferred_wc.clone();
        let setup = lookup_setup(&input.setup_matrix, &wc, &predecessor_class_of(&placements, &op), &op.class);
        let earliest = predecessor_finish + Duration::minutes(setup as i64);
        match wc_timeline.get_mut(&wc).unwrap().find_slot(earliest, op.processing_time) {
            Some((start, finish)) => placements.push(Placement {
                operation_id: op.operation_id, order_id: op.order_id,
                work_center_id: wc, start_ts: start, finish_ts: finish,
                alternate_used: false, prior_start_ts: op.prior_start_ts,
                setup_minutes: setup,
            }),
            None => {
                // try alternates
                if let Some(alt_placement) = try_alternates(&op, &mut wc_timeline, &input.setup_matrix, earliest) {
                    placements.push(alt_placement);
                } else {
                    unsched.push(op.operation_id);
                }
            }
        }
    }
    ScheduleProposal { /* construct from placements + unsched */ ... }
}
```

### D-4. Bottleneck-anchor (DBR) algorithm sketch

```rust
pub fn schedule_bottleneck_anchor(input: BottleneckInput) -> ScheduleProposal {
    // Step 1: identify bottleneck
    let bottleneck = input.work_centers.iter()
        .max_by_key(|wc| (input.demand_minutes(wc) * Decimal::from(1000) / wc.capacity_minutes).to_i64().unwrap_or(0))
        .map(|wc| wc.id.clone())
        .unwrap_or_default();
    // Step 2: anchor bottleneck operations on bottleneck timeline (forward)
    let mut placements = anchor_bottleneck(&bottleneck, &input);
    // Step 3: pre-schedule (rope) upstream operations to feed bottleneck on time
    placements.extend(schedule_upstream(&placements, &input));
    // Step 4: post-schedule (drum) downstream operations following bottleneck
    placements.extend(schedule_downstream(&placements, &input));
    // Step 5: report unschedulables
    let unsched = compute_unschedulable(&input, &placements);
    ScheduleProposal { strategy: Strategy::BottleneckAnchor, bottleneck_wc: Some(bottleneck), /* … */ }
}
```

### D-5. Constraint-propagation (AC-3 arc consistency)

For each operation, the variable domain is the legal start-time intervals (after intersecting WC calendar, predecessor finish, setup constraints). AC-3 repeatedly prunes domains until no more changes — terminates in `O(e·d^3)` where `e` is constraint count and `d` is domain size.

### D-6. Commit use-case (Cedar-gated)

```rust
pub struct CommitScheduleUseCase<R, C, K, O, A> { /* … */ }

impl<R, C, K, O, A> CommitScheduleUseCase<R, C, K, O, A>
where R: ScheduleRepository, C: CedarEvaluator, K: CapacityReservationPort,
      O: OutboxDispatcher, A: AuditEmitter,
{
    pub async fn execute(&self, input: CommitInput) -> Result<CommitOutput, UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req_commit(&input)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }

        let tx = self.repo.begin_tx().await?;
        let proposal = self.repo.load_proposal(&tx, &input.tenant_id, &input.proposal_id).await?
            .ok_or(UseCaseError::NotFound)?;
        if proposal.state != ProposalState::Preview { return Err(UseCaseError::IllegalStateTransition { from: proposal.state, to: ProposalState::Committed }); }
        if Utc::now() > proposal.expires_at { return Err(UseCaseError::ProposalExpired); }
        // Re-reserve capacity per placement
        for p in &proposal.placements {
            self.capacity.reserve_for_op(&tx, &input.tenant_id, &p, &decision).await?;
        }
        self.repo.transition_proposal_to_committed(&tx, &input.tenant_id, &input.proposal_id, Hlc::now()).await?;
        self.outbox.append(&tx, &schedule_committed_event(&proposal, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::commit_schedule(&proposal, &decision)).await?;
        tx.commit().await?;
        Ok(CommitOutput { decision_id: decision.decision_id, hlc: Hlc::now() })
    }
}
```

### D-7. Cedar context (commit)

```jsonc
{
  "principal": "oyatie::tenant::acme::user::scheduler-3",
  "action":    "production_planning::schedule::commit",
  "resource":  "production_planning::schedule::proposal::{uuid}",
  "context": {
    "tenant_id": "acme", "strategy": "bottleneck_anchor",
    "placement_count": 1247, "unschedulable_count": 3,
    "data_class": "operational",
    "policy_bundle_version": "2026.05.20-r3",
    "residency_pack": "global+kr",
    "byok_mode": "platform_default"
  }
}
```

### D-8. AsyncAPI envelopes

| Channel | Trigger | Consumers |
|---|---|---|
| `production-planning.schedule.proposal-created.v1` | preview | `dashboards`, `analytics` |
| `production-planning.schedule.committed.v1` | commit | `production-order` reservations, `mes`, `warehouse` |
| `production-planning.schedule.unschedulable.v1` | unsched > 0 | `alerting`, `dashboards` |
| `production-planning.schedule.bottleneck-shift.v1` | bottleneck WC changed since last run | `analytics` |

### D-9. Workflow with decision branches

```mermaid
flowchart TB
  A[execute(strategy, scope)] --> B[Load WC calendars + capacity envelope]
  B --> C{Strategy}
  C -- forward --> D1[Forward heuristic + AC-3]
  C -- backward --> D2[Backward heuristic + AC-3]
  C -- bottleneck_anchor --> D3[Identify bottleneck → DBR anchor + upstream + downstream]
  D1 --> E[Compose ScheduleProposal]
  D2 --> E
  D3 --> E
  E --> F{Unschedulable > 0?}
  F -- yes --> G[Emit unschedulable.v1]
  F -- no --> H[Skip]
  G --> I[Persist proposal preview]
  H --> I
  I --> J[Return proposal_id]
  %% commit path
  J --> K[CommitScheduleUseCase]
  K --> L{Cedar permit?}
  L -- deny --> Z1[PermissionDenied]
  L -- permit --> M[Reserve capacity per op]
  M --> N[Transition proposal -> committed]
  N --> O[Emit schedule.committed.v1]
```

### D-10. SLO targets

| Operation | Scale | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|---|
| `LevelCapacity` (forward, 100 ops) | small | 50 ms | 110 ms | 220 ms | In-process heuristic. |
| `LevelCapacity` (forward, 1k ops) | medium | 480 ms | 1.1 s | 2.2 s | Same heuristic. |
| `LevelCapacity` (bottleneck, 10k ops, 50 WCs) | large | 8 s | 18 s | 35 s | DBR + AC-3; daily batch sized. |
| `CommitSchedule` (1k placements) | medium | 1.2 s | 2.5 s | 5 s | Per-op capacity reserve + outbox + audit. |
| `CommitSchedule` (10k placements) | large | 12 s | 26 s | 50 s | Batched outbox; bulk reserve. |

### D-11. Audit-event registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-SCHEDULE-PROPOSAL_CREATED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-SCHEDULE-COMMITTED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-SCHEDULE-UNSCHEDULABLE_REPORTED` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-SCHEDULE-BOTTLENECK_SHIFTED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-SCHEDULE-PROPOSAL_EXPIRED` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-SCHEDULE-PERMISSION_DENIED` | security | usecase |

### D-12. Failure modes & recovery

1. **`UnschedulableOperations`** — N ops not placed; preview returned with `unschedulable` list. Scheduler triages: relax due-date, add alternate WC, or accept partial commit. Runbook `runbooks/scheduling-unschedulable.md`.
2. **`ProposalExpired`** — proposal TTL (default 30 min) elapsed before commit. Caller re-runs preview. Tunable per tenant.
3. **`CapacityReservationConflict`** (commit-time) — between preview and commit another commit consumed the slot. Caller re-runs preview with stability scope.
4. **`BottleneckMisidentification`** — utilization tied between two WCs; tiebreak by `work_center_id` lex. Logged at INFO.
5. **`PermissionDenied`** — Cedar deny on commit. Security audit; runbook `runbooks/schedule-commit-denied.md`.
6. **`PropagationNonTermination`** — AC-3 hits iteration cap (10·d²); abort with `ConstraintPropagationLimitReached` and partial proposal.

### D-13. Migration notes

Source vendor surface: SAP `CM27`/`CM28` interactive leveling + SAP APO `/SAPAPO/CDPSC` planning board + SAP PP-DS heuristics. Greenfield: empty proposal log. Lift-shift: replay committed schedules into proposal+commit history.

### D-14. Ontology projection

Schedule proposals project into ontology as nodes with edges to participating orders/operations/work-centers; useful for what-if analysis in downstream BI.

### D-15. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound  | `production-order` (IP-011) | gRPC list-operations |
| inbound  | this µservice — capacity (IP-009) | gRPC list-WC-availability |
| inbound  | `quality-management`        | AsyncAPI WC-hold events |
| inbound  | `plant-maintenance`         | AsyncAPI downtime overlay |
| outbound | `production-order`          | AsyncAPI `schedule.committed.v1` (re-reservations) |
| outbound | `manufacturing-execution-system` (IP-024) | AsyncAPI same channel |
| outbound | `warehouse`                 | AsyncAPI same channel (re-staging) |
| outbound | `analytics`                 | AsyncAPI `schedule.proposal-created.v1` |

## E. Failure-mode summary

See D-12.

## F. Migration / rollback

Feature flag `production_planning_finite_scheduling_v1`. Disabling falls back to infinite-scheduling preview (no reservations); planner pages SOSO (Send Order Stuff Out).

## G. References

- ADR-0105, ADR-0244, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315.
- TOC-author, *The Goal* (Theory of Constraints, DBR Drum-Buffer-Rope).
- Pinedo, *Scheduling: Theory, Algorithms, and Systems* (5th ed., Springer, 2016) — JSSP heuristics.
- Mackworth (1977), *Consistency in Networks of Relations* — AC-3 algorithm.
- SAP S/4HANA PP-CRP `CM27`/`CM28`; SAP APO PP-DS planning board `/SAPAPO/CDPSC`.
- Benchmarks: SAP PP-DS | Oracle Production Scheduling Cloud Service | Siemens Opcenter APS | Dassault DELMIA Quintiq | PlanetTogether APS | Asprova APS.

## H. Out of scope

- BOM/routing/order CRUD (IP-001..IP-011), MRP run (IP-002/IP-008), DDMRP (IP-018), S&OP horizon (IP-019), MES handshake (IP-024).

— end IP-021 —
