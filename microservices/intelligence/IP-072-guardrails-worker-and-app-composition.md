---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-012-worker-and-app-composition
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: -worker + -app crates (composition roots)

## Intent

Per-BC `-worker` crates (long-lived service: rule-cache hot-reload watching Postgres NOTIFY; shadow-mode runner; classifier-model registry refresh) + consolidated `-app` composition root binary wiring rest + worker + adapters across all 6 BCs.

## ChangeSet boundary

Six `-worker` crates + one consolidated `-app` per Foundry-cluster convention. Workers run continuously; apps are k8s Deployments.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-<bc>-worker/Cargo.toml` + `src/{lib.rs,main.rs}` (× 6 BCs) | create |
| `src/crates/oya-foundry-guardrails-prompt-classifier-app/Cargo.toml` + `src/main.rs` | create (the canonical composition root; others share state via mod path) |
| Per-BC app crates for separable deployments where required | create |
| `Cargo.toml` workspace | update |
| `catalog/oya-foundry-guardrails-<bc>-{worker,app}.yaml` | create |

## Code Shape

```rust
// app/src/main.rs (sketch)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let cedar_engine = CedarEngine::new(&config.cedar)?;
    let postgres = PostgresPool::connect(&config.postgres).await?;
    let classifier_client = ClassifierModelClient::new(&config.classifier_endpoint)?;
    let llm_judge = LlmJudgeViaProviders::new(&config.providers)?;
    let decision_emitter = AsyncApiEmitter::new(&config.async_api)?;
    
    let prompt_classifier_state = build_prompt_classifier_state(
        cedar_engine.clone(),
        classifier_client.clone(),
        decision_emitter.clone(),
    )?;
    // ... wire other BCs
    
    let router = Router::new()
        .nest("/prompt-classifier", prompt_classifier_rest::router())
        .nest("/output-validator", output_validator_rest::router())
        .nest("/autonomy-ceiling-gate", autonomy_level_gate_rest::router())
        .nest("/content-safety", content_safety_rule_engine_rest::router())
        .nest("/jailbreak-detector", jailbreak_detector_rest::router())
        .nest("/ai-slop-detector", ai_slop_detector_rest::router())
        .with_state(state);

    // Spawn workers
    tokio::spawn(rule_cache_reload_worker(postgres.clone()));
    tokio::spawn(shadow_mode_runner_worker(...));
    tokio::spawn(classifier_registry_refresh_worker(...));

    axum::serve(listener, router).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-<bc>-worker (× 6) --all-features
cargo check -p oya-foundry-guardrails-prompt-classifier-app --all-features
cargo nextest run -p oya-foundry-guardrails-<bc>-worker (× 6) --all-features
cargo nextest run -p oya-foundry-guardrails-prompt-classifier-app --all-features --test startup_smoke
```

## Test Plan

Per worker class: 1 per arm + ≥ 1 long-lived integration test + 1 e2e.
Per app class: composition-root smoke + 1 startup-and-shutdown.

## Halt Conditions

- Worker not graceful-shutdown-safe — refuse merge.
- App startup without health-probe alignment with k8s readiness — refuse merge.

## Next IP

[`IP-013-runtime-guardrails-coupling-lane.md`](IP-013-runtime-guardrails-coupling-lane.md)

## References

- ADR-0056, ADR-0105.
- IP-011 (rest surface).

## Wave 15 counterpart anchor

- Counterparts: AWS Bedrock Guardrails, OpenAI Moderation, Anthropic safety tooling, and NVIDIA NeMo Guardrails.
- Gap closure: this IP closes inline prompt, output, autonomy, jailbreak, and false-positive-budget enforcement before tenant-visible release.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
