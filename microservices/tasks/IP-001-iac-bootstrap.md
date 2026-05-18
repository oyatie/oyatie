---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-001: IaC bootstrap — Helm + Kustomize for Postgres + Redis + Meilisearch substrate

## Intent

Author Helm + Kustomize manifests for the tasks µservice substrate. Per
ADR-0131 (per-microservice flat layout): chart at
`microservices/tasks/iac/helm/tasks/`. Substrate components: Postgres 16
LTS for `task-store`/`project-list`/`dependency-graph` persistence (RLS
per-tenant per ADR-0117 + tenant-DEK envelope encryption per Bominal
ADR-0111); Redis 7.2 LTS for `view-engine` cache + presence; Meilisearch
0.10.0 LTS for `search-index` per-tenant cross-project index per
ADR-TASKS-0004 + ADR-TASKS-0001. OpenBao for per-tenant DEK envelope
encryption; secrets via `${openbao:secret/tasks/...}` references.
Pack-aware overlays (pack-kr ships first; eu/us/jp/sg/au/in/br/ae/ksa/
us-healthcare follow).

## ChangeSet boundary

7 Helm template files + Chart.yaml + values.yaml + Kustomize base
(kustomization + namespace) + 2 pack overlays (pack-kr + pack-eu first).
No Rust code; pure IaC + values. All secrets via
`${openbao:secret/tasks/...}` SecretReferences. Auto-assign fairness
PrometheusRule wired (ADR-TASKS-0006 EU AI Act Annex III §4 surface).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/iac/helm/tasks/Chart.yaml` | created | dependencies: meilisearch 0.10.0 |
| `microservices/tasks/iac/helm/tasks/values.yaml` | created | per-BC replica sizing; OpenBao SecretReferences |
| `microservices/tasks/iac/helm/tasks/templates/deployment.yaml` | created | per-BC Deployment (7 components) |
| `microservices/tasks/iac/helm/tasks/templates/service.yaml` | created | per-BC Service |
| `microservices/tasks/iac/helm/tasks/templates/hpa.yaml` | created | per-BC HPA (CPU 70%; min 3 max 100) |
| `microservices/tasks/iac/helm/tasks/templates/pdb.yaml` | created | PodDisruptionBudget min-available 50% |
| `microservices/tasks/iac/helm/tasks/templates/networkpolicy.yaml` | created | mesh-only ingress; egress to OpenBao + Postgres + Redis + Meilisearch + audit-chain + ontology + tenancy + workflow-engine |
| `microservices/tasks/iac/helm/tasks/templates/servicemonitor.yaml` | created | Prometheus scrape config |
| `microservices/tasks/iac/helm/tasks/templates/prometheusrule.yaml` | created | per-BC fast-burn + slow-burn alert rules + auto-assign-fairness alert (ADR-TASKS-0006) |
| `microservices/tasks/iac/kustomize/base/kustomization.yaml` | created | shared base |
| `microservices/tasks/iac/kustomize/base/namespace.yaml` | created | tasks namespace + restricted pod-security |
| `microservices/tasks/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created | initial active pack |
| `microservices/tasks/iac/kustomize/overlays/pack-eu/kustomization.yaml` | created | eu pack |
| (additional packs: us/us-healthcare/jp/sg/au/in/br/ae/ksa) | successor-IP | per-pack overlays |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/tasks/iac/helm/tasks
kubectl --dry-run=client apply -k microservices/tasks/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/tasks/iac/kustomize/overlays/pack-eu
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice tasks
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm lint + helm-test per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 7
  Deployments + Postgres + Redis + Meilisearch reach Ready within 10 min.
- Auto-assign fairness alert: synthetic skew injected; confirm alert fires.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per
  `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-TASKS-0001 (data model); ADR-TASKS-0004 (view-engine + board); ADR-TASKS-0006 (auto-assign fairness).
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- Redis cluster mode — `redis.io/docs/management/scaling/`.
- Meilisearch ops — `docs.meilisearch.com`.
