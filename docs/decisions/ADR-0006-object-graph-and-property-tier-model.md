# ADR-0006: Object Graph as the engine-enforced typed-entity layer with per-property tier classification

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `platform-object-graph`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0011

---

## Context

Tenant data in Oyatie crosses six storage shapes: scalar columns, dense vectors (embeddings, ranker features), time-series streams (telemetry, metering, audit, behavioral), geospatial geometries (logistics, industrial, public-sector mapping), encrypted blobs (PHI, PCI, PII at rest), and structured composites (FHIR resources, EDI envelopes, FHIR-extension-shaped clinical objects). A naive "one ORM-shaped table per entity" approach forces every axis to either push these shapes into JSON columns (losing index + privacy gates) or to spawn per-axis stores (losing cohesion + audit + tenancy isolation).

The cohesion thesis (ADR-0001) and the per-tenant isolation invariant (ADR-0002) require a single typed-entity layer with engine-enforced row-level isolation per tenant. The Data Use Boundary (ADR-0008) requires every property to carry a data-class annotation that propagates through search, analytics, and ads pipelines. The 12-class taxonomy needs to be enforced at the property tier — not at the surface that consumes it — so the inference-boundary check (most-restrictive-class inheritance) holds even when a derived view is constructed by a different axis.

---

## Decision

We adopt the **Object Graph (OG)** as Oyatie's single typed-entity layer. The kernel is `crates/oya-platform-object-graph-kernel`; per-property-tier adapters live in `crates/oya-platform-object-graph-adapter-{scalar,vector,timeseries,geo,ciphertext,struct}-*`. Every entity carries a `TenantId`, an `ObjectId`, a `PropertyTier` per declared property, a `data_class` per property (per ADR-0008), and an audit-chain emission hook (ADR-0003) on every state mutation.

### Property tiers (closed enumeration)

| Tier | Storage shape | Adapter (initial) | Index strategy |
|---|---|---|---|
| `scalar` | numeric, string, bool, enum, foreign keys | Postgres + Citus shard | btree, hash, partial |
| `vector` | dense float vector with declared dim | pgvector → in-house HNSW/IVF as scale demands | HNSW, IVF, PQ |
| `timeseries` | (timestamp, value, tags) tuples | Postgres TimescaleDB extension or in-house compressed columnar | time-partitioned + tag index |
| `geo` | point, line, polygon, geometry collection | PostGIS | GiST + R-tree |
| `ciphertext` | per-record envelope-encrypted blob | KMS + DEK + KEK with key-shred capability | none (opaque) |
| `struct` | composite (FHIR Resource, EDI envelope, structured clinical/financial object) | jsonb with schema validation + per-path indexes | GIN + path-specific |

### Composite properties

A property MAY be a *composite* of two or more tiers (e.g. an industrial `Asset` carries both a `geo` location and a `timeseries` telemetry stream). Each constituent property carries its own tier + data class; the parent entity carries the conjunction.

### Engine-enforced per-tenant isolation

Per-tenant row-level isolation is enforced at the Postgres adapter via Row-Level Security (RLS) policies bound to `current_setting('app.tenant_id')`. The kernel's `ObjectGraphHandle` MUST set this session variable before every query; the boundary validator (`oya-foundry-fitness-rls`) refuses adapters that issue queries without the binding. RLS policies are generated from the catalog, not handwritten — drift between catalog and policy fails CI.

```rust
// crates/oya-platform-object-graph-kernel
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

pub struct PropertyDecl {
    pub id: PropertyId,
    pub tier: PropertyTier,
    pub data_class: DataClass,
    pub regulatory_packs: &'static [RegulatoryPackId],
    pub indexable_in_search: SearchEligibility,                  // per ADR-0008
    pub ad_targetable: AdEligibility,                             // per ADR-0008
}
```

### Per-property tier enforcement gates

- **Lint-time** (`oya-foundry-fitness-property-tier`) — every property declaration in `*.proto`, SQL DDL, or Rust entity must carry tier + data_class. Missing → CI fail.
- **Compile-time** — the kernel's `EntityHandle` API requires per-property-tier accessor methods; cross-tier access (e.g. reading a `ciphertext` as a `scalar`) does not type-check.
- **Runtime** — adapter dispatch enforces per-tier semantics (e.g. ciphertext access decrypts per-DEK and emits an audit record; vector access checks dimension against catalog declaration).
- **Cross-axis** — search-axis ingestion and ads-axis eligibility evaluation read the per-property `indexable_in_search` / `ad_targetable` flags; bypass attempts fail at the singleton gate (ADR-0008 §2.2.4 layer 3).

### Schema evolution

Per-property tier changes are catalog-mediated. Adding a property is backward-compat (consumers ignore unknown fields). Removing a property is a wave-bound deprecation per ADR-0019 doc-update protocol. Changing a property's tier (e.g. `scalar` → `ciphertext` for tightening) is automatic; the reverse (loosening) requires explicit human approval per the data-use weakening rule (ADR-0008 §2.2.10).

### Boundary

- Applies to: every persisted tenant entity in every axis (SaaS workflow state, Workspace docs, vertical clinical/industrial/financial records, Foundry capability state, Cloud resource inventory, Search document corpus, Ads campaign + audience).
- Does not apply to: per-cell ephemeral cache (e.g. a search-axis term cache); per-request scratch state; build-time fixtures.

---

## Consequences

### Positive

- The 12-class data taxonomy (ADR-0008) becomes enforceable at the property level — derived views inherit the most-restrictive class because the OG knows the lineage.
- Engine-enforced per-tenant row-level isolation eliminates an entire class of cross-tenant leak bugs (Postgres RLS + RLS-policy-from-catalog drift detection).
- Search-axis indexing and ads-axis eligibility become catalog-driven, not surface-implemented — adding a property automatically adjusts both downstream surfaces.
- Per-property-tier adapter dispatch lets each axis pick the right storage shape without re-implementing tenancy + audit + RLS from scratch.

### Negative

- Per-property declaration overhead is real. Adding an entity is more verbose than a plain ORM; mitigated by codegen from the catalog.
- Cross-tier composite indexes (e.g. geo + time) require careful adapter authoring; per-axis query planners must understand the tier mix.
- Ciphertext access overhead (~5 ms per record for KMS round-trip) is the cost of correctness; cached with per-session DEK unwrap, the amortized cost is ~1 ms.

### Operational

- On-call: `EVT-OG-RLS-MISMATCH` (catalog vs policy drift) pages within 5 minutes.
- Runbooks: `runbooks/og-property-tier-migration.md`, `runbooks/og-rls-policy-regenerate.md`, `runbooks/og-ciphertext-key-shred.md` (DSR cascade).
- CI: `oya-foundry-fitness-property-tier`, `oya-foundry-fitness-rls`, `oya-foundry-fitness-og-cohesion` (catalog vs adapter drift).
- Per-property metrics: tier-usage histogram, KMS-roundtrip p99, RLS-binding miss rate (target: 0).

---

## Alternatives considered

### Alternative A — Plain ORM (Diesel/SeaORM) + per-axis stores

- **Pros:** familiar.
- **Cons:** every axis re-implements tenancy + audit + privacy; drift guaranteed; LEDG-009 demonstrated the failure mode.
- **Rejected because:** ADR-0001 forbids substrate forking.

### Alternative B — Document store as primary (MongoDB/CouchDB)

- **Pros:** schema flexibility.
- **Cons:** weak per-property-tier semantics; license posture (MongoDB SSPL forbidden by ADR-0013); RLS-equivalent enforcement is non-trivial.
- **Rejected because:** license + tier enforcement.

### Alternative C — Per-vertical entity layer with shared base traits

- **Pros:** per-vertical clarity.
- **Cons:** cross-axis contracts (Search ingestion, Ads eligibility, Foundry capability state) need a unified entity surface.
- **Rejected because:** cohesion claim requires one OG.

---

## Open questions

1. **Q1.** Vector dim catalog enforcement — should the catalog declare a maximum dim per axis, or allow per-property unconstrained? Default: per-axis maximum (search ≤ 4096; SaaS ≤ 1536). → owner: `axis-search`.
2. **Q2.** Per-record ciphertext cost — when does the per-DEK overhead justify a per-aggregate DEK instead? Default: per-record for `HARD_DENY` classes; per-aggregate for others. → ADR-0008 §2.5 Q7.
3. **Q3.** Schema-evolution review cadence — quarterly catalog audit vs per-PR? Default: per-PR validator + quarterly council audit. → ADR-0019.
4. **Q4.** Composite-property query planner — does the kernel ship a unified planner, or do per-axis adapters compose? Default: adapter-composed initially; kernel planner if perf demands. → owner: `platform-object-graph`.

---

## References

- `docs/DESIGN.md` §10 (cross-axis contract `Object Graph property tier`)
- `docs/PRIVACY-PROGRAM.md` §2.2.1 (12-class taxonomy), §2.2.4 (structural enforcement), §2.2.5 (inference boundary)
- `docs/GLOSSARY.md` §8 ("Object Graph (OG)")
- `docs/TOOLCHAIN.md` §3 (Postgres + Citus per ADR-0045; pgvector day-1 per ADR-0044/0177; PostGIS; envelope encryption)
- ADR-0001 (cohesion), ADR-0002 (Tenant kernel — RLS binding), ADR-0003 (audit chain), ADR-0007 (Cedar policy enforcement), ADR-0008 (data class boundary), ADR-0009 (cell architecture — per-tenant per-cell shard alignment), ADR-0011 (catalog → property declarations)
