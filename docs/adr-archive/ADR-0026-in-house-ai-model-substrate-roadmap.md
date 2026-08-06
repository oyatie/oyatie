---
id: ADR-0026
status: Superseded
superseded_by: [ADR-0701]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** In-house model substrate roadmap; align naming with intelligence

# ADR-0026: In-house AI model substrate — long-horizon W-AI-Model-Substrate; consume providers until per-vertical eval set favors in-house

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter — the trait extends to in-house IDs), ADR-0024 (eval harness — the cutover gate), ADR-0025 (engineering platform — supplies the data and gate substrate), ADR-0027 (vision/speech/robotics sub-substrates — built on the same kernel)

---

## Context

We do not aspire to be a frontier-LLM lab. The forces that pull us toward in-house models are different: per-vertical accuracy where regional and domain language matters more than generalist reasoning (Korean legal corpus, Korean clinical text, Japanese employment law), residency where customer data cannot leave the region, cost where high-volume inference of small specialized tasks (embedding, OCR, STT/TTS, doc layout, vertical safety classification) dwarfs the cost of frontier general inference, and concentration risk where Anthropic / OpenAI / Google outages or pricing shocks would propagate to every Oyatie surface.

The forces against are also clear: training and serving foundation models is capital-intensive; the talent market is brutal; the iteration cycle of frontier vendors is faster than ours can be. The pragmatic path is therefore not "build a frontier lab" but "build a per-task in-house substrate that consumes provider models for everything until and unless an in-house variant beats the provider on a per-vertical eval set — and then cut over." The same eval harness (ADR-0024) and provider adapter trait (ADR-0020) make the cutover structurally trivial.

---

## Decision

We commit to a long-horizon **W-AI-Model-Substrate** wave that produces in-house production model training and inference for Oyatie-specific tasks. We are not a frontier-LLM lab; we consume Anthropic / OpenAI / Gemini until the in-house variant beats the per-vertical eval set per ADR-0024. The `ProviderAdapter` trait extends to `oya-internal-<model-id>` so the cutover is one router preference change.

### In-scope model families

```rust
// crates/oya-intelligence-model-kernel/src/family.rs
pub enum ModelFamily {
    KrFirstFoundationLlm,        // Korean-first foundation; small/medium parameter counts; vertical-tuned
    EmbeddingForRagAndSearch,    // Per-locale embedding models for Search RAG (ADR-0021 RAG endpoint)
    Stt(LocaleSet),              // Speech-to-text per pack: KR + JP + EN + ES + PT + HI + AR
    Tts(LocaleSet),              // Text-to-speech same locales
    VisionOcr,                   // Document OCR + handwriting + receipt + form
    VisionDocUnderstanding,      // Layout + entity extraction + table extraction
    VerticalSafetyClassifier,    // Per-vertical safety: PHI redaction, PII redaction, fraud-pattern
    VerticalEvalScorer,          // Eval-substrate scoring models (ADR-0024 HumanJudged proxy)
}
```

### Out-of-scope

- A frontier general-purpose chat model. We do not compete with Claude / GPT / Gemini on general reasoning.
- A standalone "Oyatie LLM" sold as a product to non-Oyatie tenants in W-Public-GA. The in-house substrate is a Foundry-internal capability surface; external tenants get it through the capability registry, not as a raw model API.
- Defense / weapons-adjacent fine-tunes of any model — anti-scope per ADR-0027 binding.

### Cutover gate (`ADR-0024` integration)

```rust
// crates/oya-intelligence-model-cutover/src/lib.rs
pub fn evaluate_cutover(
    capability_id: CapabilityId,
    incumbent_route: ProviderRoute,    // e.g. claude-api
    candidate_route: ProviderRoute,    // e.g. oya-internal-kr-foundation-v3
    eval_set: EvalSet,             // per-vertical, per-cohort
) -> CutoverDecision {
    // 1. Run candidate against eval_set
    // 2. Compare per-cohort dominance vs. incumbent
    // 3. Adversarial cohort must pass at incumbent-or-better
    // 4. Linguistic cohort must pass at incumbent-or-better in the locales the capability serves
    // 5. Cost-projection must show net favorable at expected volume
    // 6. Privacy-projection must show favorable (in-house keeps data in-region)
    // If all pass: emit CutoverDecision::Promote with audit-emit; route flips on next config push
}
```

### Per-tenant LoRA adapters with consent

Per-tenant fine-tuning is offered via LoRA adapters layered on the in-house base model (never on a provider model — provider TOS routinely forbid this). Tenant consent is explicit, tier-bound, and recorded in the audit chain; the LoRA adapter weights are tenant-scoped and never cross tenant boundaries.

```rust
// crates/oya-intelligence-model-lora/src/lib.rs
pub struct TenantLoraAdapter {
    pub tenant_id: TenantId,
    pub base_model: InternalModelId,
    pub training_corpus_ref: TrainingCorpusRef, // consent-recorded; DSR-cascade-eligible
    pub consent_record: ConsentReceipt,
    pub training_pipeline_attestation: TrainingAttestation, // DP epsilon, k-anonymity, residency
    pub adapter_weights_ref: AdapterWeightsRef,            // tenant-scoped storage
    pub eval_certificate: EvalCertificate,                 // per-tenant eval pass
}
```

### Data pipeline

Training data flows through the Data Use Boundary (DUB) gate. Differential privacy and k-anonymity per the DUB are mandatory; the training pipeline emits an attestation per training run; the attestation is referenced by the resulting model artifact.

### GPU fleet

The GPU fleet lives on the cloud microservice (`cloud` provisions; Foundry consumes). Per-region pinning aligns with tenant residency: a strict-KR tenant's training and inference run on a KR-region fleet.

### Safety + red-team substrate

The eval harness (ADR-0024) is also the red-team substrate for in-house models. Adversarial cohorts include:
- Prompt-injection on the in-house model.
- Data-class violation patterns (e.g. PHI leak from a clinical model).
- Vertical-specific safety failure modes (e.g. medication dosing range, fraud-rule bypass).
- Multilingual jailbreak (a jailbreak that works in one locale but not another is still a jailbreak).

### CI lanes

- `foundry-model-eval-cutover` — gates a router-preference change to an in-house route on a per-vertical eval win.
- `foundry-model-training-attestation` — every training run emits a DP/k-anonymity attestation; missing attestation fails publish.
- `foundry-model-tenant-isolation` — synthetic test asserts a tenant's LoRA adapter never serves another tenant.
- `foundry-model-data-use-boundary` — every training corpus reference is DUB-annotated and consent-recorded.

---

## Consequences

### Positive
- Optionality: when a provider's pricing or terms shift, we have an in-house path on the highest-volume tasks.
- Residency: per-region in-house models keep regulated-tenant data inside the region.
- Cost: high-volume specialized tasks (OCR, STT, embedding) are dramatically cheaper in-house than as provider API calls.
- Concentration-risk reduction: a provider outage does not propagate to every Foundry surface.
- The cutover gate is structural (one router-preference change) not architectural — switching is cheap.

### Negative
- Capital and talent commitment over a long horizon; this is years, not months.
- Per-tenant LoRA adapter infrastructure is a non-trivial substrate (consent, attestation, isolation, eval certification).
- Operating a GPU fleet is a substantive operational surface (capacity, scheduling, failure modes).
- Risk of building models that never beat the provider — sunk cost without value.

### Operational
- Runbook: `runbooks/foundry-model-cutover.md` — eval-suite review, A/B run, router-preference change, monitor.
- Runbook: `runbooks/foundry-model-training-incident.md` — training-pipeline failure, attestation mismatch, data-flow breach.
- Runbook: `runbooks/foundry-model-lora-adapter-rollback.md` — tenant adapter rollback procedure.
- On-call: GPU-fleet capacity alerts go to `cloud`; in-house model serving alerts go to `foundry`.
- Quarterly: per-vertical cutover review — which capabilities are still on provider vs. in-house, and where is the eval gap closing or widening.

---

## Alternatives considered

1. **Stay 100% provider-only forever.** Pros: zero training capex; talent burden minimal. Cons: residency, cost, concentration risk all unbounded; per-vertical accuracy capped at provider general performance. Rejected as the long-horizon answer; accepted as the W-Foundation through W-Public-GA stance.
2. **Build a frontier general LLM in-house.** Pros: maximum sovereignty. Cons: capital and talent commitment incompatible with Oyatie's positioning; unlikely to beat frontier vendors on general reasoning. Rejected.
3. **Per-task models from open-source baselines only (no original training).** Pros: lower capex. Cons: open-source baseline weights have license + provenance + safety questions; gives us less differentiation. Adopted partially — open-source baselines are the starting point for several families, but we do original training on top.
4. **External fine-tuning vendor as system-of-record.** Pros: less to build. Cons: external SoR for our most cohesion-critical model artifacts; data-flow concerns. Rejected per the build-vs-buy posture.

---

## Open questions

1. What is the trigger to enter W-AI-Model-Substrate? Per-vertical revenue threshold? Per-capability volume threshold? Provider price shock? *Owner: `foundry` + founder.*
2. How do per-tenant LoRA adapters reconcile with DSR cascades — when a subject is erased, do we have to retrain the adapter, or is differential-privacy attestation sufficient? *Owner: `foundry` + `platform-privacy-dub`.*
3. Open-source baseline selection — do we standardize on one base family (e.g. Llama / Mistral / Qwen / DeepSeek) or maintain optionality? *Owner: `foundry`.*
4. How do we structure the talent ramp without fragmenting the team that owns the runtime + platform? *Owner: founder + `foundry`.*
5. Which capability gets the first cutover gate evaluated? (My instinct: embedding for Korean RAG — high volume, narrow task, eval-tractable.) *Owner: `foundry`; target the next pack.*

---

## References

- Internal: ADR-0020 (`ProviderAdapter` extends to `oya-internal-*`), ADR-0024 (eval harness is the cutover gate), ADR-0025 (engineering platform supplies catalog + supply chain attestation for model artifacts), ADR-0027 (vision/speech/robotics sub-substrates layer on the same kernel).
- External baselines under evaluation: Llama family, Mistral family, Qwen family, DeepSeek, Korean-tuned variants from regional vendors (informational; license analysis required per build-vs-buy).
- Compliance binding: KR PIPA (residency + DP), GDPR (training-data lawful basis), HIPAA (PHI training boundaries), KR FSC / JP FSA / PCI for vertical safety classifiers.
