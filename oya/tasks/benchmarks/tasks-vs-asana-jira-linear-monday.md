---
doc_class: Benchmark
microservice: tasks
benchmark_date: 2026-05-20
doc_status: published
---

# Benchmarks — oyatie tasks vs Asana / Jira / Linear / Monday.com

Workloads measured: (a) board-view render latency at 1 000 tasks, (b) bulk-update of 500 tasks via API, (c) dependency-graph cycle-check on a 200-node graph, (d) cross-project portfolio rollup at 100-project scale.

Hardware (oyatie): deployment_context `guest-on-oci`, tenant_class `paid` (3 AZ × 8 vCPU × 32 GiB Cloud Hypervisor VMs running the `tasks` µservice + dependencies). Comparators measured on each vendor's hosted SaaS instance with no co-location adjustment.

## Workload (a) — board-view render at 1 000 tasks

| Product | deployment_context | tenant_class / pricing class | first-paint (ms) | full-board (ms) | smooth-scroll (fps) |
|---|---|---|---:|---:|---:|
| oyatie tasks | guest-on-oci | paid | 420 | 1 100 | 60 |
| Asana | vendor-hosted SaaS | vendor pricing tier | 680 | 1 850 | 45 |
| Jira Cloud | vendor-hosted SaaS | Premium vendor pricing tier | 920 | 2 400 | 38 |
| Linear | vendor-hosted SaaS | vendor pricing tier | 380 | 980 | 60 |
| Monday.com | vendor-hosted SaaS | vendor pricing tier | 740 | 2 100 | 42 |
| ClickUp | vendor-hosted SaaS | vendor pricing tier | 1 200 | 3 100 | 32 |

Linear is the only competitor close to our render speed; their tech-stack choice (custom virtualisation + WebGL grid) is broadly similar to oyatie's. Asana and Monday are non-virtualised at this scale; the slowdown above ~ 500 cards is well-known.

Note: full-board is the "all 1 000 cards rendered" milestone (i.e., scroll-to-bottom). first-paint is the user-visible "I see something" moment.

## Workload (b) — bulk-update 500 tasks via API

| Product | deployment_context | tenant_class / pricing class | Total time (s) | Tasks/min | Atomic? |
|---|---|---|---:|---:|---|
| oyatie tasks | guest-on-oci | paid | 3.2 | 9 375 | Yes — single audit-chain transaction |
| Asana | vendor-hosted SaaS | vendor pricing tier | 78 | 385 | No — per-task API calls; partial-failure rolls forward |
| Jira Cloud bulk-edit | vendor-hosted SaaS | vendor pricing tier | 18 | 1 667 | Yes — bulk-edit endpoint with rollback |
| Linear | vendor-hosted SaaS | vendor pricing tier | 9 | 3 333 | Yes — GraphQL mutation batch |
| Monday.com | vendor-hosted SaaS | vendor pricing tier | 95 | 316 | No — per-task |
| ClickUp | vendor-hosted SaaS | vendor pricing tier | 142 | 211 | No — per-task |

oyatie's atomicity is from the single-transaction at the storage layer (Postgres with `BEGIN; UPDATE WHERE ...; INSERT INTO audit_chain ...; COMMIT;`). Asana / Monday / ClickUp have per-task calls; if call 251 of 500 fails, you have 250 tasks updated and 250 not. oyatie's all-or-nothing semantics matters for compliance-class bulk operations (e.g., GDPR Article 17 erasure-on-bulk).

## Workload (c) — dependency-graph cycle-check on 200-node graph

| Product | Has cycle-check? | Latency p99 (ms) | Approach |
|---|---|---:|---|
| oyatie tasks | Yes (write-time) | 12 | Kahn's topological-sort-on-write |
| Asana | Yes (read-time, advisory) | 280 | DFS at view-render |
| Jira | Yes (write-time) | 78 | Custom traversal |
| Linear | Yes (write-time) | 18 | Topological sort |
| Monday.com | No | n/a | Cycles allowed; UI flags but does not block |
| ClickUp | Partial (immediate-parent only) | n/a | Doesn't detect transitive cycles |

oyatie + Linear are the only products that prevent cycles at write-time with sub-25-ms p99. Monday's cycle-permissive design (cycle exists but UI flags) is intentional — they treat the graph as advisory; oyatie treats it as a strict invariant per ADR-TASKS-0002.

## Workload (d) — cross-project portfolio rollup at 100 projects, 25 000 tasks total

| Product | deployment_context | tenant_class / pricing class | First-rendering (s) | Refresh-after-mutation (s) |
|---|---|---|---:|---:|
| oyatie tasks | guest-on-oci | paid | 0.9 | 0.3 |
| Asana (with Portfolios) | vendor-hosted SaaS | vendor pricing tier | 4.2 | 1.8 |
| Jira (with Advanced Roadmaps) | vendor-hosted SaaS | vendor pricing tier | 7.1 | 3.2 |
| Linear (with cross-team views) | vendor-hosted SaaS | vendor pricing tier | 1.4 | 0.6 |
| Monday.com (with Dashboards) | vendor-hosted SaaS | vendor pricing tier | 5.8 | 2.4 |
| ClickUp (with Everything view) | vendor-hosted SaaS | vendor pricing tier | 12.1 | 6.0 |

Portfolio rollup is the hardest test of read-path optimisation. oyatie pre-computes the rollup via a materialised view in the `analytics` µservice with 30 s refresh cadence; a mutation that affects the rollup triggers a fast partial-recompute. Linear is similar in approach. Asana / Jira / Monday / ClickUp recompute on-demand which is why their first-rendering times are higher.

## Cost comparison

For a 500-seat B2B SaaS company at paid tenant_class with per_seat + per_usage billing components:

| Product | tenant_class / pricing class | Per-user / month | Annual all-in (USD) |
|---|---|---:|---:|
| oyatie tasks | paid | $19 | 114 000 |
| Asana Business | vendor pricing tier | $25 | 150 000 |
| Jira Cloud Premium | Premium vendor pricing tier | $15 | 90 000 (cheap on paper) |
| Linear Business | vendor pricing tier | $14 | 84 000 |
| Monday.com Pro | vendor pricing tier | $19 | 114 000 |
| ClickUp Business Plus | vendor pricing tier | $19 | 114 000 |

The cost difference is rarely the deciding factor at the 500-seat scale; the differentiation is feature surface + integration depth + audit posture. oyatie's per-seat price is set to match Linear/Monday; we're not trying to undercut.

## Caveats

These benchmarks are point-in-time as of 2026-05. Vendors ship continuously; expect drift. The harness at `benchmarks/tasksbench/` is reproducible and re-runs weekly in CI.

The bulk-update atomicity test uses each vendor's official API. Some vendors (notably Asana) have a "Bulk Update" UI feature that uses internal endpoints with different semantics from their public API; we measure the public API surface a customer would use programmatically.
