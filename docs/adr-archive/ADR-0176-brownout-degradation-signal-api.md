---
id: ADR-0176
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - ops-sre-reliability
  - axis-cloud
supersedes: []
superseded_by: [ADR-705]
amended_by: [ADR-0632]
related:
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0042-observability-stack-otel-and-in-house-ui.md
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0203-documentation-engine-three-tier.md
  - ADR-0258-api-versioning-model.md
  - ADR-0632
last_reconciled: 2026-08-01
reconciled_with: [ADR-0203, ADR-0258, ADR-0632]
doc_class: Architecture-Decision-Record
purpose: >
  Standardize a normative response header `oya-degradation-class:
  nominal|degraded|brownout|outage` on every public µservice RPC, plus
  a per-µservice Prometheus gauge. The header is the canonical signal
  for upstream static-stability fallback (per ADR-0009).
enforcement_status: advisory-until-public-rpc-coverage-complete
enforced_by: oya gate validate brownout-signal-coverage
---

# ADR-0176: Brown-out + graceful-degradation signal API

## Status

Accepted — 2026-05-18. Enforcement is advisory until every public RPC
on every µservice surfaces the header. Coverage tracker at
`registry/brownout/coverage-tracker.tsv`.

## ADR-0632 product-protocol reconciliation

For public traffic, the degradation signal **MUST** travel through HTTPS REST/OpenAPI 3.2.0
headers or equivalent metadata in signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, and
bidirectional WebSocket messages. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. The
same signal may be carried as metadata on internal-only gRPC/proto3 over HTTP/2 without creating a
public RPC contract.

### Public-contract reconciliation

Per ADR-0203 and ADR-0258, public contract carriers are REST documented by OpenAPI 3.2 plus
webhooks, events, and streams documented by AsyncAPI 3.1. gRPC over HTTP/2 (H2) with proto3 is
internal-only service-to-service traffic under mTLS; it is not a public API contract.


## Context

The hyperscaler invariants spec (ADR-0128) declares circuit-breaker
(INV-CIRCUIT-BREAKER) and static-stability (INV-STATIC-STABILITY)
invariants. ADR-0009 cell architecture requires cell-isolation; ADR-0042
observability stack supplies metrics. But the portfolio is missing the
*explicit signal* — how does µservice A tell its upstream caller B
"I am brown-out, please consider falling back to cached data" without
B having to infer the state from latency tails alone?

Industry consensus pattern (Netflix Hystrix, Cloudflare Workers brown-out
signal, Google's "graceful degradation" guidance in *Site Reliability
Engineering* ch. 22) is: an explicit header per response declaring the
degradation class. The receiver's static-stability strategy then
decides whether to retry, fall through to cached/local state, or surface
a user-visible error.

Without the header:

- Upstream callers infer degradation from latency tails, which is
  noisy and lagging.
- The observability dashboard cannot plot per-µservice degradation
  class as a first-class signal.
- The mesh layer (Istio per ADR-0148) cannot bias retry budgets based
  on the downstream's self-declared state.
- The static-stability invariant (ADR-0128 INV-STATIC-STABILITY) has
  no canonical input.

## Decision

### D-1. Normative response header

Every public HTTPS REST response emits the header below. Public webhooks, AsyncAPI events, SSE,
and WebSocket messages carry the same value as protocol-appropriate metadata; internal gRPC
responses carry it as response metadata without creating a public RPC contract:

```
oya-degradation-class: nominal|degraded|brownout|outage
```

Class semantics:

| Class | Meaning | Caller-side behavior |
| --- | --- | --- |
| `nominal` | All SLOs in green; no degradation | Continue normal traffic |
| `degraded` | At least one SLO budget below 25%; per-request throughput nominal | Continue traffic; reduce retry budget; consider non-essential queries deferred |
| `brownout` | Throughput intentionally capped; shed non-essential load | Fall through to cached/local state if available; reduce non-essential reads to zero |
| `outage` | Service cannot fulfill the request | Fail fast; do not retry within the per-cell budget |

The header is emitted on every response. Absent header = `nominal`
(default). The header is supplemental — the HTTP status code is
unchanged.

### D-2. Per-µservice Prometheus gauge

Every µservice publishes the gauge:

```
oya_degradation_class{microservice="<name>", cell_id="<uuid>"}  # value: 0 = nominal, 1 = degraded, 2 = brownout, 3 = outage
```

The gauge is visualized in the canonical observability dashboard
(`microservices/observability/dashboards/brownout-degradation.md`).

### D-3. Decision logic

Each µservice computes its degradation class from:

```
class = max(
  slo_burn_rate_class(slo) for slo in self.slos,
  resource_pressure_class(cpu, memory, conn_pool),
  dependency_brownout_class(self.dependencies),
)

where:
  slo_burn_rate_class:
    burn ≤ 1.0  → nominal
    burn ≤ 2.0  → degraded
    burn ≤ 14.0 → brownout
    burn  > 14.0 → outage

  resource_pressure_class:
    util ≤ 0.70 → nominal
    util ≤ 0.85 → degraded
    util ≤ 0.95 → brownout
    util  > 0.95 → outage

  dependency_brownout_class:
    every dep nominal           → nominal
    any dep degraded            → degraded
    any required dep brownout   → brownout
    any required dep outage     → outage
```

Implementation lives in `crates/oya-observability-degradation-kernel/`
(planned crate; first IP under microservices/observability/).

### D-4. Mesh integration

The service mesh sidecar (per ADR-0148 Cilium L7) reads the response
header from each downstream call and feeds the value into its retry
budget computation. The mesh respects:

| Class | Retry budget multiplier | Outgoing retry policy |
| --- | --- | --- |
| `nominal` | 1.0× | Default per-method retry policy |
| `degraded` | 0.5× | Halve retry attempts |
| `brownout` | 0.1× | Effectively no retries |
| `outage` | 0.0× | Fail immediately on first attempt |

### D-5. Caller-side static-stability hook

Each µservice's caller stack reads the header and decides:

```rust
let outcome = downstream.call(request).await;
match outcome.degradation_class() {
    DegradationClass::Brownout | DegradationClass::Outage => {
        if let Some(cached) = self.local_cache.get(request.key()) {
            return Ok(cached);
        }
        return Err(DegradationError::DownstreamUnavailable);
    }
    DegradationClass::Degraded => {
        self.metrics.record_degraded_upstream();
        return outcome.into();
    }
    DegradationClass::Nominal => outcome.into(),
}
```

The cache integration is per-µservice (some µservices have caches; some
don't). The header itself is universal.

### D-6. Audit chain integration

Degradation-class transitions (state changes, not per-request emissions)
emit audit rows of class `DegradationStateChange`. The row records
{microservice_id, cell_id, from_class, to_class, contributing_factors[]}.

## Alternatives considered

### Alt-1. Latency-based inference only

Let upstream callers infer degradation from p95/p99 latency tails.
**Rejected.** Lagging signal (latency only spikes after the µservice
is already overloaded); noisy (network jitter is indistinguishable
from real degradation); cannot distinguish `degraded` from `brownout`
because the throughput is intentionally capped in brown-out but the
per-request latency might still be nominal.

### Alt-2. Separate signal channel (gossip protocol, Redis pub/sub)

Publish degradation state to a side channel; callers subscribe.
**Rejected.** Adds substrate complexity (a new gossip layer per cell);
race conditions between the channel update and the request; the
in-band header is simpler and naturally per-request scoped.

### Alt-3. HTTP 503 only, no header

Use status 503 to mean "brown-out". **Rejected.** 503 is a *failure*
signal; brown-out is "succeeding but at reduced capacity". The
semantics are different. Status codes are the wrong axis. Also, gRPC
status codes don't map cleanly to brown-out.

## Consequences

### C-1. Positive

- **Static stability has a canonical input.** Per-cell fallback
  decisions read a single explicit signal.
- **Observability gains a first-class degradation panel.**
- **Mesh retry budgets reflect downstream reality.** Reduces retry-storm
  amplification.
- **Hyperscaler-grade.** Matches Cloudflare Workers brown-out signal,
  Google SRE guidance, Netflix Hystrix degradation hook.
- **Audit-evidence for postmortems.** State transitions land in the
  audit chain.

### C-2. Negative

- **Every µservice has to compute the gauge.** Mitigation: the kernel
  crate provides the computation; µservices just supply their SLO list
  and dependency list.
- **Header parsing happens on every request.** Mitigation: header
  parsing is microseconds; the mesh sidecar is already on the path.
- **Migration to header is a per-µservice rollout.** Mitigation:
  validator is advisory until rollout completes; tracker tracks
  per-µservice readiness.

### C-3. Sustainability

- The degradation header is the basis for power-aware load shedding:
  when a cell's PUE exceeds a threshold, the cell's µservices move to
  `degraded` class, biasing traffic to greener cells.

## Implementation surface

- `specs/brownout-degradation-signal.json` — canonical schema.
- `docs/standards/brownout-degradation-signal.md` — full standards
  doc.
- `microservices/observability/dashboards/brownout-degradation.md`
  — dashboard schema.
- `registry/brownout/coverage-tracker.tsv` — per-µservice rollout
  tracker.
- Validator lane `brownout-signal-coverage` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).
- Kernel implementation: `microservices/observability/` IP backlog item
  `IP-brownout-degradation-kernel` (planned; not authored here because
  the canonical-base observability stack is the right home, not
  PR-143).

## References

- Google SRE — *Graceful degradation* (ch. 22 of *Site Reliability
  Engineering* book).
- Cloudflare Engineering — *Worker brown-out signal* (2023 blog).
- Netflix Hystrix — *Circuit breaker + bulkhead pattern docs*.
- AWS Builders Library — *Avoiding insurmountable queue backlogs*.
- ADR-0128 (this portfolio) — hyperscaler invariants (static stability +
  circuit breaker).
- ADR-0042 (this portfolio) — observability stack (OTel + Prometheus).
- ADR-0148 (this portfolio) — Cilium service mesh.
- ADR-0009 (this portfolio) — cell architecture (static-stability hook).

## RESILIENCE-001 — messenger runtime-control-loop contract artifacts

The messenger service's brown-out / graceful-degradation runtime control loop
is specified — as a contract plus non-executable catalog fixtures, none applied
or executed at runtime — by the following PR-local artifacts (homed under the
`comms` capability per ADR-0562 §10.16 messenger move, enforced by the
`contract-slice-conformance` gate). As the brown-out/degradation-signal
authority, this ADR intentionally justifies their existence without granting any
runtime, production, or SLO-attainment claim:

- `comms/messenger/resilience/runtime-control-loop-contract.json` — the machine-readable contract (brownout classes, tail-sampling recipe, chaos-scenario + SLO-gate refs).
- `comms/messenger/chaos/scenarios/pod-kill.yaml` — non-executable chaos scenario catalog fixture referenced by the contract (per ADR-0165 chaos-engineering substrate).
- `comms/messenger/chaos/scenarios/network-delay-100ms.yaml` — non-executable chaos scenario catalog fixture referenced by the contract (ADR-0165).
- `comms/messenger/chaos/scenarios/dependency-failure.yaml` — non-executable chaos scenario catalog fixture referenced by the contract (ADR-0165).
- `comms/messenger/chaos/scenarios/disk-slow-1000ms.yaml` — non-executable chaos scenario catalog fixture referenced by the contract (ADR-0165).
- `comms/observability/slos/messenger/composition.openslo.yaml` — SLO composition joining the per-service messenger objectives referenced by the contract's `slo_gate_refs` (per ADR-0180 SLO-composition arithmetic + ADR-0139 SLO-home convention).
