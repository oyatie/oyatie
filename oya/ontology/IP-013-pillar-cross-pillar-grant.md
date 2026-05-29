---
doc_class: ImplementationPlan
ip_id: IP-013
title: pillar (org / person pillar enforcement + cross-pillar grant Cedar flow)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + council-privacy
date: 2026-05-17
depends_on: [IP-006]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-cedar-coverage
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-pillar-{kernel,domain,usecase}/
  - microservices/ontology/policy/pillar.cedar
doc_status: published
---


# IP-013: pillar (org / person + cross-pillar grant)

## Intent

Author the pillar BC per Bominal ADR-0132 — typed Object Types declared at `org-pillar` or `person-pillar`; cross-pillar reads forbidden unless explicit Cedar `CrossPillarGrant` issued via 2-person rule.

## Scope

In-scope:
- `oya-ontology-pillar-{kernel,domain,usecase}` crates (no adapter — pure logic via Cedar).
- `pillar.cedar` policy fragment authored.
- Cross-pillar grant data model: principal, allowed_pillars, data_class_cap, expires_at (≤ 30 d), signed_by[2].
- 2-person rule enforcement at grant issuance.
- Audit-chain emit on grant issued + used + revoked + expired.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 3 crates |
| 2 | Author `pillar.cedar` policy fragment |
| 3 | Author cross-pillar grant data model + issuance flow |
| 4 | Wire 2-person rule (Cedar policy requires `signed_by_two_principals` claim) |
| 5 | Tests: cross-pillar read without grant refused; with grant permitted; expired grant refused |

## Verification

- `cargo nextest run -p oya-ontology-pillar-domain --test pillar_isolation` — exit 0.
- `oya gate validate cedar-coverage --microservice ontology` includes pillar.cedar — exit 0.
- 2-person rule: grant with 1 signatory refused.

## References

- Bominal ADR-0132 (pillars).
- ADR-0140 (retired per ADR-0145) (Cedar).
- `microservices/ontology/policy/type-isolation.md` §"Pillar Isolation Invariants" TI-11..TI-13.


## A. Problem
`IP-013: pillar (org / person + cross-pillar grant)` is not a generic implementation packet; it closes the `013 pillar cross pillar grant` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Typed registry evolution with monotonic data-class/pillar rules, versioned object/link/action/function schemas, and migration receipts for caller-side read libraries. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

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
- `microservices/ontology/catalog/oya-ontology-object-type-registry-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/type-register.yaml` — verify/update as the authoritative artifact for this IP.
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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `013 pillar cross pillar grant` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
