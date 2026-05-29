---
adr_id: ADR-MS-005
microservice: data-warehouse
title: Zero-copy clone scope and Cedar boundary
date: 2026-05-21
status: accepted
wave: Wave-15A-DATA-WAREHOUSE-FIX
defect_closed: F-D3-02
binding_adrs: [ADR-0242, ADR-0244, ADR-0329, ADR-0331]
---

# ADR-MS-005 — Zero-copy clone scope

## Context

Snowflake's zero-copy clone is one of the most distinctive analytical
primitives in the market: clone a table / schema / database in O(metadata)
time, with the new object pointing to the existing data files; later
divergent writes accrue new storage on each side.

The audit F-D4-D-02 found clone was missing. IP-041 implements it. This
decision pins the scope boundary.

## Decision

Clone is allowed only:

1. Within the same `tenant_id` (no cross-tenant clone, even if the same
   oyatie account / agency owns both sub-tenants).
2. With a destination region compatible with the source's residency
   pack (e.g. KR-PIPA-pinned data cannot clone to `us-east-1`).
3. At a time-travel point within the source tenant's purchased window
   (clone AT a past version that is within `time_travel_storage_days`).

The `oyatie` reserved tenant (per ADR-0242) is *not* exempt — even oyatie
foundry self-modification (per ADR-0247) cannot clone across tenant
boundaries.

Cedar fragment `local-zero-copy-clone-scope.cedar` enforces these gates.

## Consequences

- A multi-sub-tenant agency (Diana Alvarez persona) cannot use clone to
  bridge data between her sub-tenants; she must use a governed share
  with explicit DealSet.
- Cross-cloud clone is allowed only when the destination cloud has a
  region compatible with the source residency pack.

## Alternatives considered

- Allow same-account cross-tenant clone (e.g. agency clones from
  sub-tenant A to sub-tenant B): rejected — it punches a hole in the
  tenant boundary that downstream Cedar policies cannot patch.

End of ADR-MS-005.
