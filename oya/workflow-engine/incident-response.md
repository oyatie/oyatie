---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-workflow, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/threat-model.md
  - microservices/workflow-engine/dpia.md
  - microservices/workflow-engine/compliance.md
  - microservices/workflow-engine/failure-modes.md
  - microservices/workflow-engine/multi-region.md
  - microservices/workflow-engine/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (workflow-engine µservice)

## Purpose

End-to-end incident-response procedure for workflow-engine events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` (which classifies failures by severity) and from `compliance.md` (which lists frameworks requiring notification).

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard (`docs/standards/incident-severity.md`).

| Severity | Definition | Response time (target page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier C/I/A impact affecting multiple tenants; regulatory-notification triggers; data breach; safety violation | ≤ 5 min (24/7 on-call paged) | FM-05 Valkey quorum loss cluster-wide; FM-09 cross-tenant subscription leak; FM-13 audit-chain seal gap |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss | ≤ 15 min (24/7 on-call paged) | FM-01 deadlock single-tenant; FM-02 backpressure single-subscriber; FM-03 replay storm; FM-07 outbox crash; FM-10 PII leakage |
| **Sev-3** | Localized impact; degraded but functional; backlogged operations | ≤ 1h (business hours) | FM-04 lock contention; FM-06 spec downgrade attempt; FM-08 ClickHouse drift; FM-11 stuck run; FM-12 event poisoning |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns from declaration to closure; coordinates other roles |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook steps; performs DR failover if needed |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts tenant + status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-workflow | Diagnoses root cause; proposes mitigation |
| **Privacy Lead** | council-privacy chair | Activates for breach-suspect (Sev-1 confidentiality); owns regulatory notification chain |
| **Executive Sponsor** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in `#inc-<id>` Slack |

## Escalation Path

```text
Alert fires (engine self-SLI → observability → Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (no ack in 5 min)
Primary re-paged
    ↓ (no ack in 10 min total)
Secondary on-call paged
    ↓ (no ack in 15 min)
Engineering manager (axis-workflow lead) paged + Slack alert
    ↓ (Sev-1 + no resolution in 30 min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (Sev-1 + breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (confirmed breach)
Regulatory notification chain begins
    ↓ (confirmed data subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1/2 alert fires BOTH a Mimir metric AND an OnCall page. If one channel is silent, the other still fires.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>`; pages IC | ≤ 5 min (Sev-1) / ≤ 15 min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10 min |
| **4. Containment** | OpsLead executes immediate-mitigation steps; Privacy Lead engaged if suspect | varies; aim for stabilization within RTO |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures executed | per RTO in `failure-modes.md` |
| **7. Communication** | CommsLead notifies tenants; regulatory notification per pack timelines | per §"Regulatory Notifications" |
| **8. Closure** | IC declares resolved; steady state for ≥ 30 min | – |
| **9. Postmortem** | Within 5 business days | ≤ 5 business days |
| **10. Action items** | Tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1/2 declaration.
- Updated every 30 min during active incident.
- Final resolution update within 30 min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / workflow-engine] Incident in <pack>: <one-line summary>

We are investigating an incident affecting the workflow execution substrate in <pack>
that may impact your tenant. Started at <ISO8601>.
Current status: <Investigating | Mitigating | Resolved>.
ETA to resolution: <est>.

What you may experience: <impact, e.g., workflow runs delayed; replay-debugger unavailable>
What we're doing: <action>
What you should do: <if anything; usually nothing — engine guarantees durable resumption>

We will update you within 30 minutes or upon resolution.
If this impact involves your tenant's data, we will follow with a separate breach-notification
email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev>
For questions: <support email>
```

Template — Sev-2 (operational):

```
Subject: [Sev-2 / workflow-engine] Degradation in <pack>: <one-line summary>

We are investigating a degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated step latency; subscription consumer lag>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.

For real-time updates: <status.oyatie.dev>
```

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72h)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72h: notify lead DPA |
| Breach with high risk to data subjects (Art. 34) | Also notify data subjects without undue delay |
| Late notification | Justify delay |

Template at `incident-response.md` of observability µservice; equivalent here.

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI < 500 individuals | OCR within 60d of calendar-year end |
| Breach affecting 500+ individuals | OCR within 60d + media + individual notification |
| Business Associate (oyatie) | Notify covered-entity tenant within BAA-specified window (24h to 7d) |

### KR PIPA Art. 34 (PIPC)

| Event | Notification |
|---|---|
| Breach affecting 1+ data subjects | Notify data subjects within 72h |
| Breach affecting 1000+ OR sensitive data (Art. 23) OR resident registration numbers | Notify PIPC within 72h + publish on website |

### APPI Art. 26-2 (Japan PPC)

| Event | Notification |
|---|---|
| Leakage of personal info | Notify PPC + individuals within reasonable time (typically 72h) |

### LGPD Art. 48 (Brazil ANPD)

| Event | Notification |
|---|---|
| Security incident affecting personal data | Notify ANPD + data subjects within "reasonable period" (ANPD: 2 business days) |

### DPDPA 2023 (India DPB)

| Event | Notification |
|---|---|
| Personal-data breach | Notify Data Protection Board within 72h |

### NIS2 (EU 2022/2555)

For oyatie when crossing Annex I/II thresholds:
- Early warning: ≤ 24h
- Incident notification: ≤ 72h
- Final report: ≤ 1 month

### DORA (EU 2022/2554) — pack-eu financial-services tenants

For financial-services tenants in pack-eu, DORA-defined ICT incident classification + reporting timelines apply on top of GDPR; reporting to financial regulator within 4h (major); cross-border-incident classification within 8h.

### KR-FSS

Notify FSS within 24h for incidents affecting financial data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days of resolution, IC convenes postmortem.
2. Scribe's timeline = starting input.
3. Document covers:
   - Summary (5 lines)
   - Timeline (chronological with timestamps)
   - Impact (tenant + internal)
   - Root cause (5-whys; cite FM-ID from `failure-modes.md`)
   - Lessons learned
   - Action items (owned + scheduled)
   - Was runbook adequate? (yes/partial/no + improvement)
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

**Blameless culture per Google SRE Workbook ch. 12**: postmortems focus on systems + processes; never on individuals.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers; ~7 weeks between rotations) | follow-the-sun |
| ops-sre-reliability secondary | weekly (same pool; offset by 1 week) | – |
| axis-workflow SME | weekly (3 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent; engaged manually for breach-suspect | always-on-call |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call comp + handoff per `runbooks/oncall-rotation.md` (cross-cuts with observability µservice's rotation; engine teams share follow-the-sun model).

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=incident-runbook-coverage --microservice workflow-engine` — exit 0; every FM-ID has a matching runbook.
- Quarterly DR failover drill validates the response chain end-to-end (per `multi-region.md`).
- Annual tabletop exercise simulates Sev-1 regional outage; comms + regulatory notification chain rehearsed.

## References

- `microservices/workflow-engine/failure-modes.md`.
- `microservices/workflow-engine/compliance.md` §"Regulatory Notifications" + per-pack overlays.
- `microservices/workflow-engine/multi-region.md`.
- `microservices/workflow-engine/runbooks/*`.
- `microservices/workflow-engine/dpia.md`.
- `microservices/workflow-engine/threat-model.md`.
- `docs/standards/incident-severity.md`.
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12-14.
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555; DORA 2022/2554.
