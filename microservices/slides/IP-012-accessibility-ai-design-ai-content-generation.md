---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-012-accessibility-ai-design-ai-content-generation
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + foundry-runtime-team + ops-accessibility + dpo-office
acceptance_lanes: [cargo-check, cargo-nextest, ai-act-risk-class-stamp, ai-provenance-watermark-preserved, wcag-contrast]
depends_on: [IP-002, IP-009]
---

# IP-012: accessibility + ai-design + ai-content-generation — T0/T1/T2 + EU AI Act risk-class

## Intent

Author the AI capability BCs (T0 suggest, T1 assist, T2 auto) + accessibility BC (alt-text suggest, contrast check, color-blind-safe palette). EU AI Act risk-class stamping per ADR-SLIDES-0006 is the load-bearing assertion.

## ChangeSet boundary

~14 crates across accessibility + ai-design + ai-content-generation BCs.

## Concrete File Targets

`src/crates/oya-slides-accessibility-...`, `oya-slides-ai-design-...`, `oya-slides-ai-content-generation-...`

## Code Shape

`ai-content-generation-domain/src/risk_class_enforcement.rs`:

```rust
pub fn enforce_annex_iii(req: &AiContentGenerateRequest, verdict: &RiskClassVerdict) -> Result<(), AiActViolation> {
    if verdict.class != RiskClass::HighRiskAnnexIii {
        return Ok(());
    }
    // High-risk: refuse unless pack override + Cedar grant
    if !req.deck.pack_override_annex_iii {
        return Err(AiActViolation::HighRiskRefused {
            reason: "Annex III high-risk; pack-override required + Cedar permission".into(),
        });
    }
    if !req.deck.has_cedar_grant("pack_override_annex_iii") {
        return Err(AiActViolation::HighRiskRefused {
            reason: "Cedar grant missing".into(),
        });
    }
    Ok(())
}
```

`ai-content-generation-domain/src/provenance_watermark.rs`:

```rust
pub fn embed_watermark(deck: &mut Deck, foundry_correlation_id: &str) {
    // Per ADR-SLIDES-0006: indelible provenance marker; preserved through PPTX/PDF/MP4 export.
    deck.metadata.insert("oya-ai-provenance".into(), foundry_correlation_id.into());
    for slide in &mut deck.slides {
        slide.metadata.insert("oya-ai-provenance".into(), foundry_correlation_id.into());
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-ai-content-generation-domain --test annex_iii_refusal_default
cargo nextest run -p oya-slides-ai-content-generation-domain --test provenance_watermark
cargo nextest run -p oya-slides-accessibility-domain --test wcag_contrast
cargo nextest run -p oya-slides-accessibility-domain --test color_blind_safe
oya gate validate ai-act-risk-class-stamp --microservice slides
oya gate validate ai-provenance-watermark-preserved --microservice slides
```

## Halt Conditions

- High-risk Annex III refusal test fails — STOP. AC-16 invariant.
- Provenance watermark not preserved through export — STOP. ADR-SLIDES-0006 invariant.
- WCAG contrast lane red — STOP.

## Next IP

IP-013.
