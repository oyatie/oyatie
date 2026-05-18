---
doc_class: IncidentResponsePlan
title: Incident Response Plan
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-mail + ops-security + council-privacy + ops-legal
deciders: ops-sre-reliability, axis-mail, ops-security, council-privacy, ops-legal, gtm-customer-success
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/mail/threat-model.md
  - microservices/mail/dpia.md
  - microservices/mail/failure-modes.md
  - microservices/mail/runbooks/
review_cadence: annually + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Plan (mail µservice)

## Purpose

Define severity classification, response chain, regulatory notification timelines, and post-incident review for mail-µservice incidents. Sev-1 events include mailbox-content leak, SMTP relay abuse, cross-pillar breach, eDiscovery seal mismatch, and DKIM key compromise.

## Severity Definitions

| Severity | Definition | Examples (mail µservice) | Response time | Escalation |
|---|---|---|---|---|
| **Sev-1** | Service down / data breach / regulatory breach / safety-of-data event | Mailbox-content cross-tenant leak (FM-11); SMTP IP pool blocklisting cascade (FM-09); Legal-hold drift causing held material deletion (FM-05); Cross-pillar breach (FM-11); eDiscovery seal mismatch potential tampering (FM-12); DKIM key compromise; Postgres mailbox-store outage > 30 min | 5 min to engage on-call | Page; escalate to leadership within 15 min |
| **Sev-2** | Degraded service; major-customer impact; security event without breach | SMTP relay outage (FM-01) > 15 min; Outbound delivery rejection wave (FM-02); Search index corruption (FM-04); IMAP brute-force storm (FM-07); DKIM rotation failure (FM-08); Retention sweep stalled > 24h | 15 min to engage on-call | Page; tenant comms within 1h |
| **Sev-3** | Single-customer or limited impact; degraded but workaround exists | Mailbox quota exhaustion (FM-03); KMS rotation gap (FM-06) ≤ 1h; Workflow handoff failure (FM-15); CI lane flakiness | 1h to engage | Slack; tenant comms within 4h if customer-visible |
| **Sev-4** | Internal issue; no customer impact | Internal metric alarm; CI lane warning | Next business day | Slack |

## Response Chain

### Sev-1 Response (max 5 min to engage)

```text
Detection (PagerDuty / Grafana OnCall page)
    ↓ ≤ 5 min
On-call mail engineer accepts page; opens #inc-mail-<id> channel
    ↓ ≤ 5 min
Engineer assesses; declares severity (Sev-1 confirmed)
    ↓ ≤ 10 min
Incident Commander assigned (rotating); engages:
  - ops-security (any security event)
  - council-privacy (any privacy-affecting event)
  - ops-legal (any disclosure event)
  - axis-mail lead (technical)
  - gtm-customer-success (tenant comms)
    ↓ ≤ 15 min
Status page update: tenant-visible incident posted
    ↓ ≤ 1h
Incident under control OR Sev-1 stays open with rolling updates every 30 min
    ↓
Resolution
    ↓ ≤ 5 business days
Postmortem published per docs/templates/postmortem-template.md
```

### Sev-2 Response (15 min to engage)

Similar but compressed. No Incident Commander unless multi-team needed. Tenant comms within 1h.

### Sev-3 + Sev-4

Slack-driven. Engineer resolves within working hours.

## Regulatory Notifications

### GDPR Art. 33 (72h to supervisory authority)

For Sev-1 events involving EU-resident personal data:
- **Within 72h of awareness**: notify the lead supervisory authority (Lead SA per Art. 56).
- **Notification content**: nature of breach + categories of affected subjects + categories of affected data + likely consequences + measures taken + DPO contact.
- **If 72h missed**: justification documented per Art. 33(1) second sub-paragraph.
- **Without undue delay**: notify affected data subjects per Art. 34 when "high risk to rights and freedoms".

For mail-content leaks specifically:
- Data subject = tenant employees + external recipients/senders.
- Joint controllership: tenant notifies its employees + recipients; oyatie notifies tenant + the supervisory authority directly per joint-controllership terms.

### KR PIPA Art. 34 (72h to PIPC + 72h to data subjects)

For Sev-1 involving KR-resident PII:
- **Within 72h to PIPC**: 개인정보보호위원회 (Personal Information Protection Commission).
- **Within 72h to subjects**: notification per PIPA Enforcement Decree Art. 38.
- **Records retained**: ≥ 1y per Art. 30; 5y for KR-FSS tenants.

### HIPAA §164.404 + §164.406 + §164.408 (pack-us-healthcare)

For Sev-1 involving PHI:
- **Within 60d to affected individuals**: HIPAA Breach Notification Rule §164.404.
- **Within 60d to media** if > 500 affected in single state/jurisdiction: §164.406.
- **Within 60d to HHS OCR** if > 500 affected; annual summary for < 500: §164.408.
- **HITECH §13402**: 60d notification timeline.

### Other packs

- **APPI Art. 26-2 (JP)**: notification to PPC + data subjects within reasonable time.
- **LGPD Art. 48 (BR)**: notification to ANPD + subjects within reasonable time.
- **DPDPA §8(6) (IN)**: notification to Data Protection Board + subjects "as may be prescribed".
- **NIS2 (EU)**: 24h initial + 72h detailed + 1mo final if oyatie crosses Annex I/II threshold.
- **SAMA Cybersecurity Framework (pack-ksa)**: notify SAMA within reporting window.

## Regulatory Notification Templates

Templates at `legal/breach-notification-templates/`:
- `gdpr-art33.md` — to supervisory authority
- `gdpr-art34.md` — to data subjects
- `pipa-art34-pipc.md` — to KR PIPC (Korean)
- `pipa-art34-subjects.md` — to KR data subjects (Korean)
- `hipaa-164.404.md` — to HIPAA-affected individuals
- `hipaa-164.408-ocr.md` — to HHS OCR
- `appi-26-2-ppc.md` — to JP PPC
- `lgpd-art48-anpd.md` — to BR ANPD
- `nis2-initial-72h.md` — initial NIS2 report

## Communication Channels

| Channel | Use |
|---|---|
| `#inc-mail-<id>` Slack | Per-incident war room |
| `incidents@oyatie.dev` email | Inbound incident reports |
| `status.oyatie.dev` | Tenant-facing incident status |
| PagerDuty / Grafana OnCall | Initial paging |
| Tenant DPA-listed contact | Per-tenant breach notification |

## Severity-Specific Runbooks

| Severity | Type | Runbook |
|---|---|---|
| Sev-1 | SMTP relay outage | `runbooks/smtp-relay-outage.md` |
| Sev-1 | Mailbox-content cross-tenant leak | `runbooks/security-incident.md` §"Cross-tenant leak" |
| Sev-1 | Cross-pillar breach | `runbooks/security-incident.md` §"Pillar breach" |
| Sev-1 | Legal-hold drift | `runbooks/legal-hold-engage.md` §"Drift recovery" |
| Sev-1 | eDiscovery seal mismatch | `runbooks/ediscovery-export.md` §"Seal mismatch" |
| Sev-1 | DKIM key compromise | `runbooks/dkim-rotation-recovery.md` (cross-ref) |
| Sev-1 | Postgres mailbox-store outage | `runbooks/mailbox-restore.md` §"Postgres failover" |
| Sev-2 | Outbound deliverability rejection | `runbooks/deliverability-reputation-recovery.md` (cross-ref) |
| Sev-2 | Search index corruption | `runbooks/search-index-rebuild.md` |
| Sev-2 | IMAP brute-force storm | `runbooks/imap-storm-throttle.md` |
| Sev-3 | Mailbox quota exhaustion | `runbooks/mailbox-restore.md` §"Quota" |

## Post-Incident Review

Every Sev-1 + Sev-2 incident produces a postmortem within 5 business days:
- Timeline (detect → contain → resolve)
- Root cause (5-whys; no blame)
- Contributing factors
- Action items (with owners + dates)
- Process improvements
- Lessons learned

Postmortem template: `runbooks/postmortem-template.md` (cross-ref to standard).

Action items tracked in `runbooks/postmortem-action-items.md` + lane `oya-governance-postmortem-action-items-current` ensures stale items are surfaced.

## Annual Review

This document reviewed annually + after every Sev-1 event affecting mail. Changes audit-chained.

## References

- ADR-0028 (Bominal): audit-chain.
- ADR-0117: residency.
- ADR-0135: Connect dissolution.
- ADR-0139: SLO gate.
- ADR-0131: per-microservice flat layout.
- `microservices/mail/threat-model.md`.
- `microservices/mail/dpia.md`.
- `microservices/mail/failure-modes.md`.
- `microservices/mail/runbooks/*`.
- GDPR Arts. 33 + 34.
- KR PIPA Art. 34 + Enforcement Decree Art. 38.
- HIPAA §164.404 + §164.406 + §164.408.
- APPI Art. 26-2.
- LGPD Art. 48.
- DPDPA 2023 §8(6).
- NIS2 (2022/2555).
- SAMA Cybersecurity Framework.
- Google SRE Workbook ch. 12 (Postmortem culture).
- M³AAWG abuse-reporting standards.
