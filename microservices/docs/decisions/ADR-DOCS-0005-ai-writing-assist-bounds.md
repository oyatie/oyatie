---
id: ADR-DOCS-0005
title: AI writing-assist EU AI Act bounds — Annex III-exempt by default; tenant-opt-in conformity assessment when HR-context
microservice: docs
status: Accepted
date: 2026-05-17
owner: council-privacy + axis-docs
deciders: council-privacy, axis-docs, ops-legal, pack-eu-council
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0132, ADR-0133, ADR-MAIL-0004, ADR-WS-0005]
related_artifacts:
  - microservices/docs/PRD.md (FR-14)
  - microservices/docs/capabilities/T0-suggest.yaml
  - microservices/docs/capabilities/T1-assist.yaml
  - microservices/docs/capabilities/T2-auto.yaml
  - microservices/docs/dpia.md (R-16, R-17)
purpose: |
  Settle the EU AI Act scoping for docs AI writing-assist capabilities. Each
  T0/T1/T2 capability flag carries `eu_ai_act_classification`; this ADR
  authoritatively settles the trigger surface for high-risk classification.
  Aligns with ADR-MAIL-0004 (mail classifier) and ADR-WS-0005 (workflow-studio
  AI copilot) patterns.
doc_status: published
---

# ADR-DOCS-0005: AI writing-assist EU AI Act scope — Annex III-exempt by default; tenant-opt-in conformity assessment when scoped to HR-document workflows

## Status

Accepted — 2026-05-17.

## Context

The `docs` µservice ships a tiered AI writing-assist (T0 grammar / title suggestion / TOC suggestion / formatting suggestion / link suggestion; T1 auto-summary / expand-rewrite / citation suggest / grammar bulk fix; T2 auto-translate / auto-format / auto-cite / auto-summary-on-publish). Per `capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`, each capability already flags an `eu_ai_act_classification` field; the open question is which capability shape triggers Annex III high-risk.

The EU AI Act (Regulation (EU) 2024/1689) creates two scoping cliffs:

- **Limited-risk (Arts. 50 + 52)**: transparency obligations only — user-facing labels, audit trail, no conformity assessment. Default scoping for productivity-tool AI.
- **High-risk (Annex III + Arts. 9–15 + Annex IV)**: pre-deployment conformity assessment, post-market monitoring, technical documentation, risk-management system, human oversight, training-data governance, accuracy + robustness budgets. Triggered for systems intended to be used in Annex III §3 (employment context: recruitment, performance evaluation) and §4 (workers' management).

A grammar suggestion or auto-summary applied to a personal note or business memo is plainly limited-risk. The same suggestion applied to a job-interview summary, a candidate evaluation, a performance review, or HR-policy documents falls into Annex III §3-4 because the AI's output shapes employment decisions. The scope is intent-and-effect-shaped per Recital 56 — classifier-shape alone doesn't determine it.

Blanket high-risk overscopes (kills the feature for ~99% of tenants). Blanket limited-risk underscopes (creates regulatory exposure for the 1% of tenants who genuinely run HR-document workflows). Self-declared scoping by tenants is the only honest reading.

This ADR mirrors the pattern established by ADR-MAIL-0004 (mail spam/phishing classifier) and ADR-WS-0005 (workflow-studio AI copilot) — both settled the same scoping question for sibling µservices.

## Decision

Docs AI writing-assist capabilities are **scoped Annex III-exempt (limited-risk) by default**. Tenants opt into Annex III §3-4 conformity-assessed deployment when their declared HR-document workflow brings classifier output into Annex III scope.

### Default deployment — limited-risk (Arts. 50 + 52)

- User-facing label "AI suggestion" / "AI applied" / "AI auto-translated" on every AI-affected surface per Art. 50.
- Per-decision audit-chain record (capability id + reversibility-window expiry) per Art. 13 transparency.
- User reversal always available (cancel-within-window for T1; revoke-policy + delete for T2) per Art. 14 human-oversight floor.
- Per-tenant accuracy + reversal-rate budget tracked in `cost-budget.md` + dashboards per Art. 15 accuracy bar.
- No conformity assessment required; no Annex IV technical documentation pack.

### Tenant-opt-in deployment — Annex III §3-4 high-risk (Arts. 9–15 + Annex IV)

- Triggered by tenant declaring `hr_document_workflow: true` in tenant configuration, OR by oyatie detecting per Cedar policy that classifier output is consumed by an HR workflow (employment-decision evidence in `workflow-engine`).
- On trigger: oyatie ships the tenant a per-tenant **Annex IV technical documentation pack** (training data audit, accuracy benchmarks, model card, intended-use statement, post-market-monitoring plan).
- Risk-management system per Art. 9 documented in `microservices/docs/dpia.md` R-16/R-17 with a per-tenant overlay file at `microservices/docs/dpia/tenants/<tenant>-hr-overlay.md` (created on trigger).
- Conformity assessment per Annex VI is the tenant's obligation; oyatie supplies the documentation pack that the tenant submits to its assessor. Per Art. 3 definitions: oyatie is provider; tenant is deployer.
- Post-market monitoring per Art. 72 — per-tenant accuracy telemetry sealed in audit-chain with quarterly review cadence.
- Human-oversight per Art. 14 elevated from "user reversal" to "HR-officer review on every classifier-triggered HR-document decision".

### Cedar enforcement (hard block before evidence on file)

Cedar policy `microservices/docs/policy/ai-act-hr-scope.cedar` (NEW; landing in IP-014) refuses T1/T2 capability execution on `hr_document_workflow=true` tenant docs until conformity-assessment evidence is uploaded. The refusal is a hard block, not a warning. Pack-eu overlay enforces structurally — pack-eu tenants pass `ai-act-conformance` CI lane validation before promotion past dev.

### No retroactive HR-classifier

If a tenant later toggles `hr_document_workflow=true`, AI-assist outputs generated prior to the toggle are NOT retroactively recharacterised as high-risk; the tenant runs conformity assessment forward from the toggle date.

## Alternatives Considered

### A. Blanket high-risk (every AI writing-assist is Annex III by default)

- Pros: maximum regulatory safety; no underscope risk.
- Cons: overscope — every starter/pro tenant pays the Annex IV documentation burden for a productivity feature; kills feature for ~99% of tenants; conformity-assessment fully blocks deployment.
- Rejected: Annex III intent-and-effect shape (Recital 56) is unambiguous — grammar-check / auto-summary qua productivity-assist is not in scope unless used for HR.

### B. Blanket limited-risk (no opt-in mechanism)

- Pros: simplest implementation.
- Cons: tenants whose HR workflow genuinely brings classifier output into Annex III §3-4 face exposure they cannot mitigate within oyatie's product; provider-side (oyatie) is on hook for Annex IV technical documentation per Art. 11; failure to supply that documentation when scope applies is a provider violation.
- Rejected: underscope; creates joint provider/deployer regulatory exposure.

### C. Tenant-selectable per-tenant flag (the choice; generalised) — **CHOSEN**

- Pros: matches the EU AI Act's intent-and-effect shape; tenant declares scope; oyatie supplies artifact matching declared scope.
- Accepted as the chosen mechanism; the decision §2 formalises it. Mirrors ADR-MAIL-0004 + ADR-WS-0005 pattern.

### D. Per-document flag (per-doc, not per-tenant)

- Pros: finer scope; HR-doc flagged while regular docs stay limited-risk.
- Cons: implementation complexity (per-decision scope check); auditability gets harder (one tenant has mix of decision types in audit-chain). Tenants who mix HR + non-HR docs at scale should split into separate workspaces (sub-tenancy).
- Rejected: tenant-granular is sufficient for legal scoping; per-doc adds complexity without commensurate scope-clarity benefit.

### E. Self-attestation only (tenant declares; oyatie doesn't enforce technical documentation)

- Pros: minimum oyatie-side implementation.
- Cons: provider-side obligation under Art. 11 is on oyatie regardless of tenant declaration. Not supplying when scope applies is an oyatie-side violation. Self-attestation necessary but not sufficient.
- Rejected: half-measure that leaves oyatie exposed.

## Consequences

### Architectural

- Capability YAMLs (T0/T1/T2) updated with `eu_ai_act_classification` and `hr-context-trigger` clauses pointing to this ADR.
- New tenant-configuration field `hr_document_workflow: bool` defaults `false`; toggling to `true` triggers conformity-assessment workflow + pre-flight documentation generation.
- Annex IV documentation pack auto-generated from oyatie compliance tooling: training-data audit, accuracy benchmarks, model card, intended-use statement, post-market-monitoring plan.
- Per-tenant DPIA overlay: `microservices/docs/dpia/tenants/<tenant>-hr-overlay.md` written on toggle-on; reviewed quarterly.
- Cedar policy fragment `microservices/docs/policy/ai-act-hr-scope.cedar` refuses T1/T2 execution on `hr_document_workflow=true` tenants without conformity-assessment evidence.
- Post-market monitoring: per-decision accuracy telemetry sealed in audit-chain.

### Downstream impact

1. **PRD FR-14** — directly specified.
2. **dpia.md R-16/R-17** — risks documented + mitigated.
3. **Capabilities (T0/T1/T2)** — `eu_ai_act_classification` fields point to this ADR.
4. **CI lane `oya-governance-ai-act-conformance`** (NEW; BLOCKER for pack-eu) — validates Annex IV documentation pack presence for HR-scope tenants.
5. **Pack-eu overlay** — refuses T1/T2 HR-context until evidence on file.
6. **Cross-µservice consistency**: ADR-MAIL-0004 + ADR-WS-0005 + this ADR share the same pattern; tenants opting in for one µservice can leverage the same documentation pack for the others.

### Negative

- Detection of "HR-document workflow consumption" must be honest: oyatie cannot conclusively detect every employment-decision workflow. Mitigated by (a) tenant self-declaration as primary signal, (b) Cedar-policy-observable workflow-engine consumption patterns as secondary signal, (c) annual tenant audit.
- `hr_document_workflow=true` toggle is high-friction; minimised with one-click toggle + pre-generated Annex IV pack.
- Tenants who toggle on cannot toggle off without fresh DPIA reassessment (their historical HR-decision corpus is now Annex III-shaped).

### Regulatory

- **EU AI Act (Regulation (EU) 2024/1689)**:
  - **Art. 3** — provider vs deployer distinction: oyatie is provider; tenant is deployer.
  - **Arts. 9, 10, 11, 13, 14, 15** — high-risk obligations (when triggered).
  - **Art. 50** + **Art. 52** — transparency obligations for both limited and high-risk.
  - **Art. 72** — post-market monitoring.
  - **Annex III §3 + §4** — trigger surface.
  - **Annex IV** — technical documentation pack contents.
  - **Annex VI** — conformity assessment procedure.
  - **Recital 56** — intent-and-effect-shape clarification.
- **GDPR Art. 22** — automated decision-making safeguards apply orthogonally.
- **KR PIPA Art. 22-2** (automated decisions) — orthogonal; user-reversal honors KR PIPA Art. 22-2 right-to-object.
- **HIPAA**: orthogonal (pack-us-healthcare further restricts AI deployment to PHI-trained model only; OPSWAT scan; BAA-bound LLM provider).

## References

- EU AI Act (Regulation (EU) 2024/1689) — Arts. 3, 9, 10, 11, 13, 14, 15, 50, 52, 72; Annex III §§3-4; Annex IV; Annex VI; Recital 56.
- GDPR (Regulation (EU) 2016/679) — Art. 22.
- KR PIPA Arts. 22-2, 23, 28.
- HIPAA 45 CFR §164.502(b) (minimum necessary), §164.312 (technical safeguards).
- ENISA AI Act Conformity Guidance (2025).
- EDPB Guidelines on AI Act + GDPR interaction (2025).
- NIST AI RMF (Risk Management Framework) v1.0.
- ADR-MAIL-0004 — mail spam/phishing classifier EU AI Act scope (sibling pattern).
- ADR-WS-0005 — workflow-studio AI copilot EU AI Act scope (sibling pattern).
- ADR-0131 — per-microservice flat layout.
- ADR-0132 — product-platform + bundle dissolution.
- ADR-0133 — industry best-practice conformance program.
- `microservices/docs/PRD.md` FR-14.
- `microservices/docs/capabilities/{T0-suggest,T1-assist,T2-auto}.yaml`.
- `microservices/docs/dpia.md` R-16/R-17.
