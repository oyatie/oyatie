---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-006-capability-deployment
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-check-capability-yaml-conformance]
depends_on: [IP-003, IP-005]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: capability-deployment BC (10 crates)

## Intent

Scaffold the full capability-deployment BC: admit-loop, canary rollout (1 % → 10 % → 50 % → 100 %), phase advance gated by `observability.EligibilityChanged`, automated rollback on breach. 10 crates: kernel, domain, usecase, api, adapter, adapter-postgres, rest, worker, sdk, app.

## Concrete File Targets

Crates at `microservices/intelligence/src/crates/oya-foundry-supervisor-capability-deployment-{layer}/`.

Catalog rows per crate.

## Key code

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait CapabilityDefinitionStore: Send + Sync + Sealed {
    async fn admit(&self, def: CapabilityDefinition) -> Result<DeploymentId, RepositoryError>;
    async fn get(&self, tenant: &TenantId, capability: &CapabilityId) -> Result<CapabilityDefinition, RepositoryError>;
}

#[async_trait]
pub trait RolloutVerdictEmitter: Send + Sync + Sealed {
    async fn emit_phase_transition(&self, deployment: &DeploymentId, from: RolloutPhase, to: RolloutPhase, verdict: Verdict) -> Result<(), KernelError>;
}

// usecase/src/canary_rollout.rs
pub async fn advance_canary_phase(
    deployment: &DeploymentId,
    repo: &dyn CapabilityDefinitionStore,
    observability: &dyn ObservabilityVerdictReader,  // ADR-0139 EligibilityChanged consumer
    emitter: &dyn RolloutVerdictEmitter,
) -> Result<RolloutPhase, KernelError> {
    // 1. Read current phase
    // 2. Compute observe-window (per phase: 1pct=5min, 10pct=10min, 50pct=20min)
    // 3. Read observability EligibilityChanged verdict for the deployment
    // 4. If verdict=eligible, advance phase
    // 5. If verdict=held, stay (emit RolloutPhaseHeld)
    // 6. If verdict=rollback, trigger rollback (separate use-case)
    // 7. Emit CapabilityDeployed event
}
```

## Acceptance Gates

```bash
cargo check / build / clippy / nextest per crate
cargo run -p oya-dev-cli -- gate validate capability-yaml-conformance --microservice foundry-supervisor
```

## Test Plan

| BC slice | Min unit | Min integration | Min e2e |
|---|---|---|---|
| kernel | 1 per public type | 0 | 0 |
| domain | 1 per fn + property | 0 | 0 |
| usecase | happy + 2 sad per use-case | ≥ 3 mocks | 0 |
| adapter | 1 per port-impl | ≥ 2 testcontainers Postgres | 0 |
| adapter-postgres | 1 per query | ≥ 2 testcontainers | 0 |
| rest | 1 per route + auth-fail + cedar-deny | ≥ 2 | 1 per route |
| worker | 1 per arm | ≥ 1 long-loop | 1 e2e (AC-03 canary gated) |
| sdk | 1 per method | ≥ 2 rest | 0 |
| app | smoke | 0 | 1 startup |

## Halt Conditions

- Phase advance bypasses observability verdict.
- Schema-violating capability admitted.

## Next IP

[`IP-007-supervision-event-bus.md`](IP-007-supervision-event-bus.md)

## References

- ADR-0139 §"Canary observability rollback" (precedent).
- PRD FR-01..FR-03.
- `microservices/observability/contracts/asyncapi/eligibility-events.yaml` (consumed).

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
