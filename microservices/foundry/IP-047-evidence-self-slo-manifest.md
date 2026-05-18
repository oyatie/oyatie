---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-002-self-slo-manifest
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [openslo-validate, oya-governance-self-slo-coverage]
---

# IP-002: Self-SLO manifest (OpenSLO at microservices/foundry/slos/)

## Intent

Author OpenSLO v1 manifests for foundry-evidence's own SLIs so the `observability` µservice gates this µservice's own promotion identically to every other µservice.

## ChangeSet boundary

Pure SLO manifests. No code.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/slos/record-invocation-latency.yaml` | create | p99 ≤ 500 ms over 30d; 99.9 % budget |
| `microservices/foundry/slos/pack-assembly-latency.yaml` | create | p99 ≤ 2 s over 30d; 99 % budget |
| `microservices/foundry/slos/pack-assembly-success-rate.yaml` | create | ≥ 99.99 % over 30d |
| `microservices/foundry/slos/evidence-query-latency.yaml` | create | p99 ≤ 100 ms over 30d; 99 % budget |
| `microservices/foundry/slos/regulator-export-assembly-latency.yaml` | create | p99 ≤ 30 s per 10k packs over 30d; 99 % budget |
| `microservices/foundry/slos/audit-chain-emit-backlog-depth.yaml` | create | p99 ≤ 60 s over 30d (≤ 600 s breach → Sev-1) |
| `microservices/foundry/slos/audit-chain-bridge-availability.yaml` | create | ≥ 99.99 % over 30d |
| `microservices/foundry/slos/regulator-export-delivery-success-rate.yaml` | create | ≥ 99.9 % over 90d |
| `microservices/foundry/slos/archive-cascade-lag.yaml` | create | p99 ≤ 24 h over 30d |

## Acceptance Gates

```bash
openslo validate microservices/foundry/slos/*.yaml
cargo run -p oya-dev-cli -- gate validate self-slo-coverage --microservice foundry-evidence
```

## Halt Conditions

- Any SLO target lower than declared in `PRD.md` NFR table — block (no silent regression).
- SLI series missing from Mimir (e.g., the relevant Prometheus metric not exposed by code) — block until source code ships the metric.

## Next IP

[`IP-003-capability-invocation-recorder-kernel.md`](IP-003-capability-invocation-recorder-kernel.md)

## References

- ADR-0130 (agentic SLO-gated promotion).
- `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md`.
- `docs/standards/observability-slo.md`.
