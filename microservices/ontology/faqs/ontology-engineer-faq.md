---
doc_class: FAQ
microservice: ontology
persona: ontology-engineer + data-modeler + knowledge-engineer
date: 2026-05-20
doc_status: published
---

# Ontology Engineer FAQ — ontology

## Why "Ontology" and not "Object Graph" or "Knowledge Graph"?

Per `feedback_glossary_ontology_not_object_graph` doctrine + ADR-0145. We renamed "Object Graph" → "Ontology" to align with Palantir Foundry's terminology (where the substrate is most mature in industry). The semantic distinction:

- **Object Graph**: a graph of typed objects (computer-science term).
- **Knowledge Graph**: typically RDF + OWL + SPARQL (W3C semantic-web stack).
- **Ontology**: the entity types + relationship types + their semantic meanings, modeled per-tenant. Palantir's term; what we use.

The implementation is property-graph (Apache AGE 1.5 + Citus) not RDF, but the modeling language is Ontology-shaped.

## Why Apache AGE + PostgreSQL and not Neo4j / Stardog / Neptune?

Per ADR-XXX-ontology-substrate-choice. Three drivers:

1. **PostgreSQL is already in our stack**: every µservice has PostgreSQL. Adding Apache AGE is a single extension (`CREATE EXTENSION age`); no new operational surface. Neo4j + Stardog + Neptune are new operational systems.
2. **Citus sharding**: PostgreSQL with Citus shards horizontally. Neo4j's clustering is read-replica-only; write-shard requires Neo4j Fabric (paid, complex). Stardog + Neptune have their own sharding models.
3. **PostgreSQL transactional guarantees**: ontology writes need ACID consistency with the rest of the tenant's data (which lives in other PG-backed µservices). Cross-µservice transactions via FDW are PG-native.

The trade-offs:

- AGE's Cypher dialect is a subset of Neo4j Cypher; not 100 % compatible. Documented + flagged in CI.
- Neo4j Vector Index (for semantic search) is more mature than AGE+pgvector. We route semantic search through `intelligence` µservice's Qdrant.

## How is sharding configured?

Per ADR-XXX-ontology-sharding. Citus shards entity tables by tenant_id by default. Each tenant gets a logical "distribution column" = tenant_id; all entities for a tenant land on the same shard set.

For very-large tenants (≥ 100M entities) we further shard by `entity_id_partition_key` (a hash of the primary key). The 4-shard default works for most tenants; we increase to 16 or 32 shards for the largest.

Cross-shard graph traversals are slower than within-shard. The query planner optimizes for within-shard traversal where possible; cross-shard adds ~ 100 ms p99 per shard touched.

## What's the relationship between this µservice and `intelligence`?

Per ADR-0220. The `intelligence` µservice's RAG pipeline can ingest from `ontology`:

1. `intelligence.knowledge_base_create` with `source=ontology` mode.
2. The pipeline reads tenant entities + properties.
3. Embeds each entity's properties via BGE-M3.
4. Indexes in Qdrant.
5. At query time, semantic search retrieves relevant entities + their properties.

This bridges structured (ontology) + unstructured (RAG) knowledge. A tenant query like "show me products similar to Widget Mark 1" uses ontology + RAG together.

## How do schema migrations work? Are they reversible?

Per ADR-XXX-ontology-schema-evolution. Schema migrations are versioned + auditable:

1. Tenant authors the new schema YAML.
2. `oya ontology migrate apply --dry-run` prints the migration plan.
3. `oya ontology migrate apply` executes (PostgreSQL DDL + Citus shard-aware).
4. The migration emits `ontology.migration.applied` to audit-chain.
5. The previous schema version is preserved (via PG column comments + audit-chain history).

Reversibility:

- **Add property**: reversible (drop column, data preserved in audit-chain).
- **Add entity type**: reversible (drop table, data lost unless restored).
- **Add relationship type**: reversible (drop relationship index).
- **Rename property**: reversible via a counter-rename migration.
- **Drop property**: irreversible at the storage layer; data preserved only in audit-chain emit history.
- **Drop entity type**: irreversible; recommend grace-period deprecation.

Best practice: deprecate first (mark with `deprecated_at`), wait ≥ 30 d for tenant code to migrate, then drop in a separate migration.

## How does time-travel querying work?

Per ADR-XXX-ontology-time-travel. Every entity write + relationship write retains the previous version in an append-only `entity_history` table. At query time, tenants specify the `as_of_timestamp`:

```sh
oya ontology query \
    --tenant acme-corp \
    --query 'MATCH (c:Customer {customer_id:"cust-001"}) RETURN c' \
    --as-of 2026-04-01T00:00:00Z
# Returns Customer cust-001 as it looked on 2026-04-01.
```

The query planner joins the current entity tables with the history table; the join cost is ~ 1.5-2× a current-state query.

Retention: 365 d for paid tenant_class. Beyond that, archive to cold storage; restore requires the recovery path.

## What's the entity property classification scheme?

Per ADR-XXX-ontology-pii-tagging. Each property carries a `pii_class` attribute:

- **public**: anyone with `entity::read` can see.
- **internal**: requires `entity::read::internal` permission.
- **restricted**: requires `entity::read::restricted` permission + audit-emit on every access.
- **secret**: encrypted at-rest with tenant KMS key; requires `entity::read::secret` permission + dual-control approval for some classes.

The classification informs:

- Cedar policy generation.
- Encryption-at-rest configuration.
- Audit-chain emission verbosity.
- Compliance attestation (GDPR Art. 30, HIPAA §164.514, KR PIPA Art. 22).

## How are cross-µservice queries handled?

Per ADR-0145. Cross-µservice queries use direct gRPC calls between µservices (per the inter-µservice communication reform). The `ontology` µservice exposes its query surface via gRPC; downstream µservices call it.

For complex queries that join ontology data + other µservice data (e.g., "show me customers with overdue invoices" = ontology Customer JOIN payments Invoice), the joining µservice (in this case, a tenant application or the `workflow-engine`) issues two queries + joins client-side.

We do NOT support cross-µservice federated SQL (e.g., PG foreign data wrappers across cells). The complexity vs benefit ratio doesn't favor it.

## What's the per-tenant query cost budget?

Per ADR-XXX-ontology-query-cost. Each tenant has a daily query-cost budget enforced via PostgreSQL's `pg_stat_statements`. Costly queries (high `shared_blks_read`, deep traversals, full table scans) consume more budget.

Default budget by tenant_class:

- demo_trial: 100 k "cost units" per day.
- paid: contract-specific budget bound by active billing_components.

Tenants exceeding budget get rate-limited; alerts fire. Most tenants stay well within budget.

## How do I respond when a tenant says "Palantir Foundry has X feature"?

Common Palantir features + our equivalents:

- **Pipeline Builder**: handled by `workflow-engine` + `workflow-studio` (not `ontology` directly).
- **Ontology Object Type Manager**: `oya ontology schema register/migrate`.
- **Object Set Builder**: query via Cypher-like graph patterns; results = an object set.
- **Action Templates**: handled by `workflow-engine` (a workflow definition that operates on an object set).
- **Workshop**: handled by tenant-application UI; we provide the ontology query primitives.
- **Foundry Notebook**: handled by tenant data-engineer's preferred Jupyter / VS Code / Cursor; we provide gRPC + GraphQL endpoints.

Palantir's full surface is broader; oyatie focuses on the substrate primitives + lets tenants build the application surface.

## What happens when a tenant deletes a Customer that has Orders?

Cascade-delete is configurable per-relationship-type:

```yaml
relationship_types:
  - id: PLACED_ORDER
    from_entity: Customer
    to_entity:   Order
    cardinality: one-to-many
    on_from_delete: cascade  # delete orders when customer deleted
    # alternatives: restrict (fail), set_null (orphan)
```

Cascade-delete emits one event per cascaded entity. For large cascades (a customer with 10k orders), the operation is batched + the audit-chain emits ~ 10k events. Cost-budget aware.

`set_null` is rare in ontology because relationship edges can't be null (an Order with no Customer is conceptually orphaned). We recommend `restrict` for most tenant ontologies — force the tenant to explicitly archive Orders before deleting the Customer.

## Why doesn't ontology support RDF / OWL / SPARQL?

Per ADR-XXX-ontology-property-graph-not-rdf. RDF + OWL + SPARQL are powerful but:

- The reasoning capabilities are rarely used in business ontologies.
- SPARQL query optimization is harder than property-graph + Cypher.
- The tooling ecosystem is smaller.
- Tenants are more familiar with property-graph (Neo4j-style).

We focus on the 95 % use case (business entity modeling). For tenants who need full RDF + reasoning (rare), we recommend Stardog or AnzoGraph separately.

## What's the difference between this µservice and `cell-domain`?

- `cell-domain`: the substrate's own per-cell metadata + cell registry. Internal to oyatie.
- `ontology`: tenant-facing business entity modeling. Multi-tenant; per-tenant schema.

The two never share schemas; both emit to audit-chain.
