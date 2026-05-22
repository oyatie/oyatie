---
ip_id: IP-002
title: "IP-002: connector-catalog domain + kernel crates"
microservice: connect
bounded_context: connector-catalog
layers: [domain, kernel]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0243, ADR-0249]
companion_docs:
  - microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml
  - microservices/connect/catalog/oya-connect-connector-catalog-kernel.yaml
  - microservices/connect/contracts/openapi/connect-integration.yaml
doc_status: published
---

# IP-002: connector-catalog domain + kernel crates

## Purpose

Implement `oya-connect-connector-catalog-domain` and `oya-connect-connector-catalog-kernel` — the in-memory connector registry, category taxonomy, rate-limit profile indexing, and compliance-posture query surface.

## Acceptance criteria

1. `ConnectorCatalog` domain struct loads all `catalog/connectors/*.yaml` entries at startup.
2. `CatalogQuery { q, category, pack_filter, audience_type }` returns paginated `Vec<ConnectorSummary>`.
3. Compliance filter: `pack_filter = "pack-kr"` excludes connectors where `pack_kr NOT IN pack_allow_list`.
4. Emergency-services connectors (`pagerduty`, `twilio` with `emergency_services_class=true`) always returned for authorized principals regardless of pack filter.
5. Cedar policy `connector-catalog-publishing.cedar` consulted for publish/retire operations; read-only ops bypass Cedar (no PII, no mutation).
6. Schema-drift detection: `ConnectorSchemaVersion` tracked; `SchemaDriftDetected` audit event emitted when vendor schema changes.
7. Zero `clippy::deny(warnings)` failures; ≥85% line coverage.

## Crate layout

```
crates/
  oya-connect-connector-catalog-kernel/
    src/lib.rs           — ConnectorSummary, ConnectorId, CatalogQuery types
  oya-connect-connector-catalog-domain/
    src/lib.rs           — ConnectorCatalog, CatalogQueryService, SchemaDriftDetector
    src/loader.rs        — YAML loader from catalog/connectors/*.yaml
    src/filter.rs        — pack_filter + compliance posture filter
```

## Key types (Rust)

```rust
// kernel
pub struct ConnectorId(pub String);
pub struct ConnectorSummary {
    pub id: ConnectorId,
    pub display_name: String,
    pub category: ConnectorCategory,
    pub status: ConnectorStatus,
    pub rate_limit_tier: RateLimitTier,
    pub emergency_services_class: bool,
    pub pack_allow_list: Vec<PackId>,
}

// domain
pub struct CatalogQuery {
    pub q: Option<String>,
    pub category: Option<ConnectorCategory>,
    pub pack_filter: Option<PackId>,
    pub audience_type: Option<AudienceType>,
    pub page: u32,
    pub page_size: u32,
}

impl ConnectorCatalog {
    pub fn query(&self, q: CatalogQuery) -> Result<Page<ConnectorSummary>, CatalogError>;
    pub fn get_by_id(&self, id: &ConnectorId) -> Option<&ConnectorEntry>;
}
```

## Failure modes

1. **YAML parse failure at startup** → panic with actionable error (fail-fast; no silent degradation on catalog load)
2. **Pack filter excludes all connectors** → return empty page, not error
3. **Emergency-services connector absent from catalog** → `CatalogLoadError::MissingEmergencyServicesConnector` (hard startup failure)

## Definition of done

- [ ] Crates compile with `cargo build -p oya-connect-connector-catalog-domain`
- [ ] Unit tests: catalog query with pack filter, emergency-services bypass, schema-drift detection
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Catalog record files updated (already authored at IP-002 time)


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio. See `microservices/connect/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
