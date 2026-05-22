---
doc_class: Onboarding
microservice: ontology
persona: ontology-engineer + data-modeler + knowledge-engineer
related_adrs: [ADR-0263, ADR-0329, ADR-0330, ADR-0331, ADR-0145]
date: 2026-05-20
doc_status: published
---

# Ontology Engineer onboarding — first 5 working days on `ontology`

Audience: a new ontology engineer, data modeler, or knowledge-engineer joining the `ontology` rotation. By Day-5 they will have: bootstrapped a demo_trial tenant_class cell, authored a tenant ontology schema, written entities + relationships, executed graph-pattern queries, exercised a schema migration, and walked the entity-graph-shard-rebalance runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the Palantir-Foundry-Ontology parallel + the per-tenant entity sovereignty doctrine.
2. Read `ARCHITECTURE.md` § entity-storage + § graph-traversal + § schema-validation + § sharding (∼ 60 min).
3. Open the Grafana folder `ontology`. primary boards: `ontology-read-latency`, `ontology-write-latency`, `ontology-graph-traversal-depth`, `ontology-shard-balance`, `ontology-query-cost-budget`, `ontology-schema-version`, `ontology-cedar-deny-rate`.
4. Walk `runbooks/README.md`. The on-call runbooks: `shard-rebalance-stuck.md`, `schema-migration-failed.md`, `graph-query-timeout.md`, `entity-cascade-delete-spike.md`, `cedar-deny-storm.md`, `time-travel-query-stale.md`, `graphql-resolver-recursion.md`, `tenant-quota-exceeded.md`.
5. Sit in on the Wednesday ontology-substrate handoff. Watch the outgoing rotation read the past-week shard-balance + query-cost panels.

Acceptance: you can sketch the query path: tenant API → Cedar gate → query planner → Citus shard selection → entity + relationship table scans → graph-traversal join → audit-chain emit (on writes) → result. Plus the schema-migration path: schema YAML → validator → migration plan → Citus DDL apply → backfill if needed → version bump.

## Day 2 — demo_trial tenant_class ontology cell bootstrap + first schema

```sh
cargo run -p oya-dev-cli -- ontology bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/ontology \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 10 min. Verify:

```sh
oya ontology health --cell drill-syd-1
# Expected:
#   postgres.entities: up (lag_ms=12)
#   valkey.query-cache: up (3 nodes, 0 hit-rate)
#   schema-registry: up (0 tenants registered)
#   cedar: policies-loaded
#   audit-chain.emit: up
```

Author your first tenant ontology schema. Save as `acme-ontology-v1.yaml`:

```yaml
tenant_id: drill-acme
schema_version: 1

entity_types:
  - id: Customer
    properties:
      customer_id:        { type: string, required: true, primary: true }
      email:              { type: string, format: email, required: true, indexed: true }
      full_name:          { type: string, required: true }
      created_at:         { type: datetime, required: true }
      lifetime_value_usd: { type: decimal, scale: 2, default: "0.00" }
      pii_class:          { type: enum, values: [public, internal, restricted, secret], default: restricted }

  - id: Order
    properties:
      order_id:    { type: string, required: true, primary: true }
      status:      { type: enum, values: [pending, paid, fulfilled, refunded, cancelled], required: true }
      total_minor_units: { type: integer, required: true }
      currency:    { type: string, default: USD }
      created_at:  { type: datetime, required: true }
      completed_at: { type: datetime, required: false }

  - id: Product
    properties:
      sku:         { type: string, required: true, primary: true }
      name:        { type: string, required: true }
      category:    { type: string, required: false, indexed: true }
      price_minor_units: { type: integer, required: true }
      currency:    { type: string, default: USD }

relationship_types:
  - id: PLACED_ORDER
    from_entity: Customer
    to_entity:   Order
    cardinality: one-to-many
    properties:
      placed_at:   { type: datetime, required: true }

  - id: CONTAINS_PRODUCT
    from_entity: Order
    to_entity:   Product
    cardinality: many-to-many
    properties:
      quantity:    { type: integer, required: true, minimum: 1 }
      unit_price_minor_units: { type: integer, required: true }

indexes:
  - entity: Customer
    on: [email]
  - entity: Order
    on: [created_at, status]
  - entity: Product
    on: [category]
```

Register the schema:

```sh
oya ontology schema register \
    --tenant drill-acme \
    --schema-file ./acme-ontology-v1.yaml
# Output: schema registered. tenant=drill-acme version=1
```

Verify the registration emitted to audit-chain:

```sh
oya audit query --tenant drill-acme --event-class ontology.schema.registered --since 5m
# Expected: 1 event with the schema hash + version=1.
```

Acceptance: schema registered; you can sketch the entity-table layout (`tenant_drill_acme.entity_customer`, `tenant_drill_acme.entity_order`, `tenant_drill_acme.entity_product`, plus a relationship-edge table `tenant_drill_acme.relationship_edge`).

## Day 3 — Write entities + relationships + execute queries

Write entities:

```sh
oya ontology entity create \
    --tenant drill-acme \
    --type Customer \
    --properties '{
        "customer_id":"cust-001",
        "email":"alice@acme.test",
        "full_name":"Alice Example",
        "created_at":"2026-05-20T14:00:00Z",
        "lifetime_value_usd":"12500.00",
        "pii_class":"restricted"
    }'

oya ontology entity create \
    --tenant drill-acme \
    --type Product \
    --properties '{"sku":"WIDGET-1","name":"Widget Mark 1","category":"Hardware","price_minor_units":2500}'

oya ontology entity create \
    --tenant drill-acme \
    --type Product \
    --properties '{"sku":"GADGET-1","name":"Gadget Mark 1","category":"Hardware","price_minor_units":3500}'

oya ontology entity create \
    --tenant drill-acme \
    --type Order \
    --properties '{"order_id":"ord-001","status":"paid","total_minor_units":6000,"currency":"USD","created_at":"2026-05-20T14:30:00Z"}'
```

Write relationships:

```sh
oya ontology relationship create \
    --tenant drill-acme \
    --type PLACED_ORDER \
    --from-entity-id cust-001 \
    --to-entity-id ord-001 \
    --properties '{"placed_at":"2026-05-20T14:30:00Z"}'

oya ontology relationship create \
    --tenant drill-acme \
    --type CONTAINS_PRODUCT \
    --from-entity-id ord-001 \
    --to-entity-id WIDGET-1 \
    --properties '{"quantity":1,"unit_price_minor_units":2500}'

oya ontology relationship create \
    --tenant drill-acme \
    --type CONTAINS_PRODUCT \
    --from-entity-id ord-001 \
    --to-entity-id GADGET-1 \
    --properties '{"quantity":1,"unit_price_minor_units":3500}'
```

Execute queries:

```sh
# Single entity read
oya ontology entity get --tenant drill-acme --type Customer --id cust-001

# 1-hop traversal: get all orders for a customer
oya ontology query \
    --tenant drill-acme \
    --query 'MATCH (c:Customer {customer_id:"cust-001"})-[:PLACED_ORDER]->(o:Order) RETURN o'

# 2-hop traversal: get all products in all orders for a customer
oya ontology query \
    --tenant drill-acme \
    --query 'MATCH (c:Customer {customer_id:"cust-001"})-[:PLACED_ORDER]->(o:Order)-[:CONTAINS_PRODUCT]->(p:Product) RETURN c.full_name, o.order_id, p.name, p.price_minor_units'
```

Expected output (last query):

| c.full_name | o.order_id | p.name | p.price_minor_units |
|---|---|---|---:|
| Alice Example | ord-001 | Widget Mark 1 | 2500 |
| Alice Example | ord-001 | Gadget Mark 1 | 3500 |

Audit-chain verification:

```sh
oya audit query --tenant drill-acme --event-class "ontology.*" --since 10m
# Expected: 4 entity.created + 3 relationship.created + 3 ontology.query.executed = 10 events
```

Acceptance: entities + relationships written; 2-hop graph query works; audit-chain has the events.

## Day 4 — Schema migration (backward-incompatible change)

Tenant decides to add a `customer_segment` enum to Customer + rename `total_minor_units` to `total_amount_minor_units` on Order.

Author the migration:

```yaml
# acme-ontology-v2.yaml
tenant_id: drill-acme
schema_version: 2
migrates_from: 1

entity_types:
  - id: Customer
    properties:
      customer_id:        { type: string, required: true, primary: true }
      email:              { type: string, format: email, required: true, indexed: true }
      full_name:          { type: string, required: true }
      created_at:         { type: datetime, required: true }
      lifetime_value_usd: { type: decimal, scale: 2, default: "0.00" }
      pii_class:          { type: enum, values: [public, internal, restricted, secret], default: restricted }
      customer_segment:   { type: enum, values: [smb, mid_market, enterprise], default: smb, required: false }

  - id: Order
    properties:
      order_id:                  { type: string, required: true, primary: true }
      status:                    { type: enum, values: [pending, paid, fulfilled, refunded, cancelled], required: true }
      total_amount_minor_units:  { type: integer, required: true }
      currency:                  { type: string, default: USD }
      created_at:                { type: datetime, required: true }
      completed_at:              { type: datetime, required: false }

migrations:
  - entity_type: Order
    property: total_minor_units
    rename_to: total_amount_minor_units
```

Apply the migration:

```sh
oya ontology migrate apply \
    --tenant drill-acme \
    --schema-file ./acme-ontology-v2.yaml \
    --dry-run
# Output: migration plan
#   Add property: Customer.customer_segment (default=smb)
#   Rename property: Order.total_minor_units → Order.total_amount_minor_units
#   Estimated affected rows: 1 customer + 1 order
#   Estimated wall-clock: < 1 s
```

Apply for real:

```sh
oya ontology migrate apply \
    --tenant drill-acme \
    --schema-file ./acme-ontology-v2.yaml
```

Verify the migration:

```sh
oya ontology entity get --tenant drill-acme --type Customer --id cust-001
# Expected: customer_segment="smb" (default), all other properties preserved.

oya ontology entity get --tenant drill-acme --type Order --id ord-001
# Expected: total_amount_minor_units=6000 (renamed), total_minor_units no longer exists.
```

Audit-chain:

```sh
oya audit query --tenant drill-acme --event-class ontology.migration.applied --since 5m
# Expected: 1 event with the migration plan + the schema version bump.
```

Acceptance: schema migration applied; backward-incompatible rename succeeded; audit trail intact.

## Day 5 — Shard rebalance drill + cedar-deny-storm runbook

At demo_trial tenant_class scale the data fits on one PG node; sharding is a paid tenant_class concern. Run the rebalance drill in shadow:

```sh
oya ontology drill shard-rebalance \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --shadow-mode
# Expected: shadow operation runs without changing actual shards;
# verify the migration plan would correctly rebalance entities across shards.
```

Walk the cedar-deny-storm runbook. Read `runbooks/cedar-deny-storm.md`. Scenario: a Cedar policy update accidentally denies all reads from a tenant. The runbook covers:

1. Identify the storm from `ontology-cedar-deny-rate` panel.
2. Confirm via audit-chain query (`oya audit query --event-class ontology.cedar.denied --since 5m | wc -l` — expect to see hundreds of denials).
3. Identify the offending policy: `oya audit query --event-class governance.cedar.policy.updated --since 1h`.
4. Roll back the policy via `oya governance cedar-policy rollback --policy-id ... --to-version N-1`.
5. Verify deny-rate drops back to baseline.

Target end-to-end recovery: ≤ 10 min for the drill (production target ≤ 15 min per `slos/cedar-deny-storm-recovery.openslo.yaml`).

Acceptance: drill executed; runbook walked.

## What you've learned

- demo_trial tenant_class bootstrap + schema authoring + entity/relationship writes.
- Graph-pattern queries (Cypher-like via Apache AGE).
- Schema migration with backward-incompatible changes.
- Shard rebalance drill.
- Cedar-deny-storm runbook.

Next week: paid tenant_class promotion (Citus sharding + materialised views), paid tenant_class tour (GraphQL endpoint + time-travel queries + embedding-search integration), paid tenant_class sovereign-pack tour (sovereign-pack entity types + regulator-attestation), and your first production shadow.
