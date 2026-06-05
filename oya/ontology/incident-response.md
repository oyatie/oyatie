---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-ontology, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/ontology/threat-model.md
  - microservices/ontology/dpia.md
  - microservices/ontology/compliance.md
  - microservices/ontology/failure-modes.md
  - microservices/ontology/multi-region.md
  - microservices/ontology/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (ontology µservice)

## Purpose

End-to-end incident-response procedure for ontology events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` and from `compliance.md`.

## Severity Definitions

Per Bominal ADR-0028 + oyatie `docs/standards/incident-severity.md`.

| Severity | Definition | Response time | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier confidentiality, integrity, or availability impact affecting multiple tenants; regulatory-notification triggers; data breach; safety violation | ≤ 5 min (24/7) | FM-03 RLS drift; FM-07 cross-tenant leak; FM-13 audit tampering; FM-16 pack misroute |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; gate fail-closed | ≤ 15 min (24/7) | FM-01 Postgres coordinator outage; FM-02 Citus shard failure; FM-04 schema corruption; FM-05 query OOM; FM-06 Cedar runaway; FM-08 tier escape; FM-09 audit lag; FM-11 deprecation broke tenant; FM-14 cross-pillar misuse; FM-15 DSR cascade timed out |
| **Sev-3** | Localized impact; degraded but functional | ≤ 1 h (business hours) | FM-10 ClickHouse lag; FM-12 agent loop |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard typo |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident; coordinates roles |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook; performs DR failover if needed |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts tenant + status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-ontology + relevant workload owner | Diagnoses root cause; proposes mitigation |
| **Privacy Lead (PrivacyLead)** | council-privacy chair | Activates for data-breach-suspect; owns regulatory notification chain |
| **Executive Sponsor (ExecSponsor)** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in `#inc-<id>` Slack |

## Escalation Path

```text
Alert fires (Postgres/ClickHouse/Cedar/Action engine → Alertmanager → Grafana OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (if no ack in 5min)
Primary re-paged
    ↓ (if no ack in 10min total)
Secondary on-call paged
    ↓ (if no ack in 15min)
Engineering manager (axis-ontology lead) paged + Slack alert
    ↓ (if Sev-1 and no resolution in 30min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (if Sev-1 and breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (if confirmed breach)
Regulatory notification chain begins
    ↓ (if confirmed data subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1/Sev-2 alert fires BOTH a Postgres-emitted oya_incident_active{} metric AND a Grafana OnCall page. If one channel is silent, the other still fires.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60 s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>`; pages IC | ≤ 5 min (Sev-1) / ≤ 15 min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10 min |
| **4. Containment** | OpsLead executes mitigation from `failure-modes.md` | varies |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures; service restored | per RTO in `failure-modes.md` |
| **7. Communication** | CommsLead notifies tenants; regulatory notification if data-impactful | per §"Regulatory Notifications" |
| **8. Closure** | IC declares incident resolved; steady state ≥ 30 min | – |
| **9. Postmortem** | Within 5 business days | ≤ 5 business days |
| **10. Action items** | Postmortem-generated remediation tracked | indefinite |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1 / Sev-2 declaration.
- Updated every 30 min during active incident.
- Final resolution update within 30 min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / ontology] Incident in <pack>: <one-line summary>

We are investigating an incident affecting <component> in <pack> that may impact
your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating |
Resolved>. ETA to resolution: <est>.

What you may experience: <impact, e.g., Object Type reads return error;
Function evaluations timeout; Action invocations refused>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update you again within 30 minutes or upon resolution, whichever is sooner.
If this impact involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev link>
For questions: <support email>
```

Template — Sev-2 (operational, no data impact):

```
Subject: [Sev-2 / ontology] Degradation in <pack>: <one-line summary>

We are investigating a service degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated Function read latency,
delayed Action invocation>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.

For real-time updates: <status.oyatie.dev link>
```

### Customer-facing message template

Provided at the tenant onboarding portal; pre-localized per pack. Tenants retain editorial control; oyatie suggests but does not draft.

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72 hours: notify lead DPA via the DPA's online portal. |
| Breach with high risk to data subjects (Art. 34) | Also notify affected data subjects without undue delay. |
| Late notification | Justify the delay in the same notification. |

Template — DPA notification: see observability incident-response.md (shared template).

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting fewer than 500 individuals | OCR within 60 days of end of calendar year. |
| Breach affecting 500+ individuals | OCR within 60 days + media notification + individual notification. |
| Business Associate (oyatie) | Notify covered-entity tenant within BAA-specified window (typically 24h–7d). |

### KR PIPA Art. 34 (Personal Information Protection Commission)

| Event | Notification |
|---|---|
| Breach affecting 1+ data subjects | Notify affected data subjects within 72 hours. |
| Breach affecting 1000+ data subjects OR sensitive data (Art. 23) | Notify PIPC within 72 hours + publish on website. |

### APPI Art. 26-2 (Japan PPC)

72-hour notification target.

### LGPD Art. 48 (Brazil ANPD)

2-business-day target.

### DPDPA 2023 (India DPB)

72-hour notification.

### PDPA (Singapore, Australia, etc.)

Per-pack timelines at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/incident-notification-overlay.md`.

### NIS2 (EU 2022/2555)

Early warning ≤ 24h; incident notification ≤ 72h; final report ≤ 1mo.

### KR-FSS

Within 24 hours for incidents affecting financial-data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days of resolution, IC convenes postmortem.
2. Scribe's timeline is starting input.
3. Postmortem covers:
   - Summary (5 lines)
   - Timeline
   - Impact (tenant-facing + internal)
   - Root cause (5-whys; cite FM-ID)
   - Lessons learned
   - Action items (owned + scheduled)
   - Runbook adequacy assessment
   - Trust-portal entry (if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture.

**Blameless culture per Google SRE Workbook ch. 12.**

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers) | follow-the-sun |
| ops-sre-reliability secondary | weekly (offset 1 week) | – |
| axis-ontology SME | weekly (4 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent | breach-suspect activation |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff procedure per `runbooks/oncall-rotation.md` (shared with observability rotation; reused here).

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice ontology` — exit 0; every FM-ID has a matching runbook.
- Quarterly DR failover drill validates response chain end-to-end (per `multi-region.md`).
- Annual tabletop exercise simulates a Sev-1 regional outage; comms + regulatory notification chain rehearsed.

## References

- `microservices/ontology/failure-modes.md` (FM-IDs + severity).
- `microservices/ontology/compliance.md` §"Regulatory Notifications".
- `microservices/ontology/multi-region.md` (DR failover).
- `microservices/ontology/runbooks/*`.
- `microservices/ontology/dpia.md`.
- `microservices/ontology/threat-model.md`.
- `docs/standards/incident-severity.md`.
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14.
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555.
