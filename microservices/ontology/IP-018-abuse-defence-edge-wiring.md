---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297, ADR-0253]
acceptance_status: draft
companion_docs: [microservices/ontology/policy/abuse-defence.cedar]
inbound_citations: [microservices/ontology/manifest.json]
---

# IP-018: ontology abuse-defence edge wiring

## A. Goal
Wire the ADR-0297 abuse-defence baseline on the ontology agent-gateway + REST surface with Cloudflare Bot Management at the edge and Cedar evaluation at the µservice boundary.

## B. Acceptance criteria
- `iac/<env>-edge-waf.yaml` declares per-route rate-limits + bot-mgmt sensitivity per tenant-tier.
- `policy/abuse-defence.cedar` deployed across all cells with 60s soak.
- ECH config-id served from `iac/<env>-ech-config.yaml`; rotated 90d.
- PQC hybrid `X25519MLKEM768` advertised on ClientHello / ServerHello (per row-12 ADR-0253).
- Default-path latency budget: ≤2ms added p99.
- UX-floor CI lane `oya-governance-abuse-defence-ux-floor` green.

## C. Tasks
1. IaC `iac/<env>-edge-waf.yaml` (done).
2. IaC `iac/<env>-ech-config.yaml` (done).
3. IaC `iac/<env>-pqc-cert.yaml` (done).
4. Cedar fragment `policy/abuse-defence.cedar` (done).
5. Dashboard `dashboards/abuse-defence-outcomes.json` (done).
6. Runbook `runbooks/ontology-bot-score-recalibration.md` (done).
7. Audit-event registration per ADR-0263 — `oya.ontology.abuse-defence-block`, `oya.ontology.abuse-defence-recalibrate-complete`.
8. Cross-µservice consistency check per `oya-governance-cross-consistency`.

## D. Dependencies
- ADR-0297 published.
- Cloudflare Bot Management account provisioned for ontology routes.

## E. Risks
- False-positive blocks on legitimate substrate calls; mitigated by SPIFFE workload identity + `audience_type=INTERNAL_SUBSTRATE` bypass.
- UX-floor regression; mitigated by CI lane + per-tier sensitivity tuning.

## F. References
- ADR-0297
- ADR-0253
- `policy/abuse-defence.cedar`


## A. Problem
`IP-018: ontology abuse-defence edge wiring` is not a generic implementation packet; it closes the `018 abuse defence edge wiring` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `018 abuse defence edge wiring` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
