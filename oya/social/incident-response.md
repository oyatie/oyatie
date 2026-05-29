---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-social, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/social/threat-model.md
  - microservices/social/dpia.md
  - microservices/social/compliance.md
  - microservices/social/failure-modes.md
  - microservices/social/multi-region.md
  - microservices/social/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (social µservice)

## Purpose

End-to-end incident-response procedure for social events. Covers severity classification, response roles, escalation, communication, postmortem cadence, and per-pack regulatory-notification timelines. **Cross-tenant or cross-context leak is automatically Sev-1.**

## Severity Definitions

| Severity | Definition | Page-to-ack target | Examples |
|---|---|---|---|
| **Sev-1** | Cross-tenant or cross-context confidentiality / integrity / availability impact; regulatory notification triggers; data breach; safety violation | ≤ 5 min (24/7) | FM-05 follow-graph corruption; FM-07 cross-tenant leak; FM-10 cross-context violation; FM-12 four-eyes violation; FM-13 pack misroute; FM-14 personal-tier-federation leak; FM-15 minor-list pivot |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; service unavailable for multi-tenant subset | ≤ 15 min (24/7) | FM-01 feed-render storm; FM-02 Postgres primary outage; FM-03 Valkey corruption; FM-04 media-store outage; FM-09 media-malware detected (pattern); FM-16 moderation classifier drift |
| **Sev-3** | Localised impact; degraded but functional | ≤ 1h (business hours) | FM-06 search lag; FM-08 mention storm; FM-11 notification fanout backlog; FM-17 trending poisoning; FM-18 capacity exhaustion; FM-19 ontology degraded |
| **Sev-4** | Cosmetic; tracked but not paged | next business day | dashboard label typos; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security/privacy incidents) | Owns incident lifecycle |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps |
| **Communications Lead** | gtm-customer-success | Drafts tenant + public-status + regulatory notifications |
| **SME** | axis-social + relevant BC owner | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for any data-breach-suspect (Sev-1 confidentiality); owns regulatory chain |
| **Trust + Safety Lead** | axis-social + ops-security | Activates for moderation classifier / sybil / abuse-spike incidents |
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
    ↓ (EU DSA-scope + content moderation incident → transparency log update)
```

## Cross-Tenant or Cross-Context Leak = Sev-1

Any confirmed cross-tenant or cross-context data exposure — including cross-pack residency violation, unauthorised follow-graph reveal at scale, Personal-tier federation leak, or minor-list pivot — is **automatically Sev-1** regardless of scope. Triggers:

- IC immediately engages PrivacyLead + ops-security.
- Affected scope is quarantined (writes blocked; reads only by ops-security under JIT elevation).
- Audit-chain replay of affected scope begins.
- GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 / EU DSA Art. 24 clocks may start; CommsLead drafts notifications.
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

Every Sev-1 / Sev-2 alert fires BOTH a Prometheus metric (`social_incident_active{severity="N"}` with incident-id label) AND an OnCall page. If one is silent, the other still fires; on-call playbook requires both checks.

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
| EU DSA Art. 24 | Major content-moderation incident; transparency report update | quarterly cycle + immediate for major incidents | EU DSA Coordinator |
| EU AI Act Art. 73 | Serious incident involving high-risk AI system | "without undue delay" (≤ 15 days) | Market surveillance authority |
| UK Online Safety Act 2023 | Significant illegal-content failure | Ofcom-defined window | Ofcom |
| AU Online Safety Act 2021 | Class-1 / Class-2 material failure | per BOSE | eSafety Commissioner |
| UAE PDPL | Breach | 72h | UAE Data Office |
| KSA PDPL | Breach | 72h | SDAIA |

PrivacyLead owns the regulatory chain; ops-security supports forensic-trace; CommsLead drafts notification text from `legal/breach-notification-templates.md` (Slice B).

## Communication Templates

(Templates in `legal/breach-notification-templates.md`; here are the placeholders.)

### Sev-1 tenant notification

```
Subject: [Service Incident] social Sev-1 incident-<id> in pack <pack>

We detected an incident affecting your tenant's social service starting at <UTC-ts>.
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

### EU DSA Art. 24 transparency-report update

```
Per Regulation (EU) 2022/2065 Art. 24, this is an immediate update to the
transparency report for the period <start>-<end>.

Incident: <id>
Content-moderation impact: <verdict-count-affected>
Appeal availability: <yes/no>
Restoration: <ts>
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

Postmortem template: `microservices/social/runbooks/postmortem-template.md` (Slice B).

## On-Call Rotation

| Rotation | Cadence | Owner |
|---|---|---|
| ops-sre-reliability primary | 1-week | ops-sre-reliability lead |
| ops-sre-reliability secondary | 1-week | ops-sre-reliability lead |
| ops-security primary (security/privacy incidents) | 1-week | ops-security lead |
| axis-social SME | 1-week | axis-social lead |
| Trust + Safety primary | 1-week | axis-social + ops-security |
| Privacy escalation | on-call council-privacy chair (or designate) | council-privacy |

Per-pack on-call: each active pack carries a separate primary + secondary rotation in the pack's local timezone with global escalation handoff.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Sev-1 cross-context violation tabletop | Quarterly | 2026-Q2 (scheduled) |
| Feed-render storm chaos | Quarterly | 2026-Q2 (scheduled) |
| Postgres primary failover | Quarterly | 2026-Q2 (scheduled) |
| Pack-wide DR failover | Annually (DR-pair packs) | 2026-Q3 (scheduled) |
| Regulatory-notification rehearsal | Annually | 2026-Q3 (scheduled) |
| Moderation classifier rollback | Quarterly | 2026-Q2 (scheduled) |
| Trending-topic poisoning attack drill | Quarterly | 2026-Q2 (scheduled) |

## References

- `microservices/social/threat-model.md`.
- `microservices/social/dpia.md`.
- `microservices/social/compliance.md`.
- `microservices/social/failure-modes.md`.
- `microservices/social/multi-region.md`.
- `microservices/social/runbooks/`.
- `microservices/observability/incident-response.md` (shape reference).
- `microservices/messenger/incident-response.md` (sibling reference).
- Bominal ADR-0028.
- GDPR Arts. 33, 34; KR PIPA Art. 34; HIPAA §164.404, §164.412; EU DSA Art. 24; EU AI Act Art. 73.
