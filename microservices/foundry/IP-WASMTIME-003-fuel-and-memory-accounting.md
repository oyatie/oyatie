# IP-WASMTIME-003 — Fuel + memory accounting

> ADR anchor: ADR-0200, ADR-0174.
> Owner: `oya-foundry`.
> Estimate: 3 days.

## Goal

Tie Wasmtime fuel consumption + peak memory usage into the
FinOps lane (ADR-0174) so per-tenant chargeback reflects WASM
tool execution cost.

## Why this IP

WASM tools may consume significant compute. Without per-tenant
accounting, the substrate cannot bill correctly and operators
cannot detect tenant-side cost runaway.

## Tasks

### 1. Metric emission

- Prometheus counters: `foundry_wasm_fuel_consumed_total` by
  `tenant_id`, `tool_id`.
- Prometheus histograms: `foundry_wasm_memory_peak_bytes` by
  `tenant_id`, `tool_id`.
- Prometheus histograms: `foundry_wasm_wall_clock_ms` by
  `tenant_id`, `tool_id`.

### 2. FinOps emission

- Per ADR-0174, push hourly rollups into the FinOps data
  lake with cost-attribution dimensions.

### 3. Budget alerts

- Per-tenant fuel budget cap; alert at 80% utilization;
  hard-cap at 100% returns
  `FuelExhausted` to the caller.

### 4. Tests

- Unit tests for the rollup logic.
- Integration test asserting end-to-end fuel attribution to
  a synthetic tenant.

## Failure modes

- FinOps lake unavailable: rollups buffer locally; resume on
  recovery.

## Acceptance criteria

- 100% of WASM invocations land a fuel + memory + wall-clock
  metric with tenant attribution.
- Cost-attribution dashboards reflect WASM usage.

## References

- ADR-0200, ADR-0174.
