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
