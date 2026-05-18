---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
related_adrs: [ADR-0130, ADR-0140]
doc_status: published
---

# Incident Response — docs µservice

## Purpose

Define incident classification, on-call response procedures, regulator-notification timelines (GDPR Art. 33 / KR PIPA / HIPAA / NIS2 / etc.), and post-incident review for docs.

## Severity classification

| Sev | Definition | Examples |
|---|---|---|
| Sev-1 | Customer-impacting outage or data-loss / data-breach affecting > 1 tenant; ANY CRDT silent loss confirmed | document-store unavailable for > 5 min; cross-tenant content leak; tenant-DEK compromise; ACL bypass producing content disclosure; gVisor sandbox escape |
| Sev-2 | Customer-impacting degradation or single-tenant data risk | collab cursor sync p99 > 1s; export pipeline failure rate > 10%; single-tenant DEK rotation failure; per-block ACL latency spike |
| Sev-3 | Internal-only impact; no tenant impact | worker pod restart loop without queue impact; observability collector lag; CRDT op-log compaction lag |
| Sev-4 | Non-urgent operational issue | minor SLO breach within error budget; transient single-pod failures |

## On-call rotation

- Primary: axis-docs (rotating 1-week shifts).
- Secondary: ops-sre-reliability (cross-µservice on-call).
- Tertiary: council-architecture (escalation for design-level issues).
- Security on-call (24/7): ops-security.
- Privacy on-call (24/5 + breach-trigger 24/7): council-privacy.

Paging via Grafana OnCall.

## Incident lifecycle

```
Detection → Triage → Classify (Sev) → Mitigate (runbook) → Communicate → Resolve → Post-incident review
```

### Detection signals

- SLO burn-rate alerts (Mimir → Alertmanager → OnCall).
- Audit-chain emission failure alerts.
- CRDT silent-loss-attempt counter > 0 (Sev-1 trigger).
- ACL-bypass anomaly alerts.
- DEK rotation failure alerts.
- Export sandbox escape signal (gVisor seccomp violations).
- Tenant complaint via support channel.
- External security disclosure (responsible disclosure inbox).

### Triage (within 15 min of page)

| Action | Owner |
|---|---|
| Acknowledge page | primary on-call |
| Confirm scope (affected tenants / data classes / BCs) | primary on-call |
| Classify Sev (1-4) | primary on-call |
| Page security + privacy if Sev-1 or data-related | primary on-call |
| Open incident channel (Slack / Telegram per OnCall config) | primary on-call |
| Update tenant-facing status page if Sev-1 or Sev-2 | gtm-customer-success |

### Mitigation

Per-runbook procedure (see `runbooks/`). Common patterns:
- **CRDT conflict resolution / silent-loss suspected**: `collab-conflict-resolution.md`.
- **Document version restore / corruption**: `doc-version-restore-corruption.md`.
- **Export pipeline failure / Pandoc rollback**: `export-pipeline-failure-pandoc-rollback.md`.
- **Attachment restore**: `attachment-restore.md`.
- **Share ACL drift**: `share-acl-drift.md`.
- **Editor-session storm throttle**: `editor-session-storm-throttle.md`.
- **Embed-source stale detection**: `embed-source-stale-detection.md`.

### Communication

| Audience | Channel | Trigger |
|---|---|---|
| Engineering team | OnCall channel | Sev-1, Sev-2 |
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

### CRDT silent-loss confirmed (Sev-1; AC-06 invariant breach)

1. Acknowledge within 5 min.
2. Declare Sev-1; engage axis-docs + ops-security + council-privacy.
3. Halt save-paths for affected (tenant, document) via `cargo run -p oya-dev-cli -- vcs override-paths --microservice docs --halt-saves --tenant <h> --document <d>` (2-person rule).
4. Reconstruct CRDT op stream from Postgres seal-deltas + Redis ephemeral state.
5. Forensic analysis: which op was dropped? engine-bug or adapter-bug?
6. Author hotfix; deploy via emergency-merge sign-off.
7. Tenant notification + regulator notification per pack.
8. Postmortem within 5 business days; structural fix mandatory.

### Cross-tenant data leak (Sev-1 + GDPR / PIPA breach)

1. Acknowledge within 5 min.
2. Identify affected tenants from audit-chain query.
3. Block the leaky path (Cedar policy refusal at runtime).
4. Determine scope: number of affected data subjects + content data-class.
5. Page council-privacy + ops-security.
6. Within 24h: notify affected tenants.
7. Within 72h: notify GDPR DPA per affected pack-eu tenant + PIPC per pack-kr.
8. Per-jurisdiction notification per affected pack.
9. Post-incident review within 5 business days.

### Per-block ACL bypass (Sev-1 + AC-04 breach)

1. Acknowledge within 5 min.
2. Identify affected blocks via audit-chain replay of access-events.
3. Tighten Cedar policy + Postgres block-level RLS.
4. Forensic: how did the bypass occur? regression test added.
5. Tenant + regulator notification per scope.

### Tenant-DEK compromise (Sev-1)

1. Acknowledge within 5 min.
2. Rotate tenant-DEK via OpenBao 2-person rule.
3. Re-encrypt active records with new DEK.
4. Audit-chain emit DEK-rotation event.
5. Notify affected tenant.
6. Forensic trace: how did DEK escape OpenBao?
7. Per-jurisdiction breach notification.

### Audit-chain emission gap (Sev-2)

1. Acknowledge within 15 min.
2. Identify time range from `docs_audit_emission_ack_lag_seconds`.
3. Replay emission from outbox table.
4. Verify seal continuity post-replay.
5. Document gap in compliance evidence.

### Export pipeline gVisor escape (Sev-1)

1. Acknowledge within 5 min.
2. Quarantine affected export workers; drain pool.
3. Replay attack payload in sandbox for forensic.
4. Patch gVisor / Pandoc / WeasyPrint / Chromium per ADR-DOCS-0003.
5. Tenant + ops-security notification.

### .docx injection attempt detected (Sev-2)

1. Acknowledge within 30 min.
2. Quarantine the affected import job.
3. Replay parser against captured payload in sandbox.
4. If new attack pattern, add to fuzz corpus + update parser per ADR-DOCS-0006.
5. Notify affected tenant + security team.

### Editor-session storm (Sev-2)

1. Acknowledge within 30 min.
2. Identify offending tenant via WS gateway lease metrics.
3. Apply per-tenant rate-limit.
4. Drain stuck leases.

## Post-incident review (PIR)

Per Google SRE Workbook ch. 15. Conducted within 5 business days of resolution.

- Blameless: focus on systems, not individuals.
- Outputs: root-cause analysis; corrective actions; LEAN-lane updates; runbook updates; threat-model re-review if applicable; structural fix mandatory for any AC-invariant breach.
- Action items tracked in `evidence/incidents/<incident_id>.json`.

## Drills

| Drill | Cadence | Scope |
|---|---|---|
| CRDT silent-loss simulation (ad-hoc fault injection) | Quarterly | axis-docs + ops-security |
| Per-block ACL bypass red-team | Annually | external pen-test |
| Cross-tenant-leak simulation | Annually | red-team |
| DEK compromise simulation | Annually | ops-security |
| Region failover | Quarterly | per-pack |
| Export pipeline gVisor escape simulation | Bi-annually | ops-security + axis-docs |
| Audit-chain emission failure | Quarterly | observability + audit-chain |
| Embed-loop / embed-storm chaos | Bi-annually | axis-docs |

## References

- ADR-0130: SLO-gated promotion.
- ADR-0140: Cedar policy.
- `failure-modes.md`, `runbooks/*`, `compliance.md`, `multi-region.md`.
- GDPR Arts. 33 + 34; EDPB Guidelines 9/2022.
- KR PIPA Arts. 34 + 34-3.
- HIPAA 45 CFR Part 164 Subpart D.
- NIS2 (2022/2555).
- APPI (Japan).
- Google SRE Workbook ch. 11 (managing incidents) + ch. 15 (post-incident review).
