---
id: ADR-SLIDES-0006
title: AI-design + AI-content-generation bounds — EU AI Act risk-class enforcement
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + ops-security + dpo-office + foundry-runtime-team
deciders: council-architecture, axis-workspace, ops-security, dpo-office, foundry-runtime-team, legal
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0126, ADR-0131, ADR-WS-0005]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (FR-29, AC-16)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-012)
  - microservices/slides/dpia.md
  - microservices/slides/compliance.md (EU AI Act section)
  - microservices/slides/capabilities/T0-suggest.yaml
  - microservices/slides/capabilities/T1-assist.yaml
  - microservices/slides/capabilities/T2-auto.yaml
  - microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md
purpose: Establish risk-class enforcement on AI-design (T0/T1) and AI-content-generation (T2) capabilities, with EU AI Act Annex III high-risk refusal as the default and explicit per-pack opt-in required to engage.
doc_status: published
---

# ADR-SLIDES-0006: AI capabilities risk-class enforcement — Annex III high-risk default-refusal

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The slides µservice exposes three AI capability tiers:

- **T0 — Suggest** (deterministic / local-first; low-risk by definition; per `capabilities/T0-suggest.yaml`).
- **T1 — Assist** (foundry-runtime mediated; low/medium-risk per invocation; per `capabilities/T1-assist.yaml`).
- **T2 — Auto** (full-deck-from-prompt, auto-translate, theme-cascade; per `capabilities/T2-auto.yaml`).

The **EU AI Act (Regulation (EU) 2024/1689)** entered into force 2024-08-01, with tiered obligations:
- **Art. 5** Prohibited practices — slides MUST never engage these.
- **Art. 6 + Annex III** High-risk AI systems — listed across 8 categories including:
  - Annex III (1)(b) — recruitment / employment selection.
  - Annex III (3)(a) — public-services access (education ranking).
  - Annex III (5)(b) — creditworthiness assessment.
  - Annex III (5)(c) — emergency / first-responder / medical-triage dispatch.
  - Annex III (6)(a) — administration of justice.
  - Annex III (8)(a) — biometric categorisation.
- **Art. 13** Transparency to deployer.
- **Art. 14** Human oversight.
- **Art. 16** AI provider obligations (foundry-runtime is the provider; slides is the deployer).
- **Art. 50** Transparency to affected persons.

T2 full-deck-from-prompt is the AI capability most likely to slip into Annex III high-risk territory: a tenant might prompt "Generate a slide deck explaining why we rejected this candidate" (Annex III (1)(b) employment) or "Generate a slide deck for jury deliberation" (Annex III (6)(a) justice) or "Generate a slide deck for triage in this clinical scenario" (Annex III (5)(c) medical).

Workflow-studio ADR-WS-0005 already articulated the AI risk-class framework for that µservice's LLM-copilot node-generation. This ADR applies the same framework to slides T2 specifically + extends the framework to T1 capabilities that may carry contextual risk-class (e.g., alt-text on medical images).

PRD Open Question 6 — T2 default policy: refuse on Annex III high-risk context vs allow-with-watermark vs require-human-review. Bias: refuse high-risk by default; per-pack override.

## Decision

Adopt the following risk-class enforcement framework:

1. **Risk-class enum**: `low`, `medium`, `high_risk_annex_iii` — per `contracts/openapi/slides.yaml` + `contracts/proto/slides.proto`. Stamped on every AI invocation.
2. **Risk-class authority**: **foundry-runtime is the authoritative classifier**. Slides forwards the invocation's `usage_context` (employment | credit | legal | medical | educational | marketing | general) + deck metadata; foundry-runtime evaluates against its Annex III rule-table and returns a verdict. Slides cannot stamp the risk-class itself — this is enforced by the `oya-governance-ai-act-risk-class-stamp` CI lane that requires every AI-invocation persistence to carry a foundry-runtime-emitted risk-class signature.
3. **T2 high-risk refusal default**: when `risk_class == high_risk_annex_iii`, slides REFUSES the invocation at the API surface (HTTP 403 + audit row) UNLESS:
   - Deck's pack has the explicit `pack_override_annex_iii=true` flag (per `policy/data-residency.md`); none of the 11 packs has this flag by default.
   - AND deck has a Cedar `pack_override_annex_iii` permission grant (per `policy/tenant-scope.cedar`).
   - AND tenant has signed an additional "AI-Act-Annex-III-Use" addendum to their T&C.
4. **Human oversight required** (Art. 14): T2 outputs ALWAYS require explicit human-accept gate before save; the editor UI surfaces the generated deck in a review mode; tenant must accept or discard.
5. **Provenance watermark** (Art. 13 + 50): T2-generated content carries an indelible per-deck provenance watermark (metadata field + per-slide marker) that survives PPTX/PDF/MP4 export. Documented to tenant.
6. **PHI redaction for us-healthcare pack**: when invocation operates on PHI (us-healthcare pack), `phi_redaction_required=true`; slides pre-flight redacts PHI from prompt before foundry-runtime invocation; redaction Ed25519-sealed for audit.
7. **Audit** (Art. 30 GDPR overlap): every T1 + T2 invocation emits `AiDesignSuggested` or `AiContentGenerated` Ed25519-sealed audit row with prompt_hash + completion_hash + risk_class + provenance + decision (accepted | discarded | refused).
8. **T0 always low-risk** (capability definition; never invokes high-risk).
9. **Rate limits per tenant + per-pack + per-day** for T2: enforced at API + foundry-runtime; per `capabilities/T2-auto.yaml`.
10. **Tenant opt-in required for T2**: T2 default-off in every pack; tenant must opt-in. T2 explicitly default-off in us-healthcare pack regardless.

## Alternatives Considered

### A — Allow T2 always; warn-only on Annex III

- **Pros**: Maximum tenant power; competitive parity with Gamma / Tome / Pitch (which allow generation broadly).
- **Cons**: Almost certainly fails EU AI Act conformance: Art. 16 requires AI providers + deployers to ensure high-risk systems are NOT placed on the market without conformity assessment + CE marking + human oversight + transparency. Warn-only is insufficient.
- **Rejected reason**: legal non-conformance + ethics.

### B — Refuse T2 entirely; offer only T0/T1

- **Pros**: Cleanest legal position; no Annex III exposure.
- **Cons**: Competitive parity lost (Gamma / Tome / Beautiful.ai / Microsoft Copilot Pro all offer T2-class generation). Tenant value Outcome 6 (PRD) explicitly calls for T2.
- **Rejected reason**: parity gap; tenant value gap.

### C — Allow T2; require per-invocation human accept; no Annex III check

- **Pros**: Aligns with Art. 14 human oversight.
- **Cons**: Misses Art. 16 + Annex III obligation; human-oversight is necessary but not sufficient for high-risk classification.
- **Rejected reason**: insufficient.

### D — Slides-side risk-class classifier (slides decides Annex III)

- **Pros**: Slides team owns the classifier.
- **Cons**: Slides team is not the AI risk-class subject-matter expert; foundry-runtime team is. Distributing risk-class logic across 30+ µservices that consume foundry-runtime guarantees inconsistency.
- **Rejected reason**: single-source-of-truth principle.

### E — Per-tenant Annex III override (default-off)

- **Pros**: Tenant agency.
- **Cons**: Tenants are not equipped to judge their own Annex III applicability; cross-pack residency complicates further.
- **Rejected reason**: legal-grade decision should not be tenant-self-served at scale.

### F — Per-pack Annex III pre-allow (e.g., us pack permits Annex III by default)

- **Pros**: Pack-level governance.
- **Cons**: No pack actually wants Annex III by default; the differentiator across packs is residency + retention + PHI-handling, not Annex III gate-default.
- **Rejected reason**: no pack benefit.

## Consequences

### Architectural

- `ai-design` BC crates: `oya-slides-ai-design-{kernel, domain, usecase, api, adapter, sdk}`.
- `ai-content-generation` BC crates: `oya-slides-ai-content-generation-{kernel, domain, usecase, api, adapter, sdk}`.
- Kernel port `RiskClassClient` invokes foundry-runtime SDK; foundry-runtime returns `RiskClassVerdict { class, signature, expiry }`. Slides verifies signature before persistence.
- `provenance-watermark` embedded at usecase layer; preserved through PPTX/PDF/MP4 export per ADR-SLIDES-0003.
- PHI redaction in `accessibility` BC (alt-text) + ai-design + ai-content-generation BCs when pack=us-healthcare.

### Downstream impact on other µservices and IPs

1. **IP-012 (accessibility + ai-design + ai-content-generation)** — authors the gating.
2. **foundry-runtime µservice** — provides risk-class classifier + signature; mandatory dependency.
3. **audit-chain µservice** — every AI invocation Ed25519-sealed.
4. **tenancy µservice** — per-tenant T&C addendum gate.
5. **dpia.md + compliance.md** — cite this ADR for EU AI Act conformance.

### SLOs gaining new dimensions

- `slides.ai_t1_invocation_total` per pack + per capability.
- `slides.ai_t2_invocation_total` per pack + per capability + per usage_context.
- `slides.ai_high_risk_annex_iii_refused_total` per pack.
- `slides.ai_high_risk_annex_iii_engaged_total` per pack (should be near-zero unless pack-override).
- `slides.ai_provenance_watermark_present_rate` — must equal 1.0 on T2 outputs.

### CI lanes added

- `oya-governance-ai-act-risk-class-stamp` — BLOCKER day-1; verifies every AI persistence carries foundry-runtime risk-class signature.
- `oya-governance-ai-provenance-watermark-preserved` — verifies watermark survives export pipelines.

### Risk register

- **Risk**: foundry-runtime risk-class classifier is wrong (false-negative on Annex III). **Mitigation**: external advisory review quarterly; tenant-reportable misclassification feedback loop.
- **Risk**: foundry-runtime classifier latency adds to T2 path. **Mitigation**: classifier on hot-path with cached pack-context defaults; p99 ≤ 100ms classifier overhead budget.
- **Risk**: Tenant T2 prompts containing PHI even when pack is not us-healthcare. **Mitigation**: PHI-detection pre-flight in foundry-runtime; if detected, refusal + tenant notification + per-pack PHI-policy reminder.
- **Risk**: Provenance watermark scrubbed by export pipeline regression. **Mitigation**: lane `oya-governance-ai-provenance-watermark-preserved` runs on every export-pipeline release.
- **Risk**: Per-pack override allowed without proper governance. **Mitigation**: pack-override is a per-deck Cedar grant + tenant T&C addendum + DPO sign-off; not a tenant-self-service flag.

## References

- EU AI Act (Regulation (EU) 2024/1689) Arts. 5, 6, 13, 14, 16, 50 + Annex III.
- EU AI Act timeline — `digital-strategy.ec.europa.eu/policies/regulatory-framework-ai`.
- GDPR Art. 22 (no solely automated decisions).
- GDPR Art. 9 (special category) — interaction with PHI.
- HIPAA §164.530(c) (us-healthcare pack PHI safeguards).
- ADR-WS-0005 (workflow-studio AI-copilot bounds — parent framework).
- PRD FR-29, AC-16.
- dpia.md §3.1 R-6 + R-13.
- compliance.md §"EU AI Act" + §"GDPR".
- foundry-runtime ADR family — risk-class classifier authority.
