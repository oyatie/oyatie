---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-013-observability-slo-manifests
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + axis-observability
acceptance_lanes: [oya-governance-openslo-conformance, oya-governance-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: workflow-engine OpenSLO manifests + observability self-SLOs

## Intent

Author OpenSLO manifests for workflow-engine self-SLOs at `microservices/workflow-engine/slos/`. These are consumed by the observability µservice's promotion gate (per ADR-0139) — workflow-engine cannot advance past dev until its SLOs are green.

## ChangeSet boundary

4 OpenSLO manifests (one per SLI: availability, latency, correctness, freshness).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/workflow-engine/slos/availability.openslo.yaml` | create | Engine REST availability (99.95% monthly); 30d window; multi-window multi-burn-rate |
| `microservices/workflow-engine/slos/latency.openslo.yaml` | create | Step execution latency p99 ≤ 200ms; event-to-action p99 ≤ 500ms |
| `microservices/workflow-engine/slos/correctness.openslo.yaml` | create | Deterministic-replay verification: 100% identical step sequence |
| `microservices/workflow-engine/slos/freshness.openslo.yaml` | create | Audit-chain seal latency p99 ≤ 1s |

## Code Shape

```yaml
# slos/latency.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: workflow-engine-step-latency
  displayName: Workflow engine step execution latency
  labels:
    microservice: workflow-engine
    sli: latency
spec:
  description: |
    Step execution latency p99 ≤ 200ms for local actions.
    Network-bound external actions excluded.
  service: workflow-engine
  indicator:
    metadata:
      name: step-latency
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            metricSourceRef: mimir
            type: Prometheus
            spec:
              query: |
                sum(rate(oya_workflow_engine_step_duration_seconds_bucket{le="0.2",kind="local"}[5m]))
        total:
          metricSource:
            metricSourceRef: mimir
            type: Prometheus
            spec:
              query: |
                sum(rate(oya_workflow_engine_step_duration_seconds_count{kind="local"}[5m]))
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
  objectives:
    - displayName: Step execution latency p99 ≤ 200ms
      target: 0.999
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice workflow-engine
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice workflow-engine
```

## Test Plan

- Per `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` §"OpenSLO conformance": OpenSLO v1.0 schema validation; burn-rate thresholds in sane bounds.

## Next IP

[`IP-014-branch-protection-and-hyperscaler-gates.md`](IP-014-branch-protection-and-hyperscaler-gates.md)

## References

- ADR-0139 (agentic SLO-gated promotion)
- `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md`
- `docs/standards/observability-slo.md`
- PRD §"Performance Targets"

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-013-observability-slo-manifests.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
