# Pipeline Optimization — sharper, cheaper, higher-confidence CI/CD

_Idea-refine one-pager. Horizon: production pipeline design (refines
`specs/cloud-toolchain-target.json`, ADR-0346/0349/0359/0111). Grounded in
direct observation while building + measuring the local CI farm (2026-05-25)._

## Problem Statement
How might we cut PR time-to-green and wasted compute — without weakening the
governance gates — given that the current `oya verify` over-builds massively and
re-runs work the lane already did?

## Evidence (observed this session)
- A diff of ~1342 files that is **mostly `evidence/`/`registry/`/`specs/` YAML**
  (near-zero Rust source change) flagged **551 "affected" crates**; `oya verify`
  then spent **7+ minutes** in the cargo/nextest test mirror. Affected-target
  selection is path-coarse and over-selects.
- `oya verify` **re-compiled the dev-cli test binaries a second time** — a full
  cargo pass layered on top of whatever the lane already built (double-cargo).
- Cross-agent sccache reuse is real (**100% compile-hit over a 150-crate graph**,
  measured), but **dependency *downloads* dominated wall-time** — sccache caches
  compilation, not fetches.
- Per-build friction on stock agents: sccache **download every run**,
  `HOME`/`CARGO_HOME` setup, and the `drop:ALL` → lost-`DAC_OVERRIDE` footgun.

## Recommended Direction (ranked by leverage)
1. **Precise affected-targets.** Non-Rust diff ⇒ run *zero* cargo; crate diff ⇒
   only reverse-deps of changed crates. Interim: `cargo metadata` rdeps now;
   target: Bazel `rdeps(//…, changed)` (Google TAP). _Biggest latency+cost win,
   available before the Bazel migration._
2. **Kill the double-cargo.** Make `oya verify` the governance-only overlay
   (ADR-0346 `overlay`): consume the lane's nextest results, run only the
   bespoke gates — never re-run the build/tests.
3. **Trunk-warmed cache + cached registry.** Trunk/merge-queue builds populate
   the canonical sccache (RW); PR lanes read-through ⇒ warm from first run. Add a
   cached `CARGO_HOME` / pull-through cargo registry to kill download wall-time.
4. **Distributed test sharding.** `nextest --partition count:N` across agents ⇒
   test wall ≈ total/N; compose with (1) so only affected tests are sharded.
5. **Prebuilt cosign-pinned agent image** (rust+sccache+git+registry baked, fixed
   UID) — removes per-build setup + the footguns; satisfies the spec's
   cosign-required agent image.
6. **Merge-queue speculative execution** (ADR-0111) — batch A·main, B·main+A for
   always-green trunk + throughput.
7. **Incremental/content-hashed gates** — cache gate results keyed by changed
   inputs so gate cost stays O(changed) as the repo grows (honest-claims scans
   316k lines; data-class 2234 fields today).

## Key Assumptions to Validate
- [ ] oya's verify engine can express a precise affected-graph + consume external
      test results **without** waiting on full Bazel adoption (spike: `cargo
      metadata` rdeps + a results-ingest path).
- [ ] Trunk builds can own RW cache writes while PR lanes read-through safely
      (content-addressed entries make cross-lane sharing safe — already asserted
      in `ci-farm-substrate-canonical.json`).
- [ ] ADR-0111 merge-queue projected state is implemented enough to gate on.

## minimum-first-slice Scope (highest leverage, pre-Bazel)
- (1) + (2): a `cargo metadata`-driven affected set that skips cargo entirely for
  docs/evidence/spec-only diffs and runs only rdeps for crate diffs, plus oya
  verify consuming the lane's test results instead of re-running them. This alone
  targets the 7-min → seconds-for-non-Rust-PRs win observed here.

## Not Doing (and why)
- **Gate these wins on the full Bazel migration** — (1)(2)(3) have `cargo`-native
  interim forms; don't block them on the big migration.
- **Replace nextest / rewrite gates** — the tools are fine; the problems are
  *scope precision* and *redundancy*.
- **Tune GitHub Actions** — being retired for Jenkins (ADR-0359).

## Open Questions
- Does the affected-scope over-selection come from Cargo.toml/catalog churn
  (workspace-member edits) rather than source edits? If so, (1) must treat
  manifest-only changes carefully (a Cargo.toml bump *can* be a real rebuild
  trigger, but a catalog/evidence YAML is not).
- Where do per-lane test results get published for oya to consume (JUnit XML in
  the cache? a results bucket?) — defines the (2) ingest contract.
