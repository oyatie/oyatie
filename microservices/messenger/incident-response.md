---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-messenger, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/messenger/threat-model.md
  - microservices/messenger/dpia.md
  - microservices/messenger/compliance.md
  - microservices/messenger/failure-modes.md
  - microservices/messenger/multi-region.md
  - microservices/messenger/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (messenger µservice)

## Purpose

End-to-end incident-response procedure for messenger events. Covers severity classification, response roles, escalation, communication, postmortem cadence, and per-pack regulatory-notification timelines. **Cross-channel leak is automatically Sev-1.**

## Severity Definitions

| Severity | Definition | Page-to-ack target | Examples |
|---|---|---|---|
| **Sev-1** | Cross-tenant or cross-context confidentiality / integrity / availability impact; regulatory notification triggers; data breach; safety violation | ≤ 5 min (24/7) | FM-05 channel ACL drift; FM-07 cross-tenant leak; FM-10 cross-context violation; FM-12 four-eyes violation; FM-13 pack misroute; FM-14 personal-DM admin decrypt attempt |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; service unavailable for multi-tenant subset | ≤ 15 min (24/7) | FM-01 gateway storm; FM-02 Postgres primary outage; FM-03 Redis corruption; FM-04 attachment-store outage; FM-09 malware detection (organised pattern) |
| **Sev-3** | Localised impact; degraded but functional | ≤ 1h (business hours) | FM-06 search lag; FM-08 mention storm; FM-11 read-receipt storm; FM-15 capacity exhaustion; FM-16 ontology degraded |
| **Sev-4** | Cosmetic; tracked but not paged | next business day | dashboard label typos; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security/privacy incidents) | Owns incident lifecycle |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps |
| **Communications Lead** | gtm-customer-success | Drafts tenant + public-status + regulatory notifications |
| **SME** | axis-messenger + relevant BC owner | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for any data-breach-suspect (Sev-1 confidentiality); owns regulatory chain |
| **Executive Sponsor** | council-architecture chair (Sev-1) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline in `#inc-<id>` Slack |

## Escalation Path

```text
Alert fires (observability → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (no ack in 5 min → re-page)
    ↓ (no ack in 10 min → secondary)
    ↓ (no ack in 15 min → engineering manager + Slack)
    ↓ (Sev-1 + no resolution in 30 min → directors)
    ↓ (Sev-1 + breach-suspect → council-privacy + ExecSponsor)
    ↓ (confirmed breach → regulatory notification chain)
    ↓ (GDPR-scope + data subject impact → 72h Art. 33 clock starts)
```

## Cross-Channel Leak = Sev-1

Any confirmed cross-channel data exposure — including cross-tenant, cross-context (parallel ADR-0135), cross-pack residency violation, or unauthorised channel membership — is **automatically Sev-1** regardless of scope. Triggers:

- IC immediately engages PrivacyLead + ops-security.
- Channel is quarantined (writes blocked; reads only by ops-security under JIT elevation).
- Audit-chain replay of affected channel ACL history begins.
- GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 clocks may start; CommsLead drafts notifications.
- Sev-1 postmortem within 5 business days; council-privacy + ops-security sign-off.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| 1. Detection | Alert + metric both received | ≤ 60s alert-to-page p99 |
| 2. Acknowledgement | On-call ack; open `#inc-<id>`; page IC | ≤ 5 min (Sev-1) / ≤ 15 min (Sev-2) |
| 3. Containment | Apply immediate mitigation per runbook | ≤ 15 min Sev-1 / ≤ 30 min Sev-2 |
| 4. Communication | Tenant + status page notifications | ≤ 30 min Sev-1 |
| 5. Eradication | Root-cause + fix | hours |
| 6. Recovery | Verify full service | within RTO |
| 7. Postmortem | Blameless review with action items | 5 business days |

## Two-Channel Corroboration

Every Sev-1 / Sev-2 alert fires BOTH a Prometheus metric (`messenger_incident_active{severity="N"}` with incident-id label) AND an OnCall page. If one is silent, the other still fires; on-call playbook requires both checks.

## Regulatory Notifications

| Pack / framework | Trigger | Window | Recipient |
|---|---|---|---|
| GDPR Art. 33 | Personal-data breach affecting EU subjects | 72h initial + supplementary as known | Lead supervisory authority + affected subjects (Art. 34) |
| KR PIPA Art. 34 | Personal-data leakage | "without delay" (interpreted: ≤ 24h) | KR PIPC + affected subjects |
| HIPAA §164.412 (Breach Notification Rule) | PHI breach affecting ≥ 500 individuals | 60d notification + HHS reporting | HHS + media + affected individuals |
| HIPAA §164.404 (individual notification) | PHI breach (any size) | 60d | Affected individuals |
| APPI Art. 22-2 | PII leakage at scale | "promptly" (≤ 5 business days) | JP PPC + affected subjects |
| PDPA SG | Significant breach | 72h | PDPC |
| LGPD Art. 48 | Breach with risk to subjects | "reasonable time" | ANPD + affected subjects |
| NIS2 (when engaged) | Significant incident | 24h initial + 72h detailed + 1mo final | National CSIRT |
| UAE PDPL | Breach | 72h | UAE Data Office |
| KSA PDPL | Breach | 72h | SDAIA |

PrivacyLead owns the regulatory chain; ops-security supports forensic-trace; CommsLead drafts notification text from `legal/breach-notification-templates.md` (Slice D).

## Communication Templates

(Templates in `legal/breach-notification-templates.md`; here are the placeholders.)

### Sev-1 tenant notification

```
Subject: [Service Incident] messenger Sev-1 incident-<id> in pack <pack>

We detected an incident affecting your tenant's messenger service starting at <UTC-ts>.
Scope: <scope>
Status: <status>
Workaround: <workaround>

Updates will follow at <status-page-url>.
```

### GDPR Art. 34 affected-subject notification

```
Subject: Notification of personal data incident

A personal data incident has been identified affecting your information.
Date detected: <ts>
Nature: <nature>
Likely consequences: <consequences>
Measures taken: <measures>
Contact: <DPO-contact>
```

## Postmortem Process

| Step | Owner | Deadline |
|---|---|---|
| Schedule blameless postmortem meeting | IC | within 2 business days of incident close |
| Draft postmortem document | IC + SME | within 5 business days |
| Action items prioritised + assigned | All attendees | meeting end |
| Tracker created | IC | meeting end |
| Action items closed | Assignees | per priority |
| Re-review at 30/60/90 days | IC | rolling |

Postmortem template: `microservices/messenger/runbooks/postmortem-template.md` (Slice D).

## On-Call Rotation

| Rotation | Cadence | Owner |
|---|---|---|
| ops-sre-reliability primary | 1-week | ops-sre-reliability lead |
| ops-sre-reliability secondary | 1-week | ops-sre-reliability lead |
| ops-security primary (security/privacy incidents) | 1-week | ops-security lead |
| axis-messenger SME | 1-week | axis-messenger lead |
| Privacy escalation | on-call council-privacy chair (or designate) | council-privacy |

Per-pack on-call: each active pack carries a separate primary + secondary rotation in the pack's local timezone with global escalation handoff.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Sev-1 cross-context violation tabletop | Quarterly | 2026-Q2 (scheduled) |
| WebSocket gateway storm chaos | Quarterly | 2026-Q2 (scheduled) |
| Postgres primary failover | Quarterly | 2026-Q2 (scheduled) |
| Pack-wide DR failover | Annually (DR-pair packs) | 2026-Q3 (scheduled) |
| Regulatory-notification rehearsal | Annually | 2026-Q3 (scheduled) |

## References

- `microservices/messenger/threat-model.md`.
- `microservices/messenger/dpia.md`.
- `microservices/messenger/compliance.md`.
- `microservices/messenger/failure-modes.md`.
- `microservices/messenger/multi-region.md`.
- `microservices/messenger/runbooks/`.
- `microservices/observability/incident-response.md` (shape reference).
- Bominal ADR-0028.
- GDPR Arts. 33, 34; KR PIPA Art. 34; HIPAA §164.404, §164.412.
