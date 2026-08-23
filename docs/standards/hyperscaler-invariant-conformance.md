---
doc_class: Hyperscaler-Invariant-Conformance-Standard
authored: 2026-05-18
canonical_authority: ADR-0064
overlay_for: specs/hyperscaler-architecture-invariants.json
adr_anchors:
  - ADR-0064
  - ADR-0123
  - ADR-0128
  - ADR-0139
  - ADR-0131
  - ADR-0133
related_specs:
  - specs/hyperscaler-architecture-invariants.json
  - specs/hyperscaler-gates.json
canonical_prometheusrule: microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
metric_naming_convention: microservices/observability/contracts/metric-naming-convention.md
status: canonical-base
---

# Hyperscaler-invariant conformance — canonical standard

This standard is the **canonical-base declaration** for the four
hyperscaler-grade architecture invariants drawn from
`specs/hyperscaler-architecture-invariants.json`. Per ADR-0064
canonical-base-and-localization-packs, this document is the seam; the
canonical PrometheusRule at
`microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
is the impl; each microservice supplies pure value variation by emitting
metrics under the convention documented at
`microservices/observability/contracts/metric-naming-convention.md`.

Before SWEEP-H, twenty microservices each shipped a near-identical
`hyperscaler-conformance.md` overlay declaring the same four invariants
with only the microservice name varying. Roughly 1,600 lines of
duplicated content existed only because of the µservice slug. Per
ADR-0064 the cleanest form is a single normative declaration here, with
each microservice supplying its conformance citation through
`microservices/<ms>/manifest.json#hyperscaler_inv_coverage` pointing at
the canonical alert names + canonical PrometheusRule path.

Per ADR-0123 + ADR-0133 the per-microservice HG gate (`HG-<MS>`) remains
the source of truth for promotion gating. This standard declares the
normative requirements every microservice MUST satisfy to claim
hyperscaler-bar conformance.

## Normative requirements

Every microservice in `microservices/<ms>/` MUST:

1. **Emit metrics under the canonical naming convention.** See
   `microservices/observability/contracts/metric-naming-convention.md`.
   Specifically, every microservice MUST emit:
   - `<ms>_capability_circuit_state` (gauge, with `state` label)
   - `<ms>_capability_retry_budget_exhausted_total` (counter)
   - `<ms>_responses_429_total` (counter, with `tenant_id` label)
   - `<ms>_responses_5xx_total` (counter)
   - `<ms>_responses_total` (counter)
   - `<ms>_request_success_total` (counter)
   - `<ms>_request_total` (counter)

   All metrics MUST carry the `microservice=<ms>` label so the canonical
   PrometheusRule can resolve the offending microservice in alert
   annotations via `{{ $labels.microservice }}`.

2. **Reference the canonical PrometheusRule from manifest.json.** The
   `microservices/<ms>/manifest.json#hyperscaler_inv_coverage` field MUST
   reference the canonical PrometheusRule path and the canonical alert
   names:
   - `circuit_breaker`: cites `OyaCapabilityCircuitOpen` (and/or
     `OyaCapabilityRetryBudgetExhausted`)
   - `tenant_rate_limit`: cites `OyaTenantRateLimit429Surge`
   - `golden_signals`: cites `OyaSaturationCpuOver70pct` (and/or the
     other golden-signal alerts)
   - `error_budget_burn`: cites `OyaErrorBudgetFastBurn1h14x` (and/or
     `OyaErrorBudgetSlowBurn6h6x`)

3. **NOT re-declare these four INV groups in the microservice's own
   prometheusrule.yaml.** Per-microservice PrometheusRule files are
   reserved for per-SLO burn-rate alerts (which reference per-SLI metric
   names that vary across microservices) and per-microservice
   operational alerts (e.g. `NotesE2ELeakageDetected`,
   `MeetLiveStreamEgressUnauthorized`) that do not generalize.

4. **Apply the `app.kubernetes.io/name=<ms>` Kubernetes label to every
   pod.** This allows the canonical saturation alert to join
   `container_cpu_usage_seconds_total` against `kube_pod_labels` and
   resolve a `microservice` label without per-µservice config. The
   standard Helm chart `_helpers.tpl` pattern already enforces this.

## The four invariants

### INV-CIRCUIT-BREAKER-BULKHEAD — LLM/capability circuit-breaker

**Invariant.** Every synchronous cross-service call (including LLM /
capability calls) is wrapped in a circuit breaker with the three states
`closed → half_open → open`. A per-capability `max_retry_budget` bounds
retry amplification. The breaker opens when error-rate exceeds the
`circuit_breaker_threshold` in the rolling window; half-opens after a
cooldown to probe; closes on sustained recovery. Thread/connection
pools are bulkheaded per downstream so a slow dependency cannot exhaust
shared resources.

**Canonical alerts** (in
`hyperscaler-invariants-canonical-prometheusrule.yaml`):
`OyaCapabilityCircuitOpen` (page),
`OyaCapabilityRetryBudgetExhausted` (page).

**Per-microservice surface**: emit
`<ms>_capability_circuit_state{state="open"}` and
`<ms>_capability_retry_budget_exhausted_total`; declare
`circuit_breaker_threshold` + `max_retry_budget` per capability tier in
`microservices/<ms>/capabilities/*.yaml`.

### INV-SHUFFLE-SHARDING — Per-tenant rate-limit + shuffle-sharding

**Invariant.** Per-tenant rate-limiting uses a token bucket per
`(tenant_id, capability_id)`. Tenant-to-cell assignment uses
shuffle-sharding (not simple modulo) so a noisy tenant's blast radius
is bounded to its assigned subset of cells. On bucket exhaustion the
microservice returns `429 Too Many Requests` with a `Retry-After`
header computed from the bucket's refill rate.

**Canonical alert** (in
`hyperscaler-invariants-canonical-prometheusrule.yaml`):
`OyaTenantRateLimit429Surge` (ticket).

**Per-microservice surface**: emit
`<ms>_responses_429_total{tenant_id=...}`; route tenant-to-cell
assignment through the shared shuffle-sharding function (AWS Builders
Library "Workload isolation using shuffle-sharding").

### INV-FOUR-GOLDEN-SIGNALS — Google SRE golden-signals trio

**Invariant.** Every production service exposes all four golden signals
(latency, traffic, errors, saturation). Latency + availability are
covered by per-SLO burn-rate alerts in each microservice's own
prometheusrule (they vary by SLI); this invariant explicitly covers the
**other three** of the four — traffic, errors, saturation — so the trio
is materially observable.

**Canonical alerts** (in
`hyperscaler-invariants-canonical-prometheusrule.yaml`):
`OyaSaturationCpuOver70pct` (ticket; sustained CPU > 70% / 10m),
`OyaErrors5xxRateSpike` (page; 5xx > 1% / 5m),
`OyaTrafficDrop90pct` (page; RPS drop > 90% vs prior-hour baseline).

**Per-microservice surface**: emit `<ms>_responses_total`,
`<ms>_responses_5xx_total`; carry the
`app.kubernetes.io/name=<ms>` Kubernetes label on every pod;
panel all three signals in every dashboard under
`microservices/<ms>/dashboards/`.

### INV-SLO-ERROR-BUDGET — Error-budget burn-rate policy

**Invariant.** Every Tier-1 / Tier-2 SLO carries a multi-window
burn-rate policy. Fast-burn at 14.4× (or equivalent ~5×) over a 1-hour
window pages on-call; slow-burn at ~6× over a 6-hour window opens a
ticket. Per ADR-0114 canary observability and ADR-0139 agentic
SLO-gated promotion, the burn-rate policy is the canonical
halt-the-release signal.

**Canonical alerts** (in
`hyperscaler-invariants-canonical-prometheusrule.yaml`):
`OyaErrorBudgetFastBurn1h14x` (page; aggregate),
`OyaErrorBudgetSlowBurn6h6x` (ticket; aggregate).

**Per-microservice surface**: per-SLO burn-rate alerts remain in
`microservices/<ms>/iac/helm/<chart>/templates/prometheusrule.yaml`
(they reference per-SLI metric names that do not generalize). The
canonical aggregate alerts above provide the `inv:INV-SLO-ERROR-BUDGET`
label so the conformance gate can audit by label query without scanning
every per-SLO alert. Microservices also MUST emit
`<ms>_request_success_total` + `<ms>_request_total` so the
aggregate canonical alerts compute a meaningful burn rate.

## Verification

```bash
# 1. Canonical PrometheusRule exists and declares all four INVs
test -f microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
grep -q "INV-CIRCUIT-BREAKER-BULKHEAD" microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
grep -q "INV-SHUFFLE-SHARDING"         microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
grep -q "INV-FOUR-GOLDEN-SIGNALS"      microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
grep -q "INV-SLO-ERROR-BUDGET"         microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml

# 2. No per-microservice prometheusrule re-declares the four INV groups
test -z "$(grep -rln 'inv: INV-' microservices/*/iac/helm/*/templates/prometheusrule.yaml microservices/*/iac/helm/templates/prometheusrule.yaml 2>/dev/null)"

# 3. Every manifest.json#hyperscaler_inv_coverage cites the canonical alert names
python3 -c "
import json, glob
ok = True
for m in glob.glob('microservices/*/manifest.json'):
    j = json.load(open(m))
    cov = j['hyperscaler_inv_coverage']
    needed = ['OyaCapabilityCircuitOpen','OyaTenantRateLimit429Surge','OyaSaturationCpuOver70pct','OyaErrorBudgetFastBurn1h14x']
    for fld, alert in zip(['circuit_breaker','tenant_rate_limit','golden_signals','error_budget_burn'], needed):
        if alert not in cov[fld]:
            print(f'{m}: {fld} missing canonical alert {alert}'); ok = False
print('ok' if ok else 'FAIL')"

# 4. Per-microservice HG gate registration
cargo run -p dev-cli -- gate validate hyperscaler-maturity-claims
```

## Authority

- ADR-0064 — canonical-base-and-localization-packs (this standard is
  the seam; the canonical PrometheusRule is the impl).
- ADR-0123 — hyperscaler maturity claim gate.
- ADR-0128 — hyperscaler architecture invariants (the four INV-* ids).
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0131 — per-microservice flat layout.
- ADR-0133 — industry best-practice + hyperscaler conformance program.
- `specs/hyperscaler-architecture-invariants.json` — source of truth for
  INV-* ids.
- `microservices/observability/contracts/metric-naming-convention.md` —
  canonical-base metric-naming contract.
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
  — canonical PrometheusRule (the impl).
