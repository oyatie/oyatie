---
doc_class: Tutorial
microservice: ontology
persona: ontology-engineer + data-modeler
date: 2026-05-20
doc_status: published
---

# Tutorial — Model a 5-entity tenant ontology + query it via Cypher and GraphQL

You will: model an e-commerce tenant ontology with 5 entity types + 6 relationship types, populate with synthetic data, execute graph-pattern queries via Cypher + GraphQL + REST, run a time-travel query, and verify the audit-chain emission. Total time ≤ 70 minutes.

## Pre-requisites

- A tenant cell with `tenant_class=paid` for GraphQL + time-travel support.
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `ontology_admin` Cedar role.

## Step 1 — Author the schema (≤ 15 min)

Save as `ecommerce-ontology-v1.yaml`:

```yaml
tenant_id: acme-corp
schema_version: 1
ontology_description: "E-commerce business entities for ACME Corp"

entity_types:
  - id: Customer
    description: "End-customer (buyer) of ACME products"
    properties:
      customer_id:        { type: string, required: true, primary: true }
      email:              { type: string, format: email, required: true, indexed: true, pii_class: restricted }
      full_name:          { type: string, required: true, pii_class: restricted }
      phone:              { type: string, required: false, pii_class: restricted }
      billing_country:    { type: string, required: true, indexed: true, pii_class: public }
      created_at:         { type: datetime, required: true }
      lifetime_value_usd: { type: decimal, scale: 2, default: "0.00", pii_class: internal }
      segment:            { type: enum, values: [smb, mid_market, enterprise], default: smb, indexed: true }

  - id: Product
    description: "A purchasable product"
    properties:
      sku:                { type: string, required: true, primary: true }
      name:               { type: string, required: true }
      category:           { type: string, required: true, indexed: true }
      price_minor_units:  { type: integer, required: true }
      currency:           { type: string, default: USD }
      inventory_status:   { type: enum, values: [in_stock, low, out_of_stock, discontinued], indexed: true }

  - id: Order
    description: "A customer order containing one or more products"
    properties:
      order_id:           { type: string, required: true, primary: true }
      status:             { type: enum, values: [pending, paid, fulfilled, refunded, cancelled], required: true, indexed: true }
      total_minor_units:  { type: integer, required: true }
      currency:           { type: string, default: USD }
      created_at:         { type: datetime, required: true, indexed: true }
      completed_at:       { type: datetime, required: false }
      shipping_address_country: { type: string, required: true, pii_class: public }

  - id: Shipment
    description: "A physical shipment of products"
    properties:
      shipment_id:        { type: string, required: true, primary: true }
      carrier:            { type: enum, values: [dhl, fedex, ups, australia_post, korea_post] }
      tracking_number:    { type: string, required: false }
      status:             { type: enum, values: [scheduled, in_transit, delivered, returned] }
      shipped_at:         { type: datetime, required: false }
      delivered_at:       { type: datetime, required: false }

  - id: Review
    description: "Customer review of a product"
    properties:
      review_id:          { type: string, required: true, primary: true }
      rating:             { type: integer, minimum: 1, maximum: 5, required: true }
      text:               { type: string, required: false }
      verified_purchase:  { type: boolean, default: false }
      created_at:         { type: datetime, required: true }

relationship_types:
  - id: PLACED_ORDER
    from_entity: Customer
    to_entity:   Order
    cardinality: one-to-many
    on_from_delete: restrict  # don't allow customer deletion if they have orders
    properties:
      placed_at: { type: datetime, required: true }

  - id: CONTAINS_PRODUCT
    from_entity: Order
    to_entity:   Product
    cardinality: many-to-many
    properties:
      quantity:               { type: integer, required: true, minimum: 1 }
      unit_price_minor_units: { type: integer, required: true }
      line_total_minor_units: { type: integer, required: true }

  - id: SHIPPED_VIA
    from_entity: Order
    to_entity:   Shipment
    cardinality: one-to-many
    on_from_delete: cascade

  - id: AUTHORED_REVIEW
    from_entity: Customer
    to_entity:   Review
    cardinality: one-to-many
    on_from_delete: cascade

  - id: REVIEW_OF
    from_entity: Review
    to_entity:   Product
    cardinality: many-to-one

  - id: PURCHASED_PRODUCT_VIA
    from_entity: Customer
    to_entity:   Product
    cardinality: many-to-many
    # Synthetic relationship (derived from PLACED_ORDER + CONTAINS_PRODUCT)
    # Useful for direct querying without 2-hop traversal
    properties:
      first_purchased_at: { type: datetime, required: true }
      total_quantity:     { type: integer, required: true }

indexes:
  - entity: Customer
    on: [email]
  - entity: Customer
    on: [segment, billing_country]
  - entity: Order
    on: [created_at, status]
  - entity: Product
    on: [category, inventory_status]
  - entity: Review
    on: [rating]
```

Register:

```sh
oya ontology schema register \
    --tenant acme-corp \
    --schema-file ./ecommerce-ontology-v1.yaml
```

## Step 2 — Populate with synthetic data (≤ 10 min)

```sh
oya synthetic ontology emit \
    --tenant acme-corp \
    --schema ecommerce-ontology-v1 \
    --customers 1000 \
    --products 50 \
    --orders-per-customer 2.5 \
    --reviews-per-customer 0.8 \
    --shipments-per-order 1.2
```

This generates ~ 1 000 Customers, ~ 50 Products, ~ 2 500 Orders, ~ 800 Reviews, ~ 3 000 Shipments, plus ~ 8 500 relationships.

Verify:

```sh
oya ontology stats --tenant acme-corp
# Output:
#   Customer: 1000
#   Product: 50
#   Order: 2500
#   Shipment: 3000
#   Review: 800
#   PLACED_ORDER edges: 2500
#   CONTAINS_PRODUCT edges: 6200 (avg 2.5 products per order)
#   SHIPPED_VIA edges: 3000
#   AUTHORED_REVIEW edges: 800
#   REVIEW_OF edges: 800
```

## Step 3 — Execute Cypher queries (≤ 15 min)

```sh
# Find all enterprise customers in Australia
oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer {segment:"enterprise", billing_country:"AU"})
    RETURN c.customer_id, c.full_name, c.lifetime_value_usd
    ORDER BY c.lifetime_value_usd DESC
    LIMIT 10
'

# 2-hop: customers + their orders + total spend per customer
oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer)-[:PLACED_ORDER]->(o:Order)
    WHERE o.status = "fulfilled"
    RETURN c.customer_id, c.full_name, count(o) AS order_count, sum(o.total_minor_units) AS total_spend_minor
    ORDER BY total_spend_minor DESC
    LIMIT 20
'

# 3-hop: customers + their orders + the products they purchased + categories
oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer)-[:PLACED_ORDER]->(o:Order)-[:CONTAINS_PRODUCT]->(p:Product)
    WHERE c.segment = "enterprise"
    RETURN p.category, count(DISTINCT c) AS unique_enterprise_customers, sum(o.total_minor_units) AS revenue_minor
    ORDER BY revenue_minor DESC
'

# Customers who reviewed products they didn't purchase
oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer)-[:AUTHORED_REVIEW]->(r:Review)-[:REVIEW_OF]->(p:Product)
    WHERE NOT EXISTS {
        MATCH (c)-[:PLACED_ORDER]->(:Order)-[:CONTAINS_PRODUCT]->(p)
    }
    RETURN c.customer_id, p.sku, r.rating
'
```

The third query (3-hop) is non-trivial. The query planner uses Citus shard-pushdown if the query stays within a single tenant's shard (it does, since tenant_id is the distribution column).

## Step 4 — Use the GraphQL endpoint (≤ 10 min)

oyatie auto-generates a GraphQL schema from the ontology schema. Query at `https://ontology.<cell>.oyatie.local/v1/graphql`:

```graphql
query CustomersWithOrders {
  customers(filter: {segment: enterprise, billing_country: "AU"}, first: 5) {
    customer_id
    full_name
    lifetime_value_usd
    orders(filter: {status: fulfilled}) {
      order_id
      total_minor_units
      products {
        sku
        name
        category
      }
    }
  }
}
```

The auto-generated schema includes:

- One root query per entity type (`customers`, `products`, `orders`, `shipments`, `reviews`).
- One relationship resolver per relationship type (nested queries).
- Filter argument types per property.
- Pagination via `first`, `after`.
- `__schema` introspection for tooling (Apollo, Insomnia, GraphQL Playground).

GraphQL is convenient for client-side use; Cypher is more powerful for ad-hoc analytical queries.

## Step 5 — Time-travel query (≤ 5 min)

Update a customer:

```sh
oya ontology entity update \
    --tenant acme-corp \
    --type Customer \
    --id cust-0001 \
    --properties '{"segment":"mid_market","lifetime_value_usd":"24500.00"}'
```

The previous version (segment=enterprise, lifetime_value_usd=12500.00) is preserved in history. Query as-of:

```sh
oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer {customer_id:"cust-0001"}) RETURN c
' --as-of 2026-05-20T13:00:00Z
# Returns the pre-update version.

oya ontology query --tenant acme-corp --query '
    MATCH (c:Customer {customer_id:"cust-0001"}) RETURN c
'
# Returns the post-update version.
```

The diff between the two:

```sh
oya ontology entity diff \
    --tenant acme-corp \
    --type Customer \
    --id cust-0001 \
    --as-of-a 2026-05-20T13:00:00Z \
    --as-of-b now
# Output:
#   segment: enterprise → mid_market
#   lifetime_value_usd: 12500.00 → 24500.00
```

## Step 6 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "ontology.*" --since 1h | head -30
```

Expected events:

- `ontology.schema.registered` (1)
- `ontology.entity.created` (5 350 — 1k+50+2.5k+3k+800)
- `ontology.relationship.created` (~ 13 300)
- `ontology.entity.updated` (1 — our customer segment update)
- `ontology.query.executed` (several — one per query you ran)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 1h
# Output: chain verified: 18000+ events, signature_gaps: 0
```

## Step 7 — Query embedding-augmented search (≤ 10 min; paid tenant_class feature)

Set up semantic search over your products:

```sh
oya ontology embedding-pipeline create \
    --tenant acme-corp \
    --entity-type Product \
    --properties-to-embed name,category \
    --embedding-model bge-m3
```

After ingest (~ 30 s for 50 products), search semantically:

```sh
oya ontology semantic-search \
    --tenant acme-corp \
    --entity-type Product \
    --query "outdoor camping equipment" \
    --top-k 5
# Returns the top-5 most semantically-similar products by their property embeddings,
# even if they don't contain the exact words "outdoor" or "camping".
```

This bridges structured ontology query + semantic search; an `intelligence` µservice consumer would use this to ground LLM responses in tenant entities.

## What you've learned

- 5-entity + 6-relationship ontology authoring.
- Cypher + GraphQL + REST query surfaces.
- 3-hop graph patterns with sub-second p99 latency.
- Time-travel queries with `--as-of`.
- Audit-chain verification of every entity write.
- Embedding-augmented semantic search bridging ontology + RAG.

Next tutorial: `tutorials/schema-evolution-with-grace-period.md` — evolve the ontology with a deprecation grace period for a property rename.
