---
purpose: Canonical on-call standard. Defines rotation cadence, runbook discipline, escalation paths, blameless-postmortem trigger, and SLO-burn-rate alerting thresholds.
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Canonical on-call standard. Defines rotation cadence, runbook discipline,
  escalation paths, blameless-postmortem trigger, and SLO-burn-rate alerting
  thresholds. Inspired by the Google SRE postmortem culture and SRE Workbook
  error-budget-policy chapter. Resolves the `standards/on-call.md`
  forward-reference sentinel in `docs/AGENTS.md` canonical doc map.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-runbook-index-resolves
companion_docs:
  - docs/INCIDENT-MANAGEMENT.md
  - docs/SLO-CATALOG.md
  - docs/RUNBOOKS-INDEX.md
  - docs/standards/observability.md
  - docs/standards/release-management.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# On-Call

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

[`docs/INCIDENT-MANAGEMENT.md`](../INCIDENT-MANAGEMENT.md) defines the per-incident
playbook (severity, IM/CM roles, comms). This standard defines the steady-state
on-call posture: rotation, alerts, runbooks, escalation, and the trigger for
the blameless postmortem.

## 1. Rotation cadence

Per axis SRE team:

- **Primary** + **Secondary** at all times. No solo on-call.
- Weekly handoff (Monday 09:00 KST), with a 30-min "what's burning" briefing
  covering: open incidents, error-budget state per SLO, suppressed alerts,
  pending capacity work, on-call-load metric for the prior week.
- Maximum shift load: 12 hours of paged work / week. Beyond this, the
  rotation is short-staffed and the team lead opens a capacity issue
  (Google SRE precedent — "on-call should not exceed 25% of weekly work
  capacity").
- Compensation: paid per company policy; the rotation is not volunteer
  labor.
- Cross-axis backup: each axis names a buddy axis whose secondary covers a
  paged primary who fails to acknowledge within 10 minutes.

Source: [Google SRE — Being On-Call](https://sre.google/sre-book/being-on-call/).

## 2. Alerting — SLO-burn-rate thresholds

Alerts derive from SLOs in [`docs/SLO-CATALOG.md`](../SLO-CATALOG.md). The
canonical model is **multi-window, multi-burn-rate** (Google SRE Workbook
§5).

| Burn rate | Window | Action | Page? |
|---|---|---|---|
| 14.4× | 1h | "ticket budget exhausted in 5% of window" | YES (Sev-2 page) |
| 6× | 6h | "ticket budget exhausted in 10% of window" | YES (Sev-3 page) |
| 1× | 3d | "tracking on plan" | NO (advisory) |
| 0.1× | 30d | "ahead of plan; capacity available" | NO (informational) |

Rules:

1. Every public surface in `docs/SPEC.md` has an SLO entry (lane:
   `slo-surface-coverage`).
2. Every paging alert resolves to **exactly one runbook URL** in the alert
   body. Lane: `governance-alert-runbook-link`.
3. Suppressed alerts MUST have an expiry timestamp; auto-renewal is
   forbidden. Suppression > 7 days requires team-lead approval logged
4. The **error-budget policy** is in
   [`release-management.md`](release-management.md) §6 (release-gate
   semantics) — exhausted budget freezes feature work until burn rate
   recovers.

Sources: [Google SRE — Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/),
[Google SRE Workbook — Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/),
[Google SRE Workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/),
[Splunk — Four Golden Signals](https://www.splunk.com/en_us/blog/learn/sre-metrics-four-golden-signals-of-monitoring.html).

## 3. Runbook discipline

Every runbook lives under `docs/runbooks/` and is registered in
[`docs/RUNBOOKS-INDEX.md`](../RUNBOOKS-INDEX.md). Per the
`runbook-freshness` lane (DOC-CATALOG.md §4):

| Severity tier | Verification cadence |
|---|---|
| Sev-1 | 30 days |
| Sev-2 | 60 days |
| Sev-3 | 180 days |
| Sev-4 | 365 days |

Required runbook shape (per `templates/runbook-template.md`):

1. **Last verified**: ISO date (parsable).
2. **Severity tier**: Sev-1 / Sev-2 / Sev-3 / Sev-4.
3. **SLO links**: list of `SLO-*` IDs.
4. **Symptoms**: how the on-call recognizes this incident.
   for the dual-audience contract).
6. **Mitigation**: ordered steps with explicit "if X fails, escalate to Y".
7. **Recovery verification**: how to confirm green.
8. **Postmortem trigger**: when this becomes a blameless postmortem (§5).
9. **Linked ADRs / MISTAKES rows**.

Lanes:

- `governance-runbook-discoverability` — every runbook indexed.
- `governance-runbook-orphan-check` — no runbook references a
  deleted SLO or capability.
- `governance-runbook-freshness` — `Last verified` within cadence.

## 4. Escalation paths

```
Page (Sev-2/3)  ──► Primary  ──(10 min unack)──►  Secondary
                                  │
                                  └──(10 min unack)──► Buddy axis secondary
                                                          │
                                                          └──► Team lead

Sev-1 (production-down or data-loss)  ──► Page primary + secondary + Incident Commander
                                                                  │
                                                                  ├──► Council-Architecture
                                                                  └──► Founder (CC)
```

Sev-1 declaration triggers `EVT-INCIDENT-OPENED` and pages the **Incident
Commander rotation** (per
[`docs/INCIDENT-MANAGEMENT.md`](../INCIDENT-MANAGEMENT.md)). The IC owns the
public-facing comms cadence; the on-call owns the technical mitigation.

## 5. Blameless postmortem trigger

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) (FO-10 mechanical-prevention doctrine), every
Sev-1 / Sev-2 closure triggers a postmortem. The standard adopts Google's
blameless culture verbatim:

> "Assume everyone acted in good faith with the information they had. You
> can't fix people, only their environment."

Postmortem shape (per `templates/postmortem-template.md`):

1. **Summary** — 3 sentences, plain English.
2. **Impact** — duration, customers affected, SLO budget consumed,
   regulatory implications.
3. **Timeline** — UTC + local timestamps, source citations (logs,
   dashboards, audit chain).
4. **Root cause** — single failure mode named in one sentence (per
   forbidden-operations.json FO-10 mistakes doctrine).
5. **Mitigation** — what stopped the bleeding; how long it took; gaps.
6. **Mechanical prevention** — the CI lane, hook, validator, schema check,
   or runtime gate that prevents replay. Class: `mechanical` or
   `cultural`. Adds a row to
   [`docs/MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md).
7. **Replay-as-eval** — the prevention is testable on the original failure
   mode; the test goes into `tests/regressions/` or the audit-chain
   replay fixtures.
8. **Lessons** — system lessons; never people lessons.
9. **Audit-chain emission** — `EVT-POSTMORTEM-PUBLISHED`.

Postmortems are **published broadly** (the practice is the single most
copied SRE export). Source:
[Google SRE — Postmortem Culture](https://sre.google/sre-book/postmortem-culture/),
[Google SRE Workbook — Postmortem Culture](https://sre.google/workbook/postmortem-culture/),
[Google Cloud — Fearless Shared Postmortems](https://cloud.google.com/blog/products/gcp/fearless-shared-postmortems-cre-life-lessons).

## 6. Suppressed alerts + alert tuning

- Every paging alert has a documented threshold derived from an SLO.
- Alerts that page > 2× per week without action SHOULD be re-tuned or
  retired. The `alerts/tune-weekly.md` runbook is run by the team lead.
- Suppressions appear in the on-call handoff briefing (§1).

## 7. On-call ergonomics

- Pagers route via a redundant pair (PagerDuty + Slack ping; or
  equivalent) so a single-vendor outage does not silence the rotation.
- Acknowledgement window: 10 minutes for Sev-1; 15 minutes for Sev-2.
- Out-of-hours pages MUST be followed up with a 24-hour rest window
  before the engineer returns to feature work.
- No "follow-the-sun" hand-off unless the team is staffed in ≥ 2
  timezones; otherwise the secondary covers nights.

## 8. Audit-chain emissions

Every on-call event of consequence emits an audit-chain record per
[`observability.md`](observability.md):

| Event | When | Required fields |
|---|---|---|
| `EVT-INCIDENT-OPENED` | Sev-1/2 declared | sev, surface, ic, primary |
| `EVT-INCIDENT-CLOSED` | mitigation verified | duration, slo_budget_consumed |
| `EVT-POSTMORTEM-PUBLISHED` | postmortem merged | postmortem_url, prevention_class |
| `EVT-RUNBOOK-EXECUTED` | runbook step ran | runbook_id, step_id, actor |
| `EVT-ALERT-SUPPRESSED` | suppression added | alert_id, expiry, approver |

## 9. Anti-patterns

1. **Solo on-call.**
2. **Alert with no runbook link.**
3. **Suppression without expiry.**
4. **Postmortem that names a person.**
5. **"Process fix"** for a recurring incident (per forbidden-operations.json FO-10 mistakes
   doctrine — mechanical prevention or cultural prevention, not memo-driven
   process).
6. **Pager fatigue accepted as normal** — re-tune or retire alerts.

## 10. Sources scanned

- [Google SRE — Being On-Call](https://sre.google/sre-book/being-on-call/).
- [Google SRE — Postmortem Culture](https://sre.google/sre-book/postmortem-culture/).
- [Google SRE Workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/).
- [Google SRE Workbook — Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/).
- [Google SRE — Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/).
- [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-10 mistakes doctrine.
- [`docs/INCIDENT-MANAGEMENT.md`](../INCIDENT-MANAGEMENT.md).
- [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 2 "On-call / runbooks".
