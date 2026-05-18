---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-003-router-usecase
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

# IP-003: oya-foundry-providers-router-usecase

## Intent

Orchestrator that composes kernel + domain + ports: reads tenant config, computes candidates, scores via domain algebra, emits `RouterDecision`, hands off to invoker.

## ChangeSet boundary

One new crate `microservices/foundry/src/crates/oya-foundry-providers-router-usecase/`. Depends on `oya-foundry-providers-router-kernel` + `oya-foundry-providers-router-domain`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/decide.rs` | create — `Decide` use case |
| `.../src/invoke.rs` | create — `Invoke` use case (decide + dispatch + envelope) |
| `.../src/policies/cedar_eval.rs` | create — Cedar policy evaluation hook |

## Use Case Surface (excerpt)

```rust
pub struct DecideUseCase<R, C, H, T> {
    pub config: R,
    pub credential_resolver: C,
    pub health_monitor: H,
    pub token_bucket: T,
}

impl<R, C, H, T> DecideUseCase<R, C, H, T>
where
    R: ProviderConfigRepository,
    H: HealthMonitor,
    T: TokenBucket,
{
    pub async fn run(&self, req: RoutingRequest) -> Result<RouterDecision, RouterError> {
        let cfg = self.config.load(&req.tenant_id, &req.pack).await?;
        let candidates = self.build_candidates(&cfg, &req).await?;
        let scored = candidates.iter().map(|c| (c, score_candidate(c, &req, ...)))
            .collect::<Vec<_>>();
        let best = scored.into_iter().max_by_key(|(_, s)| s);
        let decision = match best {
            Some((c, _)) if c.eligible => RouterDecision::eligible(c),
            _ => RouterDecision::no_compliant_provider(),
        };
        emit_router_decided_event(&decision).await?;
        Ok(decision)
    }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_decide_happy_path_anthropic_api` | basic flow |
| `test_decide_residency_violation_returns_no_compliant_provider` | T-08 mitigation |
| `test_decide_demoted_vendor_excluded` | T-04 / FM-FP-01 |
| `test_decide_token_bucket_exhausted_returns_429` | T-02 / FM-FP-02 |
| `test_decide_no_in_house_when_tenant_opted_out` | T-04 mitigation |
| `test_decide_emits_router_decided_event` | event emission |
| `test_decide_emits_eu_ai_act_disclosure_when_eu` | EU AI Act Art. 50 |
| `test_invoke_composes_decide_resolve_dispatch_envelope` | flow |
| `test_invoke_rejects_when_cedar_denies` | T-10 mitigation |

## Acceptance Gates

Standard cargo lanes + `port-location` (must NOT impl kernel traits in usecase; usecase consumes traits).

## Next IP

[`IP-004-router-api.md`](IP-004-router-api.md)
