---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-012-supervisor-self-slos
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane + axis-observability
acceptance_lanes: [oya-check-openslo-conformance, oya-foundry-supervisor-canary-rollout-gated]
depends_on: [IP-011]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Supervisor self-SLOs (kill-switch, deployment, supervision-bus, autonomy)

## Intent

Author OpenSLO manifests at `microservices/foundry/slos/` so that observability µservice (per ADR-0139) gates supervisor's own promotion. Registers HG-FND-SUP per ADR-0123.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry/slos/kill-switch-engage.openslo.yaml` | create |
| `microservices/foundry/slos/deployment-admit.openslo.yaml` | create |
| `microservices/foundry/slos/supervision-event-lag.openslo.yaml` | create |
| `microservices/foundry/slos/autonomy-policy-eval.openslo.yaml` | create |
| `/specs/hyperscaler-gates.json` | update — register HG-FND-SUP |

## Sample OpenSLO

```yaml
# slos/kill-switch-engage.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: kill-switch-engage-latency
  displayName: Kill-switch engage latency
  labels:
    microservice: foundry-supervisor
    bc: kill-switch-circuit-breaker
spec:
  service: foundry-supervisor
  indicator:
    metadata:
      name: kill-switch-engage-latency-p99
    spec:
      ratioMetric:
        good:
          source: prometheus
          queryType: promql
          query: 'sum(rate(oya_supervisor_kill_switch_engage_latency_seconds_bucket{le="1.0"}[5m]))'
        total:
          source: prometheus
          queryType: promql
          query: 'sum(rate(oya_supervisor_kill_switch_engage_latency_seconds_count[5m]))'
  objectives:
    - displayName: 99.99% of engages within 1s
      target: 0.9999
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice foundry-supervisor
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims # registers HG-FND-SUP
```

## Halt Conditions

- OpenSLO target < 99 % for safety SLOs (kill-switch).
- Burn-rate fast-burn threshold > 100 %.

## Next IP

[`IP-013-sdk-rust-and-ts.md`](IP-013-sdk-rust-and-ts.md)

## References

- ADR-0139; ADR-0123 (HG-FND-SUP).
- `microservices/observability/PRD.md` AC-01.
- Google SRE Workbook ch. 5.

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
