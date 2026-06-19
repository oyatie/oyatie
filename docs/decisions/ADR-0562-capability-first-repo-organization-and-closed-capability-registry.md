---
id: ADR-0562
title: "Capability-first repo organization + the closed capability registry — the ratified hyperscaler source-tree shape every reorg lane implements"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-06-14
door: one-way
owner: founder
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0245, ADR-0280, ADR-0512, ADR-0536, ADR-0550]
amends:
  - ADR-0536-hyperscaler-grounded-substrate-decision-matrix.md (its sixteen domains seed the closed capability registry; its enforcement gains face-direction + membership checks)
  - ADR-0512-canonical-monorepo-pattern.md (capability-first layout supersedes the cloud/oya/libs root assumption; libs/ dissolves into capability homes + base/; the kuberos-kernel nested-workspace carve-out)
  - ADR-0280-substrate-of-substrate-dependency-doctrine.md (DAG nodes use de-branded capability names; D-1 is the canonical substrate bootstrap ordering source)
related: [ADR-0131, ADR-0132, ADR-0510, ADR-0520, ADR-0532, ADR-0533, ADR-0537, ADR-0543, ADR-0547, ADR-0549, ADR-0555]
related_specs:
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
  - /specs/platform-architecture.json
milestone: W0
---

# ADR-0562: Capability-first repo organization + the closed capability registry

## Status

**Proposed — 2026-06-14 (founder-ratified shape authored for the founder Accept door; door: one-way).**

The reorganization shape was ratified by the founder on 2026-06-14 (cost-agnostic, four-axis
adversarial determination — hyperscaler-fidelity / dogfood-purity / maintainability / clean-arch —
with doubt-driven verification). This ADR is the durable record of that decision and the spec every
later reorg lane implements. It stays Proposed until the founder Accepts (the one-way door); it does
NOT move any crates. ADR-0328 remains the canonical sequence authority; the migration contract in
§10 is reference, not execution.

## Context

The repository is a single Rust monorepo (~700+ crates) under hyperscaler-grade discipline. Three
written layout authorities are in force and partially in tension:

- **ADR-0512** (Accepted) — the canonical monorepo pattern: one root Cargo workspace, crate =
  bounded context, service dirs as pure containers, the Buck2 graph as the parallelism/containment
  substrate. Its layout clause roots service code at `{oya,cloud}/<service>/` and shared code at
  `libs/<lib>/`.
- **ADR-0550** (Proposed) — the repository layout doctrine: `{oya,cloud}/<service>/` colocation +
  the kernel/adapter/app clean-architecture seams + the `libs/` charter (rule-of-two shared root).
- **ADR-0245** (Proposed) — substrate-vs-product layering: tier (substrate / product /
  service-cell) is a **PRD-frontmatter facet, NOT a directory split** (lines 153–161); ADR-0131
  already collapses the product-vs-substrate distinction at the directory level.

The tension the founder resolved: the current `{oya,cloud}/` split sorts the tree by **who runs or
sells** a system (cloud = platform/tenant substrate; oya = product/domain) rather than by **what the
system IS**. That is a runner/seller axis, not a capability axis. It scatters one capability across
both roots (identity lives at `cloud/cloud-iam` AND `oya/identity` AND `oya/oya-identity` AND
`oya/consent-graph` AND `oya/tenant-rbac` AND `oya/policy`), it has no hyperscaler source-tree
precedent (Google's tree is `//net //storage //compute`, not `//run //sell`), and it cannot encode
the founder's own ports-designed-for-owned-stack litmus ("would this trait change at cutover?") —
because the run-face and the sell-face of one engine end up in different roots instead of behind one
port seam. ADR-0536 then decided WHAT each substrate domain is (sixteen domains) but did not decide
the source-tree shape that gives each domain exactly one home.

Two shapes were taken to the four-axis adversarial determination:

- **Capability-first** — one top-level dir per system (the capability), faces (core/ports/adapters/
  facade) inside it. Scored **9 / 9 / 8 / 9**.
- **Tier-first** — top-level `substrate/`, `product/`, `service-cell/`. Scored **3 / 6 / 4 / 5,
  REJECTED**: it has ZERO hyperscaler source-tree precedent, it re-erects exactly the directory
  split ADR-0245 forbids (tier is a facet), and it cannot represent ADR-0245's three tiers as a
  clean tree (a single engine spans tiers via its faces, not via three directories).

Capability-first is therefore both the higher-scoring shape AND the shape already mandated by
ADR-0245's facet rule. This ADR ratifies it, closes the capability set so the shape is
mechanically safe (an open set degrades into a junk-drawer), and folds in the resolved forks.

## Decision

### §1 The shape

The top-level source tree is organized **by capability** (the primary axis), with five meta/floor
directories that are not capabilities:

```
kernel/        # rung 0: kuberos no_std kernel + sysroot (recursion floor; its own excluded workspace — the ADR-0512 carve-out, §8 Fork 2)
os/            # rung 1: cloud-os node OS (Talos-class)
base/          # Google //base: irreducible cross-capability primitives. ADMISSION-GATED (>=3 capability consumers AND strictly below all of them in the ADR-0280 DAG). NOT a util/ junk-drawer.
governance/    # meta, OFF the runtime ladder: ADRs, specs, policy-as-data, the capability registry, the dep-lint authority, the masterplan
build/         # meta, OFF the runtime ladder: buck2 prelude, toolchains, reindeer, third_party vendoring, CI engines, AND the generated sell-catalog (SKU/pricing) VIEW (build output; owns zero crates)
<capability>/  # THE PRIMARY AXIS — one dir per registered system; path = namespace = buck2 label root
  core/        #   the engine we RUN (substrate face)
  ports/       #   capability traits (the stable seam)
  adapters/    #   transient-infra impls (adapter-aws/-oci/-capi) — vanish at owned-stack cutover
  facade/      #   the multi-tenant surface we SELL (product face) — depends on core/ ONLY through ports
app/<product>/ # composition ring: deployable surfaces wiring 2+ capabilities for a tenant (SaaS verticals + the web shell)
```

**Hyperscaler precedent.** Google's source tree is capability-rooted (`//base`, `//net`,
`//storage`, `//compute`) — a system's name is its path. Meta's fbcode is product/domain-rooted at
the top with shared infra below. Microsoft's Azure SDK is `sdk/<service>/`. AWS is two-pizza
team-per-service with a service as the unit of ownership and naming (the 2002 Bezos API mandate
makes the service the boundary). All four root the tree by WHAT a system is, never by who runs or
sells it. Tier-first (`substrate/ product/ service-cell/`) has no such precedent and was rejected.

### §2 The closed capability registry

The canonical capability set is declared as data at **`specs/capability-registry.json`** (the
eventual home is `governance/capability-registry.json` after the reorg; held at `specs/` until the
`governance/` top-level dir exists, because creating a new top-level dir now would itself trip the
§6 membership lint). The registry is a **closed** enum (like ADR-0280's `tier_subtype` enum and
ADR-0245's tier facet): a crate must map to one registered capability or the membership lint fails.

The set is **coarse / Conway-aligned (founder ruling): one capability per two-pizza ownership
boundary.** It is seeded from ADR-0536's sixteen substrate domains (identity, authorization, cells,
console shell, control plane, observability, delivery fabric, KMS, network/DNS, data, storage/CAS,
compute, messaging, metering/billing, gateway/SSOT, audit) reconciled against the verified current
tree (`cloud/` + `oya/`). The mandated coarse collapses are encoded in the registry:

- **`iam/`** absorbs identity + oya-identity + consent(-graph) + tenant-rbac + policy + the embedded
  Cedar PDP as sub-modules (the cloud IdP substrate is `core/`; the product-shared identity that
  CONSUMES it is `facade/`).
- **`ci/`** absorbs cloud-ci + ci-controller + ci-tide + ci-webhook-gateway.
- **`compute/`** is ONE engine (vm + k8s-on-compute + functions) with facade sub-surfaces.
- **`k8s/`** is the owned control-plane (`core/`) + managed-k8s (`facade/`); the four
  `managed-k8s-*` dirs become facade sub-modules.
- **`secrets/`** is KMS + secrets sharing the crypto-root recursion break.
- **`billing/`** absorbs metering + billing + tax + finops/cost; **`marketplace/`** absorbs
  marketplace + plugin/app-store + SKU catalog data.

**Cross-cutting sold-ness (billing, tenancy, marketplace) are FIRST-CLASS capabilities, NOT a
`product/` junk-drawer.** The full set + one-line charter per capability + the absorbed current dirs
+ the seed domains live in `specs/capability-registry.json`. The capabilities at v1.0.0 are: `cell`,
`iam`, `tenancy`, `secrets`, `audit`, `observability`, `data`, `storage`, `compute`, `k8s`,
`network`, `gateway`, `messaging`, `intelligence`, `workflow`, `ci`, `iac`, `billing`,
`marketplace`, `console`, `compliance`, `comms`, `flags`.

### §3 The deterministic placement rule (first match wins)

For any crate or artifact, apply in order; the first match decides the home:

1. the kuberos kernel → `kernel/`; the node OS → `os/`;
2. ADRs / specs / policy-as-data / the capability registry / the dep-lint authority → `governance/`;
3. buck2 prelude / toolchains / third_party / CI engines / the generated catalog view → `build/`;
4. a primitive depended-on by **>=3 capabilities AND strictly below all of them in the ADR-0280
   DAG** → `base/` (admission-gated; a util consumed by one or two capabilities does NOT qualify —
   it lives in those capabilities or in `base/` only when the third consumer appears);
5. a deployable surface composing **2+ capabilities** for a tenant → `app/<product>/`;
6. else **exactly one REGISTERED capability**, chosen by **WHAT IT IS** (the system it implements),
   **NEVER by who runs or sells it** — within that capability the **FACE** selects the sub-fold:
   engine → `core/`, traits → `ports/`, transient-infra → `adapters/`, sold surface → `facade/`.

**Tie-breaker:** when more than one capability could plausibly claim a crate, it belongs to the
capability whose public port sits at the **lowest ADR-0280 DAG node** the crate participates in.

### §4 The non-negotiable face rule

**No crate is ever both engine and facade.** `core/` (run) and `facade/` (sell, via ports) are
mutually exclusive. *"If you cannot say which side a crate is on, it is mis-factored and must be
split."* Tier (substrate / product / service-cell), `tier_subtype`, and `bootstrap_tier` are
**manifest facets (`face:`, `tier_subtype:`, `bootstrap_tier:`), NEVER path segments** — this is
ADR-0245's rule, enforced. The face is declared in the crate manifest and mirrored by the sub-fold;
the lint asserts they agree.

### §5 Layering is a dependency-graph property, not a path property

Layering is enforced by the **ADR-0280 DAG** + a **dep-lint over BOTH the cargo and buck surfaces**
(acyclicity + direction). `kernel/`, `os/`, and `base/` are top-level dirs because they are the
**recursion floor** (rungs 0/1 and the strictly-below-all primitives), not because the tree encodes
layers as paths. The **sell-catalog** (SKUs / pricing / public API contracts) is a **GENERATED
projection** over crates carrying `face:facade` (product face), materialized as `build/` output — a
view, never a code home. A capability is "sold" because it has a `facade/` whose crates carry
`face:facade`, surfaced in the catalog view; it is never sold by living in a `product/` directory.

### §6 The membership lint (the anti-junk-drawer authority)

The lint is the precondition that makes a capability-first tree safe. It asserts:

- every crate maps to **exactly one** registered capability **and** carries a **valid face**
  (`core` / `ports` / `adapters` / `facade`) that agrees with its sub-fold;
- a **new top-level directory** that is neither a registered capability nor one of
  `kernel|os|base|governance|build|app` **FAILS** (the closed-set guarantee);
- the **`base/` admission rule** is enforced: a `base/` crate must have **>=3 capability consumers
  AND be strictly below all of them in the ADR-0280 DAG** (it is the structural backstop against
  `base/` becoming a util dumping ground);
- `app/<product>/` members compose **2+ capabilities** (a single-capability "app" is a mis-placed
  facade);
- the face rule (§4): no crate is both `core` and `facade`.

The lint runs over both cargo and buck target graphs and lands born-advisory, promoting
shadow → warn → enforce per the ADR-0536 D-7 ladder, gated on the ADR-0537 founder sign-off (§9).

### §7 The narrow split-out procedure (coarse is the default)

A capability is coarse by default. It **splits into siblings ONLY by an ADR amendment**, and only
when **both** an OWNERS boundary **and** a clean port seam exist. The split is mechanical
(re-home the sub-module's crates under a new registered sibling, update the registry, regenerate the
faces) and **non-per-PR**: a capability does not fragment opportunistically inside a feature PR.

### §8 Resolved forks (folded in as decided)

- **Fork 1 — substrate topology / bootstrap ordering.** The canonical source of the substrate
  bootstrap ordering is **ADR-0280 §D-1** (`/specs/substrate-dependency-dag.json` `bootstrap_order`).
  `platform-architecture.json`'s `substrate_dag_canonical_ordering` is **amended to match ADR-0280
  §D-1** (it is a derived mirror, not a second source of truth).
- **Fork 2 — nested workspace.** `cloud/cloud-kernel` (the kuberos kernel) becomes top-level
  **`kernel/`** as a **sanctioned nested/excluded Cargo workspace** — the `no_std` + custom-sysroot
  rung cannot share the one root workspace's std-targeted lockfile, exactly analogous to the
  release-image cargo exception. **ADR-0512 §6 is amended** to carve out the kuberos-kernel rung
  from its "no nested `[workspace]` tables" rule, and the `workspace-topology` gate **whitelists
  `kernel/`**. This is the only sanctioned nested workspace; the no-nested-workspace rule stands
  everywhere else.

### §9 The amendment set + the de-brand

This ADR **amends**:

- **ADR-0536** — its sixteen domains seed the closed registry; its enforcement gains the
  face-direction check (§4) and the membership check (§6).
- **ADR-0512** — the capability-first layout **supersedes** the `cloud/`/`oya/`/`libs/` root
  assumption; **`libs/` dissolves** into capability homes (single-capability shared code) +
  `base/` (>=3-consumer cross-capability primitives); plus the kernel carve-out (§8 Fork 2). The
  ADR-0512 invariants that survive unchanged: one root Cargo workspace (minus the `kernel/`
  carve-out), one-version policy, crate = bounded context, the Buck2 graph as the
  parallelism/containment substrate.
- **ADR-0280** — DAG nodes use **de-branded capability names** (the registry slugs).

**De-brand naming** (dropping `oya-`/`cloud-` brand prefixes from capability paths) is realized via
the existing **ADR-0532 / ADR-0533 profile mechanism** (canonical product names + the config-driven
public boundary / `profile`); the actual flip is a **later Phase-0 lane**, not this ADR.

**ADR-0245 is CITED as already-conformant** (tier is a facet, not a path) and **needs NO change**.
**ADR-0550** is refined (its `{oya,cloud}/<service>/` colocation + kernel/adapter/app seams become
the WITHIN-capability shape; the `libs/` charter is superseded by `base/` + capability homes).
**ADR-0537** (dogfood ladder, Proposed) is **implemented by this shape**; its §2 tier-dependency
lint enforcement gates on its own founder sign-off, and the §6 membership lint promotes on the same
sign-off.

### §10 Migration contract (reference, NOT executed here)

Migration is **strangler, per-capability**, after a repo-wide **Phase 0** that must land first:

1. populate the ADR-0280 DAG with all capability nodes + edges;
2. land the tier-metadata schema (`face:` / `tier_subtype:` / `bootstrap_tier:`) + a born-blocking
   coverage gate;
3. flip the de-brand profile (ADR-0532/0533);
4. build the reversible codemod + a green-snapshot oracle (the tree is byte-equivalent after the
   move modulo paths);
5. land the cargo+buck acyclicity dep-lint (§5);
6. land the membership lint (§6).

Then **each capability moves in ONE PR** via the codemod, green under all lints, before the next
capability moves. No crates move in this ADR.

#### §10.4 Phase-0 machinery: the reversible capability-move codemod (built; moves nothing)

Migration-contract item 4 (the reversible codemod + pre-move green-snapshot oracle) is realized by
the `oya-reorg-codemod-app` tool. It is a deterministic, idempotent, fail-closed engine that, given
a capability move plan (per the §3 placement rule), performs the directory `git mv` +
`Cargo.toml` package/dependency/relative-path-dep recompute (the ~200 move-fatal `../../../`
path-deps) + root workspace `members`/`exclude` rewrite (via the `oya-workspace-members-kernel`
resolver) + Rust `use`/`extern crate` kebab→snake rewrite + BUCK label/`name`/`crate` rewrite, and
emits an invertible `(old_path, new_path, old_cargo_name, new_cargo_name, buck_label)` mapping. The
inverse (`--revert`) restores the tree byte-identically; the pre-move green-snapshot oracle captures
`cargo metadata` + `buck2 targets //...`, and a dry-run shadow-apply PROVES a move resolves WITHOUT
landing it (exit 2 = unclean, fail-closed). It ships UNUSED — the strangler PRs invoke it; this ADR
moves no crate. The tool is a local bridge only (merge authority stays in cloud-ci/oya-ci). Its
tracked, born-accounted paths are `tools/oya-reorg-codemod-app/Cargo.toml`,
`tools/oya-reorg-codemod-app/BUCK`, `tools/oya-reorg-codemod-app/OWNERS`,
`tools/oya-reorg-codemod-app/src/lib.rs`, `tools/oya-reorg-codemod-app/src/model.rs`,
`tools/oya-reorg-codemod-app/src/cargo.rs`, `tools/oya-reorg-codemod-app/src/buck.rs`,
`tools/oya-reorg-codemod-app/src/rust_src.rs`, `tools/oya-reorg-codemod-app/src/plan.rs`,
`tools/oya-reorg-codemod-app/src/oracle.rs`, `tools/oya-reorg-codemod-app/src/main.rs`,
`tools/oya-reorg-codemod-app/tests/fixture_roundtrip.rs`, and
`registry/catalog/oya-reorg-codemod-app.yaml`.

#### §10.5 First executed strangler move: `messaging` capability (oya/eventing → messaging/)

The first REAL codemod run homes the `messaging` capability. The two former `oya/eventing` crates
move under the §3 placement rule with the face mirrored by the sub-fold: the CloudEvent/outbox
domain (`face: core`, the engine we run) → `messaging/core/domain` (cargo `messaging-domain`); the
file-backed outbox adapter (`face: adapters`, transient-infra impl) → `messaging/adapters/file`
(cargo `messaging-file-adapter`). The de-brand drops the `oya-eventing-` prefix to the capability
slug. The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree
dry-run (`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree); its
three dependents (`oya/application`, `oya/audit-chain`, `oya/developer-sdk` dev-cli) had their
cargo path-deps, BUCK labels, and Rust `use` idents recomputed mechanically. The capability registry
`messaging.absorbs_current_dirs` flips `oya/eventing` → `messaging`, the membership/acyclicity policy
scan roots gain `messaging`, and the root workspace gains the `messaging/*/*` member glob (ADR-0538
glob-only contract). The move's tracked, born-accounted artifact paths are
`messaging/core/domain/Cargo.toml`, `messaging/core/domain/BUCK`, `messaging/core/domain/src/lib.rs`,
`messaging/core/domain/src/cloud_event.rs`, `messaging/adapters/file/Cargo.toml`,
`messaging/adapters/file/BUCK`, `messaging/adapters/file/src/lib.rs`,
`messaging/adapters/file/tests/file_outbox.rs`, and the subtree `messaging/OWNERS`.

#### §10.6 Second executed strangler move: `iac` capability (cloud/cloud-iac/crates → iac/)

The second REAL codemod run homes the `iac` capability's five crates under the §3 placement rule,
each face mirrored by its sub-fold: the pure IaC plan/diff/drift domain engine (`face: core`) →
`iac/core/domain` (cargo `iac-domain`); the authz + DTO application boundary (`face: core`, concrete
logic over the domain) → `iac/core/api` (cargo `iac-api`); the framework-free route-table / authz-surface
HTTP contract (`face: ports`, the stable seam) → `iac/ports/rest` (cargo `iac-rest`); the transient infra
impls (`face: adapters`) → `iac/adapters/infrastructure` (cargo `iac-infrastructure`); the single-capability
deployable composition root (`face: facade`, §6: a single-capability app IS a facade) → `iac/facade/app`
(cargo `iac-app`). The de-brand drops the `oya-cloud-iac-` prefix to the capability slug; cargo names are
the path-tail leaf (the fold is carried by the path + manifest `face:` facet, not repeated in the name),
matching the §10.5 `messaging-domain` precedent. The move was performed by `oya-reorg-codemod-app` (NOT by
hand), gated on the buck2-full-tree dry-run (`cargo metadata` + `buck2 targets //...` both resolved
post-move on a shadow tree, `buck_ok=true` not null). The five crates have NO first-party dependents
OUTSIDE the capability (every cargo/BUCK/Rust reference was intra-`cloud-iac`), so the move's rewrites are
entirely intra-capability. The capability registry `iac.absorbs_current_dirs` flips `cloud/cloud-iac` →
`iac`, the membership policy scan_roots + allowed_top_level_dirs gain `iac`, the acyclicity policy
crate_root_globs gains `iac/*/*` + unclassified_roots gains `iac`, and the root workspace gains the
`iac/*/*` member glob (one glob covers all three folds; ADR-0538 glob-only contract).

**Known intra-capability face-rule gap (composition root):** `iac/facade/app` (`face: facade`) depends on
`iac/adapters/infrastructure` (`face: adapters`) — a facade→adapters edge. §4 says a facade composes core
ONLY through ports; the composition-root reality (the deployable wires concrete adapters) is the universal
exception. No intra-capability face-direction gate exists today (the membership lint checks face↔sub-fold
agreement and the closed-set; the acyclicity lint checks tier-DAG direction across capability roots, not
within-capability face direction), so this edge is NOT flagged and the move lands green. Tracked here as a
face-rule gap pending a §4 composition-root carve-out; no hack is introduced.

**De-brand residue (out of scope here, §9 later lane):** the codemod renames crate names/idents/labels/
path-deps only, never string literals or non-crate target names. So the app's `[[bin]] name = "oya-cloud-iac"`
(a binary artifact name), the runtime self-identity constants (`CLOUD_IAC_APP_BINARY_NAME`,
`CLOUD_IAC_APP_PACKAGE_NAME`, the `OYA_CLOUD_IAC_*` env-var contract, `microservices/cloud-iac/...` and
`target/oya-cloud-iac/...` path literals), and the `iac/facade/app` rust_test `crate_root` / `mapped_srcs`
SANDBOX-INTERNAL destination labels (`cloud/cloud-iac/...`; the real resolved source is the moved
`iac/facade/app/tests/cloud_iac_app.rs`, verified via `buck2 cquery inputs(...)`) are intentionally
preserved. These are application/runtime identity, not crate identifiers; their de-brand is the ADR-0532/0533
profile lane (§9), not this structural move.

**Non-crate capability artifacts retained in place:** unlike `oya/eventing` (a pure-crate dir that vanished
in §10.5), `cloud/cloud-iac/` also holds ~242 non-crate capability artifacts (docs, slos, contracts,
`iac/` GitOps manifests, catalog, tofu modules incl. `cloud/cloud-iac/tofu/modules:release-index.json`
which the app test still depends on as a live label). This crates-only strangler move homes the CRATES; the
non-crate artifacts stay at `cloud/cloud-iac/` (a separate later concern), so the old dir is NOT fully
removed — only `cloud/cloud-iac/crates/` is emptied.

The move's tracked, born-accounted artifact paths are `iac/core/domain/Cargo.toml`, `iac/core/domain/BUCK`,
`iac/core/domain/src/lib.rs`, `iac/core/domain/tests/cloud_iac_foundation.rs`,
`iac/core/domain/tests/gitops_drift_reconciliation.rs`, `iac/core/domain/tests/iac_plan_diff.rs`,
`iac/core/domain/tests/opentofu_plan_changeset.rs`, `iac/core/api/Cargo.toml`, `iac/core/api/BUCK`,
`iac/core/api/src/lib.rs`, `iac/core/api/tests/cloud_iac_api.rs`, `iac/ports/rest/Cargo.toml`,
`iac/ports/rest/BUCK`, `iac/ports/rest/src/lib.rs`, `iac/ports/rest/tests/cloud_iac_rest.rs`,
`iac/adapters/infrastructure/Cargo.toml`, `iac/adapters/infrastructure/BUCK`,
`iac/adapters/infrastructure/src/lib.rs`, `iac/adapters/infrastructure/tests/cloud_iac_infrastructure.rs`,
`iac/facade/app/Cargo.toml`, `iac/facade/app/BUCK`, `iac/facade/app/src/lib.rs`,
`iac/facade/app/src/main.rs`, `iac/facade/app/tests/cloud_iac_app.rs`, and the subtree `iac/OWNERS`.

#### §10.7 Third executed strangler move: `observability` capability (cloud/cloud-observability + oya/observability → observability/)

The third REAL codemod run homes the `observability` capability's five crates from TWO source dirs
(`cloud/cloud-observability/crates` + `oya/observability/crates`) under the §3 placement rule, each
face mirrored by its sub-fold across two faces (`core`, `adapters`):
the cloud aggregate domain engine (`face: core`) → `observability/core/aggregate`
(cargo `observability-aggregate`); the cloud audit-DTO application boundary (`face: core`, concrete
logic over the domain) → `observability/core/api` (cargo `observability-api`); the cloud
self-identity primitive (`face: core`, leaf) → `observability/core/kernel`
(cargo `observability-kernel`); the base telemetry/SLO vocabulary domain engine (`face: core`,
foundational) → `observability/core/domain` (cargo `observability-domain`); and the
tracing-subscriber transient-infra impl (`face: adapters`) → `observability/adapters/tracing`
(cargo `observability-tracing-adapter`). The de-brand drops the `oya-cloud-observability-` /
`oya-observability-` prefixes to the capability slug; cargo names are the de-branded leaf, matching
the §10.5/§10.6 precedent.

**Face reasoning — both source dirs are `core`, not `facade` (§3 "by WHAT IT IS, not cloud/oya
prefix"):** the cloud→oya dependency runs DOWNWARD — `oya-cloud-observability-domain`
(cloud aggregate) depends on `oya-observability-domain` (the base telemetry vocab) — so
`oya/observability` is the FOUNDATIONAL core of the capability, NOT a product facade; mapping it to
`facade/` would invert the dependency edge (a facade depended on by a `core` crate). Both the cloud
aggregate/api and the oya base vocab are therefore `core/`. Per the §3 tie-breaker, the oya base
vocab sits at the lowest DAG node it participates in (a leaf with one `oya-data-boundary-kernel`
dep), so it is unambiguously foundational core.

**Collision resolution (two `*-domain` crates into one capability):** both source dirs held a
`*-domain` crate (`oya-cloud-observability-domain` and `oya-observability-domain`). De-branding both
to `observability-domain` would collide on path AND cargo name. The cloud aggregate domain is
therefore RENAMED to `aggregate` (`observability/core/aggregate`, cargo `observability-aggregate`)
to reflect WHAT IT IS (it aggregates cloud-side telemetry over the base vocab), while the
foundational base telemetry vocab keeps `domain` (`observability/core/domain`, cargo
`observability-domain`). The resulting 5 paths + 5 cargo names are all distinct
(`MovePlan::validate` passes: no duplicate `old_path`/`new_path`/`old_cargo_name`/`new_cargo_name`,
no nested target).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree
dry-run (`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree,
`buck_ok=true` not null, `clean=true`). The single first-party dependent OUTSIDE the capability —
`oya/application/crates/oya-application-app` (depending on `observability-domain` +
`observability-tracing-adapter`) — had its cargo path-deps, BUCK labels, and Rust `use` idents
recomputed mechanically. The capability registry `observability.absorbs_current_dirs` gains
`observability` (the old `cloud/cloud-observability` / `oya/observability` / `oya/diagnostics`
entries are retained for the phase-2 non-crate residue + the crate-free `oya/diagnostics`), the
membership policy scan_roots + allowed_top_level_dirs gain `observability`, the acyclicity policy
crate_root_globs gains `observability/*/*` + unclassified_roots gains `observability`, and the root
workspace gains the `observability/*/*` member glob (one glob covers both faces; ADR-0538 glob-only
contract).

**Rename-aware baseline relabel exercised (ADR-0563, first MOVE PR to fire it):** this is the first
strangler move under the rename-aware path-keyed CI baseline relabel machinery (ADR-0563 / #737), so
it commits exactly ONE move-plan at `specs/reorg/observability-move-plan.json` (the codemod's
`MovePlan` bijection), and the move-manifest at `specs/reorg/move-manifest.generated.json` is
regenerated from it via `oya-reorg-codemod manifest --plan` (registry-drift byte-bound). Three
relocated source files carry PRE-EXISTING brand-residue vocabulary (a retired-brand stem the
shrink-only ratchet tracks via the `cloud-ci-brand-residue` gate) at `observability/core/aggregate/src/lib.rs`,
`observability/core/api/tests/cloud_observability_audit_api.rs`, and `observability/core/domain/src/lib.rs`
whose OLD paths are in that gate's frozen merge-base baseline; the emitter relabel maps those OLD
paths to their NEW paths in the frozen face, so the firewall reads zero new debt (the residue was
already accepted at the old path; the move adds none). This ADR §10.7 record itself names no
brand-stem token verbatim (per the same shrink-only ratchet, exactly as ADR-0563 describes the
residue). No manual signoff door is used.

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like
`cloud/cloud-iac` in §10.6, both `cloud/cloud-observability/` and `oya/observability/` also hold
non-crate capability artifacts (docs, slos, contracts, GitOps manifests), and `oya/diagnostics/`
holds ZERO crates (pure non-crate artifacts). This crates-only strangler move homes the CRATES; the
non-crate artifacts + `oya/diagnostics` stay in place and are homed in phase-2 (task #62), so only
the `crates/` subtrees are emptied.

The move's tracked, born-accounted artifact paths are `observability/core/kernel/Cargo.toml`,
`observability/core/kernel/BUCK`, `observability/core/kernel/src/lib.rs`,
`observability/core/aggregate/Cargo.toml`, `observability/core/aggregate/BUCK`,
`observability/core/aggregate/src/lib.rs`, `observability/core/api/Cargo.toml`,
`observability/core/api/BUCK`, `observability/core/api/src/lib.rs`,
`observability/core/api/tests/cloud_observability_audit_api.rs`,
`observability/core/domain/Cargo.toml`, `observability/core/domain/BUCK`,
`observability/core/domain/src/lib.rs`, `observability/core/domain/src/severity.rs`,
`observability/core/domain/src/slo.rs`,
`observability/core/domain/tests/severity_threshold_gate.rs`,
`observability/core/domain/tests/slo_burn_rate.rs`,
`observability/adapters/tracing/Cargo.toml`, `observability/adapters/tracing/BUCK`,
`observability/adapters/tracing/src/lib.rs`,
`observability/adapters/tracing/tests/slo_breach_observer.rs`, the subtree `observability/OWNERS`,
and the committed move-plan `specs/reorg/observability-move-plan.json` (reached by the existing
ADR-0563 `specs/reorg/` reachability prefix).

#### §10.8 Fourth executed strangler move: `compute` capability (cloud/cloud-compute/crates → compute/)

The fourth REAL codemod run homes the `compute` capability's eight crates from a SINGLE source dir
(`cloud/cloud-compute/crates`) under the §3 placement rule, each face mirrored by its sub-fold across
THREE faces (`core`, `facade`, `adapters`):
the compute aggregate domain engine (`face: core`) → `compute/core/domain` (cargo `compute-domain`);
the datacenter-ops domain engine (`face: core`, leaf) → `compute/core/dcops` (cargo `compute-dcops`);
the resource aggregate domain engine (`face: core`, leaf) → `compute/core/resource`
(cargo `compute-resource`); the VM product API surface (`face: facade`) → `compute/facade/vm`
(cargo `compute-vm-api`); the managed-K8s product API surface (`face: facade`) → `compute/facade/k8s`
(cargo `compute-k8s-api`); the Functions/serverless product API surface (`face: facade`) →
`compute/facade/functions` (cargo `compute-functions-api`); the transient AWS EC2 adapter
(`face: adapters`) → `compute/adapters/aws` (cargo `compute-aws-adapter`); and the transient OCI
adapter (`face: adapters`) → `compute/adapters/oci` (cargo `compute-oci-adapter`). The de-brand drops
the `oya-cloud-compute-` / `oya-cloud-dcops-` / `oya-cloud-resource-` prefixes to the capability
slug; cargo names are the de-branded leaf, matching the §10.5/§10.6/§10.7 precedent.

**Face reasoning — engine-with-facade-subsurfaces (§2 boundary note + §3 "by WHAT IT IS"):** the
capability registry records compute as ONE engine (vm + k8s-on-compute + functions) with facade
sub-surfaces, NOT three capabilities. The three `*-domain` aggregate kernels (compute, dcops,
resource) are the engine → `core/`. The vm/k8s/functions `*-api` crates are the multi-tenant product
surfaces the platform SELLS → `facade/` (the EKS-over-EC2 split: the engine is `core`, the sold
managed surface is `facade`). The aws/oci adapters are transient infra absorbed at cutover →
`adapters/`. The dependency direction is correct-downward: every `*-api` (facade) and every adapter
depends on `compute-domain` AND `compute-resource` (both `core`) — a facade→core / adapters→core
edge; `compute-domain` (core aggregate) itself depends on `compute-resource` (core leaf) — a
core→core edge. No `ports/` indirection layer exists yet, so the facade→core and adapters→core edges
are direct; this is dependency-legal and (as in §10.6's accepted facade→adapters edge) is NOT
gate-flagged — a §4 ports carve-out for these product surfaces is the documented future alternative
(`ports/{vm,k8s,functions}`), deferred. The `compute/facade/k8s` crate (managed-K8s ON compute) is
DISTINCT from the separate `k8s` capability (the owned Kubernetes control plane under
`cloud/cloud-k8s` + `managed-k8s-*`, a later move) — different source crates, not conflated.
The resulting 8 paths + 8 cargo names are all distinct (`MovePlan::validate` passes: no duplicate
`old_path`/`new_path`/`old_cargo_name`/`new_cargo_name`, no nested target).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). compute is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The first-party dependents OUTSIDE the capability — all on
`compute-resource` (formerly `oya-cloud-resource-domain`): `cloud/cloud-billing`,
`cloud/cloud-capacity`, `cloud/cloud-data`, `cloud/cloud-finops`, `cloud/cloud-kms`,
`cloud/cloud-network`, `cloud/cloud-storage` (×2 crates), and `observability/core/aggregate` — had
their cargo path-deps, BUCK labels, and Rust `use` idents recomputed mechanically by the codemod. The
capability registry `compute.absorbs_current_dirs` gains `compute` (the old `cloud/cloud-compute`
entry is retained for the phase-2 non-crate residue), the membership policy scan_roots +
allowed_top_level_dirs gain `compute`, the acyclicity policy crate_root_globs gains `compute/*/*` +
unclassified_roots gains `compute`, and the root workspace gains the `compute/*/*` member glob (one
glob covers all three faces; ADR-0538 glob-only contract).

**Rename-aware baseline relabel exercised (ADR-0563):** this move commits exactly ONE move-plan at
`specs/reorg/compute-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). One relocated source file —
`compute/core/resource/src/lib.rs` — carries PRE-EXISTING brand-residue vocabulary (a retired-brand
stem the shrink-only ratchet tracks via the `cloud-ci-brand-residue` gate) whose OLD path is in that
gate's frozen merge-base baseline; the emitter relabel maps that OLD path to its NEW path in the
frozen face (content-preserving, the move adds zero residue), so the firewall reads zero new debt.
This ADR §10.8 record itself names no brand-stem token verbatim (per the same shrink-only ratchet,
exactly as ADR-0563 describes the residue). No manual signoff door is used.

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like the
prior moves, `cloud/cloud-compute/` also holds non-crate capability artifacts (docs, slos, contracts,
GitOps manifests, tofu). This crates-only strangler move homes the CRATES; the non-crate artifacts
stay in place and are homed in phase-2 (task #62), so only the `crates/` subtree is emptied. The
deferred de-brand residue (the resource-domain brand stem above, plus any `[[bin]]`/`OYA_*` literals)
is the ADR-0532/0533 de-brand profile lane's scope (task #63), not this structural move.

**Born-accounting (ADR-0555):** the eight new crate dirs under `compute/{core,facade,adapters}/` are
reached by the `compute/*/*` member glob + the `compute/*/*` acyclicity glob, and owned by the
subtree `compute/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.8
entry. The move's tracked, born-accounted artifact paths are `compute/core/domain/Cargo.toml`,
`compute/core/domain/BUCK`, `compute/core/domain/src/lib.rs`,
`compute/core/domain/tests/cloud_compute_foundation.rs`, `compute/core/dcops/Cargo.toml`,
`compute/core/dcops/BUCK`, `compute/core/dcops/src/lib.rs`, `compute/core/resource/Cargo.toml`,
`compute/core/resource/BUCK`, `compute/core/resource/src/lib.rs`, `compute/facade/vm/Cargo.toml`,
`compute/facade/vm/BUCK`, `compute/facade/vm/src/lib.rs`,
`compute/facade/vm/tests/cloud_compute_vm_api.rs`, `compute/facade/k8s/Cargo.toml`,
`compute/facade/k8s/BUCK`, `compute/facade/k8s/src/lib.rs`,
`compute/facade/k8s/tests/cloud_compute_k8s_api.rs`, `compute/facade/functions/Cargo.toml`,
`compute/facade/functions/BUCK`, `compute/facade/functions/src/lib.rs`,
`compute/facade/functions/tests/cloud_compute_functions_api.rs`, `compute/adapters/aws/Cargo.toml`,
`compute/adapters/aws/BUCK`, `compute/adapters/aws/src/lib.rs`, `compute/adapters/oci/Cargo.toml`,
`compute/adapters/oci/BUCK`, `compute/adapters/oci/src/lib.rs`, the subtree `compute/OWNERS`, and the
committed move-plan `specs/reorg/compute-move-plan.json` (reached by the existing ADR-0563
`specs/reorg/` reachability prefix).

#### §10.9 Fifth executed strangler move: `storage` capability (cloud/cloud-storage + oya/drive + oya/recordings → storage/)

The fifth REAL codemod run homes the `storage` capability's seven crates from THREE source dirs
(`cloud/cloud-storage/crates`, `oya/drive/crates`, `oya/recordings/crates`) under the §3 placement
rule, each face mirrored by its sub-fold across FOUR faces (`core`, `ports`, `adapters`, `facade`):
the CAS/blob substrate domain engine (`face: core`, DEFINES the provider port traits
`StorageProviderObjectPort` / `StorageProviderBlockPort` + `StorageRepo`) → `storage/core/domain`
(cargo `storage-domain`); the object capability surface (`face: ports`) → `storage/ports/object-api`
(cargo `storage-object-api`); the block capability surface (`face: ports`) → `storage/ports/block-api`
(cargo `storage-block-api`); the transient S3 provider adapter (`face: adapters`) →
`storage/adapters/s3` (cargo `storage-s3-adapter`); the transient OCI provider adapter
(`face: adapters`) → `storage/adapters/oci` (cargo `storage-oci-adapter`); the Drive product domain
(`face: facade`) → `storage/facade/drive` (cargo `storage-drive-domain`); and the Recordings product
domain (`face: facade`) → `storage/facade/recordings` (cargo `storage-recordings-domain`). The
de-brand drops the `oya-cloud-storage-` / `oya-drive-` / `oya-recordings-` prefixes to the capability
slug; cargo names are the de-branded leaf, matching the §10.5/§10.6/§10.7/§10.8 precedent.

**Face reasoning — core substrate defining provider ports, with consumer-facing product facades (§2
boundary note + §3 "by WHAT IT IS"):** the capability registry records storage as ONE engine — the
content-addressed-store / blob substrate the platform RUNS. `cloud-storage`'s `*-domain` is the engine
→ `core/`: it DEFINES the outbound provider port traits (`StorageProviderObjectPort`,
`StorageProviderBlockPort`, `StorageRepo`) that the s3/oci adapters implement. The object-api /
block-api crates are the inbound capability boundary surfaces → `ports/` (ports→core is the legal
downward edge, the iac-rest precedent of §10.6). The s3/oci adapters are transient provider infra
absorbed at cutover → `adapters/` (adapters→core). The `oya/drive` (Drive) and `oya/recordings`
(Recordings) product domains are the consumer-facing surfaces the platform SELLS on top of the storage
substrate → `facade/` (the §2 iam-pattern: they sit on the object/blob substrate by product charter,
not by a Cargo-level edge into cloud-storage). The `domain` triple-clash is disambiguated by face+leaf:
`storage-domain` is the `core` engine, while `storage-drive-domain` / `storage-recordings-domain` are
`facade` products. The dependency directions are correct-downward: object-api/block-api/s3/oci each
depend on `storage-domain` (a ports→core / adapters→core edge), and the cross-capability runtime deps
are PRESERVED unchanged through the move — `block-api` still depends on
`//cloud/cloud-network/crates/oya-residency-domain` (residency, a runtime data-locality dependency,
NOT dev-only), and `object-api` still depends on `//cloud/cloud-kms/crates/oya-cloud-kms-domain` and
on the already-homed `//compute/core/resource:compute-resource`; the codemod only recomputed those
crates' OWN relative Cargo paths, leaving the cross-cap targets untouched. The resulting 7 paths + 7
cargo names are all distinct (`MovePlan::validate` passes: no duplicate `old_path`/`new_path`/
`old_cargo_name`/`new_cargo_name`, no nested target).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). storage is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The first-party dependent OUTSIDE the capability —
`oya/application/crates/oya-workspace-drive-api` (on the former `oya-drive-domain`, now
`storage-drive-domain`) — had its cargo path-dep, BUCK labels (lib + test), and Rust `use` ident
recomputed mechanically by the codemod. The capability registry `storage.absorbs_current_dirs` gains
`storage` (the old `cloud/cloud-storage` + `oya/drive` + `oya/imaging` + `oya/recordings` entries are
retained for the phase-2 non-crate residue, `oya/imaging` having zero crates today), the membership
policy scan_roots + allowed_top_level_dirs gain `storage`, the acyclicity policy crate_root_globs
gains `storage/*/*` + unclassified_roots gains `storage`, and the root workspace gains the
`storage/*/*` member glob (one glob covers all four faces; ADR-0538 glob-only contract).

**Rename-aware baseline relabel exercised (ADR-0563):** this move commits exactly ONE move-plan at
`specs/reorg/storage-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). The relocated `storage/facade/drive`
crate's `src/lib.rs` carried a PRE-EXISTING retired-brand-stem doc-comment whose OLD path was in the
`cloud-ci-brand-residue` gate's frozen merge-base baseline; this move SCRUBS that comment-only residue
(a content-preserving de-brand, no identifier or behavior change — removing residue is always
gate-allowed under the shrink-only ratchet), so the relocated file carries zero residue. The emitter
relabel maps the OLD path to its NEW path in the frozen face guarded by P4 (NEW_OCC ⊆ OLD_OCC, here
NEW_OCC = ∅), so the firewall reads a clean shrink (no new debt, the moved file is brand-clean at the
new path). This ADR §10.9 record itself names no brand-stem token verbatim (per the same shrink-only
ratchet, exactly as ADR-0563 describes the residue). No manual signoff door is used.

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like the
prior moves, `cloud/cloud-storage/`, `oya/drive/`, and `oya/recordings/` also hold non-crate
capability artifacts (docs, slos, contracts, GitOps manifests, tofu, the `oya/drive/manifest.json`
that still carries a phase-2 de-brand residue). This crates-only strangler move homes the CRATES; the
non-crate artifacts stay in place and are homed in phase-2 (task #62), so only the `crates/` subtrees
are emptied. `oya/imaging` holds zero crates and so is wholly a phase-2 non-crate concern. The
deferred de-brand residue outside the moved crates (the `oya/drive/manifest.json` brand stem, plus any
`[[bin]]`/`OYA_*` literals) is the ADR-0532/0533 de-brand profile lane's scope (task #63), not this
structural move.

**Born-accounting (ADR-0555):** the seven new crate dirs under `storage/{core,ports,adapters,facade}/`
are reached by the `storage/*/*` member glob + the `storage/*/*` acyclicity glob, and owned by the
subtree `storage/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.9
entry. The move's tracked, born-accounted artifact paths are `storage/core/domain/Cargo.toml`,
`storage/core/domain/BUCK`, `storage/core/domain/src/lib.rs`,
`storage/core/domain/tests/cloud_storage_foundation.rs`, `storage/ports/object-api/Cargo.toml`,
`storage/ports/object-api/BUCK`, `storage/ports/object-api/src/lib.rs`,
`storage/ports/object-api/tests/cloud_storage_object_api.rs`, `storage/ports/block-api/Cargo.toml`,
`storage/ports/block-api/BUCK`, `storage/ports/block-api/src/lib.rs`,
`storage/ports/block-api/tests/cloud_storage_block_api.rs`, `storage/adapters/s3/Cargo.toml`,
`storage/adapters/s3/BUCK`, `storage/adapters/s3/src/lib.rs`, `storage/adapters/oci/Cargo.toml`,
`storage/adapters/oci/BUCK`, `storage/adapters/oci/src/lib.rs`, `storage/facade/drive/Cargo.toml`,
`storage/facade/drive/BUCK`, `storage/facade/drive/src/lib.rs`,
`storage/facade/recordings/Cargo.toml`, `storage/facade/recordings/BUCK`,
`storage/facade/recordings/src/lib.rs`, `storage/facade/recordings/src/fhir_resource_type.rs`, the
subtree `storage/OWNERS`, and the committed move-plan `specs/reorg/storage-move-plan.json` (reached by
the existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.10 Sixth executed strangler move: `cell` capability (cloud/cloud-cell + cloud/cloud-capacity → cell/)

The sixth REAL codemod run homes the `cell` capability's eight crates from TWO source dirs
(`cloud/cloud-cell/crates`, `cloud/cloud-capacity/crates`) under the §3 placement rule, across TWO
faces (`core`, `ports`) with NO `facade` (cell is the ADR-0280 LEAF cellular-topology substrate — the
bootstrap floor the platform RUNS, not a product it SELLS): the cell-routing kernel (`face: core`) →
`cell/core/routing` (cargo `cell-routing`); the region domain engine (`face: core`) →
`cell/core/region` (cargo `cell-region`); the region capability surface (`face: ports`) →
`cell/ports/region` (cargo `cell-region-api`); the regional-pack domain engine (`face: core`) →
`cell/core/regional-pack` (cargo `cell-regional-pack`); the regional-pack capability surface
(`face: ports`) → `cell/ports/regional-pack` (cargo `cell-regional-pack-api`); the no-dep cell-bind
binding library (`face: ports`) → `cell/ports/cell-bind` (cargo `cell-bind-api`); the capacity kernel
(`face: core`) → `cell/core/capacity` (cargo `cell-capacity`); and the commercial-capacity domain
(`face: core`) → `cell/core/capacity-commercial` (cargo `cell-capacity-commercial`). The de-brand
drops the `oya-cell-` / `oya-cloud-region-` / `oya-regional-pack-` / `oya-cloud-cell-` /
`oya-cloud-capacity-` prefixes to the capability slug, matching the §10.5..§10.9 precedent. ONE
de-dup-path-doubling refinement applies (naming-grammar): the routing kernel homes to
`cell/core/routing` (cargo `cell-routing`), NOT `cell/core/cell` (cargo `cell-cell`), since it IS the
"Cell-routing kernel" and the leaf slug must not double the capability slug.

**Face reasoning — leaf substrate, core engines + inbound ports, no facade (§2 boundary note + §3 "by
WHAT IT IS"):** the capability registry records cell as the ADR-0280 leaf substrate (cluster-per-cell,
hard caps, the thin static-stable cell router, autosharding/rebalancing). All five engine/kernel
crates (routing, region, regional-pack, capacity, capacity-commercial) are the substrate the platform
RUNS → `core/`. The three `*-api` binding crates are the inbound capability boundary surfaces →
`ports/` (ports→core is the legal downward edge, the iac-rest / storage precedent of §10.6/§10.9): the
region-api and regional-pack-api crates each depend on their `core/` engine, and the
`cloud-cell-app` crate is a NO-DEP `rust_library` (NOT a `[[bin]]` composition-root) holding the bind
DTOs + a pure cell-lifecycle state machine that proves the `cloud-cell-bind-v1` OpenAPI contract — an
inbound API/binding boundary → `cell/ports/cell-bind`, NOT `facade`, NOT `core/app`. cell is NOT a
sold product (it is the bootstrap floor every other capability sits on), so it has NO `facade/`. The
intra-capability edges are all correct-downward and legal: `cell-region` → `cell-routing`
(core→core), `cell-region-api` → `cell-region` (ports→core), `cell-regional-pack-api` →
`cell-regional-pack` (ports→core), and `cell-capacity-commercial` → `cell-region` (core→core). The
resulting 8 paths + 8 cargo names are all distinct (`MovePlan::validate` passes: no duplicate
`old_path`/`new_path`/`old_cargo_name`/`new_cargo_name`, no nested target).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). cell is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression.

**Largest dependent blast radius to date — 14 first-party dependents OUTSIDE the capability rewritten
mechanically by the codemod (cargo path-dep + dep-key + BUCK `//`label + Rust `use`/ident across all
three surfaces):** the `cell-region` engine (formerly `oya-cloud-region-domain`, lib ident
`oya_cloud_region_domain` → `cell_region`) has thirteen cross-capability consumers, and the
`cell-routing` + `cell-regional-pack` engines add one more. The full set:
`oya/application/crates/oya-application-app` (on `cell-routing` AND `cell-regional-pack`),
`compute/core/domain`, `compute/core/resource`, `compute/core/dcops`,
`observability/core/aggregate`, `cloud/cloud-billing/crates/oya-cloud-billing-domain`,
`cloud/cloud-data/crates/oya-cloud-data-domain`, `cloud/cloud-finops/crates/oya-cloud-finops-domain`,
`cloud/cloud-iam/crates/oya-cloud-iam-domain`, `cloud/cloud-kms/crates/oya-cloud-kms-api`,
`cloud/cloud-kms/crates/oya-cloud-kms-domain`,
`cloud/cloud-marketplace/crates/oya-cloud-marketplace-domain`,
`cloud/cloud-network/crates/oya-cloud-network-domain`, and `storage/core/domain` (the already-homed
move-5 storage core, on `cell-region`). Each dependent's three surfaces were recomputed mechanically
and verified consistent (a half-rewritten dependent that compiles-by-luck is the danger this move
explicitly guards against; the post-move grep is clean of every old token in every dependent). The
capability registry `cell.absorbs_current_dirs` gains `cell` (the old `cloud/cloud-cell` +
`cloud/cell-lifecycle` + `cloud/cell-rebalancer` + `cloud/cloud-capacity` entries are retained for the
phase-2 non-crate residue — `cell-lifecycle` and `cell-rebalancer` hold zero crates today), the
membership policy scan_roots + allowed_top_level_dirs gain `cell`, the acyclicity policy
crate_root_globs gains `cell/*/*` + unclassified_roots gains `cell`, and the root workspace gains the
`cell/*/*` member glob (one glob covers both faces and all eight leaves; ADR-0538 glob-only contract).

**Rename-aware baseline relabel exercised (ADR-0563):** this move commits exactly ONE move-plan at
`specs/reorg/cell-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). THREE relocated crate files carried a
PRE-EXISTING `cloud-ci-brand-residue` first-forbidden-stem baseline entry at their OLD path —
`cloud/cloud-cell/crates/oya-cell-domain/src/lib.rs` → `cell/core/routing/src/lib.rs`,
`cloud/cloud-cell/crates/oya-cloud-region-api/src/lib.rs` → `cell/ports/region/src/lib.rs`, and
`cloud/cloud-cell/crates/oya-cloud-region-domain/src/lib.rs` → `cell/core/region/src/lib.rs`. That
residue is a domain enum-variant / density-class string (a tenant-density / cell-tier identifier, NOT
a retired-VCS-brand reference), and the codemod renames only the crate-IDENT prefixes
(`oya_cloud_region_domain::` → `cell_region::` etc.), leaving the residue byte-identical at the new
path. The emitter relabel maps each OLD path to its NEW path in the frozen face guarded by P4
(NEW_OCC ⊆ OLD_OCC, here NEW_OCC == OLD_OCC), so the firewall reads a pure relocation (no new debt,
no scrub). No manual signoff door is used.

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like the
prior moves, `cloud/cloud-cell/` and `cloud/cloud-capacity/` also hold non-crate capability artifacts
(docs, slos, contracts, GitOps manifests, the `contracts/openapi/cloud/cloud-cell-bind-v1.yaml`
contract). This crates-only strangler move homes the CRATES; the non-crate artifacts stay in place and
are homed in phase-2 (task #62), so only the `crates/` subtrees are emptied. The crate-free
`cloud/cell-lifecycle/` and `cloud/cell-rebalancer/` capability dirs hold zero crates and so are
wholly a phase-2 non-crate concern. The deferred de-brand residue outside the moved crates is the
ADR-0532/0533 de-brand profile lane's scope (task #63), not this structural move.

**Born-accounting (ADR-0555):** the eight new crate dirs under `cell/{core,ports}/` are reached by the
`cell/*/*` member glob + the `cell/*/*` acyclicity glob, and owned by the subtree `cell/OWNERS`
(axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.10 entry. The move's
tracked, born-accounted artifact paths are `cell/core/routing/Cargo.toml`, `cell/core/routing/BUCK`,
`cell/core/routing/src/lib.rs`, `cell/core/routing/tests/cell_router.rs`,
`cell/core/region/Cargo.toml`, `cell/core/region/BUCK`, `cell/core/region/src/lib.rs`,
`cell/core/regional-pack/Cargo.toml`, `cell/core/regional-pack/BUCK`,
`cell/core/regional-pack/src/lib.rs`, `cell/core/regional-pack/src/capability_pack.rs`,
`cell/core/regional-pack/src/kr_regulatory.rs`, `cell/core/regional-pack/src/pack_onboarding_phase.rs`,
`cell/core/regional-pack/src/vertical_regulatory_profile.rs`, `cell/core/capacity/Cargo.toml`,
`cell/core/capacity/BUCK`, `cell/core/capacity/src/lib.rs`, `cell/core/capacity/src/cell_budget.rs`,
`cell/core/capacity/src/committed_use.rs`, `cell/core/capacity-commercial/Cargo.toml`,
`cell/core/capacity-commercial/BUCK`, `cell/core/capacity-commercial/src/lib.rs`,
`cell/core/capacity-commercial/tests/cloud_ops_foundation.rs`, `cell/ports/region/Cargo.toml`,
`cell/ports/region/BUCK`, `cell/ports/region/src/lib.rs`,
`cell/ports/region/tests/cloud_region_api.rs`, `cell/ports/regional-pack/Cargo.toml`,
`cell/ports/regional-pack/BUCK`, `cell/ports/regional-pack/src/lib.rs`,
`cell/ports/regional-pack/tests/regulatory_pack_bind_api.rs`, `cell/ports/cell-bind/Cargo.toml`,
`cell/ports/cell-bind/BUCK`, `cell/ports/cell-bind/src/lib.rs`,
`cell/ports/cell-bind/src/cell_lifecycle.rs`, `cell/ports/cell-bind/tests/cloud_cell_bind_api.rs`, the
subtree `cell/OWNERS`, and the committed move-plan `specs/reorg/cell-move-plan.json` (reached by the
existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.11 Seventh executed strangler move: `gateway` capability (oya/connector → gateway/)

The seventh REAL codemod run homes the `gateway` capability's ten SaaS-integration crates from ONE
source dir (`oya/connector/crates`) under the §3 placement rule, across ONE face (`adapters`) with NO
`core`, `ports`, or `facade` yet: each external-system integration crate (`face: adapters`) homes to
`gateway/adapters/<vendor>-connector` (cargo `gateway-<vendor>-connector`) for the ten vendors adp,
epic-fhir, gusto, netsuite, quickbooks, rippling, salesforce, slack, teams, and workday. The de-brand
drops the `oya-connector-<vendor>-adapter` form to the capability slug `gateway-<vendor>-connector`,
matching the §10.5..§10.10 precedent. gateway is the API-gateway / SSOT-edge capability (the
`api-gateway` dag node): the Check/Report enforcement point, the public API contract surface, and
rate/quota enforcement.

**Face reasoning — all-adapters capability for now, no core/ports/facade (§2 boundary note + §3 "by
WHAT IT IS"):** the capability registry records gateway as absorbing both `oya/api-gateway` and
`oya/connector`. The `oya/api-gateway` dir holds ZERO crates today (only non-crate capability
artifacts — contracts, slos, policy, runbooks — that are a phase-2 concern, task #62), so the
api-gateway engine/port/facade crates that will become gateway's `core/` + `ports/` + `facade/` are
not yet authored. The ten crates that DO exist are all SaaS-connector ADAPTERS — outbound integration
clients for external systems (HRIS/ERP/CRM/FHIR/collaboration vendors) — so they home to `adapters/`.
gateway is therefore an ALL-ADAPTERS capability at this incremental stage; the missing faces arrive
when the api-gateway crates are authored (acceptable per the crate-first strangler — a capability is
"crate-homed" per move and "fully homed" after phase-2). Each connector's SOLE dependency is the
shared `libs/oya-shared-connector-kernel` (the connector-runtime contract substrate), which STAYS in
`libs/` — it is a cross-capability shared kernel, not a gateway-owned crate, so re-homing it is a
future move if it happens at all. There are ZERO intra-capability edges among the ten connectors and
ZERO reverse dependents anywhere in the tree (the simplest blast radius of any move to date: the
codemod rewrote no first-party dependent because none exists). The resulting 10 paths + 10 cargo
names are all distinct (`MovePlan::validate` passes: no duplicate `old_path`/`new_path`/
`old_cargo_name`/`new_cargo_name`, no nested target).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). gateway is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The capability registry `gateway.absorbs_current_dirs` gains `gateway`
(the existing `oya/api-gateway` + `oya/connector` entries are retained for the phase-2 non-crate
residue), the membership policy scan_roots + allowed_top_level_dirs gain `gateway`, the acyclicity
policy crate_root_globs gains `gateway/*/*` + unclassified_roots gains `gateway`, and the root
workspace gains the `gateway/*/*` member glob (one glob covers the single `adapters` face and all ten
leaves; ADR-0538 glob-only contract).

**Zero-dependent move, no relabel needed (ADR-0563):** this move commits exactly ONE move-plan at
`specs/reorg/gateway-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). None of the ten relocated crates carries
a frozen brand-residue baseline entry (`connector` is not a forbidden vocabulary stem), so the
rename-aware emitter relabels only the move-plan→manifest path-keyed baselines that point at the moved
tree; no scrub and no manual signoff door are used.

**Registry stores de-branded in lockstep (registry/scripts):** the SaaS connectors are
registry-tracked, so the registry SSOT store + the dependency-rationales store had their per-crate
KEYS renamed to the new crate ids (`gateway-<vendor>-connector`, capability `gateway`) — these are
internal JSON keys, not separate tracked paths, so the rename de-brands without any reachability
impact (the store FILES keep their signed-off accounting status; their tracked paths are NOT cited
here, to preserve the founder one-way-door admission). The retired-grouping-wording transitional
script + its test example crate path were updated to the new `gateway/adapters/netsuite-connector`
path (the .sh gate's own test stays green). The de-brand also scrubbed each crate `src/lib.rs`
self-name doc comment (plus the README/openapi self-name references and the slack stale
anthropic-secrets cross-ref) to the new name.

**Per-crate catalog/SLO records retained at old names (crate-first incremental, task #62):** like the
cell/storage precedent (move-5/6 left `oya-cell-domain.yaml` etc. at old names), the ten per-crate
`registry/catalog/oya-connector-<vendor>-adapter.yaml` SLO-catalog records STAY at their old paths and
are homed in phase-2. They are accepted unreachable debt in the frozen merge-base total-accounting
baseline, so leaving them in place is gate-green; RENAMING them would mint NEW unreachable tracked
paths (the move-plan→manifest relabel only relocates files UNDER the moved crate dirs, not sibling
catalog records), which total-accounting blocks on. The slo-coverage gate stays green at the old stem
(it requires only a non-blank `slo:` scalar, not a live-crate stem).

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like the
prior moves, `oya/connector/` also holds non-crate capability artifacts (contracts, slos, policy,
runbooks, IP journeys, decisions, the per-crate catalog records above), and `oya/api-gateway/` holds
wholly non-crate artifacts. This crates-only strangler move homes the CRATES; the non-crate artifacts
stay in place and are homed in phase-2 (task #62), so only the `crates/` subtree of `oya/connector/`
is emptied. The deferred de-brand residue outside the moved crates is the ADR-0532/0533 de-brand
profile lane's scope (task #63), not this structural move.

**Born-accounting (ADR-0555):** the ten new crate dirs under `gateway/adapters/` are reached by the
`gateway/*/*` member glob + the `gateway/*/*` acyclicity glob, and owned by the subtree
`gateway/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.11 entry.
The move's tracked, born-accounted artifact paths are
`gateway/adapters/adp-connector/Cargo.toml`, `gateway/adapters/adp-connector/BUCK`,
`gateway/adapters/adp-connector/src/lib.rs`, `gateway/adapters/adp-connector/README.md`,
`gateway/adapters/adp-connector/specs/cedar-policy.cedar`,
`gateway/adapters/adp-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/epic-fhir-connector/Cargo.toml`, `gateway/adapters/epic-fhir-connector/BUCK`,
`gateway/adapters/epic-fhir-connector/src/lib.rs`, `gateway/adapters/epic-fhir-connector/README.md`,
`gateway/adapters/epic-fhir-connector/specs/cedar-policy.cedar`,
`gateway/adapters/epic-fhir-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/gusto-connector/Cargo.toml`, `gateway/adapters/gusto-connector/BUCK`,
`gateway/adapters/gusto-connector/src/lib.rs`, `gateway/adapters/gusto-connector/README.md`,
`gateway/adapters/gusto-connector/specs/cedar-policy.cedar`,
`gateway/adapters/gusto-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/netsuite-connector/Cargo.toml`, `gateway/adapters/netsuite-connector/BUCK`,
`gateway/adapters/netsuite-connector/src/lib.rs`, `gateway/adapters/netsuite-connector/README.md`,
`gateway/adapters/netsuite-connector/specs/cedar-policy.cedar`,
`gateway/adapters/netsuite-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/quickbooks-connector/Cargo.toml`, `gateway/adapters/quickbooks-connector/BUCK`,
`gateway/adapters/quickbooks-connector/src/lib.rs`, `gateway/adapters/quickbooks-connector/README.md`,
`gateway/adapters/quickbooks-connector/specs/cedar-policy.cedar`,
`gateway/adapters/quickbooks-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/rippling-connector/Cargo.toml`, `gateway/adapters/rippling-connector/BUCK`,
`gateway/adapters/rippling-connector/src/lib.rs`, `gateway/adapters/rippling-connector/README.md`,
`gateway/adapters/rippling-connector/specs/cedar-policy.cedar`,
`gateway/adapters/rippling-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/salesforce-connector/Cargo.toml`, `gateway/adapters/salesforce-connector/BUCK`,
`gateway/adapters/salesforce-connector/src/lib.rs`, `gateway/adapters/salesforce-connector/README.md`,
`gateway/adapters/salesforce-connector/specs/cedar-policy.cedar`,
`gateway/adapters/salesforce-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/slack-connector/Cargo.toml`, `gateway/adapters/slack-connector/BUCK`,
`gateway/adapters/slack-connector/src/lib.rs`, `gateway/adapters/slack-connector/README.md`,
`gateway/adapters/slack-connector/specs/cedar-policy.cedar`,
`gateway/adapters/slack-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/teams-connector/Cargo.toml`, `gateway/adapters/teams-connector/BUCK`,
`gateway/adapters/teams-connector/src/lib.rs`, `gateway/adapters/teams-connector/README.md`,
`gateway/adapters/teams-connector/specs/cedar-policy.cedar`,
`gateway/adapters/teams-connector/specs/openapi.snapshot.yaml`,
`gateway/adapters/workday-connector/Cargo.toml`, `gateway/adapters/workday-connector/BUCK`,
`gateway/adapters/workday-connector/src/lib.rs`, `gateway/adapters/workday-connector/README.md`,
`gateway/adapters/workday-connector/specs/cedar-policy.cedar`,
`gateway/adapters/workday-connector/specs/openapi.snapshot.yaml`, the subtree `gateway/OWNERS`, and
the committed move-plan `specs/reorg/gateway-move-plan.json` (reached by the existing ADR-0563
`specs/reorg/` reachability prefix).

#### §10.12 Eighth executed strangler move: `flags` capability (oya/feature-flags → flags/)

The eighth REAL codemod run homes the `flags` capability's single crate from ONE source dir
(`oya/feature-flags/crates`) under the §3 placement rule, across ONE face (`core`) with NO `ports`,
`adapters`, or `facade`: the OFREP/gRPC/REST feature-flag evaluation server (`face: core`) homes to
`flags/core/server` (cargo `flags-server`, lib `flags_server`). The de-brand drops the `oya-flags`
form to the capability slug `flags-server`, matching the §10.5..§10.11 precedent. flags is the
feature-flag / configuration substrate (the `flags` dag node): dynamic config, progressive delivery,
and kill switches.

**Face reasoning — single core crate, no ports/adapters/facade (§3 "by WHAT IT IS"):** the capability
registry records flags as absorbing both `oya/feature-flags` and the crate-empty placeholder
`oya/oya-flags`. The `oya/oya-flags` dir holds ZERO crates (a neutralized BUCK with every target
commented out, no `Cargo.toml`, no `.rs` source — only non-crate artifacts), so it contributes no
crate to this move; its non-crate residue is a phase-2 concern (task #62) while the registry still
ABSORBS it so it is accounted under `flags`. The ONE crate that exists under `oya/feature-flags` is
the evaluation SERVER — the substrate engine itself, carrying its `ofrep`, `grpc`, `rest`,
`targeting`, `evaluation`, `storage`, `tenants`, and `observability` subsystems as internal modules
(the aspirational ADR-0481 three-crate kernel/rest/app split was never built). Because the crate IS
the flag-evaluation substrate, it homes to `core/`; flags is therefore a single-core-crate capability
at this incremental stage, and the missing faces would arrive only if the engine is later decomposed
(acceptable per the crate-first strangler — a capability is "crate-homed" per move and "fully homed"
after phase-2). The crate's SOLE dependency is `tokio` (a third-party workspace crate); there are
ZERO intra-capability edges and ZERO first-party reverse dependents anywhere in the tree (the
simplest blast radius of any move to date — the codemod rewrote no first-party dependent because
none exists). The resulting path + cargo name are distinct (`MovePlan::validate` passes).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). flags is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dir is not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The capability registry already records `flags.absorbs_current_dirs` with
both `oya/feature-flags` and `oya/oya-flags` (retained for the phase-2 non-crate + placeholder
residue), the membership policy scan_roots + allowed_top_level_dirs gain `flags`, the acyclicity
policy crate_root_globs gains `flags/*/*` + unclassified_roots gains `flags`, and the root workspace
gains the `flags/*/*` member glob (one glob covers the single `core` face and single `server` leaf;
ADR-0538 glob-only contract).

**Zero-dependent move, no relabel needed (ADR-0563):** this move commits exactly ONE move-plan at
`specs/reorg/flags-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). The relocated crate carries no frozen
brand-residue baseline entry (`flags` is not a forbidden vocabulary stem), so the rename-aware emitter
relabels only the move-plan→manifest path-keyed baselines that point at the moved tree; no scrub and
no manual signoff door are used.

**Registry SSOT store de-branded in lockstep (registry/stores):** the evaluation server is
registry-tracked, so the registry SSOT store's per-crate KEY was renamed to the new crate id
(`flags-server`, capability `flags`) — an internal JSON key edit, not a separate tracked path, so the
rename de-brands without any reachability impact (the store FILE keeps its signed-off accounting
status; its tracked path is NOT cited here, to preserve the founder one-way-door admission). The
per-crate dependency-rationales store holds no entry for this crate (its sole dep is the third-party
`tokio`), so no rationale edit is needed.

**Per-crate catalog/SLO records re-keyed by PR-C1 (catalog doctrine-fix):** PR-C1 applies the
founder's live-OR-explicitly-marked policy to the 9 capabilities moved by the strangler migration.
The ArtifactMove plan is committed at `specs/reorg/catalog-reorg-rekey-move-plan.json` (49 pairs;
reached by the existing ADR-0563 `specs/reorg/` reachability-registry seed). Rekeyed records
include `registry/catalog/flags-server.yaml` (from `oya-flags.yaml`) and
`registry/catalog/observability-aggregate.yaml` (from `oya-cloud-observability-domain.yaml`,
which maps to the live `observability-aggregate` crate at `observability/core/aggregate`).
All 49 rekeyed records remain accepted unreachable debt in the total-accounting baseline; the
slo-coverage gate stays green (non-blank `slo:` scalars).

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** like the
prior moves, `oya/feature-flags/` also holds non-crate capability artifacts (contracts, slos, policy,
cedar, runbooks, dashboards, IaC, IP journeys, decisions, the per-crate catalog record above), and
the `oya/oya-flags/` placeholder holds wholly non-crate artifacts (a neutralized BUCK, a README, a
catalog stub, an SLO stub). This crates-only strangler move homes the CRATE; the non-crate artifacts
stay in place and are homed in phase-2 (task #62), so only the `crates/` subtree of
`oya/feature-flags/` is emptied. The deferred de-brand residue outside the moved crate (the
`[[bin]] name = "oya-flags"` + `oya-flags-bin` BUCK target name + the module self-name doc comments)
is the ADR-0532/0533 de-brand profile lane's scope (task #63), not this structural move; it is
gate-green and non-corrupting at the buck label / cargo name level.

**Born-accounting (ADR-0555):** the new crate dir under `flags/core/server/` is reached by the
`flags/*/*` member glob + the `flags/*/*` acyclicity glob, and owned by the subtree `flags/OWNERS`
(axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.12 entry. The move's
tracked, born-accounted artifact paths are `flags/core/server/Cargo.toml`, `flags/core/server/BUCK`,
`flags/core/server/src/lib.rs`, `flags/core/server/src/main.rs`, `flags/core/server/src/config.rs`,
`flags/core/server/src/ofrep/mod.rs`, `flags/core/server/src/grpc/mod.rs`,
`flags/core/server/src/rest/mod.rs`, `flags/core/server/src/targeting/mod.rs`,
`flags/core/server/src/evaluation/mod.rs`, `flags/core/server/src/storage/mod.rs`,
`flags/core/server/src/tenants/mod.rs`, `flags/core/server/src/observability/mod.rs`, the subtree
`flags/OWNERS`, and the committed move-plan `specs/reorg/flags-move-plan.json` (reached by the
existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.13 Ninth executed strangler move: `marketplace` capability (cloud/cloud-marketplace + oya/marketplace + oya/developer-sdk → marketplace/)

The ninth REAL codemod run homes the `marketplace` capability's five crates from THREE source dirs
(`cloud/cloud-marketplace/crates`, `oya/marketplace/crates`, `oya/developer-sdk/crates`) under the §3
placement rule, across TWO faces (`core` + `facade`, no `ports`/`adapters`): the cloud marketplace
domain homes to `marketplace/core/cloud-domain` (cargo `marketplace-cloud-domain`), the cloud
marketplace kernel to `marketplace/core/cloud-kernel` (cargo `marketplace-cloud-kernel`), the SaaS
plugin-marketplace kernel to `marketplace/core/plugin-kernel` (cargo `marketplace-plugin-kernel`), the
marketplace doc-set-scaffold to `marketplace/core/doc-set-scaffold` (cargo
`marketplace-doc-set-scaffold`), and the `oya` gate-runner developer CLI to `marketplace/facade/dev-cli`
(cargo `marketplace-dev-cli`). The de-brand drops the `oya-cloud-marketplace-`, `oya-saas-plugin-`,
`oya-marketplace-`, and `oya-dev-` forms to the capability slug `marketplace-`, matching the
§10.5..§10.12 precedent. marketplace is the marketplace / plugin-app-store / SKU ENGINE (the
`marketplace` dag node): a first-class cross-cutting sold-ness capability whose generated SKU/pricing
sell-catalog VIEW is materialized as build output over face:facade crates — marketplace owns the
marketplace ENGINE, not the catalog view.

**Face reasoning — four core + one facade, no ports/adapters (§3 "by WHAT IT IS"):** the capability
registry records marketplace as absorbing `cloud/cloud-marketplace`, `oya/marketplace`,
`oya/plugin-app-store`, and `oya/developer-sdk`. The crate-empty `oya/plugin-app-store` dir holds ZERO
crates (only non-crate artifacts: catalog, cedar, contracts, dashboards, IP journeys, runbooks, slos),
so it contributes no crate to this move; its non-crate residue is a phase-2 concern (task #62) while
the registry still ABSORBS it so it is accounted under `marketplace`. The four substrate engine crates
(the cloud marketplace domain + kernel, the SaaS plugin-marketplace kernel, the doc-set-scaffold) ARE
the marketplace substrate, so they home to `core/`. The fifth crate — the `oya` gate-runner developer
CLI — is the SOLD developer-tooling surface (a very wide leaf consumer of ~90 governance/check libs);
it is the sell-face of the developer-SDK product, so it homes to `facade/dev-cli`. Its `[[bin]]` name
`oya` (and the `fake-cargo` / `fake-verify-command` test-fixture bins) are PRESERVED — the de-brand
renames only the cargo PACKAGE name (`oya-dev-cli` → `marketplace-dev-cli`), not the `[[bin]] name`
entries, so the gate-runner binary stays `oya` (its retirement is the `cli_surface_policy` /
ADR-0532/0533 de-brand lane's scope, task #63, not this structural move). marketplace is therefore a
four-core-plus-one-facade capability at this incremental stage; the missing `ports`/`adapters` faces
would arrive only if the engine is later decomposed (acceptable per the crate-first strangler — a
capability is "crate-homed" per move and "fully homed" after phase-2).

**External dependents rewritten mechanically (ADR-0563):** exactly TWO first-party crates depend on
the moved tree, both on the plugin-marketplace kernel: `cloud-billing`'s `oya-saas-bench-app` and
`oya/application`'s `oya-saas-plugin-app`. The codemod rewrote each across all three surfaces — the
Cargo path-dependency key + `path=` value (`oya-saas-plugin-marketplace-kernel` →
`marketplace-plugin-kernel`, re-pathed to `marketplace/core/plugin-kernel`), the BUCK `//`-label
(`//cloud/cloud-marketplace/crates/oya-saas-plugin-marketplace-kernel:...` →
`//marketplace/core/plugin-kernel:marketplace-plugin-kernel`), and the Rust `use`/path segment
(`oya_saas_plugin_marketplace_kernel::` → `marketplace_plugin_kernel::`). The dev-cli move is a
depth-3 → depth-2 re-depthing of its ~90 relative `path="../../.."` workspace deps, all rewritten by
the codemod; `cargo metadata --locked` resolves clean and `buck2 build //marketplace/facade/dev-cli/...`
succeeds. The resulting path + cargo name are distinct for all five moves (`MovePlan::validate`
passes).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). marketplace is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The capability registry records `marketplace.absorbs_current_dirs` with
the capability's own top-level slug `marketplace` plus the four absorbed source dirs (the self-slug
is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE marketplace/...`; the §10.7 flags
lesson). The membership policy scan_roots + allowed_top_level_dirs gain `marketplace`, the acyclicity
policy crate_root_globs gains `marketplace/*/*` + unclassified_roots gains `marketplace`, and the root
workspace gains the `marketplace/*/*` member glob (one glob covers all five faces/leaves; ADR-0538
glob-only contract).

**Per-FILE total-accounting relabel (ADR-0563 §C2):** this move commits exactly ONE move-plan at
`specs/reorg/marketplace-move-plan.json` (the codemod's `MovePlan` bijection), and the move-manifest at
`specs/reorg/move-manifest.generated.json` is regenerated from it via `oya-reorg-codemod manifest
--plan` (registry-drift byte-bound, committed==regenerated). The dev-cli crate carries accepted
per-FILE `unjustified` total-accounting debt (its source/tests reference the gate-runner's own
historical crate paths as STRING-LITERAL gate self-paths — deferred de-brand residue, task #63); the
ADR-0563 §C2 per-FILE relabel (`relabel_existence_only_file_gate`) relabels these accepted-`unjustified`
files old→new via the manifest `file_pairs` (P1-frozen + P2-exact-absent + P3-exact-present +
injective; no content guard — sound because `unjustified` is registry-row-derived keyed by path), so
total-accounting stays GREEN with no scrub and no manual signoff door.

**Include-site embedded-asset hermeticity surfaces relabeled (ADR-0545, NOT engine-covered):** the
dev-cli crate has `include_str!`/`include_bytes!` sites, so two FROZEN hermeticity surfaces (not
covered by the rename-aware emitter — same manual-per-move class as the membership/acyclicity
baselines) were moved with it: the embedded-asset-hermeticity policy's `scan_roots` gains
`marketplace` (the include sites leave the scanned `oya` corpus → site-floor RED otherwise), and the
embedded-asset-hermeticity baseline's skip-set keys for the relocated dev-cli sites
(`tests/doc_cli.rs:1003` under `skip_non_literal_argument`; six `tests/gate_cli.rs` lines under
`skip_no_owning_target`) were relabeled `oya/developer-sdk/crates/oya-dev-cli/` →
`marketplace/facade/dev-cli/` (the path tail and line numbers are byte-identical; the set-equality
test stays green).

**Registry SSOT store de-branded in lockstep (registry/stores):** the five moved crates are
registry-tracked, so each per-crate KEY in the registry SSOT store was renamed to its new crate id
(capability `marketplace`) — internal JSON key edits, not separate tracked paths, so the renames
de-brand without any reachability impact (the store FILE keeps its signed-off accounting status; its
tracked path is NOT cited here, to preserve the founder one-way-door admission). The per-crate
dependency-rationales store's `allowed_crates` references to the three crates that carry an
allowlisted external dep (`bytes`, `serde_json`, `toml`) were renamed to the new crate ids in
lockstep, keeping each list sorted.

**Per-crate catalog/SLO records retained at old stems (crate-first incremental, task #62):** like the
flags/cell/storage/gateway precedent, the per-crate `registry/catalog/*.yaml` SLO-catalog records for
the moved crates STAY at their old stems and are homed in phase-2. They are accepted unreachable debt
in the frozen merge-base total-accounting baseline, so leaving them in place is gate-green; RENAMING
them would mint NEW unreachable tracked paths (the move-plan→manifest relabel only relocates files
UNDER the moved crate dirs, not the sibling catalog records), which total-accounting blocks on. The
slo-coverage gate stays green at the old stems.

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** the absorbed
dirs (`cloud/cloud-marketplace/`, `oya/marketplace/`, `oya/developer-sdk/`, and the crate-empty
`oya/plugin-app-store/`) also hold non-crate capability artifacts (contracts, slos, policy, cedar,
runbooks, dashboards, IaC, IP journeys, decisions, the per-crate catalog records above). This
crates-only strangler move homes the CRATES; the non-crate artifacts stay in place and are homed in
phase-2 (task #62), so only the `crates/` subtrees are emptied. The deferred de-brand residue outside
the moved crates (the `[[bin]] name = "oya"` gate-runner binary + `OYA_*` constants + the dev-cli
source/test STRING-LITERAL gate self-paths) is the ADR-0532/0533 de-brand profile lane's scope (task
#63), not this structural move; it is gate-green and non-corrupting at the buck label / cargo name
level.

**Born-accounting (ADR-0555):** the new crate dirs under `marketplace/core/` and `marketplace/facade/`
are reached by the `marketplace/*/*` member glob + the `marketplace/*/*` acyclicity glob, and owned by
the subtree `marketplace/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json`
§10.13 entry. The move's tracked, born-accounted artifact roots are `marketplace/core/cloud-domain/`,
`marketplace/core/cloud-kernel/`, `marketplace/core/plugin-kernel/`, `marketplace/core/doc-set-scaffold/`,
and `marketplace/facade/dev-cli/` (each carrying its `Cargo.toml`, `BUCK`, and `src/` — and, for
dev-cli, its `tests/` and `tests/fixtures/` — subtree), the subtree `marketplace/OWNERS`, and the
committed move-plan `specs/reorg/marketplace-move-plan.json` (reached by the existing ADR-0563
`specs/reorg/` reachability prefix).

#### §10.14 Tenth executed strangler move: `compliance` capability (oya/compliance/crates → compliance/) — first doctrine-clean SLO co-move

The tenth REAL codemod run homes the `compliance` capability's seven crates from one source dir
(`oya/compliance/crates`) under the §3 placement rule, across TWO faces (`core` + `ports`, no
`adapters`/`facade`): the DLP domain homes to `compliance/core/dlp` (cargo `compliance-dlp`), the DSR
domain to `compliance/core/dsr` (cargo `compliance-dsr`), the e-discovery domain to
`compliance/core/ediscovery` (cargo `compliance-ediscovery`), the retention domain to
`compliance/core/retention` (cargo `compliance-retention`), the retention-DSR domain to
`compliance/core/retention-dsr` (cargo `compliance-retention-dsr`), the trust-portal domain to
`compliance/core/trust-portal` (cargo `compliance-trust-portal`), and the DSR cascade-execute usecase
to `compliance/ports/dsr-usecase` (cargo `compliance-dsr-usecase`). The de-brand drops the legacy
`oya-`/`-domain`/`-usecase` forms to the capability slug `compliance-`, matching the §10.5..§10.13
precedent. compliance is the compliance + governance-evidence substrate (the `compliance` dag node):
regulatory packs, data-class registry, DPIA/threat-model evidence, audit-pack generation.

**Face reasoning — six core + one ports, no adapters/facade (§3 "by WHAT IT IS"):** the six DOMAIN
crates (DLP, DSR, e-discovery, retention, retention-DSR, trust-portal) are the compliance substrate
engine, so they home to `core/`; the DSR cascade-execute USECASE crate is the inbound application port
(the use-case orchestration over the DSR domain), so it homes to `ports/` (a `ports/dsr-usecase → core/dsr`
downward edge — the legal direction, the iac-rest / storage / cell precedent of §10.6/§10.9/§10.10).
The missing `adapters`/`facade` faces would arrive only if the engine grows an external-system adapter
or a sold surface (acceptable per the crate-first strangler — a capability is "crate-homed" per move
and "fully homed" after phase-2).

**Cross-capability dependency preserved (NOT broken, NOT duplicated):** `compliance/core/trust-portal`
depends on the cloud-network residency domain (`cloud/cloud-network/crates/oya-residency-domain`), an
edge that crosses out of the capability into a not-yet-moved capability. The codemod left the
cross-capability dependency's package key, `path=`, BUCK `//`-label, and Rust `use` segment UNCHANGED
(residency is not in this move's plan), so the edge keeps resolving against its current home; it will
relabel to its capability-rooted form when the network capability is later homed — there is no
duplicate residency crate and no broken edge (the cross-tree edge resolves under both `cargo metadata
--locked` and `buck2 build //compliance/core/trust-portal/...`).

**External dependents:** NONE. No first-party crate outside the seven depends on the moved tree (the
only inbound edge is residency, which is OUTBOUND from compliance); the internal cross-crate edges
(`dsr-usecase → dsr`, `ediscovery → retention`, `retention-dsr → retention`, `trust-portal → dsr`) were
rewritten mechanically by the codemod across all three surfaces — the Cargo path-dependency key +
`path=` recompute, the BUCK `//`-label, and the Rust `use`/path segment. The resulting path + cargo
name are distinct for all seven moves (`MovePlan::validate` passes).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). compliance is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The capability registry records `compliance.absorbs_current_dirs` with the
capability's own top-level slug `compliance` plus the absorbed source dirs (the self-slug is required
or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE compliance/...`; the §10.12 flags lesson). The
membership policy scan_roots + allowed_top_level_dirs gain `compliance`, the acyclicity policy
crate_root_globs gains `compliance/*/*` + unclassified_roots gains `compliance`, and the root workspace
gains the `compliance/*/*` member glob (one glob covers all seven faces/leaves; ADR-0538 glob-only
contract).

**SLO co-move executed IN the move (doctrine-clean, NOT deferred to phase-2):** unlike §10.5..§10.13
(which homed crates only and left the per-capability SLOs for the PR-B backfill), this move co-moves the
thirteen promotion-gating SLOs `oya/compliance/slos/*.openslo.yaml` → `compliance/observability/slos/`
in the SAME move, via the codemod `ArtifactMove` (content-preserving wholesale `git mv`, no in-file
rewrite), per the ADR-0139 SLO-home convention. The committed move-plan therefore carries one
`ArtifactMove` (`oya/compliance/slos` → `compliance/observability/slos`) alongside the seven crate
moves; the regenerated move-manifest carries the SLO file pairs so the ADR-0563 path-keyed relabel +
the total-accounting baseline follow the relocated SLOs old→new. The `compliance/observability` SLO-data
subtree is added to the root `[workspace].exclude` (the `compliance/*/*` member glob would otherwise
match `compliance/observability/slos`, a non-crate dir with no `Cargo.toml`, and make cargo error — the
existing per-cap SLO-exclude class). The slo-coverage gate stays green at the new home.

**Catalog re-key executed IN the move (same pattern as PR-C1 #749):** all seven crates have
`registry/catalog/*.yaml` records carrying `slo:` rows (the original brief's "no records exist"
assumption was wrong; verified post-codemod). After the crate rename (`oya-dlp-domain` →
`compliance-dlp`, etc.), the pre-existing `registry/catalog/oya-*.yaml` filenames no longer match any
live workspace `[package].name`, which would RED the `catalog-liveness` gate
(`catalog_record_no_live_crate_unmarked`) and the `slo-coverage` gate
(`slo_row_no_live_crate_unmarked`) for all seven records. The move-plan therefore carries seven
additional `ArtifactMove` entries re-keying each record to the de-branded live crate-id filename
(content-preserving `git mv`; no in-file rewrite needed because the file content does not embed the
filename stem as a key): `registry/catalog/oya-dlp-domain.yaml` → `registry/catalog/compliance-dlp.yaml`,
`oya-dsr-domain.yaml` → `compliance-dsr.yaml`, `oya-dsr-usecase.yaml` → `compliance-dsr-usecase.yaml`,
`oya-ediscovery-domain.yaml` → `compliance-ediscovery.yaml`, `oya-retention-domain.yaml` →
`compliance-retention.yaml`, `oya-retention-dsr-domain.yaml` → `compliance-retention-dsr.yaml`,
`oya-trust-portal-domain.yaml` → `compliance-trust-portal.yaml`. Both gates stay green after the
re-key (all seven records bind to a live workspace crate-id).

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** the absorbed
dir (`oya/compliance/`) also holds non-crate capability artifacts (contracts, policy, cedar, runbooks,
dashboards, IaC, IP journeys, capabilities, scorecards, the manifest). This move homes the CRATES and
co-moves the SLOs; the other non-crate artifacts stay in place and are homed in phase-2 (task #62).

**Born-accounting (ADR-0555):** the seven new crate dirs under `compliance/core/` and
`compliance/ports/` are reached by the `compliance/*/*` member glob + the `compliance/*/*` acyclicity
glob, and owned by the subtree `compliance/OWNERS` (axis-cloud-platform) seeded via a
`specs/reachability-registry.json` §10.14 entry; the same OWNERS marker is breadth-unlimited (ADR-0555)
so it also covers the co-moved `compliance/observability/slos/` SLO subtree. The move's tracked,
born-accounted artifact roots are `compliance/core/dlp/`, `compliance/core/dsr/`,
`compliance/core/ediscovery/`, `compliance/core/retention/`, `compliance/core/retention-dsr/`,
`compliance/core/trust-portal/`, and `compliance/ports/dsr-usecase/` (each carrying its `Cargo.toml`,
`BUCK`, and `src/` — and, for `compliance/ports/dsr-usecase`, its `tests/` subtree), the co-moved SLO
subtree `compliance/observability/slos/`, the subtree `compliance/OWNERS`, the seven re-keyed catalog
records `registry/catalog/compliance-dlp.yaml`, `registry/catalog/compliance-dsr.yaml`,
`registry/catalog/compliance-dsr-usecase.yaml`, `registry/catalog/compliance-ediscovery.yaml`,
`registry/catalog/compliance-retention.yaml`, `registry/catalog/compliance-retention-dsr.yaml`,
and `registry/catalog/compliance-trust-portal.yaml` (reached by the existing `registry/catalog/`
reachability prefix), and the committed move-plan `specs/reorg/compliance-move-plan.json` (reached by
the existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.15 Eleventh executed strangler move: `console` capability (oya/ops/crates → console/) — four-face cell move with catalog re-key

The eleventh REAL codemod run homes the `console` capability's nine crates from one source dir
(`oya/ops/crates`) under the §3 placement rule, across all FOUR faces (`ports` + `core` + `adapters` +
`facade`), spread over TWO product cells (`docs-portal`, four crates; `workspace-shell`, five crates).
console is the operator/tenant console-shell substrate (the `console` dag node): one platform-owned
shell that mounts every product surface, the sole token broker, replacing the operator CLIs. The two
cells are the ops live-introspection docs portal (ADR-0066) and the ops workspace shell (ADR-0067) that
mounts every ops µservice's surface.

**Naming scheme (cargo == de-branded path-tail, all nine unique):** two crates share a cell name
(`docs-portal`, `workspace-shell`) across faces, so the LEAF dir name encodes the role to stay unique
while keeping cargo == path-tail. The scheme is path `console/<face>/<cell>-<role>` ↔ cargo
`console-<cell>-<role>`, de-branded (the legacy `oya-`/`ops-` forms drop to the `console-` capability
slug). The nine: the docs-portal kernel → `console/ports/docs-portal-kernel`
(cargo `console-docs-portal-kernel`), the docs-portal usecase → `console/core/docs-portal-usecase`
(`console-docs-portal-usecase`), the docs-portal adapter → `console/adapters/docs-portal-adapter`
(`console-docs-portal-adapter`), the docs-portal rest → `console/facade/docs-portal-rest`
(`console-docs-portal-rest`); the workspace-shell kernel → `console/ports/workspace-shell-kernel`
(`console-workspace-shell-kernel`), the workspace-shell usecase → `console/core/workspace-shell-usecase`
(`console-workspace-shell-usecase`), the workspace-shell adapter → `console/adapters/workspace-shell-adapter`
(`console-workspace-shell-adapter`), the workspace-shell rest → `console/facade/workspace-shell-rest`
(`console-workspace-shell-rest`), and the workspace-shell app (composition root) →
`console/facade/workspace-shell-app` (`console-workspace-shell-app`). All nine leaf dirs and all nine
cargo names are distinct (`MovePlan::validate` passes), and each cargo name equals its de-branded
path-tail EXACTLY (the target-parity + cargo-prefix relabel binding).

**Face reasoning — ports/core/adapters/facade by WHAT EACH IS (§3), verified against the REAL dep
direction (NOT the design-sweep's inverted kernel→ports/usecase→core gloss):** the per-cell `kernel`
crate carries ZERO dependencies and is self-described as the "port traits + types" layer (it DEFINES
the `ManifestPort`/`LiveFeedPort`/`SurfaceCatalogPort` boundary traits + the domain types, with "no
outbound I/O, no framework deps; adapter + runtime crates implement the ports"), so each kernel homes to
`ports/` (the boundary the rest of the cell depends INWARD on). The per-cell `usecase` crate is the
application use-case orchestration layer that "depends only inward on the kernel", so it homes to
`core/`. This is the hexagonal direction, NOT an inversion: the application core (`core/<cell>-usecase`)
depends on the port abstractions (`ports/<cell>-kernel`) it orchestrates — a `core → ports` edge that
is the legal downward direction (the iac-rest / storage / cell / compliance precedent of
§10.6/§10.9/§10.10/§10.14). The `adapter` crate projects the kernel types onto the OpenAPI wire schema,
so it homes to `adapters/`; the `rest` crate is the framework-free REST boundary and the `app` crate is
the hyper composition root, so both home to `facade/` (the cell's delivery surface). The dep DAG is
acyclic by construction (kernel ← usecase ← adapter ← rest ← app, all pointing toward the kernel
boundary), and because all nine are ONE `console` capability node the intra-capability face edges raise
no service→service / S-rank acyclicity violation.

**External dependents:** NONE. No first-party crate outside the nine depends on the moved tree, and
there are no cross-capability outbound edges (the only outbound deps are the shared HTTP substrate
`libs/oya-http-{router,middleware}-kernel` + `libs/oya-http-runtime-hyper-adapter`, consumed from
`libs/` by the workspace-shell app — those stay in place, and their relative `path=`/BUCK `//`-label
were recomputed by the codemod against the new app home). The internal cross-crate edges
(`docs-portal-usecase → docs-portal-kernel`, `docs-portal-adapter → docs-portal-kernel`,
`docs-portal-rest → {adapter,usecase,kernel}`, and the symmetric workspace-shell edges plus
`workspace-shell-app → {kernel,usecase,adapter,rest}`) were rewritten mechanically by the codemod across
all three surfaces — the Cargo path-dependency key + `path=` recompute, the BUCK `//`-label, and the
Rust `use`/path segment.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true`
not null, `clean=true`). console is NOT a violation source (zero entries in the acyclicity frozen
baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints carry
0 burn-down / 0 regression. The capability registry records `console.absorbs_current_dirs` with the
capability's own top-level slug `console` (the self-slug is required or the membership gate REDs
`MEM-NEW-UNMAPPED-CRATE console/...`; the §10.12 flags lesson) plus the absorbed source dirs. The
membership policy scan_roots + allowed_top_level_dirs gain `console`, the acyclicity policy
crate_root_globs gains `console/*/*` + unclassified_roots gains `console`, and the root workspace gains
the `console/*/*` member glob (one glob covers all nine faces/leaves; ADR-0538 glob-only contract). No
SLO subtree exists under the absorbed source dir, so there is NO SLO co-move and NO
`console/observability` exclude (unlike §10.14).

**Catalog re-key executed IN the move (same pattern as §10.14):** all nine crates have
`registry/catalog/*.yaml` records carrying `slo:` rows (the original brief's scout "no records exist"
assumption was wrong; verified pre-move — the gates, not the scout, are authoritative). After the crate
rename, the pre-existing legacy `registry/catalog/*.yaml` filenames no longer match any live workspace
`[package].name`, which would RED the `catalog-liveness` gate (`catalog_record_no_live_crate_unmarked`)
and the `slo-coverage` gate (`slo_row_no_live_crate_unmarked`) for all nine records. The move-plan
therefore carries nine additional `ArtifactMove` entries re-keying each record to the de-branded live
crate-id filename (content-preserving `git mv`; no in-file rewrite needed because the file content does
not embed the filename stem as a key): each legacy `oya-ops-<cell>-<role>.yaml` re-keys to
`registry/catalog/console-<cell>-<role>.yaml`. Both gates stay green after the re-key (all nine records
bind to a live workspace crate-id).

**Non-crate capability artifacts retained in place (crate-first incremental, task #62):** this move homes
the CRATES and re-keys the catalog records; any other non-crate artifacts of the absorbed dir stay in
place and are homed in phase-2 (task #62).

**Born-accounting (ADR-0555):** the nine new crate dirs under `console/ports/`, `console/core/`,
`console/adapters/`, and `console/facade/` are reached by the `console/*/*` member glob + the
`console/*/*` acyclicity glob, and owned by the subtree `console/OWNERS` (axis-cloud-platform) seeded via
a `specs/reachability-registry.json` §10.15 entry (breadth-unlimited per ADR-0555, covering the whole
console subtree). The move's tracked, born-accounted artifact roots are
`console/ports/docs-portal-kernel/`, `console/core/docs-portal-usecase/`,
`console/adapters/docs-portal-adapter/`, `console/facade/docs-portal-rest/`,
`console/ports/workspace-shell-kernel/`, `console/core/workspace-shell-usecase/`,
`console/adapters/workspace-shell-adapter/`, `console/facade/workspace-shell-rest/`, and
`console/facade/workspace-shell-app/` (each carrying its `Cargo.toml`, `BUCK`, and `src/` — and, for the
app, its `src/main.rs`), the subtree `console/OWNERS`, the nine re-keyed catalog records
`registry/catalog/console-docs-portal-kernel.yaml`, `registry/catalog/console-docs-portal-usecase.yaml`,
`registry/catalog/console-docs-portal-adapter.yaml`, `registry/catalog/console-docs-portal-rest.yaml`,
`registry/catalog/console-workspace-shell-kernel.yaml`,
`registry/catalog/console-workspace-shell-usecase.yaml`,
`registry/catalog/console-workspace-shell-adapter.yaml`,
`registry/catalog/console-workspace-shell-rest.yaml`, and
`registry/catalog/console-workspace-shell-app.yaml` (reached by the existing `registry/catalog/`
reachability prefix), and the committed move-plan `specs/reorg/console-move-plan.json` (reached by the
existing ADR-0563 `specs/reorg/` reachability prefix).

## Consequences

**Positive.** One home per capability (path = namespace = buck2 label root); the run/sell seam is a
port boundary inside one capability instead of a cross-root scatter, so the
ports-designed-for-owned-stack litmus is structurally answerable; hyperscaler-fidelity (Google
`//capability` shape); the closed registry + membership lint make a junk-drawer structurally
impossible; `base/` admission-gating prevents a util dumping ground; the sell-catalog is a generated
view so "sold-ness" never duplicates code; the shape is mechanically gate-enforced so it cannot
drift.

**Negative / cost.** A repo-wide reorganization (Phase 0 + per-capability strangler PRs) of the same
class as ADR-0512's consolidation — it MUST run as dedicated post-acceptance changes on a stable
tree (the ADR-0512 process rule applies), never in a PR drain. New lints to build and maintain
(dep-lint, membership lint). The coarse default will draw split-out pressure that §7 deliberately
makes expensive (an OWNERS boundary + a clean port seam + an ADR amendment).

**Neutral.** Package names, the dependency graph, and the one-version policy are unchanged by the
shape decision; the move is mechanical + manifest-level (paths + faces), not a code-behavior change.
This ADR is documentation + data (the registry) only; it ships no runnable crate.

## Alternatives considered

- **Tier-first (`substrate/ product/ service-cell/`).** Scored 3/6/4/5, REJECTED: zero hyperscaler
  source-tree precedent; re-erects the directory split ADR-0245 forbids; cannot represent a single
  engine that spans tiers via its faces.
- **Keep the `{oya,cloud}/` runner/seller split (status quo).** Rejected: sorts by who runs/sells,
  not what a system is; scatters one capability across both roots; no hyperscaler precedent for a
  run-vs-sell top split; cannot host the run/sell port seam inside one home.
- **Open capability set (no closed registry).** Rejected: an open set degrades into a junk-drawer
  exactly as the membership lint exists to prevent; the closed enum is what makes capability-first
  mechanically safe.
- **Fine-grained capabilities (one per current dir).** Rejected by the founder coarse/Conway
  ruling: a capability is a two-pizza ownership boundary; fragmentation re-creates the scatter the
  reorg removes. Splits are narrow and ADR-gated (§7).
- **`product/` directory for sold surfaces.** Rejected: sold-ness is a face + a generated catalog
  view, not a directory; billing/tenancy/marketplace are first-class capabilities.

## Precedent

Google source tree (`//base`, `//net`, `//storage`, `//compute` — capability-rooted; a system's
name is its path; `//base` is admission-disciplined, not a util drawer). AWS two-pizza
service-per-team (the service is the ownership + naming unit; 2002 Bezos API mandate). Microsoft
Azure SDK `sdk/<service>/`. Meta fbcode (domain-rooted top, shared infra below). The clean-arch
ports/adapters seam (Cockburn hexagonal architecture; ADR-0550 D1 kernel/adapter/app) realized as
the within-capability faces. The closed-enum-as-data discipline mirrors ADR-0280's `tier_subtype`
enum and ADR-0245's tier facet.

## Phase-0 implementation artifacts

Phase-0 of this ADR (per-service tier metadata schema + born-blocking coverage gate) introduces the
following tracked artifacts, each justified by this decision (ADR-0562) together with ADR-0536 and
ADR-0245:

- specs/microservice-tier-classification.json — the generated projection: services → tier/tier_subtype/dr_tier, referenced by specs/platform-architecture.json microservice_taxonomy.tier_classification_table_ref
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/BUCK — buck2 build targets for the born-blocking tier-field-coverage gate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/Cargo.toml — Cargo manifest for the born-blocking tier-field-coverage gate crate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/src/lib.rs — pure kernel + collector for the tier-field-coverage gate (ADR-0562/ADR-0536/ADR-0245)
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/src/main.rs — binary entry point for the tier-field-coverage gate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/src/tests.rs — unit tests for the tier-field-coverage gate kernel
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/tests/tier_field_coverage.rs — integration tests including live-corpus born-blocking-green test
- cloud/cloud-ci/gates/oya-cloud-ci-tier-field-coverage-app/tier-field-coverage-policy.json — policy DATA for the tier-field-coverage gate (enum allowlists, governed roots, minimum manifest count)

Phase-0 also introduces the tier-DEPENDENCY acyclicity gate (ADR-0245/ADR-0280/ADR-0562): the
enforcement surface that asserts the ADR-0245 cross-tier dependency rules + the ADR-0280
intra-substrate S-rank rule + a Tarjan cycle backstop over the REAL crate dependency graph read from
BOTH cargo (path-deps + workspace membership) AND buck (deps/visibility). Because the pre-move tree
carries pre-existing substrate-inversions (the very debt the reorg fixes), the gate is born-ADVISORY
against a FROZEN baseline and enforces NO REGRESSION; it flips to fully blocking when the baseline
burns down to zero. Its tracked artifacts, each justified by this decision (ADR-0562) together with
ADR-0245 and ADR-0280:

- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/BUCK — buck2 build targets for the born-advisory tier-dependency-acyclicity gate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/Cargo.toml — Cargo manifest for the tier-dependency-acyclicity gate crate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/src/lib.rs — pure kernel + cargo/buck dep-graph collector + tier-rule/S-rank/Tarjan evaluator + frozen-baseline split (ADR-0245/ADR-0280/ADR-0562)
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/src/main.rs — binary entry point + baseline re-freeze (--emit-baseline) for the tier-dependency-acyclicity gate
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/src/tests.rs — unit tests for the tier-dependency-acyclicity gate kernel
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/tests/tier_dependency_acyclicity.rs — integration tests: live-corpus zero-regression GREEN + RED wrong-tier fixture + burn-down fixture
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/tests/fixtures/red-substrate-to-product.json — RED fixture: a synthetic substrate→product edge the gate must fail closed
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/tests/fixtures/burn-down.json — burn-down fixture: a removed baselined inversion the gate must keep green
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/tier-dependency-acyclicity-policy.json — policy DATA for the tier-dependency-acyclicity gate (governed crate-root globs, tier'd service roots, unclassified meta roots, S-rank order, enforcement mode)
- cloud/cloud-ci/gates/oya-cloud-ci-tier-dependency-acyclicity-app/tier-dependency-acyclicity-baseline.json — the FROZEN known-debt baseline: the pre-move tier-dependency violations the reorg strangler burns down (the burn-down target)

Phase-0 also introduces the §6 MEMBERSHIP lint (the anti-junk-drawer authority) — born-advisory with
a frozen unmapped baseline, enforcing no-regression (no NEW unmapped crate, no NEW top-level dir
outside the closed set) and the base/-admission rule — plus the registry's `membership_lint_coverage`
extension that closes the per-crate mapping over the whole tree. These tracked artifacts are each
justified by this decision (ADR-0562 §6) together with ADR-0536, ADR-0280, and ADR-0512:

- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/BUCK — buck2 build targets for the born-advisory capability-membership lint
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/Cargo.toml — Cargo manifest for the capability-membership lint crate
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/src/lib.rs — pure kernel + crate collector for the capability-membership lint (ADR-0562 §6/ADR-0280/ADR-0512)
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/src/main.rs — binary entry point for the capability-membership lint
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/src/tests.rs — unit RED/GREEN fixtures for the capability-membership kernel (crate in no/two capabilities, new top-level dir, base/-admission, frozen-baseline advisory)
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/tests/capability_membership.rs — integration tests including the live-corpus born-advisory-green self-test and on-disk RED fixtures
- cloud/cloud-ci/gates/oya-cloud-ci-capability-membership-app/capability-membership-policy.json — policy DATA for the capability-membership lint (gate id, registry pointer, scan roots, closed meta-directory + top-level set, ignored build-artifact dirs, minimum crate count)
