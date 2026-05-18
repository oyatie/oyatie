---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-011-ai-formula-smart-fill-foundry-runtime-bridge
status: pending
owner: axis-sheets + foundry-runtime-team
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-ai-formula-validation-required]
depends_on: [IP-005, IP-008]
---

# IP-011: ai-formula — foundry-runtime bridge with T1 advisory + T2 gated per ADR-SHEETS-0005

## Intent

Author the `ai-formula` BC: bridges to foundry-runtime SDK for prose→formula drafting, smart-fill inference from N seed examples, and anomaly detection. T1 advisory (human accept); T2 cross-µservice auto-apply gated by Cedar + ChangeSet review per ADR-SHEETS-0005. PII redactor + prompt-injection scrub + formula-engine grammar validation pipeline mandatory.

## ChangeSet boundary

Six crates:
- `oya-sheets-ai-formula-{kernel,domain,usecase,api,adapter,sdk}`

## Code Shape

`ai-formula-domain/src/draft.rs` (excerpt):

```rust
pub async fn draft_formula(req: AiFormulaDraftRequest) -> Result<AiFormulaDraftResponse> {
    // STEP 1: Consent check (Cedar PERMIT 5)
    if !req.consent_acknowledged { return Err(AiFormulaError::ConsentMissing); }

    // STEP 2: EU AI Act regulated-domain check per ADR-SHEETS-0005
    if is_regulated_domain(&req) && !tenant_attested_ai_act_conformity(&req.tenant_id).await {
        return Err(AiFormulaError::AiActConformityNotAttested);
    }

    // STEP 3: PII redactor (per threat-model T-I-05)
    let (redacted_prose, pii_redacted_count) = pii_redactor::scrub(&req.prose);

    // STEP 4: Prompt-injection classifier (per threat-model T-S-05)
    if prompt_injection_detector::detect(&redacted_prose) {
        emit_audit("sheets_ai_formula_prompt_injection_detected", &req);
        return Err(AiFormulaError::PromptInjectionDetected);
    }

    // STEP 5: foundry-runtime SDK invocation (pack-resident routing)
    let completion = foundry_runtime_sdk::draft_formula(&redacted_prose, &req.target_jurisdiction).await?;

    // STEP 6: Formula-engine grammar validation
    let validation = formula_engine_sdk::validate_grammar(&completion);

    // STEP 7: Audit-chain emission
    audit_chain_sdk::emit("AiFormulaDraftRequested", &req, &completion);

    Ok(AiFormulaDraftResponse {
        draft_id: ulid_new(),
        candidate_formula: completion,
        grammar_valid: validation.valid,
        validation_errors: validation.errors,
        prompt_injection_detected: false,
        pii_redacted_count,
    })
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-ai-formula-kernel ... -p oya-sheets-ai-formula-sdk
cargo nextest run -p oya-sheets-ai-formula-domain --test test_smart_fill_corpus
cargo run -p oya-dev-cli -- gate validate ai-formula-validation-required --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_smart_fill_corpus` | AC-16 — ≥ 80% accuracy on 3-cell-seed corpus |
| `test_prose_to_formula_grammar_valid_rate` | ≥ 80% grammar-valid output across 100-prose corpus |
| `test_pii_redactor` | PII patterns scrubbed before LLM submission |
| `test_prompt_injection_detection` | OWASP LLM A01 corpus rejected; 0% bypass |
| `test_ai_act_conformity_gate` | regulated-domain workflow refuses AI-formula without tenant attestation |
| `test_audit_chain_emission` | every invocation emits AiFormulaDraftRequested seal |
| `test_pack_resident_routing` | pack-eu tenant routes to EU-resident provider only |

## Halt Conditions

- PII leak in synthetic test corpus — STOP. T-I-05.
- Prompt-injection bypass — STOP. T-S-05.
- AI Act gate not enforced for regulated domain — STOP.

## Next IP

[`IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md`](IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md)

## References

- PRD FR-14 + FR-15 + AC-05 + AC-16.
- threat-model.md T-I-05 + T-S-05 + T-D-05.
- ADR-SHEETS-0005 (AI-formula bounds).
- OWASP Top 10 LLM Applications — `owasp.org/www-project-top-10-for-large-language-model-applications/`.
- EU AI Act 2024 — Annex III.
