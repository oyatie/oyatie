---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-012-engine-adapter-foundry-runtime
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Engine adapter — foundry-runtime (in-house)

## Intent

`oya-translate-adapter-foundry-runtime` invokes in-house MT, QE, and LangDetect capabilities served on the `foundry-runtime` µservice. Drives the in-house margin per ADR-0026 + cost-budget.md.

## ChangeSet boundary

One new Rust crate at `microservices/translate/src/crates/oya-translate-adapter-foundry-runtime/`. Implements `TranslateInvoker` + `QualityEstimator` + `LanguageDetector` (per kernel ports).

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — depends on kernel + foundry-providers-router-sdk |
| `.../src/lib.rs` | create |
| `.../src/mt_invoker.rs` | create — translate via in-house capability `translate-mt-v1` |
| `.../src/qe_invoker.rs` | create — QE via `translate-qe-comet-kiwi-v1` |
| `.../src/langdetect_invoker.rs` | create — `translate-langdetect-fasttext-v1` |
| `.../src/streaming.rs` | create — streaming inference for real-time stream |
| `.../src/envelope.rs` | create — BLAKE3 + Ed25519 |
| `.../src/response_validator.rs` | create — response shape conformance |

## Code Shape

```rust
pub struct FoundryRuntimeAdapter {
    pub provider_invoker: ProviderInvoker,    // foundry-providers ProviderInvoker
    pub signing_key: ed25519_dalek::SigningKey,
    pub event_emitter: EventEmitter,
}

#[async_trait]
impl TranslateInvoker for FoundryRuntimeAdapter {
    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, RouterError> {
        let provider_req = build_translate_capability_invoke(req);
        let response = self.provider_invoker.invoke(provider_req).await?;
        let translated = parse_translate_response(&response)?;
        response_validator::check_shape(&translated)?;
        let envelope = envelope::sign(
            &self.signing_key,
            &translated.request_hash,
            &translated.response_hash,
            &metadata(req, Vendor::InHouse),
        );
        let evidence_ref = self.event_emitter.emit_translation_completed(/* ... */).await?;
        Ok(TranslationResult {
            engine: Vendor::InHouse,
            envelope_signature: envelope,
            evidence_ref,
            residency_compliant: true,    // in-house is always in-region
            ..translated
        })
    }
}
```

## Per-Capability Calls

| Port | Capability id | Streaming? |
|---|---|---|
| `TranslateInvoker::translate` | `translate-mt-v1` | yes |
| `QualityEstimator::score` | `translate-qe-comet-kiwi-v1` | no |
| `LanguageDetector::detect` | `translate-langdetect-fasttext-v1` | no |
| Real-time stream chunk translate | `translate-mt-streaming-v1` | yes |

## Test Plan

| Test | Verifies |
|---|---|
| `test_translate_response_shape_canonical` | response normalization |
| `test_envelope_sign_verify_roundtrip` | crypto |
| `test_streaming_chunk_ordering_preserved` | stream invariant |
| `tests/integration/foundry_runtime_translate_e2e.rs` | end-to-end via foundry-runtime mock |
| `tests/integration/in_house_is_always_residency_compliant.rs` | invariant |

## Halt Conditions

- Adapter calls a non-foundry-runtime endpoint (must go through provider-invoker).
- Adapter mistakenly serializes a credential (none expected; foundry-runtime is in-cluster mTLS).
- `residency_compliant` set to false (in-house should always be in-region).

## Next IP

[`IP-013-engine-adapters-external.md`](IP-013-engine-adapters-external.md)
