---
doc_class: PolicySpec
title: EU AI Act overlay — translate µservice
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-translate + ops-compliance
deciders: council-privacy, ops-compliance, axis-translate, council-architecture
related_adrs: [ADR-0135, ADR-TRANSLATE-0003]
related_artifacts:
  - microservices/translate/dpia.md
  - microservices/translate/compliance.md
  - microservices/translate/decisions/ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md
  - microservices/translate/policy/translate-tenant-scope.cedar
align_with: workflow-studio/decisions/ADR-WS-0005
review_cadence: annually + on every EU AI Act enforcement-detail publication
doc_status: published
---

# EU AI Act overlay — translate µservice

## Purpose

Operational policy that translates ADR-TRANSLATE-0003's decisions into:

- Per-call disclosure record schema.
- Per-content-class classification table.
- Tenant-side responsibilities.
- Downstream-consumer obligations.
- Enforcement timeline.

This document is the canonical operational reference for EU AI Act compliance under translate µservice.

## Classification Table

Per ADR-TRANSLATE-0003:

| Content class | Risk class | Triggers FRIA gate? | Default human-oversight? |
|---|---|---|---|
| `UiString` | Limited | No | No |
| `Marketing` | Limited | No | No |
| `CodeComment` | Limited | No | No |
| `Narrative` | Limited | No | No |
| `Subtitle` | Limited | No | No |
| `GeneralText` | Limited | No | No |
| `Legal` | **High** | YES | YES |
| `Medical` | **High** | YES | YES |
| `Employment` | **High** | YES | YES |
| `Credit` | **High** | YES | YES |

Tenants set `content_class` per call; mis-classification is tenant's responsibility per DPA (tenant is controller-deployer). Where translate µservice can heuristically detect high-risk content (e.g., regex for medical terminology in source), it tags `content_class_inferred_risk: high` in the audit event and recommends tenant reclassify; but final responsibility remains with tenant.

## Per-Call Disclosure Record Schema (`EuAiActDisclosure`)

Emitted to audit-chain topic `oya.translate.eu-ai-act.disclosure`:

```json
{
  "schema_version": "1.0",
  "event_id": "uuid",
  "decision_id": "uuid",
  "occurred_at": "2026-05-17T12:34:56.789Z",
  "tenant_id": "acme-corp",
  "pack": "eu",
  "jurisdiction": "EU",
  "ai_system": {
    "function": "machine-translation",
    "engine_vendor": "anthropic|openai|google-translate|deepl|in-house",
    "model_id": "claude-3-5-sonnet-20240620",
    "region": "europe-west3"
  },
  "classification": "limited-risk|high-risk",
  "content_class": "ui-string|marketing|legal|medical|employment|credit|...",
  "human_oversight": {
    "required": true|false,
    "satisfied_via": "workflow-engine|tenant-side|none"
  },
  "transparency_obligations": {
    "art_50": "fulfilled-via-this-event",
    "art_13_deployer_info": "available-via-/v1/policies/tenant/{tenant}"
  },
  "input_fingerprint": "blake3-hex (source segment hash; NOT plaintext)",
  "output_fingerprint": "blake3-hex (target segment hash; NOT plaintext)",
  "envelope_signature": "ed25519-hex"
}
```

This event is the **Article 12 record-keeping artifact** and the **Article 50 transparency artifact**. Retention 10 y per Art. 12 + Art. 18.

## Article-by-Article Implementation

### Art. 5 (Prohibited practices)

Not applicable — translate µservice does NOT exhibit any Art. 5 banned property (no social scoring, no real-time biometric, no subliminal manipulation, etc.).

### Art. 6 + Annex III (High-risk classification triggers)

Triggered when `content_class` ∈ {legal, medical, employment, credit}. See classification table above.

### Art. 9 (Risk management system)

- Threat-model.md + DPIA.md + this overlay + ADR-TRANSLATE-0003 = oyatie's translate-µservice risk management system per Art. 9(1).
- Per-vendor + per-engine risk register maintained.
- Quarterly review.

### Art. 10 (Data governance + management)

- TM provenance recorded per `oya-translate-tm-domain` (origin: human | mt | post-edit).
- Termbase governance per ADR-TRANSLATE-0002.
- Per-vendor DPA covers vendor-side data governance.

### Art. 11 (Technical documentation)

- This µservice's full doc suite: PRD.md + ADRs + policy/ + runbooks/ + dashboards/ + slos/.
- Per ADR-0131 + ADR-0133.

### Art. 12 (Record-keeping)

- `EuAiActDisclosure` + `TranslationCompleted` + `QualityEstimated` + `EngineRouted` events.
- 10 y retention per pack audit-chain.

### Art. 13 (Transparency + information to deployers)

- Tenant operator API at `GET /v1/policies/tenant/{tenant}` returns the per-tenant deployer information.
- `developer-docs/translate/eu-ai-act-disclosure-consumption.md` (per `sdk-plan.md`) instructs how downstream UIs render disclosure.

### Art. 14 (Human oversight)

- High-risk content classes (legal/medical/employment/credit) route through `workflow-engine` per ADR-TRANSLATE-0003.
- Limited-risk content classes do NOT require human-in-loop by default; tenant may opt in.
- Downstream UI consumer obligations documented per `developer-docs/translate/eu-ai-act-disclosure-consumption.md`.

### Art. 15 (Accuracy + robustness + cybersecurity)

- Per-vendor adapter response-shape validator + per-vendor pinned-cert + envelope signing.
- QE model reference-set evaluation per release (pass ≥ 0.99).
- mTLS + Cedar + OpenBao posture covers cybersecurity.

### Art. 27 (FRIA for deployers of high-risk AI)

- When tenant deploys translate in high-risk context, tenant (as deployer) must execute FRIA; oyatie provides FRIA template at `microservices/translate/legal/fria-template.md` (Slice D; outside M01 scope).
- `policy.fria_on_file == true` is the gate; tenants without FRIA cannot invoke QE for high-risk content classes.

### Art. 50 (Transparency obligations)

- `EuAiActDisclosure` event per call when `jurisdiction == EU`.
- Downstream UI consumer renders user-facing notice per `developer-docs`.

## Tenant Responsibilities (per DPA)

- Correctly classify `content_class` per call.
- Execute FRIA when deploying in high-risk context.
- Render Art. 50 disclosure to end users when displaying AI-generated translation.
- Honor human-oversight requirement for high-risk classes.
- Notify oyatie if tenant identifies a misclassification.

## Enforcement Timeline

- EU AI Act prohibitions (Art. 5) effective 2025-02-02.
- General-purpose AI obligations (Arts. 51 + 53) effective 2025-08-02.
- High-risk AI obligations (Arts. 6 + 9–15) effective 2026-08-02.
- Full enforcement 2027-08-02.

This overlay is positioned for full enforcement at full-effectiveness.

## Cross-µservice Alignment

- `workflow-studio/decisions/ADR-WS-0005` — sibling µservice with equivalent EU AI Act treatment.
- `foundry-providers/policy/` — EU AI Act disclosure per provider invocation (this µservice consumes that disclosure when routing via foundry-providers).

## Verification

- `tests/integration/eu_ai_act_disclosure_emitted_per_qe_call.rs`.
- `tests/integration/high_risk_content_class_requires_fria.rs`.
- `oya-translate-eu-ai-act-disclosure` BLOCKER lane in branch-protection.
- Annual EU AI Act audit (external; privacy-counsel).

## References

- EU AI Act (Reg. (EU) 2024/1689).
- EU AI Office implementation guidance (per-publication).
- ADR-TRANSLATE-0003 (this overlay's parent ADR).
- Workflow Studio ADR-WS-0005 (sibling alignment).
- `microservices/translate/compliance.md` §"EU AI Act".
- `microservices/translate/dpia.md` §3 R-06 + §4.
- ICO Sample DPIA template (informational).
- EDPB Guidelines + GDPR-AI Act interplay opinion.
