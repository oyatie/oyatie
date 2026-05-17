---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-cell-substrate, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/threat-model.md
  - microservices/cell/dpia.md
  - microservices/cell/compliance.md
  - microservices/cell/failure-modes.md
  - microservices/cell/multi-region.md
  - microservices/cell/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (cell µservice)

## Purpose

End-to-end incident-response procedure for cell substrate events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, per-pack regulatory-notification timelines.

## Severity Definitions

Per Bominal ADR-0028 (inherited) + oyatie incident-severity standard.

| Severity | Definition | Response time (page→ack) | Examples |
|---|---|---|---|
| **Sev-1** | Cross-cell or cross-pack confidentiality, integrity, or availability impact; regulatory-notification triggers; data breach | ≤ 5 min (24/7) | FM-02 cell-boundary lane drift; FM-07 cross-cell query; FM-09 cross-pack attempt; FM-11 split-brain |
| **Sev-2** | Single-cell or substrate-availability impact; operational degradation; gate fail-closed (safe default applies) | ≤ 15 min (24/7) | FM-01 Postgres outage; FM-03 scheduler outage; FM-04 pool exhaustion; FM-12 SPIRE outage; FM-13 Cluster API outage |
| **Sev-3** | Localized impact; degraded but functional | ≤ 1h (business hours) | FM-05 cell-create timeout; FM-06 migration race (well-handled); FM-08 host drain stuck; FM-14 cache poison |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident from declaration to closure |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps; performs DR failover |
| **Communications Lead** | gtm-customer-success | Drafts + sends tenant + status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-cell-substrate + cloud-k8s | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for data-breach-suspect (Sev-1 confidentiality); owns regulatory chain |
| **Executive Sponsor** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org comms |
| **Scribe** | Any on-call | Captures timeline in `#inc-<id>` Slack |

## Escalation Path

```text
Alert fires (Mimir → Alertmanager → Grafana OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (no ack 5min)
Re-paged
    ↓ (no ack 10min total)
Secondary on-call paged
    ↓ (no ack 15min)
Engineering manager (axis-cell-substrate lead) paged + Slack alert
    ↓ (Sev-1; no resolution 30min)
Director + ops-security engaged
    ↓ (Sev-1; breach-suspect)
council-privacy + ExecSponsor engaged
    ↓ (confirmed breach)
Regulatory notification chain begins (§"Regulatory Notifications")
    ↓ (GDPR-scope + data-subject impact)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1 / Sev-2 alert fires BOTH a Mimir metric AND an OnCall page; the on-call playbook requires both to be checked.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>` Slack; pages IC | ≤ 5min (Sev-1) / ≤ 15min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10min |
| **4. Containment** | Operations Lead executes immediate-mitigation per `failure-modes.md` | per RTO |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook executed; service restored | per RTO |
| **7. Communication** | Tenant + status page + regulator notifications | per §"Regulatory Notifications" |
| **8. Closure** | IC declares resolved; ≥ 30min steady state | – |
| **9. Postmortem** | Within 5 business days | – |
| **10. Action items** | Tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1 / Sev-2 declaration.
- Updated every 30 min during active.
- Final resolution update within 30 min of closure.
- `status.oyatie.dev`.

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / cell] Incident in <pack>: <one-line summary>

We are investigating an incident affecting the cell substrate in <pack> that may
impact your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating
| Resolved>. ETA to resolution: <est>.

What you may experience: <impact>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update you again within 30 minutes or upon resolution, whichever is sooner.
If this involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev>
For questions: <support@oyatie.dev>
```

Template — Sev-2 (operational; non-data-affecting): similar but lighter.

## Regulatory Notifications

| Framework | Trigger | Timeline | Owner |
|---|---|---|---|
| GDPR Art. 33 (EU DPA) | personal data breach affecting EU data subjects | 72h to DPA; without undue delay to data subjects under Art. 34 if high risk | Privacy Lead + Comms Lead |
| GDPR Art. 34 | high-risk-to-subjects breach | without undue delay | Privacy Lead |
| HIPAA §164.404 (HHS OCR) | breach of unsecured PHI | 60 days to affected individuals; 60 days to HHS for ≥ 500 individuals; annual aggregate to HHS for < 500 | Privacy Lead + BAA-tenant |
| KR PIPA Art. 34 (PIPC) | personal information leakage | 72h initial; 30 days for full notification to data subjects | Privacy Lead |
| NIS2 (2022/2555) | significant cyber incident (when oyatie crosses Annex I/II thresholds) | 24h initial; 72h detailed; 1mo final | ops-security + leadership |
| SEC / regulator | financial-services tenant impact (when applicable) | per tenant's own regulator obligations + BAA | tenant-facing; oyatie supports |
| Other packs (PDPA SG / Privacy Act AU / DPDPA IN / LGPD BR / UAE PDPL / KSA PDPL) | per local regulator | per local timeline (typically 72h) | Privacy Lead |

## On-Call Rotation

Per `runbooks/scheduler-restart.md` §"On-Call Rotation" (cross-references observability oncall-rotation conventions):

| Tier | Pool | Rotation | Pay |
|---|---|---|---|
| ops-sre-reliability primary | 6 engineers | Weekly; follow-the-sun (KR / EU / US) | per company policy |
| ops-sre-reliability secondary | Same pool, offset 1 week | Weekly | per policy |
| axis-cell-substrate SME | 3 engineers | Weekly; KR + EU primary; US business-hours fallback | per policy |
| ops-security on-call | 4 engineers | Weekly; 24/7 for Sev-1 confidentiality | per policy |
| council-privacy chair | Permanent named | Always-on for breach-suspect | – |
| Executive Sponsor | Permanent named | Sev-1 only | – |

## Post-Incident Process

| Step | Action | Owner |
|---|---|---|
| 1 | IC publishes incident summary in `#inc-<id>` within 24h of closure | IC |
| 2 | Postmortem written within 5 business days | SME + IC |
| 3 | Postmortem published to `evidence/postmortems/<year>/<incident-id>.md` | SME |
| 4 | Action items tracked (typically: lane gap fix; runbook update; alert tuning) | SME + axis-cell-substrate |
| 5 | Action items closed via PRs going through the SLO gate | SME |
| 6 | Quarterly trend review: incident class distribution | ops-sre-reliability |

## References

- Bominal ADR-0028 (audit-chain).
- ADR-0117 (cloud-native infra); ADR-0130 (SLO gate); ADR-0131 (per-µservice).
- `microservices/cell/threat-model.md`; `microservices/cell/dpia.md`; `microservices/cell/compliance.md`.
- `microservices/cell/failure-modes.md`; `microservices/cell/multi-region.md`.
- `docs/standards/incident-severity.md`.
- Google SRE Workbook chs. 5, 6, 11, 12.
