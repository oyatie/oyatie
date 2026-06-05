---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-audit-chain + cloud-iac + cloud-secrets
deciders: ops-sre-reliability, axis-audit-chain, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/audit-chain/policy/data-residency.md
  - microservices/audit-chain/capacity-model.md
  - microservices/audit-chain/cost-budget.md
  - microservices/audit-chain/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (audit-chain µservice)

## Purpose

Topology + BCDR posture for audit-chain across the 11 oyatie packs. **Key invariant: cross-pack replication is strictly forbidden for cryptographic continuity** (each pack runs its own chain with its own HSM partition).

This document is the operational reference for ops-sre-reliability during region outages and for auditors verifying audit-chain BCDR claims.

## Topology Per Pack

(Identical pack assignment as observability + tenancy; see `policy/data-residency.md`.)

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (geographic constraint) | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | us-phoenix-1 (HIPAA-eligible) | DR pair | Conditional |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture (where applicable)

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Primary region                          DR-pair region                    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ emission-rest (active)   │            │ emission-rest (warm)     │      │
│  │  HA replicas             │            │  0.6× replicas           │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ Postgres (active)         │   sync     │ Postgres (replica;       │      │
│  │  primary + replica intra  │ replicate  │  hot-standby)            │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ S3 WORM bucket            │  intra-    │ S3 WORM bucket (replicated│      │
│  │ (Object Lock Compliance)  │  pack CRR  │  intra-pack)             │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ HSM partition (active)    │   intra-   │ HSM partition (active for│      │
│  │                            │   pack    │  failover; key material   │      │
│  │                            │   key-    │  shared via OCI HSM      │      │
│  │                            │   replicate│  partition replication)  │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ sealing-worker (active    │            │ sealing-worker (warm     │      │
│  │  per-shard leaders)        │            │  standby; ready to       │      │
│  │                            │            │  promote)                │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│                                                                            │
│  Global Traffic Manager (per-pack DNS):                                    │
│  - Health check on primary's emission-rest                                 │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                          │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres index | Streaming replication primary→replica + cross-region CRR | ≤ 30s | intra-pack only |
| S3 WORM blobs | Async S3 cross-region replication intra-pack | ≤ 5min | intra-pack only |
| HSM key material | OCI Cloud-HSM partition replication intra-pack | ≤ 5min | intra-pack only (per Oracle HSM partition spec) |
| Recording rules + Cedar policies | Git-versioned; deployed identically | 0 | global (config; not tenant data) |
| Genesis records | Git + S3 mirror | 0 | global (manifest; not state) |

### Cross-pack replication: FORBIDDEN

Per `policy/data-residency.md`: no audit-chain data crosses pack boundaries. Cross-pack export is the only narrow exception (tenant-initiated, DPA-recorded, signed bundle delivery to receiving-bucket attested by tenant).

**EU-resident tenant audit data never reaches a non-EU region without tenant SCC + supplementary measures.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detect: emission-rest 99th-percentile latency spike for ≥ 5min; or region-wide OCI alarm.
2. Page ops-sre-reliability.
3. Verify scope (component vs region).
4. Component: scale out unaffected components; await OCI recovery (per `failure-modes.md`).
5. Region + DR-pair: initiate DR failover (Step below).
6. Region + no DR pair: graceful degradation; tenants notified; await region recovery.

### DR Failover (eligible packs)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair Postgres replica caught up (≤ 30s lag); HSM partition healthy in DR-pair region | ≤ 2 min |
| 2 | Promote DR-pair Postgres replica to primary | ≤ 5 min |
| 3 | Activate sealing-worker leadership in DR-pair region (HA-leader election) | ≤ 5 min |
| 4 | Update Global Traffic Manager: DNS for `audit-chain-<pack>.oyatie.dev` → DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 5 | Update workload µservices' SDK to use new audit-chain endpoint (Helm rollout) | ≤ 5 min |
| 6 | Verify three-channel root publication active in DR-pair region | ≤ 2 min |
| 7 | Drain WAL backlog | ≤ 10 min |
| 8 | Tenant notification per `incident-response.md` | ≤ 30 min |
| 9 | Engage Oracle for primary region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** |

RPO: ≤ 5min (S3 CRR cadence + Postgres replication lag worst-case).
RTO: ≤ 35min.

### Failback (after primary region recovers)

Manual, not auto-failback. Primary region must demonstrate ≥ 6h healthy state. Procedure reverses Phase 1–6 above.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill | Quarterly per DR-pair pack | Full failover + failback | ops-sre-reliability |
| HSM partition failover drill | Quarterly per DR-pair pack | HSM partition rebalance | ops-security + cloud-secrets |
| Cross-region restore drill | Monthly | S3 + Postgres backup restore validation | ops-sre-reliability |
| Cryptographic verifiability drill | Quarterly | Pull a frozen evidence bundle; validate via SDK against published keys | axis-audit-chain |
| Tabletop (regional outage) | Annually | Full incident + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos injection | Continuous (Chaos Mesh) | Pod kill, AZ partition | ops-sre-reliability |

## RPO / RTO Per Pack

| Pack | RPO | RTO | Notes |
|---|---|---|---|
| pack-kr | ≤ 5 min (intra-region replication; no DR pair) | ≤ 4h | OCI ap-seoul-1 AZ-level redundancy |
| pack-eu | ≤ 5 min (CRR) | ≤ 35 min | DR pair |
| pack-us | ≤ 5 min | ≤ 35 min | DR pair |
| pack-us-healthcare | ≤ 5 min | ≤ 35 min | DR pair; HIPAA-eligible |
| pack-jp | ≤ 5 min | ≤ 4h | single-region |
| pack-sg | ≤ 5 min | ≤ 4h | single-region |
| pack-au | ≤ 5 min | ≤ 35 min | DR pair |
| pack-in | ≤ 5 min | ≤ 35 min | DR pair |
| pack-br | ≤ 5 min | ≤ 35 min | DR pair |
| pack-ae | ≤ 5 min | ≤ 35 min | DR pair |
| pack-ksa | ≤ 5 min | ≤ 35 min | DR pair |

Per-tenant RPO/RTO disclosed at tenant onboarding per DPA.

## Tenant Notification

Per `incident-response.md`:
- Status page within 5 min of failover initiation.
- Tenant operator email within 30 min.
- Customer-facing message template available in onboarding portal.
- Regulatory notification per `compliance.md` (GDPR 72h, HIPAA 60d, KR PIPA Art. 34 72h, etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/audit-chain-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing for EU financial-services tenants.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice audit-chain` — exit 0; deployed topology matches.
- Quarterly DR-failover drill audit log.
- Annual third-party BCDR audit: ISO 22301 + NIST SP 800-34 + DORA.

## References

- `microservices/audit-chain/policy/data-residency.md`.
- `microservices/audit-chain/capacity-model.md`.
- `microservices/audit-chain/cost-budget.md`.
- `microservices/audit-chain/failure-modes.md`.
- `microservices/audit-chain/incident-response.md`.
- ADR-0117 + ADR-0028.
- OCI region docs + Cloud-HSM partition-replication docs.
- ISO/IEC 22301:2019; NIST SP 800-34; EU DORA Regulation 2022/2554.

---

## ADR-0158 Multi-Region Disposition Statement

**Disposition: `active_active` per cell for emission/query/verification; single sealing authority per `(pack, cell, tenant_partition)`.**

Per ADR-0158 (per-µservice multi-region disposition), audit-chain is active-active at the stateless edge. Local ADR-AUD-001 refines the sealing rule: authoritative roots are per cell and per tenant partition, and no fleet-wide root is authoritative for writes, disputes, or recovery. Sovereign-pinned tenants (per ADR-0162 + ADR-0164) get dedicated shards confined to their region.

| Property | Value |
|---|---|
| Disposition | `active_active` edge; per-shard sealing leader |
| RPO (intra-region) | ≤ 1 second (synchronous replication of hot leaves) |
| RTO (intra-region) | ≤ 60 seconds (failover) |
| Sovereign-pin behavior | dedicated shard in-region only; no cross-region replication beyond DR |
| Convergence model | append-only cell-local authority transfer; regional/fleet roots are witnesses only |
| Cross-region transaction policy | forbidden (data plane is strictly regional) |

Sovereign packs (`pack-ksa`, `pack-uae`, `pack-eu-sovereign`, `pack-us-gov`, `pack-kr-fsc`, `pack-kr-public`) override: per-tenant dedicated shard per ADR-0162; in-region key custody per ADR-0043 + ADR-0164.

## ADR-0162 Per-Tenant Audit-Chain Slicing Statement

Per ADR-0162 and local ADR-AUD-001, audit-chain seals partition by `tenant_id` within the owning cell. Per-pack shared shards use leaf-level partition; sovereign-tenant packs use dedicated shards.

**Sharding scheme:**
- **Shared shard per pack** — multi-tenant packs (pack-us-shared, pack-global, etc.) — per-pack Merkle tree with tenant_id leaf partition; per-tenant subtree retrieval O(log n).
- **Dedicated shard per sovereign tenant** — packs marked `dedicated_audit_shard: true` (pack-ksa, pack-uae, pack-eu-sovereign, pack-ru-if-onboarded) — per-tenant Merkle tree with in-region storage + key custody + sealing.

**Sealing cadence:**
- Hot leaf append ≤ 100 ms p99.
- Full cell-local period root completed and Ed25519-signed ≤ 1 s p99.
- Regional summary root published every minute as a witness artifact.
- Daily fleet witness published to trust portal for transparency only; it is never an authority for sovereign-pinned chain state.

**Per-tenant retrieval API:**
- Current REST projection is `contracts/openapi/audit-chain.yaml`: `POST /query`, `GET /events/{event_id}/proof`, `POST /verify`, `GET /roots/{pack}/{period_id}`, and `GET /keys/{pack}/{epoch_id}`.
- Cedar-gated through `X-Scope-OrgID`, tenant-bound principals, and `policy/tenant-scope.cedar`.
- Cursor pagination; 1000 records per page by query policy.
- Optional Merkle inclusion proof per event.
- DSR-cascade-safe: seal contains hashes + metadata only; PII fields zeroed per ADR-0008.

CI lane `oya gate validate audit-chain-per-tenant-slicing` enforces (a) Cedar-gated retrieval, (b) sovereign packs declare dedicated shards, (c) subtree leaves contain only that tenant's events.

Per-pack overlay declares `dedicated_audit_shard: true|false` at `microservices/audit-chain/iac/kustomize/components/pack-{name}/values.yaml`. See `/specs/per-tenant-audit-log-slicing-canonical.json` for the canonical declaration.
