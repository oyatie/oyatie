---
ip_id: IP-040
microservice: data-warehouse
title: Time-travel + fail-safe
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-01
binding_adrs: [ADR-0131, ADR-0244, ADR-0251, ADR-0252, ADR-0329]
counterpart_parity: Snowflake Time Travel + Fail-Safe
capabilities_touched: [time-travel-restore]
billing_components: [time_travel_storage_days, fail_safe_storage_days]
---

# IP-040 — Time-travel and fail-safe

## §1 Objective

Land Snowflake-class Time-Travel + Fail-Safe. Time-travel is
tenant-purchasable up to 90 days; fail-safe is tenant-purchasable up to
35 days; together they give a tenant up to 125 days of historical recovery.

Closes F-D4-D-01 ("Time-travel queries — Missing primitive").

## §2 Scope

In scope:

- Time-travel reads on Delta / Iceberg / Hudi tables.
- Fail-safe (post-time-travel SRE-mediated recovery).
- Per-tenant `time_travel_storage_days` + `fail_safe_storage_days`
  configuration.
- VACUUM / snapshot-expire honoring the purchased window.
- Audit-chain emission per time-travel query.

Out of scope:

- Cross-region time-travel (the time-travel window lives in the tenant's
  home cell; cross-region replication of the window is handled by
  `multi-region.md`).

## §3 Architecture

### §3.1 Time-travel SQL

```sql
SELECT * FROM orders AT(TIMESTAMP => '2026-05-07T12:00:00Z');
SELECT * FROM orders AT(VERSION => 42);
SELECT * FROM orders BEFORE(STATEMENT => '<query_id>');
```

For Delta: resolves via `_delta_log` history.
For Iceberg: resolves via snapshot timeline.
For Hudi: resolves via Hudi timeline.

### §3.2 Fail-safe

If a tenant needs to restore from a point past the time-travel window but
within the fail-safe window, the operator (with SRE attestation) invokes
`POST /v1/lake/tables/{name}/fail-safe-restore?at_timestamp=…`. The
operation is gated by `emergency-services-bypass.cedar` (two-person rule,
24 h auto-expire). The restore writes a new snapshot from the recovered
state.

### §3.3 Pricing

- `time_travel_storage_days` accrues storage cost per day held.
- `fail_safe_storage_days` accrues storage cost at SRE rate (higher
  than time-travel; reflects SRE on-call cost).

### §3.4 Resolution latency

Time-travel resolution is metadata-only; the SLO target is p99 ≤ 1 s.
The actual query then runs as a normal scan.

## §4 Cedar binding

`local-time-travel-scope.cedar` (new) — allows only within the tenant's
purchased window; refuses cross-window with `time_travel_window_exceeded`.

## §5 SLO bindings

- `slos/time-travel-resolution.openslo.yaml` — p99 time-travel resolve ≤
  1 s; the underlying scan is graded by `local-query-latency.openslo.yaml`.

## §6 Failure modes

- Query within window but log file deleted by external action → refused
  with `time_travel_metadata_missing`; promote to incident.
- Query past window → refused with `time_travel_window_exceeded`.
- Fail-safe restore outside operator + SRE attestation → refused.

## §7 Acceptance criteria

- A `paid` tenant with 14-day time-travel resolves a query at -7 days in
  ≤ 1 s.
- A query at -100 days is refused with `time_travel_window_exceeded`.
- A fail-safe restore at -30 days (within 35-day fail-safe) succeeds with
  two-person attestation.
- `demo_trial` tenant cannot use time-travel.

## §8 Risks

- Time-travel storage adds linear cost over the window; tenants must
  understand the bill.

End of IP-040.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-040-time-travel-and-fail-safe.md` matched `p99, SLO, multi-region`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-040-time-travel-and-fail-safe.md` matched `cost, emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
