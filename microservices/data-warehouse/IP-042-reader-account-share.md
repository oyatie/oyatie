---
ip_id: IP-042
microservice: data-warehouse
title: Reader-account (non-tenant) share
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-03
binding_adrs: [ADR-0131, ADR-0244, ADR-0314, ADR-0329]
counterpart_parity: Snowflake Reader Account + BigQuery Analytics Hub + Databricks Delta Sharing
capabilities_touched: [reader-account-share-publish, governed-share-create]
billing_components: [share_consumer_events]
---

# IP-042 — Reader-account (non-tenant) share

## §1 Objective

Land secure data sharing where the *consumer* is **not** an oyatie tenant.
Snowflake's Reader Account model is the anchor; BigQuery Analytics Hub's
listings and Databricks Delta Sharing recipients sit in the same slot.

Closes F-D4-D-03 ("Reader-account / non-tenant consumer share —
Not authored").

## §2 Scope

In scope:

- `share.publish` for a producer tenant to a reader-account consumer.
- Consumer registration: out-of-band, identified by signed consumer
  account ID.
- Row-level filter (Cedar entity) per consumer.
- Column projection per consumer.
- Consumer read-path: Delta Sharing protocol + Snowflake Reader Account
  protocol + BigQuery Analytics Hub protocol.
- Per-event accrual to producer's `share_consumer_events`.

Out of scope:

- Marketplace listing UX (separate `oya-marketplace-*` µservice).
- Cross-cloud Delta Sharing federation; Wave-15B.

## §3 Architecture

### §3.1 Consumer identity

The consumer is identified by a signed consumer account ID. The producer
generates a short-lived bearer token for the consumer's machine identity.
The consumer's reads carry the bearer + the share name.

### §3.2 Cedar gate

`local-secure-share-create.cedar` (new) — refuses publish if:

- Producer is `demo_trial`.
- Producer has not enabled `share_consumer_events` billing.
- No DealSet (ADR-0314) is registered when the consumer is outside
  oyatie's tenant set.

### §3.3 Filter + projection

The filter is stored as a Cedar entity attached to the share row in the
catalog (IP-034). At consumer read time, the filter is evaluated
*server-side*; the consumer can never see filtered-out rows even by
direct path query.

### §3.4 Pricing

- `share_consumer_events` accrues per consumer read row.
- The consumer pays nothing to oyatie; the producer pays.

## §4 Acceptance criteria

- A `paid` tenant publishes a share to a reader-account.
- The reader-account consumer reads via the Delta Sharing protocol and
  sees only filtered rows.
- `share_consumer_events` accrues to the producer per row read.
- A `demo_trial` tenant publish is refused.
- Producer revokes the share; consumer read fails within 5 s.

## §5 Failure modes

- Bearer token expired → consumer read refused with `share_token_expired`.
- Producer disables billing → refused with `share_billing_not_enabled`.
- Consumer abuses high-rate scans → rate-limited; emits
  `share_consumer_rate_limited`.

## §6 SLO bindings

- `slos/governed-share-consumer-lag.openslo.yaml` — p99 read-path lag
  ≤ 1 s.

## §7 Risks

- Consumer-side abuse → mitigated by rate limit + Cedar; if the consumer
  is malicious, the producer can revoke.

End of IP-042.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-042-reader-account-share.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
