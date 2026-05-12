# Oyatie — Product PRD Template

> Use this template for every per-product PRD under `products/<product-id>/PRD.md`. Copy verbatim, then fill in. Sections marked **required** must be populated before the PRD can move from `draft` → `preview`.
>
> **Pattern:** every product reads up to this template, fills in slice-specific content, and links *up* to the cross-cutting consolidated docs. No product re-states content already in `PRD.md` / `DESIGN.md` / `PRIVACY-PROGRAM.md` / `GLOSSARY.md` — instead, it links and adds slice-specific detail.

---

# Oyatie — Product PRD: <product-name>

> **Status:** draft / preview / stable / GA *(industry-standard labels per [GLOSSARY.md §11](../GLOSSARY.md))*
> **Owning team:** `teams/<team-id>/CHARTER.md` (placeholder — fill in concrete team path when authoring)
> **Owning axis:** saas / vertical-X / agent-runtime (Foundry) / foundry / cloud / search / ads-analytics
> **Catalog reference:** registry/catalog/<context>.yaml entries
> **Last updated:** YYYY-MM-DD by <author>

## 1. North star (required)

One paragraph: what this product *is*, who it serves, and why it can only exist as part of Oyatie's cohesive ecosystem (not as a standalone offering).

## 2. Target users (required)

Per-persona table:
| Persona | What they get | What they pay for |
|---|---|---|

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|

### 3.2 Out-of-scope (anti-scope)

Bulleted list. Anti-scope is binding; promotion to in-scope requires a council decision.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

Which bounded context this product owns (per [DESIGN.md §1](../DESIGN.md)). Cite the flat-crates target prefix (e.g. `crates/oya-foundry-*`).

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — DB, HTTP client, KMS, eventing impls
api       — inbound HTTP/gRPC servers
worker    — inbound queue/Kafka consumers
runtime   — composition root (binary)
```

For this product, list each crate name and one-line role.

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|

(Mirror in [DESIGN.md §10](../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-<context>-kernel-*`)

For each entity:

```rust
// example
pub struct EntityName {
    pub id: EntityId,
    pub tenant_id: TenantId,            // every record carries tenant
    pub region: RegionCode,             // for cell-routing
    pub data_class: DataClass,          // per Data Use Boundary ADR
    pub /* ...slice-specific fields... */: ...,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: u32,
}
```

Include:
- All entities, their fields, their invariants.
- Value objects (immutable, identity-less).
- Enums + their finite domains.
- Cardinality between entities (1:1, 1:N, M:N).
- Per-field `data_class` annotation (per [PRIVACY-PROGRAM.md §2.2.1](../PRIVACY-PROGRAM.md)).
- Per-entity `plane` declaration (control / data / analytics).

### 5.2 Aggregate boundaries

Which entities cluster into aggregates. Cite the consistency boundary.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|

### 5.4 Event schemas (events emitted)

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|

(All events go through the canonical eventing backbone per ADR-0050/0174 + outbox pattern.)

### 5.5 Index / search-index touchpoints

If this product's data flows into the search axis, declare:

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../DESIGN.md) + ADR-0003, every regulated capability must emit. List:

| Operation | Emits topic | Required fields |
|---|---|---|

### 5.7 Schema migration policy

Versioning, reversibility, dry-run gate.

## 6. Optimization practices (required) — *slice-level*

For this product, declare:

| Practice | Implementation choice |
|---|---|
| Cell routing | (which key the cells route on) |
| Sharding strategy | (per-tenant / per-key / per-region) |
| Caching tier | (in-memory + Redis + CDN as appropriate) |
| Bulk endpoint contract | (what bulk endpoints exist) |
| Pagination | (cursor-based, page size, filter contract) |
| Idempotency | (idempotency-key surface) |
| Batch dispatch | (which operations batch + the batch trigger) |
| Backpressure | (how downstream signals back) |
| Hot-path benchmarks | (which paths have benchmark gates) |
| Agent-driven optimization loops | (which Foundry capabilities tune this product autonomously) |
| FinOps unit-economics | (per-tenant / per-call cost model) |
| Build-cache and CI affected-graph | (which crates are in the affected graph) |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../DESIGN.md), declare:

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|

If the product is *region-agnostic* (e.g. Foundry), say so explicitly and explain why.

## 8. In-house vs external dependency posture (required)

Per the in-house build preference (PRD §3.1 §6 constraint), declare:

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|

Allowed maturity tier: `kernel-grade` (axum / tokio / serde / rustls / postgres-driver / kernel) only without ADR; everything else needs an ADR.
License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review.

## 9. Success metrics (required)

| Metric | Wave-preview target | Wave-stable target | Wave-GA target |
|---|---|---|---|

Plus structural metrics: cross-axis-contract-violation count = 0; audit-chain emission completeness = 100%; foundation-bypass count not increasing.

## 10. Risks + mitigations

Per-product risk register slice. Mirror to [`RISK-REGISTER.md`](../RISK-REGISTER.md).

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|

## 11. Open questions

Council-pending items.

## 12. Decision log

Per-product decisions (smaller-scope counterpart to ADRs). Link any cross-cutting ADR.

## 13. Sources scanned

Per-product source list (kept fresh).

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `<product-id>` | `axis-<id>` or `vertical-<id>` | scope, contract, capability | monthly | <upstream consolidated-docs the product depends on> |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` row for this product
- `machine-readable/catalog.json` row pointing at this PRD path
- `machine-readable/contracts.json` if this product exposes or consumes a cross-axis contract
- `machine-readable/risks.json` if this product adds a risk
- `machine-readable/glossary.json` if this product introduces a domain term

## Validation checks

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` (or is a planned target on the roadmap)
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
