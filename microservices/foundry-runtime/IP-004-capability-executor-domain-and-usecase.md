---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-004-capability-executor-domain-and-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-004: oya-foundry-runtime-capability-executor-{domain,usecase}

## Intent

Pure dispatch math in `-domain` (autonomy comparison; step-state transitions). Orchestrator in `-usecase` (per ADR-0106 replaces 'application' for new code): AutonomyGate → CapabilityResolver → GuardrailChecker (pre) → ProviderInvoker → GuardrailChecker (post) → EvidenceEmitter; emits step events at each transition.

## ChangeSet boundary

Two new Rust crates under `microservices/foundry-runtime/src/crates/`. Consumes kernel. Mock-port integration tests prove orchestration flow without infrastructure.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-domain/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/autonomy_arithmetic.rs` | create |
| `.../src/step_state_machine.rs` | create |
| `src/crates/oya-foundry-runtime-capability-executor-usecase/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/dispatch_use_case.rs` | create |
| `.../src/cancel_use_case.rs` | create |
| `Cargo.toml` (workspace) | update |
| `catalog/oya-foundry-runtime-capability-executor-domain.yaml` | create |
| `catalog/oya-foundry-runtime-capability-executor-usecase.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-runtime-capability-executor-domain
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = capability-executor
- layer = domain (ADR-0105: pure math; depends on kernel only)
- exemptions claimed: none
```

```
NAME: oya-foundry-runtime-capability-executor-usecase
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = capability-executor
- layer = usecase per ADR-0106 (replaces 'application' for new code)
- exemptions claimed: none
```

## Code Shape

```rust
// usecase/src/dispatch_use_case.rs
use oya_foundry_runtime_capability_executor_kernel::*;
use oya_foundry_runtime_capability_executor_domain as dom;

pub struct DispatchUseCase<A, R, G, P, E> {
    autonomy: A,
    resolver: R,
    guardrail: G,
    provider: P,
    evidence: E,
}

impl<A, R, G, P, E> DispatchUseCase<A, R, G, P, E>
where
    A: AutonomyGate, R: CapabilityResolver, G: GuardrailChecker,
    P: ProviderInvoker, E: EvidenceEmitter,
{
    pub async fn run(
        &self,
        tenant_id: &str,
        capability_id: &str,
        input: serde_json::Value,
        requested_tier: AutonomyTier,
    ) -> Result<Invocation, DispatchError> {
        // Step 1: AutonomyGate (per runtime-isolation.md TI-08 — FIRST step)
        match self.autonomy.check(tenant_id, requested_tier).await? {
            AutonomyDecision::Refuse { ceiling } => {
                return Err(DispatchError::AutonomyViolation { ceiling, requested: requested_tier });
            }
            AutonomyDecision::Permit { .. } => {}
        }

        // Step 2: Resolve capability descriptor
        let capability = self.resolver.resolve(tenant_id, capability_id).await?;

        // Step 3: Pre-flight guardrail
        match self.guardrail.check(&input, GuardrailDirection::PreFlight).await? {
            GuardrailVerdict::Block { reason } => {
                return Err(DispatchError::GuardrailBlock { direction: "preflight".into(), reason });
            }
            GuardrailVerdict::Permit => {}
        }

        // Step 4: Provider invocation
        let output = self.provider.invoke(&capability, &input).await?;

        // Step 5: Post-flight guardrail
        match self.guardrail.check(&output, GuardrailDirection::PostFlight).await? {
            GuardrailVerdict::Block { reason } => {
                return Err(DispatchError::GuardrailBlock { direction: "postflight".into(), reason });
            }
            GuardrailVerdict::Permit => {}
        }

        // Step 6: Emit evidence
        let invocation = Invocation { /* ... */ };
        let step = InvocationStep { /* ... */ };
        self.evidence.emit(&step).await?;
        Ok(invocation)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-executor-domain --all-features
cargo nextest run -p oya-foundry-runtime-capability-executor-domain --all-features
cargo check -p oya-foundry-runtime-capability-executor-usecase --all-features
cargo nextest run -p oya-foundry-runtime-capability-executor-usecase --all-features
cargo clippy -p oya-foundry-runtime-capability-executor-usecase -- -D warnings
cargo run -p oya-dev-cli -- gate validate autonomy-gate-presence --crate oya-foundry-runtime-capability-executor-usecase
```

## Test Plan

Per PHASE-01: 1 test per use case (happy + 2 sad paths) + ≥3 mocked-port integration. 90% / 80%.

| Test | Verifies |
|---|---|
| `test_dispatch_happy_path` | mocked ports → Invocation completed |
| `test_dispatch_autonomy_refusal` | requested tier > ceiling → AutonomyViolation error |
| `test_dispatch_preflight_guardrail_block` | guardrail blocks → GuardrailBlock error |
| `test_dispatch_provider_unreachable` | provider returns error → ProviderUnreachable |
| `test_autonomy_arithmetic_monotonic` | tier comparison correctness (domain) |
| `test_step_state_machine_validity` | only valid transitions accepted (domain) |

## Halt Conditions

- Any direct backend call (HTTP, file I/O) in domain or usecase — refactor to adapter.
- AutonomyGate NOT first step in dispatch flow — refactor (runtime-isolation.md TI-08).
- Guardrail not called in both directions — refactor.

## Next IP

[`IP-005-capability-registry-cache-stack.md`](IP-005-capability-registry-cache-stack.md) (cache dependency for resolver port)
[`IP-009-capability-executor-api-and-rest.md`](IP-009-capability-executor-api-and-rest.md) (REST surface)

## References

- ADR-0106 (application→usecase rename).
- `policy/runtime-isolation.md` TI-08.
- `threat-model.md` T-E-01.
