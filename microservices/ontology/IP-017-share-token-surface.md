---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0257, ADR-0244]
acceptance_status: draft
companion_docs: [microservices/ontology/policy/cross-tenant-refusal.cedar]
inbound_citations: [microservices/ontology/manifest.json]
---

# IP-017: ontology share-token surface

## A. Goal
Implement the per-tenant ontology share-token surface enabling explicit, audited, Cedar-gated cross-tenant projection per the ADR-0257 amendment.

## B. Acceptance criteria
- `POST /v1/ontology/share-tokens` issues a token with `resource_tenant_id`, `action_scope[]`, `ttl_seconds ≤ 7776000`.
- Token issuance gated by `policy/cross-tenant-refusal.cedar` requiring TENANT_ADMIN role + WebAuthn passkey.
- Token redemption emits `oya.ontology.share-token-redeem` per ADR-0263.
- Token revocation via `runbooks/share-token-revocation.md` works end-to-end.
- 100% of cross-tenant reads carry a valid share-token (no implicit bypass).

## C. Tasks
1. Crate `oya-ontology-share-token-kernel` — token type + state machine.
2. Crate `oya-ontology-share-token-adapter-postgres` — persistence.
3. REST surface in `oya-ontology-agent-gateway-rest`.
4. Cedar fragment `policy/cross-tenant-refusal.cedar` (done).
5. Runbook `runbooks/share-token-revocation.md` (done).
6. SDK update in `oya-ontology-sdk`.
7. SLO `ontology-share-token-redemption-latency`.
8. Dashboard `share-token-issuance-and-redemption.json`.

## D. Dependencies
- IP-013 (pillar-cross-pillar-grant) GA.

## E. Risks
- Token-leak; mitigated by short TTL + Cedar gate + audit chain + revocation runbook.
- Cross-tenant cache pollution; mitigated by per-tenant cache isolation in read-path library.

## F. References
- ADR-0257 amendment
- ADR-0244 tenant scoping
- `policy/cross-tenant-refusal.cedar`


## A. Problem
`IP-017: ontology share-token surface` is not a generic implementation packet; it closes the `017 share token surface` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Explicit grant materialization: cross-tenant data leaves the default-deny path only through share-token/projection contracts that carry grant, scope, region, and audit identifiers. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

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
- `microservices/ontology/catalog/oya-ontology-share-token-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/cross-tenant-refusal.cedar` — verify/update as the authoritative artifact for this IP.
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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `017 share token surface` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
