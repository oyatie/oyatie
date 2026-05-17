---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-workflow, council-architecture, council-privacy
related_adrs: [ADR-0065, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/policy/data-residency.md
  - microservices/workflow-studio/capacity-model.md
  - microservices/workflow-studio/cost-budget.md
  - microservices/workflow-studio/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (workflow-studio µservice)

## Purpose

Define the multi-region topology for Studio across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden, BCDR posture, RPO/RTO per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying BC claims.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (single-region) | YES (M03 launch) |
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
┌─ Pack <X> ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Primary region                          DR-pair region                    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ visual-canvas-rest       │            │ visual-canvas-rest (warm)│      │
│  │ + node-library-rest      │   replic   │ + node-library-rest      │      │
│  │  - HPA min 2, max 50     │ ◀────────▶ │  - 0.6× capacity         │      │
│  └──────────────────────────┘   intra-   └──────────────────────────┘      │
│  ┌──────────────────────────┐   pack     ┌──────────────────────────┐      │
│  │ collab-crdt-worker       │            │ collab-crdt-worker (warm)│      │
│  │  - HPA min 3, max 100    │            │  - 0.6× capacity         │      │
│  │  - WS lease in Redis     │            │  - cold; warmed at FO    │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ Postgres + Citus (active) │           │ Postgres replica (warm)  │      │
│  │  - HA primary+standby     │           │  - streaming + slot      │      │
│  │  - RF=3 within region     │           │  - lag ≤ 30s              │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ Redis Sentinel (active)  │            │ Redis Sentinel (warm)    │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│                                                                            │
│  Global CDN (OCI; per-pack edge PoPs)                                      │
│  - WASM bundle + node library descriptors + spec schema                    │
│  - Per-pack edge keys                                                      │
│  - Global purge on release                                                 │
│                                                                            │
│  Global Traffic Manager (per-pack DNS):                                    │
│  - Health check on Studio editor REST + WS gateway                         │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                         │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres + Citus | Streaming replication (sync on coordinator; async on workers) | ≤ 5s writes; ≤ 30s bulk | intra-pack only |
| Redis Sentinel | Async cross-AZ within region; cross-region on demand at failover | CRDT state regenerable from Postgres | intra-pack only |
| WebSocket sessions | Ephemeral; client auto-reconnects after region failover | 0 (sessions re-established) | intra-pack only |
| Per-seat license attribution | Postgres-native; same as Postgres replication | ≤ 5s | intra-pack only |
| LLM-assist prompts + completions | Postgres-native | ≤ 5s | intra-pack only |
| Audit-chain seals | Postgres-replicated | ≤ 5s | intra-pack only |
| Node library descriptors + signatures | Git-versioned + CDN-distributed | 0 (declarative) | **global** (tenant-agnostic) |
| WASM bundles + design-system primitives | CDN-distributed | 0 (declarative; release-versioned) | **global** (tenant-agnostic) |
| Cedar policies | Git-versioned | 0 | **global** |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no editor session state, draft contents, CRDT state, LLM-assist prompts, or per-seat license attribution cross pack boundaries. Narrow exceptions (tenant-executed SCC for GDPR; tenant-specific BCDR exercise) documented inline. **EU-resident tenant editor state never reaches a non-EU region without Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: Studio editor REST + WS gateway + Postgres `request_failures_total > threshold` for ≥ 5min; or AZ-level OCI outage.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected; await OCI recovery.
5. If region-level + DR pair: initiate DR failover (Step §"DR Failover").
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region healthy (Postgres replica current; Redis Sentinel reachable; WS gateway warm) | ≤ 2 min |
| 2 | Promote Postgres replica → primary (cuts off old primary writes) | ≤ 5 min |
| 3 | Promote Redis Sentinel quorum → DR-pair | ≤ 2 min |
| 4 | Update Global Traffic Manager: DNS → DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 5 | Scale WS gateway in DR-pair from 0.6× to 1.0× primary capacity (HPA) | ≤ 10 min |
| 6 | Browser clients auto-reconnect to DR-pair endpoints; resume from local edit buffer | ≤ 5 min |
| 7 | Verify two-channel corroboration (Postgres write + Mimir paging working) | ≤ 2 min |
| 8 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 9 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 5s (Postgres synchronous replication); ≤ 30s (worker tier).
RTO: ≤ 35 min (DR failover complete; tenant traffic stable on DR-pair).

### Failback

Failback manual + scheduled (not auto-failback). Primary region must demonstrate ≥ 6h healthy state before failback initiated.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (off-hours) | Quarterly per pack with DR pair | Full failover + failback | ops-sre-reliability |
| Cross-region restore drill | Monthly | Postgres snapshot-restore + integrity check on subset | ops-sre-reliability |
| Tabletop exercise | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |
| Editor durability drill (kill WS gateway mid-session; verify resume) | Per-release | Per-pack Studio cluster | axis-workflow |
| Round-trip byte-equality drill across DR failover | Per-release | Per-pack Studio cluster | axis-workflow |
| DORA testing (pack-eu financial-services) | Annually | Full BCDR + recovery time validation | ops-sre-reliability + ops-compliance |

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

Per-tenant RPO/RTO commitments in tenant SLA (`legal/dpa-template.md`).

## Tenant Notification

Tenants notified at failover initiation per `incident-response.md`:

- **Status page (public)**: updated within 5 min of failover initiation.
- **Tenant operator email**: within 30 min for Sev-1/2 affecting a tenant's pack.
- **In-editor banner**: "Editor failed over to DR region. Please reload to resume." rendered when client reconnects to DR-pair endpoint.

## Long-Running Editor Session Resumption

The Studio's local edit buffer + Postgres-backed save discipline guarantees that mid-edit sessions survive region failover IF:
1. The Postgres replica is current at failover moment (RPO ≤ 5s satisfied).
2. The DR-pair WS gateway warms up within RTO window.
3. The browser client supports auto-reconnect (Leptos client + WS reconnect logic).

Verified by:
- Integration test `tests/e2e/editor-session-failover.rs` — open editor session; force failover; verify resume with no draft loss.
- Quarterly DR drill explicitly exercises editor-session resumption.

## Per-Pack BCDR Overlay

Per-pack BCDR specifics live at `regional-packs/<pack>/workflow-studio-multi-region-overlay.md`. Example: pack-eu satisfies DORA testing when oyatie has EU financial-services tenants in scope.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice workflow-studio` — exit 0.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- `microservices/workflow-studio/policy/data-residency.md`.
- `microservices/workflow-studio/capacity-model.md`.
- `microservices/workflow-studio/cost-budget.md`.
- `microservices/workflow-studio/failure-modes.md`.
- `microservices/workflow-studio/incident-response.md`.
- `regional-packs/<pack>/workflow-studio-multi-region-overlay.md`.
- OCI region documentation.
- Postgres + Citus replication — `docs.citusdata.com/`.
- Redis Sentinel — `redis.io/topics/sentinel`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.
