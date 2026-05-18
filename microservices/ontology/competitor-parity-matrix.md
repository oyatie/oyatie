---
doc_class: CompetitorParityMatrix
title: Competitor Parity Matrix
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-ontology + council-architecture + gtm-product-marketing
deciders: axis-ontology, council-architecture
related_artifacts:
  - microservices/ontology/PRD.md §"Competitive Benchmark"
  - evidence/autoresearch/ontology-competitive-map.json
review_cadence: quarterly + on any major competitor product release
doc_status: published
---

# Competitor Parity Matrix (ontology µservice)

## Purpose

Map the Ontology µservice's feature parity against the seven primary competitors:
**Palantir Foundry Ontology**, **Palantir AIP**, **AWS Cedar**, **Open Policy Agent (OPA)**, **Apache TinkerPop**, **Neo4j**, **AWS Neptune**, **Stardog**.

Per `feedback_quality_performance_scalability_bar.md`, this µservice is held to "hyperscaler-grade in every practice". The matrix below identifies parity wins, parity gaps, and source-evidence citations.

## Parity Matrix

### Core Typed-Entity Layer (Palantir Foundry Ontology baseline)

| Feature | Palantir Foundry | oyatie Ontology | Status |
|---|---|---|---|
| Typed Object Types with property descriptors | ✅ | ✅ (per BC `object-type-registry`) | parity |
| Typed Link Types with cardinality (1:1 / 1:N / M:N) | ✅ | ✅ (per BC `link-type-registry`) | parity |
| Typed Action Types with effects + idempotency | ✅ | ✅ (per BC `action-type-registry` + `action-engine`) | parity |
| Typed Functions for read projections | ✅ | ✅ (per BC `function-type-registry` + `function-engine`) | parity |
| Property tier classification (sensitivity) | ✅ | ✅ (Tier1..Tier4 per Bominal ADR-0008) | parity |
| Object/Link RLS (row-level tenant isolation) | ✅ (Foundry "Permissions" layer) | ✅ (Postgres FORCE ROW LEVEL SECURITY) | parity |
| Schema-resource vs object/link-data permission split | ✅ | ✅ (cedar tenant-scope.cedar vs ci-scope.cedar split) | parity |
| Private / shared / public ontology spaces | ✅ | ✅ (tenant:oya-self, tenant:oya-aggregate reserved; per-tenant scopes) | parity |
| Action transaction receipt (provenance per mutation) | ✅ | ✅ (ActionInvocationReceipt with object_ids, link_ids, audit_chain_ref) | parity |
| Per-property index strategy (btree / hash / GIN / R-tree) | ✅ | ✅ (per `PropertyDescriptor.index_strategy`) | parity |
| Vector property type (embeddings) | ✅ (via pgvector or third-party) | ✅ (Bominal ADR-0108 inherited) | parity |
| Geo property type | ✅ (PostGIS or third-party) | ✅ (Bominal ADR-0109 inherited; PostGIS) | parity |
| Timeseries property type | ✅ | ✅ (Bominal ADR-0110 inherited; TimescaleDB extension or in-house) | parity |
| Ciphertext property type (envelope encryption) | ✅ | ✅ (Bominal ADR-0111 inherited; KMS-wrapped DEK per tenant per property) | parity |
| Struct property type (FHIR / EDI / nested schemas) | ✅ | ✅ (Bominal ADR-0112 inherited; jsonb + path indexes) | parity |
| Schema evolution (versioned + back-compat) | ✅ | ✅ (per `runbooks/type-registry-migration.md`) | parity |
| Audit chain (Merkle / cryptographic provenance) | ✅ | ✅ (Bominal ADR-0028 inherited; Ed25519) | parity |
| Jurisdiction overlay | partial (Palantir has multi-region; not explicit overlay) | ✅ (per `JurisdictionOverlay` + Cedar overlays) | **oyatie ahead** |
| Per-pack regional residency by default | partial | ✅ (cross-pack forbidden by default per `data-residency.md`) | **oyatie ahead** |
| Plugin SDK (custom renderers) | ✅ | planned M04 (WASM via Wasmtime per Bominal ADR-0037 inheritance) | **gap** |
| Multi-tenant SaaS-grade (10k+ tenants per cell) | ✅ | ✅ (Citus + ClickHouse) | parity |

### Agent Gateway (Palantir AIP baseline)

| Feature | Palantir AIP | oyatie Ontology | Status |
|---|---|---|---|
| LLM tool-call ingress | ✅ | ✅ (Bominal ADR-0107 inherited; per BC `agent-gateway`) | parity |
| Auto-generate OpenAI tool-spec from Function Type | ✅ | ✅ (per `agent-gateway-rest` `/agent/tool-specs`) | parity |
| Autonomy tier ceiling on tool calls | ✅ | ✅ (Cedar `agent-gateway-scope.cedar`; `AutonomyTier` enum) | parity |
| Per-LLM-session rate limit | ✅ | ✅ (per agent-gateway worker) | parity |
| Cedar policy gate on every tool-call | ✅ (Foundry permissions) | ✅ (explicit Cedar) | parity |
| Tier-filtered tool-call payload | partial (Foundry permissions) | ✅ (Function projection tier-filter) | **oyatie ahead** |
| Tenant opt-in for high-risk LLM Functions | n/a (Foundry-internal) | ✅ (per DPIA R-14) | **oyatie ahead** |
| Audit-chain emit per tool-call | ✅ | ✅ | parity |
| EU AI Act compliance (Arts. 9-15) | partial | ✅ (planned per `dpia.md` §"pack-eu") | **oyatie ahead** |

### Policy Engine (AWS Cedar + OPA baseline)

| Feature | Cedar v4 | OPA / Rego | oyatie Ontology | Status |
|---|---|---|---|---|
| Policy fragments | ✅ | ✅ | ✅ (per `policy/*.cedar`) | parity |
| Default-deny baseline | ✅ | partial | ✅ (every fragment starts with `forbid`) | parity (Cedar-aligned) |
| Per-Action permit | ✅ | ✅ | ✅ (cedar-coverage lane enforces) | parity |
| Engine timeout (bounded eval) | ✅ (no recursion) | partial (decision logs slow) | ✅ (10ms hard cap) | parity |
| Hot-reload | ✅ | ✅ (bundle) | ✅ (per `cedar-fragment-coverage` worker) | parity |
| Schema-typed entity references | ✅ | partial | ✅ | parity |
| Fuzz-testing in CI | n/a | n/a | ✅ (`oya-check-cedar-fragment-coverage`) | **oyatie ahead** |

### Graph Engine (Neo4j + AWS Neptune + Apache TinkerPop baseline)

| Feature | Neo4j | AWS Neptune | TinkerPop | oyatie Ontology | Status |
|---|---|---|---|---|---|
| Property graph traversal | ✅ (Cypher) | ✅ (Gremlin / openCypher / SPARQL) | ✅ (Gremlin) | ✅ (Link Type traversal via `traverseLink` API) | parity (different API surface) |
| Multi-language query (Gremlin / Cypher / SPARQL) | partial | ✅ | ✅ | planned (M04; via virtual graph adapters) | **gap** |
| Schema-required (vs schema-optional) | partial (schema-optional default) | partial | partial | ✅ (schema-required by design) | **oyatie ahead** |
| Per-tenant isolation | partial (database-per-tenant; expensive) | partial | n/a | ✅ (Postgres RLS + Citus tenant_id shard) | **oyatie ahead** |
| Vector index for GraphRAG | ✅ (since 5.13) | ✅ (since 2024) | n/a | ✅ (Bominal ADR-0108 inherited) | parity |
| Graph analytics (centrality, community detection, PageRank) | partial | ✅ (Neptune Analytics; openCypher procedures) | partial | planned (M04 via virtual graph adapter pattern) | **gap** |
| Explain plan / index hint receipts | ✅ | ✅ | partial | ✅ (Function EXPLAIN pre-check) | parity |

### Virtual Graph + Reasoning (Stardog baseline)

| Feature | Stardog | oyatie Ontology | Status |
|---|---|---|---|
| Virtual graph data virtualization (RDF mapping to remote sources) | ✅ | planned (M04 via virtual-source mapping receipts per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)`) | **gap** |
| Query rewriting to native source syntax | ✅ | partial (Function DSL → Postgres/ClickHouse) | parity (different surface) |
| OWL/RDFS reasoning | ✅ | n/a (oyatie chose typed-entity layer, not OWL) | **intentional difference** |
| SPARQL endpoint | ✅ | planned (M04) | **gap** |
| Pass-through security on adapters | ✅ | ✅ (Cedar policy on every adapter; data_class propagation) | parity |
| Mapping receipt with credential_mode + named_scope_policy | ✅ | planned (M04; ADR successor-IP) | **gap** |

## Source Evidence Citations

Every observed strength + parity claim is backed by an official source. Per `/specs/microservices/ontology.json#sources_scanned`:

| Competitor | Source URL | Source-evidence ref in registry |
|---|---|---|
| Palantir Foundry Ontology | `palantir.com/docs/foundry/ontology` | `palantir_ontology_resources_537_550`, `palantir_operational_layer_538_557`, `palantir_action_transaction_540`, `palantir_permission_layers_537_549` |
| Palantir AIP | `palantir.com/platforms/aip` | (Palantir AIP product page) |
| AWS Cedar | `cedarpolicy.com`, `docs.aws.amazon.com/verifiedpermissions/latest/userguide/policy-cedar.html` | (Cedar reference docs) |
| Open Policy Agent | `openpolicyagent.org` | (OPA Rego docs) |
| Neo4j | `neo4j.com/docs` | `neo4j_property_graph_212_229`, `neo4j_schema_indexes_constraints_430_441`, `neo4j_vector_indexes_353_384`, `neo4j_vector_limitations_780_790` |
| Apache TinkerPop | `tinkerpop.apache.org` | (TinkerPop reference) |
| AWS Neptune | `aws.amazon.com/neptune` | `neptune_query_languages_5_19`, `neptune_analytics_algorithms_5_111`, `neptune_vector_index_non_atomic` |
| Stardog | `docs.stardog.com` | `stardog_virtual_graphs_422_440`, `stardog_supported_sources`, `stardog_reasoning_420_426`, `stardog_virtual_security` |

LEAN lane `oya-foundry-fitness-source-backed-competitive-claims` validates that every claim in this matrix has a `source_evidence_refs` entry; un-cited claims fail the lane.

## Parity-Gap Closure Plan

### M02b launch parity wins (in-scope this phase)

- Core typed-entity layer: ✅ Palantir Foundry parity.
- Agent gateway: ✅ Palantir AIP parity.
- Cedar policy engine: ✅ AWS Cedar parity.
- Per-tenant isolation: ✅ exceeds Neo4j / Neptune (RLS + Citus).
- Jurisdiction overlay: ✅ exceeds Palantir.
- Pack residency: ✅ exceeds all competitors (cross-pack forbidden by default).
- EU AI Act compliance: ✅ planned ahead of competitor product roadmaps.

### M03 closure targets

- TypeScript + Python SDK ship (closes Tier-B SDK gap).
- Stardog-style virtual graph adapter pattern (mapping receipts; named_scope_policy).
- Graph analytics OLAP via ClickHouse (centrality, PageRank-equivalent over Object Types).

### M04-onward stretch targets

- Plugin SDK distribution (WASM via Wasmtime per Bominal ADR-0037).
- Multi-language graph query (Gremlin / Cypher / SPARQL as projection adapters).
- OWL/RDFS reasoning (Stardog-style; if customer demand).
- Apache TinkerPop Gremlin facade for Object/Link traversal.

## Anti-Pattern Catalog (refused, with detection)

Per `/specs/microservices/ontology.json#anti_patterns`:

| Anti-pattern | Why forbidden | Detection lane |
|---|---|---|
| Cross-tenant raw SQL bypass RLS | Tenant-isolation violation | `oya-foundry-fitness-no-raw-sql-cross-tenant` |
| Untyped entity creation | Defeats type-system value | Schema validation refuses |
| Tier-violating query | Data-class violation | `oya-foundry-fitness-ontology-tier-enforcement` |
| LLM tool-call without Cedar check | Privilege escalation | `oya-foundry-fitness-cedar-coverage` |
| Direct cross-cell DB access | Bypasses Ontology adapter; breaks federation | `cargo-deny` on cross-µservice imports |
| Silent stale dynamic-layer read | Hidden inconsistency | AC-12 freshness budget enforced |
| External graph DB dep | Cross-tenant isolation harder; we have RLS | `cargo-deny` denylist |
| Action without audit-chain emission | Provenance gap | `oya-foundry-fitness-audit-chain-emission` |
| Unsourced competitor benchmark claim | Source-evidence policy violation | `oya-foundry-fitness-source-backed-competitive-claims` |
| External graph engine as canonical authority | Neo4j/Neptune/Stardog inform adapters only | `cargo-deny` denylist + adapter boundary validator |
| Vector/reasoning output as policy truth | Approximate retrieval cannot authorize canonical state | Policy/action gate requires canonical entity_id + permission proof + action receipt |
| Action without transaction receipt | Mutations need deterministic provenance before state change | Ontology action receipt gate |
| Virtual graph without pass-through / named-scope security | External-source reads must preserve user/source permissions + data-class scope | Virtual_source_mapping_receipt validator (M04) |

## Verification

- `oya gate validate authority-cohesion` — exit 0; HG-ONT (hyperscaler maturity claim gate) green.
- `oya gate validate source-backed-competitive-claims --microservice ontology` — exit 0.
- Quarterly: competitor product release notes review; matrix updated; new gaps identified.

## References

- `microservices/ontology/PRD.md` §"Competitive Benchmark".
- `/specs/microservices/ontology.json` §"competitive" + "best_practices" + "anti_patterns".
- `evidence/autoresearch/ontology-competitive-map.json`.
- Palantir Foundry Ontology docs — `palantir.com/docs/foundry/ontology`.
- Palantir AIP — `palantir.com/platforms/aip`.
- AWS Cedar — `cedarpolicy.com`.
- Open Policy Agent — `openpolicyagent.org`.
- Neo4j — `neo4j.com/docs`.
- Apache TinkerPop — `tinkerpop.apache.org`.
- AWS Neptune — `aws.amazon.com/neptune`.
- Stardog — `docs.stardog.com`.
- The Open Group Architecture Framework (TOGAF) — `pubs.opengroup.org/togaf-standard`.
- Bominal ADR-0028 + ADR-0106 + ADR-0107 + ADR-0108–0112 + ADR-0132 (inherited).
