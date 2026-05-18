---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-meet, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/meet/threat-model.md
  - microservices/meet/dpia.md
  - microservices/meet/compliance.md
  - microservices/meet/failure-modes.md
  - microservices/meet/multi-region.md
  - microservices/meet/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (meet µservice)

## Purpose

End-to-end incident-response procedure for meet events. Covers severity classification, response roles, escalation, communication, postmortem cadence, and per-pack regulatory-notification timelines. **Cross-tenant recording leak, lobby bypass, and E2E decrypt attempt are automatically Sev-1.**

## Severity Definitions

| Severity | Definition | Page-to-ack target | Examples |
|---|---|---|---|
| **Sev-1** | Cross-tenant or cross-context confidentiality / integrity / availability impact; regulatory notification triggers; data breach; safety violation | ≤ 5 min (24/7) | FM-05 lobby bypass; FM-08 Postgres primary outage; FM-09 cross-tenant leak; FM-10 pack residency misroute; FM-11 E2E decrypt attempt; FM-13 unauthorized RTMP egress |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; service unavailable for multi-tenant subset | ≤ 15 min (24/7) | FM-01 SFU degraded; FM-02 coturn saturation; FM-03 recording-storage degraded; FM-04 transcription rollback; FM-06 live-caption stalled; FM-07 webinar overload; FM-12 ffmpeg sandbox alert |
| **Sev-3** | Localised impact; degraded but functional | ≤ 1h (business hours) | FM-14 retention-hold conflict; FM-15 capacity exhaustion; FM-16 ontology degraded; FM-17 calendar binding stale |
| **Sev-4** | Cosmetic; tracked but not paged | next business day | dashboard label typos; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security/privacy incidents) | Owns incident lifecycle |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps |
| **Communications Lead** | gtm-customer-success | Drafts tenant + public-status + regulatory notifications |
| **SME** | axis-meet + relevant BC owner | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for any data-breach-suspect (Sev-1 confidentiality); owns regulatory chain |
| **Executive Sponsor** | council-architecture chair (Sev-1) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline in `#inc-<id>` |
| **Active-Meeting Comms** | axis-meet on-call SME | Surface in-app banner to hosts of affected active meetings |

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
    ↓ (SEC/FINRA-scope tenant + recording integrity event → 17a-4 supervisor notification)
```

## Cross-Tenant Recording Leak = Sev-1

Any confirmed cross-tenant recording or transcript exposure — including cross-context, cross-pack residency violation, or unauthorised meeting membership — is **automatically Sev-1** regardless of scope. Triggers:

- IC immediately engages PrivacyLead + ops-security.
- Recording is quarantined (writes blocked; reads only by ops-security under JIT elevation).
- Audit-chain replay of affected recording lifecycle history begins.
- GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 / SEC 17a-4 supervisor notification clocks may start; CommsLead drafts notifications.
- Sev-1 postmortem within 5 business days; council-privacy + ops-security sign-off.

## In-Meeting Sev-1 Procedure

If a Sev-1 occurs while meetings are active:

1. Active-Meeting Comms lead surfaces an in-app banner to all affected hosts within 2 minutes: "We have detected a service incident; please consider concluding the meeting and rejoining."
2. For recording-related Sev-1: recordings are paused (write-blocked); ongoing transcription paused; tenants notified.
3. For lobby-bypass Sev-1: lobby evaluation strict-mode enabled cluster-wide; in-meeting banner warns hosts to verify attendee list.
4. For E2E-decrypt-attempt Sev-1: no user-visible change (operation already denied); ops-security forensics begin.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| 1. Detection | Alert + metric both received | ≤ 60s alert-to-page p99 |
| 2. Acknowledgement | On-call ack; open `#inc-<id>`; page IC | ≤ 5 min (Sev-1) / ≤ 15 min (Sev-2) |
| 3. Containment | Apply immediate mitigation per runbook | ≤ 15 min Sev-1 / ≤ 30 min Sev-2 |
| 4. Communication | Tenant + status page notifications + active-meeting banners | ≤ 30 min Sev-1 |
| 5. Eradication | Root-cause + fix | hours |
| 6. Recovery | Verify full service | within RTO |
| 7. Postmortem | Blameless review with action items | 5 business days |

## Two-Channel Corroboration

Every Sev-1 / Sev-2 alert fires BOTH a Prometheus metric (`meet_incident_active{severity="N"}` with incident-id label) AND an OnCall page. If one is silent, the other still fires; on-call playbook requires both checks.

## Regulatory Notifications

| Pack / framework | Trigger | Window | Recipient |
|---|---|---|---|
| GDPR Art. 33 | Personal-data breach affecting EU subjects | 72h initial + supplementary as known | Lead supervisory authority + affected subjects (Art. 34) |
| KR PIPA Art. 34 | Personal-data leakage | "without delay" (interpreted: ≤ 24h) | KR PIPC + affected subjects |
| HIPAA §164.412 (Breach Notification Rule) | PHI breach affecting ≥ 500 individuals | 60d notification + HHS reporting | HHS + media + affected individuals |
| HIPAA §164.404 (individual notification) | PHI breach (any size) | 60d | Affected individuals |
| SEC Rule 17a-4(f) supervisor notification | Recording integrity event | "promptly" | Tenant FINRA-supervisor |
| FINRA Rule 4530 | Reportable event | per FINRA timelines | FINRA |
| MiFID II (pack-eu investment-firm) | Recording integrity event | "without undue delay" | Tenant compliance officer + competent authority |
| APPI Art. 22-2 | PII leakage at scale | "promptly" (≤ 5 business days) | JP PPC + affected subjects |
| PDPA SG | Significant breach | 72h | PDPC |
| LGPD Art. 48 | Breach with risk to subjects | "reasonable time" | ANPD + affected subjects |
| NIS2 (when engaged) | Significant incident | 24h initial + 72h detailed + 1mo final | National CSIRT |
| UAE PDPL | Breach | 72h | UAE Data Office |
| KSA PDPL | Breach | 72h | SDAIA |

PrivacyLead owns the regulatory chain; ops-security supports forensic-trace; CommsLead drafts notification text from `legal/breach-notification-templates.md`.

## Communication Templates

(Templates in `legal/breach-notification-templates.md`; here are the placeholders.)

### Sev-1 tenant notification

```
Subject: [Service Incident] meet Sev-1 incident-<id> in pack <pack>

We detected an incident affecting your tenant's meet service starting at <UTC-ts>.
Scope: <scope>
Affected meetings: <count> meetings in <time-window>
Status: <status>
Workaround: <workaround>

Updates will follow at <status-page-url>.
```

### Active-meeting in-app banner

```
Service incident detected. Recording is currently paused. Your meeting can continue, but please consider rejoining if media quality degrades.
```

### GDPR Art. 34 affected-subject notification

```
Subject: Notification of personal data incident

A personal data incident has been identified affecting your information.
Date detected: <ts>
Nature: <nature; including whether meeting recordings or transcripts were involved>
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

Postmortem template: `microservices/meet/runbooks/postmortem-template.md` (Slice D).

## On-Call Rotation

| Rotation | Cadence | Owner |
|---|---|---|
| ops-sre-reliability primary | 1-week | ops-sre-reliability lead |
| ops-sre-reliability secondary | 1-week | ops-sre-reliability lead |
| ops-security primary (security/privacy incidents) | 1-week | ops-security lead |
| axis-meet SME | 1-week | axis-meet lead |
| axis-foundry-runtime SME (transcription/AI) | 1-week | axis-foundry-runtime lead |
| Privacy escalation | on-call council-privacy chair (or designate) | council-privacy |

Per-pack on-call: each active pack carries a separate primary + secondary rotation in the pack's local timezone with global escalation handoff.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Sev-1 lobby-bypass tabletop | Quarterly | 2026-Q2 (scheduled) |
| LiveKit SFU degraded chaos | Quarterly | 2026-Q2 (scheduled) |
| Recording S3 outage during active meetings | Quarterly | 2026-Q2 (scheduled) |
| Whisper GPU pool exhaustion | Quarterly | 2026-Q2 (scheduled) |
| Pack-wide DR failover | Annually (DR-pair packs) | 2026-Q3 (scheduled) |
| Regulatory-notification rehearsal | Annually | 2026-Q3 (scheduled) |
| Webinar overload (10k+ attendees) load-test | Annually | 2026-Q3 (scheduled) |

## References

- `microservices/meet/threat-model.md`.
- `microservices/meet/dpia.md`.
- `microservices/meet/compliance.md`.
- `microservices/meet/failure-modes.md`.
- `microservices/meet/multi-region.md`.
- `microservices/meet/runbooks/`.
- `microservices/messenger/incident-response.md` (shape reference).
- `microservices/observability/incident-response.md` (shape reference).
- Bominal ADR-0028.
- GDPR Arts. 33, 34; KR PIPA Art. 34; HIPAA §164.404, §164.412; SEC 17a-4(f); FINRA 4530.
