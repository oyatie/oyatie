---
doc_class: ImplementationPlan
ip_id: IP-015
title: App composition-root binaries + branch-protection update + HG-ONT registration + OpenSLO manifests
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + axis-foundry
date: 2026-05-17
depends_on: [IP-001, IP-002, IP-003, IP-004, IP-005, IP-006, IP-007, IP-008, IP-009, IP-010, IP-011, IP-012, IP-013, IP-014]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-per-microservice-layout
  - oya-governance-authority-cohesion
  - oya-governance-hyperscaler-maturity-claims
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-*-app/
  - .github/branch-protection.yaml
  - /specs/hyperscaler-gates.json
  - microservices/ontology/slos/
doc_status: published
---


# IP-015: App binaries + branch-protection + HG-ONT + OpenSLO

## Intent

Final IP of the phase. Ship composition-root `*-app` binaries that wire usecase + adapter + rest + worker; update `.github/branch-protection.yaml` with the 5 new required lanes; register HG-ONT (hyperscaler maturity claim) in `/specs/hyperscaler-gates.json`; author the µservice's own OpenSLO manifests at `slos/`.

## Scope

In-scope:
- Composition-root `*-app` binaries per BC (10+ apps).
- `.github/branch-protection.yaml` diff per `PHASE-01-TYPED-ENTITY-SUBSTRATE.md` §"branch-protection.yaml diff preview".
- `release/ontology/{staging,production}` pattern protection rules.
- `/specs/hyperscaler-gates.json` HG-ONT entry.
- OpenSLO manifests at `microservices/ontology/slos/`:
  - `function-read-availability.openslo.yaml`
  - `function-read-latency.openslo.yaml`
  - `action-invocation-availability.openslo.yaml`
  - `audit-chain-emission-completeness.openslo.yaml`
  - `dynamic-layer-freshness.openslo.yaml`

## Implementation

| Step | Action |
|---|---|
| 1 | For each BC: scaffold `*-app` composition root binary; wire usecase + adapters + rest + worker |
| 2 | Author OpenSLO manifests (5 SLIs) |
| 3 | Update `.github/branch-protection.yaml` with 5 new lanes |
| 4 | Register HG-ONT in `/specs/hyperscaler-gates.json` per ADR-0123 |
| 5 | Register catalog records for all `*-app` crates |
| 6 | End-to-end drill: deploy to dev; AC-01..AC-14 of PRD pass |

## Verification

- `cargo build --workspace --all-features` — exit 0.
- All app smoke tests pass (startup + healthcheck + shutdown).
- `oya gate validate per-microservice-layout --microservice ontology` — exit 0.
- `oya gate validate authority-cohesion` — exit 0 (HG-ONT registers green).
- `oya gate validate hyperscaler-maturity-claims` — exit 0.
- All 5 OpenSLO manifests validate against OpenSLO v1.0 schema.
- All 5 new branch-protection lanes are required on dev + staging + release/ontology/{staging,production}.

## References

- ADR-0123 (hyperscaler maturity claim gate).
- ADR-0139 (SLO gate); ADR-0131 (per-microservice flat layout).
- `microservices/ontology/PHASE-01-TYPED-ENTITY-SUBSTRATE.md` §"branch-protection.yaml diff preview".
- `microservices/observability/PRD.md` §"OpenSLO manifest convention" (sibling pattern).


## A. Problem
`IP-015: App binaries + branch-protection + HG-ONT + OpenSLO` is not a generic implementation packet; it closes the `015 app binaries and branch protection` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `015 app binaries and branch protection` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
