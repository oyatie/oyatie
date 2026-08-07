---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + privacy-governance
deciders: ops-sre-reliability, ops-security, privacy-governance, axis-cloud-iac, architecture-governance
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-iac/threat-model.md
  - microservices/cloud-iac/dpia.md
  - microservices/cloud-iac/compliance.md
  - microservices/cloud-iac/failure-modes.md
  - microservices/cloud-iac/multi-region.md
  - microservices/cloud-iac/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (cloud-iac µservice)

## Purpose

End-to-end incident-response procedure for cloud-iac events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` (classifies failures by severity) and from `compliance.md` (lists regulatory frameworks requiring notification).

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard (`docs/standards/incident-severity.md`).

| Severity | Definition | Response time (target page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier confidentiality, integrity, or availability impact affecting multiple µservices; regulatory-notification triggers; data breach; safety violation | ≤ 5min (24/7) | FM-03 registry corruption; FM-06 apply-elevation escape; FM-08 supply-chain; FM-10 cross-pack misroute |
| **Sev-2** | Single-µservice or sub-µservice impact; operational degradation without data loss; gate fail-closed (safe default applies) | ≤ 15min (24/7) | FM-01 stuck apply; FM-02 drift cascade; FM-05 reconciler down; FM-07 SLSA verify; FM-11 drift coverage gap; FM-13 audit emission |
| **Sev-3** | Localized impact; degraded but functional; backlogged operations | ≤ 1h (business hours) | FM-04 state-lock; FM-09 rollback chain > 1; FM-15 non-determinism |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident declaration → closure |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook; performs DR failover |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts and sends tenant + status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-cloud-iac + relevant workload owner | Diagnoses root cause; proposes mitigation |
| **Privacy Lead (PrivacyLead)** | privacy-governance chair | Activates for data-breach-suspect (Sev-1 confidentiality); owns regulatory notification chain |
| **Executive Sponsor (ExecSponsor)** | architecture-governance chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline in incident channel (Slack `#inc-<id>`) |

## Escalation Path

```text
Alert fires (Mimir → Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (if no ack in 5min)
Primary re-paged
    ↓ (if no ack in 10min total)
Secondary on-call paged
    ↓ (if no ack in 15min)
Engineering manager (axis-cloud-iac lead) paged + Slack
    ↓ (if Sev-1 and no resolution in 30min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (if Sev-1 and breach-suspect)
privacy-governance chair + ExecSponsor engaged
    ↓ (if confirmed breach)
Regulatory notification chain begins
    ↓ (if confirmed data-subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1 / Sev-2 alert fires BOTH a Mimir metric AND an OnCall page; on-call playbook requires both checked.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>` Slack channel; pages IC role | ≤ 5min (Sev-1) / ≤ 15min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10min |
| **4. Containment** | OpsLead executes immediate-mitigation steps from `failure-modes.md`; PrivacyLead engaged if suspect | varies; aim for RTO |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures executed; service restored | per RTO targets in `failure-modes.md` |
| **7. Communication** | CommsLead notifies tenants + regulatory | per §"Regulatory Notifications" |
| **8. Closure** | IC declares resolved; steady state ≥ 30min | – |
| **9. Postmortem** | Within 5 business days | ≤ 5 business days |
| **10. Action items** | Remediation tracked + owned + scheduled | indefinite (until done) |

## Tenant Communications

### Status page (public)

- Updated within 5min of Sev-1 / Sev-2 declaration.
- Every 30min during active incident.
- Final resolution update within 30min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / cloud-iac] Incident in <pack>: <one-line summary>

We are investigating an incident affecting <component> in <pack> that may impact
the deployment pipeline for your µservices. Started at <ISO8601>. Current status:
<Investigating | Mitigating | Resolved>. ETA: <est>.

What you may experience: <impact>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update again within 30 minutes or upon resolution.

If this impact involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev>
For questions: <support email>
```

Template — Sev-2 (operational):

```
Subject: [Sev-2 / cloud-iac] Degradation in <pack>: <one-line summary>

We are investigating a deployment-pipeline degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., delayed apply, queued promotions>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.

For real-time updates: <status.oyatie.dev>
```

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72h: notify lead DPA via online portal |
| Breach with high risk (Art. 34) | Also notify affected data subjects without undue delay |
| Late notification | Justify delay in same notification |

Template — DPA notification:

```
To: <Lead Supervisory Authority>
From: <oyatie privacy-governance chair as DPO>
Subject: Personal data breach notification under GDPR Art. 33

Date / time of breach discovery: <ISO8601>
Date / time of breach occurrence (if different): <ISO8601>
Nature of breach: <categories of personal data + categories of data subjects>
Approximate number of records affected: <est>
Likely consequences: <e.g., possible exposure via cross-µservice mutation>
Measures taken / proposed: <mitigation + DSR support>
DPO contact: <privacy-governance chair>
Joint controller cascade: tenant <tenant_id_redacted> notified at <ISO8601>;
tenant is informing its data subjects per Art. 34 where applicable.
```

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting < 500 individuals | OCR within 60d of end of calendar year |
| Breach affecting 500+ individuals | OCR within 60d + media notification + individual notification |
| Business Associate (oyatie) | Notify covered-entity tenant within BAA-specified window (typically 24h–7d) |

### KR PIPA Art. 34 (Personal Information Protection Commission)

| Event | Notification |
|---|---|
| Breach affecting 1+ data subjects | Notify affected within 72h |
| Breach affecting 1000+ OR sensitive (Art. 23) OR resident registration numbers | Notify PIPC within 72h + publish on website |

### APPI Art. 26-2 (Japan PPC)

Notify PPC + affected individuals within reasonable time (~72h of discovery).

### LGPD Art. 48 (Brazil ANPD)

Notify ANPD + data subjects within "reasonable period" (ANPD guidance: 2 business days).

### DPDPA 2023 (India DPB)

Notify Data Protection Board within 72h of awareness.

### NIS2 (EU 2022/2555)

When oyatie crosses Annex I/II thresholds:
- Early warning: ≤ 24h of awareness.
- Incident notification: ≤ 72h.
- Final report: ≤ 1mo.

### KR-FSS (financial-services KR tenants)

Notify FSS within 24h for incidents affecting financial-data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days of resolution, IC convenes postmortem.
2. Scribe's timeline is the starting input.
3. Postmortem covers:
   - Summary (5 lines)
   - Timeline (chronological events with timestamps)
   - Impact (tenant-facing + internal-facing)
   - Root cause (5-whys; cite the FM-ID from `failure-modes.md`)
   - Lessons learned
   - Action items (each owned + scheduled)
   - Runbook adequacy? (yes / partial / no + improvement)
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by architecture-governance for systemic patterns.

**Blameless culture per Google SRE Workbook ch. 12.**

## On-Call Rotation

Inherits observability on-call structure; cloud-iac adds a dedicated axis-cloud-iac SME rotation:

| Tier | Pool | Rotation | Cadence |
|---|---|---|---|
| ops-sre-reliability primary | 6 engineers | Weekly; follow-the-sun (KR / EU / US) | On-call pay per company policy |
| ops-sre-reliability secondary | Same pool offset 1 week | Weekly | Same |
| axis-cloud-iac SME | 3 engineers | Weekly; KR + EU primary; US business-hours | Same |
| ops-security on-call | 4 engineers | Weekly; 24/7 for Sev-1 confidentiality | Same |
| privacy-governance chair | Named role; permanent | Always-on-call for breach-suspect | – |
| Executive Sponsor | Named role; permanent | Sev-1 only | – |

## Verification

- cloud-ci/oya-ci governance gate `incident-runbook-coverage` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context — exit 0; every FM-ID has matching runbook.
- Quarterly DR-failover drill validates response chain end-to-end (per `multi-region.md`).
- Annual tabletop exercise simulates Sev-1 regional outage; comms + regulatory notification chain rehearsed.

## References

- `microservices/cloud-iac/failure-modes.md`.
- `microservices/cloud-iac/compliance.md` §"Regulatory Notifications".
- `microservices/cloud-iac/multi-region.md`.
- `microservices/cloud-iac/runbooks/*`.
- `microservices/cloud-iac/dpia.md`.
- `microservices/cloud-iac/threat-model.md`.
- `microservices/observability/incident-response.md` (parent template).
- `docs/standards/incident-severity.md`.
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14.
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023; NIS2 2022/2555.
