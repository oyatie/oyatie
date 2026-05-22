---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-012-autonomy-ceiling-gate
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, autonomy-gate-presence, lean-a1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Autonomy tier gate (cross-µservice wiring + LEAN lane)

## Intent

Wire the `AutonomyGate` port end-to-end:
- Tenancy lookup adapter reads tenant ceiling from tenancy µservice (signed) + caches in-process with TTL.
- Per-invocation ceiling comparison invoked as FIRST step in DispatchUseCase per `runtime-isolation.md` TI-08.
- Refusal emits `AutonomyViolationDetected` event with full audit context.
- LEAN lane `oya-check-autonomy-gate-presence` asserts call-graph order (no dispatch crate-path reaches ProviderInvoker without AutonomyGate first).
- Ceiling cache freshness alarm at 5min stale.

## ChangeSet boundary

Modifications to `capability-executor-adapter` (AutonomyGate impl + tenancy client) + capability-executor-usecase (gate-first invariant + violation event emission) + new LEAN check at `crates/oya-governance-check-autonomy-gate-presence/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-adapter/src/autonomy_gate_adapter.rs` | create |
| `.../src/tenancy_client.rs` | create (mTLS client to tenancy µservice) |
| `src/crates/oya-foundry-runtime-capability-executor-usecase/src/dispatch_use_case.rs` | modify (assert gate-first) |
| `src/crates/oya-foundry-runtime-capability-executor-usecase/src/violation_emitter.rs` | create |
| `crates/oya-governance-check-autonomy-gate-presence/Cargo.toml` | create |
| `.../src/lib.rs` | create (AST-based grep for AutonomyGate.check before ProviderInvoker.invoke) |
| `crates/oya-dev-cli/src/governance_gates.rs` | modify (register lane) |
| `/specs/quality/lanes.yaml` | modify (register lane) |

## Crate Naming

```
NAME: oya-governance-check-autonomy-gate-presence
JUSTIFICATION:
- microservice = governance (cross-cutting fitness lane crate; lives under governance per ADR-0131)
- bc-tokens = autonomy-gate-presence
- layer = check (LEAN architecture-conformance check)
- exemptions claimed: none
```

## Code Shape

```rust
// adapter/src/autonomy_gate_adapter.rs
use oya_foundry_runtime_capability_executor_kernel::*;

pub struct AutonomyGateAdapter {
    tenancy_client: TenancyClient,
    cache: Cache<String, (AutonomyLevel, Signature, Instant)>,
    cache_ttl: Duration,
}

#[async_trait]
impl AutonomyGate for AutonomyGateAdapter {
    async fn check(&self, tenant_id: &str, requested: AutonomyLevel) -> Result<AutonomyDecision, AutonomyError> {
        let (ceiling, signature, _) = self.cache.get_or_insert_async(tenant_id.into(), || async {
            let signed = self.tenancy_client.read_ceiling(tenant_id).await?;
            signed.verify_signature(&self.tenancy_pubkey)?;
            Ok::<_, AutonomyError>((signed.ceiling, signed.signature, Instant::now()))
        }).await?;

        if requested > ceiling {
            // Emit AutonomyViolationDetected via downstream channel — invoked by caller
            return Ok(AutonomyDecision::Refuse { ceiling });
        }
        Ok(AutonomyDecision::Permit { ceiling })
    }
}
```

```rust
// crates/oya-governance-check-autonomy-gate-presence/src/lib.rs
pub fn run() -> Result<(), GateError> {
    let crate_root = "microservices/foundry/src/crates/oya-foundry-runtime-capability-executor-usecase";
    let dispatch_use_case = parse_file(format!("{crate_root}/src/dispatch_use_case.rs"))?;
    let gate_invocations = find_method_calls(&dispatch_use_case, "AutonomyGate::check")?;
    let provider_invocations = find_method_calls(&dispatch_use_case, "ProviderInvoker::invoke")?;

    for prov in &provider_invocations {
        let preceding_gate = gate_invocations.iter().find(|g| g.line < prov.line);
        if preceding_gate.is_none() {
            return Err(GateError::DispatchReachesProviderWithoutAutonomyGate {
                file: dispatch_use_case.file.clone(),
                provider_line: prov.line,
            });
        }
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate autonomy-gate-presence --microservice foundry-runtime
cargo nextest run -p oya-foundry-runtime-capability-executor-usecase --test autonomy_gate
cargo nextest run -p oya-foundry-runtime-capability-executor-adapter --test autonomy_gate_adapter --features testcontainers
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_autonomy_gate_first_step_enforced` | LEAN lane fails when ProviderInvoker reachable without AutonomyGate |
| `test_autonomy_refusal_emits_violation_event` | violation event payload correctness |
| `test_ceiling_cache_freshness_under_5min` | cache TTL honoured |
| `test_ceiling_signature_invalid_refuses` | T-T-05 mitigation |
| `test_high_risk_eu_ai_act_requires_notified_body` | Cedar policy enforcement (defence-in-depth with code) |

## Halt Conditions

- LEAN lane greens but dispatch reaches provider without AutonomyGate — refactor (lane bug).
- Ceiling cache TTL > 5min — refactor (stale data risk).

## Next IP

[`IP-013-dsr-cascade-session-handler.md`](IP-013-dsr-cascade-session-handler.md)

## References

- ADR-0022 (autonomy tiers); ADR-0123 (HG-FR registration).
- `policy/runtime-isolation.md` TI-08.
- `threat-model.md` T-E-01.
- `runbooks/autonomy-violation-quarantine.md`.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
