---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-workflow, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/policy/data-residency.md
  - microservices/workflow-engine/capacity-model.md
  - microservices/workflow-engine/cost-budget.md
  - microservices/workflow-engine/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (workflow-engine µservice)

## Purpose

Define the multi-region topology for workflow-engine across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden, BCDR posture, RPO/RTO per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying BC claims.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (single-region; geographic constraint) | YES (M02b launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated from pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture

For packs with a DR pair:

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Engine workers (active)  │            │ Engine workers (warm)    │    │
│  │  - HPA min 3, max 200    │   replic   │  - 0.6× capacity         │    │
│  │  - lease state in Redis  │ ◀────────▶ │  - cold; warmed at FO    │    │
│  └──────────────────────────┘   intra-   └──────────────────────────┘    │
│  ┌──────────────────────────┐   pack     ┌──────────────────────────┐    │
│  │ Postgres + Citus (active) │           │ Postgres replica (warm)  │    │
│  │  - HA primary+standby     │           │  - streaming + slot      │    │
│  │  - RF=3 within region     │           │  - lag ≤ 30s              │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Redis Sentinel (active)  │            │ Redis Sentinel (warm)    │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ ClickHouse (active)      │            │ ClickHouse (warm)        │    │
│  │  - async repl from PG    │            │  - separate repl stream  │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on engine workers + Postgres write path                  │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres + Citus | Streaming replication (synchronous on coordinator; async on workers) | ≤ 5s on writes; ≤ 30s on bulk | intra-pack only |
| Redis Sentinel | Async cross-AZ within region; cross-region on demand at failover | Lease state regenerable from Postgres on cold-start | intra-pack only |
| ClickHouse | Async replication (replicated tables) | ≤ 30s | intra-pack only |
| Outbox event log | Postgres-native; same as Postgres replication | ≤ 5s | intra-pack only |
| Spec versions | Postgres-replicated + git-versioned | 0 (declarative) | intra-pack only |
| Cedar policies | Git-versioned | 0 | global |
| Workflow event registry | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no run state or event data crosses pack boundaries. Narrow exceptions (tenant-executed SCC for GDPR transfers; tenant-specific BCDR exercise) documented inline. **EU-resident tenant data never reaches a non-EU region without Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: engine workers + Postgres + Redis `request_failures_total > threshold` for ≥ 5min; or AZ-level OCI outage.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected; await OCI recovery (see `failure-modes.md` FM-05).
5. If region-level + DR pair: initiate DR failover (Step §"DR Failover").
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region healthy (Postgres replica caught up; Redis Sentinel reachable; engine workers warm) | ≤ 2 min |
| 2 | Promote Postgres replica → primary (Citus coordinator + workers; cuts off old primary writes) | ≤ 5 min |
| 3 | Promote Redis Sentinel quorum → DR-pair | ≤ 2 min |
| 4 | Update Global Traffic Manager: DNS → DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 5 | Scale engine workers in DR-pair from 0.6× to 1.0× primary capacity (HPA) | ≤ 10 min |
| 6 | Resume in-flight runs from Postgres state in DR-pair; rate-limited replay storm cadence | ≤ 10 min |
| 7 | Verify two-channel corroboration (Postgres write + Mimir/observability paging both working) | ≤ 2 min |
| 8 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 9 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 5s (Postgres synchronous replication on coordinator); ≤ 30s (worker tier).
RTO: ≤ 35 min (DR failover complete; tenant traffic stable on DR-pair).

### Failback

Failback is manual + scheduled (not auto-failback). Primary region must demonstrate ≥ 6h healthy state before failback initiated.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (off-hours) | Quarterly per pack with DR pair | Full failover + failback | ops-sre-reliability |
| Cross-region restore drill | Monthly | Postgres + ClickHouse snapshot-restore + integrity check on a subset | ops-sre-reliability |
| Tabletop exercise | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |
| Engine durability drill (kill engine mid-run; verify resume) | Per-release | Per-pack engine cluster | axis-workflow |
| Replay storm drill (cold-start all in-flight runs) | Per-release | Per-pack engine cluster | axis-workflow |
| DORA testing (pack-eu when financial-services tenants in scope) | Annually | Full BCDR + recovery time validation | ops-sre-reliability + ops-compliance |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 5s | ≤ 4h (depends on OCI ap-seoul-1 recovery; 3 AZs) | best-effort; OCI SLA |
| pack-eu | ≤ 5s | ≤ 35 min | – |
| pack-us | ≤ 5s | ≤ 35 min | – |
| pack-us-healthcare | ≤ 5s | ≤ 35 min | – |
| pack-jp | ≤ 5s | ≤ 4h | best-effort |
| pack-sg | ≤ 5s | ≤ 4h | best-effort |
| pack-au | ≤ 5s | ≤ 35 min | – |
| pack-in | ≤ 5s | ≤ 35 min | – |
| pack-br | ≤ 5s | ≤ 35 min | – |
| pack-ae | ≤ 5s | ≤ 35 min | – |
| pack-ksa | ≤ 5s | ≤ 35 min | – |

Per-tenant RPO/RTO commitments are in tenant SLA (`legal/dpa-template.md`). Packs without DR pair (pack-kr, pack-jp, pack-sg) have weaker RTO disclosed at tenant onboarding.

## Tenant Notification

Tenants notified at failover initiation per `incident-response.md` §"Tenant communications":

- **Status page (public)**: updated within 5 min of failover initiation.
- **Tenant operator email**: within 30 min for Sev-1/2 affecting a tenant's pack.
- **Customer-facing message template**: provided in tenant operator's onboarding portal.
- **Regulatory notification**: per `compliance.md` enforced timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).

## Long-Running Workflow Resumption

The engine's durable-execution invariant guarantees that long-running runs (paused-in-place up to 90 days) survive region failover IF:
1. The Postgres replica is current at failover moment (RPO ≤ 5s satisfied).
2. The DR-pair engine workers warm up within RTO window.
3. The deterministic-replay invariant produces identical step sequence on the replicated state.

Verified by:
- Integration test `tests/e2e/long-running-workflow-failover.rs` — start a 24h-paused workflow; force failover; verify completion against the same expected sequence.
- Quarterly DR drill explicitly exercises long-running workflow resumption with synthetic 24h-paused runs.

## Per-Pack BCDR Overlay

Per-pack BCDR specifics live at `regional-packs/<pack>/multi-region-overlay.md`. Example: pack-eu must satisfy DORA testing requirements when oyatie has EU financial-services tenants in scope.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance` — exit 0; deployed topology matches this document for every active pack.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/workflow-engine/policy/data-residency.md`.
- `microservices/workflow-engine/capacity-model.md`.
- `microservices/workflow-engine/cost-budget.md`.
- `microservices/workflow-engine/failure-modes.md`.
- `microservices/workflow-engine/incident-response.md`.
- `regional-packs/<pack>/multi-region-overlay.md`.
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Postgres + Citus replication — `docs.citusdata.com/`.
- ClickHouse replicated tables — `clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication`.
- Redis Sentinel — `redis.io/topics/sentinel`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.
