---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-005-autonomy-policy-enforcement
status: pending
execution_unit: ChangeSet
owner: ops-security + axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-check-cedar-fragment-coverage]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: Autonomy policy enforcement (kernel + domain + usecase + adapter + rest + sdk + app)

## Intent

Scaffold the full autonomy-policy-enforcement BC (8 crates: kernel, domain, usecase, api, adapter, rest, sdk, app). Cedar v4 evaluator + tenant-entitlement store; default-deny; per-invocation precondition check called by foundry-runtime.

## Concrete File Targets

Crates created at `microservices/intelligence/src/crates/oya-foundry-supervisor-autonomy-policy-enforcement-{kernel,domain,usecase,api,adapter,rest,sdk,app}/`.

Catalog rows at `microservices/intelligence/catalog/oya-foundry-supervisor-autonomy-policy-enforcement-{layer}.yaml` per crate.

## Key code

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait AutonomyEntitlementStore: Send + Sync + Sealed {
    async fn lookup(&self, tenant: &TenantId, capability: Option<&CapabilityId>) -> Result<Vec<AutonomyEntitlement>, RepositoryError>;
}

#[async_trait]
pub trait CedarEvaluator: Send + Sync + Sealed {
    async fn evaluate(&self, principal: &Principal, action: &Action, resource: &Resource, context: &Context) -> Result<PolicyDecision, KernelError>;
}

// usecase/src/precondition.rs
pub async fn evaluate_precondition(
    tenant: &TenantId,
    capability: &CapabilityId,
    requested_autonomy_level: AutonomyLevel,
    cedar: &dyn CedarEvaluator,
    entitlements: &dyn AutonomyEntitlementStore,
    kill_switch: &dyn KillSwitchStateStore,  // from kill-switch BC
) -> Result<AutonomyDecision, KernelError> {
    // 1. Read tenant entitlements
    // 2. Read kill-switch state for any matching scope
    // 3. Build Cedar context with all attributes
    // 4. Cedar evaluation (default-deny per tenant-scope.cedar PERMIT 4)
    // 5. Return permit/deny + reason + eval_ms
    ...
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-supervisor-autonomy-policy-enforcement-{kernel,domain,usecase,api,adapter,rest,sdk,app} --all-features
cargo build (each)
cargo clippy (each) -- -D warnings
cargo nextest run (each)
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-supervisor
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice foundry-supervisor
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold":
- kernel: 1 per public type + 1 per port; 90% line.
- domain: pure tier-comparison + entitlement-resolution; 95% line + property tests.
- usecase: precondition happy + 3 sad paths (no entitlement, kill-switch engaged, tier escalation); ≥ 3 integration tests against mocks; 90% line.
- adapter: Cedar v4 runtime + OpenBao tenant-resolver against testcontainers; ≥ 2 integration tests; 85% line.
- rest: precondition endpoint + auth-fail + cedar-deny; ≥ 2 cross-route + 1 e2e; 85% line.
- sdk: 1 per method; ≥ 2 against rest; 90% line.
- app: composition smoke; 60% line.

Key drill: AC-04 tier-escalation refusal — emit event chain validates `AutonomyViolated` audit-chain.

## Halt Conditions

- Default-deny missing.
- Cedar evaluation exceeds 50ms p99 in test.

## Next IP

[`IP-006-capability-deployment.md`](IP-006-capability-deployment.md)

## References

- PRD §"Bounded Contexts" + §"Performance".
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement).
- `policy/tenant-scope.cedar` PERMIT 4.
- `threat-model.md` T-E-01 + T-T-05.
- Cedar v4 — `cedarpolicy.com`.

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
