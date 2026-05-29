---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: S-013
related_packs: [gdpr, eidas, eu-ai-act]
date: 2026-05-21
---

# AI Redlining Prompt Template

CLM's AI redlining feature operates per IP-027 (obligation-extraction-confidence-review) + IP-026 (clause-deviation-negotiation-ledger). It is classified as **Limited Risk** under the EU AI Act per `legal-dimensions/eu-ai-act-classification-for-clm-ai.md` for commercial contracts (Annex III 4 high-risk only when configured for autonomous-acceptance on employment contracts).

## Model selection

```
AIModelSelection {
  default_model_id: ModelId,                    // tenant-default
  fallback_model_id: ModelId,                   // on default outage
  fine_tuning_lineage: FineTuningLineage,
  tenant_byok: ByokCredentials?,                // per `feedback_byok_everywhere_credentials`
}
```

Default model options:

- **Llama-3.1-70B-Instruct** fine-tuned on Legal-Pile-v2 + tenant-specific legal corpus (local, BYOK key for fine-tuning).
- **Llama-3.1-8B-Instruct** for cost-sensitive tenants (faster, smaller, lower quality).
- **Claude 3.7 Sonnet** via cross-emit to `intelligence` µservice.
- **GPT-4o** via cross-emit to `intelligence` µservice.
- **Custom tenant-BYOK model** via the `intelligence` µservice provider-portability layer.

## Canonical prompt structure

```
SYSTEM PROMPT (frozen, audit-chain pinned):

You are a legal-clause analysis assistant for contract negotiation. You will be
shown a contract clause and asked to perform one of the following tasks:

  1. SUGGEST_REDLINE: propose specific text edits to align the clause with the
     tenant's standard playbook position.
  2. EVALUATE_DEVIATION: classify the clause's deviation from standard
     (Fallback / Non-standard / High-risk / Prohibited / Approved-exception).
  3. EXTRACT_OBLIGATIONS: identify obligations imposed by the clause,
     specifying owner, due basis, and trigger condition.
  4. EVALUATE_RISK: identify legal or commercial risks in the clause.
  5. SUMMARIZE: produce a plain-language summary of the clause.

Output format: JSON conforming to the schema specified per task.

Do NOT generate legal advice. You are not a substitute for licensed legal
counsel. The output is advisory; final acceptance is by a human reviewer.

You will be given:
  - The clause text to analyze.
  - The clause family (per taxonomies/clause-family-taxonomy.md).
  - The contract type.
  - The tenant's standard playbook position for this clause family.
  - The counterparty's industry and jurisdiction.

You must NOT use any tenant or counterparty PII for purposes outside this task.

USER MESSAGE:

Task: {{task}}
Clause family: {{clause_family}}
Contract type: {{contract_type}}
Tenant standard position: {{standard_position_text}}
Counterparty industry: {{counterparty_industry}}
Counterparty jurisdiction: {{counterparty_jurisdiction}}

Clause text:
<<<
{{clause_text}}
>>>

[Respond with JSON only.]
```

## Output schema (per task)

### SUGGEST_REDLINE

```json
{
  "task": "suggest_redline",
  "model_id": "...",
  "model_version": "...",
  "suggested_redline": [
    {
      "span_start": <int>,
      "span_end": <int>,
      "text_before": "...",
      "text_after": "...",
      "rationale": "..."
    }
  ],
  "confidence": <float 0-1>
}
```

### EVALUATE_DEVIATION

```json
{
  "task": "evaluate_deviation",
  "deviation_classification": "Fallback | NonStandard | HighRisk | Prohibited | ApprovedException",
  "rationale": "...",
  "confidence": <float 0-1>
}
```

### EXTRACT_OBLIGATIONS

```json
{
  "task": "extract_obligations",
  "obligations": [
    {
      "owner_role": "Tenant | Counterparty | Both",
      "obligation_type": "...",
      "trigger_condition": "...",
      "due_basis_expression": "...",
      "amount": <number?>,
      "currency": "...",
      "confidence": <float 0-1>
    }
  ]
}
```

## Provenance binding

Every AI invocation records:

- `model_id`, `model_version` (pinned snapshot, not "latest").
- `prompt_template_hash` (BLAKE3 of the actual prompt sent).
- `temperature`, `top_p`, `seed`.
- `input_hash` (BLAKE3 of the clause text).
- `output_hash` (BLAKE3 of the model response).
- `inference_timestamp`.
- `inference_duration_ms`.
- `tenant_id`, `tenant_class`.
- `audit_event_id`.

Provenance is required for IP-030 (counterparty-redline-provenance) and for EU AI Act Article 13 transparency obligation.

## PII handling

The prompt strips identifiable counterparty PII before submission to the model. Specifically:

- Counterparty signatory names replaced with `[COUNTERPARTY_SIGNATORY_1]`.
- Counterparty addresses replaced with `[COUNTERPARTY_ADDRESS]`.
- Bank account numbers replaced with `[ACCOUNT_NUMBER]`.
- Pricing replaced with `[PRICE_AMOUNT]` (preserves currency unit).
- Dates preserved (often material to legal analysis).
- Entity names preserved (legal-entity names are public-record).

The stripped placeholders are re-substituted at output time so the redline applies to the actual clause.

## Confidence-band gating

Per IP-027:

- Confidence ≥ 0.95: auto-propose; queue for low-friction human review.
- 0.85 ≤ Confidence < 0.95: queue for full human review.
- 0.70 ≤ Confidence < 0.85: advisory-only; not surfaced as proposal.
- Confidence < 0.70: filtered out.

## EU AI Act Annex III boundary

The prompt explicitly directs the model NOT to autonomously accept changes. Per `legal-dimensions/eu-ai-act-classification-for-clm-ai.md`, this maintains Limited Risk status. When the tenant enables autonomous-acceptance for employment contracts, the µservice cross-emits a `eu_ai_act_annex_iii_4_classification_change` event and requires the tenant to register the system in the EU AI database.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"AIClauseAutoAccept",
  resource is Contract
) when {
  resource.contract_type in ["employment", "termination_letter", "promotion_decision"] &&
  resource.tenant.eu_ai_act_high_risk_operator_registered == false
};

forbid (
  principal,
  action == Action::"AIInferenceWithPII",
  resource is Clause
) when {
  resource.pii_stripping_applied == false
};
```

## Audit events

- `oya.contract.lifecycle.management.ai.inference_invoked`
- `oya.contract.lifecycle.management.ai.suggestion_surfaced`
- `oya.contract.lifecycle.management.ai.suggestion_accepted`
- `oya.contract.lifecycle.management.ai.suggestion_rejected`
- `oya.contract.lifecycle.management.ai.confidence_threshold_crossed`
- `oya.contract.lifecycle.management.ai.pii_stripping_verified`

## Standards references

- Regulation (EU) 2024/1689 (EU AI Act).
- ISO/IEC 42001:2023 (AI Management Systems).
- NIST AI RMF 1.0.
- Apache Llama-3.1 release notes.
- Anthropic Claude 3.7 model card.
