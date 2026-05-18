---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-001-layer-a-gpu-runner-pool-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-foundry
acceptance_lanes: [foundry-eval-iac-smoke, oya-governance-per-microservice-layout, ci-helm-lint]
---

# IP-001: Layer-A GPU Runner Pool IaC

## Intent

Helm chart for the GPU runner pool consumed by eval-runner-worker for case dispatch. gVisor / Kata sandbox enforced; per-pod ephemeral filesystem; per-pod NetworkPolicy restricts egress to provider model API allowlist.

## ChangeSet boundary

`microservices/foundry/iac/helm/gpu-runner-pool/`: Chart.yaml + values.yaml + templates/ (DaemonSet or Deployment + ServiceAccount + Role + RoleBinding + NetworkPolicy + PriorityClass).

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/gpu-runner-pool/Chart.yaml` | create |
| `iac/helm/gpu-runner-pool/values.yaml` | create |
| `iac/helm/gpu-runner-pool/templates/deployment.yaml` | create |
| `iac/helm/gpu-runner-pool/templates/networkpolicy.yaml` | create |
| `iac/helm/gpu-runner-pool/templates/priorityclass.yaml` | create |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/gpu-runner-pool/
helm template microservices/foundry/iac/helm/gpu-runner-pool/ | kubectl apply --dry-run=client -f -
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-eval
```

## References

- ADR-0024 §"Eval kernel".
- `microservices/foundry/runbooks/gpu-pool-rebalance.md`.
- threat-model.md T-E-01.
