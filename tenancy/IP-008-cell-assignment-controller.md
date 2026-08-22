---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-008-cell-assignment-controller
status: pending
owner: axis-tenancy + ops-sre-reliability
acceptance_lanes: [cargo-check, cargo-nextest, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: Cell-assignment controller (Citus + Patroni-aware)

## Intent

Build `tenancy-cell-assignment-{kernel,domain,usecase,adapter,adapter-citus,worker,app}` crates: consistent-hash shard-key derivation; cell-health probe loop (1s cadence); least-loaded cell selection; Citus pg_dist_shard rebalance orchestrator; integrity checksum before/after.

## Concrete File Targets

| Path | Action |
|---|---|
| `tenancy-cell-assignment-kernel/` | create — CellId, ShardKey, CellHealth, RebalanceTask entities + ports |
| `tenancy-cell-assignment-domain/` | create — consistent-hash derivation; least-loaded; rebalance plan |
| `tenancy-cell-assignment-usecase/` | create — assign + rebalance orchestrators |
| `tenancy-cell-assignment-adapter/` | create — Valkey cache for hot reads |
| `tenancy-cell-assignment-adapter-citus/` | create — pg_dist_shard writes; citus_move_shard_placement |
| `tenancy-cell-assignment-worker/` | create — health probe loop + rebalance scheduler |

## Code Shape

```rust
// domain/src/consistent_hash.rs
pub fn derive_shard_key(tenant_id: &TenantId, shard_count: u64) -> ShardKey {
    let h = blake3::hash(tenant_id.as_bytes());
    let key = u64::from_le_bytes(h.as_bytes()[..8].try_into().unwrap()) % shard_count;
    ShardKey(key)
}
```

```rust
// worker/src/health_probe.rs
pub async fn probe_loop(deps: Deps) -> anyhow::Result<()> {
    loop {
        let cells = deps.cell_store.list_active().await?;
        for cell in cells {
            let health = deps.health_probe.probe(&cell).await;
            deps.cell_store.update_health(&cell.id, health).await?;
            if health == CellHealth::Unhealthy {
                deps.event_sink.emit(CellHealthAlarm::from(&cell)).await?;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

```rust
// adapter-citus/src/rebalance.rs
pub async fn rebalance_shard(deps: &Deps, shard_id: i64, source: &str, target: &str) -> Result<()> {
    let pre_checksum = deps.pg.compute_shard_checksum(shard_id).await?;
    deps.pg.execute(format!("SELECT citus_move_shard_placement({shard_id}, '{source}', '{target}')")).await?;
    let post_checksum = deps.pg.compute_shard_checksum(shard_id).await?;
    if pre_checksum != post_checksum {
        return Err(RebalanceError::IntegrityCheckFailed { shard_id });
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p tenancy-cell-assignment-worker --test rebalance_on_unhealthy
cargo nextest run -p tenancy-cell-assignment-adapter-citus --test rebalance_integrity
```

## Test Plan

- Consistent-hash property tests: same tenant_id → same shard_key across runs.
- Least-loaded test: given N cells with load percentages, picks lowest.
- Rebalance integrity: pre/post checksum match.
- Health-probe loop: induced unhealthy cell triggers CellHealthAlarm + rebalance.

## Next IP

[`IP-009-dsr-cascade-runner.md`](IP-009-dsr-cascade-runner.md)
