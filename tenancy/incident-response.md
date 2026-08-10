---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-tenancy, council-architecture
related_adrs: [ADR-0018, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - tenancy/threat-model.md
  - tenancy/dpia.md
  - tenancy/compliance.md
  - tenancy/failure-modes.md
  - tenancy/multi-region.md
  - tenancy/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (tenancy µservice)

## Purpose

End-to-end incident-response procedure for tenancy events. **Tenancy is the highest-severity µservice in oyatie:** an isolation breach here cascades to every tenant simultaneously. Severity defaults are tighter than other µservices.

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard (`docs/standards/incident-severity.md`), with tenancy-specific tightenings.

| Severity | Definition | Response time (target page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | RLS bypass / JWT-key compromise / cross-tenant data exposure / catastrophic isolation failure / multi-pack outage / regulatory-notification trigger | **≤ 3 min** (tighter than the catalog-default 5min; tenancy availability is 99.99%) | FM-02 RLS drift; FM-04 JWT compromise; FM-09 pack misroute; FM-12 DCS outage |
| **Sev-2** | Single-component outage with auto-recovery in progress; DSR cascade SLA risk; Patroni / Citus auto-failover | ≤ 15 min | FM-01 Postgres primary failover; FM-03 Citus coordinator outage; FM-06 DSR incomplete; FM-08 rebalance hung; FM-11 OpenBao outage |
| **Sev-3** | Localized impact; degraded but functional; backlogged operations | ≤ 1 h | FM-05 stuck activation; FM-07 Valkey outage; FM-10 propagation lag; FM-14 audit-seal latency; FM-15 validate overload |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident from declaration to closure |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook steps; performs Patroni failover; performs Citus rebalance abort if needed |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts tenant + public status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-tenancy | Diagnoses root cause; proposes mitigation |
| **Privacy Lead (PrivacyLead)** | council-privacy chair | Activates for any data-breach-suspect incident (Sev-1 confidentiality); owns regulatory notification chain |
| **Database Lead (DBALead)** | axis-tenancy + ops-sre-reliability JIT-elevated | Performs RLS / Postgres / Citus interventions when needed (2-person rule for elevated actions) |
| **Executive Sponsor (ExecSponsor)** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in incident channel |

## Escalation Path

```text
Alert fires (RLS drift validator / Patroni / OpenBao / observability gate)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (if no ack in 3min for Sev-1; 5min for Sev-2)
Primary on-call re-paged + secondary on-call paged
    ↓ (if no ack in 5min total)
Engineering manager (axis-tenancy lead) paged + Slack alert
    ↓ (if Sev-1 and no resolution in 15min)
Director (ops-sre-reliability + ops-security directors) engaged
    ↓ (if Sev-1 and isolation-breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (if confirmed isolation breach OR mass-deletion event)
Regulatory notification chain begins (see §"Regulatory Notifications")
    ↓ (if confirmed data-subject impact + GDPR-scope)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: every Sev-1 / Sev-2 alert fires BOTH a Mimir metric (`oya_incident_active{microservice="tenancy",severity="N"}`) AND an OnCall page. If one channel silent, the other still fires.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>` Slack; pages IC | ≤ 3min (Sev-1) / ≤ 15min (Sev-2) |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10min |
| **4. Containment** | OpsLead executes immediate-mitigation from `failure-modes.md`; PrivacyLead engaged if isolation-breach-suspect | varies; aim for RTO stabilisation |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures executed | per RTO targets in `failure-modes.md` |
| **7. Communication** | CommsLead notifies tenants + regulators per per-pack timelines | per §"Regulatory Notifications" |
| **8. Closure** | IC declares incident resolved; service in steady state for ≥ 30min | – |
| **9. Postmortem** | Within 5 business days; published to ops-sre-reliability + council-architecture + auditors | ≤ 5 business days |
| **10. Action items** | Postmortem-generated remediation items tracked + owned + scheduled | indefinite (until done) |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1 / Sev-2 declaration.
- Updated every 30 min during active incident.
- Final resolution update within 30 min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email

Template — Sev-1 (data-affecting):

```
Subject: [Sev-1 / tenancy] Incident in <pack>: <one-line summary>

We are investigating an incident affecting tenancy substrate in <pack> that may impact
your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.
ETA to resolution: <est>.

What you may experience: <impact, e.g., temporary inability to authenticate / receive new JWTs>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update you again within 30 minutes or upon resolution, whichever is sooner.
If this impact involves your tenant's data, we will follow with a separate
breach-notification email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev link>
For questions: <support email>

Your tenant onboarding contact: <name>
```

Template — Sev-2 (operational; auto-recovery in progress):

```
Subject: [Sev-2 / tenancy] Degradation in <pack>: <one-line summary>

We are managing a service degradation in <pack> affecting tenancy substrate.
Started at <ISO8601>. Status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated tenant-validate latency, delayed
                          tenant activation, momentary auth-token re-issuance pause>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.

For real-time updates: <status.oyatie.dev link>
```

### Customer-facing message template (tenant forwards to its end-users)

Provided at the tenant onboarding portal; pre-localised per pack. Tenants retain editorial control.

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority, 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants (e.g., RLS bypass exposed EU tenant rows) | Within 72 hours: notify lead DPA (per tenant's establishment). |
| Breach with high risk to data subjects (Art. 34) | Also notify affected data subjects without undue delay. |
| Late notification | Justify the delay in the same notification. |

Template — DPA notification:

```
To: <Lead Supervisory Authority>
From: <oyatie council-privacy chair as DPO>
Subject: Personal data breach notification under GDPR Art. 33

Date / time of breach discovery: <ISO8601>
Date / time of breach occurrence (if different): <ISO8601>
Nature of breach: <RLS bypass | JWT-key compromise | cross-pack misroute | other>
Categories of personal data + categories of data subjects affected: <details>
Approximate number of records affected: <est>
Likely consequences: <e.g., possible cross-tenant exposure of operational metadata>
Measures taken / proposed: <RLS restored from declarative; signing-key rotated; DSR cascade
                            initiated for affected tenants if appropriate>
DPO contact: <council-privacy chair>
Joint controller cascade: tenant <tenant_id_redacted> notified at <ISO8601>;
                         tenant is informing its data subjects per Art. 34 where applicable.
```

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting fewer than 500 individuals | OCR notification within 60d of end of calendar year. |
| Breach affecting 500+ individuals | OCR within 60d + media notification (§164.406) + individual notification (§164.404). |
| Business Associate (oyatie) | Notify covered-entity tenant within the BAA-specified window (typically 24h to 7d). |

### KR PIPA Art. 34 (Personal Information Protection Commission)

| Event | Notification |
|---|---|
| Breach affecting 1+ data subjects | Notify affected data subjects within 72 hours. |
| Breach affecting 1000+ data subjects OR sensitive data (Art. 23) | Notify PIPC within 72 hours + publish on website. |

### APPI Art. 26-2 (Japan PPC)

72h notification to PPC + affected individuals.

### LGPD Art. 48 (Brazil ANPD)

2 business days notification to ANPD + data subjects (ANPD guidance).

### DPDPA 2023 (India DPB)

72h notification to DPB.

### PDPA (Singapore PDPC, Australia OAIC, etc.)

Per-pack timelines in `regional-packs/<pack>/incident-notification-overlay.md`.

### NIS2 (EU 2022/2555)

When oyatie crosses Annex I/II thresholds:
- Early warning: ≤ 24 h of awareness.
- Incident notification: ≤ 72 h.
- Final report: ≤ 1 month.

### KR-FSS (financial-services KR tenants)

24-hour notification for incidents affecting financial data integrity / availability.

### DORA (EU financial-services 2022/2554)

Major ICT-related incident reporting timelines (per ESAs guidance).

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days of resolution, IC convenes postmortem meeting.
2. Scribe's timeline is starting input.
3. Document covers:
   - Summary (5 lines)
   - Timeline (chronological events)
   - Impact (tenant-facing + internal-facing; per-tenant count if isolation-impactful)
   - Root cause (5-whys; cite FM-ID from `failure-modes.md`)
   - Lessons learned
   - Action items (each owned + scheduled)
   - Was the runbook adequate? (yes / partial / no)
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

**Blameless culture per Google SRE Workbook ch. 12**: postmortems focus on systems + processes.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers) | follow-the-sun: KR / EU / US shifts |
| ops-sre-reliability secondary | weekly (offset 1 week) | – |
| axis-tenancy SME | weekly (4 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 isolation breach | 24/7 |
| DBA on-call (JIT-elevated) | named role; permanent; engaged manually for Sev-1/2 DB-write incidents | – |
| council-privacy chair | named role; permanent | always-on-call for breach-suspect |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff per `tenancy/runbooks/jwt-key-rotation.md` §"On-call notes" (rotation-related).

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice tenancy` — exit 0; every FM-ID has matching runbook.
- Quarterly DR failover drill validates response chain end-to-end (per `multi-region.md`).
- Annual tabletop exercise simulates Sev-1 RLS-bypass incident; comms + regulatory notification chain rehearsed.

## References

- `tenancy/failure-modes.md` (FM-IDs + severity classification).
- `tenancy/compliance.md` §"Regulatory Notifications" (per-pack timelines).
- `tenancy/multi-region.md` (DR failover).
- `tenancy/runbooks/*` (per-scenario procedures).
- `tenancy/dpia.md` (data-subject impact assessment).
- `tenancy/threat-model.md` (security-incident threat IDs).
- `docs/standards/incident-severity.md`.
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14 (Postmortem; managing incidents; on-call).
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555; DORA 2022/2554.
