---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-014-runtime-self-slo-manifests
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime + axis-observability
acceptance_lanes: [openslo-conformance, vcs-promotion-readiness]
---

# IP-014: Runtime self-SLO manifests (availability / latency / correctness / freshness)

## Intent

Author the four OpenSLO manifests under `microservices/foundry-runtime/slos/` so the observability µservice's SLO engine can gate foundry-runtime's `dev → staging → production` promotion per ADR-0130. This IP is the gateway from "feature complete" to "production-promotable" — without these manifests + verdict=eligible, no promotion path past `dev` per `agentic-slo-gated-promotion.json`.

## ChangeSet boundary

Four OpenSLO YAML manifests + a one-line `slos/README.md` pointer. No Rust crate changes.

## Concrete File Targets

| Path | Action |
|---|---|
| `slos/availability.openslo.yaml` | create (target 99.95% over 30d) |
| `slos/latency.openslo.yaml` | create (p99 ≤ 50ms over 30d; the headline scalability target) |
| `slos/correctness.openslo.yaml` | create (zero autonomy-bypass + zero cross-tenant; 100% target) |
| `slos/freshness.openslo.yaml` | create (cache age ≤ 30s; 99.5% target) |
| `slos/README.md` | create (one-line pointer to PRD §"Performance Targets") |

(Note: manifests already created in this artifact pack; this IP is the formal claim + acceptance gate.)

## Acceptance Gates

```bash
# Schema-validate every manifest against OpenSLO v1.0
for slo in microservices/foundry-runtime/slos/*.openslo.yaml; do
  cargo run -p oya-observability-slo-engine-rest -- validate $slo
done

# Verify slo-engine evaluator picks them up within hot-reload window
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice foundry-runtime

# Verify initial verdict at staging tier
cargo run -p oya-dev-cli -- gate validate vcs-promotion-readiness --sha <head-sha> --env staging --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| Manifest schema conformance | every file passes OpenSLO v1.0 validator |
| Reasonable thresholds (per `observability-slo.md`) | fast-burn ≤ 100% AND target ≥ 99% |
| PromQL feasibility | every indicator expression resolves against current Mimir tenant=oya-self |
| Initial verdict | once staging traffic flows, SLI accumulates over evaluator cadence; verdict transitions from `held` (bootstrap) to `eligible` |

## Halt Conditions

- Manifest sets unrealistic threshold (e.g., 99.999% with no measurement basis) — refactor to honest target.
- Indicator expression references non-existent metric — refactor.

## Next IP

[`IP-015-hg-fr-hyperscaler-gate-registration.md`](IP-015-hg-fr-hyperscaler-gate-registration.md)

## References

- `microservices/observability/PRD.md` (SLO engine consumes manifests).
- `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md`.
- `docs/standards/observability-slo.md` (cross-cutting OpenSLO authoring rules).
- ADR-0130.
- `microservices/foundry-runtime/PRD.md` §"Performance Targets" (source-of-truth thresholds).
