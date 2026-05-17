---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-007-scheduler-binpack
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, statelessness, shardability]
---

# IP-007: oya-cell-scheduler — binpack placement engine

## Intent

Full BC scaffold: kernel + domain (binpack math) + usecase (placement orchestrator) + api + adapter (cluster-state reader) + worker (loop) + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-scheduler-{kernel,domain,usecase,api,adapter,worker,app}/` | create (8 crates total) |
| Catalog rows for each crate | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-cell-scheduler-<layer>
JUSTIFICATION:
- microservice = cell.
- bc-tokens = scheduler (placement decision authority; sibling to cell-registry, tenant-assignment, lifecycle-manager, host-pool).
- layer = <layer> per ADR-0105.
```

## Code Shape

```rust
// domain/src/binpack.rs — pure binpack math
pub fn score_placement(
    candidate: &Cell,
    request: &PlacementRequest,
) -> f64 {
    let util = candidate.capacity_envelope.utilization_pct;
    let target_band_score = if (40.0..=80.0).contains(&util) { 1.0 } else { 0.5 };
    let scope_match_score = if candidate.cell_scope == request.required_scope { 1.0 } else { 0.0 };
    let pack_match_score = if candidate.pack == request.pack { 1.0 } else { 0.0 };
    // Affinity bonus if neighbour-tenants exist (e.g., for HA cohort)
    let affinity_score = compute_affinity_bonus(candidate, request);

    target_band_score * scope_match_score * pack_match_score * (1.0 + 0.2 * affinity_score)
}
```

```rust
// usecase/src/place_tenant.rs
pub struct PlaceTenantUseCase<R, P, E> {
    cell_repo: R,
    policy: P,
    events: E,
}

impl<R: CellRepository, P: PlacementPolicy, E: CellEventEmitter> PlaceTenantUseCase<R, P, E> {
    pub async fn execute(&self, req: PlacementRequest) -> Result<PlacementDecision, UsecaseError> {
        let candidates = self.cell_repo.list_by_pack(req.pack).await?;
        let candidates = candidates.into_iter().filter(|c| c.state == CellState::Ready).collect::<Vec<_>>();
        if candidates.is_empty() {
            return Err(UsecaseError::NoCandidateCells);
        }
        let scored: Vec<(Cell, f64)> = candidates.iter().map(|c| (c.clone(), score_placement(c, &req))).collect();
        let (best_cell, score) = scored.into_iter().max_by(|a,b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or(UsecaseError::NoCandidateCells)?;
        let decision = PlacementDecision {
            tenant_id: req.tenant_id,
            target_cell: best_cell.cell_id,
            binpack_score: score,
            considered_cells: candidates.iter().map(|c| c.cell_id.clone()).collect(),
            decided_at: chrono::Utc::now(),
            signature: sign_decision(&req, &best_cell),
        };
        self.events.emit_placement_decision(&decision).await?;
        Ok(decision)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cell-scheduler-{kernel,domain,usecase,api,adapter,worker,app}
cargo nextest run -p oya-cell-scheduler-domain
cargo run -p oya-dev-cli -- gate validate statelessness --crate oya-cell-scheduler-worker
cargo run -p oya-dev-cli -- gate validate shardability --crate oya-cell-scheduler-worker
```

## Test Plan

- Domain: property tests for binpack math (monotonicity; pack-match dominance).
- Use case: 3+ scenarios (happy; no-candidate; cross-pack request refused).
- Worker: long-lived loop integration test.
- Coverage: 95% domain; 90% usecase; 85% adapter.

## Halt Conditions

- Worker holds in-memory state across restart — fix.
- Cross-pack request not refused — fix.

## Next IP

[`IP-008-lifecycle-manager-k8s.md`](IP-008-lifecycle-manager-k8s.md)

## References

- `microservices/cell/PRD.md` FR-02 + FR-07.
- Bominal ADR-0019.
- Kubernetes scheduler binpack reference.
