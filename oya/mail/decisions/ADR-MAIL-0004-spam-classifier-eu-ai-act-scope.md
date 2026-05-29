---
id: ADR-MAIL-0004
status: Accepted
date: 2026-05-17
microservice: mail
deciders: council-privacy, axis-mail, ops-legal, pack-eu-council
owner: council-privacy + axis-mail
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-MAIL-0001
related_artifacts:
  - microservices/mail/PRD.md (FR-10 DLP/abuse detection)
  - microservices/mail/capabilities/T1-assist.yaml (T1-mail-smart-classifier — EU AI Act trigger)
  - microservices/mail/dpia.md (R-04 DLP)
  - microservices/mail/runbooks/spam-rule-rollback.md
  - microservices/mail/runbooks/dlp-quarantine-release.md
purpose: Close the EU AI Act scoping gap surfaced by capabilities/T1-assist.yaml — establish when the inbound spam/phishing classifier is Annex III high-risk vs limited-risk, and how tenants opt into the high-risk conformity assessment.
---

# ADR-MAIL-0004: Spam + phishing + DLP classifier EU AI Act scope — Annex III-exempt by default; tenant-opt-in conformity assessment when scoped to employment / HR mail

## Status

Accepted — 2026-05-17.

## Context

The `mail` µservice ships an inbound classifier (Rspamd + tenant-fine-tuned ML model fusion) that decides `spam | phish | malware | dlp | clean` per inbound message and a paired outbound DLP classifier that decides `block | quarantine | allow`. Both are documented in `capabilities/T1-assist.yaml` capability `T1-mail-smart-classifier`. The capability's `eu_ai_act_classification` field already flags the open question:

```yaml
eu_ai_act_classification:
  - default: limited-risk
  - high-risk-trigger: when classifier is applied to employment-context mail
    (Annex III §3 + 4 employment recruitment) — conformity assessment required
    before deployment per pack-eu overlay
```

The EU AI Act (Regulation (EU) 2024/1689) creates two scoping cliffs:

- **Limited-risk (Arts. 50 + 52)**: transparency obligations only — user-facing labels, audit trail, no conformity assessment. Default scoping for productivity-tool classifiers.
- **High-risk (Annex III + Arts. 9–15 + Annex IV)**: pre-deployment conformity assessment, post-market monitoring, technical documentation, risk-management system, human oversight, training-data governance, accuracy + robustness budgets, post-market monitoring. Triggered for systems intended to be used in Annex III §3 (employment context: recruitment, performance evaluation) and §4 (workers' management).

A spam/phishing classifier applied to general productivity mail is plainly limited-risk: it filters unwanted communication; it does not make decisions about persons. The same classifier applied to an HR mailbox (e.g., "applications@") OR to a workflow that mines mail content to score candidates OR to outbound DLP that blocks specific employees' communications about working conditions — that's a different beast. The Annex III §3-4 scope is intent-and-effect-shaped, not classifier-shaped.

Blanket high-risk classification overscopes (kills the feature for ~99% of tenants where it's plainly limited-risk and adds enormous compliance burden to oyatie). Blanket limited-risk underscopes (creates regulatory exposure for the 1% of tenants whose HR workflow genuinely brings Annex III §3 into scope). Self-declared scoping by tenants is the only honest reading of Annex III, which is intent-shaped by Recital 56.

This question was surfaced as a derived gap, not in the PRD Open Questions section directly, but the capability file (`T1-assist.yaml`) and pack-eu DPIA explicitly flag it. The capability cannot ship without resolution.

## Decision

oyatie's inbound + outbound mail classifier is **scoped Annex III-exempt (limited-risk) by default**. Tenants opt into Annex III §3-4 conformity-assessed deployment when their declared HR-mail workflow brings classifier output into Annex III scope. Concretely:

1. **Default deployment — limited-risk (Arts. 50 + 52)**:
   - User-facing label "AI-filtered" / "Quarantined by AI" on every classifier-affected message per Art. 50.
   - Per-decision audit-chain record (top-3 contributing rules + scores) per Art. 13 transparency.
   - User reversal always available ("Not Spam" / "Release from Quarantine") per Art. 14 human oversight floor.
   - Per-tenant FP/FN budgets tracked in `cost-budget.md` per Art. 15 accuracy bar.
   - No conformity assessment required; no Annex IV technical documentation pack.

2. **Tenant-opt-in deployment — Annex III §3-4 high-risk (Arts. 9–15 + Annex IV)**:
   - Triggered by tenant declaring `hr_mail_workflow: true` in their tenant configuration, or by oyatie detecting per Cedar policy that the classifier output is consumed by an HR workflow (employment-decision evidence in `workflow-engine`). Detection is conservative — the trigger fires on declared intent + observed consumption pattern, not on guesswork.
   - On trigger: oyatie ships the tenant a per-tenant **Annex IV technical documentation pack** (training data audit, FP/FN budget evidence, accuracy benchmarks, model card, intended-use statement, post-market-monitoring plan).
   - Risk-management system per Art. 9 documented in `microservices/mail/dpia.md` R-04 with a per-tenant overlay file at `microservices/mail/dpia/tenants/<tenant>-hr-overlay.md` (created on trigger).
   - Conformity assessment per Annex VI is the tenant's obligation; oyatie supplies the documentation pack that the tenant submits to its assessor. oyatie does NOT perform the conformity assessment itself (we are the provider; the tenant is the deployer per Art. 3 definitions).
   - Post-market monitoring per Art. 72 — per-tenant FP/FN telemetry sealed in audit-chain with quarterly review cadence.
   - Human-oversight per Art. 14 elevated from "user reversal" to "HR-officer review on every classifier-triggered HR-mail decision" — the workflow MUST present the verdict to an HR officer for ratification before any employment action follows.

3. **Tenant cannot deploy classifier in HR scope WITHOUT opt-in**: Cedar policy refuses classifier execution on `hr_mail_workflow=true` tenant mailboxes until conformity-assessment evidence is uploaded. The refusal is a hard block, not a warning.

4. **Pack-eu overlay enforces the rule structurally**: pack-eu tenants pass `ai-act-conformance` CI lane validation before mail-classifier deployment promotes past dev.

5. **No retroactive HR-classifier**: if a tenant later toggles `hr_mail_workflow=true`, classifier decisions made prior to the toggle are NOT retroactively recharacterised as high-risk; the tenant must run conformity assessment forward from the toggle date. Historical decisions remain in audit-chain as limited-risk decisions.

## Alternatives Considered

### A. Blanket high-risk (every mail classifier is Annex III by default)
- Pros: maximum regulatory safety; no underscope risk; signals seriousness to regulated tenants.
- Cons: overscope — every starter/pro tenant pays the Annex IV documentation burden for a feature that is plainly productivity-tool-filtering; kills the spam/phishing classifier's economic viability for ~99% of tenants; effectively makes oyatie mail unable to filter spam for most tenants because the conformity assessment fully blocks deployment.
- Rejected: Annex III intent-and-effect shape (Recital 56) is unambiguous — spam filtering qua spam filtering is not in scope.

### B. Blanket limited-risk (every mail classifier is limited-risk, no opt-in mechanism)
- Pros: simplest implementation; lowest compliance overhead.
- Cons: tenants whose HR workflow genuinely brings classifier output into Annex III §3-4 face regulatory exposure they cannot mitigate within oyatie's product surface; the EU AI Act treats the deployer as on the hook, but the provider (oyatie) is on the hook for technical documentation per Art. 11; failing to provide that documentation when scope applies is a provider-side violation.
- Rejected: underscope; creates joint provider/deployer regulatory exposure that neither party can fix.

### C. Tenant-selectable per-tenant flag (the choice, generalised)
- Pros: matches the EU AI Act's intent-and-effect shape; tenant declares scope; oyatie supplies the artifact that matches declared scope.
- Accepted as the chosen mechanism; this ADR's Decision §2 formalises it.

### D. Mailbox-granular flag (per-mailbox, not per-tenant)
- Pros: finer scope; HR mailbox can be flagged while general mailboxes stay limited-risk.
- Cons: implementation complexity (cross-mailbox classifier model needs per-decision scope check); ops surface (every tenant configures every mailbox); auditability gets harder (one tenant has a mix of decision types in audit-chain).
- Rejected: tenant-granular is sufficient for the legal scoping; mailbox-granular adds complexity without commensurate scope-clarity benefit. Tenants who genuinely mix HR + non-HR mail at scale should split into separate tenants (sub-tenant pattern per `tenancy` µservice).

### E. Self-attestation only (tenant declares; oyatie doesn't enforce technical documentation)
- Pros: minimum oyatie-side implementation.
- Cons: provider-side obligation under Art. 11 (technical documentation) is on oyatie regardless of tenant declaration; not supplying it when scope applies is an oyatie-side violation. Self-attestation is necessary but not sufficient.
- Rejected: half-measure that leaves oyatie exposed.

## Consequences

### Positive

- Honest EU AI Act scope: limited-risk where the classifier IS limited-risk, high-risk where the tenant's intent + workflow brings Annex III §3-4 into scope. Matches Recital 56's intent-shape.
- Operational simplicity for ~99% of tenants: no conformity-assessment burden, no Annex IV documentation pack, classifier ships as a default-on productivity feature.
- Regulatory cover for the 1% of tenants who genuinely need HR-mail-scope conformity assessment: oyatie supplies the Annex IV pack; tenant runs the assessment; pack-eu overlay enforces non-bypass.
- Capability `T1-mail-smart-classifier` in `T1-assist.yaml` moves from "open question" to "specified"; the YAML's `high-risk-trigger` clause becomes implementable.
- New CI lane `ai-act-conformance --microservice mail` (per PHASE-01 §"Fitness lane gates") becomes concrete — it validates Annex IV documentation pack presence for HR-scope tenants.

### Negative

- Detection of "HR-mail workflow consumption" must be honest: oyatie cannot conclusively detect every employment-decision workflow that consumes classifier output. We mitigate with (a) tenant self-declaration as the primary signal, (b) Cedar-policy-observable workflow-engine consumption patterns as a secondary signal, (c) annual tenant audit where the tenant attests scope.
- The `hr_mail_workflow=true` toggle is a high-friction step for tenants who legitimately need it; we minimise friction with a one-click toggle + an Annex IV documentation pack pre-generated by oyatie's compliance tooling.
- Tenants who toggle on (high-risk) cannot toggle off (back to limited-risk) without a fresh DPIA reassessment because their historical HR-decision corpus is now Annex III-shaped; toggle-off is supported only at the start of a new fiscal year with the prior corpus archived under retention.
- Audit-chain records gain a `eu_ai_act_classification: limited-risk | high-risk` field per decision; downstream consumers (workflow engines, eDiscovery export) must respect the field; backwards-compat handled via default `limited-risk` for pre-toggle decisions.

### Operational

- `microservices/mail/capabilities/T1-assist.yaml` updates the `T1-mail-smart-classifier` capability's `eu_ai_act_classification` field to point at this ADR. (Update is part of the IP that lands this ADR.)
- New tenant-configuration field `hr_mail_workflow: bool` defaults `false`; toggling to `true` triggers the conformity-assessment workflow + pre-flight documentation generation.
- Annex IV documentation pack auto-generated from oyatie's compliance tooling: training-data audit (`microservices/mail/evidence/ai-act/training-data-audit-<ts>.json`), accuracy benchmarks (per-tenant FP/FN historical), model card (`microservices/mail/evidence/ai-act/model-card-<tenant>-<ts>.md`), intended-use statement, post-market-monitoring plan.
- Per-tenant DPIA overlay: `microservices/mail/dpia/tenants/<tenant>-hr-overlay.md` written on toggle-on; reviewed quarterly.
- Cedar policy fragment `microservices/mail/policy/eu-ai-act-hr-scope.cedar` (NEW) refuses classifier execution on `hr_mail_workflow=true` tenants without conformity-assessment evidence; conformity-assessment evidence file path bound by tenant config.
- Post-market monitoring: per-decision FP/FN telemetry sealed in audit-chain (already; this ADR adds the AI-Act-specific quarterly review cadence).

### Regulatory

- **EU AI Act (Regulation (EU) 2024/1689)**:
  - **Art. 3** — provider vs deployer distinction: oyatie is provider; tenant is deployer.
  - **Arts. 9, 10, 11, 13, 14, 15** — high-risk system obligations (when triggered): risk management, data governance, technical documentation, transparency, human oversight, accuracy + robustness.
  - **Art. 50** + **Art. 52** — transparency obligations for limited-risk and high-risk systems alike.
  - **Art. 72** — post-market monitoring.
  - **Annex III §3 (employment) + §4 (workers' management)** — the trigger surface.
  - **Annex IV** — technical documentation pack contents.
  - **Annex VI** — conformity assessment procedure.
  - **Recital 56** — intent-and-effect-shape clarification.
- **GDPR Art. 22** — automated decision-making safeguards apply orthogonally; user reversal + human-oversight obligations honoured.
- **KR PIPA Art. 22-2** (automated decisions): out of scope for EU AI Act but separately addressed in pack-kr DPIA overlay; classifier output respects KR PIPA Art. 22-2 right-to-object via the user-reversal path.
- **HIPAA**: out of scope (classifier ML doesn't disclose PHI; pack-us-healthcare overlay further restricts model deployment to PHI-trained model only).

## References

- EU AI Act (Regulation (EU) 2024/1689) — Arts. 3, 9, 10, 11, 13, 14, 15, 50, 52, 72; Annex III §§3-4; Annex IV; Annex VI; Recital 56
- GDPR (Regulation (EU) 2016/679) — Art. 22 (automated decision-making)
- KR PIPA Arts. 22-2, 23, 28
- HIPAA 45 CFR §164.502(b) (minimum necessary), §164.312 (technical safeguards)
- ENISA AI Act Conformity Guidance (2025)
- EDPB Guidelines on AI Act + GDPR interaction (2025)
- NIST AI RMF (Risk Management Framework) v1.0
- Rspamd documentation — `https://rspamd.com/doc/`
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-platform-and-bundle dissolution
- ADR-0133 — Industry best-practice conformance program
- ADR-MAIL-0001 — Personal-pillar key recovery (companion privacy posture)
- `microservices/mail/PRD.md` FR-10
- `microservices/mail/capabilities/T1-assist.yaml` `T1-mail-smart-classifier`
- `microservices/mail/dpia.md` R-04
- `microservices/mail/runbooks/spam-rule-rollback.md`
- `microservices/mail/runbooks/dlp-quarantine-release.md`
