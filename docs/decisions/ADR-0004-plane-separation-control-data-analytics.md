---
id: ADR-0004
status: Proposed
doc_status: published
---

# ADR-0004: Plane separation across control / data / analytics with catalog-declared plane class

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0005, ADR-0006, ADR-0011, ADR-0015

---

## Context

Every Oyatie surface — across all all microservices — falls into one of three execution profiles: low-frequency / high-trust / audit-heavy operations that *configure* the system; high-frequency / latency-bounded operations that *execute* requests; and read-mostly operations that *aggregate* for learning, reporting, and FinOps. Mixing these profiles in a single store, a single deployable, or a single transactional unit produces a cascade of operational problems: a control-plane mutation blocks a data-plane query path; an analytics scan blows out a data-plane index cache; a tenant-onboarding event is replayed alongside a billion ad-impression events.

The cohesion thesis (ADR-0001) makes plane discipline more important, not less, because every axis must compose cleanly with every other. Without explicit plane class declared at the catalog layer, cross-microservice calls become accidental cross-plane calls; a control-plane mutation in cloud-IAM accidentally synchronously waits on a data-plane storage write; a search-axis index lifecycle event gets routed through a control-plane queue sized for tens-of-events-per-second and stalls. The only sustainable answer is to make the plane a first-class declaration on every surface.

---

## Decision

Every surface in every axis declares one of three planes, validated at the catalog layer and enforced in CI.

```rust
// crates/oya-foundation-plane-kernel
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Plane {
    /// Low-frequency, high-trust, audit-heavy. Configures and gates.
    Control,
    /// High-frequency, latency-bounded, fan-out scaled. Executes requests.
    Data,
    /// Read-mostly on materialized projections. Observes, aggregates, learns.
    Analytics,
}

pub struct PlaneAssignment {
    pub crate_id: CrateId,
    pub surface: SurfaceId,                 // a logical surface inside the crate
    pub plane: Plane,
    pub cross_plane_calls: Vec<CrossPlaneCall>, // explicit declaration of allowed escapes
}

pub struct CrossPlaneCall {
    pub from_plane: Plane,
    pub to_plane: Plane,
    pub via: CrossPlaneMechanism,           // ProjectionRead | EventReplay | ReadOnlyApi
    pub justification: Justification,       // catalog-recorded rationale
}
```

### Catalog-declared plane

Every flat-crate's `registry/catalog/<crate>.yaml` carries:

```yaml
plane: control | data | analytics
cross_plane_calls:
  - from: control
    to: data
    via: ReadOnlyApi
    justification: "Cloud control-plane lists tenant resources via read-only data-plane projection per ADR-0004"
```

### Strict invariants

1. **A control-plane API never reads from the data-plane store directly.** It reads via a published projection or replays an audited event log (sourced through ADR-0005 eventing backbone). Direct DB-shared-driver access from control-plane code to a data-plane primary fails the boundary validator.
2. **A data-plane surface never writes to the control-plane store directly.** It emits events; the control plane projects them.
3. **Analytics surfaces never write to operational stores.** Reverse-ETL is a control-plane operation, explicitly classified.
4. **Cross-plane calls are explicit contracts.** A PR that adds a cross-plane edge that is not declared in the catalog is rejected by `oya-governance-plane`.

### Plane × Axis matrix (per DESIGN §2)

| Axis | Control plane | Data plane | Analytics plane |
|---|---|---|---|
| 1. SaaS | tenant onboarding, workflow publish, plugin install | workflow execution, plugin invocation | per-tenant retention, NPS |
| 2. Workspace | per-tenant onboarding, doc-template publish | doc edit, mail send, meet hosting | usage analytics, sentiment |
| 3. Vertical | per-vertical onboarding, regulatory-pack install | per-vertical execution, FHIR/EDI exchange | per-vertical KPI |
| 4. Foundry | capability publish, autonomy-ceiling policy publish, model registration; catalog publish, claim-ceiling publish, gate authoring | agent step execution, RAG retrieval; CI lane execution | agent telemetry, eval; scorecard rollups |
| 5. Cloud | resource provisioning, IAM publish, region/AZ register | tenant compute/storage/network I/O | FinOps, capacity planning |
| 6. Search | index lifecycle, crawl scheduling | query, retrieve, rank, serve | ranker training, click stream (privacy-gated) |
| 7. Ads | campaign publish, audience publish, advertiser onboarding | auction, ad serve, click & impression record | attribution, advertiser reporting |

### Cross-plane review CI lane

PRs that change a surface's `plane` field, or add/modify a `cross_plane_calls` entry, are auto-labeled `cross-plane` by `oya-governance-plane`. The label requires:

- A reviewer from each affected plane's owning team.
- A regenerated cross-plane call diagram in the PR body's `## Evidence` section.
- An emission of `EVT-PLANE-CONTRACT-CHANGED` to the audit chain (ADR-0003).

### Boundary

- Applies to: every flat-crate, every surface (where a single crate hosts multiple surfaces, each is declared independently).
- Does not apply to: dev-only test harnesses, ad-hoc benchmark binaries that touch no production data.

---

## Consequences

### Positive

- Control-plane mutations stay independent of data-plane load; an IAM publish does not synchronously block a tenant query.
- Data-plane services keep their hot-path cleanly latency-bounded; analytics scans do not steal CPU from request serving.
- Analytics-plane projections become a first-class output of the eventing backbone (ADR-0005); analytics teams stop competing for OLTP read replicas.
- Cross-plane call diagrams become an authoritative architecture artifact regenerable from the catalog.

### Negative

- Up-front declaration cost on every surface; per-microservice surfaces with mixed read/write profiles must be split, sometimes painfully.
- Some legacy crates (per ADR-0015 migration target) ship with implicit cross-plane edges that must be declared explicitly during migration.
- Read-after-write expectations across planes degrade to read-after-projection-lag (typically ms but bounded by eventing-backbone fan-out); customer-facing UX must be designed around this.

### Operational

- `oya-governance-plane` runs on every PR; cross-plane label triggers two-team review.
- Per-plane SLO catalog: control plane targets p99 < 1 s; data plane per-microservice (ads < 100 ms; search < 200 ms; SaaS workflow step < 500 ms); analytics targets per-pipeline freshness window.
- Runbooks: `runbooks/cross-plane-call-introduction.md`, `runbooks/plane-class-correction.md`.
- The plane × axis matrix above is the source of truth; quarterly review by `council-architecture` regenerates it from the catalog.

---

## Alternatives considered

### Alternative A — No plane separation; every service handles all profiles

- **Pros:** simpler service layout.
- **Cons:** every operational incident in the legacy corpus has a plane-mix root cause (analytics scans blowing out the OLTP cache; control-plane mutations blocking on data-plane writes).
- **Rejected because:** the failure mode is recurrent and unrecoverable per release.

### Alternative B — Two planes (operational vs analytics) without separating control vs data

- **Pros:** one fewer category to declare.
- **Cons:** control-plane operations (low-frequency, high-trust, audit-heavy) have profoundly different scaling characteristics from data-plane execution; conflating them defeats the cohesion claim's reliability story.
- **Rejected because:** AWS, Google, Azure, and every mature cloud provider has the three-plane model for exactly this reason.

### Alternative C — Plane declared at deployment-only (Helm chart annotation)

- **Pros:** zero source-tree change.
- **Cons:** boundary validation requires source-tree visibility; Helm-only declaration cannot enforce import discipline.
- **Rejected because:** ADR-0011 contract registry requires plane in catalog for cross-microservice review.

---

## Open questions

1. **Q1.** Some surfaces are legitimately *both* control and data (e.g. tenant-onboarding writes a control-plane record and synchronously seeds a data-plane shard). Default: declare as control; the data-plane seed is a `CrossPlaneCall { via: ReadOnlyApi }` to a seed-service. → owner: `council-architecture`.
2. **Q2.** Do regional packs declare per-pack plane overrides? Default: NO; packs plug into seams that have a fixed plane class. → ADR-0010.
3. **Q3.** Foundry's evidence-emission step itself: control or data? Default: data (it is per-step, high-frequency); the trust-portal projection is analytics. → ADR-0003 + ADR-0007.
4. **Q4.** Per-cell plane affinity (ADR-0009) — does a cell host all three planes, or do separate cells host each? Default: data-plane cells host data; control + analytics planes are global per region. → ADR-0009.

---

## References

- `docs/DESIGN.md` §2 (plane separation), §10 (cross-microservice contracts: `Plane class`), §3.0.5.3 (blast-radius classes)
- `docs/PRD.md` §6 constraint 9 (plane separation)
- `docs/CONTRADICTION-LEDGER.md` resolution batches: cross-microservice-contracts requires plane discipline
- ADR-0001 (cohesion thesis), ADR-0005 (eventing backbone — projection mechanism), ADR-0011 (cross-microservice contract registry — plane is a contract field), ADR-0015 (architectural flattening — kernel/domain/app/api/worker/adapter roles map cleanly to planes)
