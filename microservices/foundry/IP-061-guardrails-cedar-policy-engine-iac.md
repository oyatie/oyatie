---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-001-cedar-policy-engine-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, helm-lint, kubectl-apply-dry-run, cedar-default-deny-enforced, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Cedar v4 policy engine + bundle IaC

## Intent

Author Helm manifest for in-cluster Cedar engine serving (sidecar pattern + standalone evaluator pool); Cedar v4 policy bundle authoring + validation pipeline; per-pack bundle compilation pipeline. Deploys to the dedicated foundry Kubernetes cluster (sibling to foundry-runtime / foundry-providers). Versions pinned to LTS per `docs/standards/lts-versions-verified.md`.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (cedar engine) + Cedar bundle compilation pipeline + per-pack bundle Kustomize overlay (pack-kr at M01) + LEAN lane definition stub. No code; pure IaC + policy text. Cedar fragments themselves are authored in IP-006 (autonomy-tier-gate adapter); this IP provides the substrate.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/cedar-engine/Chart.yaml` | create | Cedar v4 in-cluster serving |
| `microservices/foundry/iac/helm/cedar-engine/values.yaml` | create | replica counts per `capacity-model.md`; image tag pinned to Cedar v4 LTS |
| `microservices/foundry/iac/helm/cedar-engine/values-pack-kr.yaml` | create | pack-kr overlay |
| `microservices/foundry/iac/kustomize/base/kustomization.yaml` | create | shared base (already in directory; this IP populates) |
| `microservices/foundry/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay |
| `microservices/foundry/policy/cedar-base.cedar` | create | base policy (default-deny) |
| `microservices/foundry/policy/schema.cedarschema` | create | Cedar v4 schema for entities + actions |
| `microservices/foundry/iac/cedar/build.sh` | create | bundle compilation pipeline (validates + signs bundle SHA) |

## Crate Naming

n/a — IaC + policy text only.

## Code Shape

Helm chart skeleton (`cedar-engine/values.yaml`):

```yaml
cedarEngine:
  image:
    registry: docker.io
    repository: oyatie/cedar-engine
    tag: "v4.3.0"   # Cedar v4 LTS pin per docs/standards/lts-versions-verified.md
  replicas:
    standalone: 2       # batch evaluator pool
  bundle:
    source: configmap
    configMapName: cedar-bundle-${pack}
    refreshInterval: 30s
  defaultDeny:
    enforced: true      # MANDATORY per policy/guardrail-enforcement.md (Slice D)
  metrics:
    enabled: true
    port: 9100
```

Cedar base (`policy/cedar-base.cedar`):

```cedar
// foundry-guardrails — base default-deny policy
// Cedar v4 schema per microservices/foundry/policy/schema.cedarschema
//
// Purpose: enforce default-deny on every action; per-action permits live in
//          tenant-scope.cedar / ci-scope.cedar / auditor-scope.cedar / public-read.cedar
//          plus per-tenant overlays loaded from Postgres rule-store.
// Owner: ops-security + axis-foundry-guardrails

// =============================================================================
// Default deny — every action refused unless an explicit permit matches.
// Per ADR-0140 (retired per ADR-0145) substrate; verified by oya-foundry-fitness-cedar-default-deny-enforced lane.
// =============================================================================

forbid (
  principal,
  action,
  resource
);
```

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/cedar-engine
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-guardrails
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate cedar-default-deny-enforced
bash microservices/foundry/iac/cedar/build.sh microservices/foundry/policy
```

## Test Plan

- Per PHASE-01 §"Per-IP Test Coverage Threshold" IaC class: ≥ 1 helm-install + helm-test smoke per chart.
- `microservices/foundry/tests/iac/cedar-engine.bats` runs `helm install --dry-run` + `helm test`.
- E2E: kind cluster; apply pack-kr overlay; verify cedar-engine pod reaches Ready within 5 min; verify default-deny applies on a synthetic request.

## Halt Conditions

- Cedar version drift from LTS pin — escalate to `docs/standards/lts-versions-verified.md` PR.
- Cedar bundle compilation fails on default-deny missing — refuse merge.
- OpenBao secret-reference resolution failure — block; engage cloud-secrets.

## Next IP

[`IP-002-classifier-model-serving-iac.md`](IP-002-classifier-model-serving-iac.md)

## References

- ADR-0140: Cedar policy substrate.
- ADR-0131: Per-microservice flat layout.
- `microservices/foundry/multi-region.md`.
- `microservices/foundry/capacity-model.md`.
- `docs/standards/lts-versions-verified.md`.
- Cedar docs — `cedarpolicy.com`.
