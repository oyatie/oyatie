---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-foundry-runtime, council-architecture
related_adrs: [ADR-0025, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-runtime/threat-model.md
  - microservices/foundry-runtime/dpia.md
  - microservices/foundry-runtime/compliance.md
  - microservices/foundry-runtime/failure-modes.md
  - microservices/foundry-runtime/multi-region.md
  - microservices/foundry-runtime/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (foundry-runtime µservice)

## Purpose

End-to-end incident-response procedure for foundry-runtime events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` (classifies failures by severity) and `compliance.md` (lists regulatory frameworks).

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard.

| Severity | Definition | Response time (page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier confidentiality, integrity, or availability impact affecting multiple tenants; regulatory-notification triggers; data breach; safety violation; autonomy bypass | ≤5min (24/7) | FM-03 Redis ACL drift; FM-05 descriptor signature invalid (tampering); FM-07 provider credential leak; FM-12 cross-tenant leak |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; gate fail-closed; security signal | ≤15min (24/7) | FM-01 pod crashloop; FM-02 Redis partition; FM-04 registry cache stale; FM-06 autonomy violation surge; FM-08 sibling unreachable; FM-11 pod drain failure; FM-13 prompt-injection contamination |
| **Sev-3** | Localized impact; degraded but functional | ≤1h (business hours) | FM-09 Postgres replica fail; FM-10 cold-restore latency; FM-14 capacity exhaustion; FM-15 long invocation |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident; coordinates roles |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook steps; performs DR failover |
| **Communications Lead (CommsLead)** | gtm-customer-success | Drafts + sends tenant + public status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-foundry-runtime + relevant sibling owner | Diagnoses root cause; proposes mitigation |
| **Privacy Lead (PrivacyLead)** | council-privacy chair | Activates for any data-breach-suspect incident (Sev-1 confidentiality); owns regulatory notification |
| **Executive Sponsor (ExecSponsor)** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in Slack `#inc-<id>` |

## Escalation Path

```text
Alert fires (runtime self-SLI / Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (no ack in 5min)
Primary re-paged
    ↓ (no ack in 10min total)
Secondary on-call paged
    ↓ (no ack in 15min)
Engineering manager (axis-foundry-runtime lead) paged + Slack alert
    ↓ (Sev-1 and no resolution in 30min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (Sev-1 + breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (confirmed breach)
Regulatory notification chain begins (see §"Regulatory Notifications")
    ↓ (confirmed data subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1 / Sev-2 alert fires BOTH a self-emitted metric AND an OnCall page.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| 1. Detection | Alert fires; metric + page both received | ≤60s alert-to-page p99 |
| 2. Acknowledgement | Primary on-call ack; opens `#inc-<id>` Slack; pages IC | ≤5min (Sev-1) / ≤15min (Sev-2) |
| 3. Triage | IC declares severity; assigns roles; starts timeline | ≤10min |
| 4. Containment | OpsLead executes immediate-mitigation steps from `failure-modes.md`; Privacy Lead engaged if suspect | per RTO |
| 5. Diagnosis | SME identifies root cause | varies |
| 6. Mitigation / Resolution | Runbook procedures executed; service restored | per RTO |
| 7. Communication | CommsLead notifies tenants; regulatory notification per pack timelines | per §"Regulatory Notifications" |
| 8. Closure | IC declares resolved; service in steady state for ≥30min | – |
| 9. Postmortem | Within 5 business days | ≤5 business days |
| 10. Action items | Postmortem remediation items tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page (public)

- Updated within 5min of Sev-1 / Sev-2 declaration.
- Updated every 30min during active incident.
- Final resolution update within 30min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email — Sev-1 (data-affecting)

```
Subject: [Sev-1 / foundry-runtime] Incident in <pack>: <one-line summary>

We are investigating an incident affecting <component> in <pack> that may impact
your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.
ETA to resolution: <est>.

What you may experience: <impact>
What we're doing: <action>
What you should do: <usually nothing>

We will update you again within 30 minutes or upon resolution, whichever is sooner.
If this impact involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev>
For questions: <support email>
```

### Tenant operator email — Sev-2 (operational, no data impact)

```
Subject: [Sev-2 / foundry-runtime] Degradation in <pack>: <one-line summary>

We are investigating a service degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated dispatch latency, capability invocation 429>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.
```

### Customer-facing message template

Pre-localized per pack at tenant onboarding portal. Tenants retain editorial control.

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72h: notify lead DPA via portal. |
| Breach with high risk (Art. 34) | Also notify affected data subjects without undue delay. |
| Late notification | Justify in same notification. |

Template — DPA notification:

```
To: <Lead Supervisory Authority — Irish DPC for many global SaaS, or tenant's local DPA>
From: <oyatie council-privacy chair as DPO>
Subject: Personal data breach notification under GDPR Art. 33

Date / time of breach discovery: <ISO8601>
Date / time of breach occurrence (if different): <ISO8601>
Nature of breach: <data classes + data subjects>
Approximate records affected: <est>
Likely consequences: <e.g., possible session content correlation>
Measures taken / proposed: <mitigation + DSR support>
DPO contact: <council-privacy chair>
Joint controller cascade: tenant <tenant_id_redacted> notified at <ISO8601>.
```

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI <500 individuals | OCR within 60d of calendar-year end. |
| Breach affecting 500+ | OCR within 60d + media (§164.406) + individual (§164.404). |
| Business Associate (oyatie) | Notify covered-entity tenant within BAA window (24h–7d). |

### KR PIPA Art. 34 (Personal Information Protection Commission)

| Event | Notification |
|---|---|
| Breach affecting 1+ subjects | Notify subjects within 72h. |
| Breach affecting 1000+ subjects OR sensitive data (Art. 23) OR RRN | Notify PIPC within 72h + publish on website. |

### APPI Art. 26-2 (Japan PPC)

| Event | Notification |
|---|---|
| Leakage of PII affecting 1+ persons | Notify PPC + affected individuals within 72h. |

### LGPD Art. 48 (Brazil ANPD)

| Event | Notification |
|---|---|
| Security incident affecting PII | Notify ANPD + subjects within 2 business days (ANPD guidance). |

### DPDPA 2023 (India DPB)

| Event | Notification |
|---|---|
| Personal-data breach | Notify DPB within 72h of awareness. |

### PDPA (Singapore) / Privacy Act 1988 (Australia) / UAE PDPL / KSA PDPL

Per-pack timelines in `regional-packs/<pack>/foundry-runtime-incident-notification-overlay.md`. Universal target: notify-subjects-and-supervisor within 72h.

### NIS2 (EU 2022/2555)

When oyatie crosses Annex I/II thresholds:
- Early warning: ≤24h.
- Incident notification: ≤72h.
- Final report: ≤1mo.

### EU AI Act Art. 73 (Serious incidents)

For high-risk Annex III capabilities: serious-incident notification to market surveillance authority within 15 days; immediate notification (within 2 days) if death or serious harm.

### KR-FSS (financial-services KR tenants)

Notify FSS within 24h for incidents affecting financial data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days, IC convenes postmortem.
2. Scribe's timeline = starting input.
3. Postmortem covers: Summary; Timeline; Impact; Root cause (5-whys; cite FM-ID); Lessons learned; Action items (owned + scheduled); Runbook adequacy; Trust-portal entry.
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

**Blameless culture per Google SRE Workbook ch. 12**: focus on systems + processes, never individuals.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers) | follow-the-sun (KR / EU / US) |
| ops-sre-reliability secondary | weekly (same pool; +1 offset) | – |
| axis-foundry-runtime SME | weekly (4 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent | always-on for breach-suspect |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff per `runbooks/oncall-rotation.md` (cross-references observability µservice's analogous runbook).

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice foundry-runtime` — exit 0; every FM-ID has matching runbook.
- Quarterly DR failover drill.
- Annual tabletop exercise.

## References

- `microservices/foundry-runtime/failure-modes.md` (FM-IDs + severity).
- `microservices/foundry-runtime/compliance.md` §"Regulatory Notifications" (per-pack timelines).
- `microservices/foundry-runtime/multi-region.md` (DR failover).
- `microservices/foundry-runtime/runbooks/*`.
- `microservices/foundry-runtime/dpia.md` (data-subject impact assessment).
- `microservices/foundry-runtime/threat-model.md`.
- `docs/standards/incident-severity.md`; `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14.
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555; EU AI Act Art. 73.
