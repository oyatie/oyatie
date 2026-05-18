---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-001-runtime-cluster-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-foundry-runtime
acceptance_lanes: [helm-install-smoke, kustomize-build, foundry-runtime-iac-smoke, oya-governance-per-microservice-layout]
---

# IP-001: Runtime cluster IaC (Kubernetes + Istio + SPIRE + OpenBao bindings)

## Intent

Ship Helm charts + Kustomize overlays for the foundry-runtime dedicated cluster: pod baseline (seccomp + AppArmor + non-root + RO FS), Istio mesh integration (mTLS + SPIFFE), SPIRE wiring, OpenBao SecretReference materialisation, NetworkPolicy default-deny + sibling-allowlist.

## ChangeSet boundary

All paths under `microservices/foundry/iac/`. No Rust crate changes in this IP.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/runtime-pool/Chart.yaml` | create (LTS-pinned Kubernetes 1.31 baseline) |
| `iac/helm/runtime-pool/values.yaml` | create (per-pack capacity per `capacity-model.md` XS tier) |
| `iac/kustomize/base/kustomization.yaml` | create (shared base) |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | create (pack-kr overlay) |
| `iac/kustomize/base/namespace.yaml` | create (Pod Security Standards `restricted`) |
| `iac/kustomize/base/networkpolicy-default-deny.yaml` | create |
| `iac/kustomize/base/networkpolicy-sibling-allowlist.yaml` | create (mTLS to providers/guardrails/evidence/supervisor) |
| `iac/kustomize/base/openbao-secret-references.yaml` | create |
| `iac/terraform/spire-server.tf` | create (SPIRE server config-as-code) |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/runtime-pool/
helm template microservices/foundry/iac/helm/runtime-pool/ --values microservices/foundry/iac/helm/runtime-pool/values.yaml > /tmp/runtime-pool-rendered.yaml
kubectl apply --dry-run=server -f /tmp/runtime-pool-rendered.yaml
kustomize build microservices/foundry/iac/kustomize/overlays/pack-kr/
cargo run -p oya-dev-cli -- gate validate foundry-runtime-iac-smoke
```

End-to-end kind smoke: deploy charts to ephemeral kind cluster; verify pods Ready ≤2min; verify mTLS handshake via Istio; verify NetworkPolicy refuses non-sibling egress.

## Test Plan

| Test | Verifies |
|---|---|
| `helm-install-smoke.sh` | Charts deploy clean against kind |
| `pod-security-baseline.sh` | Every runtime pod runs non-root + RO FS + seccomp + AppArmor |
| `networkpolicy-default-deny.sh` | Non-sibling egress refused |
| `spire-svid-issued.sh` | Each runtime pod receives SPIFFE SVID within 30s of start |

## Halt Conditions

- Chart deploys but pod hardening missing (seccomp / AppArmor / non-root) — refactor.
- mTLS not enforced on inter-pod traffic.

## Next IP

[`IP-002-redis-and-postgres-baseline.md`](IP-002-redis-and-postgres-baseline.md)

## References

- ADR-0117 (cloud-native infra); ADR-0131; `policy/runtime-isolation.md` TI-10, TI-11.
- Kubernetes Pod Security Standards — `kubernetes.io/docs/concepts/security/pod-security-standards/`.
- SPIRE — `spiffe.io`.
- Istio mTLS — `istio.io/latest/docs/concepts/security/`.
