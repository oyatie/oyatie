---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-003-k8s-operator-iac
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [helm-lint, helm-install-smoke, kubectl-validate, oya-check-operator-rbac-conformance]
depends_on: [IP-001, IP-002]
---

# IP-003: Kubernetes Operator + CRDs + RBAC

## Intent

Helm chart for the Foundry supervisor Kubernetes Operator (kube-rs controller-runtime), four CRDs (`Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch`), per-tenant RBAC, admission webhook + OPA Gatekeeper integration.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry-supervisor/iac/helm/supervisor-controller/Chart.yaml` | create |
| `microservices/foundry-supervisor/iac/helm/supervisor-controller/values.yaml` | create |
| `microservices/foundry-supervisor/iac/helm/supervisor-controller/templates/{crds,rbac,deployment,admission-webhook}.yaml` | create |

## Substrate selections

- kube-rs 0.96+ (controller-runtime port to Rust).
- Four CRDs: `Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch`.
- 3 controller replicas (lease-leadership-elected).
- Admission webhook signed by cert-manager certificate.
- OPA Gatekeeper policy: refuse CRD mutations not originating from supervisor SA.

## Acceptance Gates

```bash
helm lint microservices/foundry-supervisor/iac/helm/supervisor-controller
helm install --dry-run --debug -n foundry-supervisor supervisor microservices/foundry-supervisor/iac/helm/supervisor-controller
kubectl apply --dry-run=server -f microservices/foundry-supervisor/iac/helm/supervisor-controller/templates/crds/*.yaml
cargo run -p oya-dev-cli -- gate validate operator-rbac-conformance --microservice foundry-supervisor
```

## Halt Conditions

- ClusterRole permits any `*` verb.
- Admission webhook missing OPA Gatekeeper enforcement.

## Next IP

[`IP-004-agent-fleet-lifecycle-kernel.md`](IP-004-agent-fleet-lifecycle-kernel.md)

## References

- kube-rs — `kube.rs`.
- Kubernetes Operator pattern — `kubernetes.io/docs/concepts/extend-kubernetes/operator/`.
- OPA Gatekeeper — `open-policy-agent.github.io/gatekeeper/`.
- `policy/supervisor-isolation.md` TI-K-*.
