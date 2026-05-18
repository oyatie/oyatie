---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-013-dsr-cascade-session-handler
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, dsr-cascade-coverage]
---

# IP-013: TenantDsrCascade event consumer in session-state worker

## Intent

Implement DSR cascade in the session-state app: subscribe to `TenantDsrCascade` events from the tenancy µservice; scan Redis per-tenant prefix + Postgres session_mutation_log for affected subject identifiers; soft-delete affected session fragments with 30d grace; hard-delete after grace; emit `dsr_executed` audit-chain event. Per DPIA R-08 + R-12 mitigations + `policy/data-residency.md` §"DSR Cascade".

## ChangeSet boundary

New worker module in `session-state-app`. Postgres schema add for `dsr_cascade_log` table tracking each cascade.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-session-state-app/src/dsr_cascade_worker.rs` | create |
| `src/crates/oya-foundry-runtime-session-state-usecase/src/dsr_cascade_use_case.rs` | create |
| `iac/postgres-schema/006-dsr-cascade-log.sql` | create |

## Code Shape

```rust
// usecase/src/dsr_cascade_use_case.rs
pub struct DsrCascadeUseCase<S, L, A> {
    session_store: S,
    mutation_log: L,
    audit_chain: A,
}

impl<S: SessionStore, L: SessionMutationLog, A: AuditChainEmitter> DsrCascadeUseCase<S, L, A> {
    pub async fn handle(&self, event: TenantDsrCascadeEvent) -> Result<DsrReport, DsrError> {
        // 1. Identify affected sessions in Redis per-tenant prefix
        let redis_keys = self.session_store.scan_with_subject_hash(
            &event.tenant_id, &event.subject_hash,
        ).await?;

        // 2. Identify Postgres-side cold sessions
        let postgres_rows = self.mutation_log.query_with_subject_hash(
            &event.tenant_id, &event.subject_hash,
        ).await?;

        // 3. Soft-delete with 30d grace
        for key in &redis_keys {
            self.session_store.soft_delete(&event.tenant_id, key).await?;
        }
        for row in &postgres_rows {
            self.mutation_log.soft_delete(row.session_id.as_str()).await?;
        }

        // 4. Emit dsr_executed audit-chain event
        let report = DsrReport {
            tenant_id: event.tenant_id,
            subject_hash: event.subject_hash,
            redis_sessions_marked: redis_keys.len(),
            postgres_records_marked: postgres_rows.len(),
            grace_expires_at: chrono::Utc::now() + chrono::Duration::days(30),
            executed_at: chrono::Utc::now(),
        };
        self.audit_chain.emit("dsr_executed", &report).await?;
        Ok(report)
    }
}

// worker scheduler: nightly job to hard-delete past-grace sessions
pub struct DsrHardDeleteScheduler { ... }
impl DsrHardDeleteScheduler {
    pub async fn run_forever(&self) -> Result<(), WorkerError> {
        loop {
            let past_grace = self.mutation_log.find_past_grace().await?;
            for row in past_grace {
                self.mutation_log.hard_delete(row.session_id.as_str()).await?;
                self.audit_chain.emit("dsr_hard_deleted", &row).await?;
            }
            tokio::time::sleep(Duration::from_secs(3600)).await; // hourly sweep
        }
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-foundry-runtime-session-state-usecase --test dsr_cascade
cargo nextest run -p oya-foundry-runtime-session-state-app --test dsr_cascade_worker --features testcontainers
cargo run -p oya-dev-cli -- gate validate dsr-cascade-coverage --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_dsr_cascade_marks_affected_sessions` | Redis SCAN + Postgres query identify subject_hash rows |
| `test_dsr_cascade_soft_delete_30d_grace` | grace_expires_at correctly set |
| `test_dsr_hard_delete_after_grace` | scheduler hard-deletes past-grace rows |
| `test_dsr_executed_audit_chain_emitted` | event reaches audit-chain |
| `test_dsr_cross_pack_isolation` | cross-pack subject_hash not scanned (data residency invariant) |
| `test_dsr_idempotent_re_run` | re-running same event does not double-emit |

## Halt Conditions

- DSR cascade scans cross-pack — refactor (residency violation).
- Hard-delete bypasses 30d grace — refactor.
- Audit-chain event missing required fields — refactor.

## Next IP

[`IP-014-runtime-self-slo-manifests.md`](IP-014-runtime-self-slo-manifests.md)

## References

- DPIA R-08 (DSR incompleteness); R-12 (children's data DPDPA §9).
- `policy/data-residency.md` §"DSR Cascade".
- GDPR Art. 17; PIPA Art. 36; DPDPA §12; LGPD Art. 18(V)-(VI).
- tenancy µservice (event source).
