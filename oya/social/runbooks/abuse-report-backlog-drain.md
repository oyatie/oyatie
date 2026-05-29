---
doc_class: Runbook
title: Abuse-report + appeal backlog drain
microservice: social
severity: "Sev-3 (degraded) / Sev-2 (persistent > 7d backlog)"
status: Accepted
owner_team: axis-social + ops-security + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-11)
  - microservices/social/capabilities/T2-auto.yaml
  - microservices/social/compliance.md (EU DSA Arts. 16, 17, 20)
doc_status: published
---

# Runbook: Abuse-report + appeal backlog drain (FM-11 generalised)

## Trigger

Any of:
- `social_abuse_report_queue_depth` > 50k or persists > 1k for ≥ 24h.
- `social_appeal_pending_total` exceeds EU DSA Art. 20 SLA window (≤ 7 days).
- Tenant-admin escalation: backlog visible in moderator dashboard.
- Per-tenant SLA bound by tenant DPA breached.

## Severity

Sev-3 default; escalate to Sev-2 if backlog > 7 days sustained (EU DSA Art. 20 risk).

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect queue breakdown: `social_abuse_report_queue_depth` by `tenant_id`, `report_kind` | ≤ 3 min |
| 2 | Identify hot report category (spam, abuse, csam-suspect, impersonation, hate-speech) | ≤ 3 min |
| 3 | If sybil-coordinated false-flag campaign: engage foundry-guardrails sybil detector; quarantine reports from offending principals | ≤ 10 min |
| 4 | Scale moderation-worker pods (HPA or manual) | ≤ 5 min |
| 5 | Prioritise child-safety + CSAM-suspect reports (regulatory time-critical) | ≤ 5 min |
| 6 | Engage tenant moderator pool: notify Slack channel; raise queue visibility | ≤ 10 min |
| 7 | If classifier-induced (auto-report from T2 classifier): consult `content-moderation-rollback.md` | ≤ 5 min |

## Sustained Backlog Mitigation (> 24h)

| Step | Action |
|---|---|
| 1 | Escalate to council-privacy + ops-security; review for regulatory implications |
| 2 | Pause low-risk T2 auto-moderation to reduce queue inflow; rely on user reports |
| 3 | Onboard surge moderator pool (per-tenant; from tenant's own staff or oyatie partner program) |
| 4 | Communicate transparently to affected reporters + reported parties; offer interim status |
| 5 | If EU DSA Art. 20 7-day deadline breached: file regulator notification |

## Diagnosis

| Hypothesis | Signal | Action |
|---|---|---|
| Spike from viral event (controversy, breaking news) | report-rate spike correlated with trending topic | accept; surge response; tenant comms |
| Sybil-coordinated false-flag campaign | foundry-guardrails sybil signal + clustering on reporter IPs / accounts | engage ops-security; per-IP block; per-tenant attack-mode toggle |
| Classifier drift (mass auto-flags) | auto-report rate >> user-report rate | rollback classifier; see `content-moderation-rollback.md` |
| Tenant moderator under-staffing | per-tenant breakdown | tenant comms; raise moderator quota |
| New regulatory pack activation (e.g., pack-eu activation triggers DSA queue) | report-kind shift | onboarding surge |

## Recovery Verification

- `social_abuse_report_queue_depth` < 5k for ≥ 24h.
- `social_appeal_pending_total` within SLA (P95 ≤ 5d).
- All CSAM-suspect reports resolved in priority order.
- No active alerts on moderation backlog.

## Postmortem Triggers

- If recurring (≥ 2 in 90d): review moderation capacity sizing.
- If sybil-coordinated attack: ops-security playbook + per-tenant attack-mode toggle.
- If regulatory deadline breached: regulatory notification + remediation plan to authority.

## Regulatory Notes

| Framework | Obligation |
|---|---|
| EU DSA Art. 16 | Notice-and-action mechanism (must accept reports) |
| EU DSA Art. 17 | Statement of Reasons (must justify each verdict) |
| EU DSA Art. 20 | Internal complaint-handling (appeals within 6 months; oyatie SLA: 7 days) |
| EU DSA Art. 24 | Transparency report (cumulative metrics; quarterly publication) |
| US COPPA + KR 청소년 보호법 | Child-safety reports prioritised; mandatory escalation |
| UK Online Safety Act 2023 | Significant illegal-content failure → Ofcom notification |
| AU Online Safety Act 2021 | BOSE compliance; eSafety Commissioner notification per significance |

## References

- `microservices/social/failure-modes.md` FM-11.
- `microservices/social/compliance.md` §EU DSA.
- `microservices/social/capabilities/T2-auto.yaml`.
- EU DSA 2065/2022 Arts. 16, 17, 20, 24.
