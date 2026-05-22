---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-008
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-MRP (MD01/MD02/MD03 use-case orchestration)
tenant_class: substrate
persona: mrp-controller
---

# IP-008: Usecase layer for mrp-run

## A. Intent

The usecase layer wires the pure `MrpRun` domain (IP-002) to ports: BOM lookup, plant-material lookup, demand source resolution, Cedar gating, persistence, outbox dispatch, audit emission. This is where MD01-equivalent total-planning and MD03 single-item multi-level orchestrations live as concrete use-cases.

### A.1 Why usecase orchestration matters here

MRP run inputs come from **4 different bounded contexts**: `production-planning.bom-revision` (this µservice), `production-planning.plant-material`, `sales` (sales orders), `supply-chain-planning` (forecasts). The usecase is the only place where the multi-source read is **atomically snapshotted** at HLC `started_at` so the explosion is deterministic. Without this layer, the algorithm gets stale reads under contention.

## B. Acceptance criteria

- **AC-1:** `StartMrpRunUseCase::execute(input)` atomically snapshots BOM + plant materials + demands at HLC `started_at`.
- **AC-2:** Cedar gate `production_planning::mrp::start` enforced before any read.
- **AC-3:** Idempotency on `(tenant_id, idempotency_key)`.
- **AC-4:** Outbox event `EVT-PRODUCTION_PLANNING-MRP_RUN-STARTED` on accept, `EVT-PRODUCTION_PLANNING-MRP_RUN-COMPLETED` on success, `EVT-PRODUCTION_PLANNING-MRP_RUN-FAILED` on failure.
- **AC-5:** Long-running runs (>30s) push status updates to `mrp-run-progress.v1` AsyncAPI channel.
- **AC-6:** Cancellation supported via `CancelMrpRunUseCase`; in-flight workers honour cancellation token.
- **AC-7:** Scenario isolation: `scenario_id` ≠ None means runs do NOT emit to SCP and are tagged `scenario` in outbox.
- **AC-8:** Default-deny Cedar.

## C. Verification

```bash
cargo test -p oya-production-planning-mrp-usecase -- start_happy_path
cargo test -p oya-production-planning-mrp-usecase -- start_cedar_deny
cargo test -p oya-production-planning-mrp-usecase -- start_atomic_snapshot
cargo test -p oya-production-planning-mrp-usecase -- progress_updates_emitted
cargo test -p oya-production-planning-mrp-usecase -- cancel_in_flight_run
cargo test -p oya-production-planning-mrp-usecase -- idempotent_on_duplicate_key
cargo test -p oya-production-planning-mrp-usecase -- scenario_isolated_from_scp
cargo test -p oya-production-planning-mrp-usecase -- failed_event_on_circular_bom
```

## D. Detailed mechanics

### D-1. Use-case orchestration

```rust
pub struct StartMrpRunUseCase<R, B, P, D, C, O, A> {
    pub run_repo: R,
    pub bom_repo: B,
    pub plant_material_repo: P,
    pub demand_repo: D,
    pub cedar: C,
    pub outbox: O,
    pub audit: A,
}

pub struct StartMrpRunInput {
    pub tenant_id: TenantId,
    pub principal: Principal,
    pub idempotency_key: IdempotencyKey,
    pub material_id: MaterialId,
    pub plant_code: PlantCode,
    pub planning_horizon_days: u16,
    pub lot_size_strategy: LotSizeStrategy,
    pub scenario_id: Option<ScenarioId>,
}

impl<R: MrpRunRepository, B: BomRepository, P: PlantMaterialRepository,
     D: DemandRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter>
    StartMrpRunUseCase<R, B, P, D, C, O, A>
{
    pub async fn execute(&self, input: StartMrpRunInput) -> Result<StartMrpRunOutput, UseCaseError> {
        if let Some(prior) = self.run_repo.find_by_idempotency_key(&input.tenant_id, &input.idempotency_key).await? {
            return Ok(prior.into());
        }
        let decision = self.cedar.evaluate(cedar_req("production_planning::mrp::start", &input)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }

        let started_at = Hlc::now();
        let snapshot = MrpInputSnapshot {
            bom_rev: self.bom_repo.load_active(&input.tenant_id, &input.material_id, &input.plant_code).await?
                .ok_or(UseCaseError::BomNotFound)?,
            plant_materials: self.plant_material_repo.load_all_for_bom(&input.tenant_id, &input.material_id, &input.plant_code).await?,
            demands: self.demand_repo.load_within_horizon(&input.tenant_id, &input.material_id, &input.plant_code, started_at, input.planning_horizon_days).await?,
            started_at,
        };

        let mut run = MrpRun::queue(&input, started_at, decision.decision_id.clone())?;
        let tx = self.run_repo.begin_tx().await?;
        self.run_repo.save_queued(&tx, &run).await?;
        self.outbox.append(&tx, &mrp_started_event(&run, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::from(&decision, &run)).await?;
        tx.commit().await?;

        self.run_repo.enqueue_worker(&run.run_id()).await?;

        Ok(StartMrpRunOutput {
            mrp_run_id: run.run_id(), status: "queued".into(),
            cedar_decision_id: decision.decision_id,
        })
    }
}
```

### D-2. Worker loop (separate process)

```rust
pub struct MrpRunWorker<R, B, P, D, O> { /* ... */ }
impl<R: MrpRunRepository, B, P, D, O: OutboxDispatcher> MrpRunWorker<R, B, P, D, O> {
    pub async fn process_one(&self, run_id: MrpRunId) -> Result<(), WorkerError> {
        let mut run = self.run_repo.load_for_update(&run_id).await?;
        run.mark_running()?;
        let snapshot = self.load_snapshot(&run).await?;
        match run.explode(&snapshot.bom_rev, &snapshot.plant_materials, &snapshot.demands) {
            Ok(()) => {
                run.detect_anomalies();
                run.mark_completed(Hlc::now())?;
                self.run_repo.save_completed(&run).await?;
                self.outbox.append_completed(&run).await?;
            }
            Err(e) => {
                run.mark_failed(e.clone())?;
                self.run_repo.save_failed(&run).await?;
                self.outbox.append_failed(&run, &e).await?;
            }
        }
        Ok(())
    }
}
```

### D-3. Cancellation use-case

```rust
pub struct CancelMrpRunUseCase<R, C, O, A> { /* fields */ }
impl<R: MrpRunRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter>
    CancelMrpRunUseCase<R, C, O, A>
{
    pub async fn execute(&self, run_id: MrpRunId, principal: Principal) -> Result<(), UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req_cancel(&run_id, &principal)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }
        let mut run = self.run_repo.load(&run_id).await?.ok_or(UseCaseError::NotFound)?;
        run.cancel()?; // domain method; only legal from {queued, running}
        let tx = self.run_repo.begin_tx().await?;
        self.run_repo.save(&tx, &run).await?;
        self.outbox.append(&tx, &mrp_cancelled_event(&run, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::from(&decision, &run)).await?;
        tx.commit().await?;
        Ok(())
    }
}
```

### D-4. Progress streaming

Worker emits periodic `mrp-run-progress.v1` rows: `{run_id, percent, current_level, components_processed, eta_seconds}` every 2s while running. Consumed by the UI (Workflow Studio progress widget) and `costing` (early-warning supply-gap detector).

### D-5. Port traits

```rust
#[async_trait]
pub trait MrpRunRepository: Send + Sync {
    async fn begin_tx(&self) -> Result<RepoTx, RepoError>;
    async fn find_by_idempotency_key(&self, tenant_id: &TenantId, key: &IdempotencyKey) -> Result<Option<MrpRunRecord>, RepoError>;
    async fn save_queued(&self, tx: &RepoTx, run: &MrpRun) -> Result<(), RepoError>;
    async fn load_for_update(&self, run_id: &MrpRunId) -> Result<MrpRun, RepoError>;
    async fn save_completed(&self, run: &MrpRun) -> Result<(), RepoError>;
    async fn save_failed(&self, run: &MrpRun) -> Result<(), RepoError>;
    async fn enqueue_worker(&self, run_id: &MrpRunId) -> Result<(), RepoError>;
}
```

### D-6. SLO contribution

- Sync part (validation + queueing): ≤ 80ms P95.
- Worker explode + persist: ≤ 30s P95 for breadth-200 depth-8 BOM.
- Outbox dispatch: ≤ 5s P95.

### D-7. Audit and Cedar

`EVT-PRODUCTION_PLANNING-MRP_RUN-STARTED`, `*-COMPLETED`, `*-FAILED`, `*-CANCELLED` per ADR-0263. Each carries `cedar_decision_id`. Soak ≥ 60s per ADR-0294 for any policy change.

## E. Failure modes & recovery

- **Circular BOM**: domain error surfaces; usecase persists run as `failed`; outbox `MRP-RUN-FAILED` event; runbook `runbooks/mrp-circular-bom.md`.
- **Plant-material missing**: domain error; same path; runbook `runbooks/mrp-plant-material-missing.md`.
- **Repo lock contention** on `save_queued`: serializable retry up to 3 times, then surface 503.
- **Worker crash mid-explosion**: idempotent re-claim via `SELECT FOR UPDATE SKIP LOCKED`; resume from last committed level.
- **Cancellation race**: cancellation token checked at each BFS frontier expansion; race-loser cancellation emits `cancellation_no_op` audit.

## F. Migration

Phase 1: domain (IP-002).
Phase 2: this usecase.
Phase 3 (IP-013): adapter.
Phase 4 (IP-014): REST/gRPC surface.
Phase 5 (IP-016): SCP handoff.

Rollback: feature flag `production_planning_mrp_usecase_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0316.
- SAP Help: PP-MRP MD01/MD02/MD03 batch and dialog modes.
- Benchmarks: SAP S/4HANA PP-MRP | Oracle Fusion MRP | Kinaxis RapidResponse | Blue Yonder Luminate | o9 IBP.

## H. Out-of-scope

- Adapter (IP-013), surface (IP-014), SCP handoff (IP-016), domain (IP-002).

— end IP-008 —
