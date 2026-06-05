---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-observability, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/observability/threat-model.md
  - microservices/observability/dpia.md
  - microservices/observability/compliance.md
  - microservices/observability/failure-modes.md
  - microservices/observability/multi-region.md
  - microservices/observability/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (observability µservice)

## Purpose

End-to-end incident-response procedure for observability events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` (which classifies failures by severity) and from `compliance.md` (which lists the regulatory frameworks requiring notification).

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard (`docs/standards/incident-severity.md`).

| Severity | Definition | Response time (target page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier confidentiality, integrity, or availability impact affecting multiple tenants; regulatory-notification triggers; data breach; safety violation | ≤ 5 min (24/7 on-call paged) | FM-02 tenancy drift; FM-06 cross-tenant leak; FM-13 pack misroute |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; gate fail-closed (safe default applies) | ≤ 15 min (24/7 on-call paged) | FM-01 Mimir distributor outage; FM-03 worker outage; FM-04 object-storage outage; FM-07 PII redactor failure |
| **Sev-3** | Localized impact; degraded but functional; backlogged operations | ≤ 1 h (business hours; on-call eventually) | FM-12 mesh weight stuck; FM-14 lane flaky; FM-15 capacity exhaustion |
| **Sev-4** | Cosmetic; no operational impact; tracked but not paged | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident from declaration to closure; coordinates all other roles |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook steps; performs DR failover if needed |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts and sends tenant + public status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-observability + relevant workload owner | Diagnoses root cause; proposes mitigation |
| **Privacy Lead (PrivacyLead)** | council-privacy chair | Activates for any data-breach-suspect incident (Sev-1 confidentiality); owns regulatory notification chain |
| **Executive Sponsor (ExecSponsor)** | council-architecture chair (Sev-1 only) | Decision-rights for any cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in incident channel (Slack `#inc-<id>`) for the postmortem |

## Escalation Path

```text
Alert fires (Mimir/Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (if no ack in 5min)
Primary on-call re-paged
    ↓ (if no ack in 10min total)
Secondary on-call paged (ops-sre-reliability secondary)
    ↓ (if no ack in 15min)
Engineering manager (axis-observability lead) paged + Slack alert
    ↓ (if Sev-1 and no resolution in 30min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (if Sev-1 and breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (if confirmed breach)
Regulatory notification chain begins (see §"Regulatory Notifications")
    ↓ (if confirmed data subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1 / Sev-2 alert fires BOTH a Mimir metric (`oya_incident_active{severity="N"}` with the incident ID label) AND an OnCall page. If one channel is silent, the other still fires — the on-call playbook requires both to be checked.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>` Slack channel; pages IC role | ≤ 5min (Sev-1) / ≤ 15min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts incident timeline | ≤ 10min |
| **4. Containment** | OpsLead executes immediate-mitigation steps from `failure-modes.md`; Privacy Lead engaged if suspect | varies; aim for stabilization within RTO |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures executed; service restored | per RTO targets in `failure-modes.md` |
| **7. Communication** | CommsLead notifies tenants (status page + email); regulatory notification per pack timelines if data-impactful | per §"Regulatory Notifications" |
| **8. Closure** | IC declares incident resolved; service in steady state for ≥ 30min | – |
| **9. Postmortem** | Within 5 business days; published to ops-sre-reliability + council-architecture + auditors | ≤ 5 business days |
| **10. Action items** | Postmortem-generated remediation items tracked + owned + scheduled | indefinite (until done) |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1 / Sev-2 declaration.
- Updated every 30 min during active incident.
- Final resolution update within 30 min of closure.
- Lives at `status.oyatie.dev` (per cloud-iac µservice).

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / observability] Incident in <pack>: <one-line summary>

We are investigating an incident affecting <component> in <pack> that may impact
your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating |
Resolved>. ETA to resolution: <est>.

What you may experience: <impact>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update you again within 30 minutes or upon resolution, whichever is sooner.
If this impact involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev link>
For questions: <support email>

Your tenant onboarding contact: <name>
```

Template — Sev-2 (operational, no data impact):

```
Subject: [Sev-2 / observability] Degradation in <pack>: <one-line summary>

We are investigating a service degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated dashboard latency, delayed promotion>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.

For real-time updates: <status.oyatie.dev link>
```

### Customer-facing message template (tenant forwards to its end-users)

Provided at the tenant onboarding portal; pre-localized per pack. Tenants retain editorial control; oyatie suggests but does not draft.

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72 hours: notify lead DPA (per tenant's establishment) via the DPA's online portal. |
| Breach with high risk to data subjects (Art. 34) | Also notify affected data subjects without undue delay. |
| Late notification | Justify the delay in the same notification. |

Template — DPA notification:

```
To: <Lead Supervisory Authority — typically Irish DPC for many global SaaS,
    or the tenant's local DPA>
From: <oyatie council-privacy chair as DPO>
Subject: Personal data breach notification under GDPR Art. 33

Date / time of breach discovery: <ISO8601>
Date / time of breach occurrence (if different): <ISO8601>
Nature of breach: <categories of personal data + categories of data subjects>
Approximate number of records affected: <est>
Likely consequences: <e.g., possible exposure of email + IP correlation>
Measures taken / proposed: <mitigation + DSR support>
DPO contact: <council-privacy chair>
Joint controller cascade: tenant <tenant_id_redacted> notified at <ISO8601>;
                         tenant is informing its data subjects per Art. 34 where applicable.
```

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting fewer than 500 individuals | OCR notification within 60 days of end of calendar year. |
| Breach affecting 500+ individuals | OCR within 60 days + media notification (per §164.406) + individual notification (per §164.404). |
| Business Associate (oyatie) | Notify covered-entity tenant within the BAA-specified window (typically 24h to 7d). |

### KR PIPA Art. 34 (Personal Information Protection Commission)

| Event | Notification |
|---|---|
| Breach affecting 1+ data subjects | Notify affected data subjects within 72 hours. |
| Breach affecting 1000+ data subjects OR sensitive data (Art. 23) OR resident registration numbers | Notify PIPC within 72 hours + publish on website. |

### APPI Art. 26-2 (Japan PPC)

| Event | Notification |
|---|---|
| Leakage of personal information affecting 1+ persons | Notify PPC + affected individuals within reasonable time (typically within 72h of discovery). |

### LGPD Art. 48 (Brazil ANPD)

| Event | Notification |
|---|---|
| Security incident affecting personal data | Notify ANPD + data subjects within "reasonable period" (ANPD guidance: 2 business days). |

### DPDPA 2023 (India DPB)

| Event | Notification |
|---|---|
| Personal-data breach | Notify the Data Protection Board within 72 hours of awareness. |

### PDPA (Singapore, Australia, etc.)

Per-pack timelines in `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/incident-notification-overlay.md`. Universally: notify-data-subjects-and-supervisor within 72 hours target across the major jurisdictions.

### NIS2 (EU 2022/2555)

For oyatie when crossing Annex I/II thresholds:
- Early warning: ≤ 24 hours of awareness.
- Incident notification: ≤ 72 hours.
- Final report: ≤ 1 month.

### KR-FSS (financial-services KR tenants)

Notify FSS within 24 hours for incidents affecting financial data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days of incident resolution, IC convenes postmortem meeting.
2. Scribe's timeline is the starting input.
3. Postmortem document covers:
   - Summary (5 lines)
   - Timeline (chronological events with timestamps)
   - Impact (tenant-facing + internal-facing)
   - Root cause (5-whys; cite the FM-ID from `failure-modes.md`)
   - Lessons learned
   - Action items (each owned + scheduled)
   - Was the runbook adequate? (yes / partial / no + improvement)
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

**Blameless culture per Google SRE Workbook ch. 12**: postmortems focus on systems + processes, never on individuals; the postmortem document is privileged information used for improvement.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers; ~7 weeks between rotations) | follow-the-sun: KR shift / EU shift / US shift |
| ops-sre-reliability secondary | weekly (same pool; offset by 1 week) | – |
| axis-observability SME | weekly (3 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent | always-on-call for breach-suspect; engaged manually |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff procedure per `runbooks/oncall-rotation.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice observability` — exit 0; every FM-ID has a matching runbook.
- Quarterly DR failover drill validates the response chain end-to-end (per `multi-region.md`).
- Annual tabletop exercise simulates a Sev-1 regional outage; comms + regulatory notification chain rehearsed.

## References

- `microservices/observability/failure-modes.md` (FM-IDs + severity classification).
- `microservices/observability/compliance.md` §"Regulatory Notifications" (per-pack timelines).
- `microservices/observability/multi-region.md` (DR failover).
- `microservices/observability/runbooks/*` (per-scenario procedures).
- `microservices/observability/dpia.md` (data-subject impact assessment).
- `microservices/observability/threat-model.md` (security-incident threat IDs).
- `docs/standards/incident-severity.md` (cross-cutting severity standard).
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14 (Postmortem culture; managing incidents; oncall).
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555.
