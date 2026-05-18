---
id: ADR-NET-0002
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, council-privacy, ops-compliance, ops-security, axis-network, axis-foundry-runtime
owner: axis-network + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0022
  - ADR-0126
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-SOC-0001
  - ADR-SOC-0003
  - ADR-NET-0001
  - ADR-NET-0005
related_artifacts:
  - microservices/network/PRD.md (§Non-Functional Requirements — Audit + Compliance)
  - microservices/network/capabilities/T2-auto.yaml
  - microservices/network/dashboards/recommender-fairness-and-bias.json
  - microservices/network/runbooks/recruiter-classifier-rollback.md
  - microservices/network/slos/recommender-fairness-correctness.openslo.yaml
purpose: |
  Establish the legal + technical bounds for all four T2 ranker sub-capabilities
  (people-you-may-know, jobs-ranker, recruiter-stub, endorsement-aggregation)
  under EU AI Act Annex III §4 (employment) HIGH-RISK classification, EEOC
  UGESP + Title VII + ADA + ADEA, NYC LL144, CA AB-331, CO SB 24-205, IL HB
  3773, UK Equality Act 2010 + ICO ADM guidance, EU Equal Treatment Directives
  2000/43/EC + 2000/78/EC, EU Pay Transparency Directive 2023/970, GDPR Art.
  22 + Art. 25 + Art. 35, and pack-specific employment-law overlays.
---

# ADR-NET-0002: Recommender + recruiter-stub + jobs-ranker + endorsement-aggregation bounds — EU AI Act Annex III §4 HIGH-RISK; full Arts. 9-15 + 27 + 50 + 72 + 73 obligations; EEOC UGESP 4/5-rule + NYC LL144 + CA AB-331 + CO SB 24-205 compliance; GDPR Art. 22 right-to-human-review

## Status

Accepted — 2026-05-17.

## Context

The `network` µservice operates four T2 ranker sub-capabilities (per `capabilities/T2-auto.yaml`):

1. **People-you-may-know (PYMK) recommender** — surfaces candidate connections.
2. **Jobs ranker** — ranks job-search results by candidate-to-job match.
3. **Recruiter-stub ranker** (OFF BY DEFAULT) — ranks candidate-search results for recruiters when activated per tenant + per pack.
4. **Endorsement aggregation** — aggregates per-skill endorsement counts + weighting; affects rank in recruiter + jobs ranker.

All four influence employment-related decisions: PYMK surfacing affects candidate visibility to recruiters; jobs ranker affects which candidates apply; recruiter-stub ranker IS a recruitment AI; endorsement aggregation feeds recruiter-stub and jobs ranker.

**EU AI Act 2024/1689 Annex III §4 classifies AI systems "intended to be used for the recruitment or selection of natural persons, in particular to place targeted job advertisements, to analyse and filter job applications, and to evaluate candidates" as HIGH-RISK.** This applies to all four sub-capabilities — PYMK is a borderline case (the EC has stated in clarifications that recommender systems used in the employment context can fall under §4 because they "filter" candidates by surfacing them).

Concurrent regulatory regimes:

- **GDPR Art. 22** (automated decision-making with significant effect): mandatory right-to-human-review.
- **GDPR Art. 25** (privacy-by-design): minimisation + purpose-limitation.
- **GDPR Art. 35** (DPIA): high-risk processing requires DPIA (`microservices/network/dpia.md`).
- **GDPR Art. 21** (right to object): per-user opt-out toggle.
- **US Title VII** (Civil Rights Act 1964 §703): no employment discrimination on race, color, religion, sex, national origin.
- **ADA** (Americans with Disabilities Act Title I): no disability-based discrimination.
- **ADEA** (Age Discrimination in Employment Act): no age-based discrimination (40+).
- **EEOC UGESP 29 CFR §1607** (Uniform Guidelines on Employee Selection Procedures): 4/5-rule statistical disparity ratio; record-keeping 2 years.
- **NYC Local Law 144-2021** (§§20-870, 20-871, 20-872): annual independent bias audit + 10-business-day prior candidate notice + public summary in DCWP format when AEDT is used in employment decision affecting NYC residents.
- **CA AB-331** (effective Jan 2026) §22756: deployer impact assessment + consumer notice + opt-out.
- **CO SB 24-205** (Colorado AI Act, effective Feb 2026): deployer risk-management policy + algorithmic-discrimination prevention duty.
- **IL HB 3773** (2024): disclosure when AI is used in employment context.
- **UK Equality Act 2010** §13 (direct discrimination) + §19 (indirect discrimination); ICO ADM Code 2024 §6 (meaningful information about logic).
- **EU Equal Treatment Directives 2000/43/EC** (racial equality) + **2000/78/EC** (employment equality — age, disability, sexual orientation, religion).
- **EU Pay Transparency Directive 2023/970**: pay-band publishing transparency; salary insights aggregate-only.
- **EU DSA Art. 27** (recommender transparency): contributing-signals explanation API.
- **KR PIPA Art. 29-2** (automated decision opt-out): per-user opt-out toggle.
- **KR 직장 갑질 protections**: workplace harassment vector cannot be amplified by recommender.

Sibling `social` ADR-SOC-0003 establishes content-moderation classifier bounds under Annex III §1(a) (recommender systems). `network`'s recommender bounds are stricter because §4 (employment) is more sensitive than §1(a) and carries additional EEOC + NYC LL144 + state-AI-law obligations.

The decision must (a) classify all four sub-capabilities under EU AI Act + EEOC + NYC LL144, (b) define the gating regime for recruiter-stub activation (OFF by default), (c) define the per-release bias-audit cadence + 4/5-rule statistical threshold, (d) define the regulatory-notification + rollback path on failure (per `runbooks/recruiter-classifier-rollback.md`), (e) align with parallel ADR-0126's Professional-context invariant.

## Decision

oyatie network adopts the following bounds for all four T2 ranker sub-capabilities:

### 1. EU AI Act HIGH-RISK Classification Confirmed

All four sub-capabilities are classified as Annex III §4 HIGH-RISK. Full Arts. 9-15 + 27 + 50 + 72 + 73 obligations are operative from P01 launch:

- **Art. 9** (risk-management system): risk register maintained per ranker; mitigation tracker in `capabilities/T2-auto.yaml`.
- **Art. 10** (data + data governance): training data SHA recorded; representativeness check; protected-group balance; PII minimisation.
- **Art. 11** (technical documentation): per-release model card sealed (vendor: foundry-runtime).
- **Art. 12** (record-keeping): per-invocation `RecruiterSearchInvoked` event sealed to audit-chain; retention 2y minimum (EEOC UGESP) per pack.
- **Art. 13** (transparency to users): per-output `eu_ai_act_label: ai_generated_assessment`; SDK helper `formatHighRiskDecisionLabel`.
- **Art. 14** (human oversight): mandatory human-review path; appeal workflow per `paths.appeals` in OpenAPI.
- **Art. 15** (accuracy + robustness + cybersecurity): per-release golden-set eval; nDCG@10; adversarial robustness eval.
- **Art. 27** (FRIA): Fundamental-Rights Impact Assessment executed prior to recruiter-stub activation per-tenant.
- **Art. 50** (transparency obligation): UI label "AI-assessed (employment context)".
- **Art. 72** (post-deployment monitoring): continuous bias-audit + drift detector; `dashboards/recommender-fairness-and-bias.json`.
- **Art. 73** (serious-incident reporting): ≤ 15 days to market surveillance authority on bias-audit failure or material disparity event.

### 2. Recruiter-Stub Activation Gating

Recruiter-stub is **OFF BY DEFAULT** for every tenant. Activation requires ALL of the following:

- Tenant-admin explicit opt-in via tenant-admin UI (audit-chain sealed).
- EU AI Act Art. 27 FRIA attested (uploaded; council-privacy review).
- EEOC UGESP 4/5-rule bias-audit passed on the model version intended for use.
- NYC LL144 annual bias audit refreshed within rolling 12 months (when NYC tenant).
- CA AB-331 §22756 impact assessment when CA tenant.
- CO SB 24-205 risk-management policy attached when CO tenant.
- IL HB 3773 disclosure published when IL tenant.
- ops-compliance sign-off.
- council-privacy sign-off.

Cedar `tenant-scope.cedar` PERMIT 7 gates the activation; deactivation can be unilateral by tenant-admin or by ops-compliance / council-privacy under FM-15 incident.

### 3. Bias-Audit Cadence + 4/5-Rule Threshold

- **Per-release**: model version cannot reach production without passing 4/5-rule bias audit (disparity ratio ≥ 0.8) for every tracked protected group (race, gender, age, disability, locale). Golden-set is curated by ops-compliance + council-privacy; periodically refreshed.
- **Continuous**: production usage emits aggregated decisions to `oya.network.recruiter.v1.bias-audit-completed`; bias-audit pipeline computes rolling 30d disparity ratio per protected group; SLO `network-recommender-fairness-correctness` evaluates 100 % target (zero-tolerance for 4/5-rule failure).
- **Annual independent audit**: per NYC LL144 §20-870, third-party annual audit when recruiter-stub activated for NYC tenant.
- **Failure response**: any 4/5-rule failure triggers FM-15 Sev-1 with auto-rollback + EU AI Act Art. 73 serious-incident notification (≤ 15 days) per `runbooks/recruiter-classifier-rollback.md`.

### 4. GDPR Art. 22 Surface

Every user has:
- A profile toggle `automated_decision_opt_out: bool` (default false in P01; tenants may default-on per-pack).
- An SDK helper `setAutomatedDecisionPreference(opt_out: bool)`.
- A right-to-human-review surface at `POST /appeals/{decision_id}` with `request_human_review: true`.

When `automated_decision_opt_out == true`, the user is excluded from PYMK output, jobs ranker output, and recruiter-stub output; their profile is still searchable via deterministic Cedar-filtered Meilisearch, but ranker scores are not applied.

### 5. Minor-Account Protection

Minor accounts (per `policy/data-residency.md` MINOR_PROTECT class) are NEVER surfaced in:
- Recruiter-stub output (regardless of activation status).
- Jobs ranker output for above-18-only roles.
- Salary insights.
- PYMK suggestions (minor accounts get chronological-only feed).

Enforced at Cedar layer (`tenant-scope.cedar` FORBID minor_protect == true clause) + at runtime in ranker output filtering.

### 6. Pack-Aware Overlay Discipline

Per pack overlay in `capabilities/T2-auto.yaml`:
- **pack-us-healthcare**: ranker disabled for PHI accounts per HIPAA §164.502(b).
- **pack-us-nyc**: LL144 audit + candidate notice + public summary mandatory.
- **pack-us-ca**: AB-331 impact assessment + consumer notice mandatory.
- **pack-us-co**: SB 24-205 risk-management policy mandatory.
- **pack-eu**: EU AI Act full obligations + EU DSA Art. 27 + Pay Transparency aggregate-only.
- **pack-uk**: Equality Act §13 + §19 + ICO ADM code §6.
- **pack-au**: AHRC AI guidance + Fair Work Act 2009 conformance.
- **pack-in**: DPDPA 2023 notice + Equal Remuneration Act 1976.
- **pack-kr**: PIPA Art. 28 + Art. 29-2 + 직장 갑질 protection.

## Alternatives Considered

### A. Treat all four sub-capabilities as Art. 13 transparency only (Annex III §4 not invoked)

- Pros: simpler regulatory posture; faster M02 launch; only Art. 50 label needed.
- Cons: legally indefensible — EC clarifications on Annex III §4 explicitly cover recruitment-related recommenders; missing the classification creates non-compliance risk + reputational risk; enforcement guidance is hardening as of 2026.
- Rejected.

### B. Keep recruiter-stub stubbed indefinitely (never activate)

- Pros: lowest regulatory exposure.
- Cons: tenant demand for recruiter-tooling is real; loss of competitive parity with LinkedIn Recruiter + Indeed; gtm-customer-success would lose enterprise accounts; defers rather than solves the AI Act compliance question.
- Rejected: stub-with-strict-activation-gating is the correct posture (this ADR's choice).

### C. Per-tenant whitelist activation (only enterprise tenants with separate legal review)

- Pros: stricter than this ADR's choice; eliminates risk of free-tier activation.
- Cons: doesn't scale; gtm friction; tenant-admin self-service is needed for product viability.
- Partial accept: this ADR's gating (Art. 27 FRIA + NYC/CA/CO conformance + bias-audit) is in effect per-tenant whitelist gating, but mechanised rather than manual.

### D. Open-source the recruiter-stub ranker model

- Pros: external bias-audit scrutiny; community trust.
- Cons: training data may include tenant PII (cannot release); competitive moat lost; doesn't reduce the legal classification (Annex III §4 still applies regardless of openness).
- Rejected for P01; revisit at M04+ if open-weights becomes viable.

### E. Disable algorithmic ranking entirely; ship deterministic skill-match heuristic only

- Pros: lowest AI Act risk; deterministic + auditable from code.
- Cons: uncompetitive vs LinkedIn / Xing / Wantedly recruiter tooling; loss of hero-product viability; tenant churn likely.
- Partial accept: P01 ships heuristic-default with ML-driven ranker deferred to P03 per sibling ADR-SOC-0001 pattern.

## Consequences

### Positive

- All four sub-capabilities operate within EU AI Act + EEOC + NYC LL144 + CA AB-331 + CO SB 24-205 + UK Equality Act + EU Equal Treatment Directives obligations from P01 launch.
- Recruiter-stub OFF-by-default + strict activation gating eliminates accidental rollout to non-prepared tenants.
- Auto-rollback on 4/5-rule failure protects against discriminatory-impact incidents.
- Per-tenant FRIA + LL144 audit creates a defensible record for regulator engagement.
- Bias-audit dashboard + SLO + runbook + ADR-driven evidence pipeline together form a hyperscaler-grade conformance posture.

### Negative

- Tenants in NYC + CA + CO + EU must navigate additional onboarding paperwork (FRIA + LL144 audit + AB-331 impact assessment + SB 24-205 policy); gtm friction.
- Annual independent NYC LL144 audit is a recurring cost (~$10-50k/tenant/year per market quotes).
- Auto-rollback on transient noise (e.g., low-volume tenant with statistical noise) may create operational toil; mitigated by minimum-sample-size gate before triggering 4/5-rule evaluation.
- 4/5-rule threshold of 0.8 is conservative; some legitimate signals may inadvertently push below; mitigated by per-release golden-set eval + manual override with council-privacy sign-off.

### Operational

- Cargo workspace: `oya-network-recruiter-stub-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` + `oya-network-feed-timeline-*` (for PYMK + jobs ranker integration) + `oya-network-endorsement-engine-*` (for endorsement-aggregation feature pipeline).
- LEAN lane: `oya-check-eu-ai-act-employment-conformance` validates FRIA + LL144 audit + AB-331 impact assessment + SB 24-205 policy attached when relevant tenants activate.
- Per-release CI gate: bias-audit must pass 4/5-rule on all protected groups before model-version promotion (per `oya-check-bias-audit-recency`).
- SLO: `network-recommender-fairness-correctness` (zero-tolerance).
- Runbook: `recruiter-classifier-rollback.md`.
- Dashboard: `recommender-fairness-and-bias.json`.

### Regulatory

- **EU AI Act 2024/1689 Arts. 9-15 + 27 + 50 + 72 + 73**: full conformance posture for Annex III §4 high-risk system. Conformity assessment per Art. 43 conducted prior to substantial-use deployment (notified body engagement).
- **EU DSA Art. 27**: recommender transparency surface (`getRankerExplanation`).
- **GDPR Art. 22 + 21 + 35**: opt-out + right-to-human-review + DPIA.
- **EEOC UGESP**: 4/5-rule statistical disparity ≥ 0.8; record-keeping 2y.
- **NYC LL144**: annual audit + 10-day candidate notice + public summary.
- **CA AB-331**: deployer impact assessment + consumer notice + opt-out.
- **CO SB 24-205**: deployer risk-management policy + algorithmic-discrimination prevention.
- **IL HB 3773**: disclosure when AI used in employment.
- **UK Equality Act 2010**: §13 + §19 monitoring; ICO ADM code §6 meaningful info.
- **EU Equal Treatment Directives**: per-protected-group monitoring.
- **EU Pay Transparency Directive 2023/970**: salary insights aggregate-only.
- **KR PIPA Art. 29-2**: per-user opt-out.
- **KR 직장 갑질 protections**: harassment proxy never enters recruiter feature vector.

## References

- ADR-0022 — Bominal autonomy-tier classification (T0/T1/T2 inherited).
- ADR-0126 — Connect dissolution (parallel; Professional-context invariant).
- ADR-0131 — Per-microservice flat layout.
- ADR-0132 — suite-and-bundle dissolution.
- ADR-0133 — Industry best-practice conformance.
- ADR-SOC-0001 — Sibling feed-ranking ADR.
- ADR-SOC-0003 — Sibling content-moderation classifier bounds (paired Annex III §1(a) ADR; reference for pattern).
- ADR-NET-0001 — Storage layer for ranker inputs.
- ADR-NET-0005 — Endorsement-chain integrity (input to recruiter-stub ranker).
- EU AI Act 2024/1689 Annex III §4; Arts. 9-15, 27, 50, 52, 72, 73.
- GDPR Arts. 21, 22, 25, 35.
- EU DSA Regulation (EU) 2065/2022 Art. 27.
- US Title VII Civil Rights Act 1964 §703; ADA Title I; ADEA.
- EEOC UGESP 29 CFR §1607.
- NYC Local Law 144-2021 §§20-870, 20-871, 20-872; DCWP rules.
- CA AB-331 §22756 (effective Jan 2026).
- CO SB 24-205 §6-1-1701 (effective Feb 2026).
- IL HB 3773 (2024).
- UK Equality Act 2010 §§13, 19; ICO ADM Code 2024 §6.
- EU Equal Treatment Directives 2000/43/EC; 2000/78/EC.
- EU Pay Transparency Directive 2023/970.
- KR PIPA Arts. 28, 29-2; KR 직장 갑질 protections; KR 근로기준법.
- HELM benchmark `crfm.stanford.edu/helm`; NIST AI RMF 1.0.
- `microservices/network/dpia.md`; `microservices/network/dashboards/recommender-fairness-and-bias.json`; `microservices/network/runbooks/recruiter-classifier-rollback.md`; `microservices/network/slos/recommender-fairness-correctness.openslo.yaml`.
