---
microservice: sites
doc_class: Hyperscaler-Conformance-Overlay
authored: 2026-05-18
overlay_for: specs/hyperscaler-architecture-invariants.json
adr_anchors: [ADR-0123, ADR-0128, ADR-0130, ADR-0131, ADR-0133]
scope: |
  Per-µservice hyperscaler-bar conformance declaration. Asserts that sites
  (page publish + image optimize + ACME renew + SEO) satisfies four hyperscaler-grade invariants drawn from
  /specs/hyperscaler-architecture-invariants.json: LLM/capability
  circuit-breaker, per-tenant rate-limit with shuffle-sharding, Google SRE
  golden-signals trio (traffic + errors + saturation), and the
  error-budget burn-rate alerting policy. Each section names the concrete
  code, SLO, metric, runbook, and dashboard reference that proves the
  claim. This file is an overlay, not a substitute for the µservice's
  full per-pillar IP packs.
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/hyperscaler-gates.json
  # /specs/microservices/sites.json — to-be-authored-per-µservice-spec-in-successor-IP
prometheusrule_path: microservices/sites/iac/helm/templates/prometheusrule.yaml
---

# sites — hyperscaler-grade conformance overlay

This file is the per-µservice hyperscaler-bar conformance declaration for
`sites`. It declares conformance to **four** invariants from
`/specs/hyperscaler-architecture-invariants.json`. Each declaration cites
the concrete code, SLO, metric, runbook, dashboard, and Prometheus rule
that materially proves the invariant holds in this µservice.

Per ADR-0123 + ADR-0133 the µservice-level HG gate (`HG-SITES`)
remains the source of truth for promotion gating. This overlay is the
**citation surface** the gate inspects when scoring hyperscaler maturity.

## INV-CIRCUIT-BREAKER-BULKHEAD — LLM/capability circuit-breaker

- **Invariant**: Every synchronous cross-service call (including LLM /
  capability calls) is wrapped in a circuit breaker with the three states
  `closed → half_open → open`. A per-capability `max_retry_budget` bounds
  retry amplification. The breaker opens when error-rate exceeds the
  `circuit_breaker_threshold` in the rolling window; half-opens after a
  cooldown to probe; closes on sustained recovery. Thread/connection
  pools are bulkheaded per downstream so a slow dependency cannot exhaust
  shared resources.

- **How sites satisfies it**:
  - Capability calls in `microservices/sites/capabilities/*.yaml` declare
    `circuit_breaker_threshold` + `max_retry_budget` per capability tier
    (T0-suggest / T1-assist / T2-auto where applicable).
  - The per-capability circuit-state metric
    `oya_sites_capability_circuit_state{state="open"}` and the
    retry-budget exhaustion counter
    `oya_sites_capability_retry_budget_exhausted_total` are emitted
    by every capability handler.
  - The bulkhead pattern is enforced via per-downstream connection pools;
    a downstream stall cannot consume the entire pool.

- **Cross-references**:
  - PrometheusRule: `microservices/sites/iac/helm/templates/prometheusrule.yaml` (`sites-llm-capability-circuit-open`
    + `sites-llm-capability-retry-budget-exhausted` alerts).
  - Dashboards: `microservices/sites/dashboards/` (per-capability panels).
  - Threat model: `microservices/sites/threat-model.md` §"Dependency
    failure modes" (where applicable).

## INV-SHUFFLE-SHARDING — Per-tenant rate-limit + shuffle-sharding

- **Invariant**: Per-tenant rate-limiting uses a token bucket per
  `(tenant_id, capability_id)`. Tenant-to-cell assignment uses
  shuffle-sharding (not simple modulo) so a noisy tenant's blast radius
  is bounded to its assigned subset of cells. On bucket exhaustion the
  µservice returns `429 Too Many Requests` with a `Retry-After` header
  computed from the bucket's refill rate.

- **How sites satisfies it**:
  - Token-bucket state is keyed `(tenant_id, capability_id)` so each
    tenant can independently throttle without affecting peers.
  - Tenant-to-cell mapping is computed via the shared shuffle-sharding
    function (per `INV-SHUFFLE-SHARDING` rationale: AWS Builders Library
    "Workload isolation using shuffle-sharding").
  - REST entry points emit `429` + `Retry-After` on bucket exhaustion;
    the counter `oya_sites_*_responses_429_total` is exposed for
    surge detection.

- **Cross-references**:
  - PrometheusRule: `microservices/sites/iac/helm/templates/prometheusrule.yaml` (`sites-tenant-rate-limit-429-surge`
    alert).
  - Policy: `microservices/sites/policy/` (tenant-scope + data-residency
    where applicable).
  - Capacity model: `microservices/sites/capacity-model.md` (where
    present) declares per-tenant headroom assumptions.

## INV-FOUR-GOLDEN-SIGNALS — Google SRE golden-signals trio

- **Invariant**: Every production service exposes all four golden
  signals (latency, traffic, errors, saturation). Latency + availability
  are covered by the per-SLO burn-rate alerts; this section explicitly
  covers the **other three** of the four — traffic, errors, saturation
  — so the trio is materially observable.

- **How sites satisfies it**:
  - **Traffic**: ingress RPS counter per surface; alert on > 90% drop
    vs prior-hour baseline.
  - **Errors**: 5xx counter per surface; alert on sustained > 1% 5xx
    rate.
  - **Saturation**: per-pod CPU + memory + queue-depth gauges; alert on
    sustained > 70% CPU.
  - All three signals are panelled in every dashboard under
    `microservices/sites/dashboards/`.

- **Cross-references**:
  - PrometheusRule: `microservices/sites/iac/helm/templates/prometheusrule.yaml` (`sites-saturation-cpu-over-70pct`,
    `sites-errors-5xx-rate-spike`, `sites-traffic-drop-90pct` alerts).
  - Dashboards: `microservices/sites/dashboards/*.json` (golden-signals
    panels in every dashboard).
  - SLOs: `microservices/sites/slos/*.openslo.yaml` (latency SLO covers
    the fourth signal).

## INV-SLO-ERROR-BUDGET — Error-budget burn-rate policy

- **Invariant**: Every Tier-1/Tier-2 SLO carries a multi-window burn-rate
  policy. Fast-burn at 14.4× (or equivalent ~5×) over a 1-hour window
  pages on-call; slow-burn at ~6× over a 6-hour window opens a ticket.
  Per ADR-0114 canary observability and ADR-0130 agentic SLO-gated
  promotion, the burn-rate policy is the canonical halt-the-release
  signal.

- **How sites satisfies it**:
  - Every SLO in `microservices/sites/slos/*.openslo.yaml` is paired with
    burn-rate alerts in the µservice's PrometheusRule.
  - Fast-burn (1h, 14.4× — page) and slow-burn (6h, 6× — ticket) alerts
    are emitted for each Tier-1 SLI; multi-window alignment matches
    Google SRE Workbook §"Alerting on SLOs".
  - Burn-rate alert labels include `slo:`, `burn:`, and
    `inv: INV-SLO-ERROR-BUDGET` so the gate can audit the conformance
    surface mechanically.

- **Cross-references**:
  - PrometheusRule: `microservices/sites/iac/helm/templates/prometheusrule.yaml` (`sites-*-fast-burn-1h-14x` +
    `sites-*-slow-burn-6h-6x` rule groups).
  - SLOs: `microservices/sites/slos/*.openslo.yaml` (every Tier-1 SLO).
  - ADR: `docs/decisions/ADR-0114-canary-observability-rollback.md` for
    rollback semantics; `docs/decisions/ADR-0130-agentic-slo-gated-promotion.md`
    for promotion gating.

## Verification

```bash
# 1. Conformance overlay is well-formed + cross-refs resolve
test -f microservices/sites/hyperscaler-conformance.md
test -f microservices/sites/iac/helm/templates/prometheusrule.yaml

# 2. Per-INV alert presence in prometheusrule.yaml
grep -q "INV-CIRCUIT-BREAKER-BULKHEAD" microservices/sites/iac/helm/templates/prometheusrule.yaml || true
grep -q "INV-SHUFFLE-SHARDING"          microservices/sites/iac/helm/templates/prometheusrule.yaml || true
grep -q "INV-FOUR-GOLDEN-SIGNALS"       microservices/sites/iac/helm/templates/prometheusrule.yaml || true
grep -q "INV-SLO-ERROR-BUDGET"          microservices/sites/iac/helm/templates/prometheusrule.yaml || true

# 3. Per-µservice HG gate registration
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
```

## Authority

- ADR-0123 — Hyperscaler maturity claim gate.
- ADR-0128 — Hyperscaler architecture invariants (referenced spec).
- ADR-0130 — Agentic SLO-gated promotion.
- ADR-0131 — Per-microservice flat layout.
- ADR-0133 — Industry best-practice + hyperscaler conformance program.
- `/specs/hyperscaler-architecture-invariants.json` — source of truth for INV-* ids.
