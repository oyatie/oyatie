---
microservice: observability
ip: IP-029
title: OTel Tail Sampling Processor config (gateway tier deployment)
status: Drafting
owner: axis-observability
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0139, ADR-0186, ADR-0210]
---

# IP-029 — OTel Tail Sampling Processor config

## Purpose

Deploy the OTel Collector gateway tier with Tail Sampling Processor configured per ADR-0210 closed policy set. Helm chart at `iac/helm/otel-tailsampling-collector/`.

## Acceptance criteria

1. Helm chart deployed; 3-replica minimum; HPA on memory pressure.
2. Tail policies wired: `status_code=ERROR`, `latency_p99`, `slo_burn`, `audit_event`, `new_endpoint_warmup`, `random_baseline`.
3. `decision_wait: 30s`; memory budget: 256 MiB processor + 256 MiB headroom = 512 MiB request.
4. Per-µservice manifest override flows from `manifest.json` `observability.trace_sampling_recipe`.
5. High-traffic escape hatch: drop `head_bps` to 10 (0.1%) when µservice sustained > 5,000 req/sec; drop to 1 (0.01%) at > 50,000 req/sec.
6. ≥ 5 integration tests: error-trace-preserved + slow-trace-preserved + new-endpoint-preserved + baseline-1pct-sampled + memory-budget-respected.

## Cross-references

- ADR-0210 — tail-sampling policy.
- ADR-0186 — observability backplane.
- `iac/helm/otel-tailsampling-collector/`.
