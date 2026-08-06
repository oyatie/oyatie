---
id: ADR-0548
title: "Pipeline as product: neutral ratchet engine + policy packs on the paved road"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-11
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0705]
depends_on: [ADR-0510, ADR-0515, ADR-0516, ADR-0543, ADR-0544, ADR-0545, ADR-0546, ADR-0547]
amends: []
related: [ADR-0083, ADR-0111, ADR-0131, ADR-0132, ADR-0139, ADR-0363, ADR-0512, ADR-0539, ADR-0540, ADR-0541, ADR-0549]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0548: Pipeline as product — neutral ratchet engine + policy packs on the paved road

## Status

**Proposed - 2026-06-11 (authored for founder sign-off; door: one-way).**

## Context

The founder decisions that turn oyatie's enforcement pipeline into a product were made across
2026-06-09..11 but lived only in ungoverned places: a gitignored session one-pager
(`.omc/ultragoal/PRODUCT-pipeline-paved-road.md`, refined 2026-06-10 via idea-refine with founder
decisions in-session), the dispatch ledger, and agent memory. That is itself the decay class the
founder's automation-maximalism doctrine names (staleness = process failure); a binding product
direction must be a governed artifact, not session residue. This ADR converts those decisions into
doctrine and is the durable record of the founder directives, verbatim:

- **(a) — R0, 2026-06-10:** *"if any part of our pipeline is not reusable, it is not meeting our
  requirements"*.
- **(b) — the paved-road promise, 2026-06-10:** *"cloud native, hyperscaler pattern, hermetic,
  universal — so anyone can build cloud-native, hermetic applications if it goes through our
  pipeline."*
- **(c) — automation-default, 2026-06-11 (first recorded in ADR-0545/0546/0547):** *"gate should
  prioritize automation where possible; automation should be the default; enforcement is the extra
  layer."*

These sit on the standing doctrine base: pipeline = universal hermetic product (founder /goal
doctrine 2026-06-09: frictions → ledger → gates, anti-patterns unshippable, the ratchet must run on
any repo), proven patterns reimplemented Rust-native with precedent cited per decision, and the
ADR-0516 Agentic Delivery Fabric apex vision, of which the cloud-ci pipeline (ADR-0515) is the
admission component.

### Train evidence (G11 gate train, 2026-06-11)

The decisions below are not speculative: the G11 train shipped them and recorded what it cost. Per
the dispatch ledger, the train landed **5 merges this session** (#689 canonical-json, #692 its
hex-escape fast-follow, #690 embedded-asset-hermeticity, #691 kernel-purity, #693 the optional-dep
fast-follow), and the two fixer-bearing lanes took **four hostile-review REJECT verdicts** before
approval (PR #690 rounds 1–2, PR #691 rounds 1–2; each lane reached APPROVE only in round 3). The
REJECT history is the empirical core of D7 below.

## Decision

### D1 — The pipeline itself is a product: a neutral ratchet engine + policy packs

The product is a NEUTRAL ratchet engine plus policy packs layered on it. The engine hardcodes
nothing repo-specific; **all repo facts are policy-as-data**. The kernel contract is already latent
in the gate fleet: `collect(root, policy) -> observed rows` + `evaluate(policy, observed) ->
findings -> verdict`, with ratchet baseline semantics — new violations block, baselines only
shrink, exceptions are decaying leases, never permanent grants.

Precedent (proven patterns, Rust reimplementation): **OPA/Gatekeeper** — a neutral policy engine
where the `ConstraintTemplate`/`Constraint` CRD pair IS the engine/pack split; **Google Tricorder**
— program analysis as a platform of pluggable analyzers with criticism-driven feedback, where
analyzers attach fixes to findings; **Betterer-class quality ratchets** — baseline-and-shrink test
semantics. We adopt the architecture, not the dependencies.

### D2 — R0, the binding requirement

**Any non-reusable pipeline component is a requirements violation** (founder directive (a),
verbatim above). Engine code ships as crates/binaries taking policy-as-data; the ONLY repo-specific
residue permitted is policy packs + baselines + ledgers. An R0 violation is not an exception class
— it is a friction row, ratcheted down like any other debt (D5).

### D3 — First product surface: a K8s operator + CRDs

The first product surface is a Kubernetes operator with CRDs — `GatePolicy`, `Baseline`,
`Exception`, `GateRun` — built via the kernel/adapter/app crate pattern proven by the cloud-kms
operator (ADR-0543; the pure-kernel extraction landed in PR #686: serde-only operator kernel,
kube/k8s-openapi confined to the k8s adapter and the app composition root, app depends on the
kernel). Cloud-native, K8s-native,
owned-stack-first per standing doctrine: port traits model the W5 owned-stack destination; adapters
absorb transient infra (ADR-0510) and are discarded at cutover.

Surface sequencing (the engine extraction is identical work for every surface, so it de-risks the
bootstrap): (1) extract the kernel + policy-pack format to `libs/`, (2) wrap it in the operator
CRDs, (3) the GitHub required-check (today's ADR-0515 merge authority) becomes ONE ADAPTER of the
same kernel. Precedent: the Gatekeeper engine/CRD split (D1); the kernel/adapter/app seam already
shipped in-tree (ADR-0543).

### D4 — Universality is proven, not asserted: public-repo conformance

Universality (R0 + promise (b)) is demonstrated by a conformance harness that points **read-only**
collectors at public GitHub repos and snapshots verdicts — proving the engine runs on repos it has
never seen, without any distribution commitment. Initial scope: 3 pilot repos (one-pager
assumption-validation). This is a conformance strategy, not open-sourcing (see Not Doing).

### D5 — The closed loop: the FRIC-total-accounting meta-gate

The loop is closed by the friction-accounting meta-gate (ADR-0544, gate id
`cloud-ci-friction-accounting`, merged as PR #687): **every friction-ledger row must terminate in a
gate, an automation, or an explicit evidenced accepted-risk entry**. New undisposed frictions are
born-blocking; pre-existing schema/closure debt is frozen in a reviewed shrink-only baseline.
Precedent: the Google SRE postmortem action-item model (owner + verifiable closure, deferred items
explicitly accepted). The pipeline's own R0 gap (see Current State) is accounted through exactly
this loop.

### D6 — Automation-default (normative)

Founder directive (c), verbatim above, is normative for every gate in the product:

- Every gate deliverable is **detector + auto-remediator (`--fix`) + blocking backstop** — never a
  detector alone where a fix is mechanically derivable.
- A gate FAIL output prints the **exact fix command** (the ADR-0546 `AUTO_FIX_COMMAND` pattern,
  fixture-pinned so a typo'd target cannot ship).
- A detector-only gate where a fix was derivable is an **incomplete deliverable** (this was
  Alternative-3-rejected in ADR-0545 and is hereby generalized).

Precedent: the in-tree face-settle pattern (`--settle --commit` is the default path, the freshness
gate per ADR-0539 is the backstop); `gofmt`/`cargo fmt --check`, Bazel `buildifier --mode=check|fix`
and `prettier --check` (one binary, check == fix by construction); Tricorder's attached fixes. Per
the founder CLI-retirement directive, `--fix` binaries are local bridge feedback only; merge
authority stays with the gate tests behind `oya-ci-required` (ADR-0515).

### D7 — Soundness defines "possible" in automation-where-possible

The G11 train evidence is binding: **enumerative heuristic fixers lose to hostile review.** Across
four REJECT rounds on the two fixer lanes, every round of point-patches was beaten by fresh
corruption vectors:

- **PR #690** (embedded-asset, ADR-0545): round 1 — `--fix` emitted BUCK missing-comma corruption
  and the gate reported GREEN on an unparseable file; a char-boundary panic on multibyte input; a
  single-target fix versus the ANY-covering-target union (gate-GREEN/build-RED). Round 2 — a
  comment-blind comma heuristic produced double-comma corruption that the shallow
  findability-only output guard PASSED and wrote (FRIC-1781190000 / FRIC-1781190000-guard-v2).
- **PR #691** (kernel-purity, ADR-0547): round 1 — `--fix` wrote a cargo-rejected manifest
  (dangling `dep:kube` feature entry) and reported GREEN, with the corrupt output enshrined in a
  shipped test; a rename blind spot removing LIVE deps; malformed policy failing OPEN
  (FRIC-1781200000). Round 2 — four fresh feature-syntax corruption vectors (weak/implicit/
  sub-feature + target tables), and the round-1 paren-depth fix itself INTRODUCED a comment-blind
  fail-open detector hole. The post-merge optional-dep implicit-feature cross-member vector
  (FRIC-1781210000, fixed in PR #693) confirmed the class once more.

What survived review, both lanes, was the same two-layer sound bound, which is hereby doctrine for
every auto-remediator in the product:

1. **Self-validation of fixer output** — syntactic reparse AND semantic revalidation of every
   rewrite before it stands: `cargo metadata` post-write for manifests; parse round-trip with
   target-findable + injected-value-present assertions for BUCK — with **pre-image rollback**
   (first pre-image per path, so a twice-edited file rolls back to its ORIGINAL bytes).
2. **Refusal on unmodeled input** — anything the fixer cannot soundly model is reported as a
   design-action with remediation text, never rewritten (ADR-0547 round 3 descoped the BUCK fixer
   to refusal-only; ADR-0545 refuses comment-bearing blocks).

The shared fixer harness consolidating these bounds is the `oya-buck-syntax-kernel` destination
(FRIC-1781200001, status `queued-shared-kernel`), governed by ADR-0549 — authored in a parallel
lane; cited as Related, this doctrine does not block on it.

### D8 — The paved-road promise

Anyone who goes through the pipeline ships cloud-native hermetic apps: doctrine ships as policy
packs (the oyatie pack: Rust-first, zero-shell, buck2-hermetic, K8s-native presence —
operator/CRD/PDB/SLO/runbook/OWNERS), plus scaffolding that emits the proven service shape
(kernel/adapter/app crates + helm + SLOs + runbooks — the shape PR #686 hand-built; the scaffold
mechanizes it). Hermeticity evidence rides SLSA-style provenance + cosign attestation (already the
repo's values.yaml posture). Precedent: the Netflix paved road, Backstage golden paths, Google
Tricorder/paved-road analyzer programs, Tekton Chains for signed provenance.

## Current state (honest audit)

The 2026-06-10 audit recorded in the one-pager: **18 gates; 15 kernel-shaped; only 2 shipping
policy as data** — 16 baked policy into code. (The 15/18 count is `fn evaluate` presence, not full
kernel-contract conformance.) Every code-baked policy is an R0 violation: the R0 gap is the
ratchet's own backlog, converted to friction rows and ratcheted down through D5 — never a big-bang
rewrite. Since that audit, the G11 train shipped its new gates born pack-shaped (ADR-0544..0547
each carry a policy JSON the engine reads as data), establishing the migrate-as-touched direction:
new gates MUST be pack-shaped; existing gates migrate as touched.

## Scope

**MVP (in):** kernel extraction (`libs/oya-gate-kernel` + the policy-pack format); 2 existing gates
migrated to packs as proof; the FRIC-total-accounting meta-gate (landed since the one-pager:
ADR-0544, PR #687); the public-repo conformance harness (3 repos); a `GateRun`/`GatePolicy` CRD
walking skeleton before committing the CRD schema.

**Not Doing (and why), carried verbatim in substance from the one-pager:**

- **SaaS console / multi-tenant control plane** — surface #3+; needs the operator stable first.
- **Open-sourcing now** — public-repo testing is a conformance strategy, not distribution.
- **Bring-your-own-build hermeticity in v1** — hermetic-build claims require a hermetic builder;
  non-buck2 repos get the analysis/ratchet tier only. Honest tiering beats a false promise.
- **Big-bang migration of all 18 gates** — ratchet it: new gates must be pack-shaped (enforced via
  the automation-ratchet lane), existing gates migrate as touched.
- **Non-GitHub forges / non-git VCS in v1** — adapter seam reserved per ADR-0510 discipline.

**Open questions (explicitly NOT settled by this ADR):** the product name (working: cloud-ci); where
packs live (`registry/` vs a dedicated `packs/` tree vs OCI artifacts — the Gatekeeper/OPA bundle
precedent suggests versioned bundles mapping to OCI); OSS posture timing (kernel-open /
doctrine-commercial was deliberately not chosen).

## Consequences

### Positive

- The product direction stops being session residue: R0, automation-default, and the soundness
  bound are governed, citable doctrine that gates and reviews can enforce against.
- D6 + D7 together give reviewers a mechanical standard: a fixer PR without self-validation +
  rollback + refusal bounds is incomplete by doctrine, not by reviewer taste.
- The engine/pack split makes the pipeline portable by construction (R0), and the conformance
  harness (D4) keeps the claim testable rather than aspirational.
- The operator surface (D3) reuses a shipped in-tree pattern (ADR-0543), not a new invention.

### Negative

- 16 of the audited 18 gates remain R0 debt until migrated; the migrate-as-touched ratchet means
  the backlog burns down slowly and is visible in the friction ledger the whole time.
- The kernel contract is asserted from a 15/18 census of `fn evaluate` presence; per-gate contract
  conformance is an assumption to validate during kernel extraction (one-pager assumption #1).
- Soundness (D7) raises the cost of every auto-remediator: refusal-only descopes (like the BUCK
  fixer) leave real findings as design-actions until the shared harness (ADR-0549) ships.
- The operator CRD schema is a one-way door once adopters exist; hence the walking-skeleton-first
  sequencing in MVP scope.

### Operational

- buck2 remains the binding verification surface for every gate and fixer in the product.
- Conformance runs against public repos are read-only and bounded (validated on 3 pilot repos
  before any wider claim).

## Alternatives considered

**Alternative 1 — Keep the pipeline as internal repo tooling (no product framing)**
- Pros: zero extraction work; gates keep shipping at current velocity.
- Cons: violates founder directive (a) directly; repo-specific gates keep accumulating as
  unaccounted R0 debt; the paved-road promise (b) is unfalsifiable without a portable engine.
- Rejected: the founder decision of 2026-06-10 settles this.

**Alternative 2 — Adopt OPA/Gatekeeper (or Betterer) directly instead of a Rust-native engine**
- Pros: mature ecosystems; the engine/pack split comes for free.
- Cons: violates the owned-stack and Rust-purity doctrines (Rego/TypeScript cores, transient
  dependencies at the heart of merge authority); the ratchet/baseline/decaying-exception semantics
  and buck2-hermetic posture are not native to either.
- Rejected: proven-patterns doctrine adopts the architecture and reimplements Rust-native.

**Alternative 3 — GitHub required-check as the first PRODUCT surface, operator later**
- Pros: the check already exists as merge authority (ADR-0515); smallest first step.
- Cons: inverts the founder's K8s-operator-first decision; risks the engine fossilizing around one
  adapter's shape instead of CRD-modeled policy.
- Rejected: D3 sequencing gets the same de-risking (kernel extraction first) while the check
  becomes one adapter of the same kernel.

**Alternative 4 — Big-bang migration of all 18 gates to packs in one program**
- Pros: clears the R0 backlog at once.
- Cons: a mass rewrite of live merge-authority gates is exactly the high-blast-radius change the
  ratchet doctrine exists to prevent; the G11 train showed even single-gate fixers need 3 review
  rounds to be sound.
- Rejected: ratchet it — born-pack-shaped for new gates, migrate-as-touched for existing.

## Verification

This is a doctrine ADR; no engine code lands with it. Verification for this change:

- `buck2 test //cloud/cloud-ci/...` green, including the friction-accounting gate over the updated
  ledger (this ADR's row FRIC-1781220000 dispositioned `fixed-in-PR` with evidence citing this ADR).
- The decision-crosswalk face registers ADR-0548 after the faces settle
  (`oya-cloud-ci-face-settle --settle --commit` as the final commit, per ADR-0539).

## Amendment — FRIC-017 disk-reclaim productized as a data-driven Rust-first preflight (2026-06-18, pipeline-glue(b))

The paved-road doctrine "new automation never ships as shell" is applied to FRIC-017 (GitHub-hosted
`ubuntu-latest` exhausts `/` during the buck-out warm restore). The two duplicated multi-line inline
`sudo rm -rf` disk-reclaim blocks in `.github/workflows/oya-ci-required.yml` (the `buck2` and
`gate-affected-set` jobs) are retired in favor of a NEUTRAL engine + a single source-of-truth policy
pack, exactly the R0 reusable shape this ADR prescribes (classification lives in DATA; the engine is
generic).

**New preflight crate (born-blocking workspace member, pure + fs-injected evaluator):**
`ci/facade/runner-disk-reclaim/`. The runner-profile classification
(`reclaim_dirs` + `min_free_gib_after`) is policy-as-data in
`ci/facade/runner-disk-reclaim/runner-disk-reclaim-policy.json`; the
binary best-effort removes the profile's vendor preinstall dirs BEFORE the buck-out restore, logs
structured disk-before/after, and asserts the post-reclaim free-disk floor — emitting a distinct
INFRA-RED exit + signal line so a downstream disk-exhaustion is attributable to INFRA, not CODE. It
runs with ZERO dependency on the restored buck-out cache (buck2 cold-builds the tiny
std+serde_json+libc bin from source). The crate's files born-accounted here (verbatim path mention =
justification; reachable from `cargo-members`; OWNERS-covered):
`ci/facade/runner-disk-reclaim/Cargo.toml`,
`ci/facade/runner-disk-reclaim/BUCK`,
`ci/facade/runner-disk-reclaim/OWNERS`,
`ci/facade/runner-disk-reclaim/src/lib.rs`,
`ci/facade/runner-disk-reclaim/src/main.rs`, and
`ci/facade/runner-disk-reclaim/tests/runner_disk_reclaim.rs`.

The rust-first-automation-hygiene ratchet (ADR-0548 pipeline-glue(a)) counted the two inline blocks
as accepted legacy-bridge debt; this productization replaces each multi-line block with a single
irreducible-glue `buck2 run` invocation at the same `(file,job,step)` index, so the keyed
workflow-inline-shell baseline is unchanged (the residual invocation keys are accepted irreducible
runner-bootstrap glue) while the inline-shell LINE footprint shrinks 6→1. Residual follow-up
(separately tracked): surfacing the preflight's INFRA-RED signal as an infra-vs-code label on the
`oya-ci-required` required context is a broader observability item; this preflight emits the signal,
the required-context labeling is out of scope.

## Amendment — Gate self-conformance meta-gate for the seven-property gate bar (2026-06-29, GH-777)

The pipeline-as-product doctrine now has its own born-blocking meta-gate: every gate must expose
workflow registration, Buck2 unittest/gate wiring, policy-as-data boundaries, a declared fix/no-fix
contract, and scoped hermeticity exceptions with cutover metadata. This prevents the pipeline from
adding one-off detector debt while enforcing D1/D2/D6/D7 against the gate fleet itself.

The meta-gate is intentionally a shape-neutral Rust engine with repo-local facts in policy JSON:
`ci/facade/gate-self-conformance/gate-self-conformance-policy.json`
carries the current gate root, workflow path, non-gate producers, no-autofix reasons, literal-shape
rules, and scoped temporary exceptions for existing orchestrator boundaries such as freshness.
The crate files born-accounted here (verbatim path mention = justification; reachable from
`cargo-members`; OWNERS-covered by the gate tree) are:
`ci/facade/gate-self-conformance/Cargo.toml`,
`ci/facade/gate-self-conformance/BUCK`,
`ci/facade/gate-self-conformance/gate-self-conformance-policy.json`,
`ci/facade/gate-self-conformance/src/lib.rs`, and
`ci/facade/gate-self-conformance/tests/gate_self_conformance.rs`.

## Amendment — Shape-neutral platform delivery fabric planning packet (2026-06-29)

The owned pipeline product scope now includes a shape-neutral platform delivery fabric planning
packet for cloud-native, API-driven, Rust-first admission, scanner, drift, rollout, observability,
and promotion evidence surfaces. The packet is planning/evidence authority only; it does not promote
GitHub-hosted workflow execution, local shell, or repo-specific bridge names into destination
product authority.

The productization packet files born-accounted here (verbatim path mention = justification; root-hub
pointer/reachability-covered where non-crate) are:
`specs/platform-delivery-fabric-productization.json`,
`specs/fixtures/platform-delivery-fabric/parity-target-source-tracking-good.json`,
`specs/fixtures/platform-delivery-fabric/parity-target-source-tracking-bad-authority.json`,
`evidence/pipeline-productization/current-state-20260629.json`,
`evidence/pipeline-productization/metadata-ref-pattern-fixtures.json`,
`contracts/openapi/platform/platform-delivery-fabric-observability-v1.yaml`,
`contracts/openapi/platform/platform-delivery-fabric-observability-v1.meta.yaml`, and
`specs/work-area-content-hash-contract.json`.

These artifacts extend D1/D2/D3/D4/D5 without relaxing the existing guardrails: repo-local facts stay
policy-as-data, adapters stay subordinate to first-party control-plane authority, fresh evidence
expires on cadence, and contradictions create fixuptasks rather than taste-based resolutions.

## References

- `.omc/ultragoal/PRODUCT-pipeline-paved-road.md` (gitignored session one-pager, 2026-06-10 — the
  source artifact this ADR makes durable; audit counts and Not-Doing scope carried from it).
- Dispatch-ledger rows of 2026-06-11 (G11 train: merges #689, #692, #690, #691, #693; REJECT
  rounds and corruption vectors for PR #690/#691 as cited in D7).
- Founder directives: (a) R0 reusability + (b) paved-road promise (2026-06-10, one-pager);
  (c) automation-default (2026-06-11, recorded in ADR-0545/0546/0547); pipeline = universal
  hermetic product (/goal doctrine, 2026-06-09).
- FRIC-1781220000: pipeline-as-product doctrine lived only in gitignored session artifacts +
  memory; closed by this ADR.
- FRIC-1781190000 / FRIC-1781200000 / FRIC-1781200001 / FRIC-1781210000: the fixer-corruption
  friction class behind D7.
- ADR-0515 (cloud-ci merge authority), ADR-0516 (fabric apex vision), ADR-0543 (cloud-kms operator
  kernel/adapter/app precedent, PR #686), ADR-0544 (friction-accounting meta-gate, PR #687),
  ADR-0545/0546/0547 (the G11 gates: automation-default sections + Known Limitations),
  ADR-0549 (oya-buck-syntax-kernel shared fixer harness; parallel lane), ADR-0510 (transient
  adapters), ADR-0539 (freshness/settle), ADR-0541 (corpus liveness graph).
- Precedents: OPA/Gatekeeper ConstraintTemplate/Constraint; Google Tricorder; Betterer;
  Netflix paved road; Backstage golden paths; Tekton Chains; SLSA provenance + cosign;
  gofmt / buildifier / prettier (check == fix); Google SRE postmortem action items.
