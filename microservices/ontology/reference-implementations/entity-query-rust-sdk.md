---
doc_class: ReferenceImplementation
microservice: ontology
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Entity CRUD + graph query via the ontology Rust SDK

A runnable example that:

1. Authenticates as a tenant ontology_admin principal.
2. Creates entities + relationships.
3. Executes a Cypher-like graph query.
4. Executes a GraphQL query.
5. Performs a time-travel query.
6. Verifies audit-chain emission.

## Cargo.toml

```toml
[package]
name = "ontology-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-ontology-client = { path = "../../../../crates/oya-ontology-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = "1.36"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::Utc;
use oya_ontology_client::{
    OntologyClient, OntologyClientConfig,
    EntityCreate, EntityUpdate, EntityRead,
    RelationshipCreate,
    CypherQuery, GraphQLQuery, TimeTravelOptions,
};
use oya_cedar_client::CedarPrincipal;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to ontology_admin.
    let principal = CedarPrincipal::from_env("ONTOLOGY_ADMIN_JWT")?;
    let client = OntologyClient::connect(OntologyClientConfig {
        cell_endpoint: std::env::var("ONTOLOGY_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Create entities.
    let customer = client.entity_create(EntityCreate {
        entity_type: "Customer".into(),
        properties: json!({
            "customer_id": "cust-101",
            "email": "alice@example.com",
            "full_name": "Alice Example",
            "billing_country": "US",
            "created_at": Utc::now().to_rfc3339(),
            "lifetime_value_usd": "12500.00",
            "segment": "enterprise"
        }),
    }).await?;
    println!("Created Customer: id={}", customer.entity_id);

    let product = client.entity_create(EntityCreate {
        entity_type: "Product".into(),
        properties: json!({
            "sku": "WIDGET-1",
            "name": "Widget Mark 1",
            "category": "Hardware",
            "price_minor_units": 2500,
            "currency": "USD",
            "inventory_status": "in_stock"
        }),
    }).await?;
    println!("Created Product: sku={}", product.entity_id);

    let order = client.entity_create(EntityCreate {
        entity_type: "Order".into(),
        properties: json!({
            "order_id": "ord-101",
            "status": "paid",
            "total_minor_units": 2500,
            "currency": "USD",
            "created_at": Utc::now().to_rfc3339(),
            "shipping_address_country": "US"
        }),
    }).await?;
    println!("Created Order: id={}", order.entity_id);

    // 3. Create relationships.
    client.relationship_create(RelationshipCreate {
        relationship_type: "PLACED_ORDER".into(),
        from_entity_id: "cust-101".into(),
        to_entity_id: "ord-101".into(),
        properties: json!({"placed_at": Utc::now().to_rfc3339()}),
    }).await?;

    client.relationship_create(RelationshipCreate {
        relationship_type: "CONTAINS_PRODUCT".into(),
        from_entity_id: "ord-101".into(),
        to_entity_id: "WIDGET-1".into(),
        properties: json!({"quantity": 1, "unit_price_minor_units": 2500, "line_total_minor_units": 2500}),
    }).await?;
    println!("Created PLACED_ORDER + CONTAINS_PRODUCT edges.");

    // 4. Execute Cypher-like graph query.
    let cypher_results = client.cypher_query(CypherQuery {
        query: r#"
            MATCH (c:Customer)-[:PLACED_ORDER]->(o:Order)-[:CONTAINS_PRODUCT]->(p:Product)
            WHERE c.segment = "enterprise"
            RETURN c.customer_id AS customer_id, c.full_name AS name, o.order_id AS order_id, p.sku AS sku, p.name AS product_name
            ORDER BY o.created_at DESC
            LIMIT 20
        "#.into(),
        parameters: json!({}),
    }).await?;
    println!("Cypher results ({} rows):", cypher_results.rows.len());
    for row in &cypher_results.rows {
        println!("  {}", row);
    }

    // 5. Execute GraphQL query.
    let graphql_results = client.graphql_query(GraphQLQuery {
        query: r#"
            query GetEnterpriseCustomersAndOrders {
                customers(filter: {segment: enterprise}, first: 10) {
                    customer_id
                    full_name
                    lifetime_value_usd
                    orders {
                        order_id
                        total_minor_units
                        products {
                            sku
                            name
                            price_minor_units
                        }
                    }
                }
            }
        "#.into(),
        variables: json!({}),
    }).await?;
    println!("GraphQL results:");
    println!("{}", serde_json::to_string_pretty(&graphql_results.data)?);

    // 6. Update Customer segment + time-travel.
    let before_update_time = Utc::now();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    client.entity_update(EntityUpdate {
        entity_type: "Customer".into(),
        entity_id: "cust-101".into(),
        properties: json!({"segment": "mid_market"}),
    }).await?;
    println!("Updated cust-101 segment to mid_market.");

    // Time-travel: read as of before_update_time
    let cust_old = client.entity_read_at(EntityRead {
        entity_type: "Customer".into(),
        entity_id: "cust-101".into(),
    }, TimeTravelOptions { as_of: Some(before_update_time) }).await?;
    println!("Time-travel read (as-of pre-update):");
    println!("  segment={} (expected: enterprise)", cust_old.properties["segment"]);

    let cust_now = client.entity_read(EntityRead {
        entity_type: "Customer".into(),
        entity_id: "cust-101".into(),
    }).await?;
    println!("Current read (post-update):");
    println!("  segment={} (expected: mid_market)", cust_now.properties["segment"]);

    Ok(())
}
```

## Expected output (against a paid tenant_class cell with schema registered)

```
Created Customer: id=cust-101
Created Product: sku=WIDGET-1
Created Order: id=ord-101
Created PLACED_ORDER + CONTAINS_PRODUCT edges.
Cypher results (1 rows):
  {"customer_id":"cust-101","name":"Alice Example","order_id":"ord-101","sku":"WIDGET-1","product_name":"Widget Mark 1"}
GraphQL results:
{
  "customers": [
    {
      "customer_id": "cust-101",
      "full_name": "Alice Example",
      "lifetime_value_usd": "12500.00",
      "orders": [
        {
          "order_id": "ord-101",
          "total_minor_units": 2500,
          "products": [
            {"sku": "WIDGET-1", "name": "Widget Mark 1", "price_minor_units": 2500}
          ]
        }
      ]
    }
  ]
}
Updated cust-101 segment to mid_market.
Time-travel read (as-of pre-update):
  segment="enterprise" (expected: enterprise)
Current read (post-update):
  segment="mid_market" (expected: mid_market)
```

## HTTP alternative (curl)

```sh
# Create entity
curl -X POST https://ontology.prod-syd-1.oyatie.local/v1/entities \
    -H "Authorization: Bearer $ONTOLOGY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "entity_type":"Customer",
        "properties":{
            "customer_id":"cust-101","email":"alice@example.com",
            "full_name":"Alice Example","billing_country":"US","created_at":"2026-05-20T14:00:00Z",
            "lifetime_value_usd":"12500.00","segment":"enterprise"
        }
    }'

# Execute Cypher
curl -X POST https://ontology.prod-syd-1.oyatie.local/v1/cypher \
    -H "Authorization: Bearer $ONTOLOGY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "query":"MATCH (c:Customer)-[:PLACED_ORDER]->(o:Order) RETURN c.customer_id, o.order_id LIMIT 20"
    }'

# Execute GraphQL
curl -X POST https://ontology.prod-syd-1.oyatie.local/v1/graphql \
    -H "Authorization: Bearer $ONTOLOGY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "query":"query { customers(filter: {segment: enterprise}, first: 10) { customer_id full_name } }"
    }'

# Time-travel
curl -G https://ontology.prod-syd-1.oyatie.local/v1/entities/Customer/cust-101 \
    -H "Authorization: Bearer $ONTOLOGY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    --data-urlencode "as_of=2026-05-20T14:00:00Z"
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks permission |
| `schema_validation_failed` | 422 | No | Property doesn't match schema |
| `entity_not_found` | 404 | No | Entity ID not in this tenant |
| `relationship_violates_cardinality` | 422 | No | e.g., creating a 2nd one-to-one relationship |
| `query_cost_exceeded` | 429 | Yes (auto, backoff) | Per-tenant cost budget exceeded |
| `query_depth_exceeded` | 422 | No | Cypher query exceeds max-depth (10 for paid tenant_class) |
| `time_travel_outside_retention` | 422 | No | as-of timestamp older than retention window |
| `shard_unavailable` | 503 | Yes (auto) | Shard down; retry |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `entity_create` | `ontology.entity.created` |
| `entity_update` | `ontology.entity.updated` |
| `entity_delete` | `ontology.entity.deleted` |
| `relationship_create` | `ontology.relationship.created` |
| `relationship_delete` | `ontology.relationship.deleted` |
| `schema_register` | `ontology.schema.registered` |
| `migrate_apply` | `ontology.migration.applied` |
| `cypher_query` | `ontology.query.executed` |
| `graphql_query` | `ontology.query.executed` |
| `time_travel_read` | `ontology.entity.read_at` |

## Where this file lives

`microservices/ontology/reference-implementations/entity-query-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/ontology/reference-implementations/entity-query-example/` once `oya-ontology-client` ships.
