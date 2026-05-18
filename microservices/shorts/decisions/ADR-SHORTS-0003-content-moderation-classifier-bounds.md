---
id: ADR-SHORTS-0003
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-architecture, council-privacy, axis-shorts, axis-foundry-runtime, ops-security, ops-legal
owner: axis-shorts + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0022
  - ADR-0135
  - ADR-0131
  - ADR-0133
  - ADR-SHORTS-0005
  - ADR-SHORTS-0006
related_artifacts:
  - microservices/shorts/PRD.md
  - microservices/shorts/compliance.md (§EU AI Act + §EU DSA + §EU AVMSD)
  - microservices/shorts/capabilities/T2-auto.yaml
  - microservices/shorts/runbooks/moderation-classifier-rollback.md
  - microservices/shorts/dpia.md (R-09, R-14, R-18)
purpose: Establish bounds + EU AI Act + EU DSA + EU AVMSD obligations for the content-moderation classifier in shorts; aligned with social ADR-SOC-0003 + messenger ADR-MSGR-0003.
---

# ADR-SHORTS-0003: Content-moderation classifier bounds — EU AI Act high-risk; Arts. 9-15 + Art. 50 obligations operative

## Status

Accepted — 2026-05-17.

## Context

shorts ships a content-moderation classifier (NSFW + violence + minor-protection + harassment + CSAM-suspect + self-harm-suspect + hate-speech-suspect categories) operating at scale. Per PRD FR-13 + FR-14, it is a hero-product BC.

Per EU AI Act 2024/1689 Annex III §1(a): **recommender systems including content-moderation systems that significantly influence user-facing content are HIGH-RISK**. This applies to:
- The moderation classifier itself (sub-capability 1 of `capabilities/T2-auto.yaml`).
- The ranking model (sub-capability 2 of `capabilities/T2-auto.yaml`; see ADR-SHORTS-0005).

HIGH-RISK classification triggers a substantial set of obligations:
- Art. 9: risk-management system.
- Art. 10: data + governance (training-dataset SHA + bias-audit per release).
- Art. 11: technical documentation (model card per release).
- Art. 13: transparency to deployers + users.
- Art. 14: human oversight.
- Art. 15: accuracy + robustness + cybersecurity.
- Art. 50: transparency to natural persons (UI label "AI-assessed" / "AI-generated").
- Art. 52: voluntary codes of conduct.
- Art. 73: 15-day notification of serious incidents to market-surveillance authority.

Per EU DSA Regulation 2022/2065:
- Art. 16: notice-and-action workflow.
- Art. 17: Statement of Reasons per moderation action.
- Art. 20: internal complaint-handling (appeal workflow) within 7d SLA.
- Art. 24: transparency report (per-tenant quarterly).
- Art. 28: minor-protection — chronological-only default + algorithmic-opt-out + DM-restricted (see ADR-SHORTS-0006).

Per EU AVMSD 2018/1808 Art. 28b(2): video-sharing-platform minor-protection; minor-protection auto-hide cannot be reversed without human review.

Per US COPPA + CA AB-2273 + UT SMRA: minor accounts have heightened-protection automated-content surfaces.

Per UK Online Safety Act 2023: Ofcom illegal-content duty + safety-by-design report (UK-located tenant).

This ADR mirrors sibling ADRs:
- social `ADR-SOC-0003` (content-moderation classifier bounds in social).
- messenger `ADR-MSGR-0003` (content-moderation classifier bounds in messenger).
- mail spam-classifier pattern.

## Decision

oyatie shorts adopts the **EU AI Act HIGH-RISK obligations operative posture** for the content-moderation classifier from M03 launch:

1. **Classifier verdict labels** (per `capabilities/T2-auto.yaml`):
   - `ok`
   - `sensitive` (user warning shown; no auto-hide)
   - `nsfw` (auto-hide pending appeal; requires human review for restoration)
   - `violence` (auto-hide pending appeal)
   - `minor_protection_concern` (auto-hide AND mandatory human review per EU AVMSD Art. 28b(2); irreversible without human reviewer)
   - `csam_suspect` (auto-hide AND mandatory NCMEC report per US 18 USC §2258A / mandatory IWF report per UK)
   - `self_harm_suspect` (auto-hide + creator wellness intervention)
   - `harassment` (auto-hide pending appeal)

2. **Confidence threshold for auto-action**:
   - ≥ 0.95: auto-hide
   - 0.7-0.95: flag for human review
   - < 0.7: no action
   - minor_protection_concern + csam_suspect: ANY confidence triggers human review (regulatory floor; EU AVMSD)

3. **EU AI Act Art. 50 transparency**: every verdict carries `eu_ai_act_label` field (`ai_generated_assessment` / `human_assessment` / `hybrid`); client SDK renders "AI-assessed" UI badge.

4. **EU DSA Art. 17 Statement of Reasons**: every verdict carries `statement_of_reasons` (grounds, facts_and_circumstances, automated_means, redress URL).

5. **EU DSA Art. 20 appeal SLA**: 7 days; appeals emit `AppealOpened` event; reviewer SLA tracked per `slos/...` (appeal-resolution latency to be added).

6. **Per-pack overlays**:
   - **pack-eu**: all classifier verdicts HIGH-RISK; full obligations.
   - **pack-uk**: UK Online Safety Act 2023 priority detection per Ofcom; safety-by-design report.
   - **pack-us-healthcare**: auto-moderation OFF for PHI accounts per HIPAA Safe Harbor §164.502(b).
   - **pack-kr**: KR PIPA Art. 29-2 individual right to opt-out of automated decisions; UI surface enables opt-out per the obligation.
   - **pack-kr**: KR 청소년 보호법 minor-protection — auto-action mandatory; opt-out not permitted for minor accounts.
   - **pack-us**: COPPA + CA AB-2273 + UT SMRA — minor-protection auto-action mandatory.
   - **pack-au**: AU Online Safety Act 2021 + BOSE 2022 — minor-protection + illegal-content detection.

7. **EU AI Act Art. 9 risk management**: per-release risk register; mitigation tracker; covered by `microservices/shorts/dpia.md` Step 6 + R-09 + R-14 + R-18.

8. **Art. 10 data governance**: training-dataset SHA + bias-audit per release; per-release golden-set eval (macro-F1 ≥ 0.95; minor-protection recall ≥ 0.99 regulatory floor; bias-audit disparity ratio per 4/5 rule across protected groups: race, gender, age, locale).

9. **Art. 11 technical documentation**: per-classifier model card stored in foundry-runtime evidence pipeline; version-bumped per release.

10. **Art. 14 human oversight**: manual reviewer in loop for all verdicts above confidence threshold; minor-protection always requires human review.

11. **Art. 15 accuracy + robustness**: per-release golden-set eval + adversarial-robustness eval; monthly drift-monitor.

12. **Art. 73 serious-incident notification**: 15-day clock; trigger via `runbooks/moderation-classifier-rollback.md`; council-privacy + ops-legal coordination.

13. **Backfill on rollback**: per `backfill-replay.md` BF-06 — reverse false-positive auto-hides; per-affected-creator notification.

## Alternatives Considered

### A. Classify everything as low-risk (i.e., claim AI Act doesn't apply)

- Pros: lower operational overhead; no per-release model card; no transparency labels.
- Cons: Annex III §1(a) text is clear: recommender + moderation systems significantly influencing user-facing content are HIGH-RISK. Non-compliance is regulator-actionable + tenant-of-tenant reputational risk.
- Rejected: regulatorily incorrect.

### B. Classify only the ranking model as high-risk; treat moderation classifier as limited-risk

- Pros: less operational overhead on moderation; T1 capability surface only.
- Cons: text of Annex III §1(a) explicitly names moderation; carve-out untenable; tenant-of-tenant audit risk.
- Rejected: regulatorily incorrect; same Annex III paragraph.

### C. Use confidence threshold of 0.7 for auto-action (instead of 0.95)

- Pros: more conservative auto-hides; false-positive rate higher tolerated.
- Cons: massive false-positive surface; appeal queue overwhelm; EU AI Act Art. 15 accuracy fails; bias-audit fails.
- Rejected: threshold-too-low harms creators + fails accuracy bound.

### D. Defer EU AI Act compliance to a separate "EU pack activation" date

- Pros: simpler M03 launch.
- Cons: per parallel ADR-0135 + ADR-0131 + ADR-0133, oyatie's design posture is hyperscaler-grade compliance from day 1; pack-eu may activate any time; classifier carries obligations universally.
- Rejected: design-time conformance > activation-time scramble.

### E. Single global moderation model (no per-pack overlay)

- Pros: simpler operations; one model card.
- Cons: KR PIPA Art. 29-2 opt-out + CA AB-2273 minor-protection + EU AVMSD differ; per-pack overlay required.
- Rejected: regulatory diversity demands per-pack overlay.

## Consequences

### Positive

- Hyperscaler-grade EU AI Act + EU DSA + EU AVMSD compliance from M03 launch.
- Tenant-of-tenant trust (especially EU enterprise tenants) elevated by transparent classifier posture.
- Audit-chain Ed25519 seal per verdict + per appeal + per backfill creates non-repudiable record.
- Minor-protection regulatory floor enforced uniformly across all packs.
- Pre-built rollback path via `runbooks/moderation-classifier-rollback.md` + `backfill-replay.md` BF-06 reduces incident MTTR.

### Negative

- Per-release model-card production overhead + golden-set eval + bias-audit + adversarial-robustness eval = significant axis-foundry-runtime + axis-shorts labor.
- Higher false-positive rate possible during early classifier life; appeal queue scales accordingly.
- EU AI Act Art. 73 15-day clock requires ops-legal + council-privacy on-call rotation.

### Operational

- Classifier version chain tracked per release; model card per version.
- Per-release golden-set eval: macro-F1 ≥ 0.95; minor-protection recall ≥ 0.99; bias-audit disparity ratio ≥ 0.8 (4/5 rule).
- Monthly drift-monitor: real-world classifier drift alert at > 5x baseline (per `iac/helm/shorts/templates/prometheusrule.yaml`).
- Backfill on rollback path: BF-06.
- Appeal SLA dashboard tracked per pack; 7d target.

### Regulatory

- **EU AI Act Arts. 9, 10, 11, 13, 14, 15, 50, 52, 73**: all satisfied.
- **EU DSA Arts. 16, 17, 20, 24**: notice-and-action + Statement of Reasons + appeal + transparency report.
- **EU AVMSD Art. 28b(2)**: minor-protection auto-hide irreversible without human reviewer.
- **KR PIPA Art. 29-2**: individual right to opt-out of automated decisions via UI.
- **KR 청소년 보호법 + COPPA + CA AB-2273 + UT SMRA + AU OSA**: minor-protection auto-action mandatory.
- **US 18 USC §2258A**: NCMEC reporting on CSAM-suspect verdicts.
- **UK Online Safety Act 2023**: Ofcom illegal-content duty + safety-by-design report.

## References

- ADR-0022 autonomy tiers (T0/T1/T2; inherited from Bominal).
- Parallel ADR-0135 dual-context.
- ADR-0131 per-µservice flat layout.
- ADR-0133 hyperscaler best-practice.
- ADR-SOC-0003 (sibling moderation ADR; paired pattern).
- ADR-MSGR-0003 (sibling moderation ADR; paired pattern).
- ADR-SHORTS-0005 (ranking model; sibling Annex III §1(a) high-risk).
- ADR-SHORTS-0006 (minor protection + age-gate; paired).
- EU AI Act 2024/1689 Annex III §1(a); Arts. 9, 10, 11, 13, 14, 15, 50, 52, 73.
- EU DSA Regulation 2022/2065 Arts. 16, 17, 20, 24, 28.
- EU AVMSD 2018/1808 Art. 28b.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021 + BOSE 2022.
- KR PIPA Art. 29-2; KR 청소년 보호법.
- COPPA 15 USC §6501; CA AB-2273; UT Social Media Regulation Act.
- US 18 USC §2258A (NCMEC reporting).
- NIST AI RMF.
- HELM benchmark.
- `microservices/shorts/capabilities/T2-auto.yaml`.
- `microservices/shorts/runbooks/moderation-classifier-rollback.md`.
- `microservices/shorts/slos/moderation-classifier-latency.openslo.yaml`.
- `microservices/shorts/dpia.md` R-09, R-14, R-18.
