---
microservice: observability
ip: IP-031
title: Tail-sample fidelity regression test (errors + slow + new-endpoint preserved)
status: Drafting
owner: axis-observability
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0210]
---

# IP-031 — Tail-sample fidelity test

## Purpose

Regression test: inject 1000 traces with mix of (i) errors, (ii) slow > p99 threshold, (iii) new-endpoint, (iv) baseline. Assert tail processor preserves 100% of (i), (ii), (iii) + ~1% of (iv).

## Acceptance criteria

1. Test harness emits 1000 traces via OTLP gRPC into the Tail Sampling Processor gateway.
2. Query Tempo for delivered traces.
3. Assert preservation rates:
   - error traces: 100%
   - p99 slow: 100%
   - new-endpoint (within 30-day window): 100%
   - audit-event: 100%
   - SLO-burn-window: 100%
   - baseline: 1% ± 0.5%
4. Run on every PR via CI.
5. Failure budget: 0 (any drop in fidelity fails CI).

## Cross-references

- ADR-0210 — tail sampling policy.
- IP-029 — Collector config.
