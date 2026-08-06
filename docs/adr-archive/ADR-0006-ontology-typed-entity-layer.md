---
id: ADR-0006
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0006: Ontology as the engine-enforced typed-entity layer with per-property tier classification

> **Status:** Accepted
> **Owner:** `oya-ontology`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — "Ontology" renamed to "Ontology")
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0011, ADR-0055, ADR-0059

---

## Context

Tenant data in Oyatie crosses six storage shapes: scalar columns, dense vectors (embeddings, ranker features), time-series streams (telemetry, metering, audit, behavioral), geospatial geometries (logistics, industrial, public-sector mapping), encrypted blobs (PHI, PCI, PII at rest), and structured composites (FHIR resources, EDI envelopes, FHIR-extension-shaped clinical objects). A naive "one ORM-shaped table per entity" approach forces every microservice to either push these shapes into JSON columns (losing index + privacy gates) or to spawn per-service stores (losing cohesion + audit + tenancy isolation).

The cohesion thesis (ADR-0001) and the per-tenant isolation invariant (ADR-0002) require a single typed-entity layer with engine-enforced row-level isolation per tenant. The Data Use Boundary (ADR-0008) requires every property to carry a data-class annotation that propagates through search, analytics, and ads pipelines.

"Ontology" was the prior name for this layer. Per session decision 2026-05-13, it is renamed to **Ontology**, matching Palantir's established term. Bominal ADR-0106 ("Ontology architecture") translates to "Ontology architecture" in oyatie glossary. See ADR-0055.

---

## Decision

We adopt the **Ontology** as Oyatie's single typed-entity layer. The kernel is `oya-ontology-entity-kernel`; per-property-tier adapters live in `oya-ontology-adapter-{scalar,vector,timeseries,geo,ciphertext,struct}-*`. Every entity carries a `TenantId`, an `ObjectId`, a `PropertyTier` per declared property, a `data_class` per property (per ADR-0008), and an audit-chain emission hook (ADR-0003) on every state mutation.

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-ontology-entity-kernel`: slot2 = `ontology` (registered µservice, information adapter layer); slot3 = `entity` (BC); slot4 = `kernel` (pure types + ports)

### Property tiers (closed enumeration)

| Tier | Storage shape | Adapter (initial) | Index strategy |
|---|---|---|---|
| `scalar` | numeric, string, bool, enum, foreign keys | Postgres + Citus shard | btree, hash, partial |
| `vector` | dense float vector with declared dim | pgvector → in-house HNSW/IVF as scale demands | HNSW, IVF, PQ |
| `timeseries` | (timestamp, value, tags) tuples | Postgres TimescaleDB extension or in-house compressed columnar | time-partitioned + tag index |
| `geo` | point, line, polygon, geometry collection | PostGIS | GiST + R-tree |
| `ciphertext` | per-record envelope-encrypted blob | KMS + DEK + KEK with key-shred capability | none (opaque) |
| `struct` | composite (FHIR Resource, EDI envelope, structured clinical/financial object) | jsonb with schema validation + per-path indexes | GIN + path-specific |

### Engine-enforced per-tenant isolation

Per-tenant row-level isolation is enforced at the Postgres adapter via Row-Level Security (RLS) policies bound to `current_setting('app.tenant_id')`. The kernel's `OntologyHandle` MUST set this session variable before every query.

```rust
// oya-ontology-entity-kernel
pub struct EntityHandle<'tx, E: Entity> {
    pub tenant: TenantId,
    pub object_id: ObjectId,
    pub schema_version: u16,
    pub property_tiers: BTreeMap<PropertyId, (PropertyTier, DataClass)>,
    pub _tx: &'tx Transaction<'tx>,
}

pub trait Entity: Sized {
    const ENTITY_NAME: &'static str;
    const PROPERTY_DECLARATIONS: &'static [PropertyDecl];
    fn validate(&self) -> Result<(), EntityError>;
    fn audit_emission(&self, op: MutationOp) -> AuditEvent;       // ADR-0003
}
```

### Ontology bounded contexts

The Ontology µservice decomposes into these BCs (BNF v4.1):
- `oya-ontology-entity-kernel` / `oya-ontology-entity-domain` / `oya-ontology-entity-adapter`
- `oya-ontology-link-kernel` / `oya-ontology-link-domain`
- `oya-ontology-action-kernel` / `oya-ontology-action-domain`
- `oya-ontology-function-kernel`
- `oya-ontology-agent-gateway-kernel` (per Bominal ADR-0107, inherited)
- `oya-ontology-audit-chain-adapter` (chains to ADR-0003)
- `oya-ontology-pillar-kernel` (org-pillar / person-pillar per Bominal ADR-0132, inherited)

### Schema evolution

Per-property tier changes are catalog-mediated. Adding a property is backward-compat. Removing a property is a wave-bound deprecation per ADR-0019. Changing a property's tier (loosening) requires explicit human approval per the data-use weakening rule (ADR-0008 §2.2.10).

---

## Consequences

### Positive

- The 12-class data taxonomy (ADR-0008) becomes enforceable at the property level.
- Engine-enforced per-tenant row-level isolation eliminates cross-tenant leak bugs.
- Search-axis indexing and ads-axis eligibility become catalog-driven.

### Negative

- Per-property declaration overhead is real; mitigated by codegen from the catalog.
- Ciphertext access overhead (~5 ms per record for KMS round-trip); cached per session.

### Operational

- On-call: `EVT-ONTOLOGY-RLS-MISMATCH` (catalog vs policy drift) pages within 5 minutes.
- Runbooks: `runbooks/ontology-property-tier-migration.md`, `runbooks/ontology-rls-policy-regenerate.md`.
- CI: `oya-check-property-tier`, `oya-check-rls`, `oya-check-ontology-cohesion`.

---

## Alternatives considered

### Alternative A — Plain ORM (Diesel/SeaORM) + per-microservice stores

- **Rejected because:** ADR-0001 forbids substrate forking.

### Alternative B — Document store as primary (MongoDB/CouchDB)

- **Rejected because:** license posture (MongoDB SSPL forbidden by ADR-0013) + tier enforcement.

---

## Related

- ADR-0001 (cohesion thesis — one Ontology)
- ADR-0002 (Tenant kernel — RLS binding)
- ADR-0003 (audit chain)
- ADR-0007 (Cedar policy enforcement)
- ADR-0008 (data class boundary)
- ADR-0055 (Ontology renamed to Ontology)
- ADR-0059 (Workflow + Ontology = ecosystem adapter layer)
- `[[feedback-glossary-ontology-not-object-graph]]` — session decision 2026-05-13
- `[[feedback-workflow-objectgraph-adapter-layer]]` — Ontology as information adapter
- Bominal ADR-0106 (Ontology = Ontology architecture in oyatie glossary)
- Bominal ADR-0107 (Ontology agent gateway, inherited)
- Bominal ADR-0132 (org/person pillar, inherited)
