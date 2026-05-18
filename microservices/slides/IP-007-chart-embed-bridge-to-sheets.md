---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-007-chart-embed-bridge-to-sheets
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + sheets-team
acceptance_lanes: [cargo-check, cargo-nextest, chart-revocation-cascade-bounded]
depends_on: [IP-002, sheets-IP-X]  # sheets µservice prerequisite
---

# IP-007: chart BC with live-link to sheets µservice

## Intent

Author chart BC per ADR-SLIDES-0008: eventual consistency with on-open / scheduled refresh; revocation cascade ≤ 5s on sheets ACL revoke.

## ChangeSet boundary

6 crates:
- `oya-slides-chart-{kernel,domain,usecase,api,adapter,sdk}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-chart-{kernel,domain,usecase,api,adapter,sdk}/...` | create |
| `tests/integration/chart-live-link.rs` | create |
| `tests/integration/chart-revocation-cascade.rs` | create |

## Code Shape

`chart-kernel/src/ports.rs`:

```rust
pub trait ChartLiveLinkProvider {
    async fn bind(&self, req: &ChartBindRequest) -> Result<ChartBinding, BindError>;
    async fn refresh(&self, binding: &ChartBinding) -> Result<ChartRender, RefreshError>;
    async fn observe_sheets_events(&self) -> Result<EventStream, ObserveError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-chart-domain --test bind
cargo nextest run -p oya-slides-chart-adapter --test refresh_eventual
cargo nextest run --test chart_revocation_cascade -- --include-ignored
oya gate validate chart-revocation-cascade-bounded --microservice slides
```

## Halt Conditions

- Cross-pack bind succeeds — STOP. Residency invariant violated.
- Revocation cascade > 5s p95 — STOP. AC-19 invariant.

## Next IP

IP-008.
