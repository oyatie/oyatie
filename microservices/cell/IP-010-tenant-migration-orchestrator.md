---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-010-tenant-migration-orchestrator
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: Tenant migration orchestrator — end-to-end ≤ 10min p99

## Intent

Implement the 5-phase tenant migration use case (plan + lock → drain → copy → cutover → cleanup) per Bominal ADR-0009 §"Live migration" + `runbooks/tenant-migration.md`. Migration checkpoints persisted; resumable from any phase.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-tenant-assignment-usecase/src/migrate_tenant.rs` | create |
| `microservices/cell/src/crates/oya-cell-tenant-assignment-worker/src/migration_loop.rs` | create |
| Catalog row updates for tenant-assignment-usecase + tenant-assignment-worker | update |

## Code Shape

```rust
// usecase/src/migrate_tenant.rs
pub struct MigrateTenantUseCase<R, M, E> {
    repo: R,
    migration_orchestrator: M,
    events: E,
}

impl<R: CellAssignmentRepository, M: MigrationOrchestrator, E: CellEventEmitter> MigrateTenantUseCase<R, M, E> {
    pub async fn execute(&self, req: MigrationRequest) -> Result<MigrationPlan, UsecaseError> {
        // Phase 1: Plan + Lock (≤ 30s)
        let lock = self.repo.acquire_advisory_lock(&req.tenant_id).await?;
        let plan = self.migration_orchestrator.create_plan(&req).await?;
        self.events.emit_migration_planned(&plan).await?;

        // Phase 2: Drain (≤ 2min)
        self.migration_orchestrator.drain_source(&plan).await?;
        self.events.emit_migration_draining(&plan).await?;

        // Phase 3: Copy (≤ 5min p99)
        self.migration_orchestrator.copy_schema(&plan).await?;
        self.migration_orchestrator.copy_s3_prefix(&plan).await?;
        self.migration_orchestrator.rotate_credentials(&plan).await?;
        self.migration_orchestrator.schedule_target_pods(&plan).await?;
        self.events.emit_migration_copying(&plan).await?;

        // Phase 4: Cutover (≤ 1min)
        // Postgres transaction: row update + RLS re-check at commit
        self.repo.cutover(&plan).await?;
        self.events.emit_cell_rebalanced(&plan).await?;

        // Phase 5: Cleanup (≤ 2min)
        self.migration_orchestrator.cleanup_source(&plan).await?;
        let final_plan = self.repo.mark_migration_complete(&plan.migration_id).await?;
        drop(lock); // advisory lock released

        Ok(final_plan)
    }
}
```

```rust
// worker/src/migration_loop.rs — checkpoint resume logic
pub async fn resume_pending_migrations(deps: &Deps) -> anyhow::Result<()> {
    let pending = deps.repo.find_migrations_in_state(&[
        MigrationState::Planned,
        MigrationState::Draining,
        MigrationState::Copying,
        MigrationState::Cutover,
    ]).await?;
    for plan in pending {
        let usecase = deps.migrate_tenant_usecase.clone();
        tokio::spawn(async move {
            let _ = usecase.resume_from_checkpoint(plan).await;
        });
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cell-tenant-assignment-usecase --test migrate_tenant
cargo nextest run --test e2e_migration_10min_p99
```

## Test Plan

- Unit: 1 per phase (happy + interrupt simulation).
- Integration: full 5-phase migration on test cluster + Postgres.
- E2E: time-bounded — p99 ≤ 10 min across 50 migrations of representative tenant size.
- Race test: concurrent migration attempts on same tenant — advisory lock serializes.
- Cross-pack test: cross-pack migration rejected without SCC; accepted with SCC + 2-person rule.

## Halt Conditions

- p99 > 10 min — investigate phase bottleneck.
- Migration corrupts data (checkpoint replay diverges from expected) — fix.

## Next IP

[`IP-011-host-pool-drain-primitive.md`](IP-011-host-pool-drain-primitive.md)

## References

- Bominal ADR-0009 §"Live migration".
- `microservices/cell/runbooks/tenant-migration.md`.
- `microservices/cell/PRD.md` FR-05.
- CloudNativePG logical-replication.
