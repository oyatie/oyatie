---
doc_class: Runbook
title: On-call rotation + handoff
microservice: observability
status: Accepted
owner_team: ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/observability/incident-response.md §"On-Call Rotation"
  - microservices/observability/failure-modes.md (FM-09 OnCall outage)
doc_status: published
---

# Runbook: On-call rotation + handoff

## Purpose

The administrative + operational procedure for observability on-call rotation, handoff between shifts, and recovery from Grafana OnCall outage (FM-09).

## Rotation structure

Per `incident-response.md` §"On-Call Rotation":

| Tier | Pool | Rotation | Pay |
|---|---|---|---|
| ops-sre-reliability primary | 6 engineers | Weekly; follow-the-sun (KR / EU / US shifts) | On-call pay per company policy |
| ops-sre-reliability secondary | Same pool, offset 1 week | Weekly | Same |
| axis-observability SME | 3 engineers | Weekly; KR + EU primary; US business-hours fallback | Same |
| ops-security on-call | 4 engineers | Weekly; 24/7 for Sev-1 confidentiality | Same |
| council-privacy chair | Named role; permanent | Always-on-call for breach-suspect | – |
| Executive Sponsor | Named role; permanent | Sev-1 only | – |

## Weekly handoff procedure (every Monday 09:00 local time, per shift)

| Step | Action |
|---|---|
| 1 | Outgoing on-call publishes handoff doc in `#ops-handoff` Slack: open incidents (with severity, assignee, status); active deploys; runbook gaps discovered; known-flakes. |
| 2 | Incoming on-call acknowledges in same channel. |
| 3 | Grafana OnCall schedule updates automatically (rotation declared in OnCall config-as-code at `microservices/observability/iac/terraform/oncall.tf`). |
| 4 | If active Sev-1 / Sev-2: shadow period of 24h with outgoing still in the loop. |
| 5 | Outgoing posts to `#ops-handoff`: "off-rotation; back next month" + clear-state attestation. |

## On-call response standards

| Severity | Page-to-ack | Page-to-mitigation-start |
|---|---|---|
| Sev-1 | ≤ 5 min (24/7) | ≤ 15 min |
| Sev-2 | ≤ 15 min (24/7) | ≤ 30 min |
| Sev-3 | ≤ 1 h (business hours; eventually 24/7 if escalates) | ≤ 4 h |
| Sev-4 | next business day | N/A |

If on-call is unable to ack within target (illness, internet outage, etc.):
- Auto-escalation per `incident-response.md` §"Escalation Path".
- Mark "unavailable" in OnCall config immediately if known in advance.

## Grafana OnCall outage (FM-09)

### Pre-checks

1. Confirm OnCall is down: dashboard health probe `https://oncall-<pack>.oyatie.dev/api/v1/health` returns 500 / unreachable.
2. Confirm Mimir is still emitting verdicts (two-channel corroboration — Mimir is the primary signal; OnCall is paging only).
3. Confirm Alertmanager is firing webhooks to OnCall (Alertmanager state log shows successful POST OR failure).

### Recovery

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; engage ops-sre-reliability + axis-observability. | ≤ 5 min |
| 2 | Activate fallback paging via Alertmanager → Slack (`#ops-emergency`) bypassing OnCall. | ≤ 5 min |
| 3 | All current on-call notified via direct Slack mention. | ≤ 5 min |
| 4 | If OnCall outage > 30 min: failover to backup paging provider (PagerDuty trial license activated). | ≤ 15 min |
| 5 | Restore OnCall: Postgres replica promoted; OnCall pods restarted; webhook secret rotated if needed. | ≤ 1 h |
| 6 | Validate: synthetic alert fires; OnCall page received by on-call. | ≤ 10 min |

### Verification

After restoration:
- OnCall API healthy.
- Alertmanager webhook delivery success rate > 99.9%.
- HMAC signing key rotated if outage involved key compromise (FM-09 ⇒ T-S-03 in threat-model).

## On-call training + onboarding

New on-call engineers:
1. Shadow 2 weeks (primary + secondary in parallel).
2. Reverse-shadow 2 weeks (lead with mentor backup).
3. Solo with reduced rotation cadence (1 week off, 2 weeks on) for first 2 months.
4. Annual recertification: tabletop exercise + runbook walkthrough.

## On-call ergonomics

| Standard | Target |
|---|---|
| Pages per shift (week) | ≤ 5 outside business hours (signal-to-noise SLO) |
| Sleep-disrupting pages | ≤ 1 per week per engineer |
| Time off after Sev-1 incident | ≥ 1 day for the IC |
| Burnout watch | Manager 1:1 every 2 weeks during on-call month |

## References

- `microservices/observability/incident-response.md` §"On-Call Rotation".
- `microservices/observability/failure-modes.md` FM-09.
- Google SRE Workbook ch. 11 (Managing operational load).
- Grafana OnCall docs — `grafana.com/docs/oncall/latest/`.
