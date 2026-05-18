---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + ops-security + ops-sre-reliability
related_adrs: [ADR-0114, ADR-0130, ADR-0131, ADR-DRIVE-0001, ADR-DRIVE-0003, ADR-DRIVE-0006]
doc_status: published
---

# Incident Response — drive µservice

## Purpose

Define incident-severity classes, response timelines, notification chains, rollback triggers, and post-incident review obligations for the drive µservice.

## Severity classes

| Sev | Definition | Response time | Notification |
|---|---|---|---|
| Sev-1 | Critical: bytes loss, mass data exfiltration, WORM violation, encryption-at-rest bypass, mass share-link compromise, > 30 min of full unavailability | ≤ 15 min | council-architecture + ops-security + ops-sre-reliability + tenant comms within 1h |
| Sev-2 | Major: significant degradation (SLO burn > 14x for 1h+), single-cell object-store loss, share-link enumeration in progress, virus-scan worker saturation, preview render queue saturation | ≤ 30 min | axis-drive on-call + ops-sre-reliability |
| Sev-3 | Minor: localised degradation, single tenant impact, scan false-positive flood, single-preview-render failure | ≤ 4h business hours | axis-drive on-call |
| Sev-4 | Informational: drift detected, anomaly observed, no current customer impact | next business day | axis-drive on-call |

## Notification timelines

### GDPR / EDPB

- 72h notification to supervisory authority for personal-data breach (GDPR Art. 33).
- Without undue delay to affected data subjects (GDPR Art. 34).
- Recorded in `incident-register.md` at `legal/incident-register.md`.

### NIS2 (pack-eu when threshold-engaged)

- 24h early warning.
- 72h incident notification.
- 1mo final report.

### KR PIPA (pack-kr)

- 72h notification per PIPA Art. 34.
- PIPC tenant-comms template at `legal/pipa-art34-template.md`.

### HIPAA (pack-us-healthcare)

- 60-day notification to HHS OCR per 45 CFR §164.400-414.
- BAA tenant comms ≤ 60 days.

### SEC 17a-4 / FINRA (pack-us broker-dealer)

- WORM violation: immediate suspension + SEC tenant notification.
- Tenant notification within tenant SLA.

### APPI (pack-jp)

- 3-business-day notification per APPI Art. 22.

## Common incident scenarios

### IR-1: Mass share-link enumeration in progress

1. Engage `runbooks/share-link-takeover-incident.md`.
2. Rate-limit + IP-block at WAF.
3. Rotate per-tenant signing keys.
4. Revoke active links matching enumeration pattern.
5. Audit-chain replay confirms scope.
6. Sev-2 by default; Sev-1 if cross-tenant or > 10k links accessed.

### IR-2: WORM violation (objects deleted before retention floor)

1. Engage `runbooks/immutability-tier-violation.md`.
2. Sev-1 by default — zero-tolerance per ADR-DRIVE-0006.
3. Suspend the affected tenant's deletion path.
4. Engage compliance + council-privacy + ops-security.
5. Forensic audit-chain replay to identify breach path.
6. Tenant comms within 1h.
7. Regulator comms per pack notification timeline.

### IR-3: Virus reached durable bucket (scan bypass)

1. Engage `runbooks/virus-scan-rollback.md`.
2. Quarantine object immediately.
3. Trigger fan-out scan across all objects ingested since the bypass window.
4. Audit-chain replay to identify access pattern.
5. Sev-2 by default; Sev-1 if malware spread to > 100 objects or downloaded by users.

### IR-4: Object-store cell loss (degraded)

1. Engage `runbooks/object-storage-degraded.md`.
2. Sev-2 if single-cell; Sev-1 if dual-cell.
3. Trigger rebuild from neighbour cells.
4. Monitor latency + replication backlog.

### IR-5: Mass upload stuck / multipart pipeline jam

1. Engage `runbooks/upload-multipart-stuck.md`.
2. Sev-2.
3. Drain stuck sessions; restart workers.

### IR-6: DLP false-positive flood blocking tenant

1. Engage `runbooks/dlp-quarantine-release.md`.
2. Sev-3 (single tenant); Sev-2 if cross-tenant or rule misconfig.
3. Manual quarantine release after policy review.
4. Tune DLP rules; re-run on quarantined set.

### IR-7: Sync conflict storm (mass simultaneous edits)

1. Engage `runbooks/sync-conflict-resolution.md`.
2. Sev-3 (tenant-scoped).
3. Surface conflicts to user; tenant-policy may auto-resolve.

## Rollback triggers (per ADR-0114)

- p99 latency regression > +5% over 7d baseline on hero metric (download first-byte, upload multipart, search query) → auto rollback canary.
- Error-rate regression > +0.1% over 7d baseline → auto rollback canary.
- WORM correctness verdict < 100% → BLOCKER; rollback + Sev-1.
- Virus-scan correctness verdict < 100% → BLOCKER; rollback + Sev-1.
- DLP false-positive rate > 5% rolling 24h → consider rollback.

## Post-incident review

Per ISO 27001 A.5.27 (lessons from incidents):
- Within 5 business days of Sev-1 / Sev-2 resolution.
- Attendees: incident commander, on-call, owning axis, ops-security.
- Output: written post-mortem at `evidence/postmortem/<incident_id>.md`; runbook updates if signature was new; LEAN-check addition if mitigated by code change.

## Tenant communication channels

- Primary: tenant-portal status page (per-pack) + email to tenant admin.
- Sev-1: phone call to tenant security contact.
- Sev-2: in-app banner + email.
- Public: oyatie status page (per pack) updated within 15 min for Sev-1.

## References

- ADR-0114 — canary observability + rollback.
- ADR-0130 — SLO-gated promotion.
- ADR-DRIVE-0003 — share-link security model.
- ADR-DRIVE-0006 — immutability + WORM policy.
- `microservices/drive/runbooks/*.md`.
- `microservices/drive/threat-model.md`.
- `microservices/drive/compliance.md` — per-pack notification timelines.
- GDPR Art. 33 + 34; NIS2 2022/2555; KR PIPA Art. 34; HIPAA 45 CFR §164.400-414; APPI Art. 22; SEC 17a-4(f).
