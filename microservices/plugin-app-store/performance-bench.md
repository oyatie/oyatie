---
doc_class: PerformanceBench
title: "Performance benchmark plan"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Performance benchmark plan


## Continuous benchmarks

Per ADR-0123 hyperscaler-maturity-claim-gate, every release exercises these benchmarks; results posted to `microservices/<ms>/evidence/perf-benchmarks/`.

| Benchmark | Target | Tool |
|---|---|---|
| Catalog browse 10k qps | p95 ≤ 200ms; p99 ≤ 500ms | k6 |
| Install flow 100 concurrent | p99 ≤ 15s | k6 |
| Vetting pipeline 100 submissions concurrent | p99 ≤ 4h | scripted |
| Signing key issuance 100 qps | p99 ≤ 1s | k6 |
| Sandbox provision 100 concurrent | p99 ≤ 60s | scripted |
| Sandbox reset 100 concurrent | p99 ≤ 30s | scripted |
| Codegen six families | p99 ≤ 10 min | scripted |
| Payout settlement 1k batch | p99 ≤ 4h | scripted |

## Regression budget

- 10% regression: warning.
- 25% regression: block PR.
- 50% regression: page on-call.

## Comparison baselines

- vs. Apple App Store: install-with-permission-grant flow ≤ comparable to App Store TestFlight (~ 3-5s).
- vs. Stripe Connect: onboarding flow ≤ comparable to Stripe Express onboarding (~ 5-10 min).
- vs. VS Code Marketplace: catalog browse comparable (Marketplace ~150ms p95).

