---
id: ADR-0015
status: Superseded
doc_status: published
superseded_by: [ADR-0131, ADR-0512]
supersession_note: "ADR-0131 supersedes the docs-vs-crates top-level split; ADR-0512 supersedes the flat-crates location + forbidden-vocab context enum (D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-3/FC-1)."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0015: Architectural flattening target — flat-crates `crates/oya-<context>-<role>[-<capability>]/`, role taxonomy (kernel / domain / app / api / worker / adapter / runtime), dep-direction kernel←domain←app←api/worker/adapter←runtime, boundary validator, migration path from legacy modules / services / platform tree

> **Status:** Accepted
> **Supersedes:** -
> **Superseded-by:** [ADR-0131](ADR-0131-per-microservice-flat-layout.md) (partial — only the docs-vs-crates top-level split; BC and layer rules remain in force)
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0006, ADR-0011, ADR-0014, ADR-0017, ADR-0105, ADR-0106

> **ADR-0106 amendment note (2026-05-16):** this older title/body uses the pre-ADR-0106 terms `application`/`runtime`. Current canonical roles are defined by ADR-0105 + ADR-0106: `usecase` replaces `application`, `app` is the deployable/composition-root layer, and `app -> app` remains forbidden. Shared orchestration belongs in `usecase`.

---

## Context

The legacy repo tree under `modules/` `services/` `platform/` evolved over multiple eras and accumulated three problems that compound as the flat-catalog cohesion claim takes shape. First, the same bounded context lives in multiple places (a healthcare entity in `modules/healthcare-*`, a healthcare service in `services/healthcare-*`, a healthcare adapter in `platform/health-*`); cross-microservice review needs to chase the same domain across three trees. Second, the role of each crate (entity vs domain vs adapter vs runtime) is implicit; the boundary validator must infer dep direction from naming. Third, the migration target — flat `crates/oya-<context>-<role>[-<capability>]/` per legacy flat-crates ancestry (now superseded by this ADR per [`ADR-LEGACY-REGRESSION-MAPPING.md`](../ADR-LEGACY-REGRESSION-MAPPING.md)) — needs an authoritative ADR in this Foundation pack so every other ADR in 0001-0019 cites a single source for crate naming + dep-direction enforcement.

The cohesion thesis (ADR-0001) makes the flatten more important, not less, because every cross-microservice contract row (ADR-0011) cites a `source_of_truth` crate, and every license + build-vs-buy decision (ADR-0013, ADR-0014) cites a per-crate role. Without an authoritative crate-naming + role-taxonomy ADR, the registry rows become ambiguous.

---

## Decision

We adopt **flat-crates** as the canonical target, **`oya-<context>-<role>[-<capability>]`** as the naming convention, **a closed role taxonomy** with explicit dep direction, **a boundary validator** that hard-fails forbidden edges, and **a forward-only migration posture** from the legacy tree.

**Live baseline as of 2026-05-11:** the Cargo workspace has 64 members, every workspace member lives under `crates/oya-*`, every workspace member has a `registry/catalog/<crate>.yaml` record, and top-level `modules/`, `services/`, `platform/`, and `tools/` are absent. Remaining ADR-0015 work is additive split/extraction work inside the flat shape; it MUST NOT recreate the retired legacy roots.

### Crate naming

```
crates/oya-<context>-<role>[-<capability>]/
```

- `<context>` = one of the bounded-context names: `platform`, `saas`, `workspace`, `vertical-<industry>`, `foundry`, `cloud`, `search`, `ads`, `analytics`, `tooling`, `pack-<pack-id>`, `foundation`.
- `<role>` ∈ closed taxonomy below.
- `<capability>` (optional) = the specific capability inside the role (e.g. `oya-tenancy-residency-kernel` if the kernel splits by capability).

Examples: `oya-tenancy-kernel`, `oya-intelligence-runtime-policy-app`, `oya-cloud-iam-api`, `oya-search-index-vector-adapter-pgvector`, `oya-pack-kr-tax-app`.

### Role taxonomy (closed)

| Role | Reads as | Allowed I/O | Async / sync |
|---|---|---|---|
| `kernel` | Pure domain entities, value objects, invariants | None | Sync only |
| `domain` | Use cases + sealed-port traits | None directly; ports declared as traits | Sync; `async` allowed when port impls are async |
| `app` | Orchestration, sagas, commands, projections | Calls into `domain` + `adapter` | Async OK |
| `api` | Inbound HTTP/gRPC servers (REST, GraphQL, WebSocket) | Network ingress only | Async |
| `worker` | Inbound Kafka/queue consumers | Broker ingress only | Async |
| `adapter` | Adapter implementations of `domain` ports (DB, HTTP client, KMS, file, vendor SDK) | Outbound I/O | Async OK |
| `runtime` | Composition root (binaries, deployable, wiring) | Top-level main; cargo target type `bin` | n/a |

### Dep direction (forbidden-edge graph)

Allowed edges:

```
kernel ← domain ← app ← {api, worker, adapter} ← runtime
```

Reverse edges are CI errors. In particular:

- `kernel` may NOT depend on `domain`, `app`, `api`, `worker`, `adapter`, or `runtime`.
- `domain` may NOT depend on `app`, `api`, `worker`, `adapter`, or `runtime`.
- `app` may NOT depend on `api`, `worker`, or `runtime`.
- `api` and `worker` may NOT depend on each other (they are siblings; `runtime` composes both).
- `adapter` may import `domain` ports + `kernel` types; not other `adapter` crates from the same context.
- `runtime` may import everything.

### Boundary validator

`scripts/check-architecture-boundaries.sh` (descended from legacy boundary-validator ancestry; here-canonical) walks `cargo metadata` and:

1. Maps every crate to its `<context>` and `<role>` from the catalog (`registry/catalog/<crate>.yaml`).
2. Verifies every `[dependencies]` edge is in the allowed graph.
3. Hard-fails any forbidden edge.
4. Emits per-PR evidence through `oya-governance-flat-crates`, `oya-governance-catalog-records`, and `oya-governance-cargo-prefix`.

Catalog declaration:

```yaml
# registry/catalog/<crate>.yaml
context: platform
role: kernel
capability: tenant
plane: control                # ADR-0004
slo: preview-control-plane
data_classes_owned: [INTERNAL_ONLY, PII_IDENTIFYING]   # ADR-0008
operational_classes_owned: []
```

### Per-axis kernel size targets (informational)

Each axis ships kernels sized for its bounded contexts (per DESIGN §4):

| Axis | Kernel crate count | Example kernels |
|---|---|---|
| SaaS | 6-10 | `oya-tenancy-kernel`, `oya-identity-kernel` |
| Workspace | 4-8 | `oya-workspace-doc-kernel`, `oya-workspace-mail-kernel` |
| Vertical | 1-3 per vertical | `oya-vertical-healthcare-kernel`, `oya-vertical-fintech-kernel` |
| Foundry (runtime) | 4-6 | `oya-intelligence-capability-kernel`, `oya-intelligence-evidence-kernel` |
| Foundry (engineering platform) | 3-5 | `oya-intelligence-catalog-kernel`, `oya-governance-gate-kernel` |
| Cloud | 5-8 | `oya-cloud-resource-kernel`, `oya-cloud-iam-kernel` |
| Search | 3-5 | `oya-search-document-kernel`, `oya-search-index-kernel` |
| Ads | 4-6 | `oya-ads-campaign-kernel`, `oya-ads-auction-kernel` |

### Migration path from legacy tree

The original flatten inventory proceeded in phases:

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7
kernel    contracts  domain    app       api/worker/adapter  runtime  sweep
```

Per phase, while any legacy source roots exist:

1. Per-crate move PRs land one bounded context at a time; each PR moves the crate to flat target and updates internal imports.
2. The root `Cargo.toml [workspace.members]` is the serialization point — only one PR at a time may modify it (per the merge-queue invariant in ADR-0011's protocol).
3. Brand-rename precursor PRs (`oyatie-*` → `oya-*` per ADR-0017) sweep before each phase tier so internal renames do not collide with external API changes.
4. Catalog records under `registry/catalog/` are written/migrated as part of each move PR.
5. Per-deployable Helm/IaC moves to `deploy/` after the source service's runtime crate moves (Phase 6).
6. Any catalog-directory relocation from `registry/catalog/` to `catalog/` requires a new catalog protocol update; `registry/catalog/` remains the live source of truth until that lands.

This section is retained as migration doctrine and historical sequencing context. It is not a claim that the live workspace still contains the legacy roots.

### Boundary

- Applies to: every crate in the workspace.
- Does not apply to: dev-only test fixtures outside the workspace; experimental research crates explicitly outside the workspace tree (with their own catalog override).

---

## Consequences

### Positive

- Crate naming becomes mechanically unique — `<context>` × `<role>` × `<capability>` is the address.
- Boundary validator turns clean-architecture boundaries into compile-time guarantees, not review folklore.
- ADR-0011 contract registry rows can cite `source_of_truth` crates with confidence; ADR-0014 build-vs-buy matrix can apply per-role rules cleanly.
- Per-phase migration from the legacy tree is bounded and sequenced.

### Negative

- Additive split work can still be large (the legacy flat-crates plan recorded 89 move PRs and 91 target crates); per-phase coordination is real ops work.
- New contributors face a learning curve on the closed role taxonomy.
- Workspace `members =` serialization is a real bottleneck during heavy phases; mitigation: merge queue + per-PR auto-rebase + nightly affected-rebuild on `main`.

### Operational

- On-call: not applicable (architectural migration).
- Runbooks: `runbooks/flat-crates-move-pr.md`, `runbooks/per-context-flatten-phase.md`, `runbooks/workspace-members-merge-queue.md`.
- CI: `oya-governance-flat-crates` (path + legacy-root + role-boundary validator), `oya-governance-catalog-records` (every workspace member has a catalog record), and `oya-governance-cargo-prefix` (naming convention).
- Per-phase audit: every phase ends with a council review of crates moved + catalog records emitted + downstream consumers verified green.

---

## Alternatives considered

### Alternative A — Keep `modules/` `services/` `platform/` tree

- **Pros:** zero migration cost.
- **Cons:** drift across three trees; cross-microservice review impossible; ADR-0011 cannot point at a single source of truth.
- **Rejected because:** cohesion (ADR-0001).

### Alternative B — Per-axis monorepo split (one repo per axis)

- **Pros:** per-microservice autonomy.
- **Cons:** cross-microservice contract changes (ADR-0011) become cross-repo coordination; substrate kernels (ADR-0002, 0003, 0006, 0007) need to be co-developed in lockstep.
- **Rejected because:** cohesion + workspace-stays-green invariant.

### Alternative C — Flat crates without role taxonomy (just `oya-<feature>`)

- **Pros:** simpler naming.
- **Cons:** boundary validator cannot verify dep direction without role; clean-architecture invariants become aspirational.
- **Rejected because:** ADR-0011 + ADR-0014 + ADR-0007 all rely on role-typed crate identity.

---

## Open questions

1. **Q1.** Sub-context naming inside an axis (e.g. Foundry's runtime vs engineering platform — both `oya-foundry-*` but distinct sub-contexts) — separator convention? Default: `<role>-<sub-context>` so `oya-intelligence-runtime-policy-app` reads cleanly. → owner: `council-architecture`.
2. **Q2.** Cargo workspace splitting after the workspace grows past the historical 91-crate split inventory — stay one repo or shard? Default: stay one repo with `cargo build --workspace --target` sharding; live count was 64 on 2026-05-11; revisit at 200+ crates. (TOOLCHAIN §9 Q5.) → owner: `council-architecture`.
3. **Q3.** Per-pack crate naming under `oya-pack-<pack-id>-*` — inherit the same role taxonomy? Default: yes; packs are bounded contexts with kernel/domain/adapter roles. → ADR-0010.
4. **Q4.** `tooling` context for repoctl + CLI — does it host all CLI personas under one context or one per persona? Default: one context with capability segmentation (`oya-tooling-cli-dev-app`, `oya-tooling-cli-admin-app`, etc.). → ROADMAP §8 Q16.

---

## References

- `docs/DESIGN.md` §4 (per-microservice bounded contexts, four-layer hexagonal stack), §8 (architectural flattening — phase order)
- `docs/PRD.md` §6 constraint 4 (architectural flattening), constraint 5 (clean-architecture boundaries inside each crate)
- `docs/TOOLCHAIN.md` §3 (per-stack default), §9 Q5 (cargo workspace splitting)
- ADR-0001 (cohesion — substrate kernels are flat crates), ADR-0002 (Tenant + Identity kernel — flat target), ADR-0006 (Ontology — flat kernel), ADR-0011 (contract registry cites flat crates), ADR-0014 (build-vs-buy matrix per role), ADR-0017 (brand rename + Cargo prefix)
