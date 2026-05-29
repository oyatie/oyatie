---
id: ADR-0408
status: Proposed
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
supersedes: [ADR-0358]
superseded_by: []
amends: []
related: [ADR-0392, ADR-0358, ADR-0359, ADR-0111, ADR-0181, ADR-0346, ADR-0349]
related_specs:
  - /specs/cloud-toolchain-target.json
  - /specs/ci-farm-substrate-canonical.json
  - /specs/masterplan.json
milestone: M-TOOLCHAIN
depends_on: [ADR-0392, ADR-0359]
door: two-way
numbering_note: "decisions.json next_adr is ADR-0377; this ADR is deliberately allocated ADR-0408 (not the next-sequential number) to resolve the founder-assigned forward-reference for the Buck2 CI/CD reversal of the masterplan P-TOOLCHAIN CI step (sibling of ADR-0392). The numbering gaps ADR-0377..ADR-0391 and ADR-0393..ADR-0407 are left open and are NOT claimed by this lane; the ADR index will record ADR-0408 as a non-contiguous allocation alongside the existing documented gaps."
affected_surfaces:
  crates: []
  microservices: []
  specs: [/specs/cloud-toolchain-target.json, /specs/ci-farm-substrate-canonical.json, /specs/masterplan.json]
---
# ADR-0408: Buck2-driven CI/CD — RBE + affected-targets + image builds (reverses ADR-0358 §2 Bazel CI engine)

## Status

Proposed — 2026-05-29. DRAFT for founder review; this overturns the CI build-engine half of a reasoned decision (ADR-0358 §2) and must NOT auto-merge.

## Date

2026-05-29

## Supersedes

ADR-0358 (§2 toolchain build-graph only — specifically the CI build-engine + affected-target selection that ADR-0358 §2 vested in Bazel: "remote build execution, and affected-target selection ... `bazel query`/`bazel test`"). ADR-0392 supersedes the build-graph; this ADR supersedes the CI engine that drives it. The rest of ADR-0358's roadmap is intact.

## Superseded-by

—

## Related

ADR-0392 (Buck2 canonical build graph — the build-engine this CI drives), ADR-0358 (the roadmap whose §2 CI engine this reverses), ADR-0359 (Jenkins-sole-CI — complementary, NOT superseded; Jenkins is the orchestrator, Buck2 is the engine it invokes), ADR-0111 (merge-queue projected state), ADR-0181 (cosign image promotion ladder), ADR-0346 (oya verify CI mirror), ADR-0349 (Jenkins + ArgoCD substrate).

## Owner

council-architecture (with founder as deciding authority — this is a doctrine reversal).

## Context

The founder decision of 2026-05-29 reverses the build + CI/CD toolchain from Bazel to Buck2 across both local build and CI/CD. ADR-0392 records the build-graph reversal; this ADR records the CI/CD reversal — what the CI orchestrator invokes, how presubmit selects affected targets, and how images are built.

ADR-0358 §2 vested CI affected-target selection and remote execution in Bazel (`bazel query rdeps(//..., <changed>)` for presubmit; `bazel test` via RBE). `specs/cloud-toolchain-target.json` (`build_graph.affected_targets`, `ci`) and `specs/masterplan.json` P-TOOLCHAIN step 5 ("Layer remote build cache (sccache→SeaweedFS) + Bazel rules_rust affected-target selection") encode the Bazel-driven CI. Those specs are SUPERSEDED INPUTS to this ADR and need a follow-up generated-artifact update — OUT OF SCOPE for this docs-only PR (cited as superseded, not rewritten).

This ADR is explicitly COMPLEMENTARY to ADR-0359 (Jenkins completely replaces GitHub Actions). ADR-0359 settled WHO orchestrates CI (Jenkins, sole, self-hostable, covering air-gap/on-prem/colo contexts). This ADR settles WHAT BUILD ENGINE that orchestrator drives (Buck2, not Bazel). Jenkins remains the orchestrator; Buck2 is the build engine Jenkins invokes. ADR-0359 is NOT superseded.

The bespoke `oya` governance overlay is unaffected at the engine level: Buck2 knows nothing about the oya governance lanes, the gate verdict-cache, or the multispectrum-review verdicts. Those survive as the governance OVERLAY layered on top of whatever build engine Jenkins drives. What Buck2's graph-exact `rdeps` selection SUBSUMES is the cargo-side `verify_affected` change-selection heuristic — the build/test affected-set is now computed from Buck2's precise dependency graph instead of a cargo-mirror approximation.

## Decision

1. **Jenkins invokes `buck2` against self-hosted NativeLink RBE.** Jenkins (the sole CI per ADR-0359) drives `buck2 build`/`buck2 test` against a self-hosted NativeLink RBE + content-addressed cache (per ADR-0392 §4). This reverses ADR-0358 §2's `bazel test`/Bazel-RBE CI engine; Jenkins-as-orchestrator (ADR-0359) is unchanged.

2. **Graph-exact affected-target presubmit via `buck2 cquery rdeps(..., <changed>)`.** Presubmit selects the affected build/test set from Buck2's precise configured-target dependency graph — `buck2 cquery "rdeps(//..., <changed-targets>)"` — rather than a full-workspace rebuild and rather than the cargo-side change heuristic. This SUBSUMES the cargo-mirror `verify_affected` selection that the `oya` verify engine used for build/test scoping. The graph-exact selection is the Google-TAP-style affected-target presubmit ADR-0358 wanted, now sourced from Buck2 instead of `bazel query`.

3. **The oya governance overlay survives unchanged on top of Buck2.** Buck2 selects the build/test affected set; the oya GOVERNANCE-lane selection (which governance gates a change triggers) and the gate VERDICT-CACHE remain the bespoke governance overlay that Buck2 has no knowledge of. `oya` continues as the thin governance/verify orchestrator (per ADR-0358 §2's surviving overlay framing and ADR-0346's CI mirror); only the build/test engine underneath it changes from Bazel to Buck2.

4. **Cache-backed Buck2 image builds shorten the commit→image→ArgoCD-sync→Rollout loop.** Container images are built through Buck2's content-addressed action cache so unchanged layers are reused across CI runs, shortening the commit → image → ArgoCD-sync → Argo Rollouts progressive-delivery loop. Image promotion remains the cosign-signed dev→staging→prod ladder of ADR-0181 (unchanged); the merge queue remains ADR-0111's speculative+batch projected-state model (unchanged); ArgoCD + Argo Rollouts remain the deploy substrate (ADR-0349).

5. **Honesty / non-claims.** Buck2-driven CI is 0% adopted — no Jenkins pipeline invokes `buck2`, no NativeLink RBE is deployed, no `cquery rdeps` presubmit is wired, no Buck2 image build runs. This is doctrine + target, not implementation. NO numeric CI-throughput, cache-hit-rate, affected-selection-precision, or loop-latency figure is asserted; every such claim and every parity claim is `blocked_until_required_evidence_is_green` per `hyperscaler-gates.json`.

## Consequences

Positive: presubmit affected-target selection becomes graph-exact (sourced from Buck2's dependency graph, not a cargo-mirror heuristic), which is more precise than the selection it subsumes; one self-hostable CI surface (Jenkins, ADR-0359) drives one Rust-native engine (Buck2, ADR-0392) against one self-hostable RBE (NativeLink) — all hyperscaler-lens-clean; cache-backed image builds shorten the delivery loop while keeping the cosign promotion ladder (ADR-0181) and merge queue (ADR-0111) intact. Negative/cost: Jenkins pipelines + JCasC must be rewired from the Bazel-CI plan to Buck2 (`cquery`/`build`/`test` stages); NativeLink RBE must be operated; the Reindeer buckify step (ADR-0392 §2) becomes a CI prerequisite stage; no CI-performance claim may be made until the migration lands green. Neutral: ADR-0359 (Jenkins-sole-CI), ADR-0349 (ArgoCD), ADR-0181 (image promotion), ADR-0111 (merge queue), and the oya governance overlay + verdict-cache are unchanged; the Bazel-CI specs are superseded inputs awaiting a separate generated-artifact update; this ADR is doctrine, not the migration execution.
