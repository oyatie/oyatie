---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-tasks + council-privacy
related_adrs: [ADR-0117, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Multi-Region — tasks µservice

## Purpose

Define the per-pack regional deployment topology, residency enforcement, cross-region replication policy, and disaster-recovery + failover model for tasks.

## Regional topology

### Pack-to-region mapping (canonical)

| Pack | Primary region | DR region (same jurisdiction) | Postgres cluster | Valkey cluster | Meilisearch cluster |
|---|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | OCI ap-chuncheon-1 | KR-primary | KR-primary | KR-primary |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | EU-primary | EU-primary | EU-primary |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | US-primary | US-primary | US-primary |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | OCI us-phoenix-1 (BAA-eligible) | US-HIPAA-primary | US-HIPAA-primary | US-HIPAA-primary |
| pack-jp | OCI ap-tokyo-1 | OCI ap-osaka-1 | JP-primary | JP-primary | JP-primary |
| pack-sg | OCI ap-singapore-1 | OCI ap-melbourne-1 | SG-primary | SG-primary | SG-primary |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | AU-primary | AU-primary | AU-primary |
| pack-in | OCI ap-mumbai-1 | OCI ap-hyderabad-1 | IN-primary | IN-primary | IN-primary |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | BR-primary | BR-primary | BR-primary |
| pack-ae | OCI me-dubai-1 | OCI me-jeddah-1 | AE-primary | AE-primary | AE-primary |
| pack-ksa | OCI me-jeddah-1 | OCI me-dubai-1 | KSA-primary | KSA-primary | KSA-primary |

### Residency invariants

Per `policy/data-residency.md`:
- Each tenant pinned to exactly one pack at onboarding.
- Postgres + Valkey + Meilisearch clusters pack-resident; no cross-pack replication by default.
- Cross-pack data flow only via SCC-gated cross-µservice handoffs (e.g., calendar µservice's cross-pack availability projection for due-date bridge); tasks itself does NOT directly replicate cross-pack.

## Cross-region replication policy

### Within-pack replication (DR)

- Postgres: synchronous replication primary → first replica (intra-AZ), async to off-AZ replica + DR-region replica.
- Replication factor: 3.
- Replication lag SLO: ≤ 5s within AZ; ≤ 30s cross-AZ; ≤ 60s cross-region.
- **Meilisearch: search-index is rebuildable from Postgres; cross-region Meilisearch replication NOT used; DR-region rebuilds in ≤30min for 10M tasks per AC-09.**

### Cross-pack replication

- **FORBIDDEN by default.**
- LEAN check `oya-check-cross-pack-replication-prohibition` refuses build if any config introduces cross-pack replication.
- Cross-pack data flow happens at access-time only (cross-µservice handoffs via Workflow event-bus); never at-rest.

### Backup + cold storage

- Per-pack S3-compatible cold-tier (OCI Object Storage); WORM where supported.
- Backup retention: 30d hot + 12mo cold; pack-us-healthcare ≥ 6y; pack-kr employment-context ≥ 3y (근로기준법 Art. 41).
- Backup encryption: pack-resident KMS keys; cross-pack restore blocked at KMS level.
- Backup integrity: weekly hash verification + monthly restore-test drill.

## Disaster recovery

### RTO + RPO

- RTO ≤ 15 min (production tier).
- RPO ≤ 60s (within-pack same-jurisdiction).
- Meilisearch: search-index RPO is effectively infinite (rebuildable from Postgres in ≤30min).
- DR-region failover: automated via Patroni leader election + DNS update.
- Failover steps:
  1. Primary fails health-check; Patroni elects new primary in DR region.
  2. DNS TTL ≤ 30s; clients re-resolve within 30-60s.
  3. Worker pods drain pending writes; resume against new primary.
  4. Meilisearch DR cluster activated; full-rebuild from Postgres starts (degraded mode = direct-Postgres-trigram search in the meantime).
  5. Audit-chain emission for failover event.

### DR drill cadence

- Quarterly: simulated primary-region outage; failover to DR region; validate audit-chain seal continuity + Postgres WAL replay completeness + Meilisearch rebuild completion.
- Annual: full-pack restore-from-cold (PITR + WAL replay).
- DR-drill evidence committed at `evidence/dr-drills/<pack>-<unix_ts>.json`.

### Failover gate

- Auto-failover requires: primary health-check fail + Patroni quorum + observability `held` SLO state cleared.
- Manual override: ops-sre-reliability + ops-security 2-person rule via OpenBao JIT.

## Cross-pack µservice handoff (NOT replication)

When Tenant-A (pack-kr) creates a task that triggers a workflow which creates a calendar event for an attendee in pack-eu:
1. Task created in pack-kr (Tenant-A's primary).
2. Workflow event emitted to workflow-engine via Workflow event-bus.
3. Workflow-engine determines cross-pack action; cross-pack call to calendar µservice in pack-eu requires SCC-gated invite grant per calendar's `policy/event-isolation.md` Invariant 5.
4. Cross-pack call refused by Cedar unless SCC on file; surface to tenant as "cross-pack action requires pre-approval".
5. Cross-pack mesh latency budget: 200ms p99; timeout 2s; on timeout return "cross-pack action pending".

Cross-pack mesh:
- mTLS between pack mesh-gateways; SPIFFE-identity verified.
- Network policy: only `tasks-task-store-app` may make cross-µservice handoff calls.
- Per-pack-pair rate limit: max 5k cross-pack req/s.

## Failover scenarios

| Scenario | Detection | Action |
|---|---|---|
| Primary Postgres outage (intra-AZ) | health-check fail | Patroni promotes synchronous replica; ≤ 5s |
| AZ outage | AZ network partition | Patroni promotes cross-AZ replica; ≤ 30s |
| Region outage | region partition | Patroni promotes DR-region replica; DNS update; ≤ 15 min; Meilisearch rebuild from Postgres ≤30min (degraded mode in meantime) |
| Pack-wide outage (cross-jurisdiction) | pack mesh down | NOT covered by failover; pack is residency boundary; tenant degraded until pack recovers; no cross-pack failover (data residency forbids) |
| Meilisearch cluster outage | cluster health-check | Degrade to direct-Postgres-trigram search; rebuild from Postgres after cluster recovery |
| Cross-µservice handoff partition | mesh health-check | Cross-pack action degrades to "pending"; tenant operational impact bounded |
| Backup corruption | weekly hash verification | Investigate; if corrupt, restore from prior backup |

## Geo-load-balancing

- Per-pack ingress: tenant API key pre-resolves to pack tag at OIDC issuance; ingress routes to per-pack cluster.
- Cross-tenant operations: explicit per-tenant pack tag in OIDC; ingress refuses cross-pack route.

## Latency budgets

| Path | p99 budget |
|---|---|
| Intra-pack task-fetch | ≤ 200ms |
| Intra-pack search query | ≤ 300ms |
| Intra-pack cross-µservice handoff (to mail / messenger / calendar / drive) | ≤ 400ms |
| Cross-pack µservice handoff | ≤ 700ms (cross-pack hop adds ≤ 200ms) |

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0140: Cedar policy.
- `policy/data-residency.md`, `policy/task-isolation.md`, `incident-response.md`, `failure-modes.md`, `capacity-model.md`.
- Patroni HA documentation.
- OCI region map (2026-05).
- `microservices/calendar/multi-region.md` — sibling reference template.

## Per-Pack Multi-Region Overlay Sections (2026-05-17 additive)

Per ADR-0133 11-pack-overlay program. Each overlay names the
authoritative cloud region(s), data-residency boundary, and the HA
topology specific to that pack.

### pack-kr (ap-seoul-1)

- **Primary region**: ap-seoul-1 (KR; on-shore per KR PIPA Art. 17).
- **HA topology**: 3 AZs within ap-seoul-1.
- **DR region**: ap-chuncheon-1 (warm-standby; tested quarterly).
- **Cross-pack route**: refused by default; KR PIPA Art. 17 SCC-equivalent required.
- **Employment-record retention floor**: 1095d per 근로기준법 Art. 41.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-eu (eu-frankfurt-1 + eu-amsterdam-1)

- **Primary region**: eu-frankfurt-1.
- **HA topology**: 3 AZs.
- **DR region**: eu-amsterdam-1 (active-passive; SCC-bound by GDPR Chapter V).
- **Cross-pack route**: SCC + supplementary measures per Schrems II.
- **EU AI Act**: T2 auto-assign in employment-context REFUSED at Cedar layer until ADR-TASKS-0006 conformity ADR ships.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-us (us-ashburn-1 + us-phoenix-1)

- **Primary region**: us-ashburn-1.
- **HA topology**: 3 AZs.
- **DR region**: us-phoenix-1.
- **Cross-pack route**: intra-US cross-region allowed; cross-pack to EU SCC-gated.
- **EEOC / Title VII / ADA**: T2 auto-assign in employment-context REFUSED at Cedar layer until fairness-audit complete.
- **NY Local Law 144 (AEDT)**: T2 refused for pack-us-NY until AEDT audit complete.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-us-healthcare (us-ashburn-1-hipaa)

- **Primary region**: us-ashburn-1 HIPAA-eligible zone.
- **HA topology**: 3 HIPAA-eligible AZs.
- **DR region**: us-phoenix-1 HIPAA-eligible (active-passive).
- **Cross-pack route**: forbidden by default; ePHI stays in HIPAA-eligible US zones.
- **Retention**: ≥6y per HIPAA 45 CFR §164.316.
- **BAA**: signed BAA in scope for every cloud provider.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-jp (ap-tokyo-1)

- **Primary region**: ap-tokyo-1.
- **HA topology**: 3 AZs.
- **DR region**: ap-osaka-1.
- **Cross-pack route**: APPI Art. 24 — adequate-country only.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/tasks-multi-region-overlay.md`.

(Pattern follows calendar's; pack-au + pack-in + pack-br + pack-ae + pack-ksa each carry their own employment-record retention floor per local Labour-Law.)
