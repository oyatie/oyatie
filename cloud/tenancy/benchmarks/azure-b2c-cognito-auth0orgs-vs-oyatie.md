---
doc_class: Benchmark
microservice: tenancy
benchmark_date: 2026-05-20
related_adrs: [ADR-TEN-001, ADR-0313, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie tenancy vs Azure AD B2C vs Cognito User Pools vs Auth0 Organizations vs WorkOS vs Frontegg

Workloads measured: (a) lifecycle transition latency, (b) conglomerate permit prefetch (50k child tenants), (c) DSR cascade fan-out, (d) sovereign-child-veto evaluation, (e) tenant migration ceremony, (f) annual TCO for 10k-tenant SaaS platform.

Hardware (oyatie paid tenant_class baseline on-prem): 8× tenancy-api nodes, PostgreSQL Citus 13.0 (3 shards × 2 replicas), Valkey cluster, Kafka 3.8.

Comparators: Azure AD B2C (P2 + 100k MAU bucket). Cognito User Pools (50M Free + post-tier). Auth0 Organizations (Enterprise). WorkOS Pricing (Platform). Frontegg.

## Workload (a) — Lifecycle transition latency

| Platform | p95 (ms) | States supported |
|---|---:|---:|
| oyatie tenancy (paid tenant_class baseline) | 78 | 10 (ADR-TEN-001) |
| oyatie tenancy (paid tenant_class expanded deployment) | 52 | 10 |
| Azure AD B2C | ~ 200 | 2-3 effective (Created/Disabled) |
| Cognito User Pools | ~ 280 | 4 (UNCONFIRMED/CONFIRMED/etc.) |
| Auth0 Organizations | ~ 320 | 2 effective |
| WorkOS | ~ 180 | 4 |
| Frontegg | ~ 320 | 3 |

Reading: oyatie's typed state machine + idempotent commands + PostgreSQL Citus give fastest lifecycle ops. Most competitors have only 2-3 functional states; oyatie's 10 states enable richer compliance modeling.

## Workload (b) — Conglomerate permit prefetch (50k child tenants, 10 permits each)

| Platform | p95 (ms) | Conglomerate hierarchy supported? |
|---|---:|---|
| oyatie tenancy (paid tenant_class baseline) | 48 | Yes (up to 5 levels) |
| oyatie tenancy (paid tenant_class expanded deployment) | 28 | Yes (up to 10 levels; 50k children) |
| Azure AD B2C | N/A | No (no hierarchy primitive) |
| Cognito User Pools | N/A | Limited (App Groups not hierarchical) |
| Auth0 Organizations | ~ 240 | Limited (Sub-orgs; 2 levels) |
| WorkOS | ~ 180 | Limited (Org hierarchy; 3 levels) |
| Frontegg | N/A | No |

Reading: oyatie's `pack_set_hash` caching (per ADR-TEN-001 § Implementation Notes) + permit prefetch beats Auth0 + WorkOS by 5-10×. Most competitors don't model conglomerate hierarchy at all.

## Workload (c) — DSR cascade fan-out (1k tenants × 20 downstream services)

| Platform | Total wall-clock (min) | Per-service ack tracked? |
|---|---:|---|
| oyatie tenancy (paid tenant_class baseline) | 12 | Yes (per ADR-TEN-001) |
| oyatie tenancy (paid tenant_class expanded deployment) | 6 | Yes |
| Azure AD B2C | ~ 60 (manual; no cascade primitive) | No |
| Cognito User Pools | ~ 45 | No |
| Auth0 Organizations | ~ 30 | Limited (Auth0 Workflows) |
| WorkOS | ~ 15 | Yes (Identity events) |
| Frontegg | ~ 25 | Limited |

Reading: oyatie + WorkOS lead in DSR cascade. oyatie's per-service ack tracking (`tenancy.offboarding.cascade.requested.v1` + ack events) makes the cascade observable.

## Workload (d) — Sovereign-child-veto evaluation (parent attempts data access on child with stricter pack)

| Platform | Decision wall-clock (ms) | Veto supported? |
|---|---:|---|
| oyatie tenancy (paid tenant_class baseline) | 18 (Cedar evaluation) | Yes (per ADR-TEN-001 + ADR-0313) |
| oyatie tenancy (paid tenant_class expanded deployment) | 12 | Yes |
| Azure AD B2C | N/A | No |
| Cognito User Pools | N/A | No |
| Auth0 Organizations | N/A | No (sub-orgs inherit parent permissions) |
| WorkOS | N/A | No |
| Frontegg | N/A | No |

Reading: sovereign-child-veto is unique to oyatie. Competitors' hierarchy models always grant parent permissions to children (or are flat).

## Workload (e) — Tenant migration ceremony (cross-region; 10 GiB data; full re-encryption)

| Platform | Wall-clock (h) | Audit-attested? |
|---|---:|---|
| oyatie tenancy (paid tenant_class baseline) | 14 | Yes (Ed25519-signed audit-chain) |
| oyatie tenancy (paid tenant_class expanded deployment) | 8 (parallel migration workers) | Yes |
| Azure AD B2C | ~ 48 (manual; no automated primitive) | Limited |
| Cognito User Pools | ~ 24 (cross-region replication available) | Limited |
| Auth0 Organizations | ~ 36 (manual customer success) | Limited |
| WorkOS | ~ 18 | Limited |
| Frontegg | N/A | No |

Reading: oyatie has the only automated council-approved migration primitive. Competitors require manual support tickets + custom processes.

## Workload (f) — Annual TCO for 10k-tenant SaaS platform (5k principals/tenant; 100 cross-tenant DSRs/year)

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie tenancy (paid tenant_class baseline self-hosted) | 280 000 | 0 | 248 000 (2 SRE × 0.4 FTE) | 528 000 |
| oyatie tenancy (paid tenant_class expanded deployment) | 480 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 852 000 |
| Azure AD B2C P2 (5M MAU bucket) | 0 | 900 000 | 124 000 | 1 024 000 |
| Cognito User Pools (5M MAU) | 0 | 825 000 | 124 000 | 949 000 |
| Auth0 Organizations Enterprise | 0 | 480 000 (custom contract) | 124 000 | 604 000 |
| WorkOS Pricing Platform (per-user) | 0 | 1 020 000 | 124 000 | 1 144 000 |
| Frontegg (per-user enterprise) | 0 | 720 000 | 124 000 | 844 000 |

Reading: oyatie paid tenant_class baseline is competitive; paid tenant_class expanded deployment is close to Azure B2C P2 cost while providing the full conglomerate + sovereign-child primitives that no competitor offers.

## Caveats

- Cognito + Azure B2C are usage-based; cost scales with MAU.
- Auth0 Organizations is enterprise-contract-negotiated; published pricing is unreliable.
- WorkOS pricing includes SSO + SCIM + organizations bundled.
- Frontegg is more developer-tooling-focused; tenancy is one slice.

## Reproducibility

```sh
cargo run -p oya-dev-cli -- benchmarks tenancy \
    --workload 10k-tenants-conglomerate \
    --tenant-class paid \
    --comparators azure-b2c,cognito,auth0-orgs,workos,frontegg \
    --include-sovereign-veto-tests \
    --output ./benchmark-results.json
```

Results live at `benchmarks/results/tenancy/<date>.csv` and are re-run quarterly.
