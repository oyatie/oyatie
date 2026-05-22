---
id: ADR-SOC-0003
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-privacy, axis-social, axis-foundry-runtime, ops-security, ops-compliance, ops-legal
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0022
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-SOC-0001
  - ADR-MSGR-0003
related_artifacts:
  - microservices/social/PRD.md
  - microservices/social/capabilities/T2-auto.yaml
  - microservices/social/runbooks/content-moderation-rollback.md
  - microservices/social/runbooks/abuse-report-backlog-drain.md
  - microservices/social/threat-model.md (T-T-06)
  - microservices/social/dpia.md (R-13, R-14)
  - microservices/social/compliance.md (EU AI Act + EU DSA sections)
purpose: Confirm EU AI Act 2024/1689 Annex III §1(a) high-risk classification for the social content-moderation classifier + ranking model; define the obligations satisfaction posture; align with sibling messenger spam-classifier pattern.
---

# ADR-SOC-0003: Content-moderation classifier bounds — EU AI Act 2024/1689 high-risk classification; Arts. 9-15 + Art. 50 + Art. 73 obligations operative; alignment with messenger + mail spam-classifier pattern

## Status

Accepted — 2026-05-17.

## Context

The social µservice's `content-moderation` BC + `feed-timeline` BC ranking sub-capability invoke foundry-runtime T2 classifier + ranking models that:

- Issue verdicts (`spam`, `abuse`, `hate-speech`, `csam-suspect`, `self-harm-suspect`, `harassment`, `sensitive-media-unflagged`) on every user post and comment.
- Apply auto-hide (reversible via appeal per EU DSA Art. 20) on high-confidence verdicts.
- Rank feed entries when user selects algorithmic mode (per ADR-SOC-0001).

EU AI Act 2024/1689 Annex III §1(a) classifies as **HIGH-RISK** systems used for "recommender systems including content moderation systems used by very large online platforms within the meaning of Regulation (EU) 2022/2065 [EU DSA]". The social µservice's classifier + ranking fall squarely within this definition once the deploying tenant qualifies as a VLOP, and the EU AI Act's risk-management posture is required even for non-VLOP deployments because the EU AI Act applies based on the *system's design and intended use*, not just on deployer-scale.

Arts. 9-15 require:
- Art. 9: Risk-management system across lifecycle.
- Art. 10: Data + governance (training-dataset quality, bias-audit).
- Art. 11: Technical documentation (model card).
- Art. 13: Transparency to users (Art. 50 transparency-label).
- Art. 14: Human oversight (manual reviewer in appeal workflow).
- Art. 15: Accuracy + robustness + cybersecurity (reference-set eval + adversarial robustness eval).
- Art. 50: Transparency obligation (user-facing label "AI-assessed").
- Art. 52: Codes of conduct (voluntary alignment).
- Art. 73: Serious incident reporting to market surveillance authority.

EU DSA Regulation (EU) 2022/2065 Arts. 16, 17, 20, 24, 27 impose paired obligations:
- Art. 16: Notice-and-action mechanism.
- Art. 17: Statement of Reasons per moderation verdict.
- Art. 20: Internal complaint-handling (appeal) within 6 months (oyatie SLA: 7 days).
- Art. 24: Transparency report (quarterly).
- Art. 27: Recommender system transparency.
- Art. 28: Online protection of minors.

Sibling µservices have paired classifier patterns:
- `messenger` ADR-MSGR-0003 covers messenger's moderation classifier within its E2E-bounded surface (Personal DMs are server-blind; only Professional channels expose classifier input).
- `mail` (planned ADR-MAIL-0004) covers spam-classifier; same high-risk obligations since mail spam classification affects user-visible inbox routing.

KR PIPA Art. 29-2 requires opt-out + explanation for automated decisions; UK Online Safety Act 2023 requires safety-by-design report + illegal-content duty.

Pack-us-healthcare introduces HIPAA Safe Harbor §164.514: no automated assessment over PHI by default.

The decision needs to: (a) confirm the high-risk classification, (b) enumerate the obligations satisfaction mechanism + evidence trail, (c) align with sibling µservices' classifier ADRs to avoid duplication, (d) define the rollback + drift response posture, and (e) bound what gtm can claim about the classifier's behaviour.

## Decision

oyatie social adopts **EU AI Act high-risk classification with full Arts. 9-15 + 50 + 73 compliance** for both sub-capabilities:

1. **Classification: HIGH-RISK per Annex III §1(a).**
   - Both content-moderation classifier (`oya-social-content-moderation-*`) AND feed-ranking model (`oya-social-feed-timeline-*` algorithmic mode) are HIGH-RISK.
   - Heuristic ranking (P01 default per ADR-SOC-0001) is treated under Art. 50 transparency even though it's rule-based; ML ranking (P03+) is full Art. 9-15.
2. **Art. 9 risk-management:**
   - `microservices/social/dpia.md` §Step 6 carries the risk register; R-13 (algorithmic ranking discrimination) and R-14 (classifier false-positive) are the load-bearing rows.
   - Per-release risk re-assessment when classifier or ranking version changes.
3. **Art. 10 data + governance:**
   - Training-dataset SHA recorded per classifier / ranking release in `capabilities/T2-auto.yaml`.
   - Bias-audit per release: disparity ratio (4/5 rule) across protected groups (race, gender, age, locale) computed against held-out evaluation set; ≥ 0.8 ratio threshold.
   - Evidence sealed via audit-chain Ed25519 + foundry-runtime evidence pipeline.
4. **Art. 11 technical documentation:**
   - Model card per release stored at `microservices/social/evidence/model-cards/<classifier>-<version>.md` (Slice B) covering: intended use, training data, eval metrics, fairness audit, known limitations.
5. **Art. 13 + 50 transparency:**
   - Every classifier verdict carries `eu_ai_act_label: ai_generated_assessment` in the `ModerationVerdictEmitted` event payload (per `contracts/asyncapi/social-events.yaml`).
   - SDK helper `formatModerationVerdictLabel(verdict)` renders localised "AI-assessed" label per pack.
   - Ranking explanation API (`getRankingExplanation(post_id)`) exposes contributing signals per EU DSA Art. 27 + EU AI Act Art. 50.
6. **Art. 14 human oversight:**
   - High-confidence verdicts (> threshold) auto-hide reversibly; lower-confidence flag-for-review queue.
   - All appeals go through human reviewer (no auto-resolve); SLA 7 days per EU DSA Art. 20.
   - Per `runbooks/content-moderation-rollback.md`, mass false-positive events trigger Sev-1 + rollback path.
7. **Art. 15 accuracy + robustness + cybersecurity:**
   - Per-release reference-set eval: macro-F1 ≥ 0.92 across 8 verdict labels; false-positive rate < 2 % at production threshold.
   - Per-release adversarial robustness eval against synthetic prompt-injection + protected-group benchmark.
   - foundry-runtime classifier deployment signed; verdicts signed by foundry-runtime + sealed via audit-chain Ed25519.
8. **EU DSA Art. 17 Statement of Reasons:**
   - Every verdict emits a structured `statement_of_reasons` object (per `contracts/asyncapi/social-events.yaml`): grounds, facts_and_circumstances, automated_means (always true for classifier), redress (appeal URL).
9. **Art. 73 serious-incident reporting:**
   - Mass false-positive events with regulatory implications (EU AI Act non-compliance) reported to market surveillance authority within 15 days per Art. 73.
   - Coordination via council-privacy + ops-compliance.
10. **Pack-aware overrides:**
    - pack-us-healthcare: auto-moderation DISABLED on PHI accounts by default (HIPAA Safe Harbor); tenant may opt-in with BAA + per-account attestation.
    - pack-eu: full EU AI Act + EU DSA obligations operative.
    - pack-uk: UK Online Safety Act 2023 illegal-content duty; Ofcom notification per significance.
    - pack-kr: KR PIPA Art. 29-2 individual right to opt-out of automated decision.
    - Minor accounts (per `age-verification` BC): chronological-only feed default + classifier verdicts visible to parental account.
11. **Alignment with sibling µservices:**
    - This ADR pairs with `ADR-MSGR-0003` (messenger search/classifier bounds) and planned `ADR-MAIL-0004` (mail spam-classifier bounds). All three share the foundry-runtime evidence pipeline + reference-set eval + audit-chain seal pattern. Cross-citation maintained.

## Alternatives Considered

### A. Treat classifier as MINIMAL_RISK (claim auto-hide is reversible therefore not high-risk)

- Pros: drastically simpler compliance; no Art. 9-15 obligations.
- Cons: Annex III §1(a) explicitly covers content-moderation in recommender contexts; reversibility doesn't downgrade classification; council-privacy + ops-legal confirm misclassification risk = direct regulatory non-compliance.
- Rejected: misclassification.

### B. Treat classifier as LIMITED_RISK (transparency only; no Art. 9-15)

- Pros: lighter obligations; would match some online-platform interpretation.
- Cons: same Annex III §1(a) coverage; "limited" vs "high" matters and the EU AI Act distinguishes them at material level; misclassification risk = regulatory non-compliance.
- Rejected.

### C. Refuse to deploy classifier in EU pack until full Art. 9-15 + 50 + 73 pipeline is in place

- Pros: zero EU regulatory risk.
- Cons: leaves users + tenants unprotected from spam / abuse / CSAM-suspect content; EU DSA Art. 16 notice-and-action duty unmet; competitive disadvantage; not viable.
- Rejected; we adopt full compliance instead of avoidance.

### D. Outsource classifier inference to a third-party SaaS (e.g., Google Perspective API)

- Pros: existing high-risk pipeline; faster TTM.
- Cons: per-pack data-residency (Personal-tier data crossing pack boundaries forbidden by `policy/data-residency.md`); GDPR Art. 28 processor relationship complexities; EU AI Act obligations still apply to oyatie as the deployer; cost.
- Rejected; foundry-runtime in-pack is the strategic substrate.

### E. Use only rule-based classifier (no ML) to avoid EU AI Act high-risk

- Pros: rule-based systems are usually limited-risk; lower compliance overhead.
- Cons: rule-based systems are insufficient for nuanced abuse / hate-speech detection at scale; competitive disadvantage; the EU AI Act high-risk obligations may still apply when rule-based systems "significantly influence" user content per Art. 50; only marginal compliance saving.
- Rejected; the obligation set largely applies even to rule-based, and the practical detection benefit of ML is too significant.

### F. Per-tenant classifier customisation

- Pros: tenants tailor moderation to their community values; explainable.
- Cons: per-tenant ML configuration multiplies EU AI Act risk-management surface across N tenants; Art. 9 risk-management would need N separate risk registers; defer.
- Rejected for P01-P03; may revisit per-tenant policy customization (not per-tenant ML) at M04-onward.

## Consequences

### Positive

- Full EU AI Act + EU DSA + KR PIPA + UK Online Safety + AU Online Safety + HIPAA Safe Harbor compliance posture from day-1.
- Statement of Reasons + Art. 50 transparency label + appeal workflow + 7-day SLA satisfy EU DSA Art. 17 + 20 + 24.
- Audit-chain Ed25519 seal per verdict creates non-repudiable record for regulatory + tenant disputes.
- Bias-audit per release surfaces fairness regressions before deployment.
- Sibling µservices' (messenger, mail) classifier patterns align — single evidence pipeline; single rollback path.
- Sales narrative: "EU AI Act + EU DSA compliance shipped from day-1" is a competitive differentiator (per `competitor-parity-matrix.md` differentiator 8).

### Negative

- Per-release evidence pipeline complexity is non-trivial (model card + reference-set eval + bias audit + adversarial robustness eval); operational cost.
- Art. 73 serious-incident reporting requires ops-compliance + council-privacy + ops-legal coordination; not all team members are aware of 15-day window.
- pack-us-healthcare PHI default-OFF means HIPAA-Covered Entity tenants get less spam protection unless they opt-in with BAA.
- Heuristic ranking (P01 default per ADR-SOC-0001) still triggers EU AI Act Art. 50 transparency; minor UI/UX overhead.
- Per-tenant classifier customisation scheduled-for-distinct-tracked-work (Alternative F) may field gtm requests; default response: "wait for M04-onward ADR".

### Operational

- Cargo workspace: `oya-social-content-moderation-*` BC (per IP-011) integrates foundry-runtime client + audit-chain seal + Statement-of-Reasons emission.
- CI lane `oya-governance-eu-ai-act-conformance` registered in IP-015; verifies:
  - Statement-of-Reasons emission per verdict.
  - eu_ai_act_label populated.
  - reference-set eval passes for staged release.
  - Bias-audit disparity ratio ≥ 0.8.
- Runbook `runbooks/content-moderation-rollback.md` (Slice A) authored for drift / mass false-positive recovery.
- Per-tenant transparency log export (`oya_social_dsa_transparency_report_freshness_days` metric) for EU DSA Art. 24 quarterly publication.
- pack-us-healthcare overlay disables T2 auto-moderation by default; activation requires BAA + per-account attestation.
- Minor accounts (per `age-verification` BC + pack thresholds): chronological-only feed + verdict visibility to parental account.

### Regulatory

- **EU AI Act 2024/1689 Arts. 9-15 + 50 + 52 + 73**: fully satisfied per pipeline above.
- **EU DSA Regulation (EU) 2022/2065 Arts. 16, 17, 20, 24, 27, 28**: fully satisfied.
- **KR PIPA Art. 29-2**: opt-out + explanation API satisfied.
- **UK Online Safety Act 2023**: illegal-content duty + safety-by-design report; Ofcom notification per significance.
- **AU Online Safety Act 2021**: BOSE + eSafety Commissioner.
- **HIPAA 45 CFR §164.502(b) + §164.514**: minimum-necessary + safe-harbor compliance for pack-us-healthcare.
- **NIST AI RMF**: risk management framework alignment; bias-audit + reference-set eval map to NIST AI RMF Map + Measure + Manage functions.
- **ISO/IEC 23894 (AI risk management)**: voluntary alignment per Art. 52 codes-of-conduct.

## References

- EU AI Act 2024/1689 (full regulation; specifically Annex III §1(a), Arts. 9-15, 50, 52, 73).
- EU DSA Regulation (EU) 2022/2065 Arts. 16, 17, 20, 24, 27, 28.
- KR PIPA Arts. 28, 29, 29-2.
- HIPAA 45 CFR §164.502(b), §164.514.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021 BOSE.
- US Section 230 (publisher-provider distinction; pack-us context).
- NIST AI RMF.
- ISO/IEC 23894.
- ADR-0022 — Bominal autonomy-tier classification.
- ADR-0135 — Connect dissolution.
- ADR-0131 — Per-microservice flat layout.
- ADR-SOC-0001 — Feed-ranking algorithm (paired).
- ADR-MSGR-0003 — Messenger search backend selection (sibling classifier ADR pattern reference; though this ADR pairs more directly with a future ADR-MSGR-NNNN on messenger moderation).
- `microservices/social/PRD.md`.
- `microservices/social/capabilities/T2-auto.yaml`.
- `microservices/social/runbooks/content-moderation-rollback.md`.
- `microservices/social/runbooks/abuse-report-backlog-drain.md`.
- `microservices/social/threat-model.md` T-T-06.
- `microservices/social/dpia.md` R-13, R-14.
- `microservices/social/compliance.md` §EU AI Act + §EU DSA.
