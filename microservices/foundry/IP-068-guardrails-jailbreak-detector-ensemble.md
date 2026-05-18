---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-008-jailbreak-detector-ensemble
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, port-location, data-class, classifier-model-cosign-signed]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: jailbreak-detector ensemble (heuristic + classifier + LLM-judge)

## Intent

Five crates introducing the jailbreak-detector BC: `-kernel` (port traits + entities `JailbreakSignal`, `EnsembleVerdict`, `DetectorVersion`), `-domain` (ensemble scoring math + canonicalisation passes), `-usecase` (orchestrator chaining heuristic → classifier → LLM-judge fallback), `-adapter` (heuristic detectors: regex + ngram + canonicalisation), `-adapter-classifier-model` (ONNX-runtime client + LLM-judge via foundry-providers SDK).

## ChangeSet boundary

Five crates as one ChangeSet — the ensemble is meaningless without all three layers.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-jailbreak-detector-kernel/Cargo.toml` + `src/{lib.rs,entities.rs,ports.rs,errors.rs}` | create |
| `src/crates/oya-foundry-guardrails-jailbreak-detector-domain/Cargo.toml` + `src/{lib.rs,ensemble.rs,canonicalisation.rs,scoring.rs}` | create |
| `src/crates/oya-foundry-guardrails-jailbreak-detector-usecase/Cargo.toml` + `src/{lib.rs,orchestrator.rs}` | create |
| `src/crates/oya-foundry-guardrails-jailbreak-detector-adapter/Cargo.toml` + `src/{lib.rs,heuristic.rs,regex_set.rs,ngram.rs}` | create |
| `src/crates/oya-foundry-guardrails-jailbreak-detector-adapter-classifier-model/Cargo.toml` + `src/{lib.rs,classifier.rs,llm_judge.rs}` | create |
| `Cargo.toml` (workspace) | update — 5 members |
| `catalog/oya-foundry-guardrails-jailbreak-detector-{kernel,domain,usecase,adapter,adapter-classifier-model}.yaml` | create |
| `tests/jailbreak/golden_fixtures.rs` | create (red-team catalogue; 10+ known-jailbreak fixtures + 10+ legitimate-prompt fixtures) |

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait JailbreakDetectorEnsemble: Send + Sync + Sealed {
    async fn detect(&self, prompt: &str, ctx: &DetectCtx) -> Result<EnsembleVerdict, KernelError>;
}

#[async_trait]
pub trait LlmJudge: Send + Sync + Sealed {
    async fn judge(&self, prompt: &str, ensemble_disagreement: &EnsembleDisagreement) -> Result<JudgeVerdict, KernelError>;
}
```

```rust
// usecase/src/orchestrator.rs (sketch)
pub struct JailbreakOrchestrator<H, C, J> {
    heuristic: H,
    classifier: C,
    llm_judge: J,
    config: OrchestratorConfig,
}

impl<H, C, J> JailbreakOrchestrator<H, C, J>
where H: HeuristicDetector + Send + Sync,
      C: ClassifierDetector + Send + Sync,
      J: LlmJudge + Send + Sync {

    pub async fn detect(&self, prompt: &str, ctx: &DetectCtx) -> Result<EnsembleVerdict, OrchestratorError> {
        let canon = canonicalise(prompt);   // strip whitespace + zero-width + homoglyph + base64
        let h = self.heuristic.detect(&canon).await?;
        let c = self.classifier.detect(&canon).await?;
        let agree = h.is_definite() && c.is_definite() && h.verdict == c.verdict;
        if agree {
            return Ok(EnsembleVerdict::from_agreement(h, c));
        }
        // disagreement → invoke LLM-judge fallback (5% rate cap; budget per tenant)
        if ctx.llm_judge_budget_remaining() {
            let judge = self.llm_judge.judge(prompt, &EnsembleDisagreement { h: h.clone(), c: c.clone() }).await?;
            Ok(EnsembleVerdict::from_judge(h, c, judge))
        } else {
            // fail-closed when budget exhausted
            Ok(EnsembleVerdict::budget_exhausted_block())
        }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-jailbreak-detector-kernel --all-features
cargo check -p oya-foundry-guardrails-jailbreak-detector-domain --all-features
cargo check -p oya-foundry-guardrails-jailbreak-detector-usecase --all-features
cargo check -p oya-foundry-guardrails-jailbreak-detector-adapter --all-features
cargo check -p oya-foundry-guardrails-jailbreak-detector-adapter-classifier-model --all-features
cargo nextest run -p oya-foundry-guardrails-jailbreak-detector-domain --all-features
cargo nextest run -p oya-foundry-guardrails-jailbreak-detector-usecase --all-features --test ensemble_golden_fixtures
cargo nextest run -p oya-foundry-guardrails-jailbreak-detector-adapter-classifier-model --all-features --test classifier_integration
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_canonicalisation_strips_obfuscation` | zero-width + homoglyph + base64 attacks |
| `test_ensemble_agreement_path` | agreement → fast path; no LLM-judge invoked |
| `test_ensemble_disagreement_invokes_judge` | disagreement triggers fallback |
| `test_budget_exhausted_fails_closed` | budget=0 → block + budget_exhausted reason |
| `test_golden_fixtures` (10+ known jailbreaks + 10+ legitimate) | recall ≥ 0.95, precision ≥ 0.90 on golden fixtures |
| `integration_classifier_round_trip` | real ONNX-runtime against placeholder model |

## Halt Conditions

- Recall on golden fixtures < 0.90 — escalate; do not promote-to-enforce.
- LLM-judge bypass attempted — refactor; budget enforcement mandatory.

## Next IP

[`IP-009-ai-slop-detector.md`](IP-009-ai-slop-detector.md)

## References

- ADR-0056, ADR-0105.
- `policy/guardrail-enforcement.md` (ensemble policy).
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`.
- OWASP LLM Top 10 (2025) LLM01 Prompt Injection.
- MITRE ATLAS AML.T0043.
