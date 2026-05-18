---
doc_class: Runbook
title: Recruiter-stub classifier rollback + ranker fallback (EU AI Act high-risk)
microservice: network
severity: "Sev-1 (regulatory-class; EU AI Act Art. 73 + EEOC + NYC LL144)"
status: Accepted
owner_team: axis-network + axis-foundry-runtime + ops-compliance + council-privacy
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-15, FM-17)
  - microservices/network/capabilities/T2-auto.yaml
  - microservices/network/decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md
  - microservices/network/dashboards/recommender-fairness-and-bias.json
doc_status: published
---

# Runbook: Recruiter-stub classifier rollback + ranker fallback

## Trigger

- `network_recruiter_bias_audit_disparity_ratio{group="<protected_group>"}` < 0.8 sustained ≥ 1h (FM-15).
- `network_recommender_output_drift_total` > 5× baseline for ≥ 10 min (FM-17).
- Notified body or regulator flag (NYC DCWP, EU AI Act notified body, EEOC, CA AG, CO AG, ICO).
- Tenant-admin escalation: recruiter-stub producing visibly biased results.

## Severity

**FM-15 is Sev-1 by regulatory class** — EU AI Act Annex III §4 (employment) high-risk system + Art. 73 serious-incident reporting clock + NYC LL144 + EEOC + CA AB-331 + CO SB 24-205 all apply. Auto-rollback is mandatory; manual sign-off needed for redeployment.

FM-17 is Sev-2 default; escalates to Sev-1 if mass discriminatory-impact pattern detected.

## Immediate Mitigation — FM-15 Recruiter-stub bias-audit failure

| Step | Action | Time |
|---|---|---|
| 1 | Auto-rollback recruiter-stub to last-known-good model version per foundry-runtime release-pointer revert | ≤ 5 min |
| 2 | Pause recruiter-stub for all tenants in affected packs (default: NYC + CA + CO + EU pending re-audit) | ≤ 5 min |
| 3 | Cedar entitlement revoke: `recruiter-stub-activated == false` on affected packs | ≤ 5 min |
| 4 | Engage ComplianceLead + PrivacyLead | ≤ 5 min |
| 5 | EU AI Act Art. 73 serious-incident notification clock starts (≤ 15 days to market surveillance authority) | T+0 |
| 6 | Audit-chain seal of the failure event + rollback action | ≤ 1 min |
| 7 | Notify affected tenants per pack regulatory clocks | per IR |
| 8 | NYC DCWP notification per LL144 §20-872 (when NYC tenant affected) | within reporting cycle |
| 9 | EEOC: retain UGESP record-keeping for 2y; respond on EEOC charge if filed | ongoing |
| 10 | CA AG / CO AG notification per AB-331 / SB 24-205 (when CA / CO tenant affected) | per CA AG / CO AG guidance |

## Immediate Mitigation — FM-17 Recommender drift / mass-false-positive

| Step | Action | Time |
|---|---|---|
| 1 | Roll back ranker version per foundry-runtime release-pointer revert | ≤ 5 min |
| 2 | Pause T2 auto-ranking per affected pack via Cedar entitlement revoke; restore chronological default | ≤ 5 min |
| 3 | Restore PYMK / ranker output to last-known-good baseline; surface tenant UI banner | ≤ 10 min |
| 4 | Audit-chain seal of the rollback event | ≤ 1 min |
| 5 | If discriminatory-impact pattern detected: escalate to Sev-1 + FM-15 mitigation | – |

## Re-deployment Gate

Recruiter-stub re-deployment after rollback requires ALL of the following before Cedar entitlement re-activation:

- Re-run bias audit on the proposed model version against the protected-group test set (4/5-rule statistical disparity ratio ≥ 0.8 for every protected group).
- Per-release model card sealed (EU AI Act Art. 11 technical documentation).
- FRIA (Fundamental Rights Impact Assessment per Art. 27) updated when material change.
- Council-privacy sign-off.
- ops-compliance sign-off.
- For NYC tenants: independent annual bias audit refreshed (LL144 §20-870).
- For CA / CO tenants: AB-331 / SB 24-205 risk-management policy updated.
- Audit-chain seal of all sign-offs.

## Ranker Fallback (when T2 ranker is rolled back)

When T2 ranking is paused:

| Surface | Fallback |
|---|---|
| Feed-render | Heuristic chronological-first per ADR-NET P01 strategy |
| People-you-may-know panel | Disabled; surface "PYMK suggestions paused" UI banner |
| Recruiter-search | Disabled per Cedar entitlement; recruiters receive informational error |
| Jobs-ranking | Fallback to recency + skill-match heuristic (no ML) |
| Endorsement-aggregation | Display raw endorsement counts only; no aggregation-weighted ranking |

## Recovery Verification

- Bias-audit lane green: 4/5-rule disparity ratio ≥ 0.8 for every protected group over 30d window.
- `network_recruiter_bias_audit_disparity_ratio` returns to ≥ 0.8 sustained ≥ 7d.
- `network_recommender_output_drift_total` rate at 0 for ≥ 24h.
- Tenant-admin acknowledges restoration; tenant-side metrics show normal use.
- All regulatory notifications dispatched + acknowledged where required.

## Postmortem Triggers

- FM-15: postmortem mandatory within 5 business days; council-privacy + ops-compliance + axis-network sign-off; lessons-learned shared with foundry-runtime team.
- FM-17: postmortem within 5 business days for Sev-2; same as FM-15 if escalated to Sev-1.
- Recurring drift on same model family: review training-data pipeline + golden-set in foundry-runtime; consider model retirement.

## Drill Pattern

Quarterly recruiter-classifier-rollback drill (per `incident-response.md` Drills table):

1. Synthetic failure injection: lower disparity ratio for a protected group via shadow-deployed test model.
2. Verify automatic rollback fires within 15 min.
3. Verify regulatory-notification rehearsal triggers (no actual notification dispatched).
4. Verify ComplianceLead engagement protocol.

## References

- `microservices/network/failure-modes.md` FM-15, FM-17.
- `microservices/network/decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md`.
- `microservices/network/capabilities/T2-auto.yaml`.
- `microservices/network/dashboards/recommender-fairness-and-bias.json`.
- `microservices/network/incident-response.md` §"Recruiter-Stub Bias-Audit Failure = Sev-1".
- EU AI Act 2024/1689 Annex III §4 + Arts. 9-15, 27, 50, 72, 73.
- US Title VII; ADA; ADEA; EEOC UGESP 29 CFR §1607.
- NYC Local Law 144-2021 §§20-870, 20-871, 20-872.
- CA AB-331 §22756; CO SB 24-205 §6-1-1701.
- UK Equality Act 2010 + ICO ADM guidance.
