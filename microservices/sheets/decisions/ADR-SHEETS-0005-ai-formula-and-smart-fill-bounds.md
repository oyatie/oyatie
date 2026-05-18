---
id: ADR-SHEETS-0005
title: AI-formula + smart-fill scope bounds — T0 + T1 intra-µservice by default; T2 cross-µservice gated by Cedar + ChangeSet review
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, ops-security, council-legal-compliance, axis-foundry-runtime
related: [ADR-0110, ADR-0126, ADR-0131, ADR-0140]
related_external_adrs: [microservices/workflow-studio/decisions/ADR-WS-0005, microservices/docs/decisions/ADR-DOCS-0005]
related_artifacts:
  - microservices/sheets/PRD.md (FR-14, FR-15, AC-05, AC-16)
  - microservices/sheets/capabilities/T0-suggest.yaml
  - microservices/sheets/capabilities/T1-assist.yaml
  - microservices/sheets/capabilities/T2-auto.yaml
  - microservices/sheets/IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md
  - microservices/sheets/threat-model.md (T-S-05, T-I-05)
purpose: Resolve the scope of the sheets AI-formula + smart-fill capabilities — which capability tiers they operate in, EU AI Act classification, and gating for T2 cross-µservice scope.
doc_status: published
---

# ADR-SHEETS-0005: AI-formula + smart-fill — T0 + T1 intra-µservice by default; T2 cross-µservice gated by Cedar + ChangeSet review + 2-person rule

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Per PRD §FR-14 (AI-formula draft from natural-language prose), FR-15 (smart-fill from N seed examples), and the T0/T1/T2 capability framework (per `feedback_quality_performance_scalability_bar.md`), Sheets's AI features span:
- **T0 — Suggest**: formula auto-complete, range suggestion, format suggest. Pure read; no LLM call; sub-millisecond.
- **T1 — Assist**: AI-formula prose-to-formula draft + smart-fill + anomaly-flag. LLM/ML call via foundry-runtime; human reviews + accepts.
- **T2 — Auto**: auto-categorize / auto-cleanup / auto-pivot. Autonomous given human trigger.

Risk dimensions (parallel to workflow-studio ADR-WS-0005):
1. **EU AI Act (Regulation (EU) 2024/1689)**:
   - Annex III §5(b): "AI systems intended to be used for evaluating the creditworthiness of natural persons" — high-risk.
   - Annex III §5(c): "AI systems intended to be used for risk assessment and pricing in life and health insurance" — high-risk.
   - Annex III §4: "AI systems intended to be used in employment, workers management" — high-risk.
   - Sheets is used heavily in finance, HR, insurance — AI-formula drafting in these regulated domains triggers high-risk classification.
   - Arts. 9–15 (high-risk) impose risk management, data governance, technical documentation, record-keeping, transparency, human oversight, accuracy/robustness, post-market monitoring.
   - Arts. 50–55 (transparency) require labeling AI-generated content.
2. **GDPR Art. 22** — solely-automated decisions producing legal effects forbidden with exceptions.
3. **Tenant trust boundary** — sheets AI that drafts cross-µservice flows (e.g., AI-formula that sends data to mail/messenger) widens blast radius.
4. **Supply-chain risk** — LLM-emitted formula has prompt-injection attack surface (OWASP LLM A01).
5. **Audit + reversibility** — every AI action must be reversible via ChangeSet (ADR-0110).

Existing controls:
- Per-tenant LLM choice via foundry-runtime (BYO-LLM).
- PII redactor before LLM call.
- Prompt-injection scrubber.
- Formula-engine grammar validation on completion.
- 90-day archival.

## Decision

Adopt the following capability-tier scope bounds for sheets AI:

### T0 — Suggest (always-on by default; per-tenant opt-out)

- Scope: intra-sheets. Formula auto-complete from the registered ≥ 400-function library, range hints from neighbouring data, format suggestions.
- Cross-µservice scope: **forbidden**. Suggestions reference only sheets's own function library + tenant's own workbooks.
- Autonomy: zero. Suggestions advisory; tenant explicitly accepts each one.
- Cedar gate: presence-check entitlement; no per-action policy.
- EU AI Act posture: not high-risk (purely suggestive, no autonomous output).

### T1 — Assist (opt-in per-tenant; default off)

- Scope: intra-sheets. Three subcapabilities:
  1. **Formula-from-natural-language**: tenant prose → candidate formula in a single cell.
  2. **Smart-fill**: tenant provides 3-10 seed cells → infer column pattern → propose values for target range.
  3. **Anomaly-flag**: foundry-runtime scans column for outliers → "investigate?" tooltips.
- Cross-µservice scope: **forbidden by default at T1**. The AI MUST NOT propose formulas that reference workflow-engine workflows, mail addresses, messenger channels, or other cross-µservice resources at T1.
- Autonomy: bounded. AI drafts; tenant explicitly reviews and accepts/rejects.
- Cedar gate: presence-check (`sheets.ai_formula.consent`) + per-call policy.
- ChangeSet: each accepted T1 draft is one ChangeSet per ADR-0110; reversible.
- EU AI Act posture: **conditional high-risk**. Default classification: limited-risk-AI-system (Annex III §10 advisory drafting). **Conditional high-risk** when used in:
  - Credit-scoring workflows (Annex III §5(b)).
  - Insurance pricing workflows (Annex III §5(c)).
  - Employment-decision workflows (Annex III §4).
  - Tenant must explicitly attest to oyatie at onboarding whether their use-case falls in scope; attestation enables/disables high-risk mode.
- High-risk obligations when triggered:
  - Conformity assessment required pre-deployment (Art. 9).
  - Transparency UI label (Art. 13).
  - Human oversight: explicit-accept (never auto-apply) per FR-14 + FR-15 (Art. 14).
  - Record-keeping: 90d retention of prompt + completion (Art. 12).
  - Post-market monitoring: monthly review of acceptance + revert rate (Art. 72).

### T2 — Auto (opt-in per-tenant + per-capability; default off; cross-µservice gated)

- Scope: T2 has two sub-scopes:
  - **T2-intra**: autonomous recalc, autonomous CRDT merge, autonomous editor-session persistence, auto-categorize, auto-cleanup-data, auto-pivot. These are autonomous given a tenant-initiated trigger.
  - **T2-cross**: tenant requests an action that involves a cross-µservice dispatch (e.g., "auto-cleanup data + trigger workflow-engine on each cleaned row"). **This is the gated case.**
- T2-cross gate:
  1. Tenant must explicitly enable `sheets.ai.t2.auto.cross-microservice` for the specific destination µservice. Enablement is per-(tenant, destination-µservice) pair.
  2. Cedar policy must permit the cross-call.
  3. Candidate action MUST be routed through ChangeSet state machine (ADR-0110); requires human-author + reviewer-agent approval before the foreign µservice's API is invoked.
  4. 2-person rule: AI is the first author; a human OR another agent acting under a different signing key is the required reviewer.
  5. Destination µservice's own SDK contracts apply.
- EU AI Act posture: **T2-cross is high-risk under EU AI Act Arts. 9–15 by default**. Tenant DPIA mandatory before enablement. Post-market monitoring (Art. 72) mandatory.

### Forbidden under all tiers

- AI emitting formulas that bypass per-range ACL (e.g., reads a SECRET-class column). Hard error in adapter; LEAN check.
- AI emitting formulas that touch SECRET-class data without explicit consent. Hard error.
- AI emitting workflows that reference µservices that don't exist OR that tenant has no entitlement to. Hard error; precise diagnostic.
- AI output passing through to apply without formula-engine grammar validation + Cedar preview. Hard error.

## Alternatives Considered

### Alternative A — T0 + T1 intra-sheets only; T2 entirely forbidden

- **Pros**: maximally safe; trivially Art. 22-compliant.
- **Cons**: falls behind Excel Copilot + Google Sheets Gemini; "draft me a workflow when this column updates" is the obvious tenant ask.
- **Rejected**: too restrictive for hero-product ambition.

### Alternative B — Unlimited cross-µservice AI at T1 (no Cedar/ChangeSet gate)

- **Pros**: maximum capability; competitive with freest interpretation of competitor Copilots.
- **Cons**: widens blast radius unconsciously; prompt-injection risk multiplies; EU AI Act high-risk posture for any tenant in regulated industry default-on; reversibility complex.
- **Rejected**: blast radius + supply-chain attack surface + AI Act default-high-risk.

### Alternative C — T2 cross-µservice with Cedar but no ChangeSet review (auto-commit on Cedar pass)

- **Pros**: lower friction.
- **Cons**: Cedar enforces structural rules, not semantic correctness of AI interpretation; AI Act Art. 14 human-oversight harder to defend.
- **Rejected**: Cedar necessary but not sufficient; ChangeSet review is the human-oversight hatch AI Act expects.

### Alternative D — Apps-Script-equivalent (full programmatic AI authoring + execution)

- **Pros**: parity with Google Apps Script.
- **Cons**: enormous attack surface; programmatic execution inside Sheets contradicts editor-execution-forbidden invariant (T-E-04); AI Act high-risk classification near-universal.
- **Rejected at M03**: full Apps-Script-equivalent deferred to post-GA T2 review.

## Consequences

### Architectural

- `oya-sheets-ai-formula-adapter` implements the tier-aware adapter:
  - T0/T1 path: bounded to sheets's function library + per-range ACL.
  - T2-cross path: Cedar entity construction + ChangeSet draft creation + reviewer-agent invocation + 2-person rule enforcement.
- The formula-engine grammar validator rejects AI-emitted formulas that violate tier scope; structural enforcement defense-in-depth.
- `AiFormulaDraft{tenant, draft_id, prompt_hash, completion_hash, accepted_at, tier, cross_microservice_destinations[], ai_act_high_risk}` is the audit-chain payload.

### Downstream impact

1. **IP-011 (AI-formula bridge)** implements tier scope bounds.
2. **foundry-runtime µservice** — LLM-call envelope carries tier + scope; archives for audit.
3. **tenancy µservice** — per-tenant entitlements granular per destination.
4. **workflow-engine µservice** — T2-cross drafts arrive via ChangeSet state machine.
5. **observability µservice** — `recalc-engine-health.json` (and a future `ai-quality.json`) gains AI-formula SLIs.
6. **council-legal-compliance** — tenant-side DPIA template + per-tenant T2-cross enablement workflow.

### CI lanes + SLOs

- `oya-governance-cedar-preview-required` — exercised for every AI output.
- `sheets.ai_formula_t2_cross_safety_signal_count` — Sev-1 if non-zero in any 24h window.
- `sheets.ai_formula_forbidden_output_blocked_count` — informational SLI.
- `sheets.ai_formula_t2_cross_acceptance_rate` — quality SLI.

### Compliance + audit

- Every AI invocation emits `AiFormulaDraftRequested` with tier + scope.
- 90-day archival of prompt + completion.
- T2-cross drafts emit additional `AiFormulaT2CrossDraftReviewerAssigned` + `AiFormulaT2CrossDraftReviewed` events.
- EU AI Act post-market monitoring report quarterly.
- DPIA (per `dpia.md`) updated to reflect tier-aware AI scope.

### Risk register

- **Risk**: T2-cross gate fatigue — tenant disables ChangeSet review. **Mitigation**: per-tenant Cedar entitlement granular per destination; allowlist authored under `microservices/sheets/specs/`.
- **Risk**: Prompt-injection succeeds at emitting out-of-scope formula. **Mitigation**: structural rejection in grammar validator + Cedar preview; SLO on forbidden-output-blocked-count.
- **Risk**: EU AI Act classification evolves; T1 posture becomes high-risk by default. **Mitigation**: quarterly compliance review; ADR-SHEETS-0005 supersession path.
- **Risk**: Tenant BYO-LLM emits non-deterministic output. **Mitigation**: formula-engine grammar validation enforced; non-conforming output rejected.

## References

- PRD `microservices/sheets/PRD.md` FR-14, FR-15, AC-05, AC-16.
- `microservices/sheets/capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
- `microservices/sheets/IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md`.
- `microservices/sheets/dpia.md`.
- `microservices/sheets/compliance.md`.
- `microservices/sheets/competitor-parity-matrix.md` §"AI + automation", §"Claim-Boundary Rules".
- ADR-WS-0005 — workflow-studio AI-copilot bounds.
- ADR-DOCS-0005 — docs AI-assist bounds.
- Regulation (EU) 2024/1689 (EU AI Act) — Arts. 9, 10, 11, 12, 13, 14, 15, 50, 72; Annex III §4, §5(b), §5(c), §10.
- Regulation (EU) 2016/679 (GDPR) Art. 22.
- OWASP Top 10 for Large Language Model Applications.
- ADR-0110 — ChangeSet state machine.
- ADR-0126 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
- ADR-0140 — Cedar policy enforcement.
