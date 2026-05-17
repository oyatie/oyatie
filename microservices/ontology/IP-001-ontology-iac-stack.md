---
doc_class: ImplementationPlan
ip_id: IP-001
title: Ontology IaC stack (Postgres + Citus + ClickHouse + Cedar policy engine + Valkey + Kafka KRaft)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + cloud-iac
date: 2026-05-17
depends_on: []
acceptance_lanes:
  - helm-install-smoke
  - kustomize-build
  - oya-foundry-fitness-per-microservice-layout
  - oya-foundry-fitness-version-pinning-conformance
related_artifacts:
  - microservices/ontology/iac/helm/postgres/
  - microservices/ontology/iac/helm/clickhouse/
  - microservices/ontology/iac/helm/cedar-policy-engine/
  - microservices/ontology/iac/kustomize/base/
  - microservices/ontology/iac/kustomize/overlays/pack-kr/
doc_status: published
---

# IP-001: Ontology IaC stack

## Intent

Ship Helm + Kustomize charts for the Ontology µservice's Layer-A substrate (Postgres + Citus + ClickHouse + Cedar policy engine + Valkey + Kafka KRaft) under `microservices/ontology/iac/`. This is the foundational IP that scaffolds the µservice's deployment posture per ADR-0131.

## Scope

In-scope:
- `microservices/ontology/iac/helm/postgres/{Chart.yaml, values.yaml}` — Postgres 16 + Citus 12; coordinator + worker StatefulSets; FORCE RLS enabled; PITR via WAL-archiving.
- `microservices/ontology/iac/helm/clickhouse/{Chart.yaml, values.yaml}` — ClickHouse 24; ReplicatedMergeTree; row-policies enabled.
- `microservices/ontology/iac/helm/cedar-policy-engine/{Chart.yaml, values.yaml}` — Cedar v4 SDK sidecar / in-process distribution.
- `microservices/ontology/iac/kustomize/base/kustomization.yaml` — base manifests; references all Helm charts above.
- `microservices/ontology/iac/kustomize/overlays/pack-kr/kustomization.yaml` — pack-kr overlay; pinned to ap-seoul-1.

Out-of-scope:
- Layer-B `oya-ontology-*` pods — owned by IP-002..IP-015.
- Per-pack overlays beyond pack-kr — added on first-tenant onboarding per pack.
- Kafka KRaft + Valkey — depend on `cloud-secrets` µservice IaC; reused from there.

## Implementation

| Step | Action |
|---|---|
| 1 | Author `iac/helm/postgres/Chart.yaml` + `values.yaml`; pin Postgres 16.x + Citus 12.x via upstream chart |
| 2 | Author `iac/helm/clickhouse/Chart.yaml` + `values.yaml`; pin ClickHouse 24.x via Bitnami chart |
| 3 | Author `iac/helm/cedar-policy-engine/Chart.yaml` + `values.yaml`; install Cedar v4 sidecar binary |
| 4 | Author `iac/kustomize/base/kustomization.yaml` referencing Helm chart deployments |
| 5 | Author `iac/kustomize/overlays/pack-kr/kustomization.yaml` with `namespace: ontology`, `pack: kr`, OCI ap-seoul-1 zone-affinity |
| 6 | Run `helm template` + `kustomize build` to validate manifests |
| 7 | `helm-install-smoke` in kind cluster validates pods reach Ready |
| 8 | LEAN lane `oya-foundry-fitness-per-microservice-layout` validates ADR-0131 location compliance |

## Verification

- `helm template microservices/ontology/iac/helm/postgres | kubeval` — exit 0.
- `kustomize build microservices/ontology/iac/kustomize/overlays/pack-kr | kubectl apply --dry-run=server` — exit 0.
- `helm-install-smoke` in kind cluster: Postgres + Citus + ClickHouse + Cedar all reach Ready in ≤ 5 min.
- LEAN lanes green.

## References

- ADR-0131 (per-microservice flat layout).
- `microservices/ontology/PRD.md` §"Bounded Contexts".
- `microservices/ontology/capacity-model.md`.
- Citus charts — `docs.citusdata.com`.
- ClickHouse charts — `clickhouse.com/docs/en/install`.
- Cedar v4 — `cedarpolicy.com`.
