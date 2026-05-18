---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-002-layer-a-postgres-clickhouse-golden-store-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-foundry
acceptance_lanes: [foundry-eval-iac-smoke, ci-helm-lint, oya-governance-per-microservice-layout]
---

# IP-002: Layer-A Postgres + ClickHouse + Golden-Output Store IaC

## Intent

Bundle: Helm charts for Postgres (eval-set metadata; CloudNativePG-backed), ClickHouse (parity-analytics MergeTree), and golden-store (MinIO + KMS integration). Each chart with HA configuration + per-pack region pinning.

## ChangeSet boundary

`microservices/foundry/iac/helm/{postgres,clickhouse,golden-store}/`: 3 charts × {Chart.yaml + values.yaml + templates/}.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/postgres/Chart.yaml` | create |
| `iac/helm/postgres/values.yaml` | create |
| `iac/helm/clickhouse/Chart.yaml` | create |
| `iac/helm/clickhouse/values.yaml` | create |
| `iac/helm/golden-store/Chart.yaml` | create |
| `iac/helm/golden-store/values.yaml` | create |

## Acceptance Gates

```bash
for chart in postgres clickhouse golden-store; do
  helm lint microservices/foundry/iac/helm/$chart/
  helm template microservices/foundry/iac/helm/$chart/ | kubectl apply --dry-run=client -f -
done
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-eval
```

## References

- ADR-0024.
- ADR-0117 (cloud-native infrastructure).
- `microservices/foundry/runbooks/clickhouse-rebalance.md`.
- `microservices/foundry/runbooks/golden-output-restore.md`.
