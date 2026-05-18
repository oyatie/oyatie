---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy + ops-compliance
deciders: ops-sre-reliability, ops-security, council-privacy, axis-network, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/network/threat-model.md
  - microservices/network/dpia.md
  - microservices/network/compliance.md
  - microservices/network/failure-modes.md
  - microservices/network/multi-region.md
  - microservices/network/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (network µservice)

## Purpose

End-to-end incident-response procedure for `network` events. Covers severity classification, response roles, escalation, communication, postmortem cadence, and per-pack regulatory-notification timelines. **Cross-tenant or cross-context leak is automatically Sev-1**, as is **any recruiter-stub bias-audit failure that materially impacted employment decisions** (EU AI Act Art. 73 serious-incident trigger + EEOC + NYC LL144 reportable event).

## Severity Definitions

| Severity | Definition | Page-to-ack target | Examples |
|---|---|---|---|
| **Sev-1** | Cross-tenant or cross-context confidentiality / integrity / availability impact; regulatory notification triggers; data breach; safety violation; recruiter-stub serious-incident; endorsement-chain integrity compromise | ≤ 5 min (24/7) | FM-05 connection-graph corruption; FM-07 cross-tenant leak; FM-10 cross-context violation; FM-12 four-eyes violation; FM-13 pack misroute; FM-14 endorsement-chain integrity compromise; FM-15 recruiter-stub bias-audit failure; FM-16 minor-account leak |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss; service unavailable for multi-tenant subset; recommender drift; classifier rollback | ≤ 15 min (24/7) | FM-01 feed-render storm; FM-02 Postgres primary outage; FM-03 Redis corruption; FM-04 media-store outage; FM-09 media-malware detected (pattern); FM-17 recommender drift; FM-19 jobs-handoff bridge degraded |
| **Sev-3** | Localised impact; degraded but functional | ≤ 1h (business hours) | FM-06 search lag; FM-08 endorsement storm; FM-11 notification fanout backlog; FM-18 trending poisoning; FM-20 capacity exhaustion; FM-21 ontology degraded; FM-22 InMail-bridge degraded; FM-23 vCard export corruption |
| **Sev-4** | Cosmetic; tracked but not paged | next business day | dashboard label typos; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security/privacy incidents) | Owns incident lifecycle |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps |
| **Communications Lead** | gtm-customer-success | Drafts tenant + public-status + regulatory notifications |
| **SME** | axis-network + relevant BC owner | Diagnoses root cause |
| **Privacy Lead** | council-privacy chair | Activates for any data-breach-suspect (Sev-1 confidentiality); owns regulatory chain |
| **Trust + Safety Lead** | axis-network + ops-security | Activates for moderation classifier / abuse-spike / harassment-report-spike incidents |
| **Compliance Lead** | ops-compliance | Activates for recruiter-stub bias-audit failures + EEOC/NYC LL144/EU AI Act Art. 73 serious-incident reporting |
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
    ↓ (employment-context impact → EEOC + NYC LL144 + EU AI Act Art. 73 clocks)
```

## Cross-Tenant or Cross-Context Leak = Sev-1

Any confirmed cross-tenant or cross-context data exposure — including cross-pack residency violation, unauthorised connection-graph reveal at scale, Professional → Personal context bleed, or minor-account pivot — is **automatically Sev-1** regardless of scope. Triggers:

- IC immediately engages PrivacyLead + ops-security.
- Affected scope is quarantined (writes blocked; reads only by ops-security under JIT elevation).
- Audit-chain replay of affected scope begins.
- GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 / EU DSA Art. 24 clocks may start; CommsLead drafts notifications.
- Sev-1 postmortem within 5 business days; council-privacy + ops-security sign-off.

## Recruiter-Stub Bias-Audit Failure = Sev-1

Recruiter-stub is EU AI Act Annex III §4 HIGH-RISK. A failed bias audit (4/5-rule violation; protected-group disparity ratio < 0.8) that affected production decisions triggers:

- Auto-rollback recruiter-stub to last-known-good model version per `runbooks/recruiter-classifier-rollback.md`.
- Pause recruiter-stub for all NYC + CA + CO tenants pending re-audit.
- ComplianceLead engaged; EU AI Act Art. 73 serious-incident notification clock starts (≤ 15 days to market surveillance authority).
- NYC DCWP notification per LL144 §20-872 (when NYC tenant affected).
- EEOC notification scoping per UGESP record-keeping retention 2y.
- Affected-candidate notification per CA AB-331 §22756.3 + CO SB 24-205 §6-1-1701.
- Audit-chain seal of failure event + rollback action.

## Endorsement-Chain Integrity Compromise = Sev-1

Per ADR-NET-0005, endorsement chain is Merkle + per-endorser Ed25519. If integrity verification fails (Merkle root mismatch OR signature verification failure at scale):

- Quarantine affected endorsement-chain partition.
- Re-derive endorsement chain from audit-chain authoritative replay.
- Verify per-endorser Ed25519 signatures (forensic batch).
- Engage ops-security; treat as potential signing-key compromise.
- Affected endorsements marked `integrity_under_verification` in user UI.
- Postmortem within 5 business days.

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

Every Sev-1 / Sev-2 alert fires BOTH a Prometheus metric (`network_incident_active{severity="N"}` with incident-id label) AND an OnCall page. If one is silent, the other still fires; on-call playbook requires both checks.

## Regulatory Notifications

| Pack / framework | Trigger | Window | Recipient |
|---|---|---|---|
| GDPR Art. 33 | Personal-data breach affecting EU subjects | 72h initial + supplementary as known | Lead supervisory authority + affected subjects (Art. 34) |
| KR PIPA Art. 34 | Personal-data leakage | "without delay" (interpreted: ≤ 24h) | KR PIPC + affected subjects |
| KR 근로기준법 (Labor Standards Act) | Material employment-record breach | ≤ 7d | KR Ministry of Employment + affected workers |
| HIPAA §164.412 (Breach Notification Rule) | PHI breach affecting ≥ 500 individuals | 60d notification + HHS reporting | HHS + media + affected individuals |
| HIPAA §164.404 (individual notification) | PHI breach (any size) | 60d | Affected individuals |
| APPI Art. 22-2 | PII leakage at scale | "promptly" (≤ 5 business days) | JP PPC + affected subjects |
| PDPA SG | Significant breach | 72h | PDPC |
| LGPD Art. 48 | Breach with risk to subjects | "reasonable time" | ANPD + affected subjects |
| NIS2 (when engaged) | Significant incident | 24h initial + 72h detailed + 1mo final | National CSIRT |
| EU DSA Art. 24 | Major moderation incident; transparency report update | quarterly cycle + immediate for major incidents | EU DSA Coordinator |
| EU AI Act Art. 73 | Serious incident involving high-risk AI system (recruiter ranker; jobs ranker; endorsement aggregation) | "without undue delay" (≤ 15 days) | Market surveillance authority |
| NYC Local Law 144 §20-872 | Significant disparity finding in bias audit | within reporting cycle to DCWP | DCWP |
| US EEOC UGESP 29 CFR §1607.4 | Adverse-impact finding | retain records 2y; respond on EEOC charge | EEOC (on charge) |
| CA AB-331 §22756 | Algorithmic-discrimination incident | per CA AG guidance | CA AG |
| CO SB 24-205 | Algorithmic-discrimination incident | per CO AG guidance | CO AG |
| UK Online Safety Act 2023 | Significant illegal-content failure | Ofcom-defined window | Ofcom |
| AU Online Safety Act 2021 | Class-1 / Class-2 material failure | per BOSE | eSafety Commissioner |
| UAE PDPL | Breach | 72h | UAE Data Office |
| KSA PDPL | Breach | 72h | SDAIA |
| DPDPA 2023 (India) | Breach | 72h | DPDP Board |

ComplianceLead owns the employment-law chain; PrivacyLead owns the privacy chain; ops-security supports forensic-trace; CommsLead drafts notification text from `legal/breach-notification-templates.md` (Slice B).

## Communication Templates

(Templates in `legal/breach-notification-templates.md`; here are the placeholders.)

### Sev-1 tenant notification

```
Subject: [Service Incident] network Sev-1 incident-<id> in pack <pack>

We detected an incident affecting your tenant's network service starting at <UTC-ts>.
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

### EU AI Act Art. 73 serious-incident notification

```
Per Regulation (EU) 2024/1689 Art. 73, this is a serious-incident report for
the network µservice high-risk AI system (recruiter-search ranker; jobs ranker;
endorsement aggregation; people-you-may-know recommender).

System: <system_id>
Provider: oyatie GmbH
Incident: <id>
Date detected: <ts>
Nature of incident: <nature>
Impact: <impact>
Corrective action taken: <action>
Contact: <provider-contact>
```

### NYC Local Law 144 candidate notification

```
Per NYC Admin Code §20-871, automated employment decision tools may be used
in connection with your candidacy. The most recent bias-audit summary is
available at <url>. To request alternative process, contact: <contact>.
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

Postmortem template: `microservices/network/runbooks/postmortem-template.md` (Slice B).

## On-Call Rotation

| Rotation | Cadence | Owner |
|---|---|---|
| ops-sre-reliability primary | 1-week | ops-sre-reliability lead |
| ops-sre-reliability secondary | 1-week | ops-sre-reliability lead |
| ops-security primary (security/privacy incidents) | 1-week | ops-security lead |
| axis-network SME | 1-week | axis-network lead |
| Trust + Safety primary | 1-week | axis-network + ops-security |
| Compliance escalation (recruiter/bias-audit) | on-call ops-compliance | ops-compliance |
| Privacy escalation | on-call council-privacy chair (or designate) | council-privacy |

Per-pack on-call: each active pack carries a separate primary + secondary rotation in the pack's local timezone with global escalation handoff.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Sev-1 cross-context violation tabletop | Quarterly | 2026-Q2 (scheduled) |
| Recruiter-stub bias-audit failure tabletop (EU AI Act Art. 73) | Quarterly | 2026-Q2 (scheduled) |
| Endorsement-chain integrity verify drill | Quarterly | 2026-Q2 (scheduled) |
| Feed-render storm chaos | Quarterly | 2026-Q2 (scheduled) |
| Postgres primary failover | Quarterly | 2026-Q2 (scheduled) |
| Connection-graph corruption + replay rebuild | Quarterly | 2026-Q2 (scheduled) |
| InMail-bridge degraded drill | Quarterly | 2026-Q2 (scheduled) |
| ATS-bridge degraded drill | Annually | 2026-Q3 (scheduled) |
| Pack-wide DR failover | Annually (DR-pair packs) | 2026-Q3 (scheduled) |
| Regulatory-notification rehearsal (GDPR + EU AI Act + NYC LL144) | Annually | 2026-Q3 (scheduled) |
| Recommender classifier rollback | Quarterly | 2026-Q2 (scheduled) |
| Profile-export vCard corruption drill | Quarterly | 2026-Q2 (scheduled) |

## References

- `microservices/network/threat-model.md`.
- `microservices/network/dpia.md`.
- `microservices/network/compliance.md`.
- `microservices/network/failure-modes.md`.
- `microservices/network/multi-region.md`.
- `microservices/network/runbooks/`.
- `microservices/observability/incident-response.md` (shape reference).
- `microservices/social/incident-response.md` (sibling reference).
- Bominal ADR-0028.
- GDPR Arts. 33, 34; KR PIPA Art. 34; HIPAA §164.404, §164.412; EU DSA Art. 24; EU AI Act Art. 73; NYC LL144 §20-872; CA AB-331; CO SB 24-205.
