---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-calendar + council-privacy
related_adrs: [ADR-0117, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Multi-Region — calendar µservice

## Purpose

Define the per-pack regional deployment topology, residency enforcement, cross-region replication policy, and disaster-recovery + failover model for calendar.

## Regional topology

### Pack-to-region mapping (canonical)

| Pack | Primary region | DR region (same jurisdiction) | Postgres cluster | Valkey cluster |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | OCI ap-chuncheon-1 | KR-primary | KR-primary |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | EU-primary | EU-primary |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | US-primary | US-primary |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | OCI us-phoenix-1 (BAA-eligible) | US-HIPAA-primary | US-HIPAA-primary |
| pack-jp | OCI ap-tokyo-1 | OCI ap-osaka-1 | JP-primary | JP-primary |
| pack-sg | OCI ap-singapore-1 | OCI ap-melbourne-1 (Asia-Pacific DR) | SG-primary | SG-primary |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | AU-primary | AU-primary |
| pack-in | OCI ap-mumbai-1 | OCI ap-hyderabad-1 | IN-primary | IN-primary |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | BR-primary | BR-primary |
| pack-ae | OCI me-dubai-1 | OCI me-jeddah-1 | AE-primary | AE-primary |
| pack-ksa | OCI me-jeddah-1 | OCI me-dubai-1 | KSA-primary | KSA-primary |

### Residency invariants

Per `policy/data-residency.md`:
- Each tenant pinned to exactly one pack at onboarding.
- Postgres + Valkey clusters pack-resident; no cross-pack replication by default.
- Cross-pack data flow only via SCC-gated cross-tenant availability projection (free/busy only).

## Cross-region replication policy

### Within-pack replication (DR)

- Postgres: synchronous replication primary → first replica (intra-AZ), async to off-AZ replica + DR-region replica.
- Replication factor: 3 (primary + 2 replicas).
- Replication lag SLO: ≤ 5s within AZ; ≤ 30s cross-AZ; ≤ 60s cross-region (same jurisdiction).

### Cross-pack replication

- **FORBIDDEN by default.**
- LEAN check `oya-check-cross-pack-replication-prohibition` refuses build if any config introduces cross-pack replication.
- Cross-pack data flow happens at access-time only (cross-tenant projection); never at-rest.

### Backup + cold storage

- Per-pack S3-compatible cold-tier (OCI Object Storage); WORM where supported.
- Backup retention: 30d hot + 12mo cold; pack-us-healthcare ≥ 6y.
- Backup encryption: pack-resident KMS keys; cross-pack restore blocked at KMS level.
- Backup integrity: weekly hash verification + monthly restore-test drill.

## Disaster recovery

### RTO + RPO

- RTO ≤ 15 min (production tier).
- RPO ≤ 60s (within-pack same-jurisdiction).
- DR-region failover: automated via Patroni leader election + DNS update.
- Failover steps:
  1. Primary fails health-check; Patroni elects new primary in DR region.
  2. DNS TTL ≤ 30s; clients re-resolve within 30-60s.
  3. Worker pods drain pending writes; resume against new primary.
  4. Audit-chain emission for failover event.

### DR drill cadence

- Quarterly: simulated primary-region outage; failover to DR region; validate audit-chain seal continuity + Postgres WAL replay completeness.
- Annual: full-pack restore-from-cold (PITR + WAL replay).
- DR-drill evidence committed at `evidence/dr-drills/<pack>-<unix_ts>.json`.

### Failover gate

- Auto-failover requires: primary health-check fail + Patroni quorum + observability `held` SLO state cleared.
- Manual override: ops-sre-reliability + ops-security 2-person rule via OpenBao JIT.

## Cross-tenant availability across packs

When Tenant-A (pack-kr) needs availability of Tenant-B's attendee (pack-eu):
1. Cross-tenant invite grant on file (Cedar policy `cross-tenant-grant`).
2. Tenant-A's availability-resolver issues mTLS call to pack-eu availability-resolver.
3. pack-eu resolver returns `FreeBusyProjection` only (Invariant 4 of `event-isolation.md`).
4. Projection cached in pack-kr Valkey with TTL ≤ 60s + jitter; cache invalidates on grant revocation.
5. Cross-pack mesh latency budget: 100ms p99; timeout 2s; on timeout return "unknown" projection.

Cross-pack mesh:
- mTLS between pack mesh-gateways; SPIFFE-identity verified.
- Network policy: only availability-resolver pods may make cross-pack calls.
- Per-pack-pair rate limit: max 10k cross-pack req/s.

## Cross-pack invitation flow

When Tenant-A (pack-kr) invites an external attendee in Tenant-B (pack-eu):
1. Event stored in pack-kr (Tenant-A's primary).
2. Invitation dispatched via `mail` µservice; mail µservice handles cross-pack delivery per its own multi-region policy.
3. Attendee's RSVP record stored in pack-eu (Tenant-B's primary).
4. Cross-pack RSVP correlation: pack-kr event refers to pack-eu RSVP via foreign-key; deref at access time only.

## Failover scenarios

| Scenario | Detection | Action |
|---|---|---|
| Primary Postgres outage (intra-AZ) | health-check fail | Patroni promotes synchronous replica; ≤ 5s |
| AZ outage | AZ network partition | Patroni promotes cross-AZ replica; ≤ 30s |
| Region outage | region partition | Patroni promotes DR-region replica; DNS update; ≤ 15 min |
| Pack-wide outage (cross-jurisdiction) | pack mesh down | NOT covered by failover; pack is residency boundary; tenant degraded until pack recovers; no cross-pack failover (data residency forbids) |
| Cross-pack mesh partition | mesh health-check | Cross-tenant availability degrades to "unknown"; tenant operational impact bounded |
| Backup corruption | weekly hash verification | Investigate; if corrupt, restore from prior backup |

## Geo-load-balancing

- Per-pack ingress: tenant API key pre-resolves to pack tag at OIDC issuance; ingress routes to per-pack cluster.
- Cross-tenant operations: explicit per-tenant pack tag in OIDC; ingress refuses cross-pack route.

## Latency budgets

| Path | p99 budget |
|---|---|
| Intra-pack event-fetch | ≤ 200ms |
| Intra-pack availability lookup | ≤ 500ms |
| Cross-pack availability lookup | ≤ 700ms (cross-pack hop adds ≤ 200ms) |
| Cross-pack invitation dispatch | ≤ 1s |

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0140: Cedar policy.
- `policy/data-residency.md`, `policy/event-isolation.md`, `incident-response.md`, `failure-modes.md`, `capacity-model.md`.
- Patroni HA documentation.
- OCI region map (2026-05).

## Per-Pack Multi-Region Overlay Sections (2026-05-17 additive)

Per ADR-0133 11-pack-overlay program. Each overlay names the
authoritative cloud region(s), data-residency boundary, and the
HA topology specific to that pack.

### pack-kr (ap-seoul-1)

- **Primary region**: ap-seoul-1 (KR; on-shore per KR PIPA Art. 17).
- **HA topology**: 3 AZs within ap-seoul-1; sync replicas ≥1; streaming replica 2.
- **DR region**: ap-seoul-2 (warm-standby; tested quarterly).
- **Cross-pack route**: refused by default; KR PIPA Art. 17 SCC-equivalent required.
- **RTO**: ≤15 min; **RPO**: ≤60s (Postgres logical replication).

### pack-eu (eu-frankfurt-1 + eu-paris-1)

- **Primary region**: eu-frankfurt-1.
- **HA topology**: 3 AZs within eu-frankfurt-1.
- **DR region**: eu-paris-1 (active-passive; SCC-bound by GDPR Chapter V).
- **Cross-pack route**: SCC + supplementary measures required per Schrems II.
- **EU AI Act**: per `capabilities/T1-assist.yaml` Cedar refusal of HR-context T1/T2 in this pack.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-us (us-east-1 + us-west-2)

- **Primary region**: us-east-1.
- **HA topology**: 3 AZs within us-east-1.
- **DR region**: us-west-2 (active-passive).
- **Cross-pack route**: intra-US cross-region allowed; cross-pack to EU SCC-gated.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-us-healthcare (us-east-1-hipaa)

- **Primary region**: us-east-1 HIPAA-eligible zone.
- **HA topology**: 3 HIPAA-eligible AZs.
- **DR region**: us-west-2 HIPAA-eligible (active-passive).
- **Cross-pack route**: forbidden by default; ePHI must remain in HIPAA-eligible US zones.
- **CalDAV backend**: SabreDAV (per ADR-CAL-0001); deployed within HIPAA-eligible zone only.
- **RTO**: ≤15 min; **RPO**: ≤60s.
- **BAA**: every cloud provider in scope has a signed BAA.

### pack-jp (ap-tokyo-1)

- **Primary region**: ap-tokyo-1.
- **HA topology**: 3 AZs.
- **DR region**: ap-osaka-1 (warm-standby).
- **Cross-pack route**: APPI Art. 24 — adequate-country only.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-sg (ap-singapore-1)

- **Primary region**: ap-singapore-1.
- **HA topology**: 3 AZs.
- **DR region**: ap-jakarta-1 (warm-standby).
- **Cross-pack route**: PDPA §26 comparable-protection.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-au (ap-sydney-1)

- **Primary region**: ap-sydney-1.
- **HA topology**: 3 AZs.
- **DR region**: ap-melbourne-1 (warm-standby).
- **Cross-pack route**: APP 8 accountability.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-in (ap-mumbai-1)

- **Primary region**: ap-mumbai-1.
- **HA topology**: 3 AZs.
- **DR region**: ap-hyderabad-1 (warm-standby; same-country per DPDPA).
- **Cross-pack route**: DPDPA §16 whitelist.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-br (sa-saopaulo-1)

- **Primary region**: sa-saopaulo-1.
- **HA topology**: 3 AZs.
- **DR region**: sa-rio-1 (warm-standby).
- **Cross-pack route**: LGPD Art. 33 ANPD-approved.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-ae (me-dubai-1)

- **Primary region**: me-dubai-1.
- **HA topology**: 3 AZs.
- **DR region**: me-abudhabi-1 (warm-standby).
- **Cross-pack route**: PDPL Art. 22 UAE DPA approval.
- **Hijri/Gregorian dual-calendar rendering**: both per ICU4X overlay.
- **RTO**: ≤15 min; **RPO**: ≤60s.

### pack-ksa (me-riyadh-1)

- **Primary region**: me-riyadh-1.
- **HA topology**: 3 AZs.
- **DR region**: me-jeddah-1 (warm-standby; same-country).
- **Cross-pack route**: PDPL Art. 29 SDAIA-approved.
- **Hijri/Gregorian dual-calendar**: per ICU4X overlay.
- **Sharia retention extension**: per-tenant.
- **RTO**: ≤15 min; **RPO**: ≤60s.
