---
doc_class: Benchmark
microservice: workflow-studio
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie workflow-studio vs n8n Cloud vs Zapier Enterprise vs Make.com vs Workato

Workloads measured: (a) editor load time, (b) collaborative-edit propagation latency, (c) AI-assisted workflow generation E2E, (d) canvas scalability (max nodes responsive), (e) publish wall-clock, (f) annual TCO for 5 000 active workflow authors.

Hardware (oyatie paid tenant_class self-hosted): 6× studio-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 500 GiB NVMe) across 3 regions, WebSocket gateway 4 nodes, PostgreSQL shared with `workflow-engine`. Front-end loaded via CloudFront-equivalent CDN (Cloudflare or BunnyCDN).

## Workload (a) — editor load time (workflow with 50 nodes, cold cache)

| Platform | First Contentful Paint (ms) | Interactive (ms) p99 |
|---|---:|---:|
| oyatie workflow-studio (paid tenant_class baseline) | 480 | 1 180 |
| oyatie workflow-studio (paid tenant_class expanded deployment) | 380 | 920 |
| n8n Cloud | 920 | 2 800 |
| Zapier Editor | 1 240 | 3 200 |
| Make.com | 1 080 | 2 600 |
| Workato | 1 480 | 3 800 |

Reading: oyatie's React 19 + CDN + viewport virtualization gives best-in-class load times. Make.com is competitive; Zapier + Workato are bottlenecked by their bundle size + initial server-side rendering.

## Workload (b) — collaborative-edit propagation latency (User A moves a node → User B sees it)

| Platform | p50 (ms) | p99 (ms) | Multi-user supported |
|---|---:|---:|---|
| oyatie workflow-studio (paid tenant_class, Yjs + WS) | 86 | 220 | Yes (up to 5 baseline; up to 20 under paid tenant_class capacity policy) |
| n8n Cloud (Pro tier) | 280 | 720 | Yes (Pro + Enterprise tiers) |
| Zapier (Enterprise) | 320 | 880 | Yes |
| Make.com | 240 | 620 | Yes |
| Workato | 380 | 1 040 | Yes |

Reading: oyatie's Yjs-based CRDT + WebSocket delivers best-in-class propagation. Competitors typically use server-mediated state with longer round-trips.

## Workload (c) — AI-assisted workflow generation (natural-language goal → workflow draft)

| Platform | Wall-clock | Generated quality (% requiring user refinement) |
|---|---:|---:|
| oyatie workflow-studio (paid tenant_class) | 22 s | 35 % (refinement needed for 5+ nodes) |
| n8n AI (Beta) | 38 s | 45 % |
| Zapier AI (Beta) | 32 s | 42 % |
| Make.com (no AI generation) | N/A | N/A |
| Workato AI (Beta) | 28 s | 38 % |

Reading: oyatie's purpose-built workflow-DSL fine-tune (Llama 3.3 70B + LoRA) outperforms general-purpose LLM prompting for this task. The 35 % refinement rate beats all comparators.

## Workload (d) — canvas scalability (max nodes rendered at 60fps on a MacBook M3 Pro)

| Platform | Max responsive nodes | Render technology |
|---|---:|---|
| oyatie workflow-studio (paid tenant_class) | 5 000 | React Flow 12 + viewport virtualization + WebGL edges |
| n8n Cloud | 2 000 | React + naive DOM |
| Zapier | 500 | React; not designed for large workflows |
| Make.com | 2 000 | Custom canvas; competitive |
| Workato | 1 200 | React; medium scale |

Reading: oyatie leads at large-canvas scalability. Workflows above 500 nodes are rare but exist (typically generated workflows or enterprise orchestrations).

## Workload (e) — publish wall-clock (save + register + audit-emit)

| Platform | p99 wall-clock | Notes |
|---|---:|---|
| oyatie workflow-studio (paid tenant_class) | 480 ms | Workflow registered with engine + audit emit |
| n8n Cloud | 320 ms | n8n's own engine; no external audit |
| Zapier | 1 800 ms | Zapier's checks include integration verification |
| Make.com | 1 200 ms | Similar to Zapier |
| Workato | 920 ms | Enterprise validation includes role checks |

Reading: oyatie + n8n are sub-second; Zapier + Make + Workato add validation steps (integration health checks) that add ~ 0.5-1.5 s.

## Workload (f) — annual TCO for 5 000 active workflow authors

Assumptions: 5 000 active authors, ~ 50 000 workflows total, average 20-node workflow, 5 % use AI-assist.

| Platform | Licence (USD) | Hardware/Compute (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie workflow-studio (paid tenant_class, self-hosted baseline) | 0 | 320 000 (6 nodes × 3 regions + CDN) | 248 000 (2 SRE × 0.4 FTE) | 568 000 |
| oyatie workflow-studio (paid tenant_class, expanded deployment) | 0 | 720 000 | 372 000 | 1 092 000 |
| n8n Cloud (Pro per-author) | 1 200 000 (~ $240 per author per year) | 0 (managed) | 124 000 | 1 324 000 |
| Zapier Enterprise (per-author + per-task) | 1 800 000 (~ $360 per author/year + task overages) | 0 (managed) | 124 000 | 1 924 000 |
| Make.com (Enterprise tier) | 1 050 000 (~ $210 per author/year) | 0 (managed) | 124 000 | 1 174 000 |
| Workato (Enterprise per-recipe) | 2 400 000 (~ $480 per author/year) | 0 (managed) | 124 000 | 2 524 000 |

Reading: oyatie paid tenant_class baseline beats every commercial SaaS at this scale. The expanded paid tenant_class deployment competes on TCO + includes AI-assist + custom-node SDK + time-travel debug.

Caveats:

- SaaS pricing is 2026-Q2 list price; enterprise contracts get 30-50% discount.
- "Active author" is conservatively defined as 1+ publish per quarter; idle accounts often discount.
- The ops cost includes the studio-platform-engineering team; lower-touch deployments need fewer FTEs.

## Reproducibility

The benchmark harness lives at `benchmarks/studiobench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks workflow-studio \
    --workload 5000-authors-50k-workflows \
    --comparators n8n,zapier,make,workato \
    --output ./benchmark-results.json
```

Comparators require valid SaaS sandbox/trial accounts. Results live at `benchmarks/results/workflow-studio/<date>.csv` and are re-run quarterly.
