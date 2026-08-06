---
id: ADR-0547
title: "Kernel-purity dependency gate"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-11
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0549]
depends_on: [ADR-0083, ADR-0131, ADR-0132, ADR-0363, ADR-0510, ADR-0515, ADR-0538, ADR-0540, ADR-0544]
amends: []
related: [ADR-0017, ADR-0131, ADR-0132, ADR-0512, ADR-0539, ADR-0540, ADR-0544]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0547: Kernel-purity dependency gate

## Status

**Proposed - 2026-06-11 (authored for founder sign-off; door: one-way).**

## Context

The owned-stack doctrine (founder directive 2026-06-09; `CLAUDE.md` `owned_stack_policy`) is a
clean-architecture cutover: the whole stack is reimplemented Rust-native (kuberos kernel →
cloud-os → cloud-k8s → cloud services → oyatie products), and upstream substrates (kube /
k8s-openapi, rustls, sqlx / tokio-postgres, the AWS SDKs, etcd, Zitadel) are **ADR-0510 transient
adapters** that get discarded at cutover. The design rule is the ports/adapters seam: a crate
named `*-kernel` or `*-core` is the **cutover-stable** seam — its interfaces must NOT change when
the transient infrastructure is replaced — while the transient wiring lives in `*-adapter` crates
that are deliberately throwaway. The litmus every boundary must answer is *"would this interface
change at cutover?"*; if yes, the boundary is mis-drawn.

**FRIC-1781129000** (a real finding this session, cloud-kms operator review M1) is the motivating
defect: the operator's kube-agnostic orchestration traits (`ObservedStateProvider`,
`KmsOperatorActuator`, `run_reconcile_cycle`, `ExponentialBackoff`, `ReconcileCycleReport`)
originally lived inside the transient `operator-k8s-adapter` crate. Pure, cutover-stable
orchestration code was colocated with the throwaway kube wiring, so at cloud-k8s cutover that pure
code would have been discarded with the adapter. **Nothing mechanical caught it** — only reviewer
eyes did. The fast-follow extraction (a pure `oya-cloud-kms-operator-kernel` holding the
kube-agnostic traits + run loop; adapter and app depend on core) already landed in PR #686. What
is still missing is the **systemic, universal** half: a mechanism that makes the seam-mis-draw
class impossible to ship for *every* kernel in the paved-road product, not just the one a reviewer
happened to catch.

This is the founder's "automation maximalism + staleness" and "enforcement layering" doctrine
applied to architecture: a property a reviewer must check by eye is a property a gate should make
structurally impossible. It is also the "pipeline = universal hermetic product" doctrine — the
kernel-purity check is a paved-road capability every operator in the product needs, so it ships as
a portable, policy-driven gate, not an oyatie-specific script.

### Precedent (hyperscaler lens)

Static, declarative dependency-direction enforcement is a proven production pattern:

- **Bazel `deps` + visibility allowlists** constrain which targets may depend on which — direction
  enforced at the build graph where the edge is declared.
- **dependency-cruiser / import-linter** "forbidden" and "layers" contracts fail a build when a
  module imports across a banned boundary; they are self-contained config+source readers, not
  pipeline-stage plugins.
- **ArchUnit** asserts layer-access rules ("domain may not access infrastructure") as ordinary
  tests over the static dependency graph.

The kernel-purity gate is the Rust-native reimplementation of this pattern for the owned-stack
cutover seam: *kernel/core crates may not depend on transient infrastructure*. We adopt the
proven shape (declarative deny rules over the static dep graph, self-cleaning exceptions) and add
the one property the cutover demands that generic layer-linters lack: the deny set is keyed on the
**transient-vs-owned** distinction (ADR-0510), so the rule is exactly "would this dep survive
cutover?".

## Decision

Ship a **self-contained cloud-ci gate**, `cloud-ci-kernel-purity`
(`ci/facade/core-dependency-isolation`), that asserts: **no crate matching the
kernel-name globs (`*-kernel`, `*-core`) — nor any workspace-internal crate reachable through its
path-dependency closure — directly depends on a denylisted transient-tech crate**, unless an
explicit, reasoned per-(crate, dep) exception is declared in policy DATA.

Five decisions (consensus planning pass, ralplan):

### D1 — Self-contained gate, not a producer-face

The gate is a self-contained crate with its own `collect_kernel_deps(root, policy)` I/O kernel +
pure `evaluate_keyed(policy, observed)`, mirroring the **registration footprint** of the latest
gate, friction-accounting (ADR-0544): own crate, own policy JSON, one appended matrix line in
`.github/workflows/oya-ci-required.yml`. It makes **zero** edits to `libs/oya-ci-config` — no
`GateFace` variant, no producer change. Rationale: (i) R0 portability — the accounting-registry
producer is oyatie-specific, so binding a face would kill "runs on any repo"; (ii) `GateFace` is a
shared-kernel HARD-collision surface; (iii) precedent — import-linter/dependency-cruiser/ArchUnit
are self-contained. We copy friction-accounting's registration footprint only, not its
legacy-debt baseline machinery (none is needed; see D5).

### D2 — Internal-closure scan, not direct-deps-only

Direct-deps-only has a verified escape: `oya-foo-kernel` → local
`oya-data-sql-adapter-sqlx` (a `path` dep) → `sqlx` would smuggle transient infra into a kernel
while the kernel stays green, because adapter crates by design never join the gated set. So v1
walks each glob-matched kernel's **workspace-internal dependency closure** (Cargo `path` deps +
local BUCK deps) and applies the external denylist to **every reached node's direct external
deps**, keying findings `kernel → offending-crate :: dep`. This is still pure text parsing (no
`cargo metadata`), hence hermetic. Full resolver-transitive closure (features, registry deps)
stays v2 under reserved code `KP-TRANSIENT-DEP-TRANSITIVE`. Scope: `[dependencies]`,
`[build-dependencies]`, `[target.*.dependencies]`; **dev-dependencies are excluded** (test-only,
never shipped in the kernel); the BUCK side parses the `rust_library` target's `deps` only, never
`rust_test` deps.

### D3 — Read both Cargo.toml and BUCK

Per-source codes `KP-TRANSIENT-DEP-CARGO` / `KP-TRANSIENT-DEP-BUCK`. This is a buck2-first repo:
the BUCK graph is what actually ships, so a transient dep added in only one source must still be
impossible. Cross-source *drift* (a dep in Cargo but not BUCK) remains owned by target-parity
(ADR-0540) and is not re-implemented here.

### D4 — Exceptions: explicit per-(crate, dep), self-cleaning

An exception is an exact `{crate, dep, reason, adr}` tuple in policy DATA; `reason` + `adr` are
mandatory (a reviewed, cited carve-out, per the "carve-outs are DATA" doctrine). An exception that
matches no live finding emits `KP-STALE-EXCEPTION` (ESLint `reportUnusedDisableDirectives`
precedent), so the exception set is shrink-only by construction. Ships **empty**. Legitimate
kernel primitives (`aws-lc-rs`, `libc`, `zeroize`, `tokio`) are simply **not on the denylist** —
they are cutover-stable, so no exception is needed and no false positive arises by construction.

### D5 — Born-blocking, no baseline file

All 156 of today's *scanned* `*-kernel`/`*-core` crates are pure (verified: the path-dep closure of
every one is free of denylisted external deps), so the expected finding set is empty and the gate is
**RED-blocking on day one** — any NEW kernel-with-transient-dep (directly or via closure) fails
closed immediately. No shrink-only legacy baseline is required. A liveness guard
(`min_expected_kernel_crates`, set to 150 — just below the 156 census) emits `KP-EMPTY-SCAN` if the
scan finds fewer kernels than the floor, catching a silently broken glob/CWD/collect that would
otherwise be a false-green. The policy itself is **strictly parsed**: a missing `dep` key, missing
`match` key, or unknown `match` value in a deny rule emits `KP-POLICY-MALFORMED` and short-circuits
evaluation — a typo'd rule that silently dropped would be a false-green exploit.

**Coverage scope (honest census).** The 4 `oya-cloud-kernel-{frame,hal,ksync,user-layout}-kernel`
crates live in the `cloud/cloud-kernel` nested `no_std` workspace, which the root manifest lists
under `[workspace].exclude`; they are not root-workspace members and are therefore structurally
outside this gate's scan, permanently. Transient infra deps there are implausible (bare-metal
`no_std`), but the exclusion is recorded here and in the policy `_comment` rather than left implicit.
The gate runs on the root workspace; a future change to scan nested workspaces is a reserved
extension, not a v1 requirement.

### D6 — Automation-default layering (founder directive 2026-06-11)

Per the founder directive "the gate should prioritize automation where possible; automation should
be the default; enforcement is the extra layer" (face-settle precedent), the deliverable is a
detector + automation-where-derivable + blocking backstop:

- **Derivable + auto-fixed (`--fix`).** A denylisted dep declared in a crate's own manifest that
  is **not referenced anywhere in that crate's `src/**/*.rs` or `build.rs`**, is not a build-dep, is
  not renamed, is not `optional = true`, and is not feature-referenced — is a dead transient dep.
  Removing the manifest line moves no code and is purely mechanical, so the gate binary applies it
  under `--fix` **for Cargo.toml only** (see BUCK descope note below).
  Source-usage is detected by a conservative token scan that maps the dep name to its Rust ident
  (`kube-runtime` → `kube_runtime`) and over-approximates "used" (a mention in a comment counts),
  which is the SAFE direction.
  **Cargo `--fix` safety bounds** — five bounds keep the Cargo remover from corrupting a manifest
  or removing a live dep, each backed by a RED/GREEN regression fixture:
  (i) **build-dependencies are never auto-fixed** (their liveness is hard to attribute per-dep;
  `build.rs` is scanned only to MARK them live, never to remove them);
  (ii) the Cargo remover is **table-aware** — it deletes a dep line only inside a real
  `dependencies`/`build-dependencies`/`target.*` table, never `[dev-dependencies]` (live test dep)
  or `[features]` (would dangle `dep:<x>`);
  (iii) **renamed deps are never auto-fixed** — `foo = { package = "kube" }` means src uses `foo::`
  not `kube::`; liveness probes both idents, but line-removal by real name alone would leave orphaned
  `foo.workspace = true` lines, so all renamed deps are conservatively demoted to design-action;
  (iv) **feature-referenced deps are never auto-fixed** — `collect_features_referenced_deps` scans
  ALL `[features]` value strings against ALL dep tables using the full Cargo feature-entry token
  grammar (`dep:X`, `X`, `X/feat`, `X?/feat`), catching sub-feature paths (H1), optional-dep
  activation (H2), bare names (H3), and target-cfg tables (H4); any dep token matched is demoted to
  design-action so removing the dep line can never leave a dangling feature entry;
  (v) **optional deps are never auto-fixed** (MED-X1, FRIC-1781210000) — `optional = true` exports
  an **implicit** cargo feature named after the dep even when the owning manifest's `[features]`
  never mentions it, and a SIBLING workspace member can request that implicit feature
  (`features = ["kube"]`) on its path dependency; bound (iv) cannot see that request (it scans only
  the owning manifest) and the layer-2 revalidation below cannot either (no cross-member feature
  resolution under `--no-deps`), so every `optional = true` dep is demoted to design-action with
  remediation text explaining the implicit-feature export.
  **CRITICAL-A layer 2 — `cargo metadata` rollback:** after all Cargo.toml edits are written,
  `cargo metadata --no-deps --format-version 1` is run as a semantic revalidation gate (the sole
  sanctioned cargo invocation per the teammate preamble). If it fails, ALL pre-images are restored
  (first pre-image per path, so a manifest edited twice rolls back to its ORIGINAL content) and the
  findings are reclassified as design-actions in the returned error, which carries the cargo error
  text. Layer 2 validates exactly what `--no-deps` resolves: each workspace member's own manifest
  still parses and is internally consistent (e.g. a dangling `dep:<x>` feature entry in the edited
  manifest is caught). It does **not** perform cross-member feature resolution, so a sibling
  member's `features = [...]` request against another member's dep is invisible to it — which is
  why bound (v) refuses ALL optional deps up front instead of relying on layer 2. If the `cargo`
  binary itself cannot be spawned (a hermetic environment without cargo on PATH), layer 2 degrades
  explicitly: layer-1 syntactic bounds have already passed and the blocking buck2 `rust_test` gate
  remains the enforcement backstop. The validator is injected (`apply_fixes_with_validator`) so the
  rollback path is pinned by a deterministic fixture rather than the test host's cargo availability.
- **BUCK `--fix` descoped to refusal-only (round-3 revision, FRIC-1781200001).**
  *Amended by ADR-0549 (2026-06-11): this descope is CLOSED — BUCK `--fix` is re-enabled on the
  `oya-buck-syntax-kernel` sound parser + write-through fixer harness (span-exact element
  removal, reparse + no-collateral validation, first-pre-image rollback); unsound shapes still
  refuse with the file byte-identical. The paragraph below is the historical pre-kernel
  rationale.* The Starlark
  block parser (paren-depth + comment/string stripping) is not yet sound enough to guarantee safe
  rewrites against all BUCK syntaxes (multi-rule files, macro-generated blocks, unusual indentation).
  Rather than ship a fixer that could corrupt BUCK files, BUCK findings that would otherwise be
  auto-fixable are reported as **design-actions** with an actionable remediation note pointing to
  the `oya-buck-syntax-kernel` fixer harness. This is queued as `FRIC-1781200001` (status:
  `queued-shared-kernel`). The `remove_buck_dep_line` function is retained as a refusal stub
  (`Ok(false)` without writing) for the detector/test path.
- **Not safely derivable (printed, never auto-applied).** A denylisted dep that IS used in the
  kernel's source, is renamed, is a build-dep, is `optional = true`, or is feature-referenced
  requires a design act (moving code / rewriting feature entries / auditing sibling feature
  requests). The gate distinguishes the reason in `next_action` so the contributor sees the right
  remediation step, never a bare FAIL.
- **Blocking gate = backstop.** The buck2 `rust_test` gate is the enforcement layer that catches
  whatever automation did not (or could not) fix. Default binary invocation detects and reports;
  `--fix` applies the derivable Cargo subset and re-reports the residual design actions.

Every finding carries `auto_fixable` + `next_action`. The `--fix` class is covered by RED/GREEN
fixtures (`fix_removes_dead_transient_dep_and_turns_red_to_green`,
`fix_leaves_used_transient_dep_in_place`, `dead_transient_dep_is_auto_fixable`,
`used_transient_dep_is_a_design_action_not_auto_fixable`). The sound-bound fixtures are H1–H4
(feature-syntax refusal), H5 (comment-blind paren detection), H6 (None=skip removal), the
optional-dep implicit-feature refusal
(`fix_refuses_optional_dep_whose_implicit_feature_a_sibling_requests`, bound (v)), the
backslash-escape stripper fixture
(`backslash_escaped_quote_in_string_does_not_hide_following_dep`), and the two-edits-one-file
rollback fixture (`rollback_restores_original_when_same_manifest_is_edited_twice`).

### The denylist (kube/kuberos trap)

The deny set targets transient infra crate names by exact match or hyphen-bounded prefix. Two
matching traps the consensus pass caught and the policy avoids:

- `kube` **exact** + `kube-` **(hyphen) prefix** — exact alone misses `kube-runtime`/`kube-core`,
  but a bare `kube` prefix would false-positive on the **owned `kuberos`/`oya-cloud-kernel-*`**
  crates. The hyphen boundary denies the `kube-*` family without touching owned kernel names.
- `aws-sdk-` **(hyphen) prefix** denies the AWS SDK family while never matching the primitive
  `aws-lc-rs`.

Both non-matches carry explicit unit assertions so the discipline cannot silently regress.

## Consequences

- The clean-arch cutover litmus is enforced **mechanically** for kernel/core crates: a kernel that
  acquires a transient dep (directly or by absorbing a transient-carrying local crate into its
  closure) cannot reach merge. The FRIC-1781129000 seam-mis-draw class is now structurally
  impossible, closing the enforcement half of that friction.
- R0 pack-shaped: kernel globs, the denylist, exceptions, and the liveness floor are all DATA in
  `kernel-purity-policy.json`; the Rust kernel hardcodes no oyatie crate name, so any repo adopts
  the gate by repointing the policy.
- Scope deliberately bounded to v1: workspace-internal-closure over declared deps, not the full
  feature-resolved transitive closure (reserved `KP-TRANSIENT-DEP-TRANSITIVE`). This is the
  smallest change that closes the verified escape while staying hermetic and text-only.
- The operator-core extraction (FRIC-1781129000 half (a)) is already landed (#686) and tracked
  separately; this ADR delivers half (b).

## Alternatives considered

- **Producer-face integration** (a `GateFace::KernelPurity` + a producer collector). Rejected:
  binds the gate to the oyatie-specific accounting-registry producer (kills R0) and edits the
  shared `oya-ci-config` HARD-collision kernel; the latest gate (ADR-0544) already established the
  self-contained pattern as the precedent for policy-driven meta-gates.
- **Direct-deps-only.** Rejected (D2): a verified escape through a local transient-carrying
  adapter path-dep leaves a kernel green while shipping transient infra.
- **Cargo-only or BUCK-only dep source.** Rejected (D3): a buck2-first repo must enforce both;
  the shipped graph is BUCK, the declarative source of truth is Cargo.
