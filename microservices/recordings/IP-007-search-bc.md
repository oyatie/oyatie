---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-007-search-bc
status: pending
owner: axis-recordings
acceptance_lanes: [shardability, statelessness]
---

# IP-007: Search BC — Meilisearch adapter + transcript indexing

## Intent

Land cross-recording + cross-transcript search via Meilisearch 0.10.0 LTS,
per-tenant index sharded; Cedar-policy server-side filter.

## Concrete crates

`oya-recordings-search-{kernel,domain,usecase,api,adapter-meilisearch,rest,sdk,app}`.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate shardability --microservice recordings
cargo run -p oya-dev-cli -- gate validate statelessness --microservice recordings
```

## Next IP

[`IP-008-redaction-bc.md`](IP-008-redaction-bc.md)
