---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-001-host-pool-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: cloud-k8s
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Warm K8s node pool per pack via Cluster API

## Intent

Author Helm + Kustomize manifests for a warm Kubernetes node pool per pack via Cluster API. The pool maintains ≥ 2 standby nodes to absorb cell creation bursts within the ≤ 5-min p99 budget. Deploys to the per-pack management cluster.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart (oya-cell-k8s-cluster-api) + 1 shared Kustomize base + pack-kr overlay (initial active pack). No code; IaC only.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cell/iac/helm/k8s-cluster-api/Chart.yaml` | create | dep on upstream cluster-api chart pinned to LTS version |
| `microservices/cell/iac/helm/k8s-cluster-api/values.yaml` | create | MachineDeployment for warm pool; min/max replicas + bound auto-scaler |
| `microservices/cell/iac/kustomize/base/kustomization.yaml` | create | references Helm charts |
| `microservices/cell/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (region: ap-seoul-1; instance-types) |
| `microservices/cell/iac/terraform/cluster-api-rbac.tf` | create | Terraform-managed Cluster API RBAC per `policy/cell-boundary.md` |

## Crate Naming

n/a — IaC only.

## Code Shape

Helm chart skeleton (`k8s-cluster-api/values.yaml`):

```yaml
cluster-api:
  image:
    tag: "1.7.0"  # LTS pinned
clusterMachineDeployment:
  replicas: 2  # warm pool baseline
  template:
    spec:
      providerSpec:
        instanceType: "VM.Standard.E4.4-core"
        availabilityDomain: "${OCI_AD}"
warmPoolAutoscaler:
  minReplicas: 2
  maxReplicas: 20
  scaleUpTrigger: oya_cell_placement_queue_depth > 5
```

## Acceptance Gates

```bash
helm lint microservices/cell/iac/helm/k8s-cluster-api
kubectl --dry-run=client apply -k microservices/cell/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cell
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

Per PHASE-01 IaC class: ≥ 1 helm-install + helm-test smoke per chart. Test files under `microservices/cell/tests/iac/k8s-cluster-api.bats`.

## Halt Conditions

- Cluster API version drift from LTS pin — escalate.
- Cluster API CRD changes that conflict with current cell-lifecycle controller — block.

## Next IP

[`IP-002-cell-registry-postgres-schema.md`](IP-002-cell-registry-postgres-schema.md)

## References

- Bominal ADR-0009; ADR-0019.
- ADR-0117 (residency).
- Kubernetes Cluster API — `cluster-api.sigs.k8s.io`.
- `microservices/cell/capacity-model.md` §"Warm-Pool Sizing".
