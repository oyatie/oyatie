# Analytics µservice — Incident Response Plan

**Authority:** ADR-0180 DR/business continuity portfolio policy, ADR-0152 RPO/RTO, ADR-0003 audit chain, NIST SP 800-61 Rev. 2 incident handling
**Owner:** council-analytics + ops-sre-reliability + axis-compliance
**Last reviewed:** 2026-05-18

## 1. Purpose

Operationalize the response to security, availability, and data-integrity incidents affecting the analytics µservice. Every incident touching tenant data triggers GDPR 72-hour notification consideration; every incident degrading SLO triggers customer comms.

## 2. Severity matrix

| Sev | Definition | Response time | Page target |
|---|---|---|---|
| Sev 1 | Customer-facing outage OR confirmed cross-tenant data leak OR backup-compromise OR loss of audit-chain integrity | < 5 min ack, < 30 min mitigation | PagerDuty `analytics-oncall` (page) + leadership |
| Sev 2 | Tenant-visible degradation (SLO burn > 6x in 6h) OR partial cluster failure (1 shard down) | < 15 min ack | PagerDuty `analytics-oncall` (page) |
| Sev 3 | Internal-only impact (capacity team paged, dashboard slow but within SLO) | < 1h ack | Ticket → `#analytics-incidents` |
| Sev 4 | Informational (single transient alert) | Best effort | Slack ack |

## 3. Roles (RACI)

| Role | Responsibility |
|---|---|
| Incident Commander (IC) | Owns the incident; one person; declared in `#analytics-incidents` |
| Communications Lead (CL) | Customer-facing comms; status page; tenant notifications |
| Ops Lead | Runs runbook steps; executes mitigations |
| Scribe | Real-time timeline; feeds post-mortem |
| Compliance Liaison | Triggered on data-leak or 72h-notice scenarios; loops in DPO |
| Executive Sponsor | Sev 1 only; loops in leadership |

## 4. Detection sources

- **PrometheusRule alerts** (IP-001, IP-014) — AlertManager → PagerDuty + Opsgenie.
- **Audit-chain forensic query** — detects cross-tenant access patterns post-hoc.
- **Tenant report** — support escalation; case ID maps to IC declaration.
- **Cosign verification failure** — backup or container image signature failure.
- **Reverse-CI canary** — synthetic tenant query failure.

## 5. Common incident playbooks

### 5.1 Cross-tenant data leak (Sev 1)

**Trigger:** Audit-chain query shows a principal of tenant A reading tenant B's database.

1. IC declares Sev 1 in `#analytics-incidents`.
2. Ops Lead executes Cedar policy lockdown: deploy `policy/emergency-deny-all.cedar` overriding all permits.
3. Compliance Liaison opens GDPR 72h timer; preps DPO notification draft.
4. Forensic: query `system.query_log` for the suspect principal; correlate with audit-chain.
5. CL drafts tenant notification (both tenants) under DPO guidance.
6. Once contained: revert lockdown to scoped Cedar policy fix.
7. Post-mortem within 5 business days.

### 5.2 ClickHouse Keeper quorum loss (Sev 1)

**Trigger:** `ClickHouseKeeperNoLeader` alert.

1. IC declares Sev 1.
2. Ops Lead checks `kubectl get pods -n analytics -l app.kubernetes.io/component=clickhouse-keeper`.
3. If 2 of 3 Keeper pods are unhealthy, follow `runbooks/keeper-quorum-recovery.md` to restore a Keeper from snapshot.
4. While Keeper is down, DDL (CREATE/DROP/ALTER) is unavailable; reads continue.
5. CL posts status: "Read traffic unaffected; new tenant onboard delayed."
6. Once recovered, verify all replicas re-sync via `system.replication_queue`.

### 5.3 Ingest lag burn (Sev 2)

**Trigger:** MV ingest lag p99 > 30 s for 15 min.

1. IC declares Sev 2.
2. Ops Lead checks Kafka consumer lag via `clickhouse-client -q 'SELECT * FROM system.kafka_consumers'`.
3. If lag growing: increase `kafka_num_consumers` from 6 → 12 via Helm patch.
4. If MV target table merge backlog: check `system.merges`; possibly throttle ingest temporarily.
5. CL: post status if customer-visible (dashboard freshness > 30s).
6. Follow `runbooks/ingest-lag-burn.md` for full procedure.

### 5.4 Backup compromise (Sev 1)

**Trigger:** cosign signature verification fails on backup pull.

1. IC declares Sev 1.
2. Compliance Liaison opens 72h timer (potential integrity event).
3. Ops Lead: invalidate the suspect backup id in the backup catalog (`evidence/backups/analytics/`); fall back to prior valid backup.
4. Forensic: inspect signing-key audit log in OpenBao; check for unauthorized signing operations.
5. If signing key compromised: rotate per `runbooks/backup-key-rotation.md` (deferred to phase 2); for now, generate new signing key, re-sign last 30 days of valid backups.
6. Post-mortem.

### 5.5 Tenant runaway query (Sev 3)

**Trigger:** `ClickHouseQuotaExceeded{tenant_id=...}` alert.

1. Ops triages per `runbooks/clickhouse.md` §"Per-tenant quota exceeded".
2. If misbehaving: account team contacts tenant; tier review.
3. If legitimate burst: tier upgrade emitted by tenancy µservice; IP-002 re-applies QUOTA.

### 5.6 Cold-tier S3 outage (Sev 2)

**Trigger:** Cold-tier query p99 > 10 s OR S3 5xx rate > 5%.

1. Ops checks SeaweedFS health in the cell.
2. If degraded: surface customer notice ("queries on data older than 90 days may be slow").
3. Restore from cell-local backup if needed.
4. Verify TTL→TODISK is not stuck (queries pile up if cold-tier is unreachable).

### 5.7 Bad MV deployment (Sev 2)

**Trigger:** MV target table backfill missing data after deploy.

1. Ops detect via reconciliation lane (IP-014).
2. Roll back MV per `runbooks/mv-lag-triage.md`.
3. Backfill from canonical source per `backfill-replay.md` procedure.

## 6. Communication protocol

| Audience | Channel | Cadence | Owner |
|---|---|---|---|
| Engineering | `#analytics-incidents` Slack | Real-time | Scribe |
| Customer | Status page + tenant email | Every 30 min during Sev 1 | CL |
| Leadership | Slack + email | At declare + at mitigate + at close | IC |
| Regulator | Per GDPR 72h | Compliance Liaison + DPO | DPO |

## 7. Tabletop exercise

Quarterly: tabletop one of (cross-tenant leak, Keeper quorum loss, backup compromise, ingest pipeline lateral attack). Record outcomes in `evidence/tabletop/analytics-<date>.json`. Schedule documented in `evidence/tabletop-calendar.md`.

## 8. Post-mortem template

Every Sev 1/Sev 2: blameless post-mortem within 5 business days. Template:

- Timeline (Scribe's real-time log, normalized).
- Detection (how + when).
- Impact (which tenants, what duration, financial estimate).
- Root cause.
- Contributing factors.
- Action items (each tagged P0/P1/P2 with owner + due date).
- Lessons learned.

Stored at `evidence/post-mortems/analytics/<date>-<incident-id>.md`.

## 9. References

- ADR-0180 DR business continuity portfolio policy.
- ADR-0152 RPO/RTO.
- ADR-0003 audit chain.
- NIST SP 800-61 Rev. 2 Computer Security Incident Handling Guide.
- GDPR Article 33 (notification of personal data breach).
