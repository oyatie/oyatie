---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: audit-chain
milestone: M01-foundation
phase: P01-audit-chain-substrate
impl_plan_id: IP-002-self-slo-manifest
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-audit-chain
co_owners: [axis-observability]
date: 2026-05-18
related_adrs: [ADR-0139, ADR-0131, ADR-0064]
acceptance_lanes: [openslo-conformance, per-microservice-layout, oya-vcs-promotion-readiness]
depends_on: [IP-001]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002 — Self-SLO manifests for audit-chain

## Goal

Author OpenSLO v1.0 manifests that describe audit-chain's own SLIs and SLOs at `microservices/audit-chain/slos/`. Per ADR-0131 §"SLO authoring mandatory before promotion" and ADR-0139 agentic SLO-gated promotion, audit-chain cannot promote past `dev` until these manifests exist, validate, and drive the observability µservice's burn-rate alerts. SLI catalog: `emit_latency`, `seal_latency`, `verify_latency`, `hsm_avail`, `cross_channel_root_match`.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/audit-chain/slos/emit_latency.openslo.yaml` | create | ~60 LoC |
| `microservices/audit-chain/slos/seal_latency.openslo.yaml` | create | ~60 LoC |
| `microservices/audit-chain/slos/verify_latency.openslo.yaml` | create | ~60 LoC |
| `microservices/audit-chain/slos/hsm_avail.openslo.yaml` | create | ~50 LoC |
| `microservices/audit-chain/slos/cross_channel_root_match.openslo.yaml` | create | ~70 LoC; correctness SLI (no silent root mismatch) |
| `microservices/audit-chain/slos/README.md` | create | ~80 LoC; SLI catalog overview |
| `microservices/audit-chain/dashboards/audit-chain-slo-overview.json` | create | ~200 LoC; Grafana dashboard JSON pre-built against these SLOs |
| `microservices/audit-chain/runbooks/burn-rate-alert-response.md` | create | ~120 LoC; per-SLO burn-rate playbook |
| `microservices/audit-chain/decisions/ADR-0139.md` | append §"audit-chain SLOs landed" | +6 LoC |

## Code shape

`slos/cross_channel_root_match.openslo.yaml` (the correctness-class SLI; excerpt):

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-audit-chain-cross-channel-root-match
  displayName: Audit-chain — cross-channel root match (correctness SLI)
  labels:
    microservice: audit-chain
    bounded_context: chain-of-chains
    tier: correctness-critical
spec:
  service: oya-audit-chain-verify
  indicator:
    metadata:
      name: cross-channel-root-match-sli
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            type: prometheus
            spec:
              query: 'sum(rate(oya_audit_chain_cross_channel_root_match_total[5m]))'
        total:
          metricSource:
            type: prometheus
            spec:
              query: 'sum(rate(oya_audit_chain_cross_channel_root_check_total[5m]))'
  objectives:
    - target: 1.000000  # 100% — correctness SLI: zero tolerance
      displayName: 100% Merkle roots match across mirror channels
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
  alertPolicies:
    - alertPolicyRef: oya-audit-chain-correctness-burn-fast
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `openslo_schema_validates_every_manifest` | per-manifest CI | All 5 manifests conform to OpenSLO v1.0 schema |
| `recording_rule_generation_from_manifests` | observability engine test | Each manifest produces ≥ 1 Mimir recording rule |
| `error_budget_calc_matches_window` | observability engine test | Budget calc per manifest reconciles with the 30d window |
| `burn_rate_alert_wired_to_alertmanager` | observability engine test | Each SLO produces a multi-window burn-rate alert (1h / 6h fast paths) |
| `correctness_sli_uses_target_1_0` | manifest schema test | `cross_channel_root_match` target is 1.0 (zero tolerance) |
| `dashboard_panels_reference_recording_rules` | dashboard schema test | Grafana JSON panels reference the generated recording rule names |

## Evidence to emit

- `evidence/microservices/audit-chain/slo-conformance-{date}.json` — per-manifest schema validation + recording-rule-generation report
- Audit-chain seal: `oya audit-chain seal --kind slo-conformance --ms audit-chain --window 30d`
- Metrics: each manifest's SLI emits `oya_audit_chain_<sli>_*` counters consumed by the observability engine

## Rollback procedure

1. Revert ChangeSet for `microservices/audit-chain/slos/`.
2. audit-chain promotion gate (`oya-vcs-promotion-readiness`) blocks any further promotion until manifests restored.
3. Existing burn-rate alerts continue firing on already-deployed manifests (no impact on live data).
4. Emit rollback evidence JSON; coordinate with observability owner.

## Blocking dependencies

- IP-001 — storage backend IaC (must be live so manifests can reference the real metric pipelines).
- ADR-0131 — per-µservice flat layout (manifests live at `microservices/audit-chain/slos/`).
- ADR-0139 — agentic SLO-gated promotion (consumer).

## Acceptance gates

```bash
cargo run -p oya-observability-slo-engine-rest -- validate \
  microservices/audit-chain/slos/*.openslo.yaml
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice audit-chain
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice audit-chain
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice audit-chain
```

## Halt conditions

- Any manifest fails OpenSLO v1.0 schema validation: STOP.
- Recording rule fails to load in Mimir: STOP.
- Correctness SLI target ≠ 1.0: STOP (governance-critical).

## Exit criteria

1. All 5 manifests validate.
2. All 6 tests green.
3. `openslo-conformance`, `per-microservice-layout`, `oya-vcs-promotion-readiness` lanes green.
4. Burn-rate alerts visible in AlertManager + Grafana OnCall.
5. Dashboard published.
6. Runbook published.
7. ADR-0139 status updated.

## Next IP

[`IP-003-emit-domain.md`](IP-003-emit-domain.md)

## References

- ADR-0139 — agentic SLO-gated promotion.
- ADR-0131 — per-microservice flat layout.
- ADR-0064 — canonical base + localization overlay.
- OpenSLO v1.0 — `https://openslo.com/`.
- Google SRE Workbook ch. 4 (SLOs) + ch. 5 (alerting on SLOs).
- microservices/observability/PRD.md.
