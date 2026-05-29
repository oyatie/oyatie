---
doc_class: SDKPlan
title: SDK Plan (Rust + TypeScript + Python)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + axis-developer-experience
deciders: axis-foundry, axis-developer-experience, council-architecture
related_adrs: [ADR-0024, ADR-0056, ADR-0105, ADR-0131]
related_artifacts:
  - microservices/intelligence-eval/contracts/openapi/eval-runner.yaml
  - microservices/intelligence-eval/contracts/proto/eval_runner.proto
  - microservices/intelligence-eval/contracts/asyncapi/eval-events.yaml
review_cadence: per minor SDK release + annually
doc_status: published
---

# SDK Plan (foundry-eval µservice)

## Purpose

Provide first-party SDK clients for capability owners + tenant operators to interact with foundry-eval without re-implementing REST / gRPC / AsyncAPI handlers. Closes the OpenAI-Evals / Anthropic-evals / LangSmith / Braintrust SDK gap.

## Languages + Distribution

| Language | Crate / package | Distribution |
|---|---|---|
| Rust | `oya-foundry-eval-eval-runner-sdk` | crates.io (oyatie-internal) + workspace |
| TypeScript | `@oyatie/foundry-eval-sdk` | npm-internal registry |
| Python | `oyatie-foundry-eval-sdk` | pypi-internal |

## Per-Language Scope

### Rust SDK

The kernel SDK; first-class consumer of the OpenAPI + proto contracts.

```rust
use oya_foundry_eval_eval_runner_sdk::{Client, ProviderRoute, EvalRunTriggerRequest};

let client = Client::builder()
    .endpoint("https://foundry-eval.kr.oyatie.com/v1")
    .with_oidc_token(tok)
    .build()?;

// Read latest publish-gate verdict
let verdict = client.publish_gate_verdict("cap:foundry-eval:eval-run:v1").await?;
assert!(verdict.adversarial_cohort_pass);

// Trigger ad-hoc eval-run
let receipt = client.trigger_eval_run(EvalRunTriggerRequest {
    capability_id: "cap:my-cap:v1".to_string(),
    version: "v3".to_string(),
    route: ProviderRoute {
        provider: "anthropic".to_string(),
        model: "claude-opus-4.7".to_string(),
        variant: None,
    },
    reason: "M02 router-preference candidate; checking parity vs incumbent".to_string(),
    cohorts: vec!["adversarial-prompt-injection".to_string(), "linguistic-kr".to_string()],
}).await?;

// Stream progress
let mut progress = client.stream_eval_run_progress(receipt.run_id).await?;
while let Some(update) = progress.next().await {
    println!("{}/{} cases", update.cases_completed, update.cases_total);
}

// Retrieve EU AI Act §15 evidence
let s15 = client.eu_ai_act_section_15_evidence("cap:my-cap:v1", time_range).await?;
println!("Accuracy: {}; Robustness: {}", s15.accuracy_metric, s15.robustness_metric);
```

### TypeScript SDK

Wraps the Rust SDK via wasm-bindgen for cross-runtime use:

```typescript
import { FoundryEvalClient } from "@oyatie/foundry-eval-sdk";

const client = new FoundryEvalClient({
  endpoint: "https://foundry-eval.kr.oyatie.com/v1",
  oidcToken: tok,
});

const verdict = await client.publishGateVerdict("cap:foundry-eval:eval-run:v1");
console.log(verdict.adversarial_cohort_pass);
```

### Python SDK

Wraps the Rust SDK via PyO3:

```python
from oyatie.foundry_eval import Client, ProviderRoute, EvalRunTriggerRequest

client = Client(endpoint="https://foundry-eval.kr.oyatie.com/v1", oidc_token=tok)

verdict = client.publish_gate_verdict("cap:foundry-eval:eval-run:v1")
assert verdict.adversarial_cohort_pass

receipt = client.trigger_eval_run(EvalRunTriggerRequest(
    capability_id="cap:my-cap:v1",
    version="v3",
    route=ProviderRoute(provider="anthropic", model="claude-opus-4.7"),
    reason="...",
    cohorts=["adversarial-prompt-injection", "linguistic-kr"],
))
```

## Versioning

Semantic versioning per the OpenAPI contract version (1.0.0 → 2.0.0 = breaking; 1.1.0 = additive). Per ADR-0024 + ADR-0131:
- Minor version bumps add fields (forward-compatible).
- Major version bumps break (deprecation period 6 months minimum).
- SDK release cadence: per OpenAPI contract release.

## Authentication

- **OIDC token**: standard bearer; refreshed via SDK helper.
- **mTLS + SPIFFE** for service-account use (e.g., embedded in foundry-runtime, foundry-providers).
- **GitHub Actions OIDC**: for CI runner integration; SDK helper `Client::with_github_oidc()`.

## Error Handling

Typed errors per `errors.rs`:

```rust
pub enum FoundryEvalError {
    NotFound(String),
    Forbidden { policy_decision_id: String },
    RateLimited { retry_after_seconds: u32 },
    ValidationFailed(Vec<ValidationError>),
    UpstreamError { service: String, cause: String },
    Transport(reqwest::Error),
}
```

SDK handles transient errors (429, 503) with exponential backoff + jitter.

## Observability

SDK emits OTel spans for every call:
- Span name: `foundry-eval.<operation>`.
- Span attributes: `capability_id`, `version`, `route.provider`, `route.model`, `oya-pack-id`, `result_status`.
- Per `microservices/observability/` standards.

## Testing

- Per-language integration tests against a local foundry-eval mock (rust-mockito / nock / responses).
- Contract tests against live foundry-eval staging (oya-self pack).
- SDK release-gate: integration tests must pass against latest staging foundry-eval.

## Distribution

- Rust: published to internal crates registry on M01-P01 release.
- TypeScript: npm-internal on M02-P02.
- Python: pypi-internal on M02-P02.

## Migration

When OpenAPI contract bumps, SDK release follows within 5 business days. Migration guide in `docs/sdk-migration/<from>-<to>.md`.

## References

- ADR-0024 §"Eval kernel".
- ADR-0056 + ADR-0105 + ADR-0131 (BNF + layer + layout).
- OpenAPI: `microservices/intelligence-eval/contracts/openapi/eval-runner.yaml`.
- proto: `microservices/intelligence-eval/contracts/proto/eval_runner.proto`.
- AsyncAPI: `microservices/intelligence-eval/contracts/asyncapi/eval-events.yaml`.
- OpenAI Evals SDK precedent: `github.com/openai/evals`.
- LangSmith JS / Python SDK precedent: `docs.langchain.com/langsmith`.
