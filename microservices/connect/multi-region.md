---
microservice: connect
doc_class: MultiRegion
date: 2026-05-20
owner_team: axis-integration + ops-sre-reliability
status: Accepted
related_adrs: [ADR-0248, ADR-0253]
doc_status: published
---

# Multi-Region — connect

Per ADR-0248 cellular architecture. Per-pack region layout; cross-pack movement forbidden by default.

## Pack → region map

| Pack | Primary region(s) | DR pair |
|---|---|---|
| pack-kr | OCI ap-seoul-1 | ap-tokyo-1 (read-replica only; KR data does NOT cross border for storage) |
| pack-eu | OCI eu-frankfurt-1 | eu-amsterdam-1 |
| pack-us | OCI us-ashburn-1 | us-phoenix-1 |
| pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | isolated DR |
| pack-jp | ap-tokyo-1 | ap-osaka-1 |
| pack-sg | ap-singapore-1 | ap-melbourne-1 (read-replica) |
| pack-au | ap-sydney-1 | ap-melbourne-1 |
| pack-in | ap-hyderabad-1 | ap-mumbai-1 |
| pack-br | sa-saopaulo-1 | sa-vinhedo-1 |
| pack-ae | me-abudhabi-1 | me-dubai-1 |
| pack-ksa | me-jeddah-1 | me-riyadh-1 |
| pack-cn | cn-shanghai-1 | cn-beijing-1 (Alibaba Cloud; isolated) |

## Failure modes by region scope

| Mode | Detection | Behavior |
|---|---|---|
| Single AZ outage | OCI status + internal liveness | Auto-failover within region; <30s RTO |
| Full region outage | OCI region status + multi-source health | Failover to DR pair within pack; <15min RTO; <60s RPO |
| Pack-wide failure | All regions in pack unreachable | Tenant API returns 503 with `Retry-After`; webhook 503 → vendor retries per their policy |
| Vendor (external SaaS) outage | Per-connector circuit-breaker | Circuit opens; tenant dashboards surface; DLQ accumulates |

## Cross-pack movement policy

**Forbidden by default.** A tenant in pack-kr cannot have its OAuth grants stored in pack-eu. Exceptions require:
- Tenant attestation (signed by tenant admin via WebAuthn).
- Compliance pack approval (e.g., EU+US Schrems II transfer mechanism).
- ops-legal escalation review.

Audit event: `CrossPackTransferRequested` + `CrossPackTransferApproved` (or `Denied`).

## Cell tier assignments

- Tier-0 (edge): connector-catalog-rest, oauth-broker-rest, webhook-receiver-edge — internet-facing.
- Tier-1 (app): connector-adapter-worker, dlq-replay-worker — application logic.
- Tier-2 (data): Postgres + Valkey + OpenBao (in cloud-secrets µservice) — data plane.
- Tier-3 (air-gap): not used (substrate needs internet egress to call vendors).

## Cell-shard-width (per ADR-0248)

Per pack, per tier:
- Tier-0: N=8 cells per region (shuffle-sharded by tenant_id; M=64 candidates).
- Tier-1: N=16 worker pools per region (HPA per-pool).
- Tier-2: managed by cloud-data µservice; connect consumes via Cedar-gated client.

## References

- ADR-0248 cellular architecture (cell tier definitions, shuffle-sharding math)
- ADR-0117 jurisdiction-code routing
- `microservices/connect/policy/data-residency.md`
