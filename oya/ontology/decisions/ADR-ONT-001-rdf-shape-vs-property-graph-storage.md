---
id: ADR-ONT-001
title: RDF Shape versus Property Graph Storage
status: Proposed
date: 2026-05-20
microservice: ontology
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-ontology
---

# ADR-ONT-001: RDF Shape versus Property Graph Storage

## Context

- Ontology is the typed entity substrate for Object Types, Link Types, Action Types, Functions, entity storage, action invocation, query, and agent gateway surfaces.
- The local PRD classifies ontology as hero-substrate and benchmarks Palantir Foundry, Palantir AIP, Neo4j, Stardog, TigerGraph, TypeDB, and knowledge-graph platforms.
- ADR-0257 binds object type versioning, deprecation handshake, additive evolution, and consumer acknowledgement.
- ADR-0244 requires tenant, audience, home cell, and provider credential mode on every ontology row and event.
- ADR-0243 requires Cedar before type registration, action invocation, query execution, public reads, and cross-tenant projections.
- The local manifest already names Citus-backed entity-store work and ClickHouse history mirrors in implementation packet titles.
- The local manifest names capabilities `cedar-evaluate`, `query-execute`, and `type-register`.
- Product and agent workloads need typed graph traversal, object-type registration, link traversal, action invocation, and typed query projections.
- Data governance workloads need shape validation, schema revision, deprecation state, field-level data class, and proof that breaking changes followed the ADR-0257 handshake.
- RDF and SHACL provide mature semantic-web standards for shape validation and interchange.
- Property graphs provide ergonomic traversal and property-bearing edges that product developers and query authors understand.
- Apache AGE adds graph functionality to PostgreSQL and supports Cypher-style graph queries as a Postgres extension.
- Citus horizontally scales PostgreSQL through distributed tables, reference tables, and distributed query planning.
- PostgreSQL keeps relational controls, migrations, transactional writes, row-level security, and operational expertise aligned with the rest of Oyatie.
- Typed query projections give product and agent clients an ergonomic read model, but the projection layer is not the storage model.
- A pure triple store would make RDF-native validation easy but complicate tenant-scoped relational controls and Citus sharding.
- A pure property graph database would make traversal easy but could fragment the platform's Postgres and Cedar policy posture.
- A pure relational model would be governable but less expressive for link traversal and graph-shaped queries.
- The ontology must support cross-service read paths without allowing direct storage coupling.
- ADR-0145 prefers direct service contracts and library-first read paths where amended, not ad-hoc database reads.
- The ontology must support tenant-local schema evolution and cross-tenant refusal by default.
- The ontology must support object types whose properties include scalar, vector, geo, timeseries, ciphertext, and struct fields.
- The ontology must support action receipts and audit-chain evidence for graph writes.
- The ontology must support history and analytics without putting OLAP pressure on the OLTP graph store.
- The ontology must support typed query projection for client-facing read ergonomics and SDK generation.
- The ontology must support RDF shape import/export for standards interop and external governance tools.
- The ontology must make the storage choice clear enough for future implementers to avoid adding Neo4j, Stardog, or a triple store as a shadow source.
- The ontology must keep tenant sharding and row-level controls explicit.
- The ontology must keep graph traversal p99 targets realistic and measurable.
- The ontology must document how RDF shapes, property graph rows, Citus distribution, and typed query projection relate.

## Decision

- Adopt PostgreSQL plus Citus as the tenant-distributed relational substrate for ontology persistence.
- Adopt Apache AGE as the property graph extension for graph traversal and relationship queries where graph expression is valuable.
- Use RDF/SHACL-style shapes as validation and interchange artifacts, not as the primary storage engine.
- Use generated query projections as client contract layers, not as the persistence model.
- Store Object Types, Link Types, Action Types, Function Types, schema revisions, and deprecation state in relational tables.
- Store entity instances and link instances in tenant-distributed Citus tables keyed by `tenant_id`.
- Store graph labels and edge properties through AGE where traversal performance or query clarity requires graph operations.
- Keep canonical type definitions in relational tables so migrations, ADR-0257 lifecycle, and Cedar policy coverage can inspect them deterministically.
- Generate RDF shape documents from canonical Object Type and Link Type definitions for export and external validation.
- Accept RDF shape imports only through a compiler that produces canonical ontology type definitions and a validation report.
- Reject direct RDF triple writes to production storage.
- Expose generated query projection endpoints from active Object Type schema revisions.
- Require projection handlers to call ontology query usecases and Cedar policy gates; handlers never query AGE or Citus directly.
- Use Citus distribution column `tenant_id` for entity and link instance tables.
- Use Citus reference tables for small global registries such as field type enum, data class enum, and built-in function signatures.
- Use tenant-local schema revision id on every entity row and link row.
- Use HLC timestamps and audit event ids for write ordering and replay.
- Mirror graph history and high-volume analytics to ClickHouse where OLAP query patterns exceed OLTP budgets.
- Keep vector search, full-text search, and temporal analytics as typed property adapters, not ad-hoc graph extensions.
- Require every type registration and schema revision change to emit audit-chain evidence.
- Require every generated query projection to include `schema_revision` and `deprecation_state` where the type is versioned.
- Require every write to check Cedar action `ontology::type_register`, `ontology::entity_write`, or `ontology::link_write`.
- Require every read to check Cedar action `ontology::query_execute` or a narrower generated projection action.
- Use p99 query target below 50 ms for simple type and function reads.
- Use p99 query target below 250 ms for tenant-local bounded graph traversal up to depth 3.
- Require explicit asynchronous job for graph traversal depth above 3 or estimated result set above 10,000 nodes.
- Cap single query projection response at 2 MiB unless an export workflow is authorized.
- Keep AGE schema per tenant shard where Citus placement requires locality and operational isolation.
- Treat RDF export as a compatibility projection subject to the same tenant, data-class, and deprecation gates.
- Treat query projection schema generation as deterministic build output from canonical type definitions.

## Alternatives Considered

### RDF triple store as primary storage

- Pros: RDF, SHACL, SPARQL, and semantic-web tooling are mature for ontology validation and interchange.
- Pros: standards-aligned graph semantics are attractive for external governance.
- Pros: shape constraints map naturally to RDF shape documents.
- Cons: tenant-scoped Citus sharding, Postgres RLS, and relational migration controls would be harder.
- Cons: product developers and generated SDKs would still need object-oriented projections.
- Cons: operational stack would add another database family to a foundational substrate.
- Rejected as primary storage; retained as validation and interchange projection.

### Neo4j or managed property graph as primary storage

- Pros: strong property graph ergonomics and mature query UX.
- Pros: rich graph tooling, visualization, and traversal features.
- Pros: easier to explain to teams familiar with Cypher and property graphs.
- Cons: adds a separate persistence platform outside the Postgres/Citus control plane.
- Cons: tenant sharding and Cedar policy integration would need parallel implementation.
- Cons: license, managed-service, and portability posture are weaker for Oyatie substrate.
- Rejected because Apache AGE gives property graph capabilities while keeping Postgres as the operating base.

### Pure relational tables without AGE

- Pros: simplest operations and governance.
- Pros: best fit for Citus tenant distribution and standard migrations.
- Pros: Cedar and schema-evolution checks can inspect every structure directly.
- Cons: traversal queries become verbose recursive SQL and harder for graph authors.
- Cons: graph-specific optimization and query readability suffer.
- Cons: product teams may build shadow graph projections to compensate.
- Rejected because ontology is graph-shaped enough to justify AGE for bounded traversals.

### Pure query-projection storage abstraction

- Pros: client and SDK ergonomics are excellent.
- Pros: frontend teams get typed schemas and generated queries.
- Pros: hides storage details from consumers.
- Cons: A query projection layer is an execution contract, not a durable storage model.
- Cons: without canonical storage rules, resolvers can drift into ad-hoc policy and performance behavior.
- Cons: schema lifecycle and deprecation handshake need a canonical source below the projection layer.
- Rejected as storage; adopted as deterministic projection from canonical type definitions.

### TypeDB or Datalog-style knowledge base

- Pros: strong semantic modeling and rule expression.
- Pros: attractive for inference-heavy ontology workloads.
- Pros: could reduce custom rule implementation.
- Cons: new database family and operational model.
- Cons: tenant sharding, Cedar coverage, and generated query projections still require custom integration.
- Cons: inference can obscure auditability if rule execution is not tightly controlled.
- Rejected for this batch; inference can be layered as a function/action surface later.

## Consequences

- Positive: storage remains aligned with Postgres, Citus, migrations, RLS, backups, and existing operational expertise.
- Positive: property graph traversal is available without adopting a separate graph database.
- Positive: RDF shape export satisfies standards interop while canonical records remain governable.
- Positive: Generated query projections give clients typed read ergonomics without becoming source of truth.
- Positive: tenant sharding and schema revisions stay explicit.
- Positive: audit-chain and Cedar gates can inspect canonical relational records.
- Positive: ClickHouse can absorb analytical history without overloading OLTP traversal.
- Negative: Apache AGE maturity and Citus interaction must be validated carefully.
- Negative: RDF import/export compiler becomes a critical correctness surface.
- Negative: some semantic-web-native use cases will find the storage model less natural.
- Negative: graph traversal constraints must be enforced to avoid runaway queries.
- Negative: Query projection generation adds build and compatibility complexity.
- Neutral: AGE and query projection adapters are implementation choices under a stable ontology contract.
- Neutral: RDF shape documents are compatibility artifacts and may be regenerated.
- Neutral: future inference engines can consume canonical type definitions and audit receipts.
- Neutral: storage and projection versions can evolve on separate but linked timelines.
- Follow-up: add RDF shape compiler fixtures for Object Type and Link Type examples.
- Follow-up: add AGE+Citus integration benchmark for depth-3 tenant-local traversal.
- Follow-up: add query projection schema generation reference tests.
- Follow-up: add a migration guard preventing direct triple-store writes.
- Follow-up: add a ClickHouse mirror contract for graph history.

## Implementation Notes

- Data shape `ObjectTypeDefinition`: `{tenant_id, object_type_id, schema_revision, name, fields, data_classes, deprecation_state, audit_event_id}`.
- Data shape `LinkTypeDefinition`: `{tenant_id, link_type_id, schema_revision, from_type, to_type, cardinality, edge_properties, deprecation_state}`.
- Data shape `ActionTypeDefinition`: `{tenant_id, action_type_id, schema_revision, input_shape, output_shape, autonomy_tier, cedar_action_id}`.
- Data shape `FunctionTypeDefinition`: `{tenant_id, function_type_id, schema_revision, input_shape, output_shape, purity, cost_budget}`.
- Data shape `EntityInstance`: `{tenant_id, entity_id, object_type_id, schema_revision, properties_jsonb, age_vertex_id, created_hlc, audit_event_id}`.
- Data shape `LinkInstance`: `{tenant_id, link_id, link_type_id, from_entity_id, to_entity_id, properties_jsonb, age_edge_id, audit_event_id}`.
- Data shape `RdfShapeProjection`: `{tenant_id, shape_id, source_type_id, schema_revision, shacl_doc_ref, generated_hash, generated_at}`.
- Data shape `QueryProjectionSchema`: `{tenant_id, projection_id, schema_revision_set, query_schema_hash, active_from, deprecated_after}`.
- REST endpoint `POST /v1/ontology/object-types` registers Object Types after Cedar and shape validation.
- REST endpoint `POST /v1/ontology/link-types` registers Link Types after Cedar and compatibility checks.
- REST endpoint `POST /v1/ontology/entities` writes entity instances through canonical usecases.
- REST endpoint `POST /v1/ontology/links` writes link instances and AGE edge records in one local transaction.
- REST endpoint `POST /v1/ontology/rdf-shapes/import` compiles RDF shapes into proposed type definitions.
- REST endpoint `GET /v1/ontology/rdf-shapes/export/{type_id}` exports generated RDF shape projection.
- REST endpoint `POST /v1/ontology/queries/projection` executes generated query projection with Cedar preflight.
- REST endpoint `POST /v1/ontology/queries/graph-traversal` executes bounded traversal with depth and cardinality limits.
- Async event `ontology.object_type.registered.v1` carries schema revision and deprecation state.
- Async event `ontology.link_instance.written.v1` carries link type, endpoints, and audit event id.
- Async event `ontology.rdf_shape.generated.v1` carries shape hash and source revision.
- Async event `ontology.query_projection.generated.v1` carries query schema hash and revision set.
- Cedar permit `ontology::type_register::execute` requires tenant admin or authorized schema steward.
- Cedar permit `ontology::entity_write::execute` requires type write permission and data-class purpose.
- Cedar permit `ontology::link_write::execute` requires permission on both endpoint entities and link type.
- Cedar permit `ontology::query_execute::execute` requires tenant scope, projection scope, and result data-class eligibility.
- Cedar forbid `ontology::rdf_import::activate` blocks direct production activation without compiled diff review.
- SLO target `function_read_latency`: p99 below 50 ms.
- SLO target `query_execute_depth3_latency`: p99 below 250 ms for tenant-local bounded traversal.
- SLO target `dynamic_layer_freshness`: p99 lag below 2 seconds for active object/link changes.
- SLO target `audit_chain_emission_completeness`: 100 percent for type and entity writes.
- Distribution column for entity and link tables is `tenant_id`.
- AGE graph name pattern is `ontology_<tenant_hash>_<cell_id>` or shard-local equivalent.
- ClickHouse history mirror stores append-only entity and link change facts for analytics.
- Query projection context always includes tenant, principal, projection id, schema revision set, and Cedar decision id.
- RDF import compiler emits proposed Object Type and Link Type diffs plus unsupported constraint warnings.
- Migration guard rejects any production endpoint that writes raw RDF triples directly.
- Traversal planner estimates depth, edge fanout, and response size before running graph queries.
- Response cap is 2 MiB unless an export workflow with audit evidence is approved.

## Verification

- Unit test `object_type_requires_schema_revision` validates ADR-0257 handshake input.
- Unit test `entity_instance_requires_tenant_distribution_key` validates Citus sharding.
- Unit test `query_projection_requires_cedar_decision` prevents direct projection bypass.
- Unit test `rdf_import_compiles_to_type_diff_not_direct_write` protects storage contract.
- Unit test `link_write_requires_both_endpoint_permissions` validates policy composition.
- Unit test `traversal_depth_above_three_requires_async_job` enforces query bound.
- Property test `query_schema_generation_deterministic` generates projection definitions and compares hashes.
- Property test `rdf_shape_export_import_roundtrip_supported_subset` validates interchange subset.
- Property test `deprecation_state_propagates_to_projection` validates projection metadata.
- Fuzz test `rdf_shape_parser_rejects_malformed_shapes` protects import path.
- Integration test `age_citus_depth3_traversal_p99_under_250ms` validates storage choice.
- Integration test `entity_write_creates_relational_row_and_age_vertex` validates dual persistence.
- Integration test `link_write_creates_relational_row_and_age_edge` validates edge persistence.
- Integration test `clickhouse_history_mirror_receives_graph_change` validates OLAP mirror.
- Integration test `tenant_a_projection_query_cannot_read_tenant_b` validates isolation.
- Load test `type_registry_10k_types_generation` validates query and RDF projection scale.
- Load test `entity_write_5k_per_second_per_cell` validates write path.
- Chaos test `age_extension_unavailable_fails_traversal_not_type_reads` validates degradation.
- Chaos test `clickhouse_mirror_lag_does_not_block_oltp_write` validates async mirror.
- Dashboard check `query-latency.json` shows relational and graph traversal latency separately.
- Dashboard check `read-path-library-freshness.json` shows schema revision propagation lag.
- Metric check `ontology_rdf_import_unsupported_constraint_total` is visible by tenant and shape.
- Static check no endpoint writes RDF triples directly to storage.
- Static check projection schemas are generated from canonical type definitions.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- Apache AGE overview and documentation: https://age.apache.org/overview/
- Citus documentation for distributed PostgreSQL tables and multi-tenant scaling: https://docs.citusdata.com/
- W3C RDF 1.1 Concepts and Abstract Syntax.
- W3C SHACL, Shapes Constraint Language.
- PostgreSQL row-level security and extension documentation.
- Apache AGE and PostgreSQL announcement materials.
- Cedar Policy Language authorization and schema documentation: https://docs.cedarpolicy.com/
- ADR-0145, ADR-0211, ADR-0243, ADR-0244, ADR-0245, ADR-0255, ADR-0257, and ADR-0263.
- Local ontology PRD, architecture, manifest, IP-004 entity-store RLS Citus, IP-005 link-store traversal, and IP-011 query-engine documents.
