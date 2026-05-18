---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-003-tenant-lifecycle-domain
status: pending
owner: axis-tenancy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-tenancy-tenant-lifecycle-domain

## Intent

`oya-tenancy-tenant-lifecycle-domain` crate: lifecycle FSM (Created → Activated → Suspended/Resumed → DeletionRequested → Deleted with soft-delete grace), plan-tier rules, jurisdiction validators. Pure logic; no I/O.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain/Cargo.toml` | create |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain/src/lib.rs` | create |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain/src/fsm.rs` | create — state machine |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain/src/plan_tier.rs` | create |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain/src/jurisdiction_validator.rs` | create |
| `microservices/tenancy/catalog/oya-tenancy-tenant-lifecycle-domain.yaml` | create |

## Code Shape

```rust
// src/fsm.rs
pub fn next_state(current: TenantStatus, transition: Transition) -> Result<TenantStatus, FsmError> {
    use TenantStatus::*;
    use Transition::*;
    Ok(match (current, transition) {
        (Created, Activate) => Activated,
        (Activated, Suspend) => Suspended,
        (Suspended, Resume) => Activated,
        (Activated, RequestDeletion) | (Suspended, RequestDeletion) => DeletionRequested,
        (DeletionRequested, CompleteDeletion) => Deleted,
        (DeletionRequested, AbortDeletion) => Suspended,  // soft-delete grace within window
        _ => return Err(FsmError::InvalidTransition(current, transition)),
    })
}
```

## Acceptance Gates

```bash
cargo check -p oya-tenancy-tenant-lifecycle-domain --all-features
cargo nextest run -p oya-tenancy-tenant-lifecycle-domain --all-features
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-tenancy-tenant-lifecycle-domain
```

## Test Plan

Per PHASE-01 domain class: 1 test per public function + property tests for FSM. Coverage 95% line / 90% branch.

## Next IP

[`IP-004-tenant-lifecycle-usecase.md`](IP-004-tenant-lifecycle-usecase.md)

## References

- Bominal ADR-0018.
