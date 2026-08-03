---
id: ADR-0562
title: "Capability-first repo organization + the closed capability registry — the ratified hyperscaler source-tree shape every reorg lane implements"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-06-14
door: one-way
owner: founder
supersedes: [ADR-0550]
superseded_by: []
amended_by: [ADR-0615]
depends_on: [ADR-0280, ADR-0512]
amends:
  - ADR-0536-hyperscaler-grounded-substrate-decision-matrix.md (its sixteen domains seed the closed capability registry; its enforcement gains face-direction + membership checks)
  - ADR-0512-canonical-monorepo-pattern.md (capability-first layout supersedes the cloud/oya/libs root assumption; libs/ dissolves into capability homes + base/; the kuberos-kernel nested-workspace carve-out)
  - ADR-0280-substrate-of-substrate-dependency-doctrine.md (DAG nodes use de-branded capability names; D-1 is the canonical substrate bootstrap ordering source)
related: [ADR-0131, ADR-0132, ADR-0245, ADR-0510, ADR-0520, ADR-0532, ADR-0533, ADR-0536, ADR-0537, ADR-0543, ADR-0547, ADR-0549, ADR-0555]
related_specs:
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
  - /specs/platform-architecture.json
milestone: W0
---

# ADR-0562: Capability-first repo organization + the closed capability registry

## Status

**Accepted — 2026-07-10 (founder ratification; shape authored + founder-ratified 2026-06-14; door: one-way).**

The reorganization shape was ratified by the founder on 2026-06-14 (cost-agnostic, four-axis
adversarial determination — hyperscaler-fidelity / dogfood-purity / maintainability / clean-arch —
with doubt-driven verification) and formally **Accepted on 2026-07-10** under the founder's
autonomous-drive delegation, riding cross-artifact propagation in one atomic batch alongside ADR-0615
(its boundary-rulings amendment). This ADR is the durable record of that decision and the spec every
later reorg lane implements; it does NOT move any crates. ADR-0328 remains the canonical sequence
authority; the migration contract in §10 is reference, not execution. The Accept propagates the
supersession of **ADR-0550** (repository layout doctrine, superseded in full) and the scoped
amendment of **ADR-0512** (its layout clause only).

## Context

The repository is a single Rust monorepo (~700+ crates) under hyperscaler-grade discipline. Three
written layout authorities are in force and partially in tension:

- **ADR-0512** (Accepted) — the canonical monorepo pattern: one root Cargo workspace, crate =
  bounded context, service dirs as pure containers, the Buck2 graph as the parallelism/containment
  substrate. Its layout clause roots service code at `{oya,cloud}/<service>/` and shared code at
  `libs/<lib>/`.
- **ADR-0550** (Proposed at authoring; **Superseded by this ADR on Accept**) — the repository layout
  doctrine: `{oya,cloud}/<service>/` colocation + the kernel/adapter/app clean-architecture seams +
  the `libs/` charter (rule-of-two shared root).
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

The top-level source tree is organized **by capability** (the primary axis), with six meta/floor
directories that are not capabilities:

```
kernel/        # rung 0: kuberos no_std kernel + sysroot (recursion floor; its own excluded workspace — the ADR-0512 carve-out, §8 Fork 2)
os/            # rung 1: cloud-os node OS (Talos-class)
base/          # Google //base: irreducible cross-capability primitives. ADMISSION-GATED (>=3 capability consumers AND strictly below all of them in the ADR-0280 DAG). NOT a util/ junk-drawer.
governance/    # meta, OFF the runtime ladder: ADRs, specs, policy-as-data, the capability registry, the dep-lint authority, the masterplan
build/         # meta, OFF the runtime ladder: buck2 prelude, toolchains, reindeer, CI engines, AND the generated sell-catalog (SKU/pricing) VIEW (build output; owns zero crates)
third-party/   # meta, OFF the runtime ladder: reindeer-vendored third-party crate sources — TOP-LEVEL (amended by ADR-0615, founder 2026-07-10; already the live buck2 cell root `third-party//`). Owns zero first-party crates.
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

- **`iam/`** absorbs identity + oya-identity + consent(-graph) + tenant-rbac as sub-modules (the
  cloud IdP substrate is `core/`; the product-shared identity that CONSUMES it is `facade/`).
  **Amended by ADR-0615 (founder 2026-07-10):** the Cedar-backed **PBAC+ReBAC decision plane
  (`policy`)** is EXTRACTED from this collapse into its own standalone capability (§7 split;
  registry v1.1.0), reversing the coarse iam-absorbs-policy grouping — iam keeps identity /
  credentials / consent / tenant-RBAC and produces the verified principal that `policy/` evaluates.
  Precedent: AWS IAM ↔ Verified Permissions and Google IAM ↔ Zanzibar are distinct planes; ADR-0280
  §D-13.D marks `policy` "Standalone ✓" (G authoring/signing/distribution · C0 per-cell PDP).
- **`ci/`** absorbs cloud-ci + ci-controller + ci-tide + ci-webhook-gateway.
- **`compute/`** is ONE engine (vm + k8s-on-compute + functions) with facade sub-surfaces.
- **`k8s/`** is the owned control-plane (`core/`) + managed-k8s (`facade/`); the four
  `managed-k8s-*` dirs become facade sub-modules.
- **`secrets/`** is KMS + secrets sharing the crypto-root recursion break.
- **`billing/`** absorbs metering + billing + tax + finops/cost; **`marketplace/`** absorbs
  marketplace + plugin/app-store + SKU catalog data.

**Cross-cutting sold-ness (billing, tenancy, marketplace) are FIRST-CLASS capabilities, NOT a
`product/` junk-drawer.** The full set + one-line charter per capability + the absorbed current dirs
+ the seed domains live in `specs/capability-registry.json`. The capabilities at v1.0.0 (23) are:
`cell`, `iam`, `tenancy`, `secrets`, `audit`, `observability`, `data`, `storage`, `compute`, `k8s`,
`network`, `gateway`, `messaging`, `intelligence`, `workflow`, `ci`, `iac`, `billing`,
`marketplace`, `console`, `compliance`, `comms`, `flags`. **At v1.1.0 (ADR-0615, founder
2026-07-10) the set is 24:** `policy` is extracted from `iam` as the 24th capability, mapping 1:1
to the pre-existing `policy-engine` DAG node (ADR-0280 §D-13) — the 24 capabilities then map 1:1 to
the 24 DAG nodes.

### §3 The deterministic placement rule (first match wins)

For any crate or artifact, apply in order; the first match decides the home:

1. the kuberos kernel → `kernel/`; the node OS → `os/`;
2. ADRs / specs / policy-as-data / the capability registry / the dep-lint authority → `governance/`;
3. buck2 prelude / toolchains / CI engines / the generated catalog view → `build/`; reindeer-vendored
   third-party crate sources → **`third-party/`** (a TOP-LEVEL meta dir per the ADR-0615 amendment,
   founder 2026-07-10 — it is already the live `third-party//` buck2 cell root, so this aligns the
   ADR to the build graph);
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
  `kernel|os|base|governance|build|third-party|app` **FAILS** (the closed-set guarantee;
  `third-party/` added as a top-level meta dir per the ADR-0615 amendment — when a physical
  `third-party/` dir lands, the membership-lint policy's `allowed_top_level_dirs` gains it in the
  same move);
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
**ADR-0550 is SUPERSEDED in full** by this ADR (`supersedes: [ADR-0550]`; ADR-0550
`superseded_by: [ADR-0562]`, status Superseded): its `{oya,cloud}/<service>/` colocation root and
its `libs/` rule-of-two charter are replaced by capability-first (one top dir per capability +
`base/`); the clean-architecture kernel/adapter/app seams it defined are **preserved** as the
WITHIN-capability shape (the §4 face rule + the `base/` admission rule), never in tension with
capability-first.
**ADR-0537** (dogfood ladder, Proposed) is **implemented by this shape**; its §2 tier-dependency
lint enforcement gates on its own founder sign-off, and the §6 membership lint promotes on the same
sign-off.

**Relationship to ADR-0132 (no-grouping forward-policy) — a coarse capability is NOT a
bundle-µservice.** ADR-0132 forbids new multi-concern *deployable grouping SERVICES* (bundle /
vertical / suite wrappers): a shipped microservice stays flat and single-concern. An ADR-0562
`<capability>/` dir is an **ownership + namespace unit** (one two-pizza boundary, the buck2 label
root), NOT a deployable service — its member crates each remain single-concern and are deployed
individually (`core/` engines, `facade/` surfaces). The coarse-by-default rule (§7) collapses
*ownership*, never *deployables*; the composition of 2+ capabilities into a shippable product is
exactly what `app/<product>/` is for (§3 rule #5), which is itself the ADR-0132-conformant place for
tenant-facing composition. The two doctrines are therefore complementary: ADR-0132 governs the
service (deployable) axis; ADR-0562 governs the source-tree (ownership) axis. (Promoted here from
the frontmatter `related` set to an explicit reconciliation clause.)

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
`tools/oya-reorg-codemod-app/src/oracle.rs`, `tools/oya-reorg-codemod-app/src/manifest.rs`,
`tools/oya-reorg-codemod-app/src/main.rs`,
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

The PACK-001 regional-pack manifest gate fixture slice later extends the same crate with the
born-accounted surfaces `cell/core/regional-pack/src/manifest.rs`,
`cell/core/regional-pack/tests/fixtures/kr/manifest.json`,
`cell/core/regional-pack/tests/fixtures/kr/canonical-base.txt`,
`cell/core/regional-pack/tests/fixtures/kr/pack-impl.txt`,
`cell/core/regional-pack/tests/fixtures/negative/canonical-base-with-jurisdiction-markers.txt`, and
`cell/core/regional-pack/tests/fixtures/negative/pack-impl-cross-pack-reference.txt` — a hermetic
crate-local manifest parser/gate kernel plus RED/GREEN fixture data; test-fixture evidence only, no
new capability dir, crate, or pack source of truth.

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

**§10.12.1 Wave-1 BUILD-slice decomposition: `flags/core/evaluation-domain` (second core crate):**
The §10.12 move homed the bundled server `flags/core/server` AS-IS (one core crate, all subsystems as
internal-module stubs, the §877 "missing faces arrive only if the engine is later decomposed"
forward-reference). The Wave-1 BUILD slice executes exactly that anticipated decomposition for the
single highest-value, fully cloud-agnostic surface: the deterministic flag-evaluation ENGINE (rule
targeting, percentage bucketing, variant resolution — the ADR-0481 `evaluation/mod.rs` TODO) is lifted
OUT of the bundled server into a SECOND `core`-face crate `flags/core/evaluation-domain` (cargo
`flags-evaluation-domain`, lib `flags_evaluation_domain`). The new crate is PURE: zero
cloud/persistence/identity/runtime deps — it does NOT pull `tokio` and depends on nothing first-party;
its evaluation is a pure function over `(Flag, EvaluationContext)`. The flag SOURCE and any
cloud/storage/identity coupling are DEFERRED behind the crate's `FlagSource` port (clean-arch
ports-in-core per ADR-0570; the port trait is DEFINED in this `core` crate, NEVER in an `adapters/`
crate, so the port-placement gate stays green). The server's `evaluation/mod.rs` subsystem becomes a
thin re-export SEAM over the domain engine, and `flags/core/server` gains a `flags-evaluation-domain`
dependency edge (the first intra-capability edge: `core/server → core/evaluation-domain`, a forward
`core→core` edge with no cycle). This stays within the §10.12 face model (still a single `core` face,
now two leaves) and needs NO members edit: the existing `flags/*/*` member glob + `flags/*/*`
acyclicity glob already cover `flags/core/evaluation-domain`, and `flags/OWNERS` (axis-cloud-platform)
already owns the whole subtree (the §10.12 reachability seed is breadth-unlimited per ADR-0555). The
slice also wires a `flags-server-unittest` `rust_test` target so the server's new seam test compiles
under buck2 (ADR-0540 cargo/buck target parity). The new born-accounted tracked artifact paths are
`flags/core/evaluation-domain/Cargo.toml`, `flags/core/evaluation-domain/BUCK`,
`flags/core/evaluation-domain/src/lib.rs`, `flags/core/evaluation-domain/src/model.rs`,
`flags/core/evaluation-domain/src/engine.rs`, `flags/core/evaluation-domain/src/bucket.rs`, and
`flags/core/evaluation-domain/src/port.rs` (each reached by the `flags/*/*` member glob, owned by
`flags/OWNERS`, justified by this ADR). The storage/cloud/identity ADAPTERS, the OFREP/gRPC/REST
faces, and the `ports`/`adapters`/`facade` face dirs remain DEFERRED (phase-2 task #62 + the
adapter-authoring lanes), consistent with the crate-first strangler "fully homed after phase-2" rule.

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

#### §10.16 Twelfth executed strangler move: `comms` capability (oya/mail + oya/messenger + oya/meet + oya/contact-center → comms/) — four-cell move with collision-aware per-service SLO co-move + catalog re-key

The twelfth REAL codemod run homes the `comms` capability's sixteen crates from FOUR source dirs
(`oya/mail/crates`, `oya/messenger/crates`, `oya/meet/crates`, `oya/contact-center/crates`) under the §3
placement rule, across all FOUR faces (`core` + `ports` + `adapters` + `facade`), spread over FOUR product
cells (`mail`, seven crates; `messenger`, seven crates; `meet`, one crate; `contact-center`, one crate).
comms is the multi-channel communications plane (the `comms` dag node): email/mail, messenger, meet, and
contact-center — the substrate notification engine plus the tenant communication surfaces.

**Naming scheme (cargo == de-branded path-tail, all sixteen unique):** the mail and messenger cells carry a
`mailbox-store` / `message-stream` bounded-context segment that would path-double under the capability slug,
so the LEAF dir name encodes a compacted `<cell>-<context>-<role>` form to stay unique while keeping cargo ==
path-tail. The scheme is path `comms/<face>/<leaf>` ↔ cargo `comms-<leaf>`, de-branded (the legacy
`oya-mail-`/`oya-messenger-`/`oya-meet-`/`oya-contact-center-` forms drop to the `comms-` capability slug).
The sixteen: the mail domain → `comms/core/mail-domain` (cargo `comms-mail-domain`), the mail mailbox-store
usecase → `comms/core/mail-mailbox-usecase` (`comms-mail-mailbox-usecase`), the mail mailbox-store app →
`comms/core/mail-mailbox-app` (`comms-mail-mailbox-app`), the mail mailbox-store api →
`comms/ports/mail-mailbox-api` (`comms-mail-mailbox-api`), the mail mailbox-store postgres adapter →
`comms/adapters/mail-mailbox-postgres` (`comms-mail-mailbox-postgres`), the mail mailbox-store grpc →
`comms/facade/mail-mailbox-grpc` (`comms-mail-mailbox-grpc`), the mail mailbox-store rest →
`comms/facade/mail-mailbox-rest` (`comms-mail-mailbox-rest`); the messenger domain →
`comms/core/messenger-domain` (`comms-messenger-domain`), the messenger message-stream usecase →
`comms/core/messenger-stream-usecase` (`comms-messenger-stream-usecase`), the messenger app →
`comms/core/messenger-stream-app` (`comms-messenger-stream-app`), the messenger message-stream api →
`comms/ports/messenger-stream-api` (`comms-messenger-stream-api`), the messenger message-stream postgres
adapter → `comms/adapters/messenger-stream-postgres` (`comms-messenger-stream-postgres`), the messenger
message-stream grpc → `comms/facade/messenger-stream-grpc` (`comms-messenger-stream-grpc`), the messenger
message-stream rest → `comms/facade/messenger-stream-rest` (`comms-messenger-stream-rest`); the meet domain →
`comms/core/meet-domain` (`comms-meet-domain`); and the contact-center voice-routing app →
`comms/facade/contact-center-voice-routing` (`comms-contact-center-voice-routing`). All sixteen leaf dirs and
all sixteen cargo names are distinct (`MovePlan::validate` passes), and each cargo name equals its de-branded
path-tail EXACTLY (the target-parity + cargo-prefix relabel binding).

**Face reasoning — ports/core/adapters/facade by WHAT EACH IS (§3), verified against the REAL dep
direction:** each cell's `domain` crate carries the bounded-context types and homes to `core/` (the substrate
engine surface); the `*-api` crate is the inbound application port (it carries only the shared
protocol-parity kernel dep) so it homes to `ports/`; the `*-usecase` crate is the application use-case
orchestration that depends inward on the domain + api, so it homes to `core/` (a legal `core → ports`
downward hexagonal edge, the iac-rest / storage / cell / compliance / console precedent of
§10.6/§10.9/§10.10/§10.14/§10.15); the `*-app` composition crate wires the adapter + api + usecase and homes
to `core/` for the mail/messenger cells (the cell's substrate composition root, depending downward on its
own `adapters/` postgres impl — an intra-capability `core → adapters` edge that, because all sixteen are ONE
`comms` capability node, projects to a `comms → comms` self-edge and raises NO service→service / S-rank
acyclicity violation); the `*-postgres` adapter projects the domain onto the postgres store and homes to
`adapters/`; the `*-grpc` / `*-rest` framework boundaries home to `facade/` (the cell's delivery surface).
The contact-center voice-routing `app` is the cell's sole crate — a delivery composition root with a custom
`[[bin]]` — so it homes to `facade/`. The dep DAG is acyclic by construction, and because all sixteen are
ONE `comms` capability node the intra-capability face edges raise no service→service / S-rank acyclicity
violation (the §10.15 console precedent).

**External dependents:** NONE. No first-party crate outside the sixteen depends on the moved tree, and there
are no cross-capability outbound edges (the only outbound deps are the shared `libs/oya-data-boundary-kernel`
+ `libs/oya-shared-{postgres-command,protocol-parity,protocol-transport,transactional-outbox,hyperscaler-metrics}-kernel`,
consumed from `libs/` — those stay in place, and their relative `path=`/BUCK `//`-label were recomputed by
the codemod against the new homes). The internal cross-crate edges (the mail cell `mailbox-usecase →
{domain,api}`, `mailbox-app → {postgres,api,usecase}`, `mailbox-grpc → {api,app}`, `mailbox-rest →
{app,api,usecase}`; the symmetric messenger cell edges; meet-domain and contact-center-voice-routing have no
intra-capability edges) were rewritten mechanically by the codemod across all three surfaces — the Cargo
path-dependency key + `path=` recompute, the BUCK `//`-label, and the Rust `use`/path segment.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null,
`clean=true`). comms is NOT a violation source (zero entries in the acyclicity frozen baseline) and the moved
crate dirs are not in the membership unmapped baseline, so both lints carry 0 burn-down / 0 regression. The
capability registry records `comms.absorbs_current_dirs` with the capability's own top-level slug `comms` (the
self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE comms/...`; the §10.12 flags lesson)
plus the absorbed source dirs (the pre-existing seed `oya/comms-email` + `oya/emergency` are retained — both
crate-empty — and are NOT a violation source; `oya/emergency` is NOT pre-reserved beyond the existing seed,
pending the open healthcare-capability question). The membership policy scan_roots + allowed_top_level_dirs
gain `comms`, the acyclicity policy crate_root_globs gains `comms/*/*` + unclassified_roots gains `comms`, and
the root workspace gains the `comms/*/*` member glob (one glob covers all sixteen faces/leaves; ADR-0538
glob-only contract).

**SLO co-move executed IN the move (doctrine-clean, collision-aware per-service subdirs):** all four source
dirs carry promotion-gating SLOs (`oya/mail/slos`, 11; `oya/messenger/slos`, 11; `oya/meet/slos`, 12;
`oya/contact-center/slos`, 13 — 47 total). A FLAT merge into `comms/observability/slos/` would clash on the
CONFIRMED cross-service basename collisions `autosharding-events.openslo.yaml` (all four), `availability.openslo.yaml`
(meet + contact-center), and `search-latency.openslo.yaml` (mail + messenger) — `MovePlan::validate`'s
dup-`new_path` fail-closed would fire. The move therefore co-moves each source SLO dir wholesale into a
PER-SERVICE subdir (`comms/observability/slos/mail/`, `.../messenger/`, `.../meet/`, `.../contact-center/`)
via four content-preserving dir `ArtifactMove`s — collision-free AND provenance-preserving (each SLO keeps its
originating-service home). This is a deliberate, collision-driven deviation from the flat `<cap>/observability/slos/`
layout of §10.5..§10.15 (which had no four-way basename clashes); the slo-coverage gate keys by catalog-record
file stem, not SLO file path, so the nesting is gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14/§10.15):** all sixteen crates have
`registry/catalog/*.yaml` records carrying `slo:` rows (verified pre-move — the gates, not a scout, are
authoritative). After the crate rename, the legacy `oya-mail-*`/`oya-messenger-*`/`oya-meet-*`/`oya-contact-center-*`
filenames no longer match any live workspace `[package].name`, which would RED the `catalog-liveness`
(`catalog_record_no_live_crate_unmarked`) and `slo-coverage` (`slo_row_no_live_crate_unmarked`) gates for all
sixteen records. The move-plan therefore carries sixteen additional file `ArtifactMove` entries re-keying each
record to the de-branded live crate-id filename (content-preserving `git mv`; no in-file rewrite — the file
content does not embed the filename stem as a key): each legacy record re-keys to
`registry/catalog/comms-<leaf>.yaml`. Both gates stay green after the re-key (all sixteen records bind to a
live workspace crate-id).

**Custom bin de-brand (codemod gap #76, hand-patched):** the contact-center voice-routing app declares a custom
`[[bin]]` / `rust_binary` named `oya-contact-center-voice-routing` (not the snake-mirror of the package name),
which the codemod's `apply` does NOT auto-de-brand (it rewrites `[package].name`/`[lib].name`/deps/labels/idents
only). The custom `[[bin]]` name (`Cargo.toml`), the `rust_binary` target name (`BUCK`), and the clap
`#[command(name = ...)]` string literal (`src/main.rs`) were hand-de-branded to `comms-contact-center-voice-routing`,
the end-state correct de-branded form (the `use oya_contact_center_voice_routing_app::...` lib-ident references
are rewritten automatically by the codemod's rust-source pass to `comms_contact_center_voice_routing`).

**Born-accounting (ADR-0555):** the sixteen new crate dirs under `comms/core/`, `comms/ports/`,
`comms/adapters/`, and `comms/facade/` are reached by the `comms/*/*` member glob + the `comms/*/*` acyclicity
glob, and owned by the subtree `comms/OWNERS` (axis-cloud-platform) seeded via a
`specs/reachability-registry.json` §10.16 entry (breadth-unlimited per ADR-0555, covering the whole comms
subtree including the co-moved per-service SLO subdirs). The move's tracked, born-accounted artifact roots are
the sixteen crate dirs (each carrying its `Cargo.toml`, `BUCK`, and `src/` — and, for the contact-center
voice-routing crate, its `src/main.rs` + `tests/`), the co-moved SLO subtree
`comms/observability/slos/{mail,messenger,meet,contact-center}/`, the subtree `comms/OWNERS`, the sixteen
re-keyed catalog records `registry/catalog/comms-*.yaml` (reached by the existing `registry/catalog/`
reachability prefix), and the committed move-plan `specs/reorg/comms-move-plan.json` (reached by the existing
ADR-0563 `specs/reorg/` reachability prefix).

#### §10.17 Thirteenth executed strangler move: `k8s` capability (cloud/managed-k8s-* → k8s/) — four-cell move with collision-aware per-service SLO co-move + catalog re-key

The thirteenth REAL codemod run homes the `k8s` capability's seventeen crates from FOUR source dirs
(`cloud/managed-k8s-cluster-lifecycle/crates`, `cloud/managed-k8s-control-plane-host/crates`,
`cloud/managed-k8s-sla-observability/crates`, `cloud/managed-k8s-tenant-quota/crates`) under the §3 placement
rule, across all FOUR faces (`core` + `ports` + `adapters` + `facade`), spread over FOUR product cells
(`cluster-lifecycle`, three crates; `control-plane-host`, five crates; `sla-observability`, four crates;
`tenant-quota`, five crates). k8s is the managed-Kubernetes plane (the `control-plane` dag node): the SOLD
managed-k8s product (facade) above the owned control-plane (core) — the kuberos→cloud-k8s ladder rung above
`os/`. The crate-empty `cloud/cloud-k8s` dir (docs/IaC only, no `Cargo.toml`) is absorbed dir-slug-only — no
crate, SLO, or global-catalog move from it (its local `cloud/cloud-k8s/catalog/` + `cloud/cloud-k8s/slos/` are
NOT the global `registry/catalog/` and stay in place for phase-2).

**Naming scheme (cargo == de-branded path-tail, all seventeen unique):** the console-proven scheme — path
`k8s/<face>/<cell>-<role>` ↔ cargo `k8s-<cell>-<role>`, de-branded (the legacy `oya-managed-k8s-` form drops to
the `k8s-` capability slug). A bare cell leaf would collide (each cell name repeats across faces and the
`kernel`/`api`/`app`/`adapter-*` roles repeat across cells), so the leaf encodes `<cell>-<role>` to stay unique
while keeping cargo == path-tail EXACTLY. The seventeen: cluster-lifecycle kernel →
`k8s/core/cluster-lifecycle-kernel` (cargo `k8s-cluster-lifecycle-kernel`), api →
`k8s/ports/cluster-lifecycle-api` (`k8s-cluster-lifecycle-api`), app → `k8s/facade/cluster-lifecycle-app`
(`k8s-cluster-lifecycle-app`); control-plane-host kernel → `k8s/core/control-plane-host-kernel`
(`k8s-control-plane-host-kernel`), api → `k8s/ports/control-plane-host-api` (`k8s-control-plane-host-api`),
adapter-inmemory → `k8s/adapters/control-plane-host-adapter-inmemory`
(`k8s-control-plane-host-adapter-inmemory`), adapter-capi → `k8s/adapters/control-plane-host-adapter-capi`
(`k8s-control-plane-host-adapter-capi`), app → `k8s/facade/control-plane-host-app`
(`k8s-control-plane-host-app`); sla-observability kernel → `k8s/core/sla-observability-kernel`
(`k8s-sla-observability-kernel`), api → `k8s/ports/sla-observability-api` (`k8s-sla-observability-api`),
adapter-inmemory → `k8s/adapters/sla-observability-adapter-inmemory`
(`k8s-sla-observability-adapter-inmemory`), app → `k8s/facade/sla-observability-app`
(`k8s-sla-observability-app`); tenant-quota kernel → `k8s/core/tenant-quota-kernel`
(`k8s-tenant-quota-kernel`), api → `k8s/ports/tenant-quota-api` (`k8s-tenant-quota-api`), adapter-inmemory →
`k8s/adapters/tenant-quota-adapter-inmemory` (`k8s-tenant-quota-adapter-inmemory`), adapter-cedar →
`k8s/adapters/tenant-quota-adapter-cedar` (`k8s-tenant-quota-adapter-cedar`), and app →
`k8s/facade/tenant-quota-app` (`k8s-tenant-quota-app`). All seventeen leaf dirs and all seventeen cargo names
are distinct (`MovePlan::validate` passes), and each cargo name equals `k8s-` + its de-branded path-tail
EXACTLY (the target-parity + cargo-prefix relabel binding).

**Face reasoning — ports/core/adapters/facade by WHAT EACH IS (§3), verified against the REAL dep direction:**
each cell's `kernel` crate carries the pure value objects / state machine and homes to `core/` (the substrate
engine surface, std+serde only); the `*-api` crate is the inbound application port (the trait + DTO seam,
path-dep inward on the kernel only) so it homes to `ports/`; the `adapter-*` crates project a port onto a
backing impl (the control-plane-host capi/inmemory adapters, the sla-observability inmemory adapter, the
tenant-quota inmemory/cedar adapters) and home to `adapters/`; the `*-app` composition roots wire api +
adapter(s) + kernel into a delivery surface (axum admin/status API + a fail-closed `[[bin]]` for the three with
a binary) and home to `facade/` — the SOLD managed-k8s product face. The cross-cell edges are all downward and
acyclic: `cluster-lifecycle-api` (ports) → `tenant-quota-api` + `control-plane-host-api` (ports); the
`cluster-lifecycle-app` (facade) → `tenant-quota-{kernel,adapter-inmemory}` + `control-plane-host-adapter-inmemory`
(core/adapters); `sla-observability-app` (facade) → `control-plane-host-api` (ports) + (dev)
`control-plane-host-adapter-inmemory` (adapters). The dep DAG is acyclic by construction, and because all
seventeen are ONE `k8s` capability node the intra-capability face edges project to a `k8s → k8s` self-edge and
raise NO service→service / S-rank acyclicity violation (the §10.15/§10.16 console/comms precedent).

**Cross-capability dep (kept intact):** the `k8s-tenant-quota-adapter-cedar` crate has an OUTBOUND dep on
`oya/identity` (the workload-authz-cedar adapter + workload-domain), reusing the Cedar RBAC substrate
(ADR-0376/0183). `iam` is NOT yet homed, so this dep continues to point at the live `oya/identity/crates/...`
path (the relative `path=` + BUCK `//`-label were recomputed by the codemod against the new
`k8s/adapters/tenant-quota-adapter-cedar` home); it relabels when iam moves in a later strangler step. It is
NOT broken or duplicated.

**External dependents:** NONE. No first-party crate outside the seventeen depends on the moved tree (only the
root `Cargo.toml` member glob referenced them, never as a dependency), and the only outbound cross-capability
edge is the tenant-quota cedar→identity dep above. k8s is NOT a violation source (zero entries in the
acyclicity frozen baseline) and the moved crate dirs are not in the membership unmapped baseline, so both lints
carry 0 burn-down / 0 regression.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null,
`clean=true`). The capability registry records `k8s.absorbs_current_dirs` with the capability's own top-level
slug `k8s` (the self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE k8s/...`; the
§10.12/§10.16 flags/comms lesson) plus the absorbed source dirs (the four `cloud/managed-k8s-*` dirs + the
crate-empty `cloud/cloud-k8s`, all pre-existing seeds, retained). The membership policy scan_roots +
allowed_top_level_dirs gain `k8s`, the acyclicity policy crate_root_globs gains `k8s/*/*` + unclassified_roots
gains `k8s`, and the root workspace gains the `k8s/*/*` member glob (one glob covers all seventeen
faces/leaves; ADR-0538 glob-only contract).

**SLO co-move executed IN the move (doctrine-clean, collision-aware per-service subdirs):** all four source
dirs carry promotion-gating SLOs (`cloud/managed-k8s-cluster-lifecycle/slos`, 1;
`cloud/managed-k8s-control-plane-host/slos`, 2; `cloud/managed-k8s-sla-observability/slos`, 2;
`cloud/managed-k8s-tenant-quota/slos`, 1 — 6 total). A FLAT merge into `k8s/observability/slos/` would clash on
the CONFIRMED cross-service basename collision `provisioning-latency.openslo.yaml` (control-plane-host +
sla-observability) — `MovePlan::validate`'s dup-`new_path` fail-closed would fire. The move therefore co-moves
each source SLO dir wholesale into a PER-SERVICE subdir (`k8s/observability/slos/cluster-lifecycle/`,
`.../control-plane-host/`, `.../sla-observability/`, `.../tenant-quota/`) via four content-preserving dir
`ArtifactMove`s — collision-free AND provenance-preserving. This mirrors the §10.16 comms per-service-subdir
deviation; the slo-coverage gate keys by catalog-record file stem, not SLO file path, so the nesting is
gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14..§10.16):** all seventeen crates have
`registry/catalog/oya-managed-k8s-*.yaml` records. After the crate rename, the legacy filenames no longer match
any live workspace `[package].name`, which would RED the `catalog-liveness`
(`catalog_record_no_live_crate_unmarked`) and `slo-coverage` (`slo_row_no_live_crate_unmarked`) gates for all
seventeen records. The move-plan therefore carries seventeen additional file `ArtifactMove` entries re-keying
each record to the de-branded live crate-id filename `registry/catalog/k8s-<leaf>.yaml` (content-preserving
`git mv`; no in-file rewrite). Both gates stay green after the re-key (all seventeen records bind to a live
workspace crate-id).

**Custom bin de-brand (codemod gap #76, hand-patched):** three of the four `*-app` crates declare a custom
`[[bin]]` / `rust_binary` named `oya-managed-k8s-<cell>` (not the snake-mirror of the package name), which the
codemod's `apply` does NOT auto-de-brand (it rewrites `[package].name`/`[lib].name`/deps/labels/idents only).
The custom `[[bin]]` name (`Cargo.toml`) and the `rust_binary` target name (`BUCK`) for cluster-lifecycle-app,
control-plane-host-app, and tenant-quota-app were hand-de-branded to `k8s-cluster-lifecycle`,
`k8s-control-plane-host`, and `k8s-tenant-quota` (no `-bin` suffix needed — none collides with a package/lib
name, unlike the §10.16 contact-center case). The sla-observability-app is lib-only (no `[[bin]]`). One
tracing-directive string literal in `tenant-quota-app/src/main.rs` (`"oya_managed_k8s_tenant_quota=info"` — a
crate-module log-target PREFIX, not a full crate ident, so the codemod's whole-ident rust-source pass left it)
was hand-de-branded to `"k8s_tenant_quota=info"`. The forward-looking (non-emitter-bound, `non_claim`-marked)
Prometheus metric names + job label in the co-moved SLOs and the stale old-crate-name references in three
doc-comments / one Cargo.toml description were hand-de-branded to the `k8s-`/`k8s_` form so the moved tree is
fully grep-clean of `oya-managed-k8s-`/`oya_managed_k8s_` tokens.

**Born-accounting (ADR-0555):** the seventeen new crate dirs under `k8s/core/`, `k8s/ports/`, `k8s/adapters/`,
and `k8s/facade/` are reached by the `k8s/*/*` member glob + the `k8s/*/*` acyclicity glob, and owned by the
subtree `k8s/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.17 entry
(breadth-unlimited per ADR-0555, covering the whole k8s subtree including the co-moved per-service SLO subdirs).
The move's tracked, born-accounted artifact roots are the seventeen crate dirs (each carrying its `Cargo.toml`,
`BUCK`, and `src/` — and, for the three crates with a binary + the tenant-quota-app, its `src/main.rs` +
`tests/`), the co-moved SLO subtree
`k8s/observability/slos/{cluster-lifecycle,control-plane-host,sla-observability,tenant-quota}/`, the subtree
`k8s/OWNERS`, the seventeen re-keyed catalog records `registry/catalog/k8s-*.yaml` (reached by the existing
`registry/catalog/` reachability prefix), and the committed move-plan `specs/reorg/k8s-move-plan.json` (reached
by the existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.18 Fourteenth executed strangler move: `tenancy` capability (cloud/tenancy/crates → tenancy/) — substrate three-face move (no facade) with flat SLO co-move + catalog re-key

The fourteenth REAL codemod run homes the `tenancy` capability's seventeen crates from ONE source dir
(`cloud/tenancy/crates`) under the §3 placement rule, across THREE faces (`core` + `ports` + `adapters`) with
NO `facade` — tenant management IS the substrate (the `tenancy` dag node, a control-plane substrate), not a
sold product surface, so there is no facade face to home. tenancy is the structural multi-tenancy substrate:
tenant lifecycle + home-cell resolution + tenant-as-universal-scoping-primitive (ADR-0244), the scoping
primitive every other capability binds against.

**Naming scheme (cargo == de-branded path-tail, all seventeen unique):** the proven scheme — path
`tenancy/<face>/<leaf>` ↔ cargo `tenancy-<leaf>`, de-branded (the legacy `oya-tenancy-`/`oya-tenant-` forms
drop to the `tenancy-` capability slug, and the redundant `-kernel`/`-domain`/`-usecase`/`-adapter` ROLE suffix
on the source dir name is dropped from the leaf where the role is already implied by the face placement, while
the three tenant-lifecycle crates KEEP their `-domain`/`-kernel`/`-usecase` role suffix to stay distinct from
each other under one cell). The seventeen: kernel → `tenancy/core/kernel` (cargo `tenancy-kernel`), domain →
`tenancy/core/domain` (`tenancy-domain`), cell-assignment-kernel → `tenancy/core/cell-assignment`
(`tenancy-cell-assignment`), dsr-cascade-kernel → `tenancy/core/dsr-cascade` (`tenancy-dsr-cascade`),
isolation-policy-kernel → `tenancy/core/isolation-policy` (`tenancy-isolation-policy`), lifecycle-locks-kernel →
`tenancy/core/lifecycle-locks` (`tenancy-lifecycle-locks`), sub-scope-registry-kernel →
`tenancy/core/sub-scope-registry` (`tenancy-sub-scope-registry`), kyb-kyc-verifier-domain →
`tenancy/core/kyb-kyc-verifier` (`tenancy-kyb-kyc-verifier`), tenant-lifecycle-domain →
`tenancy/core/tenant-lifecycle-domain` (`tenancy-tenant-lifecycle-domain`), tenant-lifecycle-kernel →
`tenancy/core/tenant-lifecycle-kernel` (`tenancy-tenant-lifecycle-kernel`), tenant-lifecycle-usecase →
`tenancy/core/tenant-lifecycle-usecase` (`tenancy-tenant-lifecycle-usecase`), dr-pairing-usecase →
`tenancy/core/dr-pairing` (`tenancy-dr-pairing`), per-tenant-quota-usecase → `tenancy/core/per-tenant-quota`
(`tenancy-per-tenant-quota`), reserved-namespace-usecase → `tenancy/core/reserved-namespace`
(`tenancy-reserved-namespace`), api → `tenancy/ports/api` (`tenancy-api`), the tenant CLI →
`tenancy/ports/cli` (`tenancy-cli`), and the data-residency-enforcer-adapter →
`tenancy/adapters/data-residency-enforcer` (`tenancy-data-residency-enforcer`). All seventeen leaf dirs and all
seventeen cargo names are distinct (`MovePlan::validate` passes), and each cargo name equals `tenancy-` + its
de-branded path-tail EXACTLY (the target-parity + cargo-prefix relabel binding).

**Face reasoning — ports/core/adapters by WHAT EACH IS (§3), verified against the REAL dep direction:** the
kernel/domain crates carry pure value objects, the TenantSlug grammar (ADR-0095), and the multi-tenancy state
machines and home to `core/` (the substrate engine surface, std+serde only); the five `*-kernel` cells
(cell-assignment, dsr-cascade, isolation-policy, lifecycle-locks, sub-scope-registry), the kyb-kyc-verifier
domain, and the tenant-lifecycle domain/kernel/usecase triad + the dr-pairing/per-tenant-quota/reserved-namespace
usecases are all pure substrate logic with no inbound transport seam, so they home to `core/`; the `api` crate
is the inbound application port (the trait + DTO seam, path-dep inward on `tenancy-domain` only) so it homes to
`ports/`; the tenant `cli` is a thin tenant-facing delivery port (clap-only, no domain dep) and homes to
`ports/`; the `data-residency-enforcer-adapter` projects a residency-enforcement port onto a backing impl and
homes to `adapters/`. The intra-capability edges are all downward and acyclic: `tenancy-api` (ports) →
`tenancy-domain` (core); `tenancy-tenant-lifecycle-usecase` (core) → `tenancy-tenant-lifecycle-domain` +
`tenancy-tenant-lifecycle-kernel` (core). The dep DAG is acyclic by construction, and because all seventeen are
ONE `tenancy` capability node the intra-capability face edges project to a `tenancy → tenancy` self-edge and
raise NO service→service / S-rank acyclicity violation (the §10.15/§10.16/§10.17 console/comms/k8s precedent).

**Cross-capability dep (kept intact):** `tenancy-api` (ports) and `tenancy-domain` (core) both have an OUTBOUND
dep on `oya-residency-domain` (the data-residency value objects living in the `cloud-network` capability, which
is NOT yet homed). The dep continues to point at the live `cloud/cloud-network/crates/oya-residency-domain`
path (the relative `path=` + BUCK `//`-label were recomputed by the codemod against the new `tenancy/ports/api`
and `tenancy/core/domain` homes); it relabels when the network/residency capability moves in a later strangler
step. It is NOT broken or duplicated. (`tenancy-domain` also depends on `libs/oya-data-boundary-kernel`, and the
tenant-lifecycle triad on `libs/oya-shared-platform-contracts-kernel` + `libs/oya-shared-resource-provider-contract-kernel`
— both `libs/` deps were relative-path-recomputed and stay pointing at the live `libs/` homes, the legal
below-all-capabilities `base/`-class direction.)

**External dependents (rewritten):** TWO first-party crates outside the seventeen depend on the moved tree —
`oya/application/crates/oya-application-app` depends on `tenancy-domain` (the application composition root) and
`libs/oya-http-tenant-middleware-infrastructure` depends on `tenancy-kernel` (the TenantSlug grammar consumer,
ADR-0095). The codemod rewrote both dependents' `Cargo.toml` path-deps, `BUCK` `//`-labels, and `use`-ident
references (`oya_tenancy_domain` → `tenancy_domain`, `oya_tenancy_kernel` → `tenancy_kernel`) to the new homes.
tenancy is NOT a violation source (zero entries in the acyclicity frozen baseline) and the moved crate dirs are
not in the membership unmapped baseline, so both lints carry 0 burn-down / 0 regression.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null,
`clean=true`). The capability registry records `tenancy.absorbs_current_dirs` with the capability's own
top-level slug `tenancy` (the self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE
tenancy/...`; the §10.12/§10.16/§10.17 flags/comms/k8s lesson) plus the absorbed source dir `cloud/tenancy` (the
pre-existing seed, retained). The membership policy scan_roots + allowed_top_level_dirs gain `tenancy`, the
acyclicity policy crate_root_globs gains `tenancy/*/*` + unclassified_roots gains `tenancy`, and the root
workspace gains the `tenancy/*/*` member glob (one glob covers all seventeen faces/leaves; ADR-0538 glob-only
contract).

**SLO co-move executed IN the move (doctrine-clean, flat — no collision):** the single source dir carries five
promotion-gating SLOs (`cloud/tenancy/slos/{autosharding-events,availability,correctness,freshness,latency}.openslo.yaml`).
Because there is exactly ONE source SLO dir, a flat merge into `tenancy/observability/slos/` has no cross-service
basename collision (unlike the §10.16/§10.17 comms/k8s multi-service cases that needed per-service subdirs), so
the five SLOs co-move via five content-preserving file `ArtifactMove`s into the flat
`tenancy/observability/slos/` dir — collision-free AND provenance-preserving. The slo-coverage gate keys by
catalog-record file stem, not SLO file path, so the home is gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14..§10.17):** fifteen of the seventeen crates have a
`registry/catalog/oya-tenancy-*.yaml` / `registry/catalog/oya-tenant-cli.yaml` record (the two
tenant-lifecycle-domain and tenant-lifecycle-usecase crates have NO pre-existing record — none is invented this
move). After the crate rename, the fifteen legacy filenames no longer match any live workspace `[package].name`,
which would RED the `catalog-liveness` (`catalog_record_no_live_crate_unmarked`) and `slo-coverage`
(`slo_row_no_live_crate_unmarked`) gates. The move-plan therefore carries fifteen additional file
`ArtifactMove` entries re-keying each record to the de-branded live crate-id filename
`registry/catalog/tenancy-<leaf>.yaml` (content-preserving `git mv`; no in-file rewrite — the records carry no
embedded package-name field, only `context`/`role`/`capability` facets). Both gates stay green after the re-key
(all fifteen records bind to a live workspace crate-id; the two record-less crates raise no orphan record).

**Custom bin name (codemod gap #76, retirement-marked CLI, brand preserved):** the tenant `cli` crate declares
a custom `[[bin]] name = "oya-tenant"` + `default-run = "oya-tenant"` (`Cargo.toml`), a matching `rust_binary`
target name `oya-tenant` (`BUCK`), which the codemod's `apply` does NOT auto-de-brand (it rewrites
`[package].name`/`[lib].name`/deps/labels/idents only; the package de-brands to `tenancy-cli` and the BUCK
`CARGO_PKG_NAME` env was rewritten to `tenancy-cli` automatically). The `oya-tenant` bin/binary NAME is
DELIBERATELY PRESERVED: it is the Tier-A semver-protected tenant-facing binary distributed as `oya-tenant` to
external artifact channels (Homebrew, apt, winget, ghcr) per ADR-0167, and the design-sweep mapping marked it
for preservation. The flags §10.12 precedent established that a preserved branded bin name is gate-green
(`[[bin]] name = "oya-flags"` was kept as deferred de-brand-profile residue, ADR-0532/0533 lane), so preserving
`oya-tenant` is the gate-green AND distribution-stable choice — and it is a retirement-marked, local-bridge-only
CLI (never merge authority, per the all-CLI-retirement directive), so its command name is not a structural
identifier the reorg owns. The retirement marker on the crate is retained. The bin name de-brand is the
ADR-0532/0533 de-brand-profile lane's scope (task #63), not this structural move.

**Born-accounting (ADR-0555):** the seventeen new crate dirs under `tenancy/core/`, `tenancy/ports/`, and
`tenancy/adapters/` are reached by the `tenancy/*/*` member glob + the `tenancy/*/*` acyclicity glob, and owned
by the subtree `tenancy/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.18
entry (breadth-unlimited per ADR-0555, covering the whole tenancy subtree including the co-moved flat SLO dir).
The move's tracked, born-accounted artifact roots are the seventeen crate dirs (each carrying its `Cargo.toml`,
`BUCK`, and `src/` — and, for the `api` + `tenant-lifecycle-usecase` crates, their `tests/`; for the `cli`
crate, its `src/main.rs`), the co-moved flat SLO dir
`tenancy/observability/slos/{autosharding-events,availability,correctness,freshness,latency}.openslo.yaml`, the
subtree `tenancy/OWNERS`, the fifteen re-keyed catalog records `registry/catalog/tenancy-*.yaml` (reached by the
existing `registry/catalog/` reachability prefix), and the committed move-plan
`specs/reorg/tenancy-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.19 Fifteenth executed strangler move: `audit` capability (oya/audit-chain/crates → audit/) — foundational-substrate three-face move (no facade) with flat SLO co-move + catalog re-key + cross-capability messaging edge preserved

The fifteenth REAL codemod run homes the `audit` capability's eighteen crates from ONE source dir
(`oya/audit-chain/crates`) under the §3 placement rule, across THREE faces (`core` + `ports` + `adapters`) with
NO `facade` — the tamper-evident Merkle audit log IS the substrate (the S0 `audit-chain` dag node, the
always-on/no-kill-switch chain-of-custody floor every other capability seals against), not a sold product
surface, so there is no facade face to home. audit is the foundational evidence substrate: Ed25519-signed Merkle
chains + the emission/query/retention/sealing/verification port-cells (ADR-0083 amendment).

**Naming scheme (cargo == de-branded path-tail, all eighteen unique):** the proven console/k8s scheme — path
`audit/<face>/<leaf>` ↔ cargo `audit-<leaf>`, de-branded (the legacy `oya-audit-chain-` form drops to the
`audit-` capability slug and the redundant `chain` doubling is dropped from the middle, EXCEPT the central
`chain-domain` crate which KEEPS `chain` as its cell name — both because the six external dependents already
reference the de-branded `audit-chain-domain` id and because dropping it to `audit-domain` would collide with the
five per-cell `*-domain` crates' role grammar). The eighteen: domain → `audit/core/chain-domain` (cargo
`audit-chain-domain`), usecase → `audit/core/usecase` (`audit-usecase`), the five `<cell>-domain` crates →
`audit/core/<cell>-domain` (`audit-emission-domain`, `audit-query-domain`, `audit-retention-cascade-domain`,
`audit-sealing-domain`, `audit-verification-domain`), the five `<cell>-kernel` crates →
`audit/ports/<cell>-kernel` (`audit-emission-kernel`, `audit-query-kernel`, `audit-retention-cascade-kernel`,
`audit-sealing-kernel`, `audit-verification-kernel`), the five `<cell>-api` DTO crates →
`audit/ports/<cell>-api` (`audit-emission-api`, `audit-query-api`, `audit-retention-cascade-api`,
`audit-sealing-api`, `audit-verification-api`), and the file-adapter → `audit/adapters/file`
(`audit-file-adapter`). The `-cascade` segment is RETAINED on the retention cell (the source crate is
`retention-cascade`, a single cell name; dropping it would lose the cascade semantics and is not needed for
uniqueness). All eighteen leaf dirs and all eighteen cargo names are distinct (`MovePlan::validate` passes), and
each cargo name equals `audit-` + its de-branded path-tail EXACTLY (the target-parity + cargo-prefix relabel
binding).

**DTO-granularity decision (the open registry-granularity Q, resolved empirically):** the five `*-api` crates are
request/response DTOs. They home to `ports/` alongside their sibling `*-kernel` port-trait crates, NOT to
`core/`. Two of the five (`emission-api`, `sealing-api`) carry a path-dep INWARD on their own cell's `*-kernel`,
so co-locating api+kernel in `ports/` keeps that edge a legal intra-face `ports → ports` edge; placing the api in
`core/` would invert it to `core → ports`. The acyclicity gate classifies the whole `audit/` root as an
`unclassified_root` (like every freshly-homed capability), so it enforces NO core/ports SUB-tier edge — both
placements are 0-regression at the gate surface — but the ports home is the structurally honest one (a DTO is the
shape of a port's payload, it travels WITH the port contract). All five api crates therefore home to `ports/`.

**Face reasoning — ports/core/adapters by WHAT EACH IS (§3), verified against the REAL dep direction:** the
`chain-domain` crate carries the Ed25519/Merkle value objects + the AuditChain state machine (std + ed25519-dalek
+ sha2), and the `usecase` crate carries the append/emit orchestration, both pure substrate engine surfaces →
`core/`; the five `<cell>-domain` crates are pure per-cell validation/rules (envelope validation, query
validation, retention rules, Merkle-math wrapper, proof/signature verification) with no inbound transport seam →
`core/`; the five `<cell>-kernel` crates are port-only (pure types/traits, no I/O) → `ports/`; the five
`<cell>-api` crates are the DTO payload seam → `ports/` (the DTO decision above); the `file` adapter projects the
ledger persistence port onto a file backing impl → `adapters/`. The intra-capability edges are all downward and
acyclic: `audit-emission-api`/`audit-sealing-api` (ports) → their `*-kernel` (ports); `audit-emission-domain` →
`audit-emission-kernel` and `audit-sealing-domain` → `audit-sealing-kernel` (core → ports, the legal
domain-implements-port direction); `audit-query-domain` → `audit-query-api`, `audit-retention-cascade-domain` →
`audit-retention-cascade-api`, `audit-verification-domain` → `audit-verification-api` (core → ports);
`audit-usecase` + `audit-file-adapter` + `audit-sealing-domain` + `audit-verification-domain` → `audit-chain-domain`
(core/adapters → core). The dep DAG is acyclic by construction, and because all eighteen are ONE `audit`
capability node the intra-capability face edges project to an `audit → audit` self-edge and raise NO
service→service / S-rank acyclicity violation (the §10.15..§10.18 console/comms/k8s/tenancy precedent).

**Cross-capability dep (kept intact — no inversion):** `audit-usecase` (core) has an OUTBOUND dep on
`messaging-domain` (the already-homed `messaging/core/domain`, ADR-0280 messaging dag node). The codemod
recomputed the relative `path=` (`../../../messaging/core/domain`) + the BUCK `//`-label against the new
`audit/core/usecase` home; the edge continues to point at the live messaging crate, NOT broken or duplicated.
Both `audit` and `messaging` are `unclassified_roots` at this stage of the strangler, so the
`audit/* → messaging/*` edge is ALLOWED (every edge to/from an unclassified crate is allowed) and raises no
S-rank inversion. (`audit-chain-domain`, `audit-usecase`, and `audit-file-adapter` also depend on
`libs/oya-data-boundary-kernel` — relative-path-recomputed and stays pointing at the live `libs/` home, the legal
below-all-capabilities `base/`-class direction.)

**External dependents (rewritten):** SIX first-party crates outside the eighteen depend on the moved tree —
`observability/core/aggregate` + `observability/core/api` (both → `audit-chain-domain`),
`oya/intelligence/crates/oya-intelligence-cloud-mutation-domain` (→ `audit-chain-domain`),
`oya/application/crates/oya-application-app` (→ `audit-chain-domain`), `marketplace/facade/dev-cli`
(the dev-cli; → `audit-chain-domain` + `audit-file-adapter`), and
`oya/tenant-rbac/crates/oya-tenant-rbac-audit-chain-emission` (→ `audit-emission-api` + `audit-emission-kernel`).
The codemod rewrote all six dependents' `Cargo.toml` path-deps, `BUCK` `//`-labels, and `use`-ident references
(`oya_audit_chain_domain` → `audit_chain_domain`, `oya_audit_chain_file_adapter` → `audit_file_adapter`,
`oya_audit_chain_emission_api`/`_kernel` → `audit_emission_api`/`_kernel`) to the new homes. The UNRELATED
`libs/oya-check-audit-chain-seal-coverage` crate (a different, non-moved crate) was correctly left untouched.
audit is NOT a violation source (zero entries in the acyclicity frozen baseline) and the moved crate dirs are not
in the membership unmapped baseline, so both lints carry 0 burn-down / 0 regression.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null,
`clean=true`). The capability registry records `audit.absorbs_current_dirs` with the capability's own top-level
slug `audit` (the self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE audit/...`; the
§10.12/§10.16..§10.18 lesson) plus the absorbed source dir `oya/audit-chain` (the pre-existing seed, retained).
The membership policy scan_roots + allowed_top_level_dirs gain `audit`, the acyclicity policy crate_root_globs
gains `audit/*/*` + unclassified_roots gains `audit`, and the root workspace gains the `audit/*/*` member glob
(one glob covers all eighteen faces/leaves; ADR-0538 glob-only contract) with `audit/observability` added to the
exclude list (the non-crate SLO subtree).

**SLO co-move executed IN the move (doctrine-clean, flat — no collision):** the single source dir carries eight
promotion-gating SLOs (`oya/audit-chain/slos/{autosharding-events,chain-of-custody-integrity-correctness,
evidence-export-freshness,merkle-chain-verification-latency,seal-cycle-latency,seal-storage-availability,
seal-write-availability,seal-write-latency}.openslo.yaml`). Because there is exactly ONE source SLO dir with eight
unique basenames, a flat merge into `audit/observability/slos/` has no cross-service basename collision (unlike
the §10.16/§10.17 comms/k8s multi-service cases that needed per-service subdirs), so the eight SLOs co-move via
eight content-preserving file `ArtifactMove`s into the flat `audit/observability/slos/` dir — collision-free AND
provenance-preserving. The SLOs' `metadata.name` + Prometheus metric labels keep their `oya-audit-chain`/
`oya_audit_chain` tokens (those are RUNTIME-emitted metric identifiers bound to the live service emission code,
not structural reorg tokens — de-branding them would orphan the SLO from its real metric and is the ADR-0532/0533
de-brand-profile lane's scope, not this structural move). The slo-coverage gate keys by catalog-record file stem,
not SLO file path, so the home is gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14..§10.18):** all eighteen moved crates have a
`registry/catalog/oya-audit-chain-*.yaml` record (no record-less crate this move, unlike the §10.18 tenancy
case). After the crate rename, the eighteen legacy filenames no longer match any live workspace `[package].name`,
which would RED the `catalog-liveness` (`catalog_record_no_live_crate_unmarked`) and `slo-coverage`
(`slo_row_no_live_crate_unmarked`) gates. The move-plan therefore carries eighteen additional file `ArtifactMove`
entries re-keying each record to the de-branded live crate-id filename `registry/catalog/audit-<leaf>.yaml`
(content-preserving `git mv`; no in-file rewrite — the records carry no embedded package-name field, only
`context`/`role`/`capability` facets; the internal `capability: audit-chain` facet is left content-preserved, no
gate keys on it). Both gates stay green after the re-key (all eighteen records bind to a live workspace crate-id).

**Born-accounting (ADR-0555):** the eighteen new crate dirs under `audit/core/`, `audit/ports/`, and
`audit/adapters/` are reached by the `audit/*/*` member glob + the `audit/*/*` acyclicity glob, and owned by the
subtree `audit/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.19 entry
(breadth-unlimited per ADR-0555, covering the whole audit subtree including the co-moved flat SLO dir). The
move's tracked, born-accounted artifact roots are the eighteen crate dirs (each carrying its `Cargo.toml`,
`BUCK`, and `src/` — and, for the `chain-domain`, `usecase`, and `file` crates, their `tests/`), the co-moved
flat SLO dir `audit/observability/slos/*.openslo.yaml`, the subtree `audit/OWNERS`, the eighteen re-keyed catalog
records `registry/catalog/audit-*.yaml` (reached by the existing `registry/catalog/` reachability prefix), and
the committed move-plan `specs/reorg/audit-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/`
reachability prefix).

#### §10.20 Sixteenth executed strangler move: `data` capability (cloud/cloud-data + oya/{ontology,search,analytics,data-pipeline,data-warehouse} → data/) — mixed substrate/product six-source-dir move with per-service SLO co-move + catalog re-key

The sixteenth REAL codemod run homes the `data` capability's twenty-three crates from SIX source dirs
(`cloud/cloud-data/crates`, `oya/ontology/crates`, `oya/search/crates`, `oya/analytics/crates`,
`oya/data-pipeline/crates`, `oya/data-warehouse/crates`) under the §3 placement rule, across THREE faces
(`core` 16 / `ports` 2 / `facade` 5). data is the owned data plane (the `ontology` ADR-0280 dag node): the
ontology object-graph + query engine (substrate), the search corpus engines (crawler/parser/index/query/rank/
serp/rag), the OLTP/OLAP storage domain, and the analytics/pipeline/warehouse product surfaces — a MIXED
substrate/product capability (hence a facade face exists, unlike the §10.18/§10.19 tenancy/audit pure-substrate
moves).

**Count reconciliation (22-in-design-mapping vs 23-source-Cargo.toml):** the design-mapping table enumerates
twenty-two named crates; the six source dirs hold twenty-three `Cargo.toml`. The reconciliation: every one of the
twenty-three source crates is enumerated and homed (no crate left behind). The brief's inline mapping table in
fact lists all twenty-three (cloud-data 2 + ontology 6 + search 8 + analytics 5 + data-pipeline 1 +
data-warehouse 1 = 23); the "22" is the upstream design-doc count, which omits one search-domain leaf
(`search-rag-domain`, the RAG retrieval-augmentation cell) from its abbreviated list. `search-rag-domain` is a
`*-domain` clean-arch role → `data/core/search-rag` (`data-search-rag`), homed like its seven sibling
search-domain crates. All twenty-three are accounted for by clean-arch role (kernel/domain → core, api → ports,
app/service/resolver → facade).

**Naming scheme (cargo == de-branded path-tail, all twenty-three unique):** the proven console/k8s/audit scheme —
path `data/<face>/<leaf>` ↔ cargo `data-<leaf>`, de-branded (the legacy `oya-cloud-data-`/`oya-ontology-`/
`oya-search-`/`oya-analytics-`/`oya-data-pipeline-`/`oya-data-warehouse-` forms drop to the `data-` capability
slug, and the `-domain` role suffix is dropped from the search cells whose leaf is the cell name). The
twenty-three: cloud-data {kernel,domain} → `data/core/cloud-{kernel,domain}` (`data-cloud-kernel`,
`data-cloud-domain`); ontology {kernel, domain, query-engine-domain, query-engine-usecase} →
`data/core/ontology-{kernel,domain,query-engine-domain,query-engine-usecase}`, ontology api →
`data/ports/ontology-api`, ontology resolve-scorecards-app → `data/facade/ontology-scorecards-resolver`
(`data-ontology-scorecards-resolver`); the eight search `<x>-domain` crates →
`data/core/search-<x>` (`data-search-{crawler,parser,index-inverted,index-vector,query,rank,serp,rag}`);
analytics {domain,usecase} → `data/core/analytics-{domain,usecase}`, analytics api → `data/ports/analytics-api`,
analytics app → `data/facade/analytics-app`, analytics tenant-bootstrap-app →
`data/facade/analytics-tenant-bootstrap`; data-pipeline lineage-replay-service →
`data/facade/pipeline-lineage-replay-service`; data-warehouse tenant-olap-service →
`data/facade/warehouse-tenant-olap-service`. All twenty-three leaf dirs and cargo names are distinct
(`MovePlan::validate` passes), and each cargo name equals `data-` + its de-branded path-tail EXACTLY (the
target-parity + cargo-prefix relabel binding).
Wave A kernel/OS parity evidence for the `data/core/cloud-kernel` streaming partition boundary is recorded at
`evidence/multispectrum/wavea-kernel-os-tenant-affine-partition-20260625-1782430095.json`.

**Face reasoning — ports/core/adapters/facade by WHAT EACH IS (§3), verified against REAL dep direction:** the
kernel/domain/usecase engine crates (cloud-data, ontology core, the eight search-domain corpus engines, analytics
domain/usecase) are pure substrate engine surfaces → `core/`; the `*-api` DTO/transport crates (ontology-api,
analytics-api) are the port-payload seam → `ports/` (the §10.19 DTO-in-ports decision: a DTO travels WITH its
port contract; ontology-api → ontology-domain and analytics-api → analytics-{domain,usecase} are legal
`ports → core` edges); the composition-root app/service/resolver crates (ontology-scorecards-resolver,
analytics-app, analytics-tenant-bootstrap, pipeline-lineage-replay-service, warehouse-tenant-olap-service) wire a
deployable surface → `facade/` (`analytics-app → analytics-api` is a legal `facade → ports` edge). The
acyclicity gate classifies the whole `data/` root as an `unclassified_root` (like every freshly-homed
capability), so it enforces NO core/ports/facade SUB-tier edge — every intra-`data` face edge projects to a
`data → data` self-edge and raises NO service→service / S-rank acyclicity violation (the §10.15..§10.19
precedent).

**OLAP libs (phase-2, NOT moved):** `libs/oya-shared-olap-client-kernel`, `libs/oya-shared-olap-clickhouse-adapter`,
and `libs/oya-data-boundary-kernel` are libs/ frozen-baseline phase-2 strangler candidates and stay in `libs/`
(out of this move's scope). Eight moved data crates consume them (cloud-data-domain, the four ontology core
crates, and four analytics crates); the codemod recomputed the relative `path=` + the BUCK `//`-label against each
crate's new `data/<face>/<leaf>` home so the edges keep pointing at the live (unchanged-path) `libs/` homes — the
legal below-all-capabilities `base/`-class direction.

**Violation edge (the historical `policy-engine → ontology` inversion):** the acyclicity DAG's
`forbidden_edges_assertion` records the historical `policy-engine → ontology` inversion ADR-0280 closes (ontology
depends on policy-engine for authorization; the reverse is forbidden). That assertion is keyed by ADR-0280
DAG-NODE NAME (`ontology` → de-branded data node), NOT by crate path, so the path move leaves it untouched. The
acyclicity GATE'S frozen baseline (`tier-dependency-acyclicity-baseline.json`) carries TWELVE known-debt
violations (3 KMS→residency, 9 intelligence/billing→application/community) — NONE involve any moved data crate,
and NO `policy-engine` crate has a code-level dep on any `ontology` crate (grep-verified empty). So there is no
baseline subject to relabel for this move and the violation edge does NOT false-RED: the rename-aware engine's
path-keyed relabel covers any moved-crate baseline subject by construction (here: the empty set), and the
acyclicity gate is 0-regression because the moved crates are absent from both the frozen baseline and any live
inversion.

**External dependent (rewritten):** exactly ONE first-party crate outside the twenty-three depends on the moved
tree — `oya/application/crates/oya-application-app` (→ `oya-ontology-domain`). The codemod rewrote its
`Cargo.toml` path-dep, `BUCK` `//`-label, and `use`-ident references (`oya_ontology_domain` → `data_ontology_domain`)
to the new `data/core/ontology-domain` home. A grep of the whole tree confirmed no other external dependent. data
is NOT a violation source (zero entries in the acyclicity frozen baseline) and the moved crate dirs are not in
the membership unmapped baseline, so both lints carry 0 burn-down / 0 regression.

**Custom bins de-branded (#76):** five facade crates carry custom `[[bin]]` names (the codemod only auto-rewrites
a bin name that equals `snake(old_cargo_name)`, so a custom kebab bin name is left for a manual de-brand, the
§10.13 k8s precedent): `oya-resolve-scorecards` → `data-ontology-scorecards-resolver` (+ `default-run`),
`oya-analytics-app` → `data-analytics-app`, `oya-analytics-tenant-bootstrap` → `data-analytics-tenant-bootstrap`,
`oya-data-pipeline-service` → `data-pipeline-service`, `oya-data-warehouse-service` → `data-warehouse-service`,
with the matching BUCK `rust_binary` target names de-branded in lockstep.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run
(`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null,
`clean=true`). The capability registry records `data.absorbs_current_dirs` with the capability's own top-level
slug `data` (the self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE data/...`; the
§10.12/§10.16..§10.19 lesson) plus the six absorbed source dirs (the pre-existing seeds, retained). The
membership policy scan_roots + allowed_top_level_dirs gain `data`, the acyclicity policy crate_root_globs gains
`data/*/*` + unclassified_roots gains `data`, and the root workspace gains the `data/*/*` member glob (one glob
covers all twenty-three faces/leaves; ADR-0538 glob-only contract) with `data/observability` added to the exclude
list (the non-crate SLO subtree).

**SLO co-move executed IN the move (doctrine-clean, per-service subdirs — collisions):** five of the six source
dirs carry promotion-gating SLOs (search carries none) — fifty-nine `.openslo.yaml` files total. The six source
dirs carry CROSS-SERVICE BASENAME COLLISIONS (`autosharding-events.openslo.yaml` ×5, `availability` ×2,
`audit-emission-lag` ×2, `policy-decision-latency` ×2, `read-latency` ×2, `replay-freshness` ×2,
`write-latency` ×2), so a flat merge would clobber. Following the §10.16/§10.17 comms/k8s multi-service
precedent, the fifty-nine SLOs co-move via fifty-nine content-preserving file `ArtifactMove`s into PER-SERVICE
subdirs `data/observability/slos/{cloud-data,ontology,analytics,data-pipeline,data-warehouse}/` — collision-free
AND provenance-preserving. The SLOs' `metadata.name` + Prometheus metric labels keep their legacy
`oya-*`/`oya_*` tokens (RUNTIME-emitted metric identifiers bound to live service emission code, not structural
reorg tokens — the ADR-0532/0533 de-brand-profile lane's scope, not this structural move). The slo-coverage gate
keys by catalog-record file stem, not SLO file path, so the home is gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14..§10.19):** all twenty-three moved crates have a
`registry/catalog/oya-*.yaml` record. After the crate rename the twenty-three legacy filenames no longer match
any live workspace `[package].name`, which would RED `catalog-liveness` and `slo-coverage`. The move-plan
therefore carries twenty-three additional file `ArtifactMove` entries re-keying each record to the de-branded live
crate-id filename `registry/catalog/data-<leaf>.yaml` (content-preserving `git mv`; no in-file rewrite — the
records carry no embedded package-name field, only facets; the internal `capability:` facet is content-preserved,
no gate keys on it). Both gates stay green after the re-key (all twenty-three records bind to a live workspace
crate-id).

**Born-accounting (ADR-0555):** the twenty-three new crate dirs under `data/core/`, `data/ports/`, and
`data/facade/` are reached by the `data/*/*` member glob + the `data/*/*` acyclicity glob, and owned by the
subtree `data/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.20 entry
(breadth-unlimited per ADR-0555, covering the whole data subtree including the co-moved per-service SLO subdirs).
The move's tracked, born-accounted artifact roots are the twenty-three crate dirs (each carrying its
`Cargo.toml`, `BUCK`, and `src/` — and, for the crates carrying integration/foundation tests, their `tests/`),
the co-moved per-service SLO subdirs `data/observability/slos/<service>/*.openslo.yaml`, the subtree
`data/OWNERS`, the twenty-three re-keyed catalog records `registry/catalog/data-*.yaml` (reached by the existing
`registry/catalog/` reachability prefix), and the committed move-plan `specs/reorg/data-move-plan.json` (reached
by the existing ADR-0563 `specs/reorg/` reachability prefix).

#### §10.21 Seventeenth executed strangler move: `workflow` capability (oya/{workflow-engine,workflow-studio,tasks,forms}/crates → workflow/) — four-source-dir mixed substrate/product move with per-service SLO co-move + catalog re-key

The seventeenth REAL codemod run homes the `workflow` capability's forty-eight crates from FOUR source dirs
(`oya/workflow-engine/crates`, `oya/workflow-studio/crates`, `oya/tasks/crates`, `oya/forms/crates`) under the §3
placement rule, across FOUR faces (`core` 12 / `ports` 16 / `adapters` 11 / `facade` 9). workflow is the
orchestration substrate (the `workflow-engine` ADR-0280 dag node): the engine's FOUR hexagonal sub-domains —
event-bus, execution-engine, state-machine, trigger-orchestrator (each a kernel/domain/usecase core + api/rest/sdk/
worker ports + adapter/adapter-{broker,postgres} adapters cell) — plus the studio DSL/canvas authoring surfaces, the
saas-workflow product, and the tasks + forms product domains — a MIXED substrate/product capability (hence a facade
face exists, like §10.20 data).

**Count reconciliation (47-in-design-mapping vs 48-source-Cargo.toml):** the move-time design-mapping estimate
enumerated forty-seven crates; the four source dirs hold forty-eight `Cargo.toml` (workflow-engine 42 + workflow-studio
4 + tasks 1 + forms 1). The reconciliation: every one of the forty-eight source crates is enumerated and homed (no
crate left behind). The undercounted leaf is one event-bus broker adapter — the engine's event-bus cell carries SEVEN
adapters (the generic `adapter` base + SIX broker variants `adapter-{kafka,nats,pulsar,redpanda,valkey,postgres}`); the
abbreviated design list named only the five message-broker variants (kafka/nats/pulsar/redpanda/valkey) and the generic
base, omitting `oya-workflow-engine-event-bus-adapter-postgres` (the postgres outbox/event-store adapter), which is a
`adapter-postgres` clean-arch role → `workflow/adapters/event-bus-postgres` (`workflow-event-bus-postgres`), homed like
its six sibling event-bus adapters. All forty-eight are accounted for by clean-arch role.

**Naming scheme (cargo == de-branded path-tail, all forty-eight unique):** the proven console/k8s/audit/data scheme —
path `workflow/<face>/<cell>-<role>` ↔ cargo `workflow-<cell>-<role>`, de-branded (the legacy
`oya-workflow-engine-`/`oya-workflow-studio-`/`oya-saas-workflow-`/`oya-` forms all drop to the `workflow-` capability
slug). Because the four engine sub-domain cells repeat the same role inventory (kernel/domain/usecase/api/rest/sdk/
worker/adapter), the cell name is the disambiguating leaf prefix: e.g. `event-bus-kernel` / `execution-engine-kernel` /
`state-machine-kernel` / `trigger-orchestrator-kernel` stay distinct. The broker adapters de-brand to
`workflow/adapters/<cell>-<broker>` (`workflow-event-bus-kafka`, `workflow-execution-engine-postgres`,
`workflow-state-machine-postgres`, …) and the generic base adapter keeps `<cell>-adapter`
(`workflow-event-bus-adapter`). The studio cells drop the `-domain`/`-kernel` role suffix to the cell name
(`studio-dsl-emitter`, `studio-dsl-loader`, `studio-policy-preview`, `studio-visual-canvas`); the saas product keeps its
kernel/domain/app role (`saas-kernel`, `saas-domain`, `saas-app`); tasks/forms keep their `-domain` role
(`tasks-domain`, `forms-domain`). All forty-eight leaf dirs and cargo names are distinct (`MovePlan::validate` passes),
and each cargo name equals `workflow-` + its de-branded path-tail EXACTLY.

**Face reasoning — ports/core/adapters/facade by WHAT EACH IS (§3), verified against REAL dep direction (the three
move-time open questions, RESOLVED with dep evidence):**

1. **Engine `*-app` composition-wiring crates (event-bus-app, execution-engine-app, trigger-orchestrator-app) →
`ports/`.** Dep evidence (read `Cargo.toml` + `BUCK` + `src/`): each is a pure `[lib]` (no `[[bin]]`, only `lib.rs`)
that depends ONLY on its own cell's boundary siblings — `event-bus-app → {event-bus-sdk, event-bus-worker}`,
`execution-engine-app → {execution-engine-adapter, -rest, -sdk, -worker}`, `trigger-orchestrator-app →
{trigger-orchestrator-rest, -sdk, -worker}`. They aggregate the cell's deployable boundary surface (rest+sdk+worker are
all ports). They are NOT sold (substrate engine, so NOT facade) and NOT a stable trait seam either, but the brief's
primary placement (ports = boundary) is the least-inverting organizational home: app sits adjacent to the rest/sdk/worker
it wires. The one cross-face edge (`execution-engine-app → execution-engine-adapter`, a ports→adapters edge) raises NO
acyclicity violation because the whole `workflow/` root is an `unclassified_root` (every intra-`workflow` face edge
projects to a `workflow → workflow` self-edge; the §10.15..§10.20 precedent) and the crate→crate edge set is byte-identical
pre/post-move (the codemod only changes paths, never deps).

2. **Event-bus broker adapters (event-bus-{kafka,nats,pulsar,redpanda,valkey,postgres}) → `workflow/adapters/event-bus-<broker>`.**
Dep evidence: each broker adapter depends ONLY on `oya-workflow-engine-event-bus-adapter` (the generic adapter, which
depends on `event-bus-api`, a workflow port). They implement WORKFLOW's OWN event-bus adapter trait, NOT the already-homed
`messaging` capability's port — ZERO dep on any `messaging/` crate (grep-verified). DEFAULT placement applies: they are
workflow's event-bus brokers, homed in `workflow/adapters/`, NOT re-homed cross-capability to messaging. (Future-consolidation
flag: if a later move proves these brokers SHOULD implement messaging's port, they can be re-homed then; the evidence here is
unambiguous — no messaging-port dependency exists, so a cross-cap move now would be speculative.)

3. **tasks/forms cross-cap deps confirmed → belong in workflow.** Dep evidence: `oya-tasks-domain` depends on
`oya-intelligence-capability-domain` (the `intelligence` capability, still at its pre-move `oya/intelligence/crates/` home in
this branch tip) + `libs/oya-data-boundary-kernel` (a shared lib, below all capabilities); `oya-forms-domain` depends on
`libs/oya-data-boundary-kernel` only. Both are workflow's task/form PRODUCT domains (the workplace task tracker + form builder
surfaces), so they home to `workflow/facade/{tasks,forms}-domain`. The cross-cap `tasks → intelligence` edge is PRESERVED
intact (the codemod left its path/`//`-label unchanged because intelligence has not moved in this PR; relabel-on-future-move,
the §10.19 cross-capability-edge precedent), and the `libs/` edges keep pointing at the live unchanged `libs/` homes (the legal
below-all-capabilities `base/`-class direction).

The kernel/domain/usecase engine crates of all four cells → `core/` (the engine we RUN); the api/rest/sdk/worker boundary
crates → `ports/` (the stable seam); the generic + provider adapters → `adapters/` (transient-infra impls); the studio +
saas + tasks + forms product surfaces → `facade/` (the surfaces we SELL). The acyclicity gate classifies the whole
`workflow/` root as an `unclassified_root`, so it enforces NO core/ports/adapters/facade SUB-tier edge.

**No custom bins (#76):** all forty-eight moved crates are pure `[lib]` crates (no `[[bin]]` anywhere in the four source
dirs, grep-verified), so the §10.13/§10.20 custom-bin de-brand step is a no-op for this move.

**External dependents (rewritten):** exactly TWO first-party crates outside the forty-eight depend on the moved tree —
`cloud/cloud-billing/crates/oya-saas-bench-app` (→ `oya-saas-workflow-{kernel,domain,app}`, the SaaS benchmark harness) and
`oya/application/crates/oya-workspace-forms-api` (→ `oya-forms-domain`, the workspace forms API surface). The codemod rewrote
each one's `Cargo.toml` path-dep, `BUCK` `//`-label, and `use`-ident references to the new
`workflow/facade/saas-{kernel,domain,app}` and `workflow/facade/forms-domain` homes. A grep + uquery of the whole tree
confirmed no other external dependent. workflow is NOT a violation source (zero entries in the acyclicity frozen baseline)
and the moved crate dirs are not in the membership unmapped baseline, so both lints carry 0 burn-down / 0 regression.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` +
`buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability
registry records `workflow.absorbs_current_dirs` with the capability's own top-level slug `workflow` (the self-slug is
required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE workflow/...`; the §10.12/§10.16..§10.20 lesson) plus the four
absorbed source dirs (the pre-existing seeds, retained). The membership policy scan_roots + allowed_top_level_dirs gain
`workflow`, the acyclicity policy crate_root_globs gains `workflow/*/*` + unclassified_roots gains `workflow`, and the root
workspace gains the `workflow/*/*` member glob (one glob covers all forty-eight faces/leaves; ADR-0538 glob-only contract)
with `workflow/observability` added to the exclude list (the non-crate SLO subtree).

**SLO co-move executed IN the move (doctrine-clean, per-service subdirs — collision):** all four source dirs carry
promotion-gating SLOs — thirty-five `.openslo.yaml` files total (workflow-engine 7 + workflow-studio 8 + tasks 10 + forms 10).
The four source dirs carry ONE cross-service basename collision (`autosharding-events.openslo.yaml`, present in all four), so
a flat merge would clobber. Following the §10.16/§10.17/§10.20 multi-service precedent, the thirty-five SLOs co-move via
thirty-five content-preserving file `ArtifactMove`s into PER-SERVICE subdirs
`workflow/observability/slos/{workflow-engine,workflow-studio,tasks,forms}/` — collision-free AND provenance-preserving. The
SLOs' `metadata.name` + Prometheus metric labels keep their legacy `oya-*`/`oya_*` tokens (RUNTIME-emitted metric identifiers
bound to live service emission code, not structural reorg tokens — the ADR-0532/0533 de-brand-profile lane's scope, not this
structural move). The slo-coverage gate keys by catalog-record file stem, not SLO file path, so the home is gate-neutral.

**Catalog re-key executed IN the move (same pattern as §10.14..§10.20):** all forty-eight moved crates have a
`registry/catalog/oya-*.yaml` record. After the crate rename the forty-eight legacy filenames no longer match any live
workspace `[package].name`, which would RED `catalog-liveness` and `slo-coverage`. The move-plan therefore carries forty-eight
additional file `ArtifactMove` entries re-keying each record to the de-branded live crate-id filename
`registry/catalog/workflow-<leaf>.yaml` (content-preserving `git mv`; no in-file rewrite — the records carry no embedded
package-name field, only facets; the internal `capability:` facet is content-preserved, no gate keys on it). Both gates stay
green after the re-key (all forty-eight records bind to a live workspace crate-id).

**Born-accounting (ADR-0555):** the forty-eight new crate dirs under `workflow/core/`, `workflow/ports/`,
`workflow/adapters/`, and `workflow/facade/` are reached by the `workflow/*/*` member glob + the `workflow/*/*` acyclicity
glob, and owned by the subtree `workflow/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.21
entry (breadth-unlimited per ADR-0555, covering the whole workflow subtree including the co-moved per-service SLO subdirs). The
move's tracked, born-accounted artifact roots are the forty-eight crate dirs (each carrying its `Cargo.toml`, `BUCK`, and
`src/` — and, for the crates carrying integration/foundation tests, their `tests/`), the co-moved per-service SLO subdirs
`workflow/observability/slos/<service>/*.openslo.yaml`, the subtree `workflow/OWNERS`, the forty-eight re-keyed catalog records
`registry/catalog/workflow-*.yaml` (reached by the existing `registry/catalog/` reachability prefix), and the committed
move-plan `specs/reorg/workflow-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix). Per the
one-plan-per-PR contract the spent `specs/reorg/data-move-plan.json` (the §10.20 move-plan) is removed.

#### §10.22 Eighteenth executed strangler move: `iam` capability (cloud/cloud-iam + oya/{identity,policy,tenant-rbac,oya-authn-device-firmware}/crates → iam/) — five-source-dir coarse identity+authz move with per-service SLO co-move + catalog re-key

The eighteenth REAL codemod run homes the `iam` capability's sixty-three crates from FIVE source dirs
(`cloud/cloud-iam/crates`, `oya/identity/crates`, `oya/policy/crates`, `oya/tenant-rbac/crates`,
`oya/oya-authn-device-firmware/crates`) under the §3 placement rule, across all FOUR faces (`core` 18 / `ports` 16 /
`adapters` 9 / `facade` 20). iam is the COARSE Conway-aligned identity + authorization capability (the `identity` +
`policy-engine` ADR-0280 dag nodes collapsed per the founder coarse-capability ruling): the cloud IdP/PDP substrate
(principals/STS/the embedded Cedar PDP/policy-bundle distribution/external-IdP federation), the Cedar policy plane, the
tenant-RBAC authorization spine (RBAC+ABAC+PBAC), the product-shared human/workload identity plane (OIDC issuer / passkey RP
/ SCIM / SPIFFE-SVID), and the device-firmware authenticator — a MIXED substrate/product capability with all four faces.

**Substrate-vs-product layering (the heart of the move, §4 run/sell seam):** the cloud-iam IdP/PDP engine, the Cedar policy
domain, the tenant-RBAC domain/usecase + zero-dep policy/manifest leaves, the workload domain/usecase/svid kernels, and the
device-firmware domain are the engine we RUN → `core/`; the inbound API/DTO surfaces + the tenant-RBAC capability-trait
contract crates (postgres-rls write/transaction + the ten tenant-`*`-contract crates) are the port seam → `ports/` (the
DTO-travels-with-its-contract / `ports → core` legal downward edge, the §10.19/§10.20 precedent); the transient infra
backends (cloud-iam OCI/selfhosted/pdp-bundle-file, the workload authz-cedar/oidc/svid-trustd adapters, the tenant-RBAC
inmemory storage/workflow + postgres-rls storage impls) are `adapters/`; and the product-shared identity service (`oya-identity`
bin), the workload REST surface, the PDP deployable (`oya-cloud-iam-pdp-app` bin), and the tenant-RBAC app + runtime-evidence
+ readiness-gate + listener + provider-verification product surfaces that CONSUME the substrate are the faces we SELL →
`facade/`. The dependency direction is product/facade → ports → substrate/core and never inverts (verified against the
real Cargo + BUCK dep graph). The two deployable bins (`oya-identity`, `oya-cloud-iam-pdp-app`) are facades; their
facade→adapters edges (pdp-app → pdp-bundle-file/svid-trustd) are the documented universal composition-root exception
(§10.6). `cloud-iam-app` homes to `core/` as the engine's usecase composition library (no adapter
direct-deps; downstream-only edges to core and ports). `identity-workload-app` homes to `facade/` (NOT
`core/`) — it directly depends on `adapters/identity-workload-oidc` and `adapters/identity-workload-authz-cedar`
(concrete adapter impls), making it a composition root by the clean-architecture definition: it wires concrete
adapters into the use-case flow. Composition roots belong in `facade/` (sanctioned §10.6 pattern, same as
`cloud-pdp-app`). After this correction the strict layer invariant holds: **ZERO** `core→adapters`,
`core→facade`, `ports→adapters`, `ports→facade` edges exist (verified by enumerating every `path =
"../../{adapters,facade}/"` in `iam/core/*/Cargo.toml` and `iam/ports/*/Cargo.toml` — both empty). The two
postgres-rls SQL contract crates (`tenant-rbac-postgres-rls-write-contract`,
`tenant-rbac-postgres-rls-transaction-contract`) home to `adapters/` (NOT `ports/`) — write-contract depends
on `adapters/tenant-rbac-postgres-rls-storage` (imports the concrete `TenantRbacPostgresRlsStoragePlan`) and
derives its SQL statements ON the storage adapter, making it a derived artifact of the adapter cluster;
transaction-contract depends on write-contract and inherits the same classification. Both belong with the
postgres-rls storage adapter, so all four postgres-rls crates are co-located in `adapters/`. The acyclicity
gate does not yet mechanically enforce intra-capability face direction (productized as a follow-up task #81);
the invariant was verified by hand (path-dep + BUCK-label enumeration above). All sixty-three crates are ONE
`iam` capability node, so every intra-iam face edge projects to an `iam → iam` self-edge and raises NO
service→service / S-rank acyclicity violation (the §10.15..§10.21 precedent; the tier-dep gate's
`owning_service()` None-projection drops every `iam/...`-endpoint edge before classification).

**Naming scheme (cargo == de-branded path-tail, all sixty-three unique):** the proven scheme — path `iam/<face>/<leaf>` ↔
cargo `iam-<leaf>`, de-branded (the legacy `oya-cloud-iam-`/`oya-identity-`/`oya-policy-`/`oya-tenant-rbac-`/
`oya-authn-device-firmware` forms drop to the `iam-` capability slug; `oya-cloud-iam-X` de-dups to `iam-cloud-X` NOT
`iam-cloud-iam-X` to avoid path-doubling, keeping the `cloud-` cell discriminator that distinguishes the cloud-iam substrate
cell from the identity/tenant-rbac product cells). Examples: `oya-cloud-iam-api`→`iam-cloud-api`, `oya-cloud-iam-pdp-app`→
`iam-cloud-pdp-app`, `oya-cloud-iam-adapter-oci`→`iam-cloud-oci`, `oya-identity`→`iam-identity-service`, `oya-identity-domain`→
`iam-identity-domain`, `oya-policy-cedar-api`→`iam-policy-cedar-api`, `oya-tenant-rbac-domain`→`iam-tenant-rbac-domain`,
`oya-authn-device-firmware`→`iam-authn-device-firmware`. All sixty-three leaf dirs and cargo names are distinct
(`MovePlan::validate` passes), and each cargo name equals `iam-` + its de-branded path-tail EXACTLY (the target-parity +
cargo-prefix relabel binding).

**Catalog re-key (58 of 63):** fifty-eight of the sixty-three moved crates carried a pre-existing `registry/catalog/<id>.yaml`
record, each re-keyed by the codemod ArtifactMove to its de-branded `iam-*` id (file rename, content-preserving). Five
substrate crates (`oya-cloud-iam-pdp-kernel`, `oya-cloud-iam-pdp-app`, `oya-cloud-iam-pdp-bundle-file-adapter`,
`oya-identity-workload-svid-kernel`, `oya-identity-workload-svid-trustd-adapter`) carry NO catalog record (pre-existing
state, not introduced by this move); the catalog-liveness gate is live-OR-marked and these crates have neither a live nor a
dead stale record, so it is 0-regression. No record is left at an old `oya-` stem.

**SLO co-move (19):** nineteen SLOs co-move to `iam/observability/slos/<service>/` across four per-service subdirs
(`cloud-iam` 1, `identity` 13, `tenant-rbac` 4, `identity-workload-svid-kernel` 1), per-service subdirs chosen to avoid the
confirmed cross-service basename collision `autosharding-events.openslo.yaml` (present in both cloud-iam and identity). The
`identity-workload-svid-kernel` SLO lived INSIDE its crate dir (`.../slos/`), so the crate move relocated it with the crate;
it is then co-moved out to the observability home so ZERO crate-resident SLOs remain (the absolute ADR-0139 convention — all
homed capabilities keep their SLOs at `<cap>/observability/slos/`). Zero `*.openslo.yaml` remains at any old iam source path.

**Graph-invisible tests (5 found, 5 wired-live, 0 deleted):** the workload-app + workload-rest integration tests carried no
owning buck2 `rust_test` target (`tests/acceptance.rs`; `tests/{rest_endpoints,grpc_authorize_deny,coverage_gaps}.rs` + the
shared `tests/common/mod.rs` support module). All four test files were wired-live with real `rust_test` targets (the shared
`common/mod.rs` co-listed in each rest target's srcs, the proven sibling pattern); none ignored, skipped, stubbed, or deleted
— so the FULL-tier affected-set binding-workspace-coverage gate has zero graph-invisible `tests/*.rs` in the affected cone.

Post-move closeout for the retired `oya/policy` dirty subset ports only the still-valid ReBAC tuple vocabulary into the
current Cedar domain home as `iam/core/policy-cedar-domain/src/rebac.rs`, with the graph-visible regression target
`iam/core/policy-cedar-domain/tests/rebac_tuple_port.rs` wired through the sibling `iam/core/policy-cedar-domain/BUCK`
test target. This is a destination-surface registration for the IAM capability move, not a resurrection of retired
`oya/policy` or `oya-dev-cli` authority.

AUTHZ-008 burns down the shared Cedar PDP adapter straggler left in the frozen `libs/oya-shared-*` baseline:
`libs/oya-shared-pdp-adapter-cedar` moves to `iam/adapters/pdp-cedar`, with cargo/lib ids
`iam-pdp-cedar` / `iam_pdp_cedar`. This is a behavior-preserving relocation of the same embedded
`cedar-policy` engine behind `oya-shared-pdp-kernel::PolicyDecisionPoint` (ADR-0536 D-2; ADR-0243
single-decision-algorithm rule), not a new authorization implementation. The only live dependents are
rewired in place: `iam/facade/cloud-pdp-app` and `tenancy/adapters/tenant-lifecycle-authz-pdp` update
their Cargo path deps, BUCK labels, and Rust imports to the new de-branded IAM adapter home.
The active one-plan source for this straggler move is `specs/reorg/iam-pdp-cedar-move-plan.json`;
cloud-ci materializes the generated move manifest from that reviewed source, so the generated face is
not hand-edited in the contributor PR.

**External dependents (4, rewritten):** exactly four first-party crates outside the sixty-three depend on the moved tree —
`compute/core/domain` (→ cloud-iam-domain), `observability/core/aggregate` (→ cloud-iam-domain),
`k8s/adapters/tenant-quota-adapter-cedar` (→ identity-workload-authz-cedar + identity-workload-domain), and
`oya/application/crates/oya-application-app` (→ identity-domain + policy-cedar-domain). The codemod rewrote each one's
`Cargo.toml` path-dep, `BUCK` `//`-label, and `use`-ident references to the new `iam/` homes; the two human-prose
`description` mentions (k8s adapter + `libs/oya-shared-oidc-client-kernel`) were de-branded by hand (the codemod rewrites
deps/labels/idents, not prose). A grep of the whole tree confirmed no other external dependent.

**Embedded-asset hermeticity:** the one include-site in the corpus (`cloud-iam-pdp-app/tests/common/mod.rs`,
`include_str!("../../cedar/…")` — hermetic in-crate relative paths) was scanned under the `cloud` scan_root pre-move; `iam`
was added to the embedded-asset gate `scan_roots` so the site stays scanned at its new `iam/facade/cloud-pdp-app/` home (the
cedar assets travel inside the crate and remain mapped by its BUCK `srcs` glob, so it stays GREEN with no baseline change).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` +
`buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability
registry records `iam.absorbs_current_dirs` with the capability's own top-level slug `iam` (the self-slug is required or the
membership gate REDs `MEM-NEW-UNMAPPED-CRATE iam/...`; the §10.12/§10.16..§10.21 lesson) plus the seven absorbed source dirs
(the pre-existing seeds, retained — including the crate-free `oya/oya-identity` + `oya/consent-graph` phase-2 dirs). The
membership policy adds `iam` to `scan_roots` + `allowed_top_level_dirs`; the acyclicity policy adds `iam/*/*` to
`crate_root_globs` + `iam` to `unclassified_roots`; the root workspace members glob adds `iam/*/*` (collapsing the
codemod-emitted literals) with `iam/observability` excluded (the non-crate SLO subtree). The move's tracked, born-accounted
artifact roots are the sixty-three crate dirs (each carrying its `Cargo.toml`, `BUCK`, `src/`, and, where present, `tests/`),
the co-moved per-service SLO subdirs `iam/observability/slos/<service>/*.openslo.yaml`, the subtree `iam/OWNERS`, the
fifty-eight re-keyed catalog records `registry/catalog/iam-*.yaml` (reached by the existing `registry/catalog/` reachability
prefix), and the committed move-plan `specs/reorg/iam-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/`
reachability prefix). Per the one-plan-per-PR contract the spent `specs/reorg/workflow-move-plan.json` (the §10.21 move-plan)
is removed.

#### §10.23 Nineteenth executed strangler move: `network` capability (cloud/cloud-network + cloud/cloud-network-dns/crates → network/) — two-source-dir network+DNS substrate move with the residency-domain inversion hub (~25 cross-capability dependents relabeled)

The nineteenth REAL codemod run homes the `network` capability's seven crates from TWO source dirs
(`cloud/cloud-network/crates`, `cloud/cloud-network-dns/crates`) under the §3 placement rule, across THREE faces
(`core` 2 / `ports` 3 / `adapters` 2) with NO `facade` — the network + DNS substrate IS the engine we RUN (the
`network` ADR-0280 dag node: service mesh / static-stable signed DNS snapshots / cell routing data plane), not a sold
product surface, so there is no facade face to home. network is the network + DNS substrate: the cloud-network
catalog/VPC/LB/route domain + the platform-wide ADR-0049 residency-class kernel + the vpc/lb/dns capability traits + the
oci/selfhosted transient provider impls.

**Substrate layering (the run face, §4 run seam; ports/adapters -> core, never inverted):** the `cloud-network` catalog/
VPC/LB/route value-object domain (`oya-cloud-network-domain` -> `network/core/domain`) and the platform-wide ADR-0049
residency-class kernel (`oya-residency-domain` -> `network/core/residency`) are the engine we RUN -> `core/`; the
inbound vpc/lb/dns capability-trait + DTO seams (`oya-cloud-network-vpc-api` -> `network/ports/vpc`,
`oya-cloud-network-lb-api` -> `network/ports/lb`, `oya-cloud-network-dns-api` -> `network/ports/dns`) are the port seam ->
`ports/` (the `ports -> core` legal downward edge, the §10.18..§10.22 precedent: a port carries a DTO that travels with
its contract and depends inward on the domain only); the transient infra backends (`oya-cloud-network-adapter-oci` ->
`network/adapters/oci`, `oya-cloud-network-adapter-selfhosted` -> `network/adapters/selfhosted`) are `adapters/`. The
dependency direction is ports/adapters -> core and never inverts (verified against the real Cargo + BUCK dep graph):
`network-domain` (core) -> `network-residency` (core) + `cell-region` + `compute-resource` + `libs/oya-data-boundary-kernel`
(all downward to lower capabilities / base-class libs); `network-residency` (core) -> `libs/oya-data-boundary-kernel` only
(a zero-network-dep ADR-0049 kernel); each of vpc/lb/dns (ports) -> `network-domain` + `network-residency` (core) +
`libs/`; each of oci/selfhosted (adapters) -> `network-domain` + `network-residency` (core) + `libs/`. After the move the
strict layer invariant holds: **ZERO** `core->adapters`, `core->facade`, `ports->adapters`, `ports->facade` edges exist
(verified by enumerating every `path = "../../{adapters,facade}/"` and any non-downward `..` edge in `network/core/*/
Cargo.toml` and `network/ports/*/Cargo.toml` — both empty). The acyclicity gate does not yet mechanically enforce
intra-capability face direction (its `owning_service()` recognizes only `cloud/`+`oya/` top-dirs -> every `network/...`
endpoint projects to None and is dropped before classification, so it is STRUCTURALLY BLIND to a `network/*` face
inversion, the §10.22 productized follow-up task #81); the invariant was verified by hand (path-dep + BUCK-label
enumeration above). All seven crates are ONE `network` capability node, so every intra-network face edge projects to a
`network -> network` self-edge and raises NO service->service / S-rank acyclicity violation.

**Naming scheme (cargo == de-branded path-tail, all seven unique):** the proven scheme — path `network/<face>/<leaf>` ↔
cargo `network-<leaf>`, de-branded (the legacy `oya-cloud-network-`/`oya-residency-` forms drop to the `network-`
capability slug; the role suffix is dropped where the face implies it, and the source-dir cell discriminator is dropped
since the face dir is NOT in the name and the leaves are already distinct). The seven: `oya-cloud-network-domain` ->
`network/core/domain` (`network-domain`), `oya-residency-domain` -> `network/core/residency` (`network-residency`),
`oya-cloud-network-vpc-api` -> `network/ports/vpc` (`network-vpc`), `oya-cloud-network-lb-api` -> `network/ports/lb`
(`network-lb`), `oya-cloud-network-dns-api` -> `network/ports/dns` (`network-dns`), `oya-cloud-network-adapter-oci` ->
`network/adapters/oci` (`network-oci`), `oya-cloud-network-adapter-selfhosted` -> `network/adapters/selfhosted`
(`network-selfhosted`). All seven leaf dirs and cargo names are distinct (`MovePlan::validate` passes), and each cargo
name equals `network-` + its de-branded path-tail EXACTLY (the target-parity + cargo-prefix relabel binding).

**THE residency-domain inversion hub (~25 cross-capability dependents, the largest blast radius in the series):**
`network-residency` (formerly `oya-residency-domain`, rust ident `oya_residency_domain` -> `network_residency`) is a
platform-wide ADR-0049 residency-class kernel with zero network-specific deps and twenty-five EXTERNAL cross-capability
dependents OUTSIDE the seven moved crates: `cell/core/{region,regional-pack,routing}` + `cell/ports/{region,regional-pack}`
(5), `cloud/cloud-kms/crates/{oya-cloud-kms-api,oya-cloud-kms-domain,oya-cloud-kms-operator-k8s-adapter}` (3, still at
`cloud/cloud-kms` until move-20), `compliance/core/trust-portal` (1), `compute/{adapters/aws,adapters/oci,core/domain,
core/resource,facade/functions,facade/k8s,facade/vm}` (7), `data/core/cloud-domain` (1),
`observability/core/{aggregate,api}` (2), `oya/application/crates/oya-application-app` (1), `storage/core/domain` +
`storage/ports/{block-api,object-api}` (3), `tenancy/core/domain` + `tenancy/ports/api` (2). Plus `compute/core/domain`
also depends on `network-domain`. The codemod rewrote every dependent's `Cargo.toml` path-dep, `BUCK` `//`-label, and
`use`-ident references (`oya_residency_domain` -> `network_residency`, `oya_cloud_network_domain` -> `network_domain`)
mechanically; `cargo metadata --locked` exits 0 and `buck2 targets //...` resolves post-move for the whole affected cone.
residency's inbound edges are in the FROZEN acyclicity/total-accounting baselines; the ADR-0563 rename-aware engine
relabels those baselined edges old->new from the committed move-plan->manifest, so the EXISTING violations relabel forward
and this move introduces ZERO NEW violations (membership/acyclicity/total-accounting 0-NEW-regression). NOTE: residency is a
genuine platform-wide cross-cutting kernel (ADR-0049) with no network-specific coupling; it is placed at
`network/core/residency` per the closed capability-registry for this move, but its breadth (25 dependents across 9
capabilities, zero network deps) flags it as a candidate for its own cross-cutting `base/`-class or `residency` capability
home in a later registry-granularity refinement (recorded, not actioned this move).

**Catalog re-key (7 of 7):** all seven moved crates carried a pre-existing `registry/catalog/<id>.yaml` record, each
re-keyed by the codemod ArtifactMove to its de-branded `network-*` id (file rename, content-preserving — the records carry
no embedded package-name field, only `context`/`role`/`capability` facets). No record is left at an old `oya-` stem; the
catalog-liveness gate stays green (all seven bind to a live workspace crate-id).

**SLO co-move (2, per-service subdirs — collision-avoiding):** both source dirs carry a promotion-gating SLO with the SAME
basename `autosharding-events.openslo.yaml` (`cloud/cloud-network/slos/` + `cloud/cloud-network-dns/slos/`), a confirmed
cross-service basename collision. They co-move via two content-preserving file `ArtifactMove`s into per-service subdirs
`network/observability/slos/{cloud-network,cloud-network-dns}/autosharding-events.openslo.yaml` (the §10.16/§10.17/§10.20/
§10.22 collision-aware pattern), so 2-in/2-out with neither lost nor shadowed. Both SLOs were already non-crate-resident at
merge-base (RED-accepted-debt at the old paths), so the ADR-0563 rename-aware engine relabels them old->new; zero
`*.openslo.yaml` remains at any old network source path.

**Graph-invisible tests (0 in the affected cone):** all four moved crates that carry `tests/*.rs`
(`network/core/domain`, `network/ports/vpc`, `network/ports/lb`, `network/ports/dns`) already had owning buck2 `rust_test`
targets at merge-base (the codemod recomputed their `//`-label deps to the new homes); and every one of the twenty-five
external dependents that carries `tests/*.rs` already has an owning `rust_test` target. The cone scan found ZERO
graph-invisible `tests/*.rs` (the one unrelated graph-invisible test in `oya/application/.../oya-workspace-meet-api` does
NOT depend on any of the seven network crates and is outside this move's affected-set — a pre-existing condition for a
later lane). Nothing ignored, skipped, stubbed, or deleted.

**Embedded-asset hermeticity (no change):** none of the seven moved crates uses `include_str!`/`include_bytes!` (verified
by corpus grep), so the embedded-asset gate `scan_roots` is NOT extended to `network` (unlike §10.22's `iam`, which carried
cedar include-sites) — moving from the `cloud` scan_root to the unscanned `network/` home loses zero coverage because there
is no embedded asset to scan.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` +
`buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability
registry records `network.absorbs_current_dirs` with the capability's own top-level slug `network` (the self-slug is
required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE network/...`; the §10.12/§10.16..§10.22 lesson) plus the two
absorbed source dirs `cloud/cloud-network` + `cloud/cloud-network-dns` (the pre-existing seeds, retained). The membership
policy adds `network` to `scan_roots` + `allowed_top_level_dirs`; the acyclicity policy adds `network/*/*` to
`crate_root_globs` + `network` to `unclassified_roots`; the root workspace members glob adds `network/*/*` (collapsing the
codemod-emitted literals) with `network/observability` excluded (the non-crate SLO subtree). The move's tracked,
born-accounted artifact roots are the seven crate dirs (each carrying its `Cargo.toml`, `BUCK`, `src/`, and, where present,
`tests/`), the co-moved per-service SLO subdirs `network/observability/slos/<service>/autosharding-events.openslo.yaml`, the
subtree `network/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.23 entry
(breadth-unlimited per ADR-0555, covering the whole network subtree including the co-moved SLO subdirs), the seven re-keyed
catalog records `registry/catalog/network-*.yaml` (reached by the existing `registry/catalog/` reachability prefix), and the
committed move-plan `specs/reorg/network-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability
prefix). Per the one-plan-per-PR contract the spent `specs/reorg/iam-move-plan.json` (the §10.22 move-plan) is removed.

#### §10.24 Twentieth executed strangler move: `secrets` capability (cloud/cloud-kms + cloud/cloud-secrets/crates → secrets/) — two-source-dir KMS + secrets + crypto-root move (recursion break #1) across all four faces, the operator facade composition root, and the violation-source outbound edges relabeled

The twentieth REAL codemod run homes the `secrets` capability's ten crates from TWO source dirs
(`cloud/cloud-kms/crates`, `cloud/cloud-secrets/crates`) under the §3 placement rule, across ALL FOUR faces
(`core` 4 / `ports` 1 / `adapters` 4 / `facade` 1). secrets is KMS + secrets + THE crypto root (recursion break #1, the
`cloud-secrets` ADR-0280 dag node): the cloud-kms tenant-scoped key-lifecycle / envelope-authorization domain + the enclave
crypto kernel + the operator reconcile kernel + the cloud-secrets SecretReference value-object domain are the engines we RUN;
the kms capability trait + DTO seam is the port; the oci/openbao provider backends + the operator-k8s reconciler adapter +
the file secret-store adapter are the transient infra; and the K8s operator composition root is the deployable facade.

**Substrate layering (the run face, §4 run seam; ports/adapters/facade -> core, never inverted):** the cloud-kms
key-lifecycle domain (`oya-cloud-kms-domain` -> `secrets/core/kms-domain`), the enclave crypto kernel
(`oya-cloud-kms-enclave-kernel` -> `secrets/core/kms-enclave`), the operator reconcile kernel
(`oya-cloud-kms-operator-kernel` -> `secrets/core/kms-operator-kernel`), and the cloud-secrets SecretReference value-object
domain (`oya-secrets-domain` -> `secrets/core/domain`) are the engines we RUN -> `core/`; the inbound kms capability-trait +
DTO seam (`oya-cloud-kms-api` -> `secrets/ports/kms-api`) is the port seam -> `ports/` (the `ports -> core` legal downward
edge, the §10.18..§10.23 precedent: a port carries a DTO that travels with its contract and depends inward on the domain
only); the transient infra backends (`oya-cloud-kms-adapter-oci` -> `secrets/adapters/kms-oci`,
`oya-cloud-kms-adapter-openbao` -> `secrets/adapters/kms-openbao`, `oya-cloud-kms-operator-k8s-adapter` ->
`secrets/adapters/kms-operator-k8s`, `oya-secrets-file-adapter` -> `secrets/adapters/file`) are `adapters/`; and the K8s
operator deployable (`oya-cloud-kms-operator-app` -> `secrets/facade/kms-operator-app`) is the composition root -> `facade/`.
THE FACADE DECISION: `kms-operator-app` is a K8s operator/reconciler whose `Cargo.toml` depends on the CONCRETE
`secrets-kms-operator-k8s` adapter (plus the `secrets-kms-domain` + `secrets-kms-operator-kernel` core kernels); a deployable
that wires a concrete adapter is a composition root and BELONGS in `facade/`, EXACTLY the iam cloud-pdp-app §10.22 precedent —
it is NOT pure substrate machinery, so `core/` would be a face inversion. The dependency direction is ports/adapters/facade
-> core and never inverts (verified against the real Cargo + BUCK dep graph): `secrets-kms-domain` (core) -> `cell-region` +
`compute-resource` + `network-residency` + `libs/oya-data-boundary-kernel` (all downward to lower capabilities / base-class
libs); `secrets-kms-enclave` (core) -> `secrets-kms-domain` (sibling core); `secrets-kms-operator-kernel` (core) -> nothing
(a zero-dep pure kernel); `secrets-domain` (core) -> `libs/oya-data-boundary-kernel` + `libs/oya-shared-platform-contracts-kernel`
only; `secrets-kms-api` (ports) -> `secrets-kms-domain` (core) + `cell-region` + `network-residency` + `libs/`; each of
kms-oci/kms-openbao/kms-operator-k8s/file (adapters) -> `secrets-{kms-domain,kms-enclave,kms-operator-kernel,domain}` (core) +
`libs/` (+ `network-residency` for kms-operator-k8s); `secrets-kms-operator-app` (facade) -> `secrets-kms-domain` +
`secrets-kms-operator-kernel` (core) + `secrets-kms-operator-k8s` (adapter, the legitimate facade->adapters composition-root
edge). After the move the strict layer invariant holds: **ZERO** `core->adapters`, `core->facade`, `core->ports`,
`ports->adapters`, `ports->facade`, `adapters->facade` edges exist (verified by enumerating every
`path = "../../{adapters,facade}/"` and any non-downward `..` edge in `secrets/core/*/Cargo.toml` and
`secrets/ports/*/Cargo.toml` — both empty — and confirming `adapters/*` carry no `->facade` edge; the SOLE non-downward edge
in the whole capability is the legitimate `facade->adapters` composition-root edge). The acyclicity gate does not yet
mechanically enforce intra-capability face direction (its `owning_service()` recognizes only `cloud/`+`oya/` top-dirs -> every
`secrets/...` endpoint projects to None and is dropped before classification, so it is STRUCTURALLY BLIND to a `secrets/*`
face inversion, the §10.22 productized follow-up task #81); the invariant was verified by hand (path-dep + BUCK-label
enumeration above). All ten crates are ONE `secrets` capability node, so every intra-secrets face edge projects to a
`secrets -> secrets` self-edge and raises NO service->service / S-rank acyclicity violation.

**Naming scheme (cargo == de-branded path-tail, all ten unique):** the proven scheme — path `secrets/<face>/<leaf>` ↔
cargo `secrets-<leaf>`, de-branded (the legacy `oya-cloud-kms-`/`oya-secrets-` forms drop to the `secrets-` capability slug;
the role suffix is dropped where the face implies it, and the source-dir cell discriminator is dropped since the face dir is
NOT in the name and the leaves are already distinct). The ten: `oya-cloud-kms-domain` -> `secrets/core/kms-domain`
(`secrets-kms-domain`), `oya-cloud-kms-enclave-kernel` -> `secrets/core/kms-enclave` (`secrets-kms-enclave`),
`oya-cloud-kms-operator-kernel` -> `secrets/core/kms-operator-kernel` (`secrets-kms-operator-kernel`), `oya-secrets-domain` ->
`secrets/core/domain` (`secrets-domain`), `oya-cloud-kms-api` -> `secrets/ports/kms-api` (`secrets-kms-api`),
`oya-cloud-kms-adapter-oci` -> `secrets/adapters/kms-oci` (`secrets-kms-oci`), `oya-cloud-kms-adapter-openbao` ->
`secrets/adapters/kms-openbao` (`secrets-kms-openbao`), `oya-cloud-kms-operator-k8s-adapter` ->
`secrets/adapters/kms-operator-k8s` (`secrets-kms-operator-k8s`), `oya-secrets-file-adapter` -> `secrets/adapters/file`
(`secrets-file`), `oya-cloud-kms-operator-app` -> `secrets/facade/kms-operator-app` (`secrets-kms-operator-app`). All ten leaf
dirs and cargo names are distinct (`MovePlan::validate` passes), and each cargo name equals `secrets-` + its de-branded
path-tail EXACTLY (the target-parity + cargo-prefix relabel binding).

**Violation-source: outbound edges relabeled, ZERO new violations.** secrets is a VIOLATION-SOURCE — `secrets-kms-domain`,
`secrets-kms-api`, and `secrets-kms-operator-k8s` (core/ports/adapter) carry outbound edges to `network-residency`
(`network/core/residency`, moved in §10.23), `cell-region` (`cell/core/region`), and `compute-resource`
(`compute/core/resource`); these edges are in the FROZEN acyclicity/total-accounting baselines. The ADR-0563 rename-aware
engine relabels those baselined edges old->new from the committed move-plan->manifest (`oya-cloud-kms-*`/`oya-secrets-*` ->
`secrets-*` and the old `cloud/cloud-kms`/`cloud/cloud-secrets` crate paths -> the new `secrets/*` homes). Because the targets
(network/cell/compute) live in acyclicity blind zones, the relabeled edges relabel-forward OR burn down (the gate no longer
detects a `secrets/...`-endpoint edge once both ends project to None) — EITHER outcome introduces ZERO NEW violations and the
firewall GO-LIVE stays green (it fails-closed on baseline-staleness, which the rename-aware relabel prevents). Six external
cross-capability dependents OUTSIDE the ten moved crates were rewritten mechanically (Cargo path-dep + BUCK `//`-label +
`use`-ident): `data/core/cloud-domain` (-> kms-domain), `storage/core/domain` + `storage/ports/object-api` (-> kms-domain),
`oya/application/crates/oya-application-app` (-> secrets-domain), `oya/intelligence/crates/oya-intelligence-adapter-domain`
(-> secrets-domain), and `marketplace/facade/dev-cli` (-> secrets-file + secrets-domain). `cargo metadata --locked` exits 0
and `buck2 targets //...` resolves post-move for the whole affected cone.

**Catalog re-key (6 of 10):** six of the ten moved crates carried a pre-existing `registry/catalog/<id>.yaml` record, each
re-keyed by the codemod ArtifactMove to its de-branded `secrets-*` id (file rename, content-preserving — the records carry no
embedded package-name field, only `context`/`role`/`capability` facets): `oya-cloud-kms-domain` -> `secrets-kms-domain`,
`oya-cloud-kms-api` -> `secrets-kms-api`, `oya-cloud-kms-adapter-oci` -> `secrets-kms-oci`, `oya-cloud-kms-adapter-openbao` ->
`secrets-kms-openbao`, `oya-secrets-domain` -> `secrets-domain`, `oya-secrets-file-adapter` -> `secrets-file`. The other four
moved crates (kms-enclave, kms-operator-kernel, kms-operator-k8s, kms-operator-app) carry no catalog record (none existed at
merge-base). No record is left at an old `oya-` stem; the catalog-liveness gate stays green (all six bind to a live workspace
crate-id).

**SLO co-move (11, per-service subdirs — collision-avoiding):** both source dirs carry a promotion-gating SLO with the SAME
basename `autosharding-events.openslo.yaml` (`cloud/cloud-kms/slos/` + `cloud/cloud-secrets/slos/`), a confirmed
cross-service basename collision. All eleven SLOs (4 from cloud-kms: autosharding-events, kms-dek-cache-static-stability,
kms-envelope-wrap-latency-p99, kms-reconcile-convergence; 7 from cloud-secrets: audit-log-completeness, autosharding-events,
hsm-availability, key-rotation-correctness, pki-cert-issuance-latency, secret-resolve-latency, secret-write-latency) co-move
via eleven content-preserving file `ArtifactMove`s into per-service subdirs
`secrets/observability/slos/{cloud-kms,cloud-secrets}/<basename>.openslo.yaml` (the §10.16/§10.17/§10.22/§10.23
collision-aware pattern), so 11-in/11-out with neither lost nor shadowed. All eleven SLOs were already non-crate-resident at
merge-base (RED-accepted-debt at the old paths), so the ADR-0563 rename-aware engine relabels them old->new; zero
`*.openslo.yaml` remains at any old secrets source path.

**Graph-invisible tests (2 found, both wired-live):** of the fifteen `tests/*.rs` across the ten moved crates, thirteen
already had owning buck2 `rust_test` targets at merge-base (the codemod recomputed their `//`-label deps to the new homes).
The cone scan found TWO graph-invisible `tests/*.rs` in `secrets/core/domain` (formerly `oya-secrets-domain`):
`tests/cloud_secret_foundation.rs` (113 lines, exercising `evaluate_secret_bootstrap` + `SecretReference` invariants) and
`tests/secret_vault.rs` (135 lines, exercising `SecretVault`/`SecretMaterial` no-debug-leak invariants). Both are real,
valuable domain tests — they were WIRED-LIVE with new owning `rust_test` targets (`cloud-secret-foundation-test`,
`secret-vault-test`; the latter carries the `oya-data-boundary-kernel` dep its `use` requires), compile post-move, and pass.
Nothing ignored, skipped, stubbed, or deleted. Every one of the six external dependents that carries `tests/*.rs` already has
an owning `rust_test` target.

**Embedded-asset hermeticity (no change):** none of the ten moved crates uses `include_str!`/`include_bytes!` (verified by
corpus grep), so the embedded-asset gate `scan_roots` is NOT extended to `secrets` (unlike §10.22's `iam`, which carried cedar
include-sites) — moving from the `cloud` scan_root to the unscanned `secrets/` home loses zero coverage because there is no
embedded asset to scan.

**dev-cli supply-chain + loop-recovery (no validator trip):** `marketplace/facade/dev-cli` is a Cargo + BUCK dependent of
secrets-domain + secrets-file (rewritten mechanically by the codemod); its supply-chain (`supply_chain_gates.rs`,
`supply_chain_adr0039.rs`) and loop-recovery (`loop_recovery_patterns_gate.rs`, `loop_recovery_patterns.rs`) gate sources
hardcode no kms/secrets crate id or path (the only "secret" tokens are the unrelated trivy `--scanners vuln,secret,license`
config), so no de-branded `secrets-` id trips a validator; both gate test targets run green post-move.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` +
`buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability
registry records `secrets.absorbs_current_dirs` with the capability's own top-level slug `secrets` (the self-slug is required
or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE secrets/...`; the §10.12/§10.16..§10.23 lesson) plus the two absorbed
source dirs `cloud/cloud-kms` + `cloud/cloud-secrets` (the pre-existing seeds, retained). The membership policy adds `secrets`
to `scan_roots` + `allowed_top_level_dirs`; the acyclicity policy adds `secrets/*/*` to `crate_root_globs` + `secrets` to
`unclassified_roots`; the root workspace members glob adds `secrets/*/*` (collapsing the codemod-emitted literals) with
`secrets/observability` excluded (the non-crate SLO subtree). The move's tracked, born-accounted artifact roots are the ten
crate dirs (each carrying its `Cargo.toml`, `BUCK`, `src/`, and, where present, `tests/`), the co-moved per-service SLO
subdirs `secrets/observability/slos/<service>/<basename>.openslo.yaml`, the subtree `secrets/OWNERS` (axis-cloud-platform)
seeded via a `specs/reachability-registry.json` §10.24 entry (breadth-unlimited per ADR-0555, covering the whole secrets
subtree including the co-moved SLO subdirs), the six re-keyed catalog records `registry/catalog/secrets-*.yaml` (reached by
the existing `registry/catalog/` reachability prefix), and the committed move-plan `specs/reorg/secrets-move-plan.json`
(reached by the existing ADR-0563 `specs/reorg/` reachability prefix). The absorbed dirs' other non-crate artifacts
(`cloud/cloud-kms/manifest.json` + `cloud/cloud-secrets/manifest.json`, docs/PRD/IPs/contracts) are homed in phase-2 (task
#62), per the §10.5..§10.23 precedent. Per the one-plan-per-PR contract the spent `specs/reorg/network-move-plan.json` (the
§10.23 move-plan) is removed.

#### §10.25 Twenty-first executed strangler move: `billing` capability (cloud/cloud-billing + cloud/cloud-billing-tax + cloud/cloud-finops + oya/accounting/crates → billing/) — four-source-dir metering + billing + tax + finops/cost + accounting move across all four faces, the violation-source outbound edges relabeled, and the three facade service bins de-branded

The twenty-first REAL codemod run homes the `billing` capability's sixteen crates from FOUR source dirs
(`cloud/cloud-billing/crates`, `cloud/cloud-billing-tax/crates`, `cloud/cloud-finops/crates`, `oya/accounting/crates`) under
the §3 placement rule, across ALL FOUR faces (`core` 7 / `ports` 3 / `adapters` 2 / `facade` 4). billing is the first-class
cross-cutting SOLD-NESS capability (NOT a junk-drawer, per the §6.1 founder ruling): metering + billing + tax + FinOps/cost +
accounting collapse into one home because they are one commercial concern (three-clock accrual/rating/invoicing, tax,
cost-attribution). The cloud-billing tenant-guardrail/invoice/ledger domain + the metering meter-unit domain + the cloud-finops
cost/savings domain + the accounting-journal double-entry domain + the billing/finops kernels + the accounting usecase
orchestrator are the engines we RUN; the finops + accounting capability traits + DTO seams + the lib-only tax invoice/contract
surface are the ports; the accounting-journal http-runtime + inmemory-storage backends are the transient infra; and the
billing/meter/cost service deployables + the saas-bench harness are the sold/runnable facade.

**Substrate layering (the run face, §4 run seam; ports/adapters/facade -> core, never inverted):** the cloud-billing
tenant-guardrail/invoice/ledger domain (`oya-cloud-billing-domain` -> `billing/core/billing`, cargo `billing-domain`), the
metering meter-unit domain (`oya-metering-domain` -> `billing/core/metering`, `billing-metering`), the cloud-finops cost domain
(`oya-cloud-finops-domain` -> `billing/core/finops`, `billing-finops`), the accounting-journal double-entry domain
(`oya-accounting-journal-domain` -> `billing/core/accounting-journal`, `billing-accounting-journal`), the two billing/finops
kernels (`oya-cloud-billing-kernel` -> `billing/core/billing-kernel`, `billing-kernel`; `oya-cloud-finops-kernel` ->
`billing/core/finops-kernel`, `billing-finops-kernel`), and the accounting usecase orchestrator
(`oya-accounting-journal-app` -> `billing/core/accounting-app`, `billing-accounting-app`) are the engines we RUN -> `core/`;
the inbound finops + accounting capability-trait + DTO seams (`oya-cloud-finops-api` -> `billing/ports/finops-api`,
`billing-finops-api`; `oya-accounting-journal-api` -> `billing/ports/accounting-api`, `billing-accounting-api`) plus the
lib-only tax invoice/contract surface (`oya-cloud-billing-tax-app` -> `billing/ports/tax-api`, `billing-tax-api`) are the port
seams -> `ports/` (the `ports -> core` legal downward edge, the §10.18..§10.24 precedent: a port carries a DTO that travels
with its contract and depends inward on the domain only); the transient infra backends
(`oya-accounting-journal-infrastructure` -> `billing/adapters/accounting-http`, `billing-accounting-http-adapter`;
`oya-accounting-journal-storage-adapter-inmemory` -> `billing/adapters/accounting-storage-inmemory`,
`billing-accounting-storage-inmemory-adapter`) are `adapters/`; and the three service deployables + the benchmark harness
(`oya-billing` -> `billing/facade/billing-service`, `billing-service`; `oya-meter` -> `billing/facade/meter-service`,
`billing-meter-service`; `oya-cost` -> `billing/facade/cost-service`, `billing-cost-service`; `oya-saas-bench-app` ->
`billing/facade/saas-bench`, `billing-saas-bench`) are the sold/runnable -> `facade/`. THE TWO FLAGGED-PLACEMENT DECISIONS,
verified against the REAL Cargo + BUCK dep graph (NOT the designed-ahead table): (1) `oya-cloud-billing-tax-app` is placed in
`ports/tax-api` NOT `facade/` because it is a `rust_library` (`crate_root = src/lib.rs`, NO `[[bin]]`/`main.rs`) with an
EMPTY `[dependencies]` table — an API/contract surface (the invoice-API contract test exercises it), NOT a deployable that
wires adapters; an "app"-suffixed name does not make it a facade. (2) `oya-accounting-journal-app` is placed in
`core/accounting-app` NOT `facade/` because it is a `rust_library` (no bin) whose only deps are `libs/oya-data-boundary-kernel`
(a base-class lib) + `billing-accounting-journal` (a sibling core domain) — pure orchestration/usecase logic with ZERO adapter
dependency, so it is NOT a composition root and `core/` holds (the move-18 inversion lesson: a core crate depending on an
adapter is forbidden — this crate has no such edge). The dependency direction is ports/adapters/facade -> core and never
inverts (verified against the real Cargo + BUCK dep graph): `billing-domain` (core) -> `billing-metering` (sibling core) +
`cell-region` + `compute-resource` + `libs/oya-data-boundary-kernel` (all downward); `billing-metering` (core) ->
`libs/oya-data-boundary-kernel` only; `billing-finops` (core) -> `billing-domain` + `billing-metering` (sibling core) +
`cell-region` + `compute-resource` + `libs/`; `billing-kernel`/`billing-finops-kernel` (core) -> nothing (zero-dep pure
kernels); `billing-accounting-journal` (core) -> `libs/oya-data-boundary-kernel` only; `billing-accounting-app` (core) ->
`libs/oya-data-boundary-kernel` + `billing-accounting-journal` (sibling core); `billing-finops-api` (ports) -> `billing-domain`
+ `billing-finops` + `billing-metering` (core) + `libs/`; `billing-accounting-api` (ports) -> `billing-accounting-journal`
(core); `billing-tax-api` (ports) -> nothing; `billing-accounting-http-adapter` (adapter) -> `billing-accounting-api` (ports) +
`billing-accounting-app` (core) + the http kernels/runtime libs; `billing-accounting-storage-inmemory-adapter` (adapter) ->
`billing-accounting-app` (core); the facade services are self-contained deployables and `billing-saas-bench` (facade) ->
`workflow-saas-{kernel,domain,app}` (other-cap facade) + `marketplace-plugin-kernel` (other-cap core) + `oya-saas-plugin-app`
(oya/application, not-yet-moved). After the move the strict layer invariant holds: **ZERO** `core->adapters`, `core->facade`,
`core->ports`, `ports->adapters`, `ports->facade`, `adapters->facade` edges exist (verified by enumerating every
`path = "../../{adapters,facade,ports}/"` edge in `billing/core/*/Cargo.toml`, every `path = "../../{adapters,facade}/"` edge
in `billing/ports/*/Cargo.toml`, and every `path = "../../facade/"` edge in `billing/adapters/*/Cargo.toml` — ALL empty; the
SOLE non-downward edges in the whole capability are the legitimate facade->{core,ports,adapters} composition-root edges and the
adapters->ports DTO-travels-with-its-contract downward edge). The acyclicity gate does not yet mechanically enforce
intra-capability face direction (its `owning_service()` recognizes only `cloud/`+`oya/` top-dirs -> every `billing/...`
endpoint projects to None and is dropped before classification, so it is STRUCTURALLY BLIND to a `billing/*` face inversion,
the §10.22 productized follow-up task #81); the invariant was verified by hand (path-dep enumeration above). All sixteen
crates are ONE `billing` capability node, so every intra-billing face edge projects to a `billing -> billing` self-edge and
raises NO service->service / S-rank acyclicity violation.

**Naming scheme (cargo == de-branded path-tail, all sixteen unique):** the proven scheme — path `billing/<face>/<leaf>` ↔
cargo `billing-<leaf>`, de-branded (the legacy `oya-cloud-billing-`/`oya-cloud-finops-`/`oya-accounting-journal-`/`oya-`
forms drop to the `billing-` capability slug; the role suffix is dropped where the face implies it, and the source-dir cell
discriminator is dropped since the face dir is NOT in the name and the leaves are already distinct). All sixteen leaf dirs and
cargo names are distinct (`MovePlan::validate` passes), and each cargo name equals `billing-` + its de-branded path-tail
EXACTLY (the target-parity + cargo-prefix relabel binding) — e.g. `billing/core/billing` ↔ `billing-domain` (the leaf `billing`
de-dups against the capability slug to the role-tail `domain`), `billing/facade/billing-service` ↔ `billing-service`,
`billing/adapters/accounting-storage-inmemory` ↔ `billing-accounting-storage-inmemory-adapter`.

**Custom-bin de-brand (3 facade service bins; the iam §10.22 precedent).** The three facade service crates each ship a
`rust_binary` whose `[[bin]] name` + BUCK target carried the legacy `oya-` brand (`oya-billing`/`oya-billing-bin`,
`oya-meter`/`oya-meter-bin`, `oya-cost`/`oya-cost-bin`); the codemod renames a moved crate's `[package].name` but PRESERVES a
`-bin` sibling target + the `[[bin]].name` artifact name (the B1 silent-clobber guard), so these survive a bare move unchanged.
A corpus grep confirmed NONE of the three bin artifact names is runtime-coupled (no k8s workload / PVC / mount-path / image /
helm / iac / runbook reference to `oya-billing`/`oya-meter`/`oya-cost` as a binary), UNLIKE the §10.24 kms-operator runtime
scheme that was deferred. They were therefore de-branded in lockstep with the package rename (the iam `iam-identity-service` /
`iam-cloud-pdp-app` §10.22 precedent: `[[bin]].name` == package name, BUCK `rust_binary` name == `<package>-bin`):
`billing-service`/`billing-service-bin`, `billing-meter-service`/`billing-meter-service-bin`,
`billing-cost-service`/`billing-cost-service-bin`. No bin is left at an `oya-` stem; nothing runtime-coupled was touched.

**Violation-source: outbound edges relabeled, ZERO new violations.** billing is a VIOLATION-SOURCE — `billing-domain` +
`billing-finops` (core) carry outbound edges to `cell-region` (`cell/core/region`) + `compute-resource`
(`compute/core/resource`), and `billing-saas-bench` (facade) carries outbound edges to `workflow-saas-{kernel,domain,app}`
(workflow, moved), `marketplace-plugin-kernel` (marketplace, moved), and `oya-saas-plugin-app` (oya/application, still
cloud/oya-tier'd, not-yet-moved); these edges are in the FROZEN acyclicity/total-accounting baselines. The ADR-0563
rename-aware engine relabels those baselined edges old->new from the committed move-plan->manifest (`oya-cloud-billing-*` /
`oya-cloud-finops-*` / `oya-accounting-journal-*` / `oya-metering-domain` / `oya-billing` / `oya-meter` / `oya-cost` /
`oya-saas-bench-app` -> `billing-*` and the old crate paths -> the new `billing/*` homes). Because the targets
(cell/compute/workflow/marketplace) live in acyclicity blind zones — and the one still-tier'd target `oya-saas-plugin-app` is
unchanged on both ends — the relabeled edges relabel-forward OR burn down (the gate no longer detects a `billing/...`-endpoint
edge once both ends project to None), EITHER outcome introducing ZERO NEW violations; the firewall GO-LIVE stays green (it
fails-closed on baseline-staleness, which the rename-aware relabel prevents). Four external cross-capability dependents OUTSIDE
the sixteen moved crates were rewritten mechanically by the codemod (Cargo path-dep + BUCK `//`-label + `use`-ident):
`marketplace/core/cloud-domain` (-> billing-domain + billing-metering), `cell/core/capacity-commercial` (-> billing-domain +
billing-metering), `iam/facade/tenant-rbac-local-inmemory-harness` (-> billing-accounting-app + billing-accounting-journal +
billing-accounting-storage-inmemory-adapter), and `iam/facade/tenant-rbac-local-runtime-composition` (->
billing-accounting-http-adapter). The lone LIVE path-string dependent — `marketplace/facade/dev-cli`'s
`tests/fd001_data_class_taxonomy.rs`, which `fs::read`s the tax crate's `src/lib.rs` by repo-relative path — was relabeled
old->new (`cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/src/lib.rs` -> `billing/ports/tax-api/src/lib.rs`); the
dev-cli supply-chain + loop-recovery gate SOURCES hardcode no billing/accounting crate id, and the
`workspace_topology_gate`/`architecture_boundaries` references are hermetic SYNTHETIC scratch-tree fixtures (example crate
names exercising the gate classifier), not live reads, so they neither break nor need relabel. `cargo metadata --locked` exits
0 and `buck2 targets //...` resolves post-move for the whole affected cone.

**Catalog re-key (16 of 16):** all sixteen moved crates carried a pre-existing gate-bound `registry/catalog/<id>.yaml`
record, each re-keyed by the codemod ArtifactMove to its de-branded `billing-*` id (file rename, content-preserving — the
records carry no embedded package-name field, only `context`/`role`/`capability`/`plane` facets keyed by filename). No record
is left at an old `oya-` stem; the catalog-liveness gate (which scans ONLY `registry/catalog/*.yaml`) stays green (all sixteen
bind to a live workspace crate-id). The SEPARATE `oya/accounting/catalog/*.yaml` product-catalog records (3 files, NOT scanned
by the catalog-liveness gate) are non-crate artifacts of the `oya/accounting` source dir and stay behind for phase-2 (task
#62), exactly like the absorbed dirs' manifest.json/docs/contracts.

**REORG-003 metering-pipeline straggler addendum:** the later REORG-003 slice homes the previously
record-less `libs/oya-metering-pipeline-kernel` crate under `billing/core/metering-pipeline-kernel`
with cargo id `billing-metering-pipeline-kernel`. Because `catalog-liveness` is now born-blocking for
live workspace crates, the slice adds the minimal live catalog row
`registry/catalog/billing-metering-pipeline-kernel.yaml` rather than leaving a new live crate
record-less. That catalog row is a liveness/accounting record only: it carries no runtime-readiness,
SLO, or production claim beyond the existing metering pipeline kernel contract. The crate's
born-accounting file surfaces are `billing/core/metering-pipeline-kernel/BUCK`,
`billing/core/metering-pipeline-kernel/Cargo.toml`,
`billing/core/metering-pipeline-kernel/src/conformance.rs`,
`billing/core/metering-pipeline-kernel/src/lib.rs`,
`billing/core/metering-pipeline-kernel/src/reference.rs`, and
`billing/core/metering-pipeline-kernel/tests/reference_sink.rs`.

**SLO co-move (12, per-service subdirs — collision-avoiding):** two source dirs carry a promotion-gating SLO with the SAME
basename `autosharding-events.openslo.yaml` (`cloud/cloud-billing/slos/` + `cloud/cloud-billing-tax/slos/`), a confirmed
cross-service basename collision (cloud-finops + oya/accounting carry no SLO dir). All twelve SLOs (11 from cloud-billing:
audit-chain-seal-latency, autosharding-events, cap-breach-detection-latency, focus-export-completion-time, fx-lock-freshness,
invoice-generation-time, metering-event-ingest-latency, rev-share-settlement-time, seat-counting-availability,
tenant-class-read-api-latency, usage-aggregation-time; 1 from cloud-billing-tax: autosharding-events) co-move via twelve
content-preserving file `ArtifactMove`s into per-service subdirs
`billing/observability/slos/{cloud-billing,cloud-billing-tax}/<basename>.openslo.yaml` (the §10.16/§10.17/§10.22/§10.24
collision-aware pattern), so 12-in/12-out with neither lost nor shadowed. All twelve SLOs were already non-crate-resident at
merge-base, so the ADR-0563 rename-aware engine relabels them old->new; zero `*.openslo.yaml` remains at any old billing source
path.

**Graph-invisible tests (2 found, both wired-live):** of the `tests/*.rs` across the sixteen moved crates, all but two
already had owning buck2 `rust_test` targets at merge-base (the codemod recomputed their `//`-label deps to the new homes).
The cone scan found TWO graph-invisible `tests/*.rs` in `billing/core/billing` (formerly `oya-cloud-billing-domain`):
`tests/cloud_billing_foundation.rs` (157 lines, exercising `CloudBillingTenantGuardrail` invariants + the paid/trial billing
component surface) and `tests/invoice_lifecycle_transitions.rs` (336 lines, originally a RED acceptance harness for
`get_invoice`/idempotent-transition/credit-note behaviors — all since implemented, so the test is now GREEN). Both are real,
valuable domain tests — they were WIRED-LIVE with new owning `rust_test` targets (`cloud-billing-foundation-test`,
`invoice-lifecycle-transitions-test`; the latter carries the `billing-metering` + `oya-data-boundary-kernel` deps its `use`
requires), compile post-move, and pass (10 sub-tests GREEN). Nothing ignored, skipped, stubbed, or deleted. Every one of the
four external dependents that carries `tests/*.rs` already has an owning `rust_test` target.

**Embedded-asset hermeticity (no change):** none of the sixteen moved crates uses `include_str!`/`include_bytes!` (verified by
corpus grep), so the embedded-asset gate `scan_roots` is NOT extended to `billing` — moving from the `cloud`/`oya` scan_roots
to the unscanned `billing/` home loses zero coverage because there is no embedded asset to scan (the §10.23/§10.24 pattern).

**ADR-justification-source relabel (NO-OP this move).** Unlike §10.24's secrets (whose 13 operator/kms files were
KEEP-justified by ADR-0543's verbatim-path "Governed surfaces" commissioning manifest, requiring an old->new ADR-body
relabel), NONE of the sixteen billing crates' files carries an ADR commissioning/justification SOURCE: every moved-crate row in
the accounting-registry has an EMPTY `justification_ref` (98 rows checked) and no ADR body lists a moved-crate file path as a
KEEP-justification manifest. The only verbatim moved-crate path mentions in ADR-0562 are descriptive narration of PRIOR moves'
dependent blast-radius (e.g. the cell-region move's §10.x list), which accurately describe the tree state at THAT move-time and
are left as historical provenance. Total-accounting per-FILE `unjustified` relabel is therefore handled entirely by the
ADR-0563 §C2 rename-aware engine (manifest file_pairs), no construction fix required.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` +
`buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability
registry records `billing.absorbs_current_dirs` with the capability's own top-level slug `billing` (the self-slug is required
or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE billing/...`; the §10.12/§10.16..§10.24 lesson) plus the eight pre-seeded
absorbed dirs (`cloud/cloud-billing` + `cloud/cloud-billing-tax` + `cloud/cloud-finops` + `oya/accounting` source dirs plus
the four crate-empty phase-2 dirs `oya/oya-billing` + `oya/oya-meter` + `oya/oya-cost` + `oya/finops-portal`, retained). The
membership policy adds `billing` to `scan_roots` + `allowed_top_level_dirs`; the acyclicity policy adds `billing/*/*` to
`crate_root_globs` + `billing` to `unclassified_roots`; the root workspace members glob adds `billing/*/*` (collapsing the
codemod-emitted literals) with `billing/observability` excluded (the non-crate SLO subtree). The move's tracked,
born-accounted artifact roots are the sixteen crate dirs (each carrying its `Cargo.toml`, `BUCK`, `src/`, and, where present,
`tests/`), the co-moved per-service SLO subdirs `billing/observability/slos/<service>/<basename>.openslo.yaml`, the subtree
`billing/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.25 entry (breadth-unlimited per
ADR-0555, covering the whole billing subtree including the co-moved SLO subdirs), the sixteen re-keyed catalog records
`registry/catalog/billing-*.yaml` (reached by the existing `registry/catalog/` reachability prefix), and the committed
move-plan `specs/reorg/billing-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix). The
Wave A metering quota-projection evidence `evidence/multispectrum/wavea-market-billing-metering-quota-20260625-1782430229.json`
is justified by this same billing capability decision; it records a provider-neutral kernel slice and does not amend the move
contract. The
absorbed dirs' other non-crate artifacts (`oya/accounting/catalog/` product-catalog, `cloud/cloud-billing/manifest.json` +
`cloud/cloud-billing-tax/manifest.json`, docs/PRD/IPs/contracts/iac) are homed in phase-2 (task #62), per the §10.5..§10.24
precedent. Per the one-plan-per-PR contract the spent `specs/reorg/secrets-move-plan.json` (the §10.24 move-plan) is removed.

#### §10.26 Twenty-second executed strangler move, sub-batch (a): `intelligence` capability ROOT established (cloud/cloud-intelligence/crates → intelligence/) — first slice of the final + largest capability, sixteen cloud-intelligence crates across three faces, the `rest` face-inversion corrected by enumeration, the runtime-coupled bin/metric names retained for phase-2

The twenty-second REAL codemod run begins the `intelligence` capability — the FINAL + LARGEST capability (the AI/intelligence substrate: SDK adapters, the OAuth subscription-pool request pipeline + proof layer, agentic capabilities). It is decomposed into ~7 serial sub-batch PRs; this sub-batch (a) ESTABLISHES the `intelligence/` capability root and homes ONLY the sixteen `cloud/cloud-intelligence/crates` crates from ONE source dir under the §3 placement rule, across THREE faces (`core` 4 / `adapters` 10 / `facade` 2; `ports/` receives crates in later sub-batches). The remaining 126 `oya/intelligence` crates (+ `oya/detection`) follow in sub-batches (b)-(g); the capability registry maps them via `intelligence.absorbs_current_dirs` so the PARTIAL state stays membership-consistent (those crates remain mapped to `intelligence` while physically under `oya/intelligence`, and the membership + acyclicity gates verify 0-NEW-regression at the partial tip).

Sub-batch (b), tracked by issue #1338 and PR #1335, moves the OpenAPI domain into `intelligence/core/openapi-domain` while preserving the existing live contract location; its canonical Buck package marker is the exact path `contracts/openapi/foundry/BUCK`, which is part of this move's justified and accounted surface rather than a new product or governance brand.

**Substrate layering (the run face, §4 run seam; ports/adapters/facade -> core, never inverted; verified by ENUMERATION because the acyclicity gate's owning_service() recognizes only `cloud/`+`oya/` and is STRUCTURALLY BLIND to `intelligence/*` edges):** the four kernels — the OAuth seat-pool state machine (`oya-cloud-intelligence-kernel` -> `intelligence/core/kernel`, cargo `intelligence-kernel`), the tool-compat kernel (`oya-cloud-intelligence-tool-compat-kernel` -> `intelligence/core/tool-compat-kernel`, `intelligence-tool-compat-kernel`), the translation kernel (`oya-cloud-intelligence-translation-kernel` -> `intelligence/core/translation-kernel`, `intelligence-translation-kernel`), and the wire kernel (`oya-cloud-intelligence-wire-kernel` -> `intelligence/core/wire-kernel`, `intelligence-wire-kernel`) — are pure-serde leaves with ZERO intra-capability path-deps -> `core/`; the cedar-authz / codex / gemini / openbao provider adapters, the clickhouse + valkey EventSink adapters, the claude-agent + codex vendor SDK transports, the ops-infrastructure crate, and the axum reverse-proxy `rest` crate (`oya-cloud-intelligence-rest` -> `intelligence/adapters/rest`, `intelligence-rest`) are the transient infra/SDK impls -> `adapters/`; and the `app` composition root (the `cloud-intelligence` binary; `oya-cloud-intelligence-app` -> `intelligence/facade/app`, `intelligence-app`) plus the `worker` (`oya-cloud-intelligence-worker` -> `intelligence/facade/worker`, `intelligence-worker`) are the sold/runnable facade. **FACE-INVERSION CORRECTION (the §10.18 move-18 blind-gate lesson, enumerated by hand):** the `rest` crate was a SUGGESTED `ports/` placement but is ACTUALLY `adapters/` — it carries unconditional path-deps on the concrete `intelligence-codex-adapter` + `intelligence-gemini-adapter` (so it cannot be `core`/`ports`) and is depended on by `intelligence-openbao-adapter` (which implements its `SecretProviderStore` trait, so it cannot be `facade` either); `adapters/` is the unique legal whole-crate face. The `SecretProviderStore` -> `ports/` trait extraction is a deferred crate-split refactor (out of scope for a placement-only move, tracked as a follow-up IP). Enumeration of every `intelligence/core/*/Cargo.toml` shows ZERO `../../adapters` / `../../facade` edges; the only non-downward edge is the legitimate `app`(facade) -> {`core`, `rest` + four adapters} composition-root edge.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability registry adds the capability's own top-level slug `intelligence` to the pre-existing `intelligence.absorbs_current_dirs` (the self-slug is required or the membership gate REDs `MEM-NEW-UNMAPPED-CRATE intelligence/...`; the §10.12/§10.16..§10.25 lesson) alongside the pre-seeded `cloud/cloud-intelligence` + `oya/intelligence` + `oya/detection` source dirs (retained while the later sub-batches drain them). The membership policy adds `intelligence` to `scan_roots` + `allowed_top_level_dirs`; the acyclicity policy adds `intelligence/*/*` to `crate_root_globs` + `intelligence` to `unclassified_roots`; the embedded-asset hermeticity policy adds `intelligence` to `scan_roots` (preserving hermeticity coverage of the moved crates' `include_str!` sites — the cedar `mapped_srcs` policy include, the codex schema, the kernel capability-parity JSON — the §10.18 iam precedent); the root workspace members glob adds `intelligence/*/*` (collapsing the codemod-emitted literals) with `intelligence/observability` excluded (the non-crate SLO subtree). The move's tracked, born-accounted artifact roots are the sixteen crate dirs (each carrying its `Cargo.toml`, `BUCK`, `src/`, and, where present, `tests/`), the co-moved flat SLO dir `intelligence/observability/slos/*.openslo.yaml` (single source dir, no per-service collision; the SLO metricSource queries retain the runtime-coupled `oya_cloud_intelligence_*` Prometheus metric names + `service="cloud-intelligence"` label — a coordinated observability rename deferred to phase-2, the same runtime-coupling class as the retained `cloud-intelligence` bin name in the `Dockerfile` ENTRYPOINT + k8s workload image), the subtree `intelligence/OWNERS` (axis-cloud-platform) seeded via a `specs/reachability-registry.json` §10.26a entry (breadth-unlimited per ADR-0555, covering the whole intelligence subtree including the co-moved SLO dir), the eight re-keyed catalog records `registry/catalog/intelligence-*.yaml` (reached by the existing `registry/catalog/` reachability prefix; eight of the sixteen moved crates carried a pre-existing record, all re-keyed; the other eight carry none), and the committed move-plan `specs/reorg/intelligence-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix). The §10.18 move-18 ADR-justification-source relabel applies: the commissioning ADR `ADR-0542` carries a verbatim-path justification manifest listing the moved cloud-intelligence files, relabeled old->new in lockstep so the moved files stay justified. The absorbed dir's other non-crate artifacts (`cloud/cloud-intelligence/manifest.json`, docs/PRD/IPs/contracts/iac/k8s/policy/`Dockerfile`) + the remaining 126 `oya/intelligence` crates are homed in later sub-batches / phase-2 (task #62), per the §10.5..§10.25 precedent. Per the one-plan-per-PR contract the spent `specs/reorg/billing-move-plan.json` (the §10.25 move-plan) is removed. Sub-batch (a)'s test-wiring settle additionally born-accounts five NEW test-support artifacts created to make the moved crates' previously graph-invisible tests buck-wireable (the ADR-0554 affected-set wire-or-delete requirement, satisfied by wiring not deleting): the shared in-process fake-CLI test harness `intelligence/adapters/claude-agent-sdk/tests/support_fake_cli.rs` (consumed by the six hermetically-rewritten claude-agent-sdk fake-cli tests, replacing the non-hermetic python-subprocess harness); the hermetic in-process rewrite of the last python test `intelligence/adapters/claude-agent-sdk/tests/assistant_worker_fake_cli.rs` (the former in-`src/` `#[cfg(feature="network")]` `run_assistant_worker` test that wrote a `#!/usr/bin/env python3` fake CLI, converted to a `tests/`-dir integration test driving the same `support_fake_cli.rs` in-process Rust fake over the SDK's `spawn_claude_code_process` seam — eliminating the final python in the moved tree, the strictly-Rust/hermetic founder bar); and the test-only third-party forcing crate at `intelligence/testing/third-party-test-deps/` — `intelligence/testing/third-party-test-deps/Cargo.toml`, `intelligence/testing/third-party-test-deps/src/lib.rs`, and `intelligence/testing/third-party-test-deps/BUCK` — which pulls `httpmock` + `proptest` into the buck2 third-party graph so the six httpmock integration tests and the nine kernel proptest/loom invariant tests compile and run under buck2 (the reindeer-vendored forcing-crate pattern, `httpmock` `default-features=false` to drop the cookies->basic-cookies->lalrpop edge that broke the first attempt, build-green verified before wiring). These five are reached by the breadth-unlimited `intelligence/` reachability prefix seeded in §10.26a.

#### §10.27 De-brand strangler MOVE-1: `messaging` capability FLOOR (libs/oya-messaging-substrate-kernel → messaging/core/substrate-kernel) — single-crate follow-on that homes the substrate kernel left in the junk-drawer by §10.5

The `messaging` capability root was ESTABLISHED by §10.5 (the two `oya/eventing` crates), but the messaging substrate FLOOR — the `oya-messaging-substrate-kernel` crate the bus/queue/stream boundary kernels are built over — stayed in the `libs/` junk-drawer at that time. This move homes it into the already-established capability. It is an ADR-0083 kernel (pure-types + sync-traits, EMPTY `[dependencies]`) and a dependency SINK (zero out-edges), so it lands at FACE `core` per ADR-0570 (a cutover-stable kernel-tier port is PERMITTED in `core`; the placement mirrors the sole pre-existing leaf `messaging/core/domain`, §10.5). The de-brand drops the `oya-messaging-` prefix to the capability slug per ADR-0532/0533 (cargo name = de-branded path tail): `oya-messaging-substrate-kernel` → `messaging-substrate-kernel`.

**Leaf-first rewire (the sink has no out-edges; its three consumers stay in `libs/`):** the crate's three dependents — `libs/oya-bus-boundary-kernel`, `libs/oya-queue-boundary-kernel`, `libs/oya-stream-boundary-kernel` — are NOT moved by this PR (they are de-branded in their own later move); the codemod recomputes their cargo path-deps (`path = "../../messaging/core/substrate-kernel"`), BUCK labels (`//messaging/core/substrate-kernel:messaging-substrate-kernel`), and Rust `use messaging_substrate_kernel::…` idents mechanically. The resulting `libs/` → `messaging/` edge is ACYCLIC and introduces no layer inversion: the moved crate is a sink (no back-edge), and both `libs` and `messaging` are `unclassified_roots` in the acyclicity policy (`owning_service()` recognizes only `cloud/`+`oya/` and is STRUCTURALLY BLIND to `messaging/*` edges), so the edge is ALLOWED (the §10.19 audit precedent for a preserved cross-capability boundary-kernel edge).

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run (`cargo metadata` + `buck2 targets //...` both resolved post-move on a shadow tree, `buck_ok=true` not null, `clean=true`). The capability registry RETIRES the now-stale `libs/oya-messaging-substrate-kernel` membership glob from the closed `messaging` membership entry (the crate is now self-owned by the `messaging/` capability dir via `messaging.absorbs_current_dirs`, exactly as `messaging/core/domain` is); the three boundary-kernel globs stay. No membership/acyclicity POLICY data changes (the `messaging` root, `messaging/*/*` crate_root_glob, and root workspace `messaging/*/*` member glob were all seeded by §10.5, so `root_workspace_changed=false` — the glob already covers `messaging/core/substrate-kernel`). The move's tracked, born-accounted artifact paths are `messaging/core/substrate-kernel/Cargo.toml`, `messaging/core/substrate-kernel/BUCK`, `messaging/core/substrate-kernel/src/lib.rs`, `messaging/core/substrate-kernel/src/conformance.rs`, `messaging/core/substrate-kernel/src/reference.rs`, and `messaging/core/substrate-kernel/tests/reference_substrate.rs`; the newly-authored catalog record `registry/catalog/messaging-substrate-kernel.yaml` (`role: kernel`, mirroring `registry/catalog/messaging-domain.yaml`; reached by the existing `registry/catalog/` reachability prefix — the crate becomes catalog-governed under `messaging/`, so `ci/facade/service-catalog-parity` requires the row); and the committed move-plan `specs/reorg/messaging-substrate-kernel-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix). Unlike §10.26, this PR removes NO prior spent plan: the two committed plans that remain on `dev` — `specs/reorg/ci-move-plan.json` (#1216) and `specs/reorg/iam-pdp-cedar-move-plan.json` (#1184) — are LOAD-BEARING (read directly by `ci/facade/*` tests / `gate_registration.rs`) and are already excluded by the codemod's `plan_is_landed` merge-base carve-out (every old crate-dir of both is absent at the merge-base), so exactly ONE active move-plan (this one) remains and the single-active-plan invariant holds.

#### §10.28 De-brand strangler MOVE-2: `messaging` capability BOUNDARY SURFACES (libs/oya-{bus,queue,stream}-boundary-kernel → messaging/core/{bus,queue,stream}-boundary-kernel) — three-crate batch completing the trio whose floor landed in §10.27

§10.27 homed the messaging substrate FLOOR and explicitly deferred its three consumers ("they are de-branded in their own later move"). This move is that follow-on, and it completes the ADR-0536 D-13 queue/stream/bus trichotomy inside the capability: after it, no messaging crate remains in the `libs/` junk-drawer.

**Batched, not serialized, because each crate is a LEAF.** All three were verified to have ZERO fan-in — no other `Cargo.toml` or `BUCK` in the tree names `oya-bus-boundary-kernel`, `oya-queue-boundary-kernel`, or `oya-stream-boundary-kernel` — so no referrer breaks and the relocation carries none of the dual-state cost that makes a non-leaf relocation atomic-per-unit. Each depends only on `messaging-substrate-kernel`, already inside the capability, so the move SHORTENS the intra-capability path-dep (`../../messaging/core/substrate-kernel` → `../substrate-kernel`) and adds no cross-capability edge. This also RETIRES the `libs/` → `messaging/` edge §10.27 had to justify: with the consumers moved in, that edge no longer exists (verified: zero `libs/**/Cargo.toml` carry a `messaging/` path-dep and zero `libs/**/BUCK` reference a `//messaging` label), so the §10.19 cross-capability-edge precedent §10.27 leaned on is no longer load-bearing **for that specific edge**. `messaging/` still has three inbound cross-capability edges — from `marketplace/facade/dev-cli`, `audit/core/usecase`, and `oya/application/crates/oya-application-app`, all into `messaging-domain`/`messaging-file-adapter` — which this move does not touch.

**The de-brand here is name-and-path ONLY; wire identifiers are deliberately deferred.** `EVENT_TYPE_HEADER = "oya-event-type"` and the `oya-bus.` / `oya-queue.` / `oya-stream.` topic prefixes are unchanged in all three crates. These are on-the-wire values: renaming a topic or a header is a behavior change and must not ride along inside a relocation, where it would be invisible among the mechanical path edits. A later reader should not "finish the de-brand" by rewriting them without a compatibility plan. They trip no gate: the `cloud-ci-brand-residue` forbidden-stem list is four retired product/vendor names held in `libs/oya-check-brand-residue/src/forbidden_vocab.rs`, and a bare `oya` is not among them. (Those four stems are deliberately NOT quoted here. That gate is `baseline-block-on-new` and counts occurrences per file, so naming them in prose registers four new occurrences against this very file — which is exactly how the first revision of this section failed CI.)

**Face = `core`, decided on evidence rather than by analogy.** All three crates define ZERO `pub trait` items: they are generic surfaces (`WorkQueue<'a, S: MessagingSubstrate>`, `EventStream<'a, S>`, `EventBus<'a, S>`) composed OVER the substrate port, not port definitions. The `ports` face ("capability traits; the stable seam") therefore does not describe them. ADR-0570 permits a cutover-stable kernel-tier surface in `core` and its litmus — *would this interface change at owned-stack cutover?* — answers no, since these are precisely the owned surfaces the transient broker adapters are written against. The placement mirrors both pre-existing messaging leaves (`messaging/core/substrate-kernel` §10.27, `messaging/core/domain` §10.5). De-brand per ADR-0532/0533 (cargo name = capability slug + de-branded path tail): `oya-bus-boundary-kernel` → `messaging-bus-boundary-kernel`, and likewise for queue/stream.

The move was performed by `oya-reorg-codemod-app` (NOT by hand), gated on the buck2-full-tree dry-run with `buck_ok=true` (not null), `cargo_ok=true`, `clean=true`. **The `buck_ok` distinction is load-bearing and was observed live on this move:** invoked via `buck2 run`, the codemod's own dry-run reported `buck_ok: null` with `"buck2 not on PATH; cargo-only dry-run"` — the binary does not inherit an ambient `PATH` under `buck2 run`, so the buck2 half of the proof silently did not execute while the run still exited 0. The gating dry-run was therefore re-executed against the built binary directly with `PATH` exported, which is what produced `buck_ok=true`. This is the `oracle.rs` fail-OPEN degradation (a cargo-only result presented as a clean dry-run) and it is a false-green in exactly the place the move is about to be trusted; it is filed as a follow-up rather than fixed here, so this move's evidence stands on the re-run, not on the first invocation.

No membership/acyclicity POLICY data changes: the `messaging` root, the `messaging/*/*` `crate_root_glob`, and the root-workspace `messaging/*/*` member glob were all seeded by §10.5, so `root_workspace_changed=false` — the glob already covers the three new leaves. The capability registry RETIRES the three now-stale `libs/oya-{bus,queue,stream}-boundary-kernel` membership globs from the closed `messaging` membership entry (the crates are now self-owned by the `messaging/` capability dir via `messaging.absorbs_current_dirs`), exactly as §10.27 retired the substrate-kernel glob; that empties the `messaging` `absorbs_current_crate_globs` entry, which is removed rather than left as a dead husk. The move's tracked, born-accounted artifact paths are enumerated below. They are spelled out one per bullet, byte-exactly and unbraced, because the total-accounting producer indexes decision bodies into path-like tokens and keeps only tokens that EQUAL a tracked path — a braced, globbed, abbreviated, or line-wrapped path matches nothing and leaves the artifact `unjustified`. This section originally wrote them as `messaging/core/{bus,queue,stream}-boundary-kernel/...`, which registered zero of them.

The three newly-authored catalog records (`role: kernel`, mirroring `registry/catalog/messaging-substrate-kernel.yaml`; reached by the existing `registry/catalog/` reachability prefix — note that reachability and justification are SEPARATE gates, and the prefix satisfies only the former):

- `registry/catalog/messaging-bus-boundary-kernel.yaml`
- `registry/catalog/messaging-queue-boundary-kernel.yaml`
- `registry/catalog/messaging-stream-boundary-kernel.yaml`

The relocated crate files (three per crate; these crates carry no `tests/` dir today):

- `messaging/core/bus-boundary-kernel/Cargo.toml`
- `messaging/core/bus-boundary-kernel/BUCK`
- `messaging/core/bus-boundary-kernel/src/lib.rs`
- `messaging/core/queue-boundary-kernel/Cargo.toml`
- `messaging/core/queue-boundary-kernel/BUCK`
- `messaging/core/queue-boundary-kernel/src/lib.rs`
- `messaging/core/stream-boundary-kernel/Cargo.toml`
- `messaging/core/stream-boundary-kernel/BUCK`
- `messaging/core/stream-boundary-kernel/src/lib.rs`

And the committed move-plan `specs/reorg/messaging-boundary-kernels-move-plan.json` (reached by the existing ADR-0563 `specs/reorg/` reachability prefix).

**The catalog rows are REQUIRED, not decorative, and this is the move's one non-mechanical obligation.** `catalog_liveness.workspace_member_globs` defaults to a fifteen-root list that includes `messaging/**` (`libs/oya-ci-config/src/lib.rs:1081`), so a live workspace member under `messaging/` must carry a `registry/catalog/<crate_id>.yaml` row or an explicit exemption; `libs/**` is NOT in that list, which is why these three crates needed no row at their old home and need one now. The violation code `catalog_live_crate_without_row` is dispositioned `frozen_empty: true`, so a missing row cannot be laundered into the accepted baseline by regeneration — it is a hard RED. Any future move INTO one of those fifteen roots inherits the same obligation, and it is invisible from the diff: the codemod relocates crates but does not author catalog rows. Like §10.27, this PR removes no prior spent plan: every committed plan on `dev` is already excluded by the codemod's `plan_is_landed` merge-base carve-out, so exactly ONE active move-plan (this one) remains and the single-active-plan invariant holds.

**Codemod defect found and worked around, not absorbed.** The codemod's path-rewriting pass edited §10.27's HISTORICAL prose, rewriting "the crate's three dependents — `libs/oya-bus-boundary-kernel`, … — are NOT moved by this PR" into the post-move paths, which makes that sentence assert something false about what §10.27 did. Narrative ADR text is a record of a past state, not a live reference to be relabeled. The mechanical edit to §10.27 was REVERTED and this section appended instead; the defect (path rewriting must exclude ADR narrative bodies, or at minimum the `docs/decisions/` prose outside frontmatter and governed path lists) is filed as a follow-up.

#### §10.29 De-brand strangler MOVE-3: the `os/` meta directory (cloud/cloud-os/crates → os/) — 41-crate single-block batch, the rung-1 node OS, zero catalog work in either direction

(Section numbering assumes §10.28, the messaging boundary-surfaces batch, lands first. Both were authored concurrently and both append here; if this one lands first, renumber.)

**Why 41 — and why 41 is not a preference.** The move-plan schema carries exactly ONE `capability` field and exactly one plan may be active, so a batch **cannot span destination roots**. Batch size is therefore not a dial to be tuned for risk appetite: it is the size of one destination block. Of the remaining blocks — governance 128, app 110, intelligence 109, unmapped 60, **os 41**, build 22, kernel 20, ci 12, data 5 — `os/` is the only one in the 40–70 window. There is no "next crate excluded": crate 42 would have to come from a different destination root, which the schema forbids.

`os/` is also uniquely cheap, and this is the reason to take it now rather than later: **`os/` is not one of the fifteen `catalog_liveness.workspace_member_globs` roots, and none of the 41 crates has a `registry/catalog/*.yaml` row today.** So this batch authors zero catalog records and renames zero — against 91–128 catalog renames for every larger block. It is the only 40+ block in the repo with zero catalog work in both directions. (Contrast §10.28, where three crates moving INTO a governed root created three mandatory rows; the obligation is a property of the destination, never of the crate.)

**Faces, decided on evidence.** `os/core/` takes 40 crates: every one is node-OS runtime code, and the `core` charter is "the engine we RUN (substrate face)". Two calls worth recording because the first reading was wrong: `init-app` lands in `core`, NOT `facade`, because the facade charter is "the multi-tenant surface we SELL" and an OS init/PID-1 is not sold multi-tenant (`kernel/` likewise has no facade). `proto-api` lands in `core`, NOT `ports`, despite defining seven public traits — every implementor of those traits lives *inside `proto-api` itself*, so they are internal trait+impl pairs rather than a cross-crate seam, and the `ports` charter ("capability traits; the stable seam") does not describe them. `os/harness/difftest-app` takes the one integration-test-only crate, mirroring the landed `kernel/harness/asterinas-real-boot` precedent.

**Why `os/` shows only two faces when every landed CAPABILITY shows four — and why that is the rule, not a shortcut.** The obvious objection to this batch is that it looks like a relocation rather than an implementation of the decided clean-architecture shape: `iam` decomposes into 69 crates across `core`/`ports`/`adapters`/`facade`, `storage` and `k8s` and `network` likewise, while `os/` lands 40 crates in `core` and 1 in `harness`. The answer is §3, which is FIRST MATCH WINS: rule **1** sends the node OS to `os/`, and the face rule is rule **6**, scoped explicitly to "within that **capability**". `os/` is a META directory (`owns_crates: true`), absent from the closed 24-capability registry — so rule 6 never reaches it. The governing precedent is the only other crate-owning meta dir, `kernel/`, which lands exactly `core/` + `harness/`. Matching it is the conforming outcome; decomposing `os/` into four capability faces would be the deviation.

**The face calls were still made on dependency evidence, because a meta dir is not a licence to skip the analysis.** Four crates look port- or adapter-shaped by NAME and were checked individually rather than waved through. `etcd-domain`, `platform-domain` and `runtime-cri-domain` carry NO transient third-party infra dependency — `etcd-domain`'s entire dependency set is `os-kernel` plus workspace lints. They are domain models ABOUT those subsystems, not vendor impls of them, and the `adapters` charter ("transient-infra impls; vanish at owned-stack cutover") does not describe a crate with no infra dep to vanish. `proto-api` is the genuinely arguable one: `*-api` lands in `ports/` everywhere else (`iam/ports/identity-api`, `storage/ports/object-api`), and it defines seven public traits. It stays in `core/` because every implementor of all seven lives INSIDE `proto-api` itself — an internal trait+impl pair is not a cross-crate seam, and §10.26 settled that face is decided by dependency evidence rather than by crate name. If a second crate ever implements one of those traits, `proto-api` becomes a real port and should move.

**Deferred, and named so it is not mistaken for finished.** Extracting a trait into its own `ports/` crate is a crate-SPLIT refactor, which §10.26 established is "out of scope for a placement-only move" when it deferred the `SecretProviderStore` extraction on exactly these grounds. Nothing in this batch splits a crate. If the owned-stack cutover later replaces the upstream-k8s-facing surfaces (`kubelet-domain`, `kubernetes-domain`, `k8s-control-domain`, `runtime-cri-domain` — ADR-0510 transitional), the crates that grow a vendor dependency at that point become the `os/adapters/` population, and the trait they are written against becomes `os/ports/`. That is a real follow-up, not a hypothetical, and it is deliberately not this PR.

**This move also closes a registry inaccuracy it would otherwise have deepened.** The closed `faces` list declared exactly four faces; `harness` was absent, even though ADR-0611 landed `kernel/harness/asterinas-real-boot` on `dev`. Nothing rejected it — the `faces` array has zero code consumers, so no gate reads it — which is precisely why the drift survived: a closed registry that nothing enforces will describe the tree less and less accurately, silently. Rather than become the second offender, this amendment adds the fifth charter row, scoped to the META directories whose runtime engines have no sold surface. A capability still keeps its tests beside the crate they exercise; `harness` is not a capability face.

**Verification.** Codemod dry-run `clean=true`, `cargo_ok=true`, `buck_ok=true` (the binary was invoked directly with `PATH` exported — under `buck2 run` the oracle fails OPEN to a cargo-only proof while still exiting 0; see the follow-up filed against that behaviour). Workspace membership was proven unchanged by resolving the member globs against the tree on both sides: **903 resolved members before, 903 after, zero lost, zero unexpectedly gained, and all 41 new paths are members.**

**The codemod removed a member glob, and that is correct.** `cloud/*/crates/oya-*` was dropped from the root manifest because after this move it matches nothing unexcluded — the only remaining match is `cloud/cloud-kernel`, which is an EXCLUDED separate no_std workspace. Verified by resolution, not by inspection. The `os/*/*` glob was added by hand BEFORE the codemod ran; without it the codemod appends literal-path members, which is `workspace_member_explicit_path`, a `frozen_empty` violation that cannot be baselined.

**Five hand edits the codemod does not perform**, each because it rewrites crate identifiers and paths but not policy DATA that happens to name them: `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json` (relabel the `init-app` skip entry) and `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json` (add `os` to `scan_roots`) — **these two must land TOGETHER**: that gate asserts measured-skips == baseline by SET EQUALITY, so the relabel alone leaves `os/` unscanned (measured 3 vs baseline 4) and the `scan_roots` addition alone leaves measured holding the new key against a baseline holding the old; either alone is RED and only both are green. Then `specs/reachability-registry.json` (the trustd `OWNERS` file rides the `git mv`; its registry entry does not), `ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json` (add the crate-root glob and the unclassified root, or the 41 crates silently leave the gate's corpus — `owning_service()` returns `None` outside `cloud/`/`oya/`, so `unclassified` is the correct class, exactly as for `libs/`, `tools/`, `ci/`), and `specs/masterplan.json` (the ADR-0537 rung-1 `source_anchors` gains `os`; the gate asserts only that the array is non-empty strings, so this is a correctness edit rather than a gate requirement).

**A sixth edit was attempted, REJECTED by a gate, and reverted — recorded because the reasoning that produced it looks correct and is not.** `ci/facade/automation-language-policy` carries two `exclude_prefixes` entries scoped to `cloud/cloud-os/`. Retargeting them to `os/` seems like the obvious way to preserve an existing exemption across a relocation. It is not: `rust_first_automation_scan_scope_narrowing` fires with *"candidate scan exclusions may shrink but must not add a protected-scope blind spot"*. An exclusion is keyed to a PATH, and `os/` was never excluded at the merge-base — so retargeting does not move an exemption, it **creates a new one covering 41 crates** while deleting an old one that now covers nothing. The anti-laundering rule is deliberately asymmetric for exactly this reason, and a relocation is precisely when the laundering would look like bookkeeping.

The entries are therefore left pointing at `cloud/cloud-os/`, where they are now inert. Retiring them is a shrink, which the gate permits, and belongs in its own change where the shrink is the subject rather than a side effect.

**But reverting the exclusion was only half the problem, and the other half is the one worth recording.** A first draft of this section claimed the 41 crates were "scanned by the policy at their new home — verified green (45 tests pass)". Both halves of that were wrong, and the second half was the more dangerous kind of wrong: those 45 tests are the gate crate's own unit suite over policy DATA. They never scan `os/`, so they were never evidence for the claim they were cited for.

The crates were in fact scanned by **nothing**. A gate's scan scope is `roots` MINUS `exclude_prefixes`, and the relocation moved the tree out of the `roots` half while everyone was looking at the `exclude_prefixes` half. At the merge-base, `cloud/cloud-os/**` sat inside the `cloud` root of `scan.roots` and `cli_package_authority.roots`; at `os/**` it sat inside no root at all. **Deleting an exclusion and relocating out of a root produce the identical blind spot, and `rust_first_automation_scan_scope_narrowing` only differences `exclude_prefixes`** — so the loud, well-designed anti-laundering rule that correctly caught the exclusion edit was structurally blind to the larger hole opening beside it. The same shape applies to `caller-supplied-authorization`, whose `scan_roots` also carried `cloud` and not `os`.

Fixed by adding `os` to `scan.roots`, `cli_package_authority.roots`, `interpreter_command_authority.roots`, and `caller-supplied-authorization`'s `scan_roots` — each verified to introduce zero findings before being added, and permitted because the ceiling rule forbids only REMOVING protected terms. `endpoint-authorization-coverage` is deliberately NOT extended: `os/core/apid-domain` calls `self.router.route(req, &self.local_endpoint)`, which matches neither the axum nor the owned-router grammar that gate parses, and it REDs on a `.route(` path it cannot resolve to a concrete string. Extending it needs that grammar handled first.

Two `Command::new("python3")` test bridges were also found under `os/` (`os/core/runtime-cri-domain/src/image_cache.rs`, `os/core/init-app/src/main.rs`), both shelling out to inline Python solely to set a file mtime. They predate this move and were invisible because `interpreter_command_authority.roots` never contained `cloud`. Replaced with `std::fs::FileTimes`, which is why extending that root is now safe rather than a fresh RED.

**The generalizable lesson: a relocation changes a gate's scan scope in two independent ways, and only one of them is ratcheted.** Every future capability move must check that its destination root appears in every `roots`/`scan_roots` allowlist that contained its source root — there is no gate asserting that those allowlists agree with the live top-level directory census, and until there is, this check is manual.

**Known debt, named rather than hidden.** `cloud/cloud-os/manifest.json` stays where it is, with its now-stale `crates[]` array; the directory is retained so the tier-classification entry keyed on that manifest and the manifest-count floor both stay valid, and re-homing it is a phase-2 concern. The `[[bin]]` target names inside `os/core/init-app/BUCK` keep their pre-move spelling: the codemod deliberately never clobbers a `-bin` sibling's name (documented and unit-tested in its `buck.rs`), and §10.26 set the same precedent for the runtime-coupled names it retained for phase-2. Historical records under `docs/plans/` and `evidence/` that name the old paths are left untouched, for the same reason §10.28 reverted a mechanical rewrite of §10.27's prose: a record of a past state must not be relabeled.

**The only tracked artifact this move CREATES** — every other changed file is either a rename the ADR-0563 rename-aware per-file relabel carries, or an edit to a file that already exists — is the committed move-plan:

- `specs/reorg/os-move-plan.json`

That path is spelled byte-exactly and unbraced, because the total-accounting producer keeps only path-like tokens that EQUAL a tracked path; §10.28 originally brace-expanded its new paths and registered none of them.

#### §10.30 De-brand strangler MOVE-4: the `ci` capability's FIRST core-face leaf (oya/ci-tide/crates/oya-ci-tide-kernel → ci/core/tide-kernel) — single-crate leaf-first move, consumers rewired in place

§10.14 homed the cloud-ci gate fleet into `ci/facade/` and established the `ci` capability root, but the merge-queue component's pure-domain kernel stayed under `oya/ci-tide/crates/`. This move homes it, and it is the capability's first `core/` leaf (`ci/` carried only `adapters/`, `facade/`, and `ports/` before).

**Face = `core`, on evidence.** The crate is an ADR-0083 kernel — pure types plus sync traits, no I/O, no async, no network, its only cargo dependency `serde` (its BUCK target additionally names `third-party//:serde_json`) — and a dependency SINK with zero intra-repo out-edges in either the cargo or the buck2 graph. ADR-0570 permits a cutover-stable kernel-tier surface in `core`, and its litmus (*would this interface change at owned-stack cutover?*) answers no: the eligibility predicate is written against the owned merge-queue model, not against a vendor forge API. **The `ports` question is answered honestly, and NOT by citing §10.29's `proto-api` call.** That call turned on the traits' implementors living *inside* `proto-api` itself; here the one public trait, `ForgeClient`, is implemented by a DIFFERENT crate (`oya-ci-tide-github-adapter`), so the seam genuinely is cross-crate and §10.29's reasoning does not transfer — it points the other way ("if a second crate ever implements one of those traits, `proto-api` becomes a real port and should move"). What keeps this crate out of `ports` is that the `ports` test is CONJUNCTIVE: a trait/DTO-ONLY crate whose implementors live elsewhere. This crate is not trait-only. Its 1140-line source exposes the `is_mergeable` eligibility predicate plus the config, forge-state, review, and status types that predicate decides over — `ForgeClient` is one of the twenty-three top-level `pub` items, declared last. It is the engine we RUN, which is the `core` charter. Extracting `ForgeClient` into a `ci/ports/` seam is the correct end state, and it is a crate-SPLIT refactor that §10.26 established is out of scope for a placement-only move: it is named here as the real follow-up, exactly as §10.29 named its own, and it is deliberately not this PR. De-brand per ADR-0532/0533 (cargo name = capability slug + de-branded path tail): `oya-ci-tide-kernel` → `ci-tide-kernel`.

**LEAF-FIRST, and the deferral is a gate fact rather than a preference.** The two consumers — `oya/ci-tide/crates/oya-ci-tide-app` (the poll-loop composition root and its binary) and `oya/ci-tide/crates/oya-ci-tide-github-adapter` (the forge HTTP client) — are NOT moved by this PR. They stay where they are and get their cargo path-dep, BUCK label, and Rust `use`-ident rewired into `ci/core/tide-kernel`, exactly as §10.27 rewired the three `libs/` consumers of the messaging floor. One consequence of the ident rewrite is worth naming so it is not read as an unrelated edit: in `oya/ci-tide/crates/oya-ci-tide-app/src/bin/oya-ci-tide.rs` the renamed `use` line SORTS differently (`ci_tide_kernel` now precedes `oya_ci_tide_app`), and `rustfmt`'s default `reorder_imports` moves it to the head of the group. Rewriting the ident in place leaves that file `rustfmt --check` RED against the declared `cargo fmt --all -- --check` quality lane, so the reorder is part of the mechanical rename, not a stylistic edit; the file was `rustfmt`-clean at the merge-base and is `rustfmt`-clean here. The reason they are deferred is `ci/facade/facade-core-layering`: unlike the `intelligence` and `compute` capabilities whose facade→core edges were admitted under `facade_core_no_ports_layer`, `ci/` already HAS a `ports/` layer (`ci/ports/path-resolver`), so an app crate landing at `ci/facade/tide-app` with a `//ci/core/tide-kernel:ci-tide-kernel` BUCK edge would emit a NEW `facade_core_direct_dep` key against a shrink-only baseline that exists precisely to make a new one impossible to ship. Routing the app through a `ports/` seam is a crate-SPLIT refactor, which §10.26 established is out of scope for a placement-only move; it is the real follow-up, and it is deliberately not this PR. The resulting `oya/` → `ci/` edge is acyclic (the moved crate is a sink) and introduces no layer inversion; `ci` is already an `unclassified_roots` entry in the tier-dependency acyclicity policy, whose `owning_service()` recognizes only `cloud/`+`oya/` and is structurally blind to `ci/*` endpoints.

**No policy DATA changes are induced.** The root workspace `ci/*/*` member glob, the `ci/*/*` `crate_root_glob`, and the `ci` `unclassified_roots` entry were all seeded by §10.14, so the destination is a member and a gate-corpus resident by construction (`root_workspace_changed=false`). The §10.29 scan-scope lesson was applied by enumeration rather than by assertion: every `roots`/`scan_roots`/`crate_root_globs` allowlist under `specs/`, `ci/`, `libs/`, `registry/`, `build/`, `tools/`, and `.github/` that admits the source root `oya` also admits `ci` — with exactly ONE exception, `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json` (`scan_roots` lists `oya` but not `ci`). That exception is not a blind spot this move opens: the relocated crate contains no `include_str!`/`include_bytes!` and ships no non-Rust asset (its three files are `Cargo.toml`, `BUCK`, `src/lib.rs`), so it contributes nothing to that gate's corpus at either address. The pre-existing gap — every `ci/facade/*` gate crate is likewise outside that scan root — is recorded here as a follow-up, not repaired inside a placement-only move. The capability registry's `ci.absorbs_current_dirs` retains `oya/ci-tide`, which is correct and not stale — the two consumer crates still live there, and §10.26 set the precedent of retaining a source dir while later slices drain it.

**The catalog obligation, and why it is authoring rather than re-keying.** The crate carried no `registry/catalog/*.yaml` row: it was one of the 197 entries in the shrink-only `uncatalogued` baseline of `ci/facade/crate-catalog-coverage`. A moved crate has a NEW package name that is absent from that baseline, and that file's own contract is explicit that removing a name is burn-down and always allowed while ADDING one is not — so the move must author the row rather than relabel the baseline entry. `oya-ci-tide-kernel` is therefore removed from `uncatalogued` (which also avoids the stale-entry code firing on a listed crate that no longer exists) and `_provenance.uncatalogued_total` is decremented from 197 to 196 to keep the declared total equal to the array it describes. The two consumer crates keep their names, so their two baseline entries stay untouched.

**Content was NOT edited, and one known-stale citation is deliberately carried forward.** This is a relocation: only the path, the cargo package name, the `[lib]`/BUCK crate identifiers, the BUCK target names, and the one doc-comment occurrence of the package name changed. The crate header and module docs cite a decision that has since been superseded; that citation is left exactly as it stands. Re-anchoring it is an editorial change that would be invisible among the mechanical path edits and would erase the crate's provenance link to the decision it was actually built under — the §10.28 wire-identifier precedent and the §10.29 historical-record precedent both apply, and the re-anchor belongs in its own change where it is the subject. The prose in the manifest header and the module docs still reads `oya-ci` ("the `oya-ci` tide component"), and that is correct rather than an incomplete de-brand: `oya-ci` there names the LIVE CI product and its protected context — the same `oya-ci-required` string this crate declares as `DEFAULT_REQUIRED_STATUS_CONTEXT` — not the cargo package. ADR-0532/0533 de-brand the package/lib/target identifiers, which this move did; the product name is not a de-brand target, and rewriting prose would in any case be the content edit this paragraph rules out. For the same reason the dated audit snapshots under `docs/audit/` that name the old crate path are left untouched: a record of a past state must not be relabeled. Three references survive there — `docs/audit/initial-sweep-2026-06-06/architecture/10-oya-verticals-a.md:149`, `docs/audit/initial-sweep-2026-06-06/HANDOFF.md:68`, and `docs/audit/initial-sweep-2026-06-06/_phase0/PRODUCER-DRAFT.md:91` — and they are the ONLY surviving occurrences of `oya-ci-tide-kernel` outside this section and the move-plan that records the rename.

**The tracked artifacts this move CREATES** — every other changed file is either a rename the ADR-0563 rename-aware per-file relabel carries, or an edit to a file that already exists — are the newly-authored catalog record (`role: kernel`, mirroring `registry/catalog/messaging-substrate-kernel.yaml`; reached by the existing `registry/catalog/` reachability prefix) and the committed move-plan (reached by the existing ADR-0563 `specs/reorg/` prefix):

- `registry/catalog/ci-tide-kernel.yaml`
- `specs/reorg/ci-tide-kernel-move-plan.json`

The three relocated files, named byte-exactly and unbraced for the same reason §10.28 records:

- `ci/core/tide-kernel/Cargo.toml`
- `ci/core/tide-kernel/BUCK`
- `ci/core/tide-kernel/src/lib.rs`

This move-plan is a NEW file rather than an append to `specs/reorg/ci-move-plan.json`. That plan is LANDED — every one of its move `old_path` entries is absent at the merge-base — so it is INERT and contributes no manifest pairs. Appending a pending move to it would flip `plan_is_landed` false for the whole plan and re-arm 46 stale relabel pairs for gate crates that moved in immutable history, handing out unearned baseline-substitution licences; the MUST-PASS #5 landed-plan carve-out exists to stop exactly that. With this file added, exactly ONE active move-plan exists and the single-active-plan invariant holds.

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
- ci/facade/service-tier-metadata/BUCK — buck2 build targets for the born-blocking tier-field-coverage gate
- ci/facade/service-tier-metadata/Cargo.toml — Cargo manifest for the born-blocking tier-field-coverage gate crate
- ci/facade/service-tier-metadata/src/lib.rs — pure kernel + collector for the tier-field-coverage gate (ADR-0562/ADR-0536/ADR-0245)
- ci/facade/service-tier-metadata/src/main.rs — binary entry point for the tier-field-coverage gate
- ci/facade/service-tier-metadata/src/tests.rs — unit tests for the tier-field-coverage gate kernel
- ci/facade/service-tier-metadata/tests/tier_field_coverage.rs — integration tests including live-corpus born-blocking-green test
- ci/facade/service-tier-metadata/tier-field-coverage-policy.json — policy DATA for the tier-field-coverage gate (enum allowlists, governed roots, minimum manifest count)

Phase-0 also introduces the tier-DEPENDENCY acyclicity gate (ADR-0245/ADR-0280/ADR-0562): the
enforcement surface that asserts the ADR-0245 cross-tier dependency rules + the ADR-0280
intra-substrate S-rank rule + a Tarjan cycle backstop over the REAL crate dependency graph read from
BOTH cargo (path-deps + workspace membership) AND buck (deps/visibility). Because the pre-move tree
carries pre-existing substrate-inversions (the very debt the reorg fixes), the gate is born-ADVISORY
against a FROZEN baseline and enforces NO REGRESSION; it flips to fully blocking when the baseline
burns down to zero. Its tracked artifacts, each justified by this decision (ADR-0562) together with
ADR-0245 and ADR-0280:

- ci/facade/layer-dependency-acyclicity/BUCK — buck2 build targets for the born-advisory tier-dependency-acyclicity gate
- ci/facade/layer-dependency-acyclicity/Cargo.toml — Cargo manifest for the tier-dependency-acyclicity gate crate
- ci/facade/layer-dependency-acyclicity/src/lib.rs — pure kernel + cargo/buck dep-graph collector + tier-rule/S-rank/Tarjan evaluator + frozen-baseline split (ADR-0245/ADR-0280/ADR-0562)
- ci/facade/layer-dependency-acyclicity/src/main.rs — binary entry point + baseline re-freeze (--emit-baseline) for the tier-dependency-acyclicity gate
- ci/facade/layer-dependency-acyclicity/src/tests.rs — unit tests for the tier-dependency-acyclicity gate kernel
- ci/facade/layer-dependency-acyclicity/tests/tier_dependency_acyclicity.rs — integration tests: live-corpus zero-regression GREEN + RED wrong-tier fixture + burn-down fixture
- ci/facade/layer-dependency-acyclicity/tests/fixtures/red-substrate-to-product.json — RED fixture: a synthetic substrate→product edge the gate must fail closed
- ci/facade/layer-dependency-acyclicity/tests/fixtures/burn-down.json — burn-down fixture: a removed baselined inversion the gate must keep green
- ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json — policy DATA for the tier-dependency-acyclicity gate (governed crate-root globs, tier'd service roots, unclassified meta roots, S-rank order, enforcement mode)
- ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-baseline.json — the FROZEN known-debt baseline: the pre-move tier-dependency violations the reorg strangler burns down (the burn-down target)

Phase-0 also introduces the §6 MEMBERSHIP lint (the anti-junk-drawer authority) — born-advisory with
a frozen unmapped baseline, enforcing no-regression (no NEW unmapped crate, no NEW top-level dir
outside the closed set) and the base/-admission rule — plus the registry's `membership_lint_coverage`
extension that closes the per-crate mapping over the whole tree. These tracked artifacts are each
justified by this decision (ADR-0562 §6) together with ADR-0536, ADR-0280, and ADR-0512:

- ci/facade/module-membership/BUCK — buck2 build targets for the born-advisory capability-membership lint
- ci/facade/module-membership/Cargo.toml — Cargo manifest for the capability-membership lint crate
- ci/facade/module-membership/src/lib.rs — pure kernel + crate collector for the capability-membership lint (ADR-0562 §6/ADR-0280/ADR-0512)
- ci/facade/module-membership/src/main.rs — binary entry point for the capability-membership lint
- ci/facade/module-membership/src/tests.rs — unit RED/GREEN fixtures for the capability-membership kernel (crate in no/two capabilities, new top-level dir, base/-admission, frozen-baseline advisory)
- ci/facade/module-membership/tests/capability_membership.rs — integration tests including the live-corpus born-advisory-green self-test and on-disk RED fixtures
- ci/facade/module-membership/capability-membership-policy.json — policy DATA for the capability-membership lint (gate id, registry pointer, scan roots, closed meta-directory + top-level set, ignored build-artifact dirs, minimum crate count)

Wave-D G003/G006 phase-0 foundation evidence is also justified by this ADR because it records the
no-false-green boundary work that keeps the capability reorg honest while workflow, data/ontology,
intelligence, billing, cell/capacity, and supporting cloud substrates are advanced in parallel. The
tracked evidence artifacts are DATA, not generated faces, and must remain under the reviewed
`evidence/` tree rather than `.omx/` runtime state:

- evidence/wave-d-g003-g006/g003/graphql-boundary-evidence.md — GraphQL residue and boundary evidence for the zero-GraphQL capability boundary.
- evidence/wave-d-g003-g006/g003/runtime-state-classification.md — runtime-state classification evidence used to avoid treating OMX/team state as repo source.
- evidence/wave-d-g003-g006/g008/cell-capacity-evidence.md — cell/capacity substrate evidence for the foundation wave.
- evidence/wave-d-g003-g006/g008/data-storage-no-op-evidence.md — data/storage no-op evidence that documents the verified absence of a merge-safe runtime slice.
- evidence/wave-d-g003-g006/g009/workflow-no-op-evidence.md — workflow substrate no-op evidence for the parallel foundation wave.
- evidence/wave-d-g003-g006/integration/foundation-wave-evidence.md — integrated Wave-D foundation evidence tying G003/G006 lane outputs to the phase-0 capability reorg.

## 2026-07-09 build-tooling governed path registration

ADR-0562 maps build/CI engines and workspace-manifest tooling to the `build` meta home. The
Cargo.lock move/canonicalization helper is that build-tooling substrate, not a product capability.
The following governed paths are intentionally owned by `cloud-ci-platform` and registered as
build/CI tooling for the reorg codemod and metadata xtask:

- libs/oya-cargo-lock-transform-kernel/BUCK
- libs/oya-cargo-lock-transform-kernel/Cargo.toml
- libs/oya-cargo-lock-transform-kernel/OWNERS
- libs/oya-cargo-lock-transform-kernel/src/lib.rs
