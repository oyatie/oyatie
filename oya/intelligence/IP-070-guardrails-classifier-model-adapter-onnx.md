---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-010-classifier-model-adapter-onnx
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, classifier-model-cosign-signed]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-foundry-guardrails-prompt-classifier-adapter-classifier-model (ONNX runtime client)

## Intent

Backend-qualified adapter implementing `ClassifierModelServer` against the in-cluster ONNX-runtime classifier-model-serving deployment (IP-002). Shared between prompt-classifier and jailbreak-detector kernels (the classifier-model serving infra is multi-model; one adapter is sufficient).

## ChangeSet boundary

One crate `oya-foundry-guardrails-prompt-classifier-adapter-classifier-model`; the jailbreak-detector's `-adapter-classifier-model` (IP-008) re-uses via direct dependency. Per-model version pinning enforced.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-prompt-classifier-adapter-classifier-model/Cargo.toml` | create |
| `.../src/{lib.rs,client.rs,model_registry.rs,integrity_check.rs}` | create |
| `Cargo.toml` workspace | update |
| `catalog/oya-foundry-guardrails-prompt-classifier-adapter-classifier-model.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-guardrails-prompt-classifier-adapter-classifier-model
JUSTIFICATION: microservice=foundry-guardrails; bc=prompt-classifier; layer=adapter; backend=classifier-model per ADR-0105 §"Amendment 3"
```

## Code Shape

```rust
pub struct ClassifierModelClient {
    http: reqwest::Client,
    base_url: url::Url,
    model_registry: ModelRegistry,  // maps model_id → pinned version + Cosign-verified SHA
}

#[async_trait]
impl ClassifierModelServer for ClassifierModelClient {
    async fn infer(&self, model_id: &str, input: &[u8]) -> Result<Vec<f32>, KernelError> {
        let model = self.model_registry.get(model_id)?;
        // Cosign signature pre-verified at pod start (init container); runtime re-checks SHA matches expected
        let resp = self.http
            .post(self.base_url.join(&format!("/v1/models/{}/infer", model.id))?)
            .header("X-Model-Sha", &model.sha)  // server side double-checks
            .body(input.to_vec())
            .send().await?;
        // ... parse response; emit per-inference observability metric
    }
    fn model_version(&self, model_id: &str) -> Result<ClassifierModelVersion, KernelError> {
        self.model_registry.get(model_id).map(|m| m.into())
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-prompt-classifier-adapter-classifier-model --all-features
cargo nextest run -p ...adapter-classifier-model --all-features --test onnx_integration
buck2 build //:quality-lane-registry-authority-check # lane=classifier-model-cosign-signed
```

## Test Plan

- 1 test per port-impl method.
- ≥ 2 against real ONNX-runtime testcontainer.
- Negative: tampered model SHA → request refused.

## Halt Conditions

- Model SHA mismatch detected at runtime — alert; refuse use.
- Per-inference latency p99 > 80ms on baseline model — escalate.

## Next IP

[`IP-011-rest-and-grpc-surface.md`](IP-011-rest-and-grpc-surface.md)

## References

- IP-002 (classifier-model-serving IaC).
- IP-004 (prompt-classifier-kernel port).
- ONNX Runtime — `onnxruntime.ai`.
- Cosign — `docs.sigstore.dev/cosign/`.

## Wave 15 counterpart anchor

- Counterparts: AWS Bedrock Guardrails, OpenAI Moderation, Anthropic safety tooling, and NVIDIA NeMo Guardrails.
- Gap closure: this IP closes inline prompt, output, autonomy, jailbreak, and false-positive-budget enforcement before tenant-visible release.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
