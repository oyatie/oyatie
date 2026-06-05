---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-foundry-control-plane, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-supervisor/threat-model.md
  - microservices/intelligence-supervisor/dpia.md
  - microservices/intelligence-supervisor/compliance.md
  - microservices/intelligence-supervisor/failure-modes.md
  - microservices/intelligence-supervisor/multi-region.md
  - microservices/intelligence-supervisor/runbooks/
review_cadence: quarterly + after every Sev-1/2
doc_status: published
---

# Incident Response Playbook (foundry-supervisor µservice)

## Purpose

End-to-end incident-response procedure for foundry-supervisor events. Covers severity classification, response roles, escalation paths, communications, regulatory notifications, postmortem cadence.

**Critical rule per ADR-0133 hyperscaler-safety claim:** supervisor-down = **Sev-1 always**. Per the threat-model + DPIA, the supervisor is the safety-critical surface; any outage of the kill-switch engage path or autonomy-precondition path is Sev-1 regardless of tenant blast radius. Sev-2 reserved for degraded-but-functioning (e.g., dashboard latency, supervision-bus backlog).

## Severity Definitions

Per Bominal ADR-0028 (inherited) and oyatie incident-severity standard (`docs/standards/incident-severity.md`).

| Severity | Definition | Response time (page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Supervisor control-plane unavailable; kill-switch latency breach; autonomy-policy fail-open; cross-pack misroute; cross-tenant leak; **any supervisor-down event** | ≤ 5 min (24/7) | FM-01 kill-switch latency; FM-13 cross-pack misroute; T-I-01 cross-tenant leak |
| **Sev-2** | Degraded but functioning; gate fail-closed safe default applies | ≤ 15 min (24/7) | FM-02 deployment stuck; FM-04 Valkey failover; FM-05 Postgres master loss; FM-06 Operator crashloop; FM-08 supervision-bus backlog |
| **Sev-3** | Localized + degraded; not blocking | ≤ 1 h (business hrs) | FM-09 Cedar latency; FM-12 schema-violation flood |
| **Sev-4** | Cosmetic; not paged | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident from declaration to closure |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps; DR failover if needed |
| **Communications Lead** | gtm-customer-success or designated | Tenant + status-page + regulatory notifications |
| **SME** | axis-foundry-control-plane | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for any data-breach-suspect incident; owns regulatory chain |
| **Executive Sponsor** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline in `#inc-<id>` Slack |

## Escalation Path

```text
Alert fires (Mimir/Alertmanager → OnCall)
    ↓
Primary on-call paged
    ↓ (if no ack in 5 min)
Primary re-paged
    ↓ (if no ack in 10 min)
Secondary on-call paged (ops-sre-reliability secondary)
    ↓ (if no ack in 15 min)
Engineering manager (axis-foundry-control-plane lead) paged + Slack alert
    ↓ (Sev-1 + no resolution in 30 min)
Director + ops-security director engaged
    ↓ (Sev-1 + breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (confirmed breach)
Regulatory notification chain begins (see §"Regulatory Notifications")
    ↓ (data subject impact + GDPR-scope)
72h clock starts (GDPR Art. 33)
    ↓ (high-risk Annex III tenant impact)
EU AI Act Art. 73 serious-incident report to EU AI Office
```

Two-channel corroboration: every Sev-1/2 fires Mimir metric `oya_supervisor_incident_active{severity="N"}` AND OnCall page. If one channel silent, the other still fires.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| 1. Detection | Alert fires; metric + page received | ≤ 60s alert-to-page p99 |
| 2. Acknowledgement | Primary on-call ack; opens `#inc-<id>` Slack; pages IC | ≤ 5 min (Sev-1) / ≤ 15 min (Sev-2) |
| 3. Triage | IC declares severity; assigns roles; starts timeline | ≤ 10 min |
| 4. Containment | OpsLead executes immediate-mitigation; PrivacyLead engaged if suspect | varies; RTO per failure-modes.md |
| 5. Diagnosis | SME root-causes | varies |
| 6. Mitigation / Resolution | Runbook procedures | per RTO |
| 7. Communication | Tenant + status-page + regulatory notifications | per §"Regulatory Notifications" |
| 8. Closure | IC declares resolved; steady state ≥ 30 min | – |
| 9. Postmortem | Within 5 business days | – |
| 10. Action items | Tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page (public)

- Updated within 5 min of Sev-1/2 declaration.
- Updated every 30 min during active incident.
- Final resolution within 30 min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email — Sev-1 (data-affecting)

```
Subject: [Sev-1 / foundry-supervisor] Incident in <pack>: <one-line summary>

We are investigating an incident affecting the agent supervision control plane in
<pack> that may impact your tenant. Started at <ISO8601>.

What you may experience: <impact, e.g., deployment held; kill-switch latency
elevated; autonomy evaluations slow>
What we're doing: <action>
What you should do: <usually nothing; we will engage kill-switch if needed>

We will update you within 30 minutes or upon resolution. If your data is involved,
we will follow with a separate breach-notification email per your DPA within 72h.

For real-time updates: <status.oyatie.dev>
For questions: <support email>
```

### Tenant operator email — Sev-2 (operational; no data impact)

```
Subject: [Sev-2 / foundry-supervisor] Degradation in <pack>: <one-line summary>

We are investigating service degradation in <pack> affecting <component>.
Started at <ISO8601>.

What you may experience: <e.g., dashboard latency, delayed deployment>
What we're doing: <action>

This incident is not affecting your tenant data. We will update at resolution.

For real-time updates: <status.oyatie.dev>
```

## Regulatory Notifications

### GDPR Art. 33 (EU Supervisory Authority; 72-hour clock from awareness)

| Event | Notification |
|---|---|
| Personal-data breach affecting EU-resident tenants | ≤ 72 h notify lead DPA |
| High risk to data subjects (Art. 34) | also notify subjects |

### EU AI Act Art. 73 (EU AI Office; serious-incident reporting)

| Event | Timeline |
|---|---|
| Serious incident affecting high-risk Annex III tenant capability (e.g., supervisor-down + tenant impact + Annex III sub-domain) | ≤ 15 days; widespread harm or fundamental-rights infringement ≤ 2 days; data breach ≤ 72h (alignment with GDPR) |

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI < 500 individuals | OCR ≤ 60 d after end of calendar year |
| 500+ individuals | OCR ≤ 60 d + media + individual notification |
| Business Associate (oyatie) | Notify covered-entity tenant within BAA window (24h–7d) |

### KR PIPA Art. 34 (PIPC)

| Event | Notification |
|---|---|
| Breach 1+ data subjects | Notify subjects ≤ 72 h |
| Breach 1000+ subjects OR sensitive (Art. 23) OR resident-registration-numbers | Notify PIPC ≤ 72 h + publish |

### APPI Art. 26-2 (Japan PPC)

| Event | Notification |
|---|---|
| Leakage of personal information 1+ persons | Notify PPC + individuals ≤ 72 h reasonable time |

### LGPD Art. 48 (Brazil ANPD)

| Event | Notification |
|---|---|
| Security incident affecting personal data | ANPD + subjects within 2 business days |

### DPDPA 2023 (India DPB)

| Event | Notification |
|---|---|
| Personal-data breach | DPB ≤ 72 h |

### PDPA (Singapore + Australia + others)

Per-pack timelines in `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-supervisor-incident-notification-overlay.md`.

### NIS2 (EU 2022/2555)

When oyatie crosses Annex I/II thresholds:
- Early warning ≤ 24 h
- Incident notification ≤ 72 h
- Final report ≤ 1 month

### KR-FSS (financial-services KR tenants)

≤ 24 h notification on incidents affecting financial-data integrity / availability.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:
1. Within 5 business days, IC convenes postmortem.
2. Scribe's timeline is starting input.
3. Document covers: Summary (5 lines) / Timeline / Impact (tenant + internal) / Root cause (5-whys; cite FM-ID) / Lessons learned / Action items (owned + scheduled) / Was runbook adequate? / Trust-portal entry.
4. Published `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture.

**Blameless culture per Google SRE Workbook ch. 12:** focus on systems + processes, never individuals.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers; ~7 weeks between) | follow-the-sun (KR / EU / US shifts) |
| ops-sre-reliability secondary | same pool, offset 1 week | – |
| axis-foundry-control-plane SME | weekly (3 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent | always-on-call for breach-suspect |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff per `runbooks/oncall-rotation.md` (cross-references observability's runbook).

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice foundry-supervisor` — exit 0; every FM-ID has a matching runbook.
- Quarterly DR-failover + EU AI Act post-market-monitoring drills.
- Annual tabletop exercise.

## References

- `microservices/intelligence-supervisor/failure-modes.md`.
- `microservices/intelligence-supervisor/compliance.md`.
- `microservices/intelligence-supervisor/multi-region.md`.
- `microservices/intelligence-supervisor/runbooks/*`.
- `microservices/intelligence-supervisor/dpia.md`.
- `microservices/intelligence-supervisor/threat-model.md`.
- `docs/standards/incident-severity.md`.
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain).
- Google SRE Workbook ch. 12–14.
- GDPR Arts. 33+34; EU AI Act Arts. 60+73; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555.
