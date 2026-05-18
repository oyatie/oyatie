---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-009-tenant-assignment-stack
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, cell-rls-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-cell-tenant-assignment — full BC stack

## Intent

Full BC scaffold for the tenant-assignment hot path: kernel + domain + usecase + api + adapter + adapter-postgres + rest + worker + sdk + app (11 crates).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-tenant-assignment-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}/` | create (11 crates) |
| Catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-cell-tenant-assignment-<layer>
JUSTIFICATION:
- microservice = cell.
- bc-tokens = tenant-assignment (tenant→cell binding hot path).
- layer = <layer>.
- -adapter-postgres backend-qualified per ADR-0105 Amendment 3.
- -sdk: client library with in-process LRU cache + event-driven invalidation.
```

## Code Shape

```rust
// sdk/src/client.rs
pub struct CellAssignmentClient {
    inner: ApiClient,
    cache: Mutex<lru::LruCache<TenantId, (CellAssignment, Instant)>>,
    cache_ttl: Duration,
    event_subscriber: EventSubscriber,
}

impl CellAssignmentClient {
    pub async fn get_assignment(&self, tenant_id: &TenantId) -> Result<CellAssignment, ClientError> {
        if let Some((cached, fetched_at)) = self.cache_lookup(tenant_id) {
            if fetched_at.elapsed() < self.cache_ttl {
                return Ok(cached);
            }
        }
        let assignment = self.inner.fetch(tenant_id).await?;
        self.cache_insert(tenant_id, assignment.clone());
        Ok(assignment)
    }

    pub async fn subscribe_to_changes(&self) -> Result<(), ClientError> {
        // Subscribes to CellAssigned + CellRebalanced events;
        // invalidates own cache on tenant_id match.
        self.event_subscriber.start_invalidation_loop(self.cache.clone()).await
    }
}
```

```rust
// usecase/src/assign_tenant.rs
pub struct AssignTenantUseCase<R, S, E> {
    repo: R,
    scheduler: S,
    events: E,
}

impl<R: CellAssignmentRepository, S: SchedulerClient, E: CellEventEmitter> AssignTenantUseCase<R, S, E> {
    pub async fn execute(&self, req: AssignmentRequest) -> Result<CellAssignment, UsecaseError> {
        // 1. Verify no existing assignment (idempotency)
        if let Some(existing) = self.repo.get(&req.tenant_id).await? {
            return Ok(existing); // idempotent
        }
        // 2. Request placement decision
        let decision = self.scheduler.request_placement(&req).await?;
        // 3. Verify pack match (Postgres RLS will re-check at commit)
        if decision.target_cell.pack != req.pack {
            return Err(UsecaseError::CrossPackRefused);
        }
        // 4. Write assignment
        let assignment = CellAssignment::new(req.tenant_id.clone(), decision.target_cell, req.pack, AssignmentScope::Primary);
        self.repo.insert(&assignment).await?;
        // 5. Emit event
        self.events.emit_cell_assigned(&assignment).await?;
        Ok(assignment)
    }
}
```

## Acceptance Gates

```bash
cargo check --workspace
cargo nextest run -p oya-cell-tenant-assignment-usecase --test assign_tenant
cargo run -p oya-dev-cli -- gate validate cell-rls-conformance
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cell
```

## Test Plan

- Unit: 1 per public type + 1 per use case (happy + 2 sad).
- Integration: assign happy path; cross-pack rejected; idempotent re-assign.
- E2E: hot-path latency drill — p99 ≤ 50 ms on 10k requests.
- Coverage: 95% domain; 90% usecase; 85% adapter; 90% sdk.

## Halt Conditions

- p99 lookup latency > 50 ms — investigate cache or Postgres tuning.
- Cross-pack assignment accepted in test — fix.

## Next IP

[`IP-010-tenant-migration-orchestrator.md`](IP-010-tenant-migration-orchestrator.md)

## References

- `microservices/cell/PRD.md` FR-01, FR-05.
- `microservices/cell/policy/cell-boundary.md`.
- Bominal ADR-0009 + ADR-0019.
