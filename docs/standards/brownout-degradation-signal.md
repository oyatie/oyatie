---
contract: brownout-degradation-signal
authored: 2026-05-18
canonical_authority: ADR-0176
related_specs:
  - /specs/brownout-degradation-signal.json
related_adrs:
  - ADR-0042
  - ADR-0044
  - ADR-0128
  - ADR-0148
  - ADR-0176
status: canonical-base
authorities_cited:
  - Google SRE — Graceful degradation (ch. 22)
  - Cloudflare Engineering — Worker brown-out signal (2023)
  - Netflix Hystrix — circuit breaker + bulkhead docs
  - AWS Builders Library — Avoiding insurmountable queue backlogs
---

# Brown-out + graceful-degradation signal standards

## Header contract

Every public µservice RPC (HTTP + gRPC + AsyncAPI) emits:

```
degradation-class: nominal|degraded|brownout|outage
```

Absent header = `nominal` (default). The header is supplemental — the
HTTP status code is unchanged.

## Class semantics

| Class | Meaning | Caller-side behavior |
| --- | --- | --- |
| `nominal` | All SLOs in green; no degradation | Continue normal traffic |
| `degraded` | At least one SLO budget below 25%; throughput nominal | Continue; reduce retry budget; defer non-essential queries |
| `brownout` | Throughput intentionally capped; shed non-essential load | Fall through to cached/local state; reduce non-essential reads to zero |
| `outage` | Service cannot fulfill request | Fail fast; no retry within per-cell budget |

## Prometheus gauge

```
degradation_class{microservice="<name>", cell_id="<uuid>"}
  # 0 = nominal, 1 = degraded, 2 = brownout, 3 = outage
```

Dashboard: `microservices/observability/dashboards/brownout-degradation.md`.

## Decision logic

A µservice computes its class from:

```
class = max(
  slo_burn_rate_class(slo) for slo in self.slos,
  resource_pressure_class(cpu, mem, conn_pool),
  dependency_brownout_class(self.required_dependencies)
)
```

Thresholds:

| Signal | Nominal | Degraded | Brownout | Outage |
| --- | --- | --- | --- | --- |
| SLO burn rate | ≤ 1.0 | ≤ 2.0 | ≤ 14.0 | > 14.0 |
| Resource pressure (util) | ≤ 0.70 | ≤ 0.85 | ≤ 0.95 | > 0.95 |
| Required dependency worst class | nominal | degraded | brownout | outage |

## Caller-side static-stability hook

```rust
match outcome.degradation_class() {
    DegradationClass::Brownout | DegradationClass::Outage => {
        if let Some(cached) = self.local_cache.get(request.key()) {
            return Ok(cached);
        }
        return Err(DegradationError::DownstreamUnavailable);
    }
    DegradationClass::Degraded => {
        self.metrics.record_degraded_upstream();
        outcome.into()
    }
    DegradationClass::Nominal => outcome.into(),
}
```

## Mesh retry budget multipliers

| Class | Retry multiplier | Outgoing policy |
| --- | --- | --- |
| nominal | 1.0× | Default per-method retry |
| degraded | 0.5× | Halve retry attempts |
| brownout | 0.1× | Effectively no retries |
| outage | 0.0× | Fail on first attempt |

## Audit chain

Class transitions (state changes, not per-request) emit audit rows of
class `DegradationStateChange` with
`{microservice_id, cell_id, from_class, to_class, contributing_factors[]}`.

## Coverage tracker

Per-µservice rollout in `registry/brownout/coverage-tracker.tsv`.
Validator lane `brownout-signal-coverage` is advisory until coverage
reaches 100%.

## Worked example

Foundry runtime µservice in cell `c-9876`:

- Eval-run-latency SLO burn rate: 9.2 (burning fast).
- CPU utilization: 0.81.
- Required dependency `observability` reports `degraded`.

Class computation: `max(brownout, degraded, degraded) = brownout`.
Header emitted: `degradation-class: brownout`. Mesh sidecars
upstream reduce retry budget to 0.1×.
