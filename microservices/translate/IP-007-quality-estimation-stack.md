---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-007-quality-estimation-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate + council-privacy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, eu-ai-act-bounds]
---

# IP-007: Quality Estimation stack (`oya-translate-qe-*`)

## Intent

Per-segment quality estimation via COMET-Kiwi-class model served on `foundry-runtime`. EU AI Act bounds per ADR-TRANSLATE-0003 — QE deployed as **limited-risk** AI (Art. 50 transparency) unless used to make automated decisions affecting high-risk content classes (medical / employment / credit / legal), in which case `EuAiActClassification::HighRisk` flag triggers FRIA + human-oversight gate.

## ChangeSet boundary

Crates: `oya-translate-qe-{kernel, domain, usecase, api, adapter-foundry-runtime, rest, worker, sdk, app}`.

## Algorithm

```rust
pub struct QualityEstimatorImpl {
    pub foundry_invoker: ProviderInvoker,    // foundry-providers
    pub policy_repo: QualityPolicyRepository,
    pub eu_ai_act_emitter: EuAiActDisclosureEmitter,
}

#[async_trait]
impl QualityEstimator for QualityEstimatorImpl {
    async fn score(&self, src: &str, tgt: &str, src_lang: &LanguageTag, tgt_lang: &LanguageTag, content_class: ContentClass)
        -> Result<QualityScore, RouterError>
    {
        // 1. Resolve EU AI Act classification per ADR-TRANSLATE-0003
        let classification = classify(content_class);

        // 2. If high-risk class, enforce FRIA-on-file gate
        if classification == EuAiActClassification::HighRisk {
            self.policy_repo.assert_fria_on_file(content_class).await?;
        }

        // 3. Invoke foundry-runtime QE capability
        let qe_response = self.foundry_invoker.invoke(/* qe-comet-kiwi capability */).await?;

        // 4. Disclosure (per Art. 50; always emit when EU pack)
        self.eu_ai_act_emitter.emit(/* request + response + classification */).await?;

        Ok(QualityScore {
            score: qe_response.score,
            model_id: qe_response.model_id,
            eu_ai_act_classification: classification,
        })
    }
}

fn classify(content_class: ContentClass) -> EuAiActClassification {
    match content_class {
        ContentClass::Medical | ContentClass::Employment | ContentClass::Credit => EuAiActClassification::HighRisk,
        ContentClass::Legal => EuAiActClassification::HighRisk,  // err on the side of high-risk for legal translations
        ContentClass::Marketing | ContentClass::UiString | ContentClass::CodeComment
            | ContentClass::Narrative | ContentClass::Subtitle | ContentClass::GeneralText
            => EuAiActClassification::LimitedRisk,
    }
}
```

## Golden Eval Set

`microservices/translate/capabilities/eval/qe-golden.jsonl`:
- WMT QE shared-task subset.
- Per-pack tenant-de-identified samples.
- Pass threshold: 0.99 correctness against reference scores within ±2 points.

## Test Plan

| Test | Verifies |
|---|---|
| `test_qe_score_in_range_0_100` | invariant |
| `test_high_risk_class_requires_fria` | gate enforced |
| `test_eu_ai_act_disclosure_emitted_always` | every call emits |
| `test_qe_model_rollback_on_eval_regression` | FM-20 mitigation tied to canary deploy |
| `tests/integration/qe_foundry_runtime_invocation.rs` | end-to-end through foundry-runtime |
| `tests/integration/qe_golden_eval_pass_99.rs` | golden set pass-rate ≥ 0.99 |

## Halt Conditions

- High-risk content class served QE without FRIA on file.
- `EuAiActDisclosure` event not emitted.
- QE model rollback runbook not exercised in load test.

## Next IP

[`IP-008-language-detection-stack.md`](IP-008-language-detection-stack.md)
