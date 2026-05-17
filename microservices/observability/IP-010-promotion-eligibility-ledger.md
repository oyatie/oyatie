---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-010-promotion-eligibility-ledger
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-nextest, oya-governance-mimir-tenancy-enforced, oya-governance-mimir-recording-rule-conformance]
---

# IP-010: Promotion-eligibility ledger — Mimir-native recording rules

## Intent

Per ADR-0130 (revised) the ledger is **Mimir-native** (Prometheus recording rules emit aggregates; the time-series IS the ledger). No git-tracked JSONL. This IP authors the recording rules + ensures the adapter-mimir crate's verdict emission path produces the correct metric shapes per `/specs/agentic-slo-gated-promotion.json` §"promotion_eligibility_ledger".

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/iac/helm/mimir/recording-rules.yaml` | create — 2 recording rules per spec |
| `microservices/observability/iac/helm/mimir/alerting-rules.yaml` | create — burn-rate alert rule definitions per spec |
| `microservices/observability/src/crates/oya-observability-slo-engine-adapter-mimir/src/verdict_emitter.rs` | update — assert metric-emit shapes match spec |
| `microservices/observability/tests/integration/ledger_round_trip.rs` | create — emit verdict, query back via recording rule |

## Code Shape

```yaml
# recording-rules.yaml
groups:
  - name: oya-promotion-rules
    interval: 60s
    rules:
      - record: oya:current_verdict:by_microservice_env
        expr: max by (microservice, target_env, verdict) (oya_promotion_eligibility_verdict == 1)
      - record: oya:all_eligible:by_sha
        expr: min by (source_sha, target_env) (count by (microservice, source_sha, target_env, verdict) (oya_promotion_eligibility_verdict{verdict="eligible"}))
```

## Acceptance Gates

```bash
cargo nextest run -p oya-observability-slo-engine-adapter-mimir --test ledger_round_trip
cargo run -p oya-dev-cli -- gate validate mimir-recording-rule-conformance
cargo run -p oya-dev-cli -- gate validate mimir-tenancy-enforced
# Mimir promtool rule check:
promtool check rules microservices/observability/iac/helm/mimir/recording-rules.yaml
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_verdict_metric_shape_matches_spec` | metric name + labels + value match spec |
| `integration_ledger_round_trip` | emit verdict → recording rule pre-aggregates → query returns expected |
| `test_recording_rule_oya_all_eligible_by_sha` | 1 iff all microservices touched by SHA are eligible |

## Halt Conditions

- Recording-rule output drifts from `oya:current_verdict:by_microservice_env` spec — fix in spec or rule
- Audit-chain evidence not emitted alongside Mimir write — fail

## Next IP

[`IP-011-per-component-release-pointers.md`](IP-011-per-component-release-pointers.md)

## References

- ADR-0130 §"Layer-B item 12 — Promotion-eligibility ledger Mimir-native"
- `/specs/agentic-slo-gated-promotion.json` §"promotion_eligibility_ledger"
