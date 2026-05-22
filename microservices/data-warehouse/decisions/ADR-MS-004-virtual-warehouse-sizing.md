---
adr_id: ADR-MS-004
microservice: data-warehouse
title: Virtual warehouse sizing model
date: 2026-05-21
status: accepted
wave: Wave-15A-DATA-WAREHOUSE-FIX
defect_closed: F-D3-02
binding_adrs: [ADR-0254, ADR-0329, ADR-0331]
---

# ADR-MS-004 — Virtual warehouse sizing model

## Context

Three counterparts ship three different sizing models:

- Snowflake: T-shirt sizes (XS, S, M, L, XL, 2XL, 3XL, 4XL, 5XL, 6XL),
  each doubling the cluster size; multi-cluster warehouses scale
  horizontally; per-cluster credits per hour.
- BigQuery: slot reservations (100-slot granularity), autoscaling slot
  pools, on-demand for non-reserved.
- Databricks: SQL warehouse sizes (2X-Small … 4X-Large), instance pools,
  spot/on-demand mix, DBU per hour.

The audit F-D4-D-09 noted virtual warehouses were "named but unsized".
This decision picks a single sizing surface that maps cleanly to all
three counterparts.

## Decision

Oyatie exposes the **T-shirt sizing** surface as the canonical primitive:

| Size | Compute units | Maps to (Snowflake / BigQuery / Databricks) |
|---|---|---|
| XS | 1 | XS / 100 slots / 2X-Small |
| S | 2 | S / 200 slots / X-Small |
| M | 4 | M / 400 slots / Small |
| L | 8 | L / 800 slots / Medium |
| XL | 16 | XL / 1600 slots / Large |
| 2XL | 32 | 2XL / 3200 slots / X-Large |
| 3XL | 64 | 3XL / 6400 slots / 2X-Large |
| 4XL | 128 | 4XL / 12800 slots / 3X-Large |
| 5XL | 256 | 5XL / 25600 slots / 4X-Large |
| 6XL | 512 | 6XL / 51200 slots / – |

Per-warehouse knobs:

- `auto_suspend_seconds` (default 60).
- `auto_resume` (default true).
- `min_clusters` / `max_clusters` for multi-cluster scaling.
- `scaling_policy` ∈ {`STANDARD`, `ECONOMY`}.

`demo_trial` is capped at size XS, max 1 cluster.

Compute units accrue to `paid.billing_components.compute_credits` at a
linear rate. 1 compute unit-hour = 1 compute credit.

Pods run on Cloud Hypervisor with Kata containers per ADR-0254. Cold-start
target p99 ≤ 800 ms; warm resume ≤ 100 ms.

## Consequences

- The T-shirt sizing is the *external* surface; internally, oyatie can
  swap the underlying instance type per cloud profile.
- A tenant migrating from Snowflake sees identical sizing semantics.
- A tenant migrating from BigQuery sees their slot reservation converted
  to a T-shirt size.
- A tenant migrating from Databricks sees their SQL warehouse converted.

## Alternatives considered

- Slot-based reservation (BigQuery-style) as canonical: rejected because
  Snowflake is the larger market and T-shirt sizes are easier for
  operators to reason about.

End of ADR-MS-004.
