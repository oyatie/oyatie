---
doc_class: Benchmark
microservice: community
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie community vs Discourse vs Khoros vs Bevy vs Mighty Networks vs Circle

Workloads measured: (a) post-create + read latency, (b) deep-thread render latency, (c) LLM moderation E2E, (d) identity-verification flow wall-clock, (e) cross-tenant federation query, (f) annual TCO for a 1M-user community.

Hardware (oyatie paid on-prem): 8× community-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0 (3 shards × 2 replicas), Elasticsearch 8.16 (3 nodes), SeaweedFS-S3 (6 nodes).

Comparators: Discourse self-host on equivalent hardware. Khoros Cloud (Enterprise tier). Bevy Cloud (Enterprise tier). Mighty Networks Pro tier. Circle Plus tier.

## Workload (a) — post-create + read latency

| Platform | Create p99 (ms) | Read p99 (ms) |
|---|---:|---:|
| oyatie community (paid) | 124 | 38 |
| oyatie community (paid advanced) | 88 | 28 |
| Discourse self-host | 280 | 120 |
| Khoros | 480 | 240 |
| Bevy | 320 | 184 |
| Mighty Networks | 380 | 220 |
| Circle | 320 | 180 |

Reading: oyatie's PostgreSQL + Citus + indexed materialized-path tree gives best-in-class read latency. SaaS competitors run on managed databases with higher round-trip overhead.

## Workload (b) — deep-thread render latency (depth-7 thread, ~ 500 comments)

| Platform | p99 (ms) |
|---|---:|
| oyatie community (paid) | 320 |
| oyatie community (paid advanced) | 220 |
| Discourse | 1 800 |
| Khoros | 2 200 |
| Bevy | 1 200 |
| Mighty Networks | 1 600 |
| Circle | 1 400 |

Reading: oyatie's materialized-path tree pattern (per IP-005) supports deep threading with a single indexed query. Other platforms typically traverse parent pointers recursively → N+1 queries.

## Workload (c) — LLM moderation E2E latency

| Platform | p99 (ms) | Auto-remove confidence threshold | Queue threshold |
|---|---:|---:|---:|
| oyatie community (paid, intelligence-µservice bridge) | 480 | 0.95 | 0.40 |
| Discourse + Perspective API plugin | 1 800 | 0.85 | 0.40 |
| Khoros AI Moderation | 1 200 | 0.90 | 0.50 |
| Bevy (manual moderation only) | N/A | N/A | N/A |
| Mighty Networks AI Moderation | 1 400 | 0.85 | 0.50 |
| Circle AI Moderation | 1 600 | 0.88 | 0.40 |

Reading: oyatie's LLM moderation (Llama 3.3 70B + moderation LoRA) runs on dedicated GPU pool with lower latency than competitor SaaS APIs.

## Workload (d) — identity-verification flow wall-clock (corporate-email magic-link, redemption time)

| Platform | p99 wall-clock | Anonymous-with-verified-identity supported? |
|---|---:|---|
| oyatie community (paid) | 24 s | Yes (TeamBlind-class) |
| Discourse | 18 s | No (no verified-anonymous primitive) |
| Khoros | 30 s | No |
| Bevy | 28 s | No |
| Mighty Networks | 24 s | No |
| Circle | 22 s | No |

Reading: oyatie + Discourse + Mighty + Circle are all sub-30s for email magic-link. Only oyatie offers the TeamBlind-class verified-anonymous primitive — none of the competitors model this.

## Workload (e) — cross-tenant federation query (verified-corporate-email reader from tenant-A reads board on tenant-B)

| Platform | p99 (ms) | Supported? |
|---|---:|---|
| oyatie community (paid, Cedar-gated) | 184 | Yes |
| Discourse (self-host; cross-instance not supported) | N/A | No |
| Khoros (org-bound) | N/A | No |
| Bevy (single-tenant) | N/A | No |
| Mighty Networks | N/A | No |
| Circle | N/A | No |

Reading: cross-tenant federation is unique to oyatie. The competitors are all per-tenant-only. This matters for TeamBlind-class cross-company communities + cross-tenant professional networks.

## Workload (f) — annual TCO for 1M-user community + 100M posts/year + 1B comments/year

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie community (paid self-hosted) | 420 000 | 0 | 248 000 (2 SRE × 0.4 FTE) | 668 000 |
| oyatie community (paid advanced) | 960 000 | 0 | 372 000 | 1 332 000 |
| Discourse self-host (with plugins) | 96 000 | 0 | 372 000 (3 SRE × 0.4 FTE; plugin maintenance) | 468 000 |
| Discourse Hosting (Business) | 0 | 84 000 | 124 000 | 208 000 (limited to ~ 100k MAU; this is for small scale) |
| Khoros (Enterprise) | 0 | 1 800 000 - 4 800 000 | 124 000 | 1 924 000 - 4 924 000 |
| Bevy (Enterprise) | 0 | 240 000 - 600 000 | 124 000 | 364 000 - 724 000 |
| Mighty Networks (Pro) | 0 | 240 000 (~ $20/year per active user × 1M users discounted) | 124 000 | 364 000 |
| Circle (Plus per-member) | 0 | 360 000 (~ $30/year per member × 1M discounted) | 124 000 | 484 000 |

Reading: Discourse self-host is the cheapest option BUT lacks the LLM moderation + TeamBlind-class anonymous + cross-tenant federation. Bevy and Mighty are competitive on cost but limited in feature set.

oyatie paid is competitive on cost with the SaaS alternatives + ships ALL the features (TeamBlind anonymous, cross-tenant federation, LinkedIn reputation, Handshake jobs at paid advanced, SecureDrop at paid compliance-pack).

Caveats:

- Discourse self-host requires significant ops investment (plugin compatibility, version upgrades, search-index maintenance).
- SaaS pricing is heavily negotiated; the listed range reflects mid-tier to enterprise. 30-50% discount common at enterprise scale.
- The 1M-user assumption may not be representative for all tenants; lower-scale tenants get proportionally cheaper.

## Reproducibility

The benchmark harness lives at `benchmarks/communitybench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks community \
    --workload 1m-users-100m-posts-yr \
    --tenant-class paid \
    --comparators discourse,khoros,bevy,mighty,circle \
    --output ./benchmark-results.json
```

Comparator runs require valid SaaS sandbox / trial accounts. Discourse comparator requires a self-host instance. Results live at `benchmarks/results/community/<date>.csv` and are re-run quarterly.
