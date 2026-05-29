---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-008-language-detection-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: Language Detection stack (`oya-translate-langdetect-*`)

## Intent

Per-text language detection. FastText / LangID-class model served on `foundry-runtime`. Per PRD §"Performance": p99 ≤ 50 ms for ≤ 4 KB input.

## ChangeSet boundary

Crates: `oya-translate-langdetect-{kernel, domain, usecase, api, adapter-foundry-runtime, rest, worker, sdk, app}`.

## Algorithm

```rust
pub struct LanguageDetectorImpl {
    pub foundry_invoker: ProviderInvoker,
}

#[async_trait]
impl LanguageDetector for LanguageDetectorImpl {
    async fn detect(&self, text: &str) -> Result<LanguageDetection, RouterError> {
        // 1. Bound input (≤ 4 KB)
        let bounded = if text.len() > 4096 { &text[..4096] } else { text };

        // 2. Fast-path heuristic (latin/cjk/cyrillic/arabic script bucket)
        let script_bucket = whatlang_quick_script_detect(bounded);

        // 3. Invoke FastText/LangID model via foundry-runtime
        let response = self.foundry_invoker.invoke(/* langdetect capability */).await?;

        // 4. Canonicalize to BCP 47 + ISO 639-3
        let canonical = canonicalize_to_bcp47(&response.language_code);

        Ok(LanguageDetection {
            language: canonical,
            confidence: response.confidence,
            alternates: response.alternates,
            script: script_bucket,
        })
    }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_detect_korean_text` | round-trip ko |
| `test_detect_japanese_text` | round-trip ja |
| `test_detect_chinese_simplified_vs_traditional` | zh-CN vs zh-TW |
| `test_detect_arabic_text` | ar |
| `test_detect_hebrew_text` | he |
| `test_detect_codeswitched_returns_alternates` | mixed-lang |
| `test_input_bound_4kb_truncation` | ≤ 4 KB |
| `test_p99_latency_under_50ms` | budget |
| `tests/integration/reference_eval_pass_95.rs` | reference eval pass ≥ 0.95 |

## Halt Conditions

- Detection latency > 50 ms p99 on standard hardware.
- BCP 47 canonicalization fails for any ISO 639-3 supported code.
- Cross-tenant contamination (not applicable to langdetect; stateless).

## Next IP

[`IP-009-document-translation-stack.md`](IP-009-document-translation-stack.md)

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/translate/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/translate/IP-008-language-detection-stack.md:19` - Per-text language detection. FastText / LangID-class model served on `foundry-runtime`. Per PRD §"Performance": p99 ≤ 50 ms for ≤ 4 KB input.; `microservices/translate/IP-008-language-detection-stack.md:68` - | `test_p99_latency_under_50ms` | budget |.
