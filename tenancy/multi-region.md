---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-tenancy + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-tenancy, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/tenancy/policy/data-residency.md
  - microservices/tenancy/capacity-model.md
  - microservices/tenancy/cost-budget.md
  - microservices/tenancy/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (tenancy µservice)

## Purpose

Define the multi-region topology for tenancy across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, Patroni HA + Citus topology, RPO/RTO targets per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (single-region) | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional (KSA NCA cloud-residency requirements) |

## In-Pack DR-Pair Architecture (tenancy specifics)

For packs with a DR pair:

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres (Patroni HA)    │            │ Postgres (Patroni warm)  │    │
│  │  - primary               │   stream   │  - async replica         │    │
│  │  - sync replica × 2      │ ◀────────▶ │  - standby cluster       │    │
│  │  - async replica × N     │   repli-   │  - separate Patroni DCS  │    │
│  │  - WAL archive (S3)      │   cation   │  - cross-region S3 CRR   │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Citus coordinator + N    │            │ Citus warm-standby       │    │
│  │ workers                  │            │  - same shape; 0.6× cap  │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Valkey (3-node Sentinel) │            │ Valkey replica (warm)    │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ tenancy crates (rest +   │            │ tenancy crates (warm)    │    │
│  │ worker + app pods)       │            │ replica set scale=0      │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on primary's Postgres + Citus + tenancy-rest             │
│  - On failure: DNS failover → DR pair (TTL 60s)                          │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres WAL | Streaming (sync within-region; async cross-region) | ≤ 1s sync; ≤ 30s async | intra-pack only |
| Postgres data + WAL archive | S3 CRR (cross-region intra-pack) | ≤ 5 min | intra-pack only |
| Citus shards | Logical replication on shard moves; transactional cut-over | ≤ 30s within-region; ≤ 5min cross-region | intra-pack only |
| Patroni DCS (etcd) | Multi-region etcd cluster within-pack | ≤ 5s | intra-pack only |
| Valkey | Sentinel-managed; warm replica cross-region | ≤ 30s | intra-pack only |
| Tenancy crates' deployment | Git-versioned + ArgoCD-managed on both sides | 0 (declarative) | intra-pack only |
| RLS YAML manifests | Git-versioned | 0 | global (manifests are config; not tenant data) |
| Cedar policy fragments | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no tenant metadata crosses pack boundaries. The narrow exceptions (tenant-executed SCCs for GDPR transfers; tenant-specific BCDR exercise) are documented inline. **EU-resident tenant metadata never reaches a non-EU region without a Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: Postgres + Citus + tenancy-rest `request_failures_total > threshold` for ≥ 5min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected components; await OCI recovery (see `failure-modes.md` FM-01).
5. If region-level + DR pair exists: initiate DR failover (Step §"DR Failover" below).
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region healthy (Postgres reachable; Citus warm; tenancy crates buildable) | ≤ 2 min |
| 2 | Promote DR-pair Postgres replica to primary via Patroni (`patronictl failover`) | ≤ 30 s |
| 3 | Promote DR-pair Citus coordinator (Patroni-managed; replica → primary) | ≤ 30 s |
| 4 | Scale DR-pair tenancy crates from 0 to primary-tier replica count | ≤ 3 min (Helm rollout) |
| 5 | Update Global Traffic Manager DNS records to DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 6 | Verify validate hot path reachable from DR-pair; per-µservice JWT-verifier caches refresh | ≤ 5 min |
| 7 | Verify lifecycle write path resumes; Citus coordinator accepts writes | ≤ 5 min |
| 8 | Verify Valkey cache warm; cache-hit rate climbs (cold start) | ≤ 5 min |
| 9 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 10 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 5 min (sync replicas absorb within-region writes; cross-region async has ≤ 5min lag).
RTO: ≤ 35 min (DR failover complete; tenant traffic stable on DR-pair).

### Failback (after primary region recovers)

Failback to primary is **manual** and scheduled. The primary region must demonstrate ≥ 6h of healthy state before failback initiated. Procedure mirrors DR Failover steps in reverse, with the warm-standby (now primary) becoming warm again.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack at a time | ops-sre-reliability |
| Cross-region restore drill (Postgres + WAL replay → DR-pair → integrity check) | Monthly | Snapshot-restore + integrity check on a tenant-id subset | ops-sre-reliability |
| Patroni failover drill (single primary-loss simulation) | Monthly per pack | Within-pack Patroni primary loss; verify ≤ 10s recovery | ops-sre-reliability |
| Citus rebalance drill | Quarterly | Synthetic shard rebalance; verify row checksums | axis-tenancy |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection (random pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 1 s (sync within-region) | ≤ 4 h (OCI region recovery; ap-seoul-1 has 3 AZs) | best-effort; OCI region SLA |
| pack-eu | ≤ 5 min (CRR async) | ≤ 35 min (DR failover) | – |
| pack-us | ≤ 5 min | ≤ 35 min | – |
| pack-us-healthcare | ≤ 5 min | ≤ 35 min | – |
| pack-jp | ≤ 1 s (no DR pair) | ≤ 4 h | best-effort |
| pack-sg | ≤ 1 s (no DR pair) | ≤ 4 h | best-effort |
| pack-au | ≤ 5 min | ≤ 35 min | – |
| pack-in | ≤ 5 min | ≤ 35 min | – |
| pack-br | ≤ 5 min | ≤ 35 min | – |
| pack-ae | ≤ 5 min | ≤ 35 min | – |
| pack-ksa | ≤ 5 min | ≤ 35 min | – |

Per-tenant RPO/RTO commitments are part of the tenant SLA (per DPA at `legal/dpa-template.md`). Packs without DR pair (pack-kr, pack-jp, pack-sg) have weaker RTO disclosed at onboarding.

## Tenant Notification

Per `incident-response.md` §"Tenant communications":

- **Status page (public)**: updated within 5 min of failover initiation.
- **Tenant operator email**: sent within 30 min for any Sev-1/2 affecting tenant's pack.
- **Customer-facing message-template**: provided in tenant operator's portal.
- **Regulatory notification**: per `compliance.md` enforced timelines (GDPR 72h; HIPAA 60d; KR PIPA 72h; etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `regional-packs/<pack>/tenancy-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing requirements when oyatie has EU financial-services tenants in scope; pack-kr financial-services tenants engage KR-FSS BCDR guidance with stricter RTO.

## Verification

- `cargo run -p dev-cli -- gate validate multi-region-conformance --microservice tenancy` — exit 0.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/tenancy/policy/data-residency.md`.
- `microservices/tenancy/capacity-model.md`.
- `microservices/tenancy/cost-budget.md`.
- `microservices/tenancy/failure-modes.md`.
- `microservices/tenancy/incident-response.md`.
- `regional-packs/<pack>/tenancy-multi-region-overlay.md`.
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Patroni HA documentation — `patroni.readthedocs.io`.
- Citus operational guide — `docs.citusdata.com`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.

---

## ADR-0158 Multi-Region Disposition Statement

**Disposition: `active_active` (global tenant registry replicated).**

Per ADR-0158, the tenancy µservice is declared `active_active`. The tenant-registry (`tenant_id → home_region + allowed_regions + residency_class + pack_id`) is a global table replicated via Patroni cross-region async (~5 second lag). DNS anycast points the api-gateway tier (ADR-0157) at the nearest replica for routing decisions.

| Property | Value |
|---|---|
| Disposition | `active_active` |
| RPO (cross-region) | ≤ 5 seconds (async replication lag) |
| RTO (intra-region) | ≤ 60 seconds |
| Sovereign-pin behavior | sovereign tenants pinned to their cell at the api-gateway routing layer |
| Convergence model | last-writer-wins on tenant-config updates; conflicts resolved by `updated_at` |
| Cross-region transaction policy | forbidden |

This µservice IS the global control plane for ADR-0158 sovereign-tenant routing. Its tenant-registry replica in every cell lets the local api-gateway tier reject mismatched cells at edge.

## ADR-0163 Per-Tenant Environment Tiers Statement

Per ADR-0163, every tenant has three environment tiers — `test`, `staging`, `prod` — cell-isolated; API keys prefix-tagged.

**Tier definitions:**
- **`test`** — sandbox. 90-day TTL default. Outbound side effects intercepted + logged, not delivered. API keys `sk_test_` / `pk_test_`.
- **`staging`** — pre-production. Durable. Outbound to test recipients only. API keys `sk_stage_` / `pk_stage_`. No prod-data copy without ChangeSet approval.
- **`prod`** — production. Durable + residency-bound. Outbound live. API keys `sk_live_` / `pk_live_`. Destructive ops require admin acknowledgment.

**Isolation contract:**
- Separate PostgreSQL schemas per tier within the cell's PG cluster; RLS enforced.
- api-gateway tier (ADR-0157) reads API-key prefix; routes to env-tier-specific workload pool. `sk_test_` request never reaches `prod` schema.
- Audit-chain per-tier subtree (audit-chain µservice partitions by `(tenant_id, env_tier)` per ADR-0162).

**API-key issuance Cedar gates:**
- `sk_test_` issuable by tenant developer or higher.
- `sk_stage_` issuable by tenant maintainer or higher.
- `sk_live_` issuable by tenant admin only.

**Destructive-operation acknowledgment (prod tier only):**
- Cedar condition `prod_destructive_acknowledged: true`.
- Request header `x-prod-destructive-ack: true`.
- UI prompt before send.
- Audit-chain seal captures (who, when, what).

**Operations covered:** DSR delete, tenant offboarding, bulk delete > 100 rows, cell migration, residency-class change.

CI lane `oya gate validate tenant-environment-tier` enforces (a) every outbound-effect µservice checks `env_tier`, (b) every API-key issuance validates Cedar tier-grant, (c) every prod destructive op carries the ack header.

See `/specs/tenant-environment-tiers-canonical.json` for the canonical declaration.
