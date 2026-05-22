---
doc_class: IncidentResponse
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0248
  - ADR-0263
companion_docs:
  - microservices/feature-flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/runbooks/flag-mutation-cascade.md
  - microservices/feature-flags/failure-modes.md
  - microservices/feature-flags/multi-region.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Incident Response — Feature Flags

## Severity classification

| SEV | Criteria | Response time | Escalation |
|---|---|---|---|
| **SEV-1** | Flag evaluation completely unavailable; kill-switch not responding; emergency-services flags blocked | Immediate (pager) | PD: `feature-flags-sre` → `axis-platform-oncall` → CTO |
| **SEV-2** | Flag evaluation degraded (>1ms p99 or >0.01% error rate); kill-switch delayed (>1s propagation); pack override misapplied | 15 minutes | PD: `feature-flags-sre` |
| **SEV-3** | Replication lag >5s; experiment assignment anomaly; stale targeting rule; SLO budget burn >5× | 1 hour | Slack: `#feature-flags-oncall` |
| **SEV-4** | Documentation gap; CI lane warning; non-production issue | Next business day | Jira ticket |

## Incident runbooks index

| Incident | Runbook | SEV |
|---|---|---|
| Kill-switch not propagating | `runbooks/killswitch-engaged.md` | SEV-1 |
| Flag mutation causing cascade failures across µservices | `runbooks/flag-mutation-cascade.md` | SEV-1/2 |
| Experiment statistical significance violation | `runbooks/experiment-stat-sig-violation.md` | SEV-2/3 |
| Experiment rollback required | `runbooks/experiment-rollback.md` | SEV-2 |
| Audit log replay required | `runbooks/audit-replay.md` | SEV-2/3 |
| Pack override not applying or applying incorrectly | `runbooks/pack-override-cascade.md` | SEV-2 |
| Stale targeting rule causing incorrect variant assignment | `runbooks/stale-targeting-rule.md` | SEV-3 |
| a11y flag violation (accessibility regression) | `runbooks/a11y-flag-violation.md` | SEV-3 |
| Flag evaluation regression / latency spike | `runbooks/flag-evaluation-regression.md` | SEV-2/3 |

## First-response checklist (all incidents)

1. Acknowledge PagerDuty alert within SLA.
2. `kubectl get pods -n feature-flags` — verify pod health across cells.
3. Check dashboards: `dashboards/flag-state-overview.json` (current flag state), `dashboards/killswitch-history.json` (recent kill-switches).
4. Check metrics: `oya_feature_flag_eval_duration_seconds` (latency), `oya_feature_flag_eval_total` (volume), `oya_feature_flag_killswitch_active` (active kill-switches).
5. If emergency-services path affected: SEV-1 immediately regardless of metric thresholds.
6. Open incident channel in Slack: `#incident-<timestamp>`.
7. Determine if kill-switch is the right response (see `runbooks/killswitch-engaged.md`).

## Communication templates

### SEV-1 initial (within 5 minutes)

```
INCIDENT OPEN: feature-flags SEV-1
Time: <timestamp>
Impact: <describe: evaluations unavailable / kill-switch delayed>
Affected tenants: <all / specific segment>
Current status: investigating
IC: <name>
Bridge: <link>
Next update: 15 minutes
```

### SEV-1 update (every 15 minutes)

```
INCIDENT UPDATE: feature-flags SEV-1
Time: <timestamp>
Status: <investigating / identified / mitigating / monitoring>
Root cause hypothesis: <hypothesis>
Actions taken: <list>
ETA to resolution: <estimate>
Next update: <time>
```

## Post-incident

- Post-mortem required for all SEV-1 and SEV-2 incidents.
- Post-mortem template: `docs/templates/post-mortem-template.md`.
- Timeline: draft within 24h; published within 5 business days.
- Action items: tracked in Jira; each assigned owner + due date.
- Repeat incidents: second occurrence triggers immediate architectural review.

## Emergency kill-switch decision tree

```
Is there a production regression affecting users?
  YES → Can it be reversed by disabling a flag?
          YES → Engage kill-switch (runbooks/killswitch-engaged.md)
          NO → Rollback deployment (runbooks/flag-mutation-cascade.md §rollback)
  NO  → Monitor; do not engage kill-switch pre-emptively

Is the kill-switch for an emergency-services flag?
  YES → Emergency-services bypass MUST be active; do not add friction to emergency path
  NO  → Normal kill-switch procedure
```
