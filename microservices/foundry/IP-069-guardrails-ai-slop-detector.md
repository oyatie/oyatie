---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-009-ai-slop-detector
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: ai-slop-detector — catalogue-driven pattern detection

## Intent

Four crates introducing the ai-slop-detector BC: `-kernel` (port traits + entities `AiSlopPattern`, `SlopScore`, `SlopRationale`), `-domain` (pattern composition + scoring), `-usecase` (orchestrator), `-adapter` (heuristic + light BERT classifier). Source-of-truth catalogue: `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`. 100% catalogue coverage is a CI-tracked metric per PRD parity-gap #5.

## ChangeSet boundary

Four crates + golden-fixture catalogue under `tests/aislop/catalogue.rs`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-ai-slop-detector-kernel/...` | create |
| `src/crates/oya-foundry-guardrails-ai-slop-detector-domain/...` | create |
| `src/crates/oya-foundry-guardrails-ai-slop-detector-usecase/...` | create |
| `src/crates/oya-foundry-guardrails-ai-slop-detector-adapter/...` | create |
| `Cargo.toml` workspace | update |
| `catalog/oya-foundry-guardrails-ai-slop-detector-*.yaml` | create |
| `tests/aislop/catalogue.rs` | create (one test per catalogue entry) |

## Code Shape

```rust
// kernel/src/entities.rs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiSlopPattern {
    StubInjection,           // emits TODO / placeholder marker in output
    VerbosePreamble,         // "Certainly! I'd be happy to help you with..."
    FabricatedCitation,      // references to non-existent sources
    ShotgunPattern,          // multiple alternative answers without commitment
    VerboseWithoutSubstance, // bullet-points with no semantic content
    HedgingOverload,         // "perhaps", "might", "could be" density excessive
    ApologeticFiller,        // repeated apologies; non-task content
    InconsistentDirective,   // self-contradictory steps
    OverGeneralisation,      // claim covering too broad a scope
    BoilerplateClosing,      // "I hope this helps! Let me know if..."
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlopScore {
    #[data_class(INTERNAL_ONLY)]
    pub pattern: AiSlopPattern,
    #[data_class(INTERNAL_ONLY)]
    pub confidence: f64,
    #[data_class(INTERNAL_ONLY)]
    pub rationale: SlopRationale,
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-ai-slop-detector-kernel --all-features
cargo nextest run -p oya-foundry-guardrails-ai-slop-detector-domain --all-features
cargo nextest run -p oya-foundry-guardrails-ai-slop-detector-usecase --all-features --test catalogue_coverage
cargo run -p oya-dev-cli -- gate validate aislop-catalogue-coverage --threshold 1.0
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_catalogue_coverage` | 100% of `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md` entries have a corresponding detector |
| `test_legitimate_output_low_score` | non-slop output scores below threshold |
| `test_per_pattern_recall_precision` | recall ≥ 0.85, precision ≥ 0.80 per pattern |

## Halt Conditions

- Catalogue entry without corresponding detector — refuse merge.
- Recall < 0.80 on any pattern — escalate.

## Next IP

[`IP-010-classifier-model-adapter-onnx.md`](IP-010-classifier-model-adapter-onnx.md)

## References

- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`.
- `docs/quality/ai-slop-defense/defense-in-depth-architecture.md`.
- ADR-0056, ADR-0105.
