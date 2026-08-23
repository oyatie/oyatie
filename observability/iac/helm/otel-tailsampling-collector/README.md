# observability-otel-tailsampling-collector

OpenTelemetry Collector gateway tier configured with the **Tail Sampling Processor**
per ADR-0210.

## Position in the pipeline

```
[ per-node agent collectors ]  →  [ THIS gateway tail-sampling tier ]  →  [ Tempo ]
        (DaemonSet, per ADR-0186 Stage 1)              (Deployment, this chart)              (per ADR-0186 Stage 2)
```

## Policy set (closed)

Per ADR-0210 — first match wins:

1. `status_code=ERROR` → 100%
2. `latency > p99_threshold_ms` → 100%
3. `slo_burn_window_active` → 100%
4. `audit_event` (`EVT-*` prefix) → 100%
5. `new_endpoint_warmup` (30-day TTL) → 100%
6. `random_baseline` → 1%

## Memory budget

256 MiB processor buffer + headroom = 512 MiB request, 1 GiB limit. HPA on memory pressure
(target 70%; min 3; max 24).

## Per-µservice manifest binding

Per-µservice `manifest.json` declares `observability.trace_sampling_recipe.head_bps` (head
sample rate at the agent tier) and `p99_latency_threshold_ms` (override for the
`latency-p99` policy threshold). The CD layer regenerates the `values.yaml`
`policies[].latency.threshold_ms` from the per-µservice manifest at promotion time.

## Cross-references

- ADR-0186 — observability backplane.
- ADR-0210 — tail sampling.
- `docs/standards/trace-sampling-tier.md` — canonical policy.
