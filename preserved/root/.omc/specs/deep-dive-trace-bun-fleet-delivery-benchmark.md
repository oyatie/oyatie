# Deep Dive Trace: bun-fleet-delivery-benchmark

**Status:** trace complete — findings only, no plan approved.
**Date:** 2026-07-27
**Axis (user-selected):** delivery velocity — Bun's post as the standard for agentic throughput, not runtime perf.
**Caveat applied throughout:** working tree is `preserve/hermes-w1-dirty-20260630` (stale/dirty). All load-bearing findings verified against `origin/dev` or the live GitHub API.

## Observed Result

The user asked to benchmark oyatie's agentic delivery velocity against Bun's Zig→Rust port (535,496 lines, 11 days, 6,502 commits, peak 1,300 lines/min, up to 64 concurrent Claude instances across 4 worktrees, adversarial diff-only review, `cargo check` as work queue), and to prepare implementation work covering tests, reviews, merge, code quality, cross-model checks, and avoidance of merge conflict and wasted work.

## Prior Art — this question was already asked and answered

`.omc/specs/deep-interview-pipeline-optimization-program.md` (2026-07-26, one day before this trace) already distilled the same Bun post, adopted six of its techniques by name, and ruled the fanout question:

- `| Bun-scale (64) is the design point | R6 simplifier | 8 lanes — simplest useful, certified |`
- Non-Goals: `16/64-lane Bun-scale certification (revisit after 8-lane SLOs are green)`
- Numeric DoD: FULL-tier `~41 min → ≤10 min`; warm local gate-lane `≤2 min`; zero EMFILE at 8 lanes; 9-item dependency-ordered queue

**Consequence:** this trace should not produce a competing plan. Its value is the measured evidence that the existing spec lacks, and the identification of a prerequisite its queue does not name.

## Ranked Hypotheses

| Rank | Hypothesis | Confidence | Evidence | Why it leads |
|---|---|---|---|---|
| 1 | Shared-state contention makes lanes structurally non-disjoint | **High** | `Cargo.lock` modified by **200 of last 200** commits on dev | Mechanically measured, single number, explains every downstream symptom |
| 2 | No review authority exists to make fanout safe | **High** | Live GitHub API: `required_pull_request_reviews: null` | Verified against the authoritative source, not repo config |
| 3 | Premise mismatch — Bun's preconditions absent (no impl-independent oracle) | **High** | Tests are buck2 `rust_test` in the same graph as impl | Blocks pass@4 economics and mass-refactor validation |
| 4 | Validation loop cost caps throughput | **High** | 71-min median, 38% success, 38% cancelled, 11 runs on worst PR | Measured from 115 real runs |
| 5 | Fleet machinery is prose-only | **Partly refuted** | `check_lane_disjointness` + reorg codemod are real | Machinery exists; adoption does not |

## Evidence Summary

### H1 — Shared-state contention (the binding constraint)

`Cargo.lock`: **200/200** of the last 200 `origin/dev` commits modify it. The `os` capability move (#1416, 41 crates) rewrote 652 of its lines and also touched:

- `Cargo.toml` (workspace members)
- `specs/capability-registry.json` — 12 commits in 2 days
- `specs/reachability-registry.json`, `specs/masterplan.json`
- `ci/facade/crate-registration/src/lib.rs`
- `ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json`
- `ci/facade/embedded-asset-hermeticity/{policy,baseline}.json`

**This is the conflict surface.** Lane 1 confirmed the strangler is serial for **correctness** (shared mutable state), reinforced by policy — not by tool limitation. The codemod (`tools/oya-reorg-codemod-app`) is a pure per-capability structural transform that does not itself require serialization.

**The contradiction:** `tools/oya-fabric-loop-state-app::check_lane_disjointness` is a real, tested mechanical lane-arbiter. It cannot help, because no two lanes are ever disjoint — they all write `Cargo.lock`.

**Bun's structural difference (the transferable lesson):** Bun settled crate boundaries and manifests **once, in the prep phase**, then fanned 64 agents over *leaf files* that no other agent touched. Shared-manifest churn was per-wave, not per-work-unit. oyatie's reorg re-mutates shared registries on **every** move, making work units maximally contended by construction.

### H2 — No review authority (the safety blocker)

Verified against the **live GitHub API**, not repo config:

```
required_pull_request_reviews: null
contexts: ["oya-ci-required"]
strict: false
```

- `.github/branch-protection.yaml:49` — `required_approving_reviews: 0`; line 55 — "`oya-pr-review` is intentionally ABSENT from required checks"
- `infra/branch-protection/dev.json` — `"live": false`
- Only **5** workflow files exist on origin/dev. `pr-review.yml` — which `oya-intelligence-pr-review-dispatcher-app/src/main.rs` claims invokes it — **does not exist**
- `oya-intelligence-subagent-runtime-app` has **no HTTP dependency** (no reqwest/tokio) — `--mode anthropic-api` hard-errors. It is physically incapable of calling a model
- `rollup.rs` — **empty findings ⇒ APPROVE** (test `empty_findings_default_to_approve`)
- `oya-bot-autofix` (ADR-0531) — ~380-line diff renderer, no caller, no remediation producer, no webhook

**No LLM verdict is admissible to any gate, because no CI lane can make an LLM call at all.**

The one review-adjacent gate that *does* block is `ci/facade/baseline-ratchet/tests/gate_registration.rs` — it fails CI if any document *claims* review admission is live while F-PR5-06 is open. It is the inverse of a review gate.

### H3 — Premise mismatch

Bun's method depends on preconditions oyatie does not hold:

| Precondition | Bun | oyatie |
|---|---|---|
| Impl-independent acceptance oracle | 60k TS assertions, language-agnostic to the port | buck2 `rust_test` in the same graph — a mass refactor invalidates test and impl together |
| Known-correct reference implementation | The Zig original | Absent for net-new work |
| Fast local signal | `cargo check`, seconds | Trustworthy signal is a ~71-min CI run |
| Work units disjoint by construction | One file per agent | Every unit writes `Cargo.lock` |

**Consequence for model routing:** DeepSWE v1.1 data (113 tasks, 50 configs, `/artifacts/v1.1/leaderboard-live.json`) shows 4 × `gpt-5-6-luna/max` reaches **90.3% pass@4 at $9.17** vs 1 × `claude-opus-5/max` at **73.6% pass@1 for $10.43** — cheaper, better, faster. But pass@4 is unspendable without a mechanical verifier to identify which attempt passed. **The oracle gap blocks the fanout and the cheap routing through one shared cause.**

### H4 — Validation loop cost

Measured from 115 concluded `oya-ci-required` runs:

- **Median successful run: 71 min** (max 87) — worse than the 31-min documented cold-build figure
- **38% success / 24% failure / 38% cancelled**
- Worst PRs consumed **11 CI runs**; several burned 4–7
- FULL tier: ~31 min cold build per pass, up to 2 passes, 120-min timeout ceiling
- No remote execution, no shared cache (`.buckconfig` has no `[buck2_re_client]`; ADR-0560/0612 both `Proposed`). Every lane pays its own build

### H5 — Machinery (partly refuted)

**Exists and is real:**
- `tools/oya-reorg-codemod-app` — atomic git-mv + Cargo/BUCK/import rewrite across ~200 move-fatal path-dep files
- `tools/oya-fabric-loop-state-app` — two-plane claim/heartbeat state + `check_lane_disjointness`
- `oya/ci-tide/` — 2,098 lines of real Tide, `dry_run=true` default, batching+speculation explicitly deferred, zero deployment refs
- `marketplace/facade/dev-cli/src/commands/merge_queue.rs` — working, unit-tested speculative-window batching algorithm (Zuul-style) — in the **retired** CLI, behind a synthetic oracle
- 43-leg gate matrix; `gate-self-conformance` is an anti-unwired-gate gate; GATE-4 `automation-coverage` catches `advisory_claiming_enforced`; `baseline-ratchet` has an inert-door detector

**Does not exist:**
- No PORTING.md / LIFETIMES.tsv analogue (zero grep hits)
- No compiler-errors-as-work-queue tooling
- No worktree fan-out automation in owned repo code
- **No gates for `todo!()`, `unimplemented!()`, `#[ignore]`, or deleted assertions** — the exact slop classes fanout produces at scale
- No git hooks at all (installer retired in #634); `tools/hooks/*.sh` are advisory, always exit 0, scoped to `libs/oya-check-*`
- Auto-rebase (#123) — referenced nowhere in the tree

**Adoption gap:** every fabric-loop commit is dated **2026-07-02**. Built and exercised at n=2 lanes in a single day; the only later touch is a 2026-07-22 restart anchor. Twenty-five days, zero further use. One-off demo, not adopted practice.

## Evidence Against / Missing

- **H1**: batching is empirically proven — #1416 landed 41 crates in one PR, contradicting the "strictly serial one-move-per-PR" playbook (which is agent scratch, absent from origin/dev, and self-contradicting on ADR-0563 vs ADR-0614)
- **H4**: CI fan-out is genuine (9 jobs concurrent); merge admission is GitHub-native `merge_group`, mechanical once green; the trusted-baseline fast path (#1323/#899) already targets the FULL-tier 2× cost
- **H4**: `docs/**` and `**/*.md` are declared inert, so doc-heavy commits do *not* pay FULL tier
- **Not measured**: the FULL-vs-narrow tier distribution across real PRs. Worst-case bounds are documented; modal cost is not

## Convergence

H1 (contention), H2 (no review), H3 (no oracle) converge on a single structural statement: **oyatie's work units are neither independently executable nor independently verifiable.** Bun's method requires both. This is one cause with three faces, not three problems.

## Most Likely Explanation

The obstacle to Bun-scale delivery in oyatie is **not agent capability, model choice, or orchestration sophistication**. It is that (a) every work unit writes the same handful of shared files, so lanes cannot be disjoint; (b) nothing reviews agent output before merge, and the slop classes fanout produces are ungated; and (c) no implementation-independent oracle exists to make cheap-model-×N verification spendable.

All three are enumerable and bounded. Most of the required machinery already exists in-tree, disconnected. **The work is wiring, not building.**

## Critical Unknowns

1. **Reorg direction is unresolved, not frozen.** `reorg-pipeline-consensus-plan.md` records `APPROVED SCOPE` for the freeze; `reorg-completion-plan.md` (2026-07-27, newest) is `PENDING FOUNDER DECISION` between finish (Option B) and stop (Option C). Scope of the contention fix depends on this.
2. **Modal CI cost** — FULL-vs-narrow distribution across real PRs is unmeasured.
3. **Bun prep-phase detail** — whether "settle shared state once, then fan out over leaf files" is Bun-stated or my inference (lane 4 outstanding at time of writing).

## Recommended Discriminating Probe

Take one capability move and attempt it **twice concurrently** in two worktrees. Measure exactly which files collide. That converts the contention hypothesis from measured-by-proxy (commit frequency) to demonstrated-by-reproduction, and produces the precise shard list for the fix.

---

# ADDENDUM — documentation study (2026-07-27)

Read first-hand: ADR-0562 (§1–§10.12, §10.26–§10.29 + Consequences), ADR-0615 (full), ADR-0056 (full), ADR-0570 (full), ADR-0512. Plus reader syntheses on capability shape and de-brand.

## The decisive constraint: moves cannot be parallelized

**ADR-0512**, quoted in ADR-0562:

> "a structural migration of this class **MUST run as a dedicated, exclusive, post-acceptance change on a stable tree** … **never merged concurrently in a PR drain. Violating this is what broke `dev` and motivated this ADR.**"

**ADR-0562 §10.29** — batch size is not tunable:

> "The move-plan schema carries exactly ONE `capability` field and exactly one plan may be active, so **a batch cannot span destination roots.** Batch size is therefore not a dial to be tuned for risk appetite: **it is the size of one destination block.**"

**Consequence:** the Bun-style fanout cannot target the moves. It must target the **prep** — precomputing per-crate decisions centrally, exactly as Bun's `LIFETIMES.tsv` precomputed per-field lifetimes ("trust it over local guessing"). Moves then execute serially against precomputed answers.

## A move requires three orthogonal taxonomies

| Taxonomy | Values | Encoded in | Authority |
|---|---|---|---|
| Capability | 24, closed | top-level dir | ADR-0562 §3 (first-match-wins; tie-break = lowest ADR-0280 DAG node) |
| Face | core/ports/adapters/facade (+`harness`, meta-dirs only) | path sub-fold **and** manifest `face:` facet | ADR-0562 §4 |
| Layer | 12 (ADR-0565 removed `graphql`; ADR-0056 still prints 13 — stale) | crate **name suffix** | ADR-0056 BNF |

**"Port" is overloaded.** ADR-0056: a port is a trait declaration in the `kernel` *layer*. ADR-0562: `ports/` is a *face*, "the stable seam." §10.6 homed a framework-free HTTP route-table at `iac/ports/rest` — face=ports, layer=rest, zero traits.

Face is decided on **dependency evidence, not name** (§10.26 reclassified a "SUGGESTED ports/" crate as `adapters/`; §10.29 put a 7-public-trait crate in `core` because every implementor lived inside it). §4: *"If you cannot say which side a crate is on, it is mis-factored and must be split."* No gate checks this.

## Per-move obligations beyond `git mv` (20)

**Codemod does:** git-mv · Cargo.toml name/dep/path-dep recompute (~200 move-fatal `../../../`) · workspace member globs · Rust `use`/`extern crate` · BUCK labels/`crate_root`/`mapped_srcs` (field-agnostic) · Cargo.lock (owned transform kernel, byte-exact, injective, fail-closed) · ADR **path** citations. Invertible; fixture round-trip proven.

**Human authors, every move:** registry `absorbs_current_dirs` + **self-slug** (omit ⇒ `MEM-NEW-UNMAPPED-CRATE`) · membership `scan_roots`/`allowed_top_level_dirs` · acyclicity `crate_root_globs`/`unclassified_roots` (**no gate — silent coverage loss**) · workspace glob **before** codemod runs (else `workspace_member_explicit_path`, `frozen_empty`, unbaselineable) · `[workspace].exclude` for non-crate SLO subtree · SLO co-move · **catalog row** (`catalog_live_crate_without_row` is `frozen_empty` — "a missing row cannot be laundered into the accepted baseline by regeneration") · OWNERS + reachability entry · ADR §10.x with **byte-exact unbraced unwrapped** paths · embedded-asset policy+baseline **as a pair** (set equality; either alone is RED) · every other gate's `roots` that contained the source root · face assignment.

**§10.29's generalizable lesson:**
> "a relocation changes a gate's scan scope in two independent ways, and **only one of them is ratcheted** … **there is no gate asserting that those allowlists agree with the live top-level directory census, and until there is, this check is manual.**"

**Self-reported false green (§10.29):** a draft claimed 41 crates were "scanned by the policy at their new home — verified green (45 tests pass)." Both halves wrong; those were the gate's own unit tests over policy data. **"The crates were in fact scanned by nothing."**

## De-brand — premise corrected

**De-brand is NOT deferred: 474 of 926 crates (51%) are already de-branded.** Deferred are (a) the profile flip and (b) a residue class.

**The profile is not a rename engine.** `NamingConfig::neutral()` sets `required_prefix: ""`, and both naming gates short-circuit. "Flip the profile" = *stop enforcing* `oya-`. Renaming is a separate codemod act.

**LANDMINE — a naive `profile = 'neutral'` flip silently darkens the layer gate repo-wide.** `libs/oya-governance-predictable-naming-kernel/src/lib.rs:123`:
```rust
crate_name.starts_with(prefix) && crate_name.len() > prefix.len()
```
With `check_family_prefix: ""`, `starts_with("")` is TRUE for every crate ⇒ **every crate classifies as check-family** ⇒ `declared_role = None` ⇒ violation skipped ⇒ `cloud-ci-bnf-layer-suffix` returns **zero findings on any corpus**. The gate's neutral test only asserts `bnf_missing_oya_prefix` is absent; the blind spot is untested. *(Code trace, NOT execution-verified.)*

**Surgical alternative:** keep `profile = 'oyatie'`, author `[naming] required_prefix = ""` in `oya-ci.toml`. `NamingConfig` fields carry `#[serde(default)]`, so `allowed_roles`/`check_family_prefix`/carve-outs survive. Drops the prefix rule, keeps the layer-enum rule live.

**Phase-0 item 3 was routed around, not satisfied.** `oya-ci.toml:21` is still `profile = 'oyatie'`. Instead an advisory hatch landed (`artifact-inventory-registry/src/main.rs:888`) making partially-de-branded rows advisory — "so visibility expansion does not become a northstar-debrand merge blocker." Half the workspace is de-branded and green via the hatch, not the profile.

**`cargo == path-tail` is enforced by ZERO gates.** The natural enforcer (`PackageNamePathMismatch`) is prefix-gated, so it is **dead on all 474 de-branded rows**. The core invariant of the de-branded world rests on author discipline.

**Residue accumulates ~1 divergence per move — 23 measured** (de-branded package + still-branded bin). **17 of 23 are in `ci/`** — the tooling that would run the de-brand lane (bootstrap-order problem, uncosted). Growth is **undetectable**: `oya` is not in `forbidden_stems`. Tracking is **"task #63", which appears 10× in ADR-0562 prose and nowhere else in the repo** — a prose IOU, not a work item.

**The residue class needs splitting.** Some entries are permanent by design: `oya-tenant` is Tier-A semver-protected and distributed via Homebrew/apt/winget/ghcr (ADR-0167); `marketplace-dev-cli`'s bin is `oya` itself. Those will never de-brand. Filing them with deferred cleanup conflates two different things.

**Highest-fan-in crate is undecided.** `oya-data-boundary-kernel`: fan-in 128, 301 files, 673 occurrences. Codemod handles ~95%; **~40 hand-edited governance/registry/doc sites are cross-checked by nothing** (8 JSON registries, 30 `.md` including 6 ADRs — step 5b rewrites path citations, never crate names in prose). It lives in `libs/`, which §9 says "dissolves," and sits in `frozen_unmapped_baseline` — **no decided destination, and its move lands in the same PR as its rename.**

**ADR-0532's ratified rename set is unexecuted AND stale** — its targets have since moved and been renamed to different de-branded names.

## Target shape is substantially unbuilt (verified on origin/dev)

`base/`, `build/`, `app/` **do not exist**. `policy/` is a registered, scan-rooted, allowlisted **empty slot**. `libs/` holds ~180 crates including all 60 frozen-unmapped. `governance/` contains only `corpus/`. The closed `faces` array **has zero code consumers** — `data/ops`, `comms/messenger`, `iac/modules`, `intelligence/testing` already exist in no face list. §10.29: *"a closed registry that nothing enforces will describe the tree less and less accurately, silently."*

## Known unenforced invariants

- **No intra-capability face-direction gate** (§10.6). `facade → adapters` at composition roots is "the universal exception"; the §4 carve-out was never written.
- `owning_service()` "recognizes only `cloud/`+`oya/` and is **STRUCTURALLY BLIND**" to capability roots (§10.26/§10.27) — every moved crate is outside its classification.
- `port-placement` (ADR-0570) is status **Proposed**, born-advisory, detect-only, 5 frozen violations; flips blocking only at baseline 0.
- 69 `oya-check-*` crates exist; **exactly one** is referenced in any workflow, and that one is not in the required fan-in.

## Gate-code reality (verified against origin/dev)

**ADR-0562 §10.6 is STALE.** `facade-core-layering` DID land and IS in the required matrix (`oya-ci-required.yml:187`). It enforces **same-capability `facade/* → core/*`** via static BUCK parsing (deliberate: `intelligence/facade/worker` carries the edge in BUCK with zero Cargo path-deps). 35 baselined keys, shrink-only. **`facade → adapters` remains unenforced** — the §10.6 composition-root gap survives in part. Cross-capability core reach is explicitly out of scope: *"12 packages / 21 edges reach ANOTHER capability's core (marketplace/facade/dev-cli alone reaches 9). That is a different rule … deliberately NOT gated here."*

**`port-placement` baseline is 6, not the 5 ADR-0570 states.** Still advisory; flips blocking at 0.

**`core-dependency-isolation` (`cloud-ci-kernel-purity`)** — born-blocking, **frozen-empty, zero tolerance**, scope `["*-kernel","*-core"]` (census 156). Enforced "purity" is a **21-entry transient-infra denylist** (kube/rustls/sqlx/postgres/sea-orm/diesel/aws-sdk/etcd/zitadel…) applied across the **workspace-internal path-dep closure**. `tokio`, `axum`, `reqwest`, `tonic`, `hyper` are **permitted**. ADR-0056's "zero I/O, zero async, zero business logic" has **no gate anywhere**.

**`crate-layer-suffix` never sees de-branded crates.** The producer filters the corpus first (`artifact-inventory-registry/src/main.rs:861-874`): `if name.starts_with(prefix)`. ⇒ **ADR-0056's layer enum has ZERO enforcement over the entire capability-first tree**, and coverage shrinks with every de-brand. Also: **`-core` is a BNF violation** (`oya-bar-core` → `bnf_unknown_role`) — `core` is simultaneously the primary face, a kernel-purity glob, and a non-canonical layer; only the de-brand filter prevents the collision.

**Tier blindness quantified.** `owning_service()` splits on `service_roots: ["cloud","oya"]`; an unclassified pair `continue`s. Of 905 governed crates: **238 tier-classified, 667 tier-blind — 402 of the blind sit under capability roots** (iam 68, workflow 48, os 41, intelligence 32, comms 24, data 23, tenancy 22 …). ~402 already-moved crates are structurally invisible to R1–R4. `service-tier-metadata` has the same root list, so moved crates also drop out of tier/`tier_subtype`/`dr_tier` coverage.

**The `face:` manifest facet is NOT implemented.** ADR-0562 §4/§6 says "the lint asserts they agree" — zero `face` field reads in `module-membership` or `service-tier-metadata`. Live proof: non-canonical sub-folds exist unchallenged (`intelligence/testing`, `comms/messenger`, `data/ops`, `iac/modules`, `observability/observability`).

## Six mechanisms by which a mechanically-correct `git mv` creates violations

| # | Mechanism | Effect |
|---|---|---|
| (a) | Entering `<cap>/facade/` | `oya/` has no `oya/facade/`, so the crate was invisible. Same BUCK edge, new path ⇒ `facade_core_direct_dep`, unbaselined, **RED** |
| (b) | Entering `<cap>/adapters/` | A `pub trait ThingStore` green at `oya/<svc>/crates/…` ⇒ `PP-PORT-IN-ADAPTER`, **fails closed**. Source text unchanged |
| (c) | Rename pulls into kernel-purity | Cargo name becomes path tail; `observability/core/kernel` ⇒ `observability-kernel` ⇒ matches `*-kernel` ⇒ enters a **frozen-empty zero-tolerance** gate with closure-walk. A path-dep on an `sqlx` neighbour was fine before, `KP-TRANSIENT-DEP-CARGO` after |
| (d) | `TDA-STALE-BASELINE` | Baseline subjects are `"<from-dir> -> <to-dir>"`; moving a baselined crate without re-emitting in the same PR is **automatic RED**. Same class for `port-placement` (keyed on `member_path`) |
| (e) | **The facade-core code flip — LATENT NOW** | Emitted code depends on whether the capability has *any* `ports/` dir; the two codes have **separate baselines**. `intelligence/` and `compute/` have **no `ports/`**. The first PR adding *any* crate under `intelligence/ports/` — a purely additive, obviously-correct improvement — **REDs twice**: keys flip to `facade_core_direct_dep` (unbaselined) *and* their `facade_core_no_ports_layer` rows go stale (a vanished entry is itself a finding). Only clean path: create `ports/`, rewire both facades, edit both baselines, one PR. Identical trap in `compute/` |
| (f) | **Scan-root change silently DROPS the crate** | `oya/<svc>/crates/X` → `<cap>/core/x` ⇒ `owning_service()` → `None` ⇒ R1–R4 stop applying. **A live `TDA-SUBSTRATE-UPWARD` is erased by relocation, not fixed.** The de-brand additionally drops it from `crate-layer-suffix` |

**The synthesis sentence:** a single move scopes a crate **into** facade/adapters/kernel-purity rules it never had to satisfy, and **out of** the tier and BNF-layer rules it did. **Capability-first relocation currently trades ADR-0245 tier enforcement for ADR-0562 face enforcement. The two do not overlap.**

That is the mechanical form of "the move adds debt": mechanism (f) is a *laundering* direction — violations disappear without being fixed.

## Revised conclusion

The Bun-shaped work in oyatie is **the prep, not the moves**. Precompute the per-crate decision table for the remaining 466 — destination, face (with dependency evidence), layer, catalog row, SLO files, scan-root deltas, fan-in, blockers — adversarially reviewed, then trusted verbatim. That workload is parallel, touches no tracked file, and is exactly Bun's `LIFETIMES.tsv` pattern. Moves stay serial per ADR-0512.

Shared-state sharding therefore matters **less** for the reorg than first concluded, and remains relevant for concurrent non-move work. `ADR-0562.md` itself (46 commits, 300 → 2,838 lines) is a real conflict surface.

## Stale-prose hazards found (worth fixing regardless)

- **ADR-0515's `homes:` and D1 "verified ground truth" point at `cloud/cloud-ci/gates/*`, which does not exist on origin/dev.** Gates live at `ci/facade/`.
- `strangler-move-playbook.md` asserts ADR-0563 retains `move-manifest.generated.json`; ADR-0614 de-committed it.
- `oya-intelligence-pr-review-dispatcher-app/src/main.rs:16-17` claims its FAILURE blocks merge via branch protection. False on origin/dev.
- Crate counts differ by document: 510/927 (44.9%, shallow clone) vs 466/926 (full clone).
