---
id: ADR-0550
title: "Repository layout doctrine — hyperscaler monorepo + clean-architecture seams"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-11
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0562]
depends_on: [ADR-0131, ADR-0132, ADR-0510, ADR-0512, ADR-0543, ADR-0547]
amends: [ADR-0512]
related: [ADR-0017, ADR-0056, ADR-0105, ADR-0357, ADR-0362, ADR-0509, ADR-0515, ADR-0538, ADR-0540, ADR-0544, ADR-0545, ADR-0546, ADR-0548, ADR-0549]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0550: Repository layout doctrine — hyperscaler monorepo + clean-architecture seams

## Status

**Superseded — 2026-07-10 by ADR-0562 (founder ratification 2026-07-10).** Authored (Proposed)
2026-06-11 for founder sign-off. The layout doctrine below is superseded **in full** by ADR-0562's
capability-first repo organization: the `{oya,cloud}/<service>/` colocation root (D1) and the
`libs/` rule-of-two charter (D2) are replaced by one top-level dir per registered capability
(`core/ports/adapters/facade` faces) plus `base/` for admission-gated cross-capability primitives.
The clean-architecture seams this ADR established — the kernel/adapter/app crate boundaries and the
"structure encodes ports/adapters seams for frictionless cutover" litmus (D3) — are **preserved and
carried forward** by ADR-0562 §4 (the non-negotiable face rule) and §6 (the membership lint); they
were never in tension with capability-first. This ADR is retained as the durable record of the
doctrine that capability-first absorbed; consult ADR-0562 for the live layout authority.

## Context

The repository's layout rules exist, but no single normative source states them. Today they are
scattered across four artifact classes:

- **ADR-0131** (as amended): per-service colocation — `{oya,cloud}/<service>/` owns PRD,
  contracts, specs, runbooks, SLOs, IaC, threat model, decisions, catalog, evidence, and code.
- **ADR-0132** (with ADR-0362's grandfather-clause retirement): the no-grouping forward-policy —
  every service is flat and single-concern; bundles/verticals are forbidden.
- **ADR-0512**: the canonical monorepo pattern — one root Cargo workspace, crate = bounded
  context, service dirs as pure containers, Buck2 graph as the parallelism/containment substrate.
- **CLAUDE.md prose + agent memory**: the founder structure doctrine (2026-06-10) — *"structure
  encodes ports/adapters seams, tech-agnostic, frictionless owned-stack cutover; flat is not the
  principle"* — and the ports-designed-for-owned-stack litmus *"would this trait change at
  cutover?"* (2026-06-09), neither of which is stated normatively in any ADR.

Meanwhile the tree itself moved past the written rules. The founder structure directive of
2026-06-10 (recorded in ADR-0543) required the crate layout to **encode the clean-architecture
seam**: the cloud-kms operator shipped as three crates — `oya-cloud-kms-operator-kernel` (pure,
zero kube deps), `oya-cloud-kms-operator-k8s-adapter` (ADR-0510 transient absorber), and
`oya-cloud-kms-operator-app` (composition root) — in PR #686. ADR-0547 then made the kernel half
of that seam *mechanically enforced*: the `cloud-ci-kernel-purity` gate asserts no `*-kernel` /
`*-core` crate (nor its workspace-internal path-dependency closure) depends on denylisted
transient tech, born-blocking in the `oya-ci-required` merge authority. ADR-0548 D3 names the
kernel/adapter/app crate pattern as the product's proven service shape, and ADR-0549 set the
`libs/` placement precedent (`libs/oya-buck-syntax-kernel`: a pure, std-only shared kernel born
in `libs/` with two concrete consumers migrated in the same change).

But ADR-0512's operative text still says the clean-architecture layers are **modules, not
crates** ("`domain` / `ports` / `adapters` / `api` as `mod`s"; "no layer-per-crate"). Read
literally, that forbids the exact crate shape PR #686 shipped, ADR-0547 enforces, and ADR-0548
productizes. The doctrine the repo actually runs on is governed only by accumulated precedent,
which is the staleness/decay class the founder's automation-maximalism doctrine names. This ADR
is the single normative source (ledgered as FRIC-1781260000) and the reconciliation.

### Precedent (hyperscaler + literature lens)

- **Google** (Winters/Manshreck/Wright, *Software Engineering at Google*): one monorepo,
  one-version policy, per-area/service directories, and **BUILD visibility as the API-surface
  control** — reuse is granted by allowlist, not by reachability. ADR-0512 already adopts the
  topology; this ADR adopts the visibility idiom as the seam-enforcement destination.
- **Meta / Buck2**: fine-grained targets with `visibility` attributes; the build graph is the
  dependency-rule enforcement point. Buck2 is rustc-direct (ADR-0512), so crate boundaries are
  target boundaries are enforcement boundaries.
- **Hexagonal / ports-and-adapters** (Cockburn) and the **Clean Architecture dependency rule**
  (Martin): dependencies point inward; the domain never knows the infrastructure. The owned-stack
  twist (ADR-0510): our "infrastructure" is *transient by declaration* and gets discarded at
  cutover, so the seam must also be a **deletion boundary**, not only a compile-time direction.
- **ArchUnit / import-linter / dependency-cruiser** (via ADR-0547): layer-access rules enforced
  over the static dependency graph — proof that direction rules belong to gates, not reviewers.

## Decision

### D1 — Canonical service shape: `{oya,cloud}/<service>/` + kernel/adapter/app seams

The service root is `{oya,cloud}/<service>/` per ADR-0131 as amended by ADR-0512: `oya/` for
product/domain services, `cloud/` for platform/tenant-substrate services, the service directory
a **pure container** (never a crate at its root) colocating the ADR-0131 artifact set, with code
under `<service>/crates/<crate>/`. The legacy `microservices/` root is gone from the tree
(verified at this ADR's base); flat top-level `crates/` remains forbidden.

Within a service, three **named seams** are the canonical crate roles:

| Seam | Naming | Role | Cutover fate |
|---|---|---|---|
| **kernel** | `*-kernel`, `*-core` (gate-scanned globs); `-domain`/`-usecase` are kernel-class by posture | Pure domain + port traits. Trait shapes model the W5 owned-stack destination. Zero transient-tech dependencies. | **Survives unchanged.** |
| **adapter** | `*-adapter*` (tech name allowed only here, e.g. `-adapter-sqlx`, `-k8s-adapter`) | Absorbs ADR-0510 transient tech (kube, sqlx, rustls, aws-sdk-*, etcd, Zitadel, …). Deliberately throwaway. | **Deleted/replaced as a unit.** |
| **app** | `*-app` | Composition root: wiring, config, binary. The only place adapters meet kernels. | **Rewired, not redesigned.** |

**Dependency direction rules** (the seam matrix; "X → Y" = X may depend on Y):

- kernel → kernel only (including `libs/` shared kernels). A kernel never depends on an adapter,
  an app, or transient tech — enforced for the `*-kernel`/`*-core` globs by ADR-0547's
  closure-walking gate, so a kernel cannot acquire a transient dep even through a local path-dep.
- adapter → kernel (+ its own transient tech + `libs/`). An adapter never depends on an app.
- app → kernel + adapter + `libs/`. **app → app stays forbidden** (a composition root composes
  inward, never another deployable).
- `{oya,cloud}/<a>/crates/**` never imports `{oya,cloud}/<b>/crates/**` (ADR-0131): cross-service
  composition flows through contracts, Workflow events, and Ontology reads/writes; shared code
  flows through `libs/`.

**Seam-as-crate rule (this is the ADR-0512 amendment).** ADR-0512 clause 3/4 ("crate = bounded
context; clean-arch layers as modules, no layer-per-crate") is narrowed, not discarded:

- Crate = bounded context **remains the sizing default**, and module-level layering with
  `pub(crate)` remains the rule *inside* any single crate, proportional to complexity. The
  five-layer crate ceremony ADR-0512 rejected (ADR-0357's shape) stays rejected.
- BUT two boundaries are **crate boundaries by mandate, never modules**: (1) the transient-tech
  seam (kernel vs adapter) and (2) the composition root (app). Three reasons, each structural:
  (a) **mechanical enforceability** — ADR-0547 reads the crate dependency graph (Cargo.toml +
  BUCK); a module-level seam is invisible to it, which is exactly how FRIC-1781129000 shipped
  pure orchestration code inside a throwaway adapter; (b) **Buck2 granularity** — visibility and
  caching operate on targets = crates; (c) **discardability** — cutover deletes adapters as
  units, and a deletion boundary must be a physical unit.
- An adapter crate exists **only when transient tech exists** in that bounded context; a
  pure-computation bounded context stays one crate. The cloud-kms operator (one bounded context,
  three crates, PR #686 / ADR-0543) is the canonical worked example.

The repo-facts of this shape (root names, container name, seam globs, denylist, direction
matrix) are **pack data, not engine code**, per ADR-0548 R0: the oyatie policy pack encodes
`{oya,cloud}/` + `crates/` + the ADR-0547 policy; another repo adopts the doctrine by writing
its own pack.

### D2 — `libs/` charter

`libs/<crate>/` is the shared root: flat, one crate per entry, directory basename ==
`[package].name` (ADR-0512 R7).

**Belongs in `libs/`:** single-concern code consumed by ≥ 2 services. The seam taxonomy of D1
applies unchanged: shared **kernels** (pure; e.g. `oya-buck-syntax-kernel` (ADR-0549, std-only),
`oya-json-kernel`, `oya-shared-pdp-kernel`), shared **adapters** (one transient tech absorbed
for many consumers; e.g. `oya-data-sql-adapter-sqlx`), and
shared **apps** only for genuinely shared runtime binaries (e.g. the transactional-outbox worker
family). A shared adapter whose concern is owned by a registered capability homes to that
capability instead, per ADR-0562 (e.g. `iam-pdp-cedar` at `iam/adapters/pdp-cedar`).

**Does NOT belong in `libs/`:** service-specific code (exactly one consuming service ⇒ it lives
in that service's `crates/`); product features; bundle/vertical/grouping names — ADR-0132
applies to libraries too: a lib is one concern, never a suite.

**Placement rule (rule of two):** code is born in its owning service and is extracted to
`libs/` when the second consumer appears. Born-in-`libs/` is justified only when ≥ 2 concrete
consumers exist at landing time (the ADR-0549 precedent: both gate consumers migrated in the
same change) or when an ADR designates the crate as product substrate (ADR-0548 D3's
`libs/oya-gate-kernel` extraction).

**Direction + visibility:** `libs/` never depends on `{oya,cloud}/**` — the shared root is
strictly inward of every service. Naming carries the `oya-` prefix (ADR-0017, cargo-prefix gate)
and the BNF layer suffix (ADR-0056/ADR-0105, bnf-layer-suffix gate). Destination enforcement is
Buck2 `visibility`: `libs/` targets repo-visible, service crates service-visible by default
(Google BUILD-visibility idiom); until that lands, the direction rule is reviewed doctrine plus
the D5 gate destination.

### D3 — The cutover litmus is a layout review question the structure answers

Every layout review asks the ports-designed-for-owned-stack question: **"would this trait change
at cutover?"** The doctrine's job is to make the answer "no" *by construction*, not by reviewer
vigilance: port traits live in kernels, and transient types cannot appear in kernel signatures
because the crates defining them are unreachable from kernel crates (ADR-0547's denylist over
the kernel's workspace-internal closure). This is the founder enforcement-layering doctrine
applied to structure — the rule holds with hooks disabled because it is a property of the
dependency graph, not of process.

Corollaries:

- Transient tech may name an **adapter crate** and nothing else — never a service, a top-level
  tree, a kernel, or a trait. Structure stays tech-agnostic (founder doctrine 2026-06-10).
- Cutover cost is bounded by construction: replace adapter crates, rewire apps, kernels
  untouched. ADR-0543 records this as a shipped property of the cloud-kms operator ("swapping
  kube-rs for the owned cloud-k8s substrate replaces the adapter crate only").
- The layout review checklist is three questions: (1) would this trait change at cutover?
  (2) would a hyperscaler shape it this way — cite the precedent; (3) does any transient tech
  name anything outside an adapter crate?

### D4 — Flat topology × internal seams, stated precisely

The two rules govern different axes and do not conflict:

- **Flat is the SERVICE topology** (between services): one concern = one service = one folder
  directly under `oya/` or `cloud/` (ADR-0132). No bundle, vertical, or grouping directories; a
  service is never a parent of another service; `libs/` is likewise flat (one crate per entry).
- **Seams are the INTERNAL shape** (within a service): the service folder is a pure container;
  crates live under `crates/`; bounded contexts split kernel/adapter/app per D1 where transient
  tech exists.

*"Flat is not the principle"* (founder, 2026-06-10): flatness is the **consequence** of
single-concern service boundaries; the **principle** is that structure encodes the
clean-architecture seams so the owned-stack cutover is frictionless. A flat catalog with seamful
internals is therefore not a tension but the design: ADR-0132 governs the catalog, this ADR's D1
governs the crate graph.

### D5 — Enforcement: what is mechanical today, what is queued

Honest audit at this ADR's base (d5d8be5d4), against the `cli_surface_policy` rule that merge
authority lives only in the cloud-ci gates behind `oya-ci-required` (ADR-0515):

**In merge authority today (gate matrix in `.github/workflows/oya-ci-required.yml` + the binding
`buck2 test //cloud/cloud-ci/...` lane):**

| Doctrine clause | Gate | Authority basis |
|---|---|---|
| Kernel seam purity (D1 kernel row, closure-walked) | `cloud-ci-kernel-purity` | ADR-0547, born-blocking |
| `oya-` crate-name prefix | `cloud-ci-cargo-prefix` | ADR-0017 |
| BNF layer-suffix naming | `cloud-ci-bnf-layer-suffix` | ADR-0056/0105 lineage, born-blocking |
| Every first-party crate dir covered by the root workspace member globs | `cloud-ci-workspace-glob-coverage` | ADR-0538 |
| Cargo/BUCK target parity (a crate cannot exist in one graph only) | `cloud-ci-target-parity` | ADR-0540 |

**Local-bridge only — NO merge authority (retirement-marked CLI per `cli_surface_policy`):**

- `oya gate validate workspace-topology` — ADR-0512 R1–R7 (no flat `crates/`, no nested
  `[workspace]`, dir==name, canonical member prefixes).
- `oya gate validate architecture-boundaries` — the role dependency-edge matrix (including the
  app→app ban); its header still describes pre-pure-split paths (`crates/<name>`,
  `registry/catalog/`), i.e. it has drifted.
- `governance/check/no-grouping` (ADR-0132/ADR-0362) — runs only through `oya-dev-cli`.

**Not implemented anywhere:** ADR-0131's promised `per-service-layout` and
`aggregation-index-generation` gate packets (no Rust implementation exists in the tree).

**Live drift the audit surfaced** (proof the CLI-only tier does not hold — the founder
enforcement-layering audit question "does the rule hold with hooks disabled?" answers no): the
root manifest's member globs sanction two shapes the workspace-topology R6 check rejects as
written — `cloud/cloud-ci/gates/<crate>` (gate crates in a `gates/` container instead of
`crates/`) and `oya/office/oya-*` (crates at the service root with no container). Both shipped
because nothing in merge authority checks the container shape; coverage-by-glob (ADR-0538)
proves "somewhere the manifest sanctions", not "the doctrine shape".

**Destination:** one pack-shaped cloud-ci **service-layout gate family** per ADR-0548 (policy as
data: roots, container name, seam direction matrix, no-grouping patterns; detector + `--fix`
where mechanically derivable; born-blocking with a reviewed shrink-only baseline carrying
exactly the live variances above), plus extension of purity coverage beyond the
`*-kernel`/`*-core` globs to the other kernel-class suffixes (`-domain` first) as ADR-0547
policy data. Ledgered as **FRIC-1781270000** (status `queued`) per the ADR-0544/0548 D5
closed-loop discipline; the doctrine-scatter friction itself is **FRIC-1781260000**
(`fixed-in-PR` by this ADR). The two live variances are explicitly *recorded, not silently
blessed*: their disposition (migrate vs sanction-in-policy) is decided in the gate-family
review, not unilaterally here.

## Consequences

### Positive

- One citable, normative layout source. Reviews and gates cite ADR-0550 instead of
  reconstructing doctrine from CLAUDE.md prose, memory rows, and three partially superseded
  ADRs; drift between those artifacts stops being undetectable (FRIC-1781260000 closed).
- The ADR-0512 contradiction is resolved by governed amendment instead of accumulating
  precedent: module-layering stays the in-crate rule, the transient seam and composition root
  are crate boundaries, and both halves are now written down.
- The cutover litmus becomes structural: for gate-scanned kernels the answer to "would this
  trait change at cutover?" is "no" by dependency-graph construction, not reviewer vigilance.
- The doctrine composes with the pipeline product (ADR-0548): every repo-fact in D1/D2 is pack
  data, so the layout ratchet is portable by construction.

### Negative

- Doctrine is ahead of full enforcement: the topology/direction/colocation clauses remain
  review-enforced until FRIC-1781270000's gate family lands; the friction ledger makes that gap
  permanently visible (which is the point, but it is open debt).
- The two live variances (`cloud/cloud-ci/gates/`, `oya/office/` root-level crates) are now
  formally recorded debt with an undecided disposition — naming them creates obligation.
- Purity enforcement is glob-bounded: a pure crate named outside `*-kernel`/`*-core` (e.g.
  `-domain`) is doctrine-covered but not yet gate-covered; until the policy extension lands,
  naming a kernel-class crate outside the scanned globs weakens its protection.

### Operational

- buck2 remains the binding verification surface; no engine code lands with this ADR.
- New services MUST land in the D1 shape from birth; the kernel-purity gate already makes the
  kernel half born-blocking.

## Alternatives considered

**Alternative 1 — Status quo: doctrine distributed across ADR-0131/0132/0512 + CLAUDE.md + memory**
- Pros: zero authoring cost; each artifact is individually correct in its lane.
- Cons: the operative rules contradict (ADR-0512 modules-not-crates vs the shipped, gate-enforced
  kernel/adapter/app shape); founder doctrine lives in ungoverned memory; reviewers cannot cite
  a single source. This is the decay class ADR-0548 was written to stop.
- Rejected: FRIC-1781260000 is this alternative's cost, already paid.

**Alternative 2 — Module-level seams only (enforce ADR-0512 clause 4 strictly)**
- Pros: fewer crates; honors ADR-0512's text as written.
- Cons: the seam becomes mechanically invisible — ADR-0547 reads crate graphs, Buck2 visibility
  reads targets, and cutover cannot delete a module; FRIC-1781129000 is the recorded defect of a
  module-grade seam (pure code trapped in a throwaway adapter, caught only by reviewer eyes).
- Rejected: contradicts the founder structure directive (2026-06-10) and the shipped enforcement.

**Alternative 3 — Full layer-per-crate (revive ADR-0357's shape)**
- Pros: maximally fine Buck2 targets; every layer independently visible.
- Cons: crate ceremony without enforcement value — only the transient seam and composition root
  need physical boundaries; ADR-0512's rejection of five-crate bounded contexts was correct and
  the G11 train showed cost accrues per crate (BUCK wiring, parity, registration).
- Rejected: ADR-0512's rejection stands everywhere except the two mandated seams.

**Alternative 4 — Tech-named trees (a `k8s/` tree, a `sqlx/` layer, vendor-named services)**
- Pros: makes the transient surface easy to find.
- Cons: structure stops being tech-agnostic; at cutover the *tree* would change, maximizing
  friction — the inverse of the doctrine; transient tech must be deletable without renaming
  anything that survives.
- Rejected: transient tech names adapter crates only (D3).

**Alternative 5 — Service-local copies instead of a `libs/` root**
- Pros: no shared-crate coupling; every service self-contained.
- Cons: duplicated kernels drift independently — the exact defect class ADR-0549 closed for BUCK
  parsers ("the duplication itself is the defect"); one-version policy (ADR-0512) exists to
  prevent this.
- Rejected: shared single-concern code has exactly one home (`libs/`), guarded by the rule of two.

## Verification

This is a doctrine ADR; no engine code lands with it.

- `buck2 test //cloud/cloud-ci/...` green, including the friction-accounting gate (ADR-0544)
  over the updated ledger: FRIC-1781260000 dispositioned `fixed-in-PR` citing this ADR;
  FRIC-1781270000 appended as `queued` with the gate-family destination.
- The decision-crosswalk face registers ADR-0550 after the faces settle
  (`oya-cloud-ci-face-settle --settle --commit` as the final commit, `--verify` before push,
  per ADR-0539 / FRIC-1781250000).
- ADR-0512 carries the `amended_by: [ADR-0550]` backlink and an inline amendment note (operative
  text of clauses 3/4 narrowed per D1); ADR-0131/0132 are related, not amended — their operative
  text is unchanged.

## References

- ADR-0131 (per-service colocation; as amended by ADR-0512), ADR-0132 + ADR-0362 (no-grouping
  flat catalog), ADR-0512 (canonical monorepo pattern; amended here), ADR-0510 (transitional
  substrate adapters / owned-stack cutover posture), ADR-0543 (cloud-kms operator
  kernel/adapter/app, PR #686), ADR-0547 (kernel-purity gate; FRIC-1781129000), ADR-0548
  (pipeline-as-product: R0 pack-shape, D3 service shape, D5 closed loop), ADR-0549
  (`libs/oya-buck-syntax-kernel` placement precedent), ADR-0515 (merge authority), ADR-0538
  (workspace glob coverage), ADR-0540 (target parity), ADR-0017 (crate prefix), ADR-0056/0105
  (BNF naming + layer enum), ADR-0544 (friction accounting).
- Founder directives: structure doctrine + "flat is not the principle" (2026-06-10);
  ports-designed-for-owned-stack litmus (2026-06-09); owned-stack policy + CLI retirement +
  enforcement layering (2026-06-09, CLAUDE.md `owned_stack_policy` / `cli_surface_policy`).
- FRIC-1781260000 (layout doctrine scattered — closed by this ADR); FRIC-1781270000
  (layout-enforcement gap — queued); FRIC-1781129000 (the seam-mis-draw motivating defect).
- Precedents: *Software Engineering at Google* (monorepo, one-version, BUILD visibility);
  Meta/Buck2 target visibility; Cockburn ports-and-adapters; Martin's Clean Architecture
  dependency rule; ArchUnit / import-linter / dependency-cruiser (via ADR-0547); AWS / Google /
  Microsoft / Oracle / Stripe per-service folder practice (via ADR-0131 §Context).
