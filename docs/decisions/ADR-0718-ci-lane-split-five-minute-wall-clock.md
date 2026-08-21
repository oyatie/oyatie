---
doc_status: published
id: ADR-0718
title: "CI lane split and five-minute wall clock: fast format lane, parallel clippy, and nextest"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-18
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0716]
amended_by: []
depends_on: [ADR-0716]
related: [ADR-0554, ADR-0700]
milestone: W0
deliverables:
  - id: ADR-0718-D1
    description: "Split the required lint job into a fast blocking format lane and a separate advisory clippy job that runs in parallel and is absent from the oya-ci-required fan-in."
    exit_criteria: "The lint job runs rustfmt --check only, installs no cargo cache, and reports in under two minutes; clippy runs as its own job, keeps continue-on-error, and does not appear in the fan-in needs list; the fan-in verdict is unchanged in meaning because clippy was already continue-on-error and could never fail a merge."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D2
    description: "Authorize the workflow inline-shell baseline replacement window required by D1, because the baseline is keyed <file>::<job>::<step> and moving a step between jobs is a remove plus an add rather than a shrink."
    exit_criteria: "The baseline declares a replacement_window with a bumped schema_version, a reason, and this ADR; the key count does not increase; validate_replacement_window_authorization admits the window and a window missing reason or ADR still fails closed."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D3
    description: "Run the workspace and kernel suites under cargo-nextest, and update the two gates that encode the literal cargo test invocation so they assert the property rather than the spelling."
    exit_criteria: "rust_first_automation_hygiene accepts either runner while still requiring --locked --workspace; gate-self-conformance derives workflow_registered from either runner so no gate crate is unregistered by the swap; doctest coverage is unaffected, evidenced by zero doc examples across the .rs corpus and doctest = false already declared by the large majority of manifests."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D4
    description: "Amend ADR-0716-D1's warm wall-clock exit criterion from under fifteen minutes to under five minutes, and require that the figure be measured on two consecutive warm runs rather than asserted."
    exit_criteria: "Two consecutive green post-merge runs on dev report a warm wall clock under five minutes for the oya-ci-required fan-in; if the constant-factor work in D1-D3 plus the smoke retirement does not reach it, the measurement is recorded and D5 opens."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D5
    description: "Conditional, and deliberately not pre-authorized: if D4's measurement shows the full-workspace compile is itself the floor, re-introduce affected-set selection on the cargo graph rather than on buck2, reversing the narrow clause of ADR-0716-D1 that removed affected-set baselines."
    exit_criteria: "Opened only with D4's recorded measurement attached. Any such change must preserve ADR-0554's fail-closed property: a derivation failure escalates to the full workspace run and never skips."
    verified_by: "ADR-0718-D4 measurement record"
---

# ADR-0718: CI lane split and five-minute wall clock: fast format lane, parallel clippy, and nextest

## Status

**Accepted** (founder directive 2026-08-18: reduce CI wall clock to five minutes per run).
Amends ADR-0716, which made the cargo workspace graph the CI merge path and set the warm
wall-clock exit criterion at fifteen minutes.

## Context

ADR-0716 replaced a 1,262-line, 12-job buck2 workflow with a cargo-graph workflow and set
"warm PR wall clock under fifteen minutes" as its exit criterion. That target was met. The
founder directive now sets five minutes, and the measurements below say the remaining
distance is not evenly distributed across the workflow.

Three facts, each measured rather than assumed:

**The blocking lane waits on an advisory compile.** `rustfmt --check` compiles nothing — it
is differential over the changed `*.rs` files only. It nonetheless shares a job with
`cargo clippy --locked --workspace --all-targets`, which is `continue-on-error: true` and
has therefore never been able to fail a merge. The blocking format verdict cannot report
until that advisory full-workspace compile finishes, and the job restores and saves a shared
cargo cache that the format check has no use for.

**Cache hits do not prevent recompilation.** A run logged `Cache hit for: v0-rust-cargo-ci`
with `Cargo.lock` untouched and still compiled 1,565 crates. `swatinem/rust-cache` caches
registry dependencies, not workspace members, and this workspace has 885 members. First-party
crates therefore rebuild on every run regardless of cache state.

**Caching alone cannot reach the target.** A fully warm, no-change
`cargo test --locked --workspace --no-run` against a 75 GB target directory did not complete
within ten minutes locally. The binding constraint is that CI compiles and tests every
workspace member on every pull request, including documentation-only ones.

A fourth fact is organisational rather than technical: this repository already contains an
affected-set runner (ADR-0554) that computes a merge-base diff, classifies each changed file,
takes an owner and reverse-dependency closure, and builds and tests only the affected targets,
failing closed to a full run on any derivation error. It drives buck2. ADR-0716 moved the
merge path to cargo and removed the affected-set baselines, on the reasoning that the
machinery existed only to make cold buck2 affordable. That reasoning was sound for its
evidence; the evidence above is new, because it shows the full-workspace cargo compile is
itself the wall.

## Decision

1. **The blocking lane must not wait on an advisory one.** The required `lint` job runs the
   differential format check alone. Clippy moves to its own job, keeps `continue-on-error`,
   runs in parallel with `test`, and is absent from the `oya-ci-required` fan-in. This removes
   no enforcement: a `continue-on-error` job could never fail a merge, so its absence from the
   fan-in changes the verdict's meaning not at all.

2. **Moving a baselined step requires an authorized replacement window, not a quiet rekey.**
   The workflow inline-shell baseline is keyed `<file>::<job>::<step>`. Moving the clippy step
   is therefore a remove plus an add, which the shrink-only ceiling correctly refuses. This
   ADR is the authorization that ADR-0716-D3's replacement-window mechanism requires. The
   mechanism deliberately demands an ADR that does not exist on the protected merge-base, so
   an amendment to ADR-0716 could not have served; that is why this is a new decision record
   rather than an edit to ADR-0716.

3. **Tests run under cargo-nextest, and the gates assert the property, not the spelling.**
   Two gates encode the literal `cargo test --locked --workspace`:
   `rust_first_automation_hygiene` asserts the gate fleet runs under it, and
   `gate-self-conformance` derives `workflow_registered` from it, so a naive swap silently
   unregisters gate crates from the fan-in. Both are updated to accept either runner while
   still requiring `--locked --workspace`. Doctest coverage is unaffected: the corpus contains
   no doc examples, and the large majority of manifests already declare `doctest = false`.

4. **The warm wall-clock exit criterion becomes five minutes, measured on two consecutive
   warm runs.** This amends ADR-0716-D1's fifteen-minute figure.

5. **Affected-set selection on cargo is NOT authorized by this ADR.** It is named as a
   conditional deliverable so that the option is recorded with its evidence requirement, not
   so that it may be started. Reversing a founder decision on speculation is the failure mode
   this clause exists to prevent: D5 opens only with D4's measurement attached, and any such
   work must preserve ADR-0554's fail-closed property, where a derivation failure escalates to
   the full workspace run and never skips.

## Consequences

The first run after the linker and runner changes land will be slower, not faster: `RUSTFLAGS`
participates in the rust-cache key, so the cache is cold exactly once. Wall-clock claims must
be read from the second run onward, which is why D4 requires two consecutive measurements.

Clippy leaving the fan-in makes its red visible but non-blocking, which is what it already
was. If the repo-wide lint-debt cleanup lands and clippy becomes blocking, it must be added
back to the fan-in in that same change; this ADR does not pre-authorize that.

The five-minute target may not be reachable by constant factors alone. That outcome is an
expected result of D4, not a failure of it, and it is precisely the evidence D5 requires.
The per-PR Windows/macOS smoke is retired (see ADR-0716 amendment 2026-08-21).
The Linux `test` job is a two-leg native matrix (amd64 + arm64); wall clock is
the slower leg. The background `cargo build --workspace --tests` prefetch is
removed: nextest is the compile proof, and two cargo processes on one target
dir serialize on the build lock. Product `cargo build --release` / `buck2
build //...` are CD-train work, not this gate.

## Amendment 2026-08-18: measured step profile, and the linker deferred

The linker swap named in the first draft of this record is DEFERRED, not adopted. mold is
not installable through the pinned `install-action` (it falls through to `cargo-binstall`,
which cannot infer a binary for the `mold` crate), `rui314/setup-mold` publishes no tagged
release to pin, and `lld` does not appear in the runner image manifest. Installing it by
`apt` is refused on the same grounds as everywhere else in this workflow: an unretried
package install already cancelled a live-postgres lane and ejected a merge-queue entry.

The first per-step profile of the required lane, measured after the debuginfo change landed,
reprioritises the remaining work:

| step | seconds | share |
| --- | --- | --- |
| Workspace tests | 681 | 70.4% |
| Materialize generated faces | 192 | 19.9% |
| Kernel workspace tests | 35 | 3.6% |
| checkout (`fetch-depth: 0`) | 20 | 2.1% |
| everything else | 39 | 4.0% |
| **total** | **967** | **16.1 min** |

Two consequences. First, the `test` job IS the wall clock: every other job — both smoke legs,
lint, and both live-postgres lanes — completes inside it, so the fan-in never waits on them.
Optimising anything else changes billable minutes, not wall clock. Second, face
materialization is 192 seconds of buck2 work performed before a single test runs; in a
300-second budget that single step would consume 64% of it. Neither fact was visible before
the profile, and D4's measurement obligation exists precisely to surface this class.

### What the profile changed, and what is still not authorized

The profile above reprioritised the work and added three changes to this record, all inside the
same replacement window:

- **sccache** as `RUSTC_WRAPPER` with the GitHub Actions backend. `swatinem/rust-cache` caches
  registry dependencies but NOT workspace members, of which there are 885; a run logged a cache
  HIT with `Cargo.lock` untouched and still compiled 1,565 crates. `CARGO_INCREMENTAL=0` is a
  prerequisite and is already set workflow-wide.
- **`buck-out` caching — proposed, measured, and REJECTED.** The observation was right that
  buck2's output is cached nowhere, but caching it is wrong on three counts. Measured `buck-out`
  directories in live worktrees run 940 MB to 15 GB, against a 10 GB per-repository Actions cache
  budget with LRU eviction — a multi-gigabyte entry would evict the `rust-cache` entries that do
  work, making compile worse to speed up a smaller step. Restoring roughly a gigabyte costs a
  large fraction of the 192 seconds it was meant to save. And `buck-out` carries buck2 daemon and
  materializer state with absolute paths, so a restored directory is not reliably reused across
  runners. Decisively, the overlap above already hides the whole 192 seconds behind the 681-second
  compile, so the cache would buy no marginal wall clock at all while spending the entire cache
  budget. Not adopted.
- **Compile/materialize overlap.** The two largest steps run strictly in series today and are
  independent: no build script reads materializer output, and the gate crates read the faces at
  RUNTIME through a path join rather than a compile-time include. Backgrounding the workspace
  build during materialization reclaims the smaller of the two, roughly 190 seconds. This GROWS
  the step from one shell line to four, against a policy whose direction is to shrink workflow
  shell. That is a real cost, taken deliberately: Actions has no native intra-job parallelism to
  express the overlap declaratively, and the step still owns no branching or git topology. It is
  recorded in the baseline rather than hidden.

Two levers remain OUTSIDE this record. **Larger runners** are the single biggest remaining factor
— compile is CPU-bound on the four-core default, and the account's plan supports larger hosted
runners — but the label must exist before `runs-on` names it, or every job queues forever; that is
an organisation settings change, not a workflow edit, and it is not made here. **Affected-set
selection** remains D5: conditional, evidence-gated, and still not authorized.

### Amendment: larger runners are unavailable, so D5 becomes the binding path

The founder has ruled out larger hosted runners. The four-core default is therefore fixed, and
that closes the constant-factor route: after the overlap the test job is still on the order of
thirteen minutes, against a five-minute target. Two levers remain, and only one of them is
authorized.

**Workspace-crate caching, adopted here.** `swatinem/rust-cache` exposes `cache-workspace-crates`
and it defaults to FALSE — which is precisely the measured defect, a logged cache hit with an
untouched lockfile that still compiled 1,565 crates. Enabling it is a one-line change and it is
viable specifically because the debuginfo change already landed: without that, `target/` for 885
members would be too large to store against the 10 GB per-repository cache budget. The test lane
also takes its OWN cache key, because the live-postgres lanes compile a small subset and sharing a
key would let their thin save overwrite this lane's cache on every run.

**Sharding, NOT adopted here, and now genuinely available.** Earlier in this work sharding was
rejected on the grounds that each shard would pay its own full compile, multiplying rather than
dividing. That objection was correct while nothing cached workspace members. With sccache and
workspace-crate caching in place the repeated compile becomes mostly cache hits, and `cargo
nextest` supports native partitioning, so N standard four-core runners give 4N cores without a
larger-runner SKU. It is deliberately left for a separate change so that D4's two-run measurement
can attribute the caching effect before another variable is introduced.

D5 — affected-set selection on the cargo graph — remains the structural answer and remains
unauthorized. With larger runners off the table it is now the only lever that can plausibly reach
five minutes for the common case, because it is the only one that stops compiling 885 members for
a change that touches none of them.

## Amendment 2026-08-21 — smoke retirement landed here, not a phantom ADR

The paragraph that deferred Windows/macOS smoke retirement to "its own decision
record, not cited here by number" is superseded. Minting a new live ADR to delete
a job that already could not fail a merge fails the hyperscaler bar (more law for
less work) and the corpus ratchet (`live_adr_files` is at ceiling). The deletion
amends ADR-0716's merge-path job list; this record records that the complementary
retirement it named has executed.
