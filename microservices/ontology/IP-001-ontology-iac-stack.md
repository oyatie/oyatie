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
  - oya-governance-per-microservice-layout
  - oya-governance-version-pinning-conformance
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
| 8 | LEAN lane `oya-governance-per-microservice-layout` validates ADR-0131 location compliance |

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


## A. Problem
`IP-001: Ontology IaC stack` is not a generic implementation packet; it closes the `001 ontology iac stack` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Kubernetes-first runtime placement for the ontology substrate with network policy, OpenBao secret binding, and independent SLO surfaces for registry, query, and audit-chain workers. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/ontology/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/openapi/ontology.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/proto/ontology.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/runbooks/type-registry-migration.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/iac/network-policy.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/iac/openbao-policy.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/ontology/PRD.md` and `microservices/ontology/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `ontology`.
2. Diff the declared contract in `microservices/ontology/contracts/openapi/ontology.yaml` and `microservices/ontology/contracts/proto/ontology.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/ontology/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/policy/tenant-scope.cedar`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and `microservices/ontology/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/ontology/PRD.md`
- `microservices/ontology/ARCHITECTURE.md`
- `microservices/ontology/contracts/openapi/ontology.yaml`
- `microservices/ontology/contracts/proto/ontology.proto`
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`
- `microservices/ontology/policy/tenant-scope.cedar`
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`
- `microservices/ontology/runbooks/type-registry-migration.md`
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml`
- `microservices/ontology/competitor-parity-matrix.md`
- `microservices/ontology/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `001 ontology iac stack` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
