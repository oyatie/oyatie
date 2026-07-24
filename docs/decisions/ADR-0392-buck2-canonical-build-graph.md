---
id: ADR-0392
status: Accepted
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
supersedes: [ADR-0358]
superseded_by: []
amends: []
amended_by: [ADR-0522, ADR-0524]
related: [ADR-0358, ADR-0408, ADR-0359, ADR-0346, ADR-0111, ADR-0181, ADR-0357, ADR-0092]
related_specs:
  - /specs/cloud-toolchain-target.json
  - /specs/masterplan.json
  - /specs/root-hub-pointers.json
milestone: M-TOOLCHAIN
depends_on: []
door: two-way
numbering_note: "decisions.json next_adr is ADR-0377; this ADR is deliberately allocated ADR-0392 (not the next-sequential number) to resolve the founder-assigned forward-reference for the Buck2 build-graph reversal of the masterplan P-TOOLCHAIN build-graph step. The numbering gap ADR-0377..ADR-0391 is left open and is NOT claimed by this lane; the ADR index will record ADR-0392 as a non-contiguous allocation alongside the existing documented gaps. ADR-0408 (its CI/CD sibling) is allocated by the same forward-reference convention."
affected_surfaces:
  crates: []
  microservices: []
  specs: [/specs/cloud-toolchain-target.json, /specs/masterplan.json, /specs/root-hub-pointers.json]
---
# ADR-0392: Buck2 canonical build graph (reverses ADR-0358 §2 Bazel rules_rust)

## Status

**Accepted — 2026-06-08 (founder-ruled; ratified at the WAVE-1 convergence door; door: two-way).**
Originally Proposed 2026-05-29; ratified to Accepted as part of the WAVE-1 fabric convergence
(resolve-every-Proposed rule). This overturns a reasoned decision (ADR-0358 §2).

## Amendment (2026-06-08, WAVE-1 fabric convergence)

This ADR is **amended/extended in place** (no tombstone; git history preserves the pre-amendment body):

- **ADR-0522** generalizes "the build graph" into "one lifecycle graph driven by four runners"
  (build · CI · CD · dev-env) — the same buck2 target graph, with thin generated adapters per runner.
- **ADR-0524** adds the kernel-side buckification this ADR never scoped (ADR-0392 covered only the
  cloud build graph): the kernel port enters the one buck2 graph additive-first, retiring `build.sh`
  and the tracked `out/*.elf` carrier blobs.

ADR-0392 remains an UPSTREAM `depends_on` of the hermetic execution model (ADR-0525). The Buck2 +
Reindeer + NativeLink decision below is unchanged.

## Implementation note (2026-07-24; no planning or generator authority expansion)

The repository-owned, fail-closed semantic overlay for Reindeer output is implemented at
`ci/facade/dependency-automation/src/third_party_overlay.rs`, with pure regressions beside the
module, filesystem/live-face regressions in
`ci/facade/dependency-automation/tests/dependency_automation.rs`, and narrow ownership at
`ci/facade/dependency-automation/OWNERS`. The canonical regeneration wrapper invokes the existing
`//ci/facade/dependency-automation:oya-cloud-ci-dependency-automation-app-bin` Buck2 target. These
surfaces implement Decision 2 as a local generator bridge; they do not expand merge authority or
authorize manual edits to `third-party/BUCK`.

## Date

2026-05-29

## Supersedes

ADR-0358 (§2 toolchain build-graph only — "Toolchain overhaul = Bazel `rules_rust` build graph"). The rest of ADR-0358's roadmap (strangler-fig posture §1, define-production-100-first §3, masterplan planning authority §4) stays intact and is NOT reversed here.

## Superseded-by

—

## Related

ADR-0408 (Buck2-driven CI/CD — the sibling reversal of the CI engine), ADR-0358 (the roadmap whose §2 this reverses), ADR-0359 (Jenkins-sole-CI, unchanged), ADR-0346 (oya verify CI mirror), ADR-0111 (merge queue), ADR-0181 (image promotion), ADR-0357 (monorepo nesting — already names Meta/Buck2 monorepo practice), ADR-0092 (workspace dependency-seam policy).

## Owner

council-architecture (with founder as deciding authority — this is a doctrine reversal).

## Context

The founder decision of 2026-05-29 reverses the build + CI/CD toolchain from Bazel to Buck2, covering both the local build graph and CI/CD. This ADR records the build-graph half; ADR-0408 records the CI/CD half.

ADR-0358 §2 chose Bazel `rules_rust` over Buck2 with an explicit, reasoned objection (quoted verbatim from ADR-0358 §Context):

> "Bazel `rules_rust` is chosen over Buck2 because Buck2 requires Reindeer to vendor all Cargo deps (hostile to Git/code-review) and is less battle-tested in OSS, while `rules_rust` supports Cargo.toml-as-SSOT (`crate_universe`) with mature RBE."

`specs/cloud-toolchain-target.json`, `specs/masterplan.json` (P-TOOLCHAIN, ~line 5860-5960, including the explicit `"rejected": "Buck2 (Reindeer vendoring hostile to Git/code-review; less OSS-battle-tested)"`), and `specs/root-hub-pointers.json` all encode Bazel `rules_rust` + `crate_universe` + RBE + affected-targets as the canonical build graph. Those machine-readable specs are SUPERSEDED INPUTS to this ADR — they require a follow-up generated-artifact update, which is intentionally OUT OF SCOPE here (this is a docs-only decision PR; the specs are cited as superseded, not rewritten).

This ADR must honestly confront ADR-0358's objection rather than wave it away. The objection is factually correct: Buck2 has no native Cargo-graph reader, so Cargo dependencies must be "buckified" by [Reindeer](https://github.com/facebookincubator/reindeer) into Buck2 target definitions. We ACCEPT this tradeoff with open eyes (see Decision §2 and Consequences).

Research grounding (2026-05): Buck2 is Meta's open-source successor to Buck, the build system Meta runs across its monorepo at a scale far beyond most OSS Bazel deployments; Buck2's core is a Rust binary with a Starlark-configured, fully-hermetic, content-addressed action graph and constraint-based incrementality (it recomputes the exact minimal set of affected actions from a precise dependency graph). `buck2-prelude` ships first-party Rust rules. Reindeer (also Meta, the tool Meta itself uses to vendor third-party Rust into its Buck monorepo) reads `Cargo.toml` + `Cargo.lock` and GENERATES a checked-in `third-party/rust/BUCK` plus a vendored/fixups layout — i.e. the buckified third-party graph is a generated, pinned, code-reviewable artifact, not opaque vendoring. NativeLink is an open-source, self-hostable Remote Build Execution + CAS backend (Apache-2, Rust) that speaks the Bazel Remote Execution v2 API and is used in production with Buck2. Self-hostable NativeLink passes the hyperscaler-lens filter (active upstream + clean license + fully self-hostable + a hyperscaler-internal equivalent, with no managed-service dependency); a managed RBE SaaS would NOT pass.

## Decision

1. **Buck2 + `buck2-prelude` + Reindeer-buckified third-party is the canonical build graph.** Buck2 (the Rust binary) drives the build/test action DAG with content-addressed, graph-exact incrementality. `buck2-prelude` supplies the first-party Rust toolchain rules. This reverses ADR-0358 §2's "Bazel `rules_rust` build graph"; everything else in ADR-0358 stands.

2. **Reindeer buckifies `Cargo.lock` into a checked-in `third-party/rust/BUCK` — tradeoff ACCEPTED and justified.** ADR-0358's objection ("Buck2 requires Reindeer to vendor all Cargo deps, hostile to Git/code-review") is acknowledged as accurate, and we accept it deliberately. Justification: (a) Reindeer's output is a GENERATED, PINNED, CHECKED-IN, code-reviewable artifact — the same character as `Cargo.lock` itself or as Bazel `crate_universe`'s lock — not hand-maintained opaque vendoring; the buckification is re-runnable from `Cargo.toml`/`Cargo.lock` and diffable in review. (b) `Cargo.toml`/`Cargo.lock` REMAIN the human-edited dependency SSOT; Reindeer is a one-way generator from that SSOT (developers add deps in Cargo, then regenerate — they never hand-edit the BUCK output), which preserves the dependency-seam policy (ADR-0092). (c) In exchange we gain Buck2's graph-exact incrementality + correctness and Meta's production-monorepo pedigree at a scale Bazel-rules_rust has not matched in OSS. (d) The "less battle-tested in OSS" half of ADR-0358's objection is the weaker half: Buck2 is battle-tested at Meta's monorepo scale (larger than most OSS Bazel installs), and `rules_rust` first-party scaling was itself flagged as a watch-item in ADR-0358's own Consequences ("watch `rules_rs` for first-party scaling").

3. **`toolchains//:rust` is pinned to the workspace Rust + linker parity with `.cargo/config.toml`.** The Buck2 Rust toolchain target pins the same Rust channel as `rust-toolchain.toml` (currently 1.96.0) and reproduces the workspace's existing `.cargo/config.toml` rustflags / linker selection (mold + clang where configured) so Buck2 and `cargo` produce link-parity artifacts. Cargo stays usable for local dev; Buck2 is the canonical graph for hermetic/cache/affected-target builds.

4. **Self-hostable NativeLink RBE is the remote backend.** Buck2 targets a self-hosted NativeLink RBE + content-addressed cache (replacing ADR-0358's "bazel RBE" target and the interim sccache→SeaweedFS cache). NativeLink passes the hyperscaler-lens (self-hostable, Apache-2, active upstream, hyperscaler-internal equivalent); a managed RBE service is rejected for the same lens reason.

5. **Honesty / non-claims.** Buck2 is 0% adopted — there is no `BUCK` file, no `buck2-prelude` vendor, no Reindeer-generated `third-party/rust/BUCK`, no NativeLink deployment, and no migration executed by this ADR. This is doctrine + target, not implementation. NO numeric build-speedup, cache-hit-rate, or incrementality figure is asserted; any such claim is `blocked_until_required_evidence_is_green` per `hyperscaler-gates.json` and may only be made once the migration lands and the evidence is green.

## Consequences

Positive: the canonical build graph moves to a Rust-native engine (Buck2) consistent with the kernel+OS bespoke-Rust ambition and the Meta-monorepo production pedigree; graph-exact incrementality + content-addressed correctness; a self-hostable NativeLink RBE that passes the hyperscaler-lens with no managed-service dependency; `Cargo.toml`/`Cargo.lock` remain the human dependency SSOT (Reindeer is a generator off it). Negative/cost: we accept the Reindeer buckification step (an extra generated `third-party/rust/BUCK` that must be regenerated and reviewed when deps change) — the exact objection ADR-0358 raised; the migration across the first-party crates is a substantial program; Buck2 first-party Rust + NativeLink operational maturity must be proven before any parity claim. Neutral: ADR-0358's strangler-fig posture, define-production-100-first phase, and masterplan planning authority are unchanged; ADR-0359 (Jenkins-sole-CI) is complementary (Jenkins drives Buck2 — see ADR-0408); the machine-readable specs that encode Bazel are superseded inputs awaiting a separate generated-artifact update; this ADR is doctrine, not the migration execution.
