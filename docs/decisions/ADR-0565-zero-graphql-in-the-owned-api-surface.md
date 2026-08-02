---
id: ADR-0565
title: "Zero GraphQL in the owned API surface — the canonical surface set is REST + gRPC + async + realtime, and GraphQL returns only by an ADR that explicitly reverses this"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-06-21
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0056, ADR-0105, ADR-0253, ADR-0258]
amended_by: [ADR-0632]
# NOTE: the Phase-00 product spec under docs/products/<de-brand-target>/ also named GraphQL in its
# transport-parity scope; its GraphQL retraction is deferred to the de-brand of that directory
# (it cannot be cited here without re-introducing the brand-residue token the brand-residue gate forbids).
depends_on: [ADR-0358, ADR-0094]
related: [ADR-0051, ADR-0066, ADR-0091, ADR-0145, ADR-0150, ADR-0157, ADR-0193, ADR-0253, ADR-0258, ADR-0342, ADR-0512, ADR-0532, ADR-0536]
related_specs:
  - /specs/api-contract-ssot-canonical.json
  - /specs/schema-registry-canonical.json
  - /specs/planning-closure-contract.json
  - /specs/masterplan.json
  - /specs/microservices/manifest-schema.json
milestone: W0
---
# ADR-0565: Zero GraphQL in the owned API surface

## Status

**Accepted — 2026-08-01 (founder-ratified; door: one-way).** The founder confirmed a zero-GraphQL
owned API surface after the north-star interview. The public/internal exposure split and the exact
non-GraphQL product contract are specified separately; this decision binds only the removal and
fail-closed reintroduction rule. Reintroduction requires a later Accepted ADR that explicitly
reverses this decision.

## ADR-0632 product-protocol reconciliation

The non-GraphQL surface set is exposure-aware: public contracts are HTTPS REST documented by OpenAPI 3.2.0, signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, and bidirectional WebSocket sessions. Public gRPC, gRPC-Web, and Connect are forbidden. gRPC and gRPC streaming are internal-only gRPC/proto3 over HTTP/2 and do not enter public SDK or gateway compatibility.

## Context

A repo-wide investigation found exactly two GraphQL artifacts in the owned stack, both of which were
liabilities rather than load-bearing surfaces:

1. The intelligence HUSK — `oya/intelligence/crates/oya-intelligence-api-graphql-{kernel,adapter}`.
   Hand-authored stub types (`GraphqlField`/`GraphqlType`/`GraphqlSchema`/`GraphqlRequest`/
   `GraphqlResponse`) over `BTreeMap<String, String>` payloads. No GraphQL library, no schema, no
   resolver, no parser — the "transport" returned empty maps. Zero consumers (no BUCK/Cargo dep edge
   pointed at it). It also VIOLATED `specs/api-contract-ssot-canonical.json`, which mandated that
   any GraphQL surface be a GENERATED projection of the shared Rust-native contract source, never a
   separately hand-maintained schema — the husk was exactly the hand-maintained schema the SSOT
   forbade.

2. The analytics SDL — `oya/analytics/contracts/graphql-v1.sdl`. A real, well-formed generated Relay
   schema (Tenant-scoped `workflowExecutionDashboard` / `billingRollup` / `auditLog` Connection
   queries, cursor pagination per ADR-0150), status `planned`, with NO resolver and NO server. It was
   registered in `oya/analytics/catalog/contracts.json`, `oya/analytics/catalog/oya-analytics-api.json`,
   the `data/ports/analytics-api` doc surface, and `registry/catalog/data-analytics-api.yaml`.

The SSOT, the masterplan `api_contracts` block, the planning-closure contract, the schema-registry
canonical spec, the microservice manifest schema, and the platform-architecture spec all carried
GraphQL as a first-class (generated-only) member of the canonical API surface set. ADR-0258 listed
GraphQL among the public surfaces; ADR-0253 D-14 named GraphQL Federation v2 via a BFF tier.

The analytics BFF had a real, recognizable benefit: an OLAP read-aggregation surface where a single
typed graph query composes per-tenant dashboard, billing-rollup, and audit-log reads with Relay
cursor pagination. That is the textbook good fit for GraphQL. But it was planned-and-unbuilt, and a
second hand-edited-vs-generated schema family is exactly the drift surface the SSOT exists to prevent.

## Decision

**The owned stack carries NO GraphQL surface.** Not a REST-named husk, not a generated
Backend-for-Frontend (BFF). The canonical owned API surface set is:

- **REST** — OpenAPI 3.2.0
- **internal-only gRPC** — proto3 over HTTP/2
- **event / async** — AsyncAPI 3.1.0
- **realtime** — public SSE (one-way server push) and WebSocket (bidirectional); gRPC streaming is internal-only

All of these are generated or validated from the one shared Rust-native contract source of truth
(ADR-0094 contract-first, the api-contract-ssot model). There is NO generated-BFF carve-out: the
zero-GraphQL rule admits no "but GraphQL is fine when it is generated" exception. Analytics
dashboards aggregate via REST/gRPC composition when they are actually built.

**GraphQL is admissible ONLY via a future ADR that explicitly reverses this one.** Absent such an
ADR, any GraphQL library dependency, any `.graphql`/SDL artifact, and any GraphQL resolver are
forbidden in the owned stack.

### Enforcement (recorded by THIS PR)

This PR enforces the decision by deletion and by de-blessing the vocabulary, not by flag. The
enforcement is partial-by-design — it closes the crate-NAME axis now and stages the artifact-EXTENSION
axis to issue #772 (see "Follow-up and staging" below); it does NOT claim the full reintroduction
surface is sealed in this merge:

- DELETED the husk crates `oya/intelligence/crates/oya-intelligence-api-graphql-kernel/` and
  `oya/intelligence/crates/oya-intelligence-api-graphql-adapter/` (whole directories, including their
  BUCK targets and the inert `**/*.graphql` glob those targets carried).
- DELETED the analytics contract `oya/analytics/contracts/graphql-v1.sdl` and every registration of
  it (the analytics catalog `contracts.json` graphql entry, the `oya-analytics-api.json` contract +
  justification, the `data/ports/analytics-api` doc surface, `registry/catalog/data-analytics-api.yaml`,
  and the `registry/stores/registry-store.json` mirror).
- DELETED the husk catalog records `registry/catalog/oya-intelligence-api-graphql-{kernel,adapter}.yaml`
  and the husk adapter's entry in `registry/dependency-rationales.json`.
- UPDATED the SSOT (`specs/api-contract-ssot-canonical.json`) and the dependent canonical specs
  (masterplan, planning-closure, schema-registry, manifest-schema, platform-architecture,
  cloud-strangler-migration-target, root-hub-pointers, oyatie-doctrine, tasks/plan + tasks/todo) to
  declare the four-surface set and ZERO GraphQL.
- DE-BLESSED `graphql` from the role/layer vocabulary SSOT: removed it from `allowed_roles` in
  `oya-ci.toml`, from `ALLOWED_ROLES` in
  `libs/oya-governance-predictable-naming-kernel/src/lib.rs` (13 → 12, with the parity test and the
  bnf-layer-suffix gate doc updated in lockstep), from the `layers` enum in
  `specs/microservices/manifest-schema.json`, from `canonical_enum` in `specs/crate-naming-audit.json`,
  and from the only live manifest that still declared it (`oya/workplace-integration/manifest.json`).
  Because the EXISTING predictable-naming gate reads `allowed_roles` as policy-as-data, this single
  removal makes that gate fail-CLOSED against any future `*-graphql` crate as of this merge.
- REMOVED the live `graphql` recognizers that mapped the role into a typed value: the
  `"graphql" => CatalogRole::Api` arm in
  `intelligence/core/catalog-domain/src/lib.rs`, the `"graphql"` valid
  contract-extension match in `marketplace/facade/dev-cli/src/api_contract_registry.rs`, and the
  `ArchitectureLayer::Graphql` variant (plus its array/slug/scaffold-count consumers) in
  `data/facade/warehouse-tenant-olap-service/src/domain/mod.rs`.

### Follow-up and staging (what is DONE here vs. tracked separately)

The enforcement above is staged. Three items are explicitly NOT in this PR:

- **Vocabulary de-blessing is DONE in this PR.** The role/layer enum SSOT no longer blesses `graphql`,
  so the existing predictable-naming gate is fail-CLOSED against any new `*-graphql` crate as of this
  merge. This also covers the tooling-tier metadata-layer check (`tools/oya-xtask-metadata-augment-app`
  `LAYER_VALUES`) and the `oya/workplace-integration` manifest `layer_enum` justification string. No
  follow-up is required for the crate-naming axis.
- **The active `*.graphql`/SDL file-reintroduction gate is tracked as issue #772 (branch
  `agent/no-graphql-gate`).** Interim gap: a `.graphql` file added to an EXISTING crate is not yet
  auto-blocked until #772 lands — the de-blessing above only closes the crate-NAME axis, not the
  artifact-EXTENSION axis.
- **The regen-reintroduction risk behind the inert `**/*.graphql` BUCK glob is CLOSED.** This ADR
  originally deferred the glob sweep because the Python BUCK generator would have reintroduced the
  glob on the next full-tree regen, making any manual sweep futile. That generator
  (`scripts/gen_first_party_buck.py`) is now deleted: all 868 workspace members already carry a BUCK
  file, so its only non-no-op mode was `--force`, which clobbers hand-edited BUCK. With no generator
  there is no regen that can reintroduce the glob, and the sweep of the committed BUCK files is a
  plain mechanical edit whenever it is worth doing. The committed files still carry the inert glob;
  it is dead weight (the source files it would match were deleted), not a live GraphQL surface.

### Enforcement gate (LANDED — the artifact-EXTENSION axis #772 closes here)

The active reintroduction gate this ADR deferred to issue #772 (branch `agent/no-graphql-gate`) is now
LANDED. It is a born-blocking cloud-ci gate (enforcement-layering doctrine: the drop above is the
construction, this gate is the recurrence backstop) that fails CLOSED if the CANDIDATE tree
reintroduces — WITHOUT the artifact citing an ALLOWLISTED + VALIDATED authorizing (reversing) ADR id —
ANY of: a GraphQL execution/parse library in ANY `Cargo.toml` in the tree (members AND non-members,
resolving `[workspace.dependencies]` renames and `{ workspace = true }` inheritance; the forbidden set
is policy DATA: the async-graphql family, juniper, graphql-parser, graphql-client, cynic, apollo-*, …);
a forbidden GraphQL crate in the resolved `Cargo.lock` graph (the transitive-reintroduction catch); or
a `.graphql`/`.graphqls`/`.gql`/`.gqls`/`.sdl` GraphQL schema file. It evaluates the candidate tree
directly (NOT a frozen merge-base baseline), so the verdict is identical at PR-tier and push-tier
(avoiding the gate-baseline PR/push asymmetry false-green); the frozen baseline is EMPTY (the tree is
GraphQL-free post-drop), so any new GraphQL artifact fails closed on arrival. The ADR escape-hatch is
NOT a bare-token match: an artifact launders ONLY by citing an `ADR-NNNN` that is BOTH (1) enumerated in
the gate policy `authorizing_adrs` allowlist (EMPTY today — nothing authorizes GraphQL, so a fabricated
or typo id cannot launder) AND (2) validated against the real `docs/decisions` tree (an Accepted ADR
that reverses ADR-0565). A file can never self-launder by naming the rule it would be violating
(ADR-0565); reintroducing GraphQL requires first Accepting a reversing ADR and adding its id to the
allowlist in the same reviewed change. KNOWN LIMITATION: an inline-SDL string literal / derive macro
with no schema file is not caught by the schema-file walk, but any real GraphQL server needs a GraphQL
library, which the manifest legs + the `Cargo.lock` leg DO catch. This also closes the
artifact-EXTENSION axis the "Follow-up and staging" item above tracked as an interim gap.

This ADR OWNS and JUSTIFIES the gate crate; its verbatim tracked paths are:
`ci/facade/graphql-usage-policy/Cargo.toml`,
`ci/facade/graphql-usage-policy/BUCK`,
`ci/facade/graphql-usage-policy/OWNERS`,
`ci/facade/graphql-usage-policy/no-graphql-without-adr-policy.json`,
`ci/facade/graphql-usage-policy/src/lib.rs`,
`ci/facade/graphql-usage-policy/src/main.rs`, and
`ci/facade/graphql-usage-policy/tests/no_graphql_without_adr.rs`.

## Rationale

- **The husk had zero benefit and violated the generated-SSOT rule.** A hand-authored stub with no
  GraphQL library, no resolver, and zero consumers is pure carrying cost: it inflates the crate count,
  presents a fake "transport parity" claim, and is the precise hand-maintained-schema anti-pattern the
  SSOT forbids. Deleting it is strictly subtractive.
- **The analytics BFF benefit is real, but founder chose simplicity over a planned-but-unbuilt
  surface.** GraphQL's OLAP read-aggregation ergonomics are genuine; the founder weighed that against
  a fifth contract surface family, a second generator + drift-gate leg, a GraphQL runtime in the
  owned stack, and a parallel schema-evolution model — and chose zero-GraphQL maintainability. REST +
  gRPC composition covers the analytics read path without a new surface.
- **Cohesive-owned-substrate doctrine.** The owned stack is designed as a few cohesive substrates, not
  a maximal protocol menu. Every surface we keep is a surface we own end-to-end (generator, drift gate,
  runtime, versioning model). A surface we would only ever generate-but-not-build is not owned; it is
  aspirational debt.
- **No-good-for-now.** A planned-unbuilt GraphQL leg is a "good for now" placeholder that the doctrine
  rejects: it is either a real owned surface or it is deleted. It was deleted.

## Consequences

- The api-contract-ssot drift gate (when built) covers REST/gRPC/async only; the realtime bindings
  project the same typed payloads. No GraphQL emitter, validator, or resolver is in scope for any
  downstream ChangeSet unless a reversing ADR lands first.
- The intelligence reorg move-22 sub-batch (e) has two fewer crates to home — dropping the husk
  pre-empts that part of the move.
- ADR-0258 and ADR-0253 keep their history; their GraphQL-bearing surface lists are amended (not
  rewritten) by the `amends` frontmatter above. Their REST/gRPC/AsyncAPI legs are unaffected.
- Any future GraphQL proposal is a one-way ADR that must justify the surface against this decision and
  carry its own generator + drift-gate + runtime ownership plan.

## Alternatives considered

- **Keep GraphQL as a generated-only BFF (the prior SSOT position).** Rejected: the only candidate BFF
  (analytics) was unbuilt, and a generated-only carve-out still requires a GraphQL generator, runtime,
  and drift-gate leg the owned stack would have to own for one consumer. The carve-out reintroduces the
  whole surface's cost for a single planned use.
- **Keep the analytics SDL, drop only the husk.** Rejected: this leaves a dangling planned surface with
  no resolver and a registered contract that the api-contract-ssot drift gate would eventually have to
  either generate-and-diff (needs the generator) or special-case (a permanent exception). Zero-GraphQL
  removes the special case.
- **Soft-deprecate (mark husk + SDL deprecated, delete later).** Rejected per the
  automation-maximalism / no-good-for-now doctrine: a soft-deprecation is a flag, not a fix. The
  artifacts have zero consumers, so deletion is safe now and is the construction-over-reaction move.
