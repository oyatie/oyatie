---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-010-retention-legal-hold-bcs
status: pending
owner: ops-compliance + axis-recordings + council-privacy
acceptance_lanes: [retention-policy-correctness, legal-hold-chain-of-custody-correctness]
load_bearing: true
---

# IP-010: Retention-policy BC + Legal-hold BC (LOAD-BEARING)

## Intent

Land the per-pack retention policy enforcement + load-bearing 100 %
correctness legal-hold engagement (per ADR-RECORDINGS-0002). The two
canonical load-bearing CI lanes go green here.

## Concrete crates

- `oya-recordings-retention-policy-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}`
- `oya-recordings-legal-hold-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}`

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate retention-policy-correctness --microservice recordings
cargo run -p oya-dev-cli -- gate validate legal-hold-chain-of-custody-correctness --microservice recordings
cargo nextest run -p oya-recordings-legal-hold-kernel -- engagement_latency_p99
```

## Next IP

[`IP-011-playback-share-link-watermark-bcs.md`](IP-011-playback-share-link-watermark-bcs.md)
