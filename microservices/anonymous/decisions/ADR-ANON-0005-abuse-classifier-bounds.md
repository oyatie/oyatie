---
id: ADR-ANON-0005
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, axis-foundry-runtime, council-privacy, council-architecture, general-counsel
owner: axis-anonymous + axis-foundry-runtime + general-counsel
supersedes: []
superseded_by: []
related:
  - ADR-COMM-0001
  - ADR-ANON-0001
related_artifacts:
  - microservices/anonymous/PRD.md (FR-19, FR-20, FR-27)
  - microservices/anonymous/capabilities/T1-assist.yaml
  - microservices/anonymous/capabilities/T2-auto.yaml
  - microservices/anonymous/runbooks/abuse-classifier-rollback.md
purpose: |
  Classify the abuse-classifier under EU AI Act risk categories; document
  whether GDPR Art. 22 is triggered; fix EU DSA Art. 14/16/17 transparency
  obligations; set NCMEC CyberTipline reporting flow.
---

# ADR-ANON-0005: Abuse-classifier bounds — EU AI Act limited-risk (not Annex III); Art. 50 transparency applies; GDPR Art. 22 not triggered (anonymous users); EU DSA Arts. 14/16/17 fully honoured

## Status

Accepted — 2026-05-17.

## Context

The anonymous µservice runs an AI abuse-classifier (foundry-runtime T1) that takes post bodies and returns a classification (harassment / hate-speech / spam / threat / doxxing / impersonation / CSAM-suspect). The classifier informs T1-assist (human-review queue prioritization) and T2-auto (high-confidence auto-actions: hide, quarantine, revoke-affinity).

The regulatory questions:

1. **Is this classifier "high-risk" under EU AI Act Annex III?** Annex III lists employment/credit/migration/critical-infrastructure as high-risk categories. We need to determine whether content moderation on an anonymous platform falls into any of these.
2. **Is GDPR Art. 22 triggered?** Art. 22 governs automated decisions with "significant effect" on identifiable persons. We need to determine whether anonymous users qualify as "identifiable" in the relevant sense.
3. **What transparency obligations apply under EU AI Act Art. 50?**
4. **What transparency / appeal / statement-of-reasons obligations apply under EU DSA Arts. 14, 16, 17?**
5. **How does CSAM-suspect feed into NCMEC CyberTipline reporting under 18 USC §2258A?**

## Decision

### Classification: EU AI Act limited-risk (NOT Annex III high-risk)

The abuse-classifier is **limited-risk** because:

- **Annex III §1 (biometric identification)**: not applicable.
- **Annex III §2 (critical-infrastructure management)**: not applicable.
- **Annex III §3 (education/training)**: not applicable.
- **Annex III §4 (employment)**: not applicable — the platform's anonymous users are not employees of oyatie, and a moderation verdict has no employment consequence.
- **Annex III §5 (access to essential services)**: not applicable — a community-platform post is not an essential service.
- **Annex III §6 (law enforcement)**: not applicable — moderation is platform-policy enforcement, not state law enforcement.
- **Annex III §7 (migration)**: not applicable.
- **Annex III §8 (administration of justice)**: not applicable.

**Therefore, EU AI Act Art. 50 transparency obligations apply** (the classifier is "limited-risk AI" requiring disclosure that the user is interacting with an AI system + AI-generated content labelling), but **Annex III high-risk obligations (Arts. 9-15) do not apply**.

### GDPR Art. 22 NOT triggered

Art. 22 governs "a decision based solely on automated processing... which produces legal effects concerning him or her or similarly significantly affects him or her." The classifier:

- operates on anonymous users (the platform structurally cannot identify the user per I1);
- the user is therefore not "identifiable" in the Art. 22 sense at the moment of classification;
- the user CAN appeal (EU DSA Art. 14); a human moderator reviews appeals;
- the user CAN obtain a statement-of-reasons (EU DSA Art. 17).

**Therefore Art. 22 is not the load-bearing regulatory anchor**; EU DSA Arts. 14/16/17 are. Note: this analysis is conservative; if a future ADR redefines anonymity boundaries, Art. 22 may need re-analysis.

### EU AI Act Art. 50 transparency obligations

Every classifier verdict surfaces with an `ai_assessed_label` field carrying:

- `"T0 — not AI-generated; rule-based heuristics"` for T0-suggest tier
- `"T1 — AI-assessed; human review available; classifier_version <version>"` for T1-assist
- `"T2 — auto-action; AI-assessed; appeal available; classifier_version <version>"` for T2-auto

LEAN lane `oya-check-ai-assessed-label-present` enforces presence at CI time.

### EU DSA Arts. 14, 16, 17 obligations

- **Art. 14 (internal complaint mechanism)**: every moderation verdict carries an `appeal_link`. Appeals are reviewed by a human moderator from a different organisational unit than the original verdict-issuer (per ADR-COMM-0001 inherited).
- **Art. 16 (notification of action)**: affected users receive notification when an auto-action (T2) is taken.
- **Art. 17 (statement-of-reasons)**: every moderation verdict includes a `statement_of_reasons` field in the DSA Annex I format (action taken, reason, classifier_version if AI, appeal mechanism URL).
- **Art. 27/28 (transparency report)**: per-quarter aggregate counts; per-pack breakdown.

### NCMEC CyberTipline (18 USC §2258A)

CSAM-suspect classifier verdicts at confidence ≥ 0.95 trigger an immediate human review; a confirmed CSAM-suspect post produces:

1. **Within 1h**: post quarantined (T2 auto-action; not deleted — preservation required for chain-of-custody).
2. **Within 48h**: NCMEC CyberTipline report filed per 18 USC §2258A.
3. **Chain-of-custody hash** initialised at quarantine; chain-of-custody seal recorded at NCMEC filing.
4. **No notification to user** (statutory non-notification per CSAM laws).

Path E in `runbooks/legal-process-court-order-receipt.md` covers NCMEC preservation requests.

## Alternatives Considered

### A. Classify as Annex III high-risk

- **Pros**: Maximal regulatory rigour; auditors comfortable.
- **Cons**: Triggers Arts. 9-15 obligations (post-market monitoring, risk-management, conformity assessment) that are disproportionate to the actual risk; Art. 9 risk-management quality posture isn't materially better than what we already do; increased compliance cost.
- **Rejected because**: Conservative-but-not-overcautious is the right posture; Annex III applies to specific high-stakes categories that content moderation on an anonymous platform doesn't fit.

### B. Classify as minimal-risk (skip Art. 50 transparency)

- **Pros**: Less labeling overhead.
- **Cons**: AI Act Art. 50 transparency obligation is statutory for ANY AI-content-generation or AI-decision-disclosure to users; skipping is non-compliant.
- **Rejected because**: Statutorily required.

### C. Argue GDPR Art. 22 IS triggered (conservative posture)

- **Pros**: Maximally user-protective.
- **Cons**: The Art. 22 protections (right not to be subject to automated decisions) effectively REQUIRE human-in-loop on EVERY classifier verdict, which is impractical at scale. Art. 22(2) permits automated decisions under contractual necessity + suitable safeguards (which we provide via EU DSA Art. 14), so the conservative claim doesn't actually add user protection.
- **Rejected because**: EU DSA Art. 14 already provides equivalent or stronger user protections; double-claiming would be incoherent.

### D. NCMEC reporting only on user-reported CSAM (skip classifier-triggered)

- **Pros**: Lower false-positive risk for NCMEC.
- **Cons**: 18 USC §2258A requires "actual knowledge" reporting; a classifier verdict at confidence ≥ 0.95 + human review is statutorily "actual knowledge."
- **Rejected because**: Statutory floor.

### E. Auto-delete on CSAM-suspect (instead of quarantine)

- **Pros**: Faster removal from the platform.
- **Cons**: Destroys evidence; violates 18 USC §2258A preservation obligation.
- **Rejected because**: Preservation is statutorily required.

## Consequences

### Positive

- **Regulatory clarity.** EU AI Act Art. 50 applied; Annex III not over-applied; GDPR Art. 22 not over-applied.
- **User protections honoured via EU DSA Arts. 14/16/17** instead of GDPR Art. 22 (functionally equivalent + cleaner regulatory posture for anonymous platform).
- **NCMEC reporting path documented.** 18 USC §2258A compliance via Path E.
- **CI enforcement of transparency labels.** LEAN lane refuses any classifier verdict without `ai_assessed_label`.

### Negative

- **EU AI Act risk-classification analysis must be re-done if the classifier expands scope.** Mitigated: documented in this ADR; any expansion triggers a superseding ADR.
- **EU DSA Art. 14 appeal flow imposes operational cost.** Mitigated: appeals are bounded by appeal-volume × moderator-review-time; resourced in capacity-model.md.

### Operational

- T1-assist + T2-auto capability YAMLs reflect this classification.
- `runbooks/abuse-classifier-rollback.md` handles regressions.
- Quarterly transparency-report aggregator reflects per-pack volume + reversal rate.
- Classifier-version history tracked in audit-chain.

### Regulatory

- EU AI Act Art. 50 transparency: labelled at every verdict.
- EU DSA Art. 14: appeal-link present.
- EU DSA Art. 17: statement-of-reasons present.
- EU DSA Art. 27: quarterly transparency report.
- GDPR Art. 22: documented as not-triggered (conservative analysis above).
- 18 USC §2258A: NCMEC CyberTipline path E.

### Invariant Preservation

Maintains I1, I7. Adds regulatory clarity for the moderation surface.

## References

- EU AI Act Reg. 2024/1689 Arts. 6, 50; Annex III
- EU DSA Reg. 2022/2065 Arts. 14, 16, 17, 27, 28
- GDPR Reg. 2016/679 Art. 22
- 18 USC §2258A (NCMEC CyberTipline)
- ADR-COMM-0001 (moderation chain-of-responsibility — inherited; appeal hop)
- ADR-ANON-0001 (anonymity foundation)
