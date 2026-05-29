---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
related_adrs: [ADR-0139, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Incident Response — calendar µservice

## Purpose

Define incident classification, on-call response procedures, regulator-notification timelines (GDPR Art. 33 / KR PIPA / HIPAA / NIS2 / etc.), and post-incident review for calendar.

## Severity classification

| Sev | Definition | Examples |
|---|---|---|
| Sev-1 | Customer-impacting outage or data-loss / data-breach affecting > 1 tenant | event-store unavailable for > 5 min; cross-tenant data leak; tenant-DEK compromise |
| Sev-2 | Customer-impacting degradation or single-tenant data risk | availability-resolver p99 > 2s; recurrence storm affecting tenant SLO; single-tenant DEK rotation failure |
| Sev-3 | Internal-only impact; no tenant impact | worker pod restart loop without queue drain impact; observability collector lag |
| Sev-4 | Non-urgent operational issue | minor SLO breach within error budget; transient single-pod failures |

## On-call rotation

- Primary: axis-calendar (rotating 1-week shifts).
- Secondary: ops-sre-reliability (cross-µservice on-call).
- Tertiary: council-architecture (escalation for design-level issues).
- Security on-call (24/7): ops-security.
- Privacy on-call (24/5 + breach-trigger 24/7): council-privacy.

Paging via Grafana OnCall; integration spec in `observability/policy/oncall-scope.cedar` (referenced).

## Incident lifecycle

```
Detection → Triage → Classify (Sev) → Mitigate (runbook) → Communicate → Resolve → Post-incident review
```

### Detection signals

- SLO burn-rate alerts (Mimir → Alertmanager → OnCall).
- Audit-chain emission failure alerts.
- Cross-tenant query anomaly alerts.
- DEK rotation failure alerts.
- Tenant complaint via support channel.
- External security disclosure (responsible disclosure inbox).

### Triage (within 15 min of page)

| Action | Owner |
|---|---|
| Acknowledge page | primary on-call |
| Confirm scope (affected tenants / data classes) | primary on-call |
| Classify Sev (1-4) | primary on-call |
| Page security + privacy if Sev-1 or data-related | primary on-call |
| Open incident channel (Slack / Telegram per OnCall config) | primary on-call |
| Update tenant-facing status page if Sev-1 or Sev-2 | gtm-customer-success |

### Mitigation

Per-runbook procedure (see `runbooks/`). Common patterns:
- **Recurrence storm**: scale up worker pods; rate-limit RRULE submissions; runbook `recurrence-storm.md`.
- **Availability cache storm**: enable single-flight; warm cache from Postgres; runbook `availability-cache-rebuild.md`.
- **Time-zone DB stale**: trigger refresh; verify upstream IANA; runbook `timezone-db-refresh.md`.
- **Room booking conflict**: identify race source; tighten lock scope; runbook `room-booking-conflict.md`.
- **.ics import failure**: investigate parse error; surface to tenant; runbook `ics-import-failure.md`.
- **Backup / restore**: PITR; runbook `calendar-restore.md`.

### Communication

| Audience | Channel | Trigger |
|---|---|---|
| Engineering team | OnCall channel (Slack / Telegram) | Sev-1, Sev-2 |
| Customer-success | Internal Slack | Sev-1, Sev-2 |
| Affected tenants | Status page + per-tenant email | Sev-1 (always); Sev-2 (if customer-visible) |
| Public status page | status.oyatie.dev | Sev-1 |
| Council leadership | OnCall channel | Sev-1 |
| External regulator (DPA / PIPC / OCR / etc.) | Per-jurisdiction notification | Sev-1 with personal-data scope; per below |

### Resolution

- All immediate-impact mitigations applied.
- Tenant-facing status: "resolved".
- Post-incident review scheduled within 5 business days.

## Regulator notification timelines

### GDPR (Art. 33 + 34)

| Trigger | Timeline | Channel |
|---|---|---|
| Personal-data breach detected | 72h notification to supervisory authority | Per-DPA per-pack notification portal |
| High-risk breach affecting individuals | "without undue delay" notification to data subjects | Tenant DPA upstream-notification clause |
| Breach record-keeping | Forever | RoPA + breach register |

### KR PIPA (Art. 34 + 34-3)

| Trigger | Timeline | Channel |
|---|---|---|
| Personal info leak detected | 24h notification to affected users + within 72h to PIPC | PIPC portal + per-user notification |
| If ≥ 1000 users affected | Also report to KISA | KISA portal |

### HIPAA (45 CFR Part 164 Subpart D)

| Trigger | Timeline | Channel |
|---|---|---|
| PHI breach affecting < 500 individuals | Notify HHS OCR within 60 days of end of calendar year + notify individuals within 60 days | HHS OCR portal + individual notice |
| PHI breach affecting ≥ 500 individuals | Notify HHS OCR + media within 60 days | HHS OCR portal + media outlet in affected state |
| Breach record-keeping | 6 years | Compliance retention |

### NIS2 (2022/2555)

| Trigger | Timeline | Channel |
|---|---|---|
| Significant incident affecting essential / important entity | Initial 24h + detailed 72h + final 1 month | National CSIRT |

### APPI (Japan)

| Trigger | Timeline | Channel |
|---|---|---|
| Personal info leak affecting > 1000 users or sensitive info | Notify PPC within reasonable period + affected individuals | PPC portal + individual notice |

### Other packs

- pack-sg PDPA: notify PDPC within 72h + affected individuals.
- pack-au Privacy Act: notify OAIC + affected individuals within reasonable period.
- pack-in DPDPA 2023: notify Data Protection Board + affected individuals.
- pack-br LGPD: notify ANPD within reasonable period.
- pack-ae PDPL: notify UAE Data Office.
- pack-ksa PDPL: notify SDAIA.

## Specific incident playbooks

### Cross-tenant data leak (Sev-1 + GDPR / PIPA breach)

1. Acknowledge within 5 min.
2. Identify affected tenants from audit-chain query (which tenants were exposed to which).
3. Block the leaky path (Cedar policy refusal at runtime).
4. Determine scope: number of affected data subjects.
5. Page council-privacy + ops-security.
6. Within 24h: notify affected tenants.
7. Within 72h: notify GDPR DPA per affected pack-eu tenant (Art. 33).
8. Within 72h: PIPC notification per affected pack-kr tenant.
9. Per-jurisdiction notification per affected pack.
10. Post-incident review within 5 business days.

### Tenant-DEK compromise (Sev-1)

1. Acknowledge within 5 min.
2. Rotate tenant-DEK via OpenBao 2-person rule.
3. Re-encrypt active records with new DEK.
4. Audit-chain emit DEK-rotation event.
5. Notify affected tenant.
6. Forensic trace: how did DEK escape OpenBao?
7. Per-jurisdiction breach notification.
8. Engineering education + secret-scanner improvement.

### Audit-chain emission gap (Sev-2)

1. Acknowledge within 15 min.
2. Identify time range of emission failure from `calendar_audit_emission_ack_lag_seconds`.
3. Replay emission for missing range from outbox table.
4. Verify seal continuity post-replay.
5. Document gap in compliance evidence.
6. Post-incident review.

### .ics injection attempt detected (Sev-2)

1. Acknowledge within 30 min.
2. Quarantine the affected import job.
3. Replay parser against captured payload in sandbox.
4. If new attack pattern, add to fuzz corpus + update parser.
5. Notify affected tenant + security team.

### Recurrence storm (Sev-2)

1. Acknowledge within 30 min.
2. Identify offending tenant via worker logs.
3. Rate-limit tenant's RRULE submissions.
4. Drain worker queue with smaller batches.
5. Post-incident review: was rate-limit appropriate? Should bounds be tighter?

## Post-incident review (PIR)

Per Google SRE Workbook ch. 15. Conducted within 5 business days of resolution.

Template at `runbooks/post-incident-review-template.md` (referenced).

- Blameless: focus on systems, not individuals.
- Outputs: root-cause analysis; corrective actions; LEAN-lane updates; runbook updates; threat-model re-review if applicable.
- Action items tracked in `evidence/incidents/<incident_id>.json`.

## Drills

| Drill | Cadence | Scope |
|---|---|---|
| Cross-tenant-leak simulation | Annually | red-team |
| DEK compromise simulation | Annually | ops-security |
| Region failover | Quarterly | per-pack |
| Recurrence storm simulation | Bi-annually | axis-calendar |
| .ics injection corpus replay | Quarterly | axis-calendar + ops-security |
| Audit-chain emission failure | Quarterly | observability + audit-chain |

## References

- ADR-0139: SLO-gated promotion.
- ADR-0140: Cedar policy.
- `failure-modes.md`, `runbooks/*`, `compliance.md`, `multi-region.md`.
- GDPR Arts. 33 + 34; EDPB Guidelines 9/2022.
- KR PIPA Arts. 34 + 34-3.
- HIPAA 45 CFR Part 164 Subpart D.
- NIS2 (2022/2555).
- APPI (Japan).
- Google SRE Workbook ch. 11 (managing incidents) + ch. 15 (post-incident review).
