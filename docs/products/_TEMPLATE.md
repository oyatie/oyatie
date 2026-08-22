---
doc_status: published
---

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

## 2a. Acceptance criteria (required) — *each AC-NN has a stable ID + back-linked test_id*

Per `agent-durable-goal.json#spec_contract.acceptance_criteria_rule`: every PRD acceptance criterion carries a stable ID and is back-linked from the test that proves it. This is the load-bearing structure that lets autonomous agents verify "done" without human interpretation.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| AC-01 | (precondition) | (action) | (postcondition) | T-01 | (e.g., `crates/oya-<context>-kernel/tests/<name>.rs::test_<func>`) |

ID stability rule: ACs are **append-only**. Renumber forbidden — retire by adding `status: superseded_by: AC-NN` rather than re-using a slot. Per `agent-durable-goal.json#OP-11` no-stubs.

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|

### 3.2 Out-of-scope (anti-scope)

Bulleted list. Anti-scope is binding; promotion to in-scope requires a council decision.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

Which bounded context this product owns (per [DESIGN.md §1](../DESIGN.md)). Cite the flat-crates target prefix (e.g. `crates/foundry-*`).

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

## 9b. Verification commands (required) — *one runnable check per metric*

Per `agent-durable-goal.json#score_cards.design_principle`: deterministic checks; LLM judgment forbidden in pass/fail. Every success metric above has a runnable command an agent (or CI lane) can invoke to verify the metric.

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| (metric name) | `oya gate validate <lane>` OR `cargo nextest -p <crate> -- <test>` OR `jq '...' <evidence-path>` | `exit 0` / `count == 0` / `>= threshold` | `governance-<lane>` |

Anti-pattern (forbidden): `TBD verification` or `manual check` — every metric MUST have a runnable command. If a metric can't be verified mechanically, it doesn't belong in this table (move to §11 open questions).

## 10. Risks + mitigations

Per-product risk register slice. Mirror to [`RISK-REGISTER.md`](../RISK-REGISTER.md).

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|

## 11. Open questions

Council-pending items.

## 11b. Competitive analysis (required) — *who we beat and on what*

Per-competitor table. NOT vague positioning — concrete + measurable differentiators.

| Competitor | What they do well | Where we beat them | Measurable target |
|---|---|---|---|

Per `agent-durable-goal.json#identity.quality_bar`: Stripe + Palantir + Linear. Per `feedback_quality_performance_scalability_bar`: hyperscaler-grade. NO "we're better in spirit" framing. Every row must cite a measurable target (latency p99, throughput, error budget, time-to-feature, tenant-isolation guarantee, etc.).

## 11c. Best practices (required) — *production-grade patterns this product enforces*

Bulleted list. Each item: a positive pattern + why it's load-bearing + how it's mechanically enforced (which CI lane / fitness check / kernel-level guard).

Per `agent-durable-goal.json#OP-11`: NO "good enough for `MVP`" framing. Every practice is hyperscaler-target.

## 11d. Patterns + anti-patterns (required) — *what to do, what to never do*

### 11d.1 Sanctioned patterns

| Pattern | When to use | Reference |
|---|---|---|

### 11d.2 Anti-patterns (forbidden)

| Anti-pattern | Why forbidden | Detection lane |
|---|---|---|

Per `agent-durable-goal.json#tdd_contract.test_first_anti_patterns_forbidden` for test-related; per this product for product-specific.

## 11f. User experience (required for user-facing surfaces) — *real end-user needs, not toy demos*

Per user directive 2026-05-16 — "think about the end user experience as well; think what their actual needs will look like." For any product with a user-facing surface (UI, CLI, API consumed by an external dev) declare:

| Field | Content |
|---|---|
| `ux_personas_ref` | Pointer to user-journey spec at `/specs/microservices/<id>/ux.json` |
| `accessibility_coverage` | Minimum WCAG 2.2 AA; document any extension to AAA |
| `responsive_breakpoints` | Closed enum: mobile-portrait / mobile-landscape / tablet / desktop / wide-desktop |
| `internationalization_scope` | Closed enum: en-only / multi-language-fixed-set / locale-aware-dynamic |
| `design_system_components_used` | Pointer to `/specs/design-system/<component>.json` rows used |
| `journey_critical_paths` | Per-persona table of top-3 happy-path journeys + time-to-success target |
| `error_state_coverage` | How errors surface: inline / toast / modal / page; per-error class |
| `offline_behavior` | Behavior when network breaks; required for editor-class products per `/specs/microservices/workflow-studio.json#AC-03` |
| `keyboard_navigation_coverage_pct` | Minimum 100% for power-user products |
| `loading_state_coverage` | Skeleton / spinner / progressive-render policy per surface |

Per `agent-skills:frontend-ui-engineering`: production-quality UIs, not AI-generated slop. Anti-patterns: spinner-only-loading-states; alert-modal-as-error-surface; missing-keyboard-shortcuts on power-user surfaces; inaccessible color contrast.

## 11g. Frontend components (required for products with rendered UI)

| Component | Source | Variants | Tested-at-breakpoint |
|---|---|---|---|

Components must be sourced from `/specs/design-system/` catalog. Custom one-off components require a design-system-promotion ADR before merge per `agent-durable-goal.json#OP-11` no-stubs (one-off custom = stub of "should have promoted to design-system but didn't").

## 11e. Goals (required) — *production-quality, hyperscaler-grade targets, not `MVP`*

Per user directive 2026-05-16 captured in `agent-durable-goal.json#OP-11.user_directive_verbatim`: "We are not building `MVP`, demo, or sample. We are building full hyperscaler production platform and ecosystem. That is scalable, secure, performant, and efficient."

| Dimension | Target | Verification |
|---|---|---|
| Scalability | (concrete number — tenants, RPS, GB/day) | (benchmark lane / load-test artifact) |
| Security | (e.g., per-tenant isolation by row-level security + signed audit-chain + Cedar policy at every edge) | (governance-* lane) |
| Performance | (latency p50/p99, throughput) | (perf-budget lane in CI) |
| Efficiency | (cost per tenant per month, per call, per GB) | (FinOps unit-economics dashboard) |
| Reliability | (SLO targets — availability, MTTR, error budget) | (canary-observability + post-deploy lanes per agent-durable-goal.json#pipeline.lane_categories.post_deploy_observability) |

NO "`MVP-shape`" targets. NO "we'll improve in v2" caveats. If a target can't be met in initial GA, the product doesn't ship to GA.

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

`governance-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` (or is a planned target on the roadmap)
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
