---
adr_id: ADR-MS-003
microservice: data-warehouse
title: Secure data sharing model (tenant + reader-account)
date: 2026-05-21
status: accepted
wave: Wave-15A-DATA-WAREHOUSE-FIX
defect_closed: F-D3-02
binding_adrs: [ADR-0244, ADR-0314, ADR-0329, ADR-0331]
---

# ADR-MS-003 — Secure data sharing model

## Context

Snowflake Secure Data Sharing, BigQuery Analytics Hub, and Databricks
Delta Sharing all expose live, no-copy data shares from a producer to a
consumer. They differ in how they identify the consumer:

- Snowflake: another Snowflake account (tenant) OR a Snowflake Reader
  Account (non-tenant, runs on producer's compute).
- BigQuery: another GCP project (tenant) OR an Analytics Hub listing
  (consumer subscribes via Analytics Hub).
- Databricks: another Databricks workspace (tenant) OR a Delta Sharing
  recipient (non-tenant, runs on their own compute).

The union envelope is: a share's *consumer* may be a tenant OR a
non-tenant ("reader-account" in oyatie terminology). Audit found
`reader-account` was ABSENT from data-warehouse (F-D4-D-03).

## Decision

The share producer is always an oyatie tenant. The share consumer is
one of:

1. **`tenant`** — another oyatie tenant; consumer reads through their
   own oyatie principal; Cedar filter still applies at consumer read.
2. **`reader_account`** — a non-tenant signed consumer account; consumer
   reads via the Delta Sharing protocol over HTTP/3 with a bearer token
   that the producer rotates; Cedar filter still applies.

Both consumer kinds are billed to the *producer* via
`paid.billing_components.share_consumer_events`. The consumer pays
nothing to oyatie.

Reader-account shares require:

- Producer `tenant_class == "paid"`.
- Producer's `billing_components` includes `share_consumer_events`.
- A signed DealSet (ADR-0314) registered with the marketplace µservice.

Cedar fragment `local-secure-share-create.cedar` enforces these gates.

## Consequences

- A producer can revoke a reader-account at any time; revocation
  propagates within 5 s globally.
- Non-tenant consumers do not get oyatie tenant identity; their identity
  is the share-pinned consumer account ID.
- High-rate reader-account abuse is mitigated by per-consumer rate limit
  + Cedar refusal at threshold.

## Alternatives considered

- Tenant-only shares (drop reader-account): rejected because it forces
  every share consumer to onboard as an oyatie tenant, which defeats
  the cross-vendor sharing use case.
- Per-share billing flips to consumer-paid: rejected because the
  consumer has no oyatie tenant; nowhere to bill them. The
  Snowflake/BigQuery/Databricks consensus is producer-pays.

End of ADR-MS-003.
