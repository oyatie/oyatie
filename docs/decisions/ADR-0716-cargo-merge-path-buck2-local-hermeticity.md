---
doc_status: published
id: ADR-0716
title: "Cargo is the CI merge path; buck2 is local hermeticity plus a weekly smoke"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-14
door: two-way
owner: council-architecture
supersedes: [ADR-0560]
superseded_by: []
amends: [ADR-0700, ADR-0554, ADR-0556]
amended_by: [ADR-0718]
depends_on: []
related: [ADR-0515, ADR-0532, ADR-0613]
milestone: W0
deliverables:
  - id: ADR-0716-D1
    description: "Rewrite the required CI as a cargo-graph workflow: lint (fmt + clippy), test (materialize faces + cargo test --workspace), two live-postgres lanes, and the zero-build fan-in. All job/step names are self-explanatory, debranded, and carry no ADR or PR numbers."
    exit_criteria: "The workflow contains no buck2 build/test verdict steps (buck2 appears only as the face materializer's internal helper, installed digest-pinned), no serial producer barrier, no affected-set baselines, and no artifact upload/download; swatinem/rust-cache with a shared key warms every lane. Warm PR wall clock is under 15 minutes (measured on two consecutive green runs), and the single protected oya-ci-required context is produced by the fan-in job only."
    verified_by: "oya-ci-required"
  - id: ADR-0716-D2
    description: "Add buck2-weekly-smoke.yml: a scheduled, non-blocking buck2 build //... honesty check. Retire the cache-integrity canary workflows and the build-cache-policy gate crate; specs/cache-warm-license.json stays as the declarative warm-reads kill-switch at warm_reads_licensed=false."
    exit_criteria: "The canary schedule, canary executor, and docs-graph-drift adapter are deleted; the weekly smoke builds the full graph from a clean checkout with the digest-pinned installer; a red weekly run does not affect any PR, and no gate asserts the existence of the retired workflows."
    verified_by: "buck2-weekly-smoke workflow run + oya-ci-required"
  - id: ADR-0716-D3
    description: "Amend the automation-language-policy inline-shell ratchet from a one-way shrink-only ceiling to shrink-only PLUS a reviewed replacement window (schema_version bump + reason + ADR), and give the gate crate a Cargo manifest so the cargo merge path enforces it."
    exit_criteria: "The gate crate is a cargo workspace member; cargo test --workspace executes the live-corpus ratchet tests; the replacement window is consumed by the ceiling validator and a window without reason+ADR fails closed."
    verified_by: "oya-ci-required"
  - id: ADR-0716-D4
    description: "Reduce PR paperwork: slim the PR template to issue/summary/verification/reviewer-verdict, and remove the post-merge product-completion packet requirement from the operating contract."
    exit_criteria: "The template has no required SLSA/SBOM/audit-emission/traceability fields; AGENTS.md and CLAUDE.md no longer require a post-merge packet; the local pr-traceability validator and its dev-cli callers are retired with this change."
    verified_by: "oya-ci-required"
---

# ADR-0716: Cargo is the CI merge path; buck2 is local hermeticity plus a weekly smoke

## Status

**Accepted** (founder directive 2026-08-14). Supersedes ADR-0560's canary trust anchor and
amends the CI-execution clauses of ADR-0700, the workspace-coverage mechanism of ADR-0554,
and the cache-warm doctrine of ADR-0556.

## Context

The required CI grew to 1,262 workflow lines and 12 jobs: a producer-regen barrier (~294 s),
a gate matrix, four bespoke gate jobs, a buck2 lane, a 377-line affected-set job with
merge-base baselines, two live-postgres lanes, and a fan-in. Seven jobs each installed buck2
separately. The buck2 remote cache (NativeLink CAS, ADR-0556/ADR-0560) was never deployed:
`warm_reads_licensed` is false and the cache-integrity canary workflow fails daily by design
while no CAS endpoint exists. Every buck2 lane therefore cold-builds. The affected-set
machinery exists only to make cold buck2 affordable; the canary exists only to license a
cache that does not exist; the PR template demands SLSA/SBOM/audit-emission fields that CI
no longer checks. None of this adds a verdict; it adds wall clock and paperwork.

Cargo needs none of it: the workspace already globs every crate, the gate fleet is ordinary
workspace members, and swatinem/rust-cache warms the build across runs. The gate crates'
only external inputs are the generated faces, produced by one materialize invocation.

The prior claim that buck2 is merge-grade while cargo is "supplementary local feedback only"
is inverted by this ADR. (The rationale is NOT "Meta does not run buck2 on GitHub Actions" —
Meta's own buck2 repository does. The rationale is that THIS repository has no live buck2
remote cache, so buck2 in CI means full rebuilds per lane, while cargo has turnkey caching.)

## Decision

1. **The Cargo workspace graph is the CI merge path.** The required workflow runs lint, test,
   and live-postgres, fanned in to the single protected `oya-ci-required` context. No
   Windows/macOS job: production is Linux VMs (amd64 per-PR; arm64 is a nightly and
   release-train gate, not a presubmit). No buck2 build/test verdict step (the face
   materializer keeps buck2 as an internal helper via the digest-pinned installer), no
   producer artifact handoff, no affected-set baselines, no daily-red canary.
2. **buck2 remains a local hermeticity tool**, kept honest by a weekly non-blocking
   `buck2 build //...` smoke. A red smoke means BUCK/reindeer wiring rotted and must be
   fixed or buck2 retired — it never blocks a PR.
3. **The canary and its trust chain are retired.** The canary workflows and the
   build-cache-policy gate crate are deleted; `specs/cache-warm-license.json` keeps
   `warm_reads_licensed: false` as the declarative kill-switch if a CAS is ever stood up.
4. **CI names are self-explanatory and debranded.** Job and step names describe the work
   (`lint`, `test`, `live-postgres-adapters`, `Fan-in verdict`) and
   reference no ADR or PR numbers. The protected context name `oya-ci-required` is branch
   protection infrastructure, not branding; the dual-emit `merge-admission-required` job
   remains the forever-name mirror, and the protection flip stays founder-only.
5. **The inline-shell ratchet gains a reviewed replacement window.** automation-language-policy
   stays shrink-only for ordinary PRs, but a deliberate CI redesign may replace the baseline
   wholesale by declaring a `replacement_window` (strictly higher schema_version + non-empty
   reason + ADR). The PR review is the admission control. The gate crate gains a Cargo
   manifest and workspace membership so the cargo merge path enforces it.
6. **PR paperwork shrinks.** The PR template reduces to issue, summary, verification, and
   reviewer verdict. The post-merge product-completion packet requirement is removed.

## Consequences

- **Positive:** one CI file, wall clock dominated by one cached cargo build, no
  quota-coupled artifact handoff, no daily-red noise, self-explanatory checks.
- **Negative:** cargo-green no longer implies buck2-green; the weekly smoke is the only
  guard against BUCK-graph rot. Losing the affected-set ratchet means pre-existing build
  debt on dev blocks PRs until fixed (accepted: the debt is retired by the same cleanup wave).
- **Risks:** gate crates that assumed buck2 test semantics (cwd, env) may need porting when
  first run under cargo; the rust-cache tag pin (v2) is a mutable tag, documented debt.

## Rules carry why

- **achieves:** fast, cacheable, self-explanatory CI; one honest merge path.
- **origin:** 12-job cold-build workflow, serial producer barrier, daily-red canary theater,
  PR-name noise pointing authors at unrelated gates.
- **rule:** cargo is the merge path; buck2 is local + weekly smoke; CI job/step names are
  self-explanatory, debranded, and reference no ADR/PR numbers; ratchet replacements ride a
  reviewed replacement window.
- **ensure:** this ADR + the workflow + automation-language-policy enforcement + reviewer
  lens on new workflow steps.
- **overturn_when:** a live buck2 remote cache is deployed and measured to beat cargo's
  wall clock on this fleet, with a recorded measurement and an ADR that re-adopts it.

## Amendment 2026-08-21 — Linux dual-arch nextest; no product cargo/buck2 on the PR

A hyperscaler tests the OS/arch it ships, in parallel, and builds release
artifacts on the CD train.

- **OS:** Linux VMs only. There is no consumer Windows, macOS, or Android
  binary. The previous "soft" `cross-platform-smoke` matrix is deleted.
- **Arch (D88 restored):** `oya-ci-required` runs the workspace nextest
  natively on amd64 (`ubuntu-24.04`) AND arm64 (`ubuntu-24.04-arm`) in
  parallel. Wall clock is `max` of the two, not the sum. D88-amend (arm64
  nightly only, to chase a five-minute budget the amd64 test job already
  misses) is overruled. rustfmt, advisory clippy, and live-postgres stay
  single-arch — they are not the native compile of the product graph.
- **Compile proof on the PR is `cargo nextest`, not `cargo build`.** A
  background `cargo build --workspace --tests` next to the materializer
  contended for Cargo's target lock and did not overlap. Product
  `cargo build --release` and image builds belong on the weekly train
  (D63). The remaining `cargo build -p ci-artifact-inventory-registry
  --bin …` is a test-fixture binary that gate tests exec; nextest does
  not build unrelated bins.
- **`buck2 build //...` stays off the merge path** (weekly honesty job,
  then the CD train). The face materializer still shells out to `buck2
  build` of a handful of face-tool bins — that is an ADR-0716 leak to
  cut in a follow-up by driving those bins through cargo. It is not a
  product graph build.
