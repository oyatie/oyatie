---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-025
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-REM (Repetitive Manufacturing) line balancing + PP-DS heuristic-based line layout — transactions OPMK (line maintenance), MF50 (line loading), CM33 (line capacity overview); SAP DMC line-balancing screens
tenant_class: substrate
persona: industrial-engineer + lean-engineer + assembly-line-supervisor
---

# IP-025: Production-line balancing algorithm with takt-time computation + workstation load smoothing

## A. Intent

Implements **production-line balancing** — the algorithmic assignment of work-content (tasks) to workstations along a production line such that:

1. **Takt time** is met: every workstation completes its assigned work within `takt = available_production_time / required_output`.
2. **Cycle time variance** is minimized across workstations (load smoothing).
3. **Precedence constraints** are respected (some tasks must precede others).
4. **Workstation capability constraints** are honored (some tasks require specific equipment / skills).
5. **Ergonomic / safety constraints** are honored (no workstation overloaded beyond ergonomic threshold per ISO 11228).

This is the discrete-manufacturing analogue to repetitive-manufacturing capacity planning. SAP transactions: `OPMK` (line maintenance), `MF50` (line loading), `CM33` (line capacity overview); SAP DMC line-balancing module. Oracle Fusion equivalent: Production Scheduling Cloud line-balancing solver; Dynamics 365 SCM: Lean manufacturing kanban rules; Siemens Tecnomatix Plant Simulation (deep-engineering tool); Industrial Engineering toolkits: ProModel, FlexSim, Arena (used at line-layout design phase).

### A.1 Algorithm family

Line balancing is **NP-hard** (reducible to bin packing). Classical algorithms used:

- **RPW (Ranked Positional Weight)** — Helgeson & Birnie (1961). Compute each task's positional weight (own duration + downstream task durations); assign tasks to stations in descending RPW order, packing into station capacity. Simple, deterministic, ≤ 10% sub-optimal vs MILP for typical lines.
- **LCR (Largest Candidate Rule)** — at each station, among feasible tasks (precedence cleared), pick the largest-duration that fits.
- **MILP (Mixed-Integer Linear Programming)** — for small lines (<50 tasks); GLPK / CBC; provably optimal but slow.
- **Simulated annealing** — for medium lines (50-200 tasks); good quality, tunable runtime.
- **Genetic algorithm** — for large lines (>200 tasks) and multi-objective formulations (cost + variance + safety).

Oyatie ships RPW + LCR as default; MILP / SA / GA pluggable per tenant pack.

### A.2 Why this is non-trivial

1. **Mixed-model lines** — single line produces multiple variants (e.g., 3 car models, 5 SKU variants). Balancing must work for the **weighted-average product mix** AND each individual model.
2. **Recomputation cadence** — line balance recomputes on (i) BOM change → IP-001, (ii) routing change → IP-010, (iii) demand-mix change → S&OP IP-019, (iv) workstation outage → IP-009.
3. **Validity window** — balance is stable across a planning horizon; over-frequent rebalancing causes worker re-training overhead. Hysteresis enforced.
4. **Cedar gate on publish** — published line balance changes workstation assignments → affects labour scheduling → HR-impact → Cedar permit `production_planning::line_balance::publish` required.
5. **Audit + reasoning chain** — every published balance includes algorithm, inputs, RPW values per task, station assignments, idle-time per station, smoothing-index for traceability.

## B. Acceptance criteria

- **AC-1:** `ComputeLineBalanceUseCase::execute(line_id, takt_seconds, model_mix, algorithm)` returns `LineBalanceProposal` with per-station task list + idle time + smoothing-index.
- **AC-2:** Algorithms supported: `Algorithm::Rpw`, `Algorithm::Lcr`, `Algorithm::Milp`, `Algorithm::SimulatedAnnealing`, `Algorithm::GeneticAlgorithm`.
- **AC-3:** Takt time = `available_production_time_per_shift / required_output_per_shift`; computed from S&OP demand (IP-019) + shift calendar (IP-009).
- **AC-4:** Precedence DAG respected; cycle detection rejects invalid input with `TaskGraphHasCycle` typed error.
- **AC-5:** Mixed-model: weighted-average task duration computed per task = `Σ_m (mix_share_m * duration_m)`; each model's individual cycle time still ≤ takt (typical mix-balancing constraint).
- **AC-6:** Smoothing-index = `sqrt(Σ_i (takt - station_load_i)²)`; lower is better.
- **AC-7:** `PublishLineBalanceUseCase::execute(proposal)` Cedar-gated; emits `production-line.balance-published.v1` AsyncAPI.
- **AC-8:** Hysteresis: re-publish allowed only if new smoothing-index improves by ≥ 5% AND new max-station-load below 95% of takt.
- **AC-9:** Audit emission per ADR-0263.
- **AC-10:** Cross-tenant defence-in-depth.

## C. Verification

```bash
cargo test -p oya-production-planning-linebalance-usecase -- rpw_algorithm_simple_line
cargo test -p oya-production-planning-linebalance-usecase -- lcr_algorithm_assignment
cargo test -p oya-production-planning-linebalance-usecase -- milp_algorithm_optimal_small_line
cargo test -p oya-production-planning-linebalance-usecase -- simulated_annealing_medium_line
cargo test -p oya-production-planning-linebalance-usecase -- genetic_algorithm_large_line
cargo test -p oya-production-planning-linebalance-usecase -- takt_time_from_sop_and_calendar
cargo test -p oya-production-planning-linebalance-usecase -- precedence_dag_cycle_rejected
cargo test -p oya-production-planning-linebalance-usecase -- mixed_model_weighted_duration
cargo test -p oya-production-planning-linebalance-usecase -- smoothing_index_computation
cargo test -p oya-production-planning-linebalance-usecase -- hysteresis_blocks_marginal_rebalance
cargo test -p oya-production-planning-linebalance-usecase -- publish_cedar_gated
cargo test -p oya-production-planning-linebalance-usecase -- ergonomic_threshold_enforced
cargo test -p oya-production-planning-linebalance-usecase -- cross_tenant_load_rejected
```

## D. Detailed mechanics

### D-1. Data model

```sql
CREATE TABLE production_planning.production_line (
    tenant_id     TEXT NOT NULL,
    line_id       TEXT NOT NULL,
    plant_code    TEXT NOT NULL,
    workstations  JSONB NOT NULL,  -- [{station_id, capability_tags, ergonomic_limit_seconds}]
    state         TEXT NOT NULL CHECK (state IN ('draft','active','retired')),
    hlc           TEXT NOT NULL,
    decision_id   UUID NOT NULL,
    PRIMARY KEY (tenant_id, line_id)
) PARTITION BY HASH (tenant_id);

CREATE TABLE production_planning.line_balance_proposal (
    tenant_id     TEXT NOT NULL,
    proposal_id   UUID NOT NULL,
    line_id       TEXT NOT NULL,
    algorithm     TEXT NOT NULL CHECK (algorithm IN ('rpw','lcr','milp','simulated_annealing','genetic_algorithm')),
    takt_seconds  NUMERIC(10,4) NOT NULL,
    model_mix     JSONB NOT NULL,
    assignments   JSONB NOT NULL,    -- {station_id: [task_id, ...]}
    station_loads JSONB NOT NULL,    -- {station_id: seconds}
    smoothing_index NUMERIC(10,4) NOT NULL,
    max_station_load_pct NUMERIC(5,4) NOT NULL,
    rpw_table     JSONB,
    reasoning_chain JSONB NOT NULL,
    state         TEXT NOT NULL CHECK (state IN ('preview','published','superseded','rejected')),
    authored_by   TEXT NOT NULL,
    published_by  TEXT,
    published_at  TIMESTAMPTZ,
    hlc           TEXT NOT NULL,
    decision_id   UUID NOT NULL,
    PRIMARY KEY (tenant_id, proposal_id)
) PARTITION BY HASH (tenant_id);
```

### D-2. Rust types

```rust
#[derive(Debug, Clone)]
pub enum Algorithm { Rpw, Lcr, Milp, SimulatedAnnealing, GeneticAlgorithm }

#[derive(Debug, Clone)]
pub struct ProductionLine {
    pub tenant_id: TenantId, pub line_id: LineId, pub plant_code: PlantCode,
    pub workstations: Vec<Workstation>, pub state: LineState,
    pub hlc: Hlc, pub decision_id: DecisionId,
}

#[derive(Debug, Clone)]
pub struct Workstation {
    pub station_id: StationId, pub capability_tags: BTreeSet<CapabilityTag>,
    pub ergonomic_limit_seconds: Decimal,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub task_id: TaskId, pub duration_per_model: BTreeMap<ModelId, Decimal>,
    pub capability_required: BTreeSet<CapabilityTag>,
    pub predecessors: BTreeSet<TaskId>,
}

#[derive(Debug, Clone)]
pub struct LineBalanceProposal {
    pub tenant_id: TenantId, pub proposal_id: Uuid, pub line_id: LineId,
    pub algorithm: Algorithm, pub takt_seconds: Decimal,
    pub model_mix: BTreeMap<ModelId, Decimal>,    // shares sum to 1.0
    pub assignments: BTreeMap<StationId, Vec<TaskId>>,
    pub station_loads: BTreeMap<StationId, Decimal>,
    pub smoothing_index: Decimal,
    pub max_station_load_pct: Decimal,
    pub rpw_table: Option<BTreeMap<TaskId, Decimal>>,
    pub reasoning_chain: ReasoningChain,
    pub state: ProposalState, pub hlc: Hlc, pub decision_id: DecisionId,
}
```

### D-3. RPW algorithm

```rust
pub fn compute_rpw(tasks: &[Task], mix: &BTreeMap<ModelId, Decimal>) -> BTreeMap<TaskId, Decimal> {
    let weighted_dur: BTreeMap<TaskId, Decimal> = tasks.iter()
        .map(|t| (t.task_id.clone(), t.duration_per_model.iter()
            .map(|(m, d)| mix.get(m).copied().unwrap_or(Decimal::ZERO) * *d).sum()))
        .collect();
    // RPW = task duration + sum of all downstream task durations (DFS)
    let succ = build_successor_index(tasks);
    let mut rpw: BTreeMap<TaskId, Decimal> = BTreeMap::new();
    fn dfs(t: &TaskId, succ: &BTreeMap<TaskId, BTreeSet<TaskId>>, dur: &BTreeMap<TaskId, Decimal>, memo: &mut BTreeMap<TaskId, Decimal>) -> Decimal {
        if let Some(v) = memo.get(t) { return *v; }
        let mut total = dur.get(t).copied().unwrap_or(Decimal::ZERO);
        if let Some(s) = succ.get(t) {
            for n in s { total += dfs(n, succ, dur, memo); }
        }
        memo.insert(t.clone(), total);
        total
    }
    for t in tasks {
        let v = dfs(&t.task_id, &succ, &weighted_dur, &mut rpw);
        rpw.insert(t.task_id.clone(), v);
    }
    rpw
}

pub fn assign_rpw(tasks: &[Task], rpw: &BTreeMap<TaskId, Decimal>, takt: Decimal,
                  stations: &[Workstation], mix: &BTreeMap<ModelId, Decimal>)
    -> Result<BTreeMap<StationId, Vec<TaskId>>, LbError>
{
    let mut ordered: Vec<_> = tasks.iter().collect();
    ordered.sort_by(|a, b| rpw.get(&b.task_id).cmp(&rpw.get(&a.task_id)));
    let mut assignments: BTreeMap<StationId, Vec<TaskId>> = stations.iter().map(|s| (s.station_id.clone(), Vec::new())).collect();
    let mut loads: BTreeMap<StationId, Decimal> = stations.iter().map(|s| (s.station_id.clone(), Decimal::ZERO)).collect();
    let mut assigned: BTreeSet<TaskId> = BTreeSet::new();
    for task in ordered {
        if !task.predecessors.is_subset(&assigned) {
            return Err(LbError::PrecedenceUnsatisfiable { task: task.task_id.clone() });
        }
        let weighted_dur: Decimal = task.duration_per_model.iter()
            .map(|(m, d)| mix.get(m).copied().unwrap_or(Decimal::ZERO) * *d).sum();
        let candidate_station = stations.iter()
            .filter(|s| task.capability_required.is_subset(&s.capability_tags))
            .filter(|s| loads.get(&s.station_id).copied().unwrap_or(Decimal::ZERO) + weighted_dur <= takt)
            .filter(|s| loads.get(&s.station_id).copied().unwrap_or(Decimal::ZERO) + weighted_dur <= s.ergonomic_limit_seconds)
            .min_by_key(|s| loads.get(&s.station_id).copied().unwrap_or(Decimal::ZERO));
        match candidate_station {
            Some(s) => {
                assignments.get_mut(&s.station_id).unwrap().push(task.task_id.clone());
                *loads.get_mut(&s.station_id).unwrap() += weighted_dur;
                assigned.insert(task.task_id.clone());
            }
            None => return Err(LbError::Unassignable { task: task.task_id.clone() }),
        }
    }
    Ok(assignments)
}
```

### D-4. Smoothing-index

```rust
pub fn compute_smoothing_index(loads: &BTreeMap<StationId, Decimal>, takt: Decimal) -> Decimal {
    let n = loads.len();
    if n == 0 { return Decimal::ZERO; }
    let sum_sq: Decimal = loads.values().map(|l| (takt - *l) * (takt - *l)).sum();
    // Decimal::sqrt is not in std; use Newton-Raphson for fixed precision
    decimal_sqrt(sum_sq)
}
```

### D-5. Hysteresis on publish

```rust
pub fn passes_hysteresis(prior: Option<&LineBalanceProposal>, candidate: &LineBalanceProposal) -> bool {
    match prior {
        None => true,
        Some(p) => {
            let improvement = (p.smoothing_index - candidate.smoothing_index) / p.smoothing_index;
            improvement >= dec!(0.05) && candidate.max_station_load_pct < dec!(0.95)
        }
    }
}
```

### D-6. Cycle detection (precedence DAG)

```rust
pub fn assert_acyclic(tasks: &[Task]) -> Result<Vec<TaskId>, LbError> {
    // Kahn's topological sort
    let mut indegree: BTreeMap<TaskId, usize> = tasks.iter().map(|t| (t.task_id.clone(), t.predecessors.len())).collect();
    let mut q: VecDeque<TaskId> = indegree.iter().filter(|(_, &d)| d == 0).map(|(t, _)| t.clone()).collect();
    let mut sorted = Vec::new();
    let succ = build_successor_index(tasks);
    while let Some(t) = q.pop_front() {
        sorted.push(t.clone());
        if let Some(succs) = succ.get(&t) {
            for s in succs {
                if let Some(d) = indegree.get_mut(s) {
                    *d -= 1;
                    if *d == 0 { q.push_back(s.clone()); }
                }
            }
        }
    }
    if sorted.len() != tasks.len() {
        return Err(LbError::TaskGraphHasCycle);
    }
    Ok(sorted)
}
```

### D-7. Cedar context (publish)

```jsonc
{
  "principal": "oyatie::tenant::acme::user::industrial-engineer-3",
  "action":    "production_planning::line_balance::publish",
  "resource":  "production_planning::production_line::LINE-ASSY-01",
  "context": {
    "tenant_id": "acme", "plant_code": "P01",
    "smoothing_index": 18.7, "max_station_load_pct": 0.92,
    "algorithm": "rpw",
    "labour_impact_count": 47,
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
| `production-planning.production-line.balance-proposed.v1` | preview | `dashboards`, `industrial-engineering-review-queue` |
| `production-planning.production-line.balance-published.v1` | publish | `production-order`, `manufacturing-execution-system`, `hr-shift-scheduling`, `dashboards` |
| `production-planning.production-line.balance-rejected.v1` | hysteresis blocked | `analytics` |
| `production-planning.production-line.takt-recomputed.v1` | takt change | `mrp-run`, `s-and-op`, `dashboards` |
| `production-planning.production-line.cycle-violation.v1` | task-graph cycle detected | `alerting`, `industrial-engineering` |

### D-9. Workflow with decision branches

```mermaid
flowchart TB
  A[ComputeLineBalance(line_id, takt, mix, algo)] --> B[Topo-sort tasks (precedence DAG)]
  B --> C{Cyclic?}
  C -- yes --> Z1[TaskGraphHasCycle]
  C -- no --> D{Algorithm}
  D -- rpw --> E1[Compute RPW table]
  D -- lcr --> E2[Largest-candidate rule]
  D -- milp --> E3[GLPK MILP solver]
  D -- sa --> E4[Simulated annealing]
  D -- ga --> E5[Genetic algorithm]
  E1 --> F[Assign tasks to stations]
  E2 --> F
  E3 --> F
  E4 --> F
  E5 --> F
  F --> G[Compute station loads]
  G --> H[Compute smoothing index + max load %]
  H --> I[Persist proposal preview]
  I --> J[Return proposal_id]
  J --> K[PublishLineBalance]
  K --> L{Hysteresis pass?}
  L -- no --> M[Emit balance-rejected.v1; preserve prior]
  L -- yes --> N{Cedar permit?}
  N -- deny --> Z2[PermissionDenied]
  N -- permit --> O[Transition prior -> superseded; transition proposal -> published]
  O --> P[Emit balance-published.v1]
  P --> Q[Audit + commit]
```

### D-10. SLO targets

| Operation | Scale | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|---|
| `ComputeLineBalance` (RPW, 30 tasks, 6 stations) | small | 18 ms | 42 ms | 85 ms | In-process. |
| `ComputeLineBalance` (RPW, 200 tasks, 30 stations) | medium | 280 ms | 620 ms | 1.2 s | O(N²) in tasks. |
| `ComputeLineBalance` (MILP, 50 tasks, 10 stations) | small | 800 ms | 2 s | 5 s | GLPK solver. |
| `ComputeLineBalance` (SA, 200 tasks) | medium | 4 s | 9 s | 18 s | 10k iteration budget. |
| `ComputeLineBalance` (GA, 500 tasks) | large | 35 s | 75 s | 150 s | Population 100, generations 200. |
| `PublishLineBalance` | — | 24 ms | 55 ms | 110 ms | Cedar + DB + outbox + audit. |

### D-11. Audit-event registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-PROPOSED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-PUBLISHED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-REJECTED_HYSTERESIS` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-TAKT_RECOMPUTED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-CYCLE_VIOLATION` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-ERGONOMIC_VIOLATION` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-LINE_BALANCE-PERMISSION_DENIED` | security | usecase |

### D-12. Failure modes & recovery

1. **`TaskGraphHasCycle`** — precedence DAG contains cycle; industrial engineer must correct. Hard error; runbook `runbooks/line-balance-cycle.md`.
2. **`Unassignable`** — task longer than takt OR no station has required capability. Suggest takt relax / station capability addition. Runbook `runbooks/line-balance-unassignable.md`.
3. **`ErgonomicViolation`** — assignment would exceed station ergonomic limit (per ISO 11228). Algorithm backtracks; if no valid assignment, fail with `ErgonomicInfeasible`.
4. **`HysteresisBlockedRebalance`** — improvement < 5%; preserves prior balance, emits rejected event. Recovery: planner increases takt or accepts current.
5. **`MilpSolverTimeout`** — MILP exceeds time budget. Fall back to RPW + emit `solver-fallback.v1`. Runbook `runbooks/lb-solver-timeout.md`.
6. **`PermissionDenied`** — Cedar deny on publish (HR impact too large). Security audit; planner escalates.

### D-13. Migration notes

Source vendor surface: SAP `OPMK` (line maintenance), `MF50` (line loading), `CM33` (capacity overview); SAP DMC line-balancing screens. Greenfield: empty line catalogue. Lift-shift: replay historical balance publishes for audit history.

### D-14. Ontology projection

```rust
pub fn project_line_balance(p: &LineBalanceProposal) -> OntologyDelta {
    let mut d = OntologyDelta::new()
        .upsert_node(NodeRef::line_balance_proposal(p.tenant_id.clone(), p.proposal_id))
        .upsert_edge(Edge::balance_for_line(p.proposal_id, p.line_id.clone()));
    for (station, tasks) in &p.assignments {
        for task in tasks {
            d = d.upsert_edge(Edge::task_assigned_to_station(p.proposal_id, task.clone(), station.clone()));
        }
    }
    d.with_attrs([("smoothing_index", p.smoothing_index), ("takt_seconds", p.takt_seconds), ("algorithm", p.algorithm.to_string())])
     .with_hlc(p.hlc.clone())
}
```

### D-15. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound  | `bom` (IP-001)           | direct repo (line tasks reference BOM components) |
| inbound  | `routing` (IP-004)       | direct repo (line tasks reference routing operations) |
| inbound  | `s-and-op` (IP-019)      | AsyncAPI `s-and-op.plan-approved.v1` (drives takt) |
| inbound  | `capacity-calendar` (IP-009) | gRPC `capacity.v1.GetShiftCalendar` |
| inbound  | `quality-management`     | AsyncAPI ergonomic-incident events |
| outbound | `production-order` (IP-011)  | AsyncAPI `production-line.balance-published.v1` |
| outbound | `manufacturing-execution-system` (IP-024) | AsyncAPI same channel |
| outbound | `hr-shift-scheduling`    | AsyncAPI same channel (labour reassignment) |
| outbound | `dashboards`             | AsyncAPI same channel |

## E. Failure-mode summary

See D-12.

## F. Migration / rollback

Feature flag `production_planning_line_balance_v1`. Disabling halts new publishes; existing active balances remain in use; planners use SAP/MES native balance until flag re-enabled.

## G. References

- Helgeson & Birnie (1961), *Assembly Line Balancing Using the Ranked Positional Weight Technique*, Journal of Industrial Engineering 12(6).
- Scholl & Becker (2006), *State-of-the-art exact and heuristic solution procedures for simple assembly line balancing*, EJOR 168(3).
- Pinedo, *Scheduling: Theory, Algorithms, and Systems* (5th ed., Springer, 2016).
- ISO 11228 (ergonomic limits for manual handling).
- ADR-0105, ADR-0244, ADR-0263, ADR-0294, ADR-0297, ADR-0315.
- SAP S/4HANA `OPMK`, `MF50`, `CM33`; SAP DMC line balancing.
- Benchmarks: SAP PP-REM line balancing | SAP DMC | Oracle Production Scheduling Cloud | Siemens Tecnomatix Plant Simulation | ProModel | FlexSim | Arena.

## H. Out of scope

- BOM (IP-001), routing (IP-004), production-order (IP-011), capacity leveling (IP-021), MES (IP-024), DDMRP (IP-018), S&OP (IP-019).

— end IP-025 —
