---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-application, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md
  - microservices/application/dpia.md
  - microservices/application/compliance.md
  - microservices/application/failure-modes.md
  - microservices/application/multi-region.md
  - microservices/application/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (application µservice)

## Purpose

End-to-end incident response for Application Shell events. Covers severity
classification, response roles, escalation paths, communication templates,
postmortem cadence, and per-pack regulatory-notification timelines. Cross-
referenced from `failure-modes.md` (severity per failure) and from
`compliance.md` (regulatory frameworks requiring notification).

## Severity Definitions

Per Bominal ADR-0028 inherited; oyatie incident-severity standard
(`docs/standards/incident-severity.md`).

| Severity | Definition | Page-to-ack target | Examples |
|---|---|---|---|
| Sev-1 | Production confidentiality, integrity, or availability impact affecting multiple tenants; regulatory-notification triggers; data breach; auth bypass | ≤ 5 min (24/7 on-call paged) | FM-01 CDN poisoning; FM-02 module integrity fail; FM-07 cross-tenant route; FM-13 pack misroute |
| Sev-2 | Single-tenant or sub-tenant impact; operational degradation without data loss; gate fail-closed | ≤ 15 min (24/7 on-call paged) | FM-04 WASM corruption; FM-05 IdP outage; FM-06 session storm; FM-16 TTI breach |
| Sev-3 | Localized impact; degraded but functional | ≤ 1 h (business hours) | FM-08 shard imbalance; FM-11 CDN purge backlog |
| Sev-4 | Cosmetic | next business day | dashboard label typo; doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| Incident Commander (IC) | Rotating on-call lead (ops-sre-reliability; ops-security for security incidents) | Owns incident end-to-end |
| Operations Lead (OpsLead) | ops-sre-reliability secondary | Executes runbook; DR failover if needed |
| Communications Lead (CommsLead) | gtm-customer-success | Drafts tenant + status-page + regulatory notifications |
| SME (Subject-Matter Expert) | axis-application + relevant BC owner | Diagnoses root cause |
| Privacy Lead (PrivacyLead) | council-privacy chair | Activates for Sev-1 confidentiality; owns regulatory notification chain |
| Executive Sponsor (ExecSponsor) | council-architecture chair (Sev-1) | Decision-rights for cross-org or external comms |
| Scribe | Any on-call team member | Captures timeline + decisions in `#inc-<id>` for postmortem |

## Detection → Page → Ack

```text
Burn-rate / FM detection
    ↓
observability fires Alertmanager → Grafana OnCall
    ↓
Pager: on-call primary (5-min escalation to secondary at 5 min unacked)
    ↓
On-call ack via OnCall API
    ↓
Open #inc-<id> Slack channel (auto-created)
    ↓
IC declared; severity set; roles assigned
```

## Sev-1 Response

1. **0-5 min**: Ack page; declare incident; open `#inc-<id>`; assign IC + OpsLead + CommsLead + SME.
2. **5-15 min**: Identify failure mode (cross-reference `failure-modes.md`); execute runbook's immediate mitigation step.
3. **15-30 min**: Confirm containment; assess data-breach posture (PrivacyLead engages if confidentiality impact suspected); status-page update.
4. **30-60 min**: Verify SLI recovery; CommsLead drafts tenant comms.
5. **60+ min**: PrivacyLead initiates regulatory-notification chain if breach confirmed (timelines below).
6. **Within 5 BDs**: Postmortem published at `evidence/postmortems/<year>/<id>.md`.

## Sev-2 Response

1. **0-15 min**: Ack; declare; assign IC + OpsLead + SME.
2. **15-60 min**: Execute runbook; verify recovery.
3. **60 min - 4 h**: Drafted tenant comms (if customer-impacting).
4. **Within 5 BDs**: Postmortem (if material lessons).

## Sev-3 / Sev-4

Tracked in ticket queue; runbook-executed during business hours; no
postmortem unless recurrence pattern detected.

## Regulatory-Notification Timelines

| Framework | Notification trigger | Timeline | Recipient |
|---|---|---|---|
| GDPR Art. 33 | Personal-data breach | ≤ 72 h (data-subject if high risk) | Lead supervisory authority + data subjects |
| KR PIPA Art. 34 | Personal-data leak | ≤ 72 h | PIPC + affected users |
| HIPAA Breach Notification Rule | PHI breach | ≤ 60 days (≤ 60 to OCR; ≤ 60 to individuals) | HHS OCR + Covered Entity tenant |
| CPRA / CCPA | Personal info breach | per state AG rules | California AG + affected residents |
| LGPD Art. 48 | Data subject impact | reasonable time | ANPD + data subjects |
| SOC 2 (no regulator) | Material control failure | Per audit period | Auditor + tenant per DPA |
| DPDPA 2023 (IN) | Personal data breach | ≤ 72 h | Data Protection Board |
| UAE PDPL | Significant impact | per regulation | UAE DPA |
| KSA PDPL | Data breach | per regulation | SDAIA |

## Communication Templates

### Tenant comms (Sev-1, customer-impacting)

```
Subject: [oyatie] Sev-1 incident — Application Shell — <YYYY-MM-DD HH:MM UTC>

Summary: <one-line technical description>
Impact: <which tenants / which surfaces / since when>
Status: investigating | mitigated | resolved
Next update: <time> (or as developments warrant)

Workarounds: <if any>
Workgroup contacts: incidents@oyatie.dev
```

### Status page (Sev-1 / Sev-2)

- Public status page at `status.oyatie.dev` (Anonymous Cedar read).
- Updated within 15 min of incident declaration.
- Updated on every state transition (investigating → identified → mitigating → monitoring → resolved).

### Postmortem template

`docs/templates/postmortem-template.md` — covers: timeline, root cause,
contributing factors, customer impact, recovery actions, follow-up
action items.

## Escalation Matrix

| Trigger | Escalate to |
|---|---|
| 15-min ack failure | Secondary on-call |
| 30-min unresolved Sev-1 | ExecSponsor (council-architecture chair) |
| Cross-µservice impact (Sev-1) | All affected µservice axis leads |
| Regulatory notification needed | PrivacyLead + ExecSponsor |
| Sub-processor escalation needed | gtm-customer-success + ops-legal |

## On-call Rotation

- Primary: ops-sre-reliability (24/7 rotation; weekly).
- Secondary: axis-application BC owner (24/7; weekly).
- Security on-call: ops-security (24/7; bi-weekly; pages first on Sev-1 security incidents).
- All rotations managed in Grafana OnCall.

## Postmortem Cadence

- Sev-1: Published within 5 business days; reviewed by council-architecture.
- Sev-2: Published within 10 business days (if material).
- Action items: tracked in `evidence/postmortems/action-items.jsonl`;
  closed via PRs themselves SLO-gate-enforced.

## References

- ADR-0028 audit chain (audit emission on every state transition).
- ADR-0123 cross-product auth (hyperscaler maturity gate).
- `microservices/application/failure-modes.md` (severity per failure).
- `microservices/application/compliance.md` (regulatory frameworks).
- `microservices/observability/incident-response.md` (precedent).
