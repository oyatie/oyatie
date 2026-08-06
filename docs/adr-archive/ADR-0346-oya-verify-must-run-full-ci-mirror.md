---
id: ADR-0346
title: oya verify --ci-required MUST locally mirror the full CI matrix (cargo fmt + cargo check + cargo clippy + cargo nextest + oya gate run-all + oya doc adr-index + oya lint adr-shape) and block on exit-0 of EACH step before returning success
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-platform
  - axis-dev-cli
  - ops-sre-reliability
owners:
  - council-architecture
  - ops-platform
  - axis-dev-cli
  - ops-sre-reliability
supersedes: []
superseded_by: [ADR-700]
amends:
  - ADR-0212-buildability-doctrine.md (the buildability doctrine asserted that every µservice + every workspace crate MUST compile + pass tests + pass clippy + pass fmt before landing; this ADR makes the LOCAL pre-push entry point — `oya verify --ci-required` — the canonical mirror of those CI lanes so a developer or agent can prove buildability locally without trusting CI to discover it after push. The doctrine's "must build" assertion is preserved verbatim; the new clause is that the local verifier MUST exercise every gate CI exercises and MUST block on exit-0 of EACH step, not delegate exclusively to a subset) Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.
  - ADR-0221-agentic-development-pipeline-hardening.md (the agentic-development-pipeline-hardening ADR established that "hooks are guidance, CI gates enforce" — the human / agent developer surface uses hooks for ergonomics but the corpus is gated by CI; this ADR maintains that posture while removing the silent-regression failure mode where the local verifier diverged from the CI gate matrix and let agents push PRs that failed CI 7 different ways in a single cycle. The hook-vs-gate doctrine is unchanged; `oya verify --ci-required` is the local rehearsal of the CI gates, not a replacement) Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md (the substance-bar doctrine asserted that CI enforcement is the canonical surface; this ADR clarifies that the local mirror `oya verify --ci-required` must reach the same bar for the gate categories CI gates enforce so an agent cannot claim "verified locally" while having only run `cargo check` lib-only. Substance-bar discipline is now applied to the local verifier itself: skip-flags are explicit, default is full-mirror, and exit-code semantics distinguish pass / fail / invalid-argument) Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.
  - ADR-0324-anti-script-anti-template-doctrine.md (the anti-script doctrine refuses ad-hoc shell wrappers that drift from the canonical entrypoint; this ADR refuses `scripts/local-ci.sh`, per-cargo-command shell aliases, and "fix locally as developers ad-hoc" as alternatives. The canonical entrypoint is `oya verify` per ADR-0212 / ADR-0221; this ADR makes that entrypoint complete rather than shipping a parallel shell wrapper that would itself become a drift surface)
related_adrs:
  - ADR-0083-test-budget-tiers.md
  - ADR-0105-thirteen-layer-enum.md
  - ADR-0106-naming-justification.md
  - ADR-0107-naming-bnf-v4.md
  - ADR-0108-sunset-lifecycle-automation.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0218-opentofu-not-terraform.md
  - ADR-0221-agentic-development-pipeline-hardening.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-anti-template-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/decision-principles.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_pre_push_full_ci_mirror_2026_05_21
  - feedback_codex_dispatch_canonical_2026_05_21
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
  - feedback_no_silent_regression
  - feedback_repeat_mistake_prevention
  - feedback_clean_architecture_requirements
  - feedback_quality_performance_scalability_bar
  - feedback_automate_everything
  - feedback_oya_git_canonical_2026_05_18
  - feedback_deprecate_external_agent_coord_tooling
companion_docs:
  - tools/hooks/_canonical-primitives.md
  - crates/oya-dev-cli/src/commands/verify.rs
  - scripts/hooks/pre-push.sh
  - .github/workflows/pr-tests.yml
  - docs/standards/dependency-policy.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_pre_push_full_ci_mirror_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-verify-implementation-lands
enforced_by:
  - oya-governance-oya-verify-ci-mirror-coverage (new lane; refuses corpus changes to `crates/oya-dev-cli/src/commands/verify.rs` that do not invoke cargo fmt + cargo check + cargo clippy + cargo nextest + oya gate run-all by static analysis; promoted to BLOCKER 14 days post Wave 15-ZA implementation lands)
  - oya-governance-oya-verify-ci-step-exit-semantics (new lane; refuses verify.rs source changes that swallow non-zero exit codes from any of the five mandatory mirror steps; refuses changes that conflate fmt-fail with check-fail in the exit code emitted to the caller)
  - oya-governance-oya-verify-skip-flag-allowlist (new lane; refuses verify.rs changes that add a skip flag outside the closed allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}` per D-8; new skip flags require an ADR amendment per `feedback_no_silent_regression`)
  - oya-governance-oya-submit-calls-verify (new lane; refuses changes to `oya submit` that bypass `oya verify --ci-required` per D-10 — preserves the existing call chain, refuses regressions)
  - oya-governance-oya-verify-exit-code-contract (new lane; refuses verify.rs changes that violate the closed exit-code enum `{0 = ALL passed, 1 = at least one failed, 2 = invalid arguments}` per D-11)
purpose: >
  Establish that `./bin/oya verify --ci-required` is the canonical local
  pre-push verifier and MUST locally mirror the full CI matrix —
  invoking `cargo fmt --all --check`, `cargo check --workspace
  --all-targets --keep-going`, `cargo clippy --workspace --all-targets
  --keep-going -- -D warnings`, `cargo nextest run --workspace
  --no-fail-fast` (or `cargo test --workspace` if nextest is absent),
  `oya gate run-all --ci-required`, `oya doc adr-index --write`
  (advisory; warn-only), and `oya lint adr-shape` against new ADRs in
  the current commit range — and MUST block on exit-0 of EACH step
  before returning success to the caller. Flag-controlled skipping via
  the closed allowlist `{--skip-fmt, --skip-clippy, --skip-nextest,
  --skip-gates}` supports incremental development; default invocation
  runs every step. Exit-code contract is closed: 0 = ALL passed; 1 =
  at least one failed; 2 = invalid arguments. `oya submit` MUST call
  `oya verify --ci-required` before push (current behavior preserved
  + enhanced by this ADR). The doctrine is binding because PR #177
  on 2026-05-21 surfaced 7 CI failures (cargo-fmt + cargo-clippy +
  cargo-nextest + oya-vcs-admission + oya-governance-dependency-seam
  + oya-governance-fitness-aspirational-enforcement +
  oya-governance-fitness-honest-claims) that the current verify
  implementation — which dispatches only to `oya gate run-all` and
  does NOT call cargo fmt/check/clippy/nextest directly — missed
  entirely. Out of scope: actual Rust code changes to
  `crates/oya-dev-cli/src/commands/verify.rs`; that implementation is
  sequenced as Wave 15-ZA-oya-verify-full-ci-mirror in
  `/specs/master-plan-sequencing.json`. This ADR authors doctrine
  only; the source-level extension is a separate PR landing under
  Wave 15-ZA after this ADR is Accepted.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0346: oya verify --ci-required MUST locally mirror the full CI matrix and block on exit-0 of EACH step before returning success

## Status

Proposed on 2026-05-21.

This ADR is the canonical local-verifier-completeness decision binding `./bin/oya verify --ci-required` to be a full mirror of the CI gate matrix that runs against every PR opened against `dev`. The current implementation (`crates/oya-dev-cli/src/commands/verify.rs` as of 2026-05-21) delegates to `oya gate run-all` and forwards arguments verbatim — but `oya gate run-all` does NOT invoke `cargo fmt --check`, does NOT invoke `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, and does NOT invoke `cargo nextest run --workspace`. The CI workflow at `.github/workflows/pr-tests.yml` invokes all three independently as separate required jobs. The gap between the local verifier and the CI matrix is the named pressure resolved by this ADR. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

The ADR is binding because the gap produced a measurable failure on 2026-05-21: PR #177 was pushed after a local `cargo check --workspace` returned exit-0; CI subsequently surfaced **7 distinct failures** in a single cycle — cargo-fmt, cargo-clippy, cargo-nextest, oya-vcs-admission, oya-governance-dependency-seam, oya-governance-fitness-aspirational-enforcement, oya-governance-fitness-honest-claims — none of which the local pre-push check exercised. The named-pressure analysis in §A below traces each surfaced CI failure to a specific verifier step that this ADR mandates.

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0345 (OSS stewardship class) is the immediately-prior sibling decision; the six-candidate batch ADR-0340..0345 closes the realignment-substance arc; this ADR adds a verifier-completeness clause that protects every prior + subsequent ADR's `enforced_by` lane assertions from silent local-side bypass. Without the full-mirror clause, an agent could author a new lane in an ADR, declare `enforced_by`, and then land a PR locally green via `oya verify` while CI later surfaces a different result — exactly the failure mode `feedback_no_silent_regression` and `feedback_repeat_mistake_prevention` are designed to prevent.

It directly amends ADR-0212 (buildability doctrine) by tightening the LOCAL surface of the buildability claim: every workspace crate MUST compile + pass tests + pass clippy + pass fmt before landing; the local verifier MUST exercise each of those gates. It directly amends ADR-0221 (agentic-development-pipeline-hardening) by preserving the "hooks are guidance, CI gates enforce" posture and clarifying that the local mirror is a developer-side rehearsal of the CI gates, not a replacement gate. It directly amends ADR-0322 (substance-bar) by applying the substance-bar discipline to the verifier surface itself — skip flags are explicit, default is full-mirror, exit-code semantics are closed. It directly amends ADR-0324 (anti-script-anti-template) by refusing the parallel `scripts/local-ci.sh` wrapper and per-cargo-command shell aliases as legitimate alternatives; the canonical entrypoint per ADR-0212 / ADR-0221 is `oya verify`, and this ADR makes that entrypoint complete. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

Enforcement transitions from `advisory-until-verify-implementation-lands` to `BLOCKER` per the lane sequence in §E below: at landing of the Wave 15-ZA implementation PR (which extends `crates/oya-dev-cli/src/commands/verify.rs` to invoke the five mandatory mirror steps directly rather than delegating only to `oya gate run-all`), the five new lanes promote from REPORT-ONLY to BLOCKER 14 days after the Wave 15-ZA PR merges. The 14-day soak is shorter than the typical 30-day window (per ADR-0345 + ADR-0344 precedent) because the verifier-coverage lanes are static-analysis lanes on a single source file (`crates/oya-dev-cli/src/commands/verify.rs`); the corpus impact at promotion is bounded.

The decision does not change which gates CI runs. The decision does not retire `oya gate run-all` — the local verifier continues to call it as one of the five mandatory mirror steps. The decision does not introduce a new dependency to the corpus. The decision does not change the `pre-push` git hook surface beyond preserving the existing `scripts/hooks/pre-push.sh` invocation of `oya verify --ci-required`. The decision does not change the ADR-0221 hook-vs-gate doctrine: hooks remain guidance, CI gates remain the enforcement surface; the local verifier is the developer-side rehearsal. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

**Implementation queued as Wave 15-ZA.** Actual extension of `crates/oya-dev-cli/src/commands/verify.rs` is a separate PR (this ADR is doctrine-only). The Wave 15-ZA entry is added to `/specs/master-plan-sequencing.json` as part of this ADR's required-artifact contract; the implementation PR cites this ADR in its commit-message provenance.

## Date

2026-05-21.

## Context

### A.1 Named pressure: PR #177 surfaced 7 CI failures the local verifier missed

On 2026-05-21, PR #177 was pushed to `dev` after the author ran `cargo check --workspace` locally and observed exit-0. CI subsequently surfaced **7 distinct failures** in a single cycle. Each failure traces to a gate that the local pre-push check did not exercise:

1. **cargo-fmt** — `cargo fmt --all -- --check` exit non-zero. The author had run `cargo check` lib-only; `cargo check` does not validate formatting. The CI workflow at `.github/workflows/pr-tests.yml` job `cargo-fmt` (lines 38-50) is independent of `cargo-check`.
2. **cargo-clippy** — `cargo clippy --workspace --all-targets --keep-going -- -D warnings` exit non-zero. The author did not run clippy. The CI workflow job `cargo-clippy` (lines 52-79) escalates warnings to errors via `-D warnings`; the local environment never observed those warnings as failures because no local invocation used `-D warnings`.
3. **cargo-nextest** — `cargo nextest run --workspace --no-fail-fast` exit non-zero. The author did not run tests. `cargo check` validates compilation but not runtime assertions; the test-side API drift introduced by the PR was invisible to `cargo check`.
4. **oya-vcs-admission** — the VCS admission gate refused content; the local pre-push hook did not run the gate because the local verifier path stopped at `oya gate run-all` without invoking the per-lane admission gate.
5. **oya-governance-dependency-seam** — the dependency-seam governance gate refused a Cargo.toml topology change; the local verifier did not exercise the governance lane.
6. **oya-governance-fitness-aspirational-enforcement** — the aspirational-enforcement fitness lane refused content claiming enforcement without lane evidence; the local verifier did not run the fitness lane locally.
7. **oya-governance-fitness-honest-claims** — the honest-claims fitness lane refused a doc claiming enforcement without an enforced_by reference; the local verifier did not exercise the lane.

The seven failures arrived in **one** CI cycle (post the 2026-05-15 surface-all-failures posture per `pr-tests.yml` lines 7-19), giving the author a single feedback loop to fix everything. But the time cost was non-trivial: the author had to wait for CI to discover what the local verifier should have surfaced before the push. Per `feedback_pre_push_full_ci_mirror_2026_05_21.md`, the rule learned was "before ANY push that lands corpus-wide changes (≥10 files modified or any merge resolution), run the FULL CI mirror locally + block on exit-0 of each step before pushing."

The named pressure this ADR resolves is the gap between the current `oya verify` implementation (which dispatches only to `oya gate run-all`) and the CI matrix (which independently runs cargo-fmt + cargo-clippy + cargo-check + cargo-nextest + oya-vcs-admission + multiple governance lanes). The gap is the verifier-completeness gap; this ADR closes it by mandating the full mirror.

### A.2 Named pressure: the current verify implementation is a documented thin alias

The current source at `crates/oya-dev-cli/src/commands/verify.rs` (read in full as part of this ADR's authoring) carries the comment: *"`oya verify` is intentionally a thin alias for `oya gate run-all`. Any positional/flag passthrough is forwarded verbatim so the local pre-push hook can call `oya verify --include-deferred` etc. without a parallel parser."*

The thin-alias posture was correct in the 2026-05-15 design window when the local verifier's job was to dispatch to the canonical gate aggregator. The posture became insufficient when the CI matrix at `.github/workflows/pr-tests.yml` was widened to run `cargo-fmt` + `cargo-clippy` + `cargo-check` + `cargo-nextest` as INDEPENDENT required jobs (per the 2026-05-15 surface-all-failures finding) without a corresponding extension of `oya verify`. The CI matrix moved; the local mirror did not.

The 2026-05-21 PR #177 incident is the exact evidence that the gap matters. The thin-alias comment in `verify.rs` documents intent; the CI failure documents consequence. This ADR closes the gap.

### A.3 Named pressure: agents trust the verifier as their pre-push contract

Per `feedback_repeat_mistake_prevention`, agentic developers and human developers both treat `oya verify --ci-required` as the canonical pre-push contract. When the verifier exits 0, the implicit claim is "this branch is ready to push and will pass CI." When the verifier exits 0 and CI subsequently fails, the contract is broken — and the broken-contract failure mode is exactly the silent-regression class `feedback_no_silent_regression` is designed to prevent.

The verifier's contract MUST be "exit-0 means CI will pass." The current implementation cannot honor that contract because it does not run the gates CI runs. This ADR makes the contract honorable by requiring the verifier to run every gate CI runs (or to fail with a non-zero exit if any gate fails).

For agentic developers, the gap is especially dangerous: a Claude or codex subagent may report "local verification PASS" based on `oya verify` exit-0, push the branch, and discover the CI failures after the dispatcher has moved on. The dispatcher's confidence in subagent self-reports per `feedback_verify_deliverables_not_just_line_count_2026_05_20` is undermined if the local verifier itself is incomplete. The full-mirror clause restores the trust.

### A.4 Named pressure: ADR-0212 buildability doctrine carries an implicit "verifiable locally" clause

ADR-0212 (buildability doctrine) asserts that every µservice + every workspace crate MUST compile + pass tests + pass clippy + pass fmt before landing. The CI lanes that enforce buildability — cargo-fmt + cargo-clippy + cargo-check + cargo-nextest — are the canonical enforcement surface. The implicit clause not written into ADR-0212 but operationally required is: *the local developer + local agent MUST be able to verify buildability before pushing.* Without a local mirror, the buildability assertion is testable only post-push, which violates the "verify deliverables not just line count" discipline per `feedback_verify_deliverables_not_just_line_count_2026_05_20`.

This ADR makes the implicit clause explicit: `oya verify --ci-required` is the canonical local entrypoint for the buildability verification; it MUST exercise every gate the buildability doctrine enforces in CI. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

### A.5 Named pressure: ADR-0221 hook-vs-gate doctrine — local rehearsal is part of the discipline

ADR-0221 (agentic-development-pipeline-hardening) established the doctrine "hooks are guidance, CI gates enforce." The doctrine is preserved verbatim by this ADR. The clarification this ADR adds is that the LOCAL verifier `oya verify --ci-required` is the canonical rehearsal of the CI gates — not a replacement gate, not a substitute for CI, but a developer-side rehearsal so the developer can fix issues before pushing rather than after the CI cycle reveals them. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

The rehearsal posture matters because the CI cycle has a measurable cost (~5-20 minutes per cycle for the workspace) and because the surface-all-failures CI design (per `.github/workflows/pr-tests.yml` lines 7-19) means a single broken push exposes everything at once — but only after the cycle completes. A complete local rehearsal collapses the feedback loop from ~5-20 minutes to ~1-5 minutes locally, with no loss of fidelity if the rehearsal matches the CI matrix.

### A.6 Named pressure: hyperscaler precedent — local CI mirrors are the norm

Hyperscaler engineering teams ship local CI mirrors as part of their developer tooling. Google's `presubmit` (Piper) runs the same build + test + lint suite locally that the post-submit CI runs. Microsoft's `azure-pipelines-task-lib` ships a local-runner mode that mirrors the cloud-side pipeline. Meta's `arc presubmit` runs the build farm's pre-merge gates against a developer's local diff. Amazon's internal `brazil-build` ships a local mode that mirrors the CodePipeline build matrix. Apple's `xcodebuild test` (via `xcode-cloud`) runs locally before the cloud-side CI confirms. Netflix's `nebula-test` mirrors the post-merge CI on a developer's laptop.

Across hyperscalers, the local mirror is the canonical pre-push tool. Oyatie's posture per `feedback_quality_performance_scalability_bar` (industry-leader quality bar) requires matching this pattern; the full-mirror clause in this ADR brings `oya verify` into alignment.

### A.7 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_pre_push_full_ci_mirror_2026_05_21.md` — "before ANY push that lands corpus-wide changes, run the FULL CI mirror locally + block on exit-0 of each step before pushing."
- Anchor 2: ADR-0212 (buildability doctrine). The buildability claim's local-verifiable clause is made explicit by this ADR.
- Anchor 3: ADR-0221 (agentic-development-pipeline-hardening). Hook-vs-gate doctrine preserved; local rehearsal clarified.
- Anchor 4: ADR-0322 (substance-bar). Substance-bar applies to the verifier surface itself; skip-flags are explicit, default is full-mirror, exit-code semantics are closed.
- Anchor 5: ADR-0324 (anti-script-anti-template). Parallel shell wrappers refused; `oya verify` is the canonical entrypoint.
- Anchor 6: ADR-0345 (OSS stewardship class). The verifier's coverage extension is part of the realignment-substance arc; the verifier coverage gate is itself an OSS-stewardship lane (council-architecture-owned, Maintainer class for the `oya-dev-cli` crate).
- Anchor 7: `feedback_pre_push_full_ci_mirror_2026_05_21` — the canonical learned-rule memory.
- Anchor 8: `feedback_no_silent_regression` — exit-0 from the local verifier with subsequent CI failure is a silent regression.
- Anchor 9: `feedback_repeat_mistake_prevention` — second-occurrence prevention; PR #177 is the named occurrence prompting this ADR.
- Anchor 10: `feedback_verify_deliverables_not_just_line_count_2026_05_20` — verify-the-deliverable discipline applies to the verifier itself.
- Anchor 11: `feedback_automate_everything` — mechanical pre-push verification MUST be scripted, not run ad-hoc.
- Anchor 12: `feedback_oya_git_canonical_2026_05_18` — `oya git` + `oya submit` chain remains canonical; `oya verify` is the gate `oya submit` invokes.

### A.8 What this ADR does not assert

- **A.8.1** Does not change the CI matrix at `.github/workflows/pr-tests.yml`. CI jobs continue verbatim; the local verifier mirrors them.
- **A.8.2** Does not retire `oya gate run-all`. The local verifier continues to call it as one of the five mandatory mirror steps (per D-5).
- **A.8.3** Does not introduce a new shell wrapper at `scripts/local-ci.sh`. The canonical entrypoint per ADR-0212 + ADR-0221 + ADR-0324 is `oya verify`; this ADR makes that entrypoint complete rather than shipping a parallel wrapper.
- **A.8.4** Does not change the `scripts/hooks/pre-push.sh` git-hook surface beyond preserving the existing `oya verify --ci-required` invocation. The hook continues to call the verifier; the verifier becomes complete.
- **A.8.5** Does not change the ADR-0221 hook-vs-gate doctrine. Hooks remain guidance; CI gates remain the enforcement surface; the local verifier is the developer-side rehearsal of the CI gates.
- **A.8.6** Does not impose a closed maximum runtime budget for the verifier. The verifier runs as long as the slowest mirrored step (typically `cargo nextest` for the workspace). Developers may use skip flags (per D-8) to accelerate incremental development without breaking the default full-mirror.
- **A.8.7** Does not require the verifier to run against pre-commit state (working tree). The verifier runs against the committed state at the time of invocation. Pre-commit verification is out of scope; the pre-commit hook surface remains a separate concern.
- **A.8.8** Does not author the actual Rust code changes to `crates/oya-dev-cli/src/commands/verify.rs`. The implementation is sequenced as Wave 15-ZA-oya-verify-full-ci-mirror after this ADR is Accepted.
- **A.8.9** Does not change the `oya submit` command's contract beyond preserving the existing call to `oya verify --ci-required` (per D-10). The submit-calls-verify chain is enforced by the new `oya-governance-oya-submit-calls-verify` lane.
- **A.8.10** Does not introduce a new dependency to the corpus. `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo nextest` are already corpus-required tools per dependency-policy + the existing CI workflow.
- **A.8.11** Does not bypass the ADR-0263 audit-chain emission for verifier invocations. The verifier itself does not emit audit-chain rows (it is a local-developer tool, not a runtime µservice); this ADR does not change that posture.
- **A.8.12** Does not change Cedar evaluation surface per ADR-0243. The verifier is a local tool; Cedar gates apply at runtime, not at verifier-invocation time.

## Decision

### B.1 Decision statement

`./bin/oya verify --ci-required` is the canonical local pre-push verifier. The verifier MUST locally mirror the full CI matrix at `.github/workflows/pr-tests.yml` and MUST block on exit-0 of EACH step before returning success to the caller. The five mandatory mirror steps are:

1. **D-1:** `cargo fmt --all --check` — formatting validation.
2. **D-2:** `cargo check --workspace --all-targets --keep-going` — compilation validation including tests/benches/examples.
3. **D-3:** `cargo clippy --workspace --all-targets --keep-going -- -D warnings` — lint validation with warnings escalated to errors.
4. **D-4:** `cargo nextest run --workspace --no-fail-fast` — runtime test validation (or `cargo test --workspace` if nextest is absent).
5. **D-5:** `oya gate run-all --ci-required` — canonical gate aggregator (existing behavior preserved).

Two advisory steps run additionally (warn on non-zero; do not gate exit code):

6. **D-6:** `oya doc adr-index --write` — ADR index refresh (advisory).
7. **D-7:** `oya lint adr-shape` — ADR-shape lint against new ADRs in the current commit range (advisory unless new ADRs are present, in which case BLOCKER).

Flag-controlled skipping uses the closed allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}` for incremental development; default invocation runs every mandatory step. The exit-code contract is closed: `0` = ALL passed; `1` = at least one mandatory step failed; `2` = invalid arguments. `oya submit` MUST call `oya verify --ci-required` before push (current behavior preserved + enhanced by this ADR).

The doctrine is binding on the `crates/oya-dev-cli/src/commands/verify.rs` source file; the new `oya-governance-oya-verify-ci-mirror-coverage` lane (§E.1) validates by static analysis that the source invokes each of the five mandatory mirror commands.

Drives developer + agent confidence in the pre-push contract:

- **Local fidelity to CI.** Exit-0 from `oya verify --ci-required` means CI will pass for the gate categories mirrored.
- **Silent-regression containment.** Per `feedback_no_silent_regression`, the verifier-coverage drift is contained by static analysis of the verifier source file.
- **Hyperscaler-grade rigor.** Matches the local-mirror posture of Google `presubmit`, Microsoft `azure-pipelines-task-lib`, Meta `arc presubmit`, Amazon `brazil-build`, Apple `xcodebuild test`, Netflix `nebula-test`.

### B.2 Numbered decision clauses

B2.001. `./bin/oya verify --ci-required` is the canonical local pre-push verifier. The verifier MUST locally mirror the full CI matrix at `.github/workflows/pr-tests.yml`.

B2.002. The verifier MUST block on exit-0 of EACH mandatory mirror step (D-1..D-5) before returning success (exit code `0`) to the caller.

B2.003. The five mandatory mirror steps are: D-1 `cargo fmt --all --check`; D-2 `cargo check --workspace --all-targets --keep-going`; D-3 `cargo clippy --workspace --all-targets --keep-going -- -D warnings`; D-4 `cargo nextest run --workspace --no-fail-fast` (with fallback to `cargo test --workspace` when nextest is absent); D-5 `oya gate run-all --ci-required`.

B2.004. The two advisory steps are: D-6 `oya doc adr-index --write` (warn-only; non-zero does not gate the verifier exit code); D-7 `oya lint adr-shape` against new ADRs in the current commit range (advisory unless new ADRs are present, in which case the step is BLOCKER and contributes to the exit code).

B2.005. The verifier MUST invoke the mandatory steps in the order D-1 → D-2 → D-3 → D-4 → D-5. Advisory steps D-6 + D-7 run after the mandatory steps (do not block earlier steps).

B2.006. The verifier MUST NOT short-circuit after the first failed mandatory step. Each mandatory step MUST run regardless of prior-step outcome, so the user sees ALL failures in one invocation (mirroring the CI surface-all-failures posture per `.github/workflows/pr-tests.yml` lines 7-19). The verifier accumulates the per-step exit codes and emits a non-zero overall exit code if any mandatory step failed.

B2.007. Skip flags are limited to the closed allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}`. Each flag suppresses exactly one mandatory mirror step. New skip flags require an ADR amendment per `feedback_no_silent_regression`.

B2.008. The skip flags are intended for incremental development (e.g., rapid clippy iteration with `--skip-nextest`). They MUST NOT be used in CI invocations or in pre-push automation. The verifier MAY emit a warning when a skip flag is used and the calling environment looks like CI (heuristic: `CI=true` environment variable).

B2.009. The exit-code contract is closed: `0` = ALL passed (or only advisory steps failed); `1` = at least one mandatory step failed; `2` = invalid arguments (unknown flag, unparseable input, missing required tool). No other exit codes are defined; the verifier MUST NOT emit a different exit code.

B2.010. `oya submit` MUST call `oya verify --ci-required` before push. The call is non-bypassable from the `oya submit` source; the `oya-governance-oya-submit-calls-verify` lane (§E.4) enforces by static analysis on the submit source file. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

B2.011. The pre-push git hook at `scripts/hooks/pre-push.sh` continues to invoke `oya verify --ci-required` per the existing 2026-05-15 design; this ADR does not change the hook surface, only the verifier internals.

B2.012. The verifier output format MUST mirror the CI lane output for visual continuity. Per-step section headers MUST identify the mandatory step by D-N reference (e.g., "D-1: cargo fmt --all --check") so a developer comparing local output to CI output can locate the failed step easily.

B2.013. The verifier MUST emit a final summary line of the form: `oya verify: [PASS|FAIL] (mandatory: N/5, advisory: M/2)`. The mandatory count reflects the number of D-1..D-5 steps that exited 0; the advisory count reflects D-6 + D-7.

B2.014. When `cargo nextest` is not installed on the local machine, the verifier MUST fall back to `cargo test --workspace --no-fail-fast` (or `cargo test --workspace` if `--no-fail-fast` is unavailable in the installed cargo version) and MUST emit a warning indicating the fallback. The fallback is functionally equivalent for the verifier's contract; nextest is preferred per CI for performance reasons.

B2.015. The verifier MUST refuse to run if the working tree is not at a committed state when invoked without an explicit `--allow-dirty` flag. (Reserved for future extension; the v1 implementation MAY treat this as a future-version field and accept the working-tree state verbatim.)

B2.016. The verifier MUST emit per-step timing information (elapsed seconds per step) at the end of each step's section. The timing is informational; developers can use it to identify slow steps when tuning skip-flag usage.

B2.017. The verifier MUST inherit the calling shell's environment variables (CARGO_TERM_COLOR, RUSTFLAGS, etc.) verbatim. The verifier MUST NOT override `CARGO_INCREMENTAL` (CI uses `0`; local developers may use `1`); the local-CI fidelity gap from this is bounded (incremental compilation does not affect compilation correctness, only build speed).

B2.018. The verifier MUST detect that the calling repo is at the workspace root before running cargo commands. If invoked from a subdirectory, the verifier MUST either (a) cd to the workspace root automatically and inform the caller, or (b) exit with `2` (invalid arguments) and an actionable error message.

B2.019. The verifier MUST NOT cache results across invocations. Each invocation runs every mandatory step (subject to skip flags). Caching is the cargo build cache's job; the verifier is stateless.

B2.020. The verifier MAY parallelize independent mandatory steps if implementation complexity is bounded; the v1 implementation MAY run sequentially. Parallelization is a future optimization, not a B.2 requirement.

B2.021. The verifier MUST refuse to be invoked recursively. If `oya verify` is invoked from within `oya verify` (detected via environment variable `OYA_VERIFY_RUNNING=1`), the inner invocation MUST exit with code `2` and an actionable error message. Recursion would produce unbounded runtime.

B2.022. The verifier MUST gracefully handle missing tools. If `cargo fmt` is not installed (e.g., `rustfmt` component absent), the D-1 step MUST exit non-zero with an actionable error message identifying the missing tool; the verifier MUST treat the missing-tool case as a step failure, not as a step skip.

B2.023. The verifier MUST log invocation metadata (timestamp, repo HEAD SHA, flags used, environment variables consulted) to `target/oya-verify-runs/<hlc-timestamp>.json` for post-hoc debugging. The log file is bounded in size (single JSON object per invocation) and is gitignored.

B2.024. The verifier MUST emit ANSI color codes when `stdout` is a tty (matching CI's `CARGO_TERM_COLOR=always` posture by default, and the developer's shell-controlled posture for interactive use).

B2.025. The advisory step D-7 `oya lint adr-shape` MUST detect new ADRs in the current commit range. The commit range is computed as `git diff --name-only origin/dev...HEAD -- 'docs/decisions/ADR-*.md'`. If new ADRs are detected, D-7 MUST run on them and MUST gate the exit code (BLOCKER for new ADRs); otherwise D-7 is informational and warn-only.

B2.026. The verifier MUST NOT bypass the `oya gate run-all --ci-required` step (D-5) under any skip flag. The `--skip-gates` flag suppresses D-5 specifically; documented intent is "suppress the gate aggregator while iterating on a non-gate concern." Use is informational only; `oya submit` does not pass `--skip-gates`.

B2.027. The verifier's source is `crates/oya-dev-cli/src/commands/verify.rs`. The `oya-governance-oya-verify-ci-mirror-coverage` lane (§E.1) validates by static analysis (Rust source grep or AST inspection per `mcp__plugin_oh-my-claudecode_t__ast_grep_search` capability) that the source invokes each of the five mandatory mirror commands.

B2.028. The verifier MAY consult `/specs/oya-verify-ci-mirror.json` for the canonical list of mandatory mirror commands. The spec file is authored under this ADR's required-artifact contract and is the source-of-truth for the lane's static-analysis target. The spec file content is the closed enum `["cargo fmt --all --check", "cargo check --workspace --all-targets --keep-going", "cargo clippy --workspace --all-targets --keep-going -- -D warnings", "cargo nextest run --workspace --no-fail-fast", "oya gate run-all --ci-required"]`.

B2.029. The verifier exits `2` if any mandatory mirror step's command-string in the source disagrees with the spec file's enum at runtime. This protects against the silent-drift failure mode where the source file gets updated but the spec file does not (or vice versa).

B2.030. The new five lanes (§E.1..§E.5) are: oya-governance-oya-verify-ci-mirror-coverage (E.1); oya-governance-oya-verify-ci-step-exit-semantics (E.2); oya-governance-oya-verify-skip-flag-allowlist (E.3); oya-governance-oya-submit-calls-verify (E.4); oya-governance-oya-verify-exit-code-contract (E.5).

B2.031. The five new lanes are REPORT-ONLY at Acceptance + advisory-until-verify-implementation-lands. Promotion to BLOCKER occurs 14 days after the Wave 15-ZA implementation PR merges into `dev`.

B2.032. The Wave 15-ZA-oya-verify-full-ci-mirror sub-wave authors: (i) the extended `crates/oya-dev-cli/src/commands/verify.rs` invoking D-1..D-7 directly; (ii) the `/specs/oya-verify-ci-mirror.json` spec file; (iii) the integration test at `crates/oya-dev-cli/tests/verify_full_mirror.rs` that exercises the verifier end-to-end against a fixture branch; (iv) the per-lane static-analysis implementation for §E.1..§E.5; (v) a REMEDIATION-NOTES entry at `microservices/oya-dev-cli/remediation-notes/2026-05-21-oya-verify-full-ci-mirror.md` (if the µservice's manifest carries that path; the dev-cli is currently a crate, not a µservice — see implementation-PR context).

B2.033. The ADR is binding on the `oya-dev-cli` crate only at this layer. Per-µservice impact is bounded — every µservice's CI configuration is unchanged; only the local-developer pre-push tool changes.

B2.034. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. Review evidence at `evidence/debate/ADR-0346/` after this ADR lands in a review-track PR.

B2.035. The 14-day sunset window starts on Wave 15-ZA implementation PR merging into `dev`. The five new lanes promote from REPORT-ONLY to BLOCKER at day 14; the shorter-than-30-day window reflects the lanes' static-analysis-on-a-single-source-file scope.

B2.036. The ADR is final on Acceptance. No exception clause is provided for verifier source changes after the 14-day Wave-15-ZA-post-merge sunset window without lane-green evidence.

B2.037. Three Rejected Alternatives are recorded in §F below: (i) `scripts/local-ci.sh` shell-wrapper alternative (rejected — parallel entrypoint drift surface per ADR-0324); (ii) per-cargo-command shell aliases (rejected — no static-analysis surface, no CI mirror); (iii) "fix locally as developers ad-hoc" (rejected — PR #177 is the exact evidence that ad-hoc does not work).

B2.038. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

B2.039. The ADR's enforcement and sunset run in coordination with Wave 15-ZA-oya-verify-full-ci-mirror. The Wave 15-ZA entry is added to `/specs/master-plan-sequencing.json` as part of this ADR's required-artifact contract.

B2.040. The ADR explicitly preserves all prior `oya verify` callers verbatim. No caller-side change is required by this ADR; the verifier's exit-code contract is preserved (0 = ALL passed; 1 = at least one mandatory step failed; 2 = invalid arguments) and prior callers continue to interpret exit codes the same way they did pre-ADR.

### B.3 What this decision does not do

- This ADR does not change the CI matrix at `.github/workflows/pr-tests.yml`. CI jobs continue verbatim.
- This ADR does not retire `oya gate run-all`. The local verifier continues to call it as D-5.
- This ADR does not change the `oya submit` command's external behavior beyond enforcing the existing `oya verify --ci-required` call chain.
- This ADR does not change the pre-push git hook surface at `scripts/hooks/pre-push.sh`.
- This ADR does not change the ADR-0221 hook-vs-gate doctrine.
- This ADR does not introduce a new dependency to the corpus.
- This ADR does not author the actual Rust code changes to `crates/oya-dev-cli/src/commands/verify.rs`; that work is sequenced as Wave 15-ZA.

## Consequences

### C.1 Positive consequences

- **Local fidelity to CI.** Exit-0 from `oya verify --ci-required` means CI will pass for the gate categories mirrored. The pre-push contract is honorable, not aspirational.
- **Feedback-loop collapse.** Developers + agents who run the full mirror locally collapse the feedback loop from ~5-20 minutes (CI cycle) to ~1-5 minutes (local mirror) without loss of fidelity.
- **Silent-regression containment.** Per `feedback_no_silent_regression`, the verifier-coverage drift is contained by static analysis of the verifier source file. The `oya-governance-oya-verify-ci-mirror-coverage` lane refuses source changes that drop a mandatory mirror step.
- **Agent trust restoration.** Agentic developers who report "local verification PASS" can trust the verifier; the dispatcher's confidence in subagent self-reports per `feedback_verify_deliverables_not_just_line_count_2026_05_20` is restored.
- **Hyperscaler-grade rigor.** Matches the local-mirror posture of Google `presubmit`, Microsoft `azure-pipelines-task-lib`, Meta `arc presubmit`, Amazon `brazil-build`, Apple `xcodebuild test`, Netflix `nebula-test`.
- **Buildability doctrine completion.** ADR-0212's implicit "verifiable locally" clause is made explicit; the buildability claim is checkable at every push.
- **Substance-bar applied to tooling.** The verifier surface gains the same substance-bar discipline applied to documentation per ADR-0322: skip flags are explicit, default is full-mirror, exit-code semantics are closed.
- **Repeat-mistake prevention.** Per `feedback_repeat_mistake_prevention`, PR #177 is the named occurrence; this ADR is the second-occurrence prevention control.
- **No new shell wrapper.** Refuses the `scripts/local-ci.sh` drift surface; the canonical entrypoint per ADR-0324 remains `oya verify`.
- **Closed exit-code enum.** Per-step exit semantics + overall exit semantics are documented; callers can rely on the contract.

### C.2 Negative consequences

- **Local runtime increase.** Running the full mirror takes longer than running `oya gate run-all` alone. Typical timings (on a 16-core workstation): cargo fmt ~5s; cargo check workspace ~60-180s (first run); cargo clippy ~120-300s (first run); cargo nextest ~60-300s. Aggregate ~5-15 minutes first run; ~1-3 minutes incremental. Skip flags mitigate for incremental development.
- **`oya verify` is no longer a "quick" command.** Pre-ADR, the verifier was a thin alias for `oya gate run-all` and completed in seconds for the simple gate set. Post-ADR, the verifier is a full local CI run. Developers who relied on the thin-alias semantics will need to use `oya gate run-all` directly for the lighter-weight gate-only run.
- **Implementation cost.** Wave 15-ZA must extend `crates/oya-dev-cli/src/commands/verify.rs` from ~62 lines (current) to ~250-400 lines (estimated for the five-step orchestration + skip-flag parsing + spec-file consultation + exit-code accumulation). Plus the integration test. Plus the five static-analysis lanes. Estimated 1 codex batch + 1 Claude orchestration session per ADR-0328 batch discipline.
- **CI fidelity gap residual.** The local mirror cannot 100% mirror CI because CI runs on `ubuntu-latest` (linux/amd64); the developer may be on macOS/arm64 or Windows/x86_64. Compilation + lint may behave differently across architectures. The mirror is faithful but not identical; this is acceptable per industry precedent (Google `presubmit` carries the same caveat).
- **Skip-flag misuse risk.** Developers may overuse `--skip-fmt` + `--skip-clippy` + `--skip-nextest` for speed, defeating the verifier's purpose. The `CI=true` environment-variable warning per B2.008 mitigates; corpus discipline + code-review catch the rest.
- **Verifier-recursion guard adds boilerplate.** Per B2.021, the verifier sets `OYA_VERIFY_RUNNING=1` and checks for it at start. Minor implementation overhead.
- **Verifier-log files accumulate.** Per B2.023, `target/oya-verify-runs/*.json` accumulates per-invocation logs. Bounded by `cargo clean`; not a long-term concern.
- **Advisory-step distinction adds cognitive load.** D-6 + D-7 are advisory, D-1..D-5 are mandatory. Developers must remember the distinction. The final-summary line per B2.013 mitigates by reporting both counts.

### C.3 Neutral consequences

- **CI matrix unchanged.** `.github/workflows/pr-tests.yml` jobs continue verbatim.
- **`oya gate run-all` unchanged.** The verifier calls it as D-5; gate aggregator behavior is unchanged.
- **`oya submit` external behavior unchanged.** Submit continues to call the verifier; verifier's exit code continues to gate the push.
- **Pre-push git hook surface unchanged.** `scripts/hooks/pre-push.sh` continues to call `oya verify --ci-required`.
- **Cedar evaluation surface unchanged.** Per ADR-0243; the verifier is a local tool, not a runtime µservice.
- **Audit-chain emission unchanged.** Per ADR-0263; the verifier does not emit audit-chain rows.
- **Tenant scoping unchanged.** Per ADR-0244; the verifier operates at developer-laptop scope, not at tenant scope.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Verifier source carries D-1..D-7 step invocations | Static-analysis lane `oya-governance-oya-verify-ci-mirror-coverage` green; spec at `/specs/oya-verify-ci-mirror.json` matches source |
| Reliability | Exit-code contract closed | E.5 lane green; no verifier exit-code outside `{0, 1, 2}` observed in `target/oya-verify-runs/*.json` logs |
| Performance | Local runtime ≤ ~15 minutes first run; ≤ ~3 minutes incremental | Verifier emits per-step timing; aggregate timing within budget on reference workstation |
| Observability | Per-invocation log + final-summary line | `target/oya-verify-runs/*.json` populated per invocation |
| Substance-bar | Skip-flag allowlist closed; exit-code closed; default full-mirror | E.3 + E.5 lanes green |
| Hyperscaler alignment | Local-mirror posture matches Google/MS/Meta/Amazon/Apple/Netflix precedent | Posture is documented; verifier runs the full CI mirror |
| Resilience | Recursion guard + missing-tool detection | B2.021 + B2.022 implemented; integration test exercises both paths |
| Repeat-mistake prevention | PR #177 incident is the named occurrence; this ADR is the second-occurrence control | Lane evidence at `evidence/lanes/oya-governance-oya-verify-ci-mirror-coverage/` |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** Google's `presubmit` (Piper monorepo tooling, public documentation via Google's "Engineering Productivity" talks at Google I/O 2016+) runs the same build + test + lint suite locally as the post-submit CI runs against the merged commit. Microsoft's `azure-pipelines-task-lib` (https://github.com/microsoft/azure-pipelines-task-lib) ships a local-runner mode for pipeline tasks. Meta's `arc presubmit` (Phabricator → Sapling tooling) runs the build farm's pre-merge gates against a developer's local diff. Amazon's `brazil-build` (internal tooling; public via AWS public engineering content) carries a local mode that mirrors the CodePipeline build matrix. Apple's `xcodebuild test` (via Xcode Cloud) runs locally before the cloud-side CI confirms. Netflix's `nebula-test` (https://github.com/nebula-plugins) mirrors the post-merge Spinnaker CI surface on a developer's laptop. The hyperscaler convergence on a complete local CI mirror is unambiguous; Oyatie's posture per `feedback_quality_performance_scalability_bar` requires matching this pattern. This ADR brings `oya verify` into alignment.

**Failure-mode tree.** Failure modes:
(1) Verifier source removes a mandatory mirror step → `oya-governance-oya-verify-ci-mirror-coverage` lane refuses (BLOCKER post-soak);
(2) Verifier source adds a step not in the spec → static-analysis warns; ADR amendment required;
(3) Verifier source swallows a non-zero exit from a mandatory step → `oya-governance-oya-verify-ci-step-exit-semantics` lane refuses (BLOCKER post-soak);
(4) Verifier source adds a skip flag outside the allowlist → `oya-governance-oya-verify-skip-flag-allowlist` lane refuses (BLOCKER post-soak);
(5) `oya submit` source bypasses `oya verify --ci-required` → `oya-governance-oya-submit-calls-verify` lane refuses (BLOCKER post-soak);
(6) Verifier emits an exit code outside `{0, 1, 2}` → `oya-governance-oya-verify-exit-code-contract` lane refuses (BLOCKER post-soak);
(7) Cargo nextest not installed locally → verifier falls back to `cargo test --workspace --no-fail-fast` per B2.014; warning emitted;
(8) Verifier invoked from subdirectory → either auto-cd to workspace root or exit `2` per B2.018;
(9) Verifier invoked recursively → inner invocation exits `2` per B2.021;
(10) Verifier-log file unwritable → verifier emits warning; primary verification continues.

**Capacity math.** Verifier runtime per invocation: D-1 ~5s; D-2 ~60-180s; D-3 ~120-300s; D-4 ~60-300s; D-5 ~10-60s; D-6 ~5s; D-7 ~5s. Aggregate ~5-15 minutes first run; ~1-3 minutes incremental on a 16-core workstation. Verifier source size estimate: ~250-400 lines (vs current ~62 lines). Static-analysis lane runtime per check: ~1-5s (single Rust source file inspection). Integration test runtime: ~5-10 minutes (exercises the full verifier against a fixture branch).

**Observability hooks.** Per-invocation log at `target/oya-verify-runs/<hlc-timestamp>.json` containing:
- `repo_head_sha`: HEAD SHA at invocation time.
- `invoked_flags`: list of flags passed (`--ci-required`, `--skip-*`, etc.).
- `mandatory_steps`: array of `{step: "D-1", command: "cargo fmt --all --check", exit_code: 0, elapsed_seconds: 4.2}`.
- `advisory_steps`: array same shape.
- `aggregate_result`: `PASS` or `FAIL`.
- `aggregate_exit_code`: `0` / `1` / `2`.
- `verifier_version`: oya-dev-cli crate version.
- `invocation_environment`: `{ci: bool, tty: bool, cpu_count: int}`.

**Rollback path.** Per-verifier-step rollback: revert the verify.rs source to pre-ADR thin-alias posture; the corpus continues operating with the gap. Per-lane rollback: disable a specific static-analysis lane via the lane registry; the lane is REPORT-ONLY before promotion. Cross-feature rollback: revert this ADR with a successor ADR that reinstates the thin-alias posture; no implicit rollback path is provided.

**Multi-region awareness.** The verifier is a local-developer tool; multi-region cell decisions per ADR-0240 do not affect verifier behavior. The verifier runs on the developer's laptop in whatever locale + region; CI runs on `ubuntu-latest` (linux/amd64) in GitHub-managed runners. The mirror is faithful but not identical (per industry precedent caveat).

**Sovereign-cell awareness.** Sovereign cells (HIPAA / GDPR-strict / CSAP / PCI / IL5) inherit the corpus-wide verifier discipline. Sovereign developers run the same verifier on the same source; no per-pack carve-out.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. The verifier source is versioned alongside the corpus. The `/specs/oya-verify-ci-mirror.json` spec file is versioned with a `version` field; spec-file changes require ADR amendment per `feedback_no_silent_regression`. Schema deprecation follows the 12-month corpus-wide deprecation rule.

## D. Detailed mechanics — eleven adoption surfaces

The full-CI-mirror shape touches eleven adoption surfaces in the verifier itself, in the spec file, in the integration test, in the static-analysis lanes, and in the calling chain. Subsections D-1 through D-11 enumerate each surface. Numbering is normative and corresponds to the step identifiers in B.2.

### D-1: cargo fmt --all --check (mandatory, first step)

D-1.1. The verifier MUST invoke `cargo fmt --all --check` as the first mandatory mirror step.

D-1.2. The command validates that every file under the workspace conforms to `rustfmt` formatting per the workspace `rustfmt.toml` (or workspace defaults if absent).

D-1.3. Exit-non-zero indicates one or more files require formatting. The verifier MUST NOT auto-format on behalf of the user; the user is expected to run `cargo fmt --all` separately if they wish to fix the failure.

D-1.4. The CI mirror at `.github/workflows/pr-tests.yml` job `cargo-fmt` (lines 38-50) runs this exact command. The local mirror is byte-for-byte identical.

D-1.5. The `--skip-fmt` flag suppresses D-1. When used, the verifier emits a one-line skip notice and proceeds to D-2.

D-1.6. The expected runtime is ~5 seconds. D-1 is the fastest mandatory step.

D-1.7. The static-analysis target for E.1 lane verification is the literal string `"cargo fmt --all --check"` (or `"cargo fmt --all -- --check"` which is equivalent per cargo CLI semantics) in the verifier source file.

### D-2: cargo check --workspace --all-targets --keep-going (mandatory, second step)

D-2.1. The verifier MUST invoke `cargo check --workspace --all-targets --keep-going` as the second mandatory mirror step.

D-2.2. The command validates compilation of every workspace member including tests, benches, and examples (per `--all-targets`).

D-2.3. The `--keep-going` flag (stable since Rust 1.74) continues checking every crate even after one fails, surfacing all compile errors in a single invocation per the surface-all-failures CI doctrine.

D-2.4. Exit-non-zero indicates one or more crates failed to compile. The verifier records the per-crate error output and includes it in the per-step section.

D-2.5. The CI mirror at `.github/workflows/pr-tests.yml` job `cargo-check` (lines 81-103) runs this exact command. The local mirror is byte-for-byte identical.

D-2.6. There is no skip flag for D-2. Compilation validation is required; the corpus cannot ship un-compilable code.

D-2.7. The expected runtime is ~60-180 seconds first run; ~5-30 seconds incremental.

D-2.8. The static-analysis target for E.1 lane verification is the literal string `"cargo check --workspace --all-targets --keep-going"` in the verifier source file.

### D-3: cargo clippy --workspace --all-targets --keep-going -- -D warnings (mandatory, third step)

D-3.1. The verifier MUST invoke `cargo clippy --workspace --all-targets --keep-going -- -D warnings` as the third mandatory mirror step.

D-3.2. The command runs the clippy linter against every workspace member and every target (tests, benches, examples) with warnings escalated to errors per `-- -D warnings`.

D-3.3. The `--keep-going` flag continues linting every crate even after one fails, surfacing all lint errors in a single invocation per the surface-all-failures CI doctrine.

D-3.4. Exit-non-zero indicates one or more clippy lint failures (including warnings). The verifier records the per-crate output.

D-3.5. The CI mirror at `.github/workflows/pr-tests.yml` job `cargo-clippy` (lines 52-79) runs this exact command. The local mirror is byte-for-byte identical.

D-3.6. The `--skip-clippy` flag suppresses D-3. When used, the verifier emits a one-line skip notice and proceeds to D-4.

D-3.7. The expected runtime is ~120-300 seconds first run; ~10-60 seconds incremental.

D-3.8. The static-analysis target for E.1 lane verification is the literal string `"cargo clippy --workspace --all-targets --keep-going -- -D warnings"` in the verifier source file.

### D-4: cargo nextest run --workspace --no-fail-fast (mandatory, fourth step) with cargo test fallback

D-4.1. The verifier MUST invoke `cargo nextest run --workspace --no-fail-fast` as the fourth mandatory mirror step when `cargo-nextest` is installed locally.

D-4.2. When `cargo-nextest` is not installed (verifier detection: `cargo nextest --version` exit-non-zero), the verifier MUST fall back to `cargo test --workspace --no-fail-fast`. The fallback is functionally equivalent for the verifier's contract.

D-4.3. The `--no-fail-fast` flag continues running tests in every crate even after one fails, surfacing all test failures in a single invocation per the surface-all-failures CI doctrine.

D-4.4. Exit-non-zero indicates one or more test failures.

D-4.5. The CI mirror at `.github/workflows/pr-tests.yml` job `cargo-nextest` (lines 105-133) runs this command with `NEXTEST_PROFILE: ci`. The local mirror omits the `NEXTEST_PROFILE` override (or applies the workspace default `local` profile); the local profile may differ marginally from CI's `ci` profile. The mirror is faithful but not identical at this layer.

D-4.6. The `--skip-nextest` flag suppresses D-4. When used, the verifier emits a one-line skip notice and proceeds to D-5.

D-4.7. The expected runtime is ~60-300 seconds first run; ~30-120 seconds incremental.

D-4.8. The static-analysis target for E.1 lane verification is the literal string `"cargo nextest run --workspace --no-fail-fast"` (or `"cargo test --workspace --no-fail-fast"` as the fallback) in the verifier source file.

### D-5: oya gate run-all --ci-required (mandatory, fifth step)

D-5.1. The verifier MUST invoke `oya gate run-all --ci-required` as the fifth mandatory mirror step.

D-5.2. The command runs the canonical gate aggregator per the current verifier behavior. Existing call-chain semantics are preserved.

D-5.3. The `--ci-required` flag forwards to the gate aggregator and triggers the hosted required-check mirrors (fmt/check/clippy/nextest/admission/provider-execution per the current verify.rs documentation comment lines 23-25) for the lanes that the gate aggregator owns. Note: the redundant invocation of fmt/check/clippy/nextest at the gate-aggregator layer is acceptable; the cargo cache amortizes the cost across runs. The clarification this ADR adds is that the verifier ALSO invokes fmt/check/clippy/nextest DIRECTLY (D-1..D-4) so the local mirror does not solely rely on the gate aggregator's mirroring behavior — which historically did not catch the seven PR #177 failures.

D-5.4. Exit-non-zero indicates the gate aggregator detected a violation.

D-5.5. The `--skip-gates` flag suppresses D-5. Use is documented as "suppress the gate aggregator while iterating on a non-gate concern"; `oya submit` does not pass `--skip-gates`.

D-5.6. The expected runtime is ~10-60 seconds depending on the gate set.

D-5.7. The static-analysis target for E.1 lane verification is the literal string `"oya gate run-all --ci-required"` (or `"gate run-all --ci-required"` via internal dispatch) in the verifier source file.

### D-6: oya doc adr-index --write (advisory, sixth step)

D-6.1. The verifier MUST invoke `oya doc adr-index --write` as the sixth (advisory) step.

D-6.2. The command refreshes the ADR index at `docs/decisions/INDEX.md` (or equivalent canonical index path per the existing oya-doc-cli conventions) from the current set of ADR files.

D-6.3. Exit-non-zero indicates the index could not be regenerated. The verifier emits a warning and continues; D-6 does NOT gate the overall exit code.

D-6.4. The expected runtime is ~5 seconds.

D-6.5. D-6 is advisory because index regeneration is a build-side concern; a non-zero from the index regenerator does not invalidate the verifier's pre-push contract.

D-6.6. The static-analysis target for E.1 lane verification is the literal string `"oya doc adr-index --write"` (or `"doc adr-index --write"` via internal dispatch) in the verifier source file.

### D-7: oya lint adr-shape (advisory, seventh step) — BLOCKER when new ADRs in commit range

D-7.1. The verifier MUST invoke `oya lint adr-shape` as the seventh (conditionally-advisory) step against any new ADRs in the current commit range.

D-7.2. The commit range is computed as `git diff --name-only origin/dev...HEAD -- 'docs/decisions/ADR-*.md'`. If the result is empty, D-7 is informational (no ADRs to lint). If the result is non-empty, D-7 runs on the listed files.

D-7.3. When new ADRs are detected and D-7 exits non-zero, the verifier treats D-7 as BLOCKER and the overall exit code reflects the failure.

D-7.4. When no new ADRs are present, D-7 is informational; exit-non-zero is unlikely (the linter has nothing to refuse) and does NOT gate the overall exit code.

D-7.5. The expected runtime is ~5 seconds.

D-7.6. The static-analysis target for E.1 lane verification is the literal string `"oya lint adr-shape"` (or `"lint adr-shape"` via internal dispatch) in the verifier source file.

### D-8: Skip-flag allowlist (closed) and accelerator semantics

D-8.1. The verifier accepts the closed skip-flag allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}`. No other skip flags are valid.

D-8.2. Each skip flag suppresses exactly one mandatory mirror step: `--skip-fmt` suppresses D-1; `--skip-clippy` suppresses D-3; `--skip-nextest` suppresses D-4; `--skip-gates` suppresses D-5.

D-8.3. There is no `--skip-check` flag. D-2 (cargo check) is unconditionally required; compilation validation cannot be skipped.

D-8.4. Multiple skip flags MAY be combined (e.g., `--skip-clippy --skip-nextest` runs only D-1 + D-2 + D-5 mandatory + D-6 + D-7 advisory).

D-8.5. The verifier MUST emit a warning when any skip flag is used. The warning identifies which step was skipped.

D-8.6. The verifier MUST emit a stronger warning when `CI=true` environment variable is set AND a skip flag is used (per B2.008). The intent is to discourage skip-flag use in automated environments.

D-8.7. Adding a new skip flag requires an ADR amendment per `feedback_no_silent_regression`.

### D-9: Output format mirrors CI lane output

D-9.1. The verifier MUST emit per-step section headers identifying the mandatory step by D-N reference and command-string. Example: `=== D-1: cargo fmt --all --check ===`.

D-9.2. The verifier MUST emit per-step section footers reporting exit code + elapsed seconds. Example: `--- D-1: PASS (4.2s) ---` or `--- D-1: FAIL (exit 1, 4.5s) ---`.

D-9.3. The verifier MUST emit a final summary line of the form: `oya verify: [PASS|FAIL] (mandatory: N/5, advisory: M/2)`.

D-9.4. The verifier MUST inherit CARGO_TERM_COLOR semantics from the calling environment per B2.024. CI defaults to `CARGO_TERM_COLOR=always`; local developers typically have tty-based defaults.

D-9.5. The verifier MUST NOT emit a "spinner" or interactive progress widget. Output is line-oriented and machine-parseable.

D-9.6. The per-invocation log at `target/oya-verify-runs/<hlc-timestamp>.json` carries the same data as the on-stdout summary but in machine-readable JSON.

### D-10: oya submit MUST call oya verify --ci-required before push

D-10.1. `oya submit` MUST call `oya verify --ci-required` before invoking the underlying push. The call is non-bypassable from the `oya submit` source; the `oya-governance-oya-submit-calls-verify` lane (§E.4) enforces by static analysis on the submit source file. Historical/local verifier text only; provenance only, never merge authority; current protected-branch authority is `oya-ci-required` plus cloud-ci/Rust gate packets.

D-10.2. If `oya verify --ci-required` exits non-zero, `oya submit` MUST refuse to push and MUST surface the verifier's output to the user.

D-10.3. `oya submit` MUST NOT pass any skip flag through to `oya verify`. The submit-path always runs the full mirror.

D-10.4. The current `oya submit` implementation already calls the verifier per the 2026-05-15 design; this ADR preserves that chain and adds the static-analysis lane to refuse regressions.

D-10.5. The pre-push git hook at `scripts/hooks/pre-push.sh` is independent of `oya submit`; both call the verifier. The hook fires on `git push` (raw git); `oya submit` fires on `oya submit` (Oyatie wrapper).

### D-11: Exit-code semantics (closed contract)

D-11.1. The verifier exit-code contract is closed: `0` = ALL mandatory steps passed (advisory steps' results do not affect exit code except where D-7 escalates to BLOCKER per B2.025); `1` = at least one mandatory step failed; `2` = invalid arguments (unknown flag, unparseable input, missing required tool, verifier-recursion detected, working-tree-not-at-workspace-root).

D-11.2. The verifier MUST NOT emit any other exit code. The static-analysis lane `oya-governance-oya-verify-exit-code-contract` (§E.5) refuses source changes that emit a non-`{0, 1, 2}` exit code.

D-11.3. Callers (including `oya submit` per D-10 and `scripts/hooks/pre-push.sh`) MUST interpret the exit code per the closed contract: `0` → continue with the push; `1` → refuse the push, surface the verifier output; `2` → refuse the push, surface the argument-error message.

D-11.4. The contract is preserved verbatim from any pre-ADR caller's interpretation. No caller-side change is required by this ADR.

## E. Enforcement-by-lanes

The full-CI-mirror discipline is enforced by five new lanes, each owning a specific aspect of the verifier-completeness contract. Numbering is normative.

### E.1 oya-governance-oya-verify-ci-mirror-coverage (E.1)

E.1.1. The lane validates by static analysis on `crates/oya-dev-cli/src/commands/verify.rs` that the source invokes each of the five mandatory mirror commands D-1..D-5.

E.1.2. The static-analysis surface is the literal command-strings per D-1.7, D-2.8, D-3.8, D-4.8, D-5.7. The lane uses `ast-grep` (per the available `mcp__plugin_oh-my-claudecode_t__ast_grep_search` tool capability) or grep-based source inspection to confirm each command-string is present.

E.1.3. The lane consults `/specs/oya-verify-ci-mirror.json` as the source-of-truth list of mandatory mirror commands. If the spec list and the source invocations disagree, the lane refuses with an actionable message naming the missing command.

E.1.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-verify-implementation-lands. Promotion to BLOCKER 14 days after Wave 15-ZA implementation PR merges.

E.1.5. The lane is owned by axis-dev-cli + council-architecture jointly. Joint ownership reflects the verifier's cross-cutting role.

### E.2 oya-governance-oya-verify-ci-step-exit-semantics (E.2)

E.2.1. The lane validates by static analysis that the verifier source does not swallow non-zero exit codes from any of the five mandatory mirror steps.

E.2.2. The static-analysis pattern refuses Rust source patterns of the form `let _ = command.status()` or `command.status().ok()` or equivalent that discard a non-zero exit code. The verifier MUST capture per-step exit codes and accumulate them per B2.006.

E.2.3. The lane refuses source patterns that conflate fmt-fail with check-fail in the exit code emitted to the caller. The verifier's overall exit code per D-11 must reflect the union of mandatory-step failures, not collapse them.

E.2.4. Lane status: REPORT-ONLY at Acceptance; BLOCKER 14 days post Wave 15-ZA merge.

E.2.5. The lane is owned by axis-dev-cli + ops-sre-reliability jointly.

### E.3 oya-governance-oya-verify-skip-flag-allowlist (E.3)

E.3.1. The lane validates by static analysis that the verifier source declares exactly the closed skip-flag allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}` per D-8.

E.3.2. The lane refuses source changes that add a new skip flag without a corresponding ADR amendment per `feedback_no_silent_regression`.

E.3.3. The lane refuses source changes that remove or rename any of the four allowlist flags without ADR amendment.

E.3.4. Lane status: REPORT-ONLY at Acceptance; BLOCKER 14 days post Wave 15-ZA merge.

E.3.5. The lane is owned by axis-dev-cli + council-architecture jointly.

### E.4 oya-governance-oya-submit-calls-verify (E.4)

E.4.1. The lane validates by static analysis on `crates/oya-dev-cli/src/commands/submit.rs` (or equivalent submit-source path per the current oya-dev-cli layout) that `oya submit` invokes `oya verify --ci-required` before the push step.

E.4.2. The lane refuses source changes that bypass the call (e.g., conditional skip of the verifier under a `--force` flag) without an ADR amendment.

E.4.3. The lane refuses source changes that pass any skip flag through from `oya submit` to `oya verify` per D-10.3.

E.4.4. Lane status: REPORT-ONLY at Acceptance; BLOCKER 14 days post Wave 15-ZA merge.

E.4.5. The lane is owned by axis-dev-cli + ops-platform jointly.

### E.5 oya-governance-oya-verify-exit-code-contract (E.5)

E.5.1. The lane validates by static analysis that the verifier source emits only exit codes in the closed enum `{0, 1, 2}` per D-11.

E.5.2. The lane refuses source patterns that emit literal exit codes outside the enum (e.g., `ExitCode::from(3)`, `process::exit(127)`).

E.5.3. The lane refuses source patterns that emit a non-deterministic exit code (e.g., `ExitCode::from(rand::random())`) — this is a paranoia guard; no such pattern is expected in practice.

E.5.4. Lane status: REPORT-ONLY at Acceptance; BLOCKER 14 days post Wave 15-ZA merge.

E.5.5. The lane is owned by axis-dev-cli + ops-sre-reliability jointly.

### E.6 Pre-push hook (existing surface, optional opt-in)

E.6.1. The pre-push git hook at `scripts/hooks/pre-push.sh` calls `oya verify --ci-required` per the existing 2026-05-15 design. The hook is opt-in (developers install via `oya hook install pre-push` or by copying the script into `.git/hooks/`).

E.6.2. The hook surface is preserved verbatim by this ADR; no hook-source change is required.

E.6.3. The hook is not enforced by a lane (per ADR-0221 "hooks are guidance, CI gates enforce"); developers may bypass the hook with `git push --no-verify`. CI catches what the hook would have caught.

E.6.4. The hook's role is ergonomic — collapse the developer's pre-push feedback loop without imposing the gate.

## F. Alternatives Rejected

### F.1 Rejected: scripts/local-ci.sh shell-wrapper alternative

F.1.1. Alternative: author a shell wrapper at `scripts/local-ci.sh` that invokes cargo fmt + cargo check + cargo clippy + cargo nextest + oya gate run-all sequentially.

F.1.2. Why rejected:

- **ADR-0324 anti-script-anti-template doctrine.** The doctrine refuses parallel shell wrappers that drift from the canonical Rust-strict entrypoint per `feedback_rust_strict_only_no_python_2026_05_20`. The canonical entrypoint per ADR-0212 + ADR-0221 is `oya verify`; shipping `scripts/local-ci.sh` would create a parallel entrypoint that becomes a drift surface — developers + agents would gradually use the shell wrapper instead of the canonical `oya verify`, and the wrapper would drift from the CI matrix over time (the wrapper is not Rust-strict, has no static-analysis surface, and is not under any naming-justification discipline per ADR-0106).
- **No static-analysis surface.** A shell script cannot be statically analyzed for "did the author invoke each of the five mandatory mirror steps?" with the same fidelity as a Rust source file. The verifier-coverage lane per §E.1 is a Rust-source-grep / AST-search; a shell script would require shell-AST tooling not currently in the corpus.
- **No exit-code contract.** Shell scripts conventionally emit `$?` exit codes that are platform-specific (POSIX shells may emit values up to 255; some shells may differ). The closed enum `{0, 1, 2}` per D-11 is enforced in Rust; in shell it would be advisory.
- **Drift surface from CI evolution.** When the CI matrix at `.github/workflows/pr-tests.yml` evolves (e.g., a new gate is added), the canonical `oya verify` is updated by the same engineers who update the CI workflow; a shell wrapper would be updated by a separate path and lag behind. The dual-source-of-truth problem is exactly the named pressure this ADR resolves.
- **`oya verify` is the canonical entrypoint per ADR-0212 + ADR-0221.** Shipping a parallel `scripts/local-ci.sh` would create user-facing ambiguity ("which one do I run?") that the canonical-entrypoint doctrine refuses.

F.1.3. Conclusion: rejected. The full-CI-mirror discipline lands in the canonical `oya verify` entrypoint per the doctrine-compliant path.

### F.2 Rejected: per-cargo-command shell aliases (e.g., `alias oya-fmt='cargo fmt --all --check'`)

F.2.1. Alternative: ship a set of per-cargo-command shell aliases or Makefile targets (e.g., `make verify-fmt`, `make verify-clippy`) that developers invoke individually.

F.2.2. Why rejected:

- **No single entrypoint.** Developers must remember to invoke each alias separately. Forgetting one (which is the PR #177 failure mode — the author forgot `cargo fmt --check` + `cargo clippy` + `cargo nextest`) is exactly the named pressure this ADR resolves.
- **No CI mirror discipline.** Aliases do not enforce "run ALL of these before pushing"; they enforce only "if you ran the alias, it did this one thing." The verifier's full-mirror clause requires a single entrypoint that runs all mandatory steps by default.
- **No skip-flag allowlist.** Aliases do not have a structured allowlist for incremental development; each alias is a separate invocation. The closed skip-flag allowlist per D-8 cannot be enforced.
- **No exit-code contract.** Each alias emits its own exit code; the aggregate exit-code contract per D-11 cannot be enforced.
- **No static-analysis surface.** Aliases live in developer shell-rc files, not in the corpus. They cannot be lane-enforced.
- **ADR-0324 anti-script-anti-template doctrine.** Same reasoning as F.1.
- **Makefile alternative carries the same issues.** A Makefile is a shell-wrapper-equivalent; the per-target rules suffer the same dual-source-of-truth + static-analysis-absence issues.

F.2.3. Conclusion: rejected. The full-CI-mirror discipline requires a single canonical entrypoint, not per-step aliases.

### F.3 Rejected: "fix locally as developers ad-hoc"

F.3.1. Alternative: take no action; developers run `cargo check` + `cargo fmt` + `cargo clippy` + `cargo nextest` + `oya gate run-all` ad-hoc per their own discretion before pushing.

F.3.2. Why rejected:

- **PR #177 is the named occurrence proving this does not work.** On 2026-05-21, the author of PR #177 ran `cargo check --workspace` locally and observed exit-0; CI subsequently surfaced 7 distinct failures. The ad-hoc discipline failed at the exact moment it was tested. Per `feedback_repeat_mistake_prevention`, second-occurrence prevention is the controlling discipline; the second occurrence would be unacceptable.
- **`feedback_automate_everything`.** Anything mechanical (running 5 verifier commands before pushing) MUST be scripted, not run ad-hoc. The verifier IS the automation.
- **Subagent + agent dispatchability.** Agentic developers cannot be relied upon to run ad-hoc verification steps in the correct order with the correct flags. The canonical `oya verify` is the dispatch-friendly entrypoint that subagents + Claude/codex orchestrators invoke once.
- **`feedback_no_silent_regression`.** Ad-hoc verification produces silent regression when developers + agents skip steps or use weaker flags. The full-CI-mirror clause closes the gap.
- **Hyperscaler precedent.** Every hyperscaler engineering team (Google / Microsoft / Meta / Amazon / Apple / Netflix per A.6) ships a local CI mirror as canonical tooling. Ad-hoc verification is not the industry-leader posture.
- **Cost-of-CI argument.** Pushing without local verification consumes CI runners + delays feedback by ~5-20 minutes. The local mirror collapses the loop; ad-hoc verification does not.

F.3.3. Conclusion: rejected. The PR #177 incident is the explicit evidence; the corpus-wide discipline is `oya verify --ci-required`.

### F.4 Rejected: server-side mandatory-CI-pass before push acceptance

F.4.1. Alternative: configure GitHub branch protection on `dev` to refuse pushes until all CI checks pass, eliminating the need for a local mirror entirely.

F.4.2. Why rejected:

- **Branch protection already enforces.** The branch-protection ruleset at `.github/branch-protection.yaml` already declares the CI jobs as required checks; PRs to `dev` block until they pass. The CI-pass-before-merge gate exists; this ADR does not change it.
- **Pre-merge ≠ pre-push.** Branch protection blocks the MERGE, not the PUSH. A developer can push to a branch + open a PR + watch CI fail + push fixes — the local-mirror discipline collapses this loop to "run locally, fix, push once." Branch protection alone does not produce the loop-collapse benefit.
- **CI runner cost.** Each push-without-local-verification consumes CI runners. At corpus scale, the cost is meaningful (~5-20 minutes × N pushes/day across the team).
- **Agent dispatch latency.** Agentic developers dispatched in parallel waves (per `feedback_dispatch_ceiling_claude_only_2026_05_20`) cannot wait for CI cycles before knowing whether their work is valid; the local mirror produces sub-3-minute confirmation that CI cannot match.
- **Developer experience.** Wait-for-CI-then-fix is a worse developer experience than fix-locally-then-push. Hyperscaler precedent per A.6 is unambiguous on this point.

F.4.3. Conclusion: rejected as the standalone solution. Branch protection complements the local mirror; it does not replace it. This ADR preserves the existing branch-protection ruleset verbatim.

### F.5 Rejected: client-side git pre-push hook only (no oya verify extension)

F.5.1. Alternative: extend `scripts/hooks/pre-push.sh` to invoke cargo fmt + cargo check + cargo clippy + cargo nextest + oya gate run-all directly, without extending `oya verify`.

F.5.2. Why rejected:

- **Hook is opt-in.** Per ADR-0221 "hooks are guidance, CI gates enforce." Developers may not install the hook; agents in fresh worktrees may not have the hook configured. The canonical entrypoint must be invokable without hook installation.
- **Dual entrypoint problem.** Same as F.1 — `oya verify` and `scripts/hooks/pre-push.sh` would diverge over time; the dual-source-of-truth problem returns.
- **`oya submit` calls the verifier, not the hook.** Per D-10, `oya submit` MUST call `oya verify --ci-required`; the hook is a separate code path. Extending only the hook would leave the `oya submit` path under-covered.
- **Verifier static-analysis surface.** The verifier source `crates/oya-dev-cli/src/commands/verify.rs` is a Rust file that can be statically analyzed by the §E lanes. The hook source `scripts/hooks/pre-push.sh` is a shell script with weaker static-analysis tooling per F.1.

F.5.3. Conclusion: rejected. The verifier IS the canonical surface; the hook calls the verifier.

## G. Multispectrum review v2.4.0

Per ADR-0322 §D-2, multispectrum review v2.4.0 applies. The 11-13 facets evaluate this ADR. Per-facet evidence files live at `evidence/debate/ADR-0346/`.

### G.1 Facet F1 — Naming + BNF + 13-layer enum (ADR-0105/0106/0107)

The new lane names conform to v4 BNF + 13-layer enum:

- `oya-governance-oya-verify-ci-mirror-coverage` — kebab-case; layer `governance`; substrate `oya-verify`; concern `ci-mirror-coverage`. Naming-justification: governance lane validating verifier-source completeness via static analysis on the verifier crate. ✓
- `oya-governance-oya-verify-ci-step-exit-semantics` — same shape. ✓
- `oya-governance-oya-verify-skip-flag-allowlist` — same shape. ✓
- `oya-governance-oya-submit-calls-verify` — kebab-case; layer `governance`; substrate `oya-submit`; concern `calls-verify`. Naming-justification: governance lane validating submit-source-calls-verifier. ✓
- `oya-governance-oya-verify-exit-code-contract` — same shape. ✓

The substrate prefix is the canonical Rust crate `oya-dev-cli` (which hosts both `verify` and `submit` subcommands); the lane names use the user-facing subcommand identifier (`oya-verify`, `oya-submit`) per Oyatie naming convention.

### G.2 Facet F2 — Architectural cleanness (clean-architecture)

Per `feedback_clean_architecture_requirements`:

- 12-layer enum inward-only flow: the verifier is at the Tier 3 (Application) layer per ADR-0248; it depends on cargo (Tier 2 Capability) + gate-aggregator (Tier 2 Capability); no inverse dependency.
- Port-in-kernel: not applicable; verifier is a local-developer tool, not a runtime µservice.
- Cross-product refusal: not applicable; verifier is dev tooling, not a product.

✓ clean architecture maintained.

### G.3 Facet F3 — Multi-context (oyatie-public / guest-on-aws / guest-on-oci / on-prem / colo / oyatie-as-provider)

The verifier runs on developer laptops; multi-context awareness is not in scope. Developers running on macOS/arm64 / Windows/x86_64 / Linux/x86_64 / Linux/arm64 all run the same verifier; CI's `ubuntu-latest` may behave marginally differently per A.6 caveat. ✓ documented.

### G.4 Facet F4 — Tenant scoping (ADR-0244)

Not applicable. The verifier is a local-developer tool; tenant_id is not in scope. ✓

### G.5 Facet F5 — Cedar gating (ADR-0243)

Not applicable. The verifier is a local-developer tool; Cedar gates apply at runtime, not at verifier-invocation time. ✓

### G.6 Facet F6 — Substance-bar + bespoke authoring (ADR-0322)

Per-section content is bespoke. The named pressures in §A name specific PR #177 failures + specific hyperscaler-precedent products + specific corpus ADRs. The Rejected Alternatives in §F name specific competing patterns + specific reasons. The detailed mechanics in §D enumerate per-step command-strings + per-step expected runtimes + per-step skip semantics — all bespoke to the verifier surface. ✓

### G.7 Facet F7 — Bominal inheritance precedence

Per `feedback_bominal_inheritance_precedence`, Bominal inherits Oyatie ADR decisions 1:1 by default. This ADR's verifier-extension is Oyatie-specific and not directly applicable to Bominal at the source-code level (Bominal has its own dev-cli or equivalent), but the doctrine (local mirror MUST cover CI gates) is inheritable. Bominal applies the same doctrine to its own verifier under its own migration plan. ✓

### G.8 Facet F8 — No silent regression (Linus-style)

Per `feedback_no_silent_regression`. The new five lanes refuse silent regressions: source removing a mandatory step (E.1); source swallowing exit codes (E.2); skip-flag drift (E.3); submit bypassing verifier (E.4); exit-code drift (E.5). ✓

### G.9 Facet F9 — Self-modification (ADR-0247)

Per ADR-0247 self-hosting + self-modification. The verifier is a developer-side tool, but the same Foundry agent runs `oya verify --ci-required` against its own commits per the existing 2026-05-15 design. The full-mirror clause means Foundry agents also benefit from the verifier-completeness; they will not push a branch that fails CI for the gate categories mirrored. ✓

### G.10 Facet M1 — Meta: governance lane vocabulary (ADR-0345 §E.7 inheritance)

Per ADR-0345's vocabulary hygiene clause (the `oya-governance-*` lane-name prefix). The five new lanes use `oya-governance-*` prefix exclusively; no `oya-governance-fitness-*` lanes are introduced. ✓

### G.11 Facet M2 — Meta: ADR-shape compliance (line-floor + frontmatter)

Line floor: ≥ 600 lines (this ADR delivers substantially more). Frontmatter: id + title + status + date + owners + amends + related_adrs + related_specs + related_memory + companion_docs + inbound_citations + doc_class + shape + authority_tier + line_floor + bespoke_authoring_requirement + enforcement_status + enforced_by + purpose — all present. ✓

### G.12 Facet F10 — Quality + performance + scalability bar

Per `feedback_quality_performance_scalability_bar`. The verifier matches industry-leader local-mirror tooling (Google `presubmit`, etc.) per A.6 + C.5. Runtime budget per C.5: ~5-15 minutes first run; ~1-3 minutes incremental. Within hyperscaler-grade developer-experience norms. ✓

### G.13 Facet F11 — Verify-the-deliverable (not just line count)

Per `feedback_verify_deliverables_not_just_line_count_2026_05_20`. The verifier MUST do real verification, not just report line counts or trivial PASS. This ADR's substance bar (D-1..D-7 mandatory steps + closed exit-code enum + static-analysis lane coverage) operationalizes "verify the deliverable" for the verifier surface itself. ✓

### G.14 Acceptance signal (multispectrum-review v2.4.0 verdict)

When all 13 facets verdicts are APPROVE per the per-facet subagent reviews at `evidence/debate/ADR-0346/`, this ADR is Accepted. The aggregate verdict is computed by the multispectrum-review v2.4.0 aggregator per ADR-0322 §D-2.

## H. Sunset

Per ADR-0108 sunset discipline.

H.1. The 14-day sunset window starts on Wave 15-ZA implementation PR merging into `dev`. The five new lanes promote from REPORT-ONLY to BLOCKER at day 14.

H.2. The shorter-than-30-day window (vs. ADR-0345 + ADR-0344's 30-day windows) reflects the lanes' static-analysis-on-a-single-source-file scope. The corpus-wide impact at promotion is bounded; the verifier source is owned by axis-dev-cli and changes infrequently.

H.3. After the 14-day window, ANY change to `crates/oya-dev-cli/src/commands/verify.rs` or `crates/oya-dev-cli/src/commands/submit.rs` that violates the five lanes is REFUSED by CI. The path to landing such a change is an ADR amendment per `feedback_no_silent_regression`.

H.4. Per-µservice manifest impact is bounded — no µservice changes are required by this ADR. The verifier is dev-tooling, not µservice code.

H.5. The /specs/oya-verify-ci-mirror.json spec file's version field is the canonical revision marker. Bumping the version requires ADR amendment.

H.6. The ADR is announced in the realignment-wave findings aggregation, in the next ADR-0327 promotion gate report, and in the developer-experience operator runbook.

## I. Cross-references

I.1. **`crates/oya-dev-cli/src/commands/verify.rs`** — the current verifier source (thin alias for `oya gate run-all`); the Wave 15-ZA implementation extends this file to invoke D-1..D-7 directly.

I.2. **`crates/oya-dev-cli/src/commands/submit.rs`** (or equivalent submit-source path) — the `oya submit` source; the §E.4 lane validates it calls `oya verify --ci-required` per D-10.

I.3. **`.github/workflows/pr-tests.yml`** — the CI workflow that defines the gate matrix the verifier mirrors. Specifically jobs `cargo-fmt` (lines 38-50), `cargo-clippy` (lines 52-79), `cargo-check` (lines 81-103), `cargo-nextest` (lines 105-133), `oya-vcs-admission` (lines 135-155), and the per-governance lanes.

I.4. **`scripts/hooks/pre-push.sh`** — the existing pre-push git hook; preserved verbatim by this ADR.

I.5. **`tools/hooks/_canonical-primitives.md`** — the canonical-primitives doctrine; `oya verify --ci-required` is the canonical local pre-push entrypoint per the Lifecycle Skill Map injected at SessionStart.

I.6. **`/specs/oya-verify-ci-mirror.json`** — the spec file authored under this ADR's required-artifact contract; the closed enum of mandatory mirror commands.

I.7. **`/specs/master-plan-sequencing.json`** — Wave 15-ZA entry added as part of this ADR's required-artifact contract.

I.8. **ADR-0212** (`docs/decisions/ADR-0212-buildability-doctrine.md`) — buildability doctrine; this ADR makes the implicit "verifiable locally" clause explicit.

I.9. **ADR-0221** (`docs/decisions/ADR-0221-agentic-development-pipeline-hardening.md`) — hooks-are-guidance + CI-gates-enforce doctrine; preserved verbatim by this ADR.

I.10. **ADR-0322** (`docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md`) — substance-bar doctrine applied to the verifier surface itself.

I.11. **ADR-0324** (`docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`) — anti-script doctrine refusing parallel shell wrappers per F.1 + F.2 + F.5.

I.12. **ADR-0345** (`docs/decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md`) — immediately-prior ADR; the verifier-coverage extension is part of the realignment-substance arc; the `oya-dev-cli` crate is Maintainer-class per ADR-0345's stewardship registry.

I.13. **`feedback_pre_push_full_ci_mirror_2026_05_21.md`** — the canonical learned-rule memory documenting the PR #177 incident + the seven CI failures.

I.14. **`feedback_codex_dispatch_canonical_2026_05_21.md`** — codex dispatch must include `-c model_reasoning_effort=xhigh`; same "mirror full CI locally" discipline applies to codex output verification.

I.15. **`feedback_verify_deliverables_not_just_line_count_2026_05_20.md`** — verify-the-deliverable discipline; applies to the verifier itself.

I.16. **`feedback_no_silent_regression.md`** — silent-regression containment; the verifier-coverage drift is contained by the §E lanes.

I.17. **`feedback_repeat_mistake_prevention.md`** — second-occurrence prevention; PR #177 is the named occurrence prompting this ADR.

I.18. **`feedback_automate_everything.md`** — mechanical pre-push verification MUST be scripted; the verifier IS the automation.

I.19. **`evidence/debate/ADR-0346/`** — multispectrum-review v2.4.0 per-facet evidence files (authored when this ADR lands in a review-track PR).

I.20. **`microservices/oya-dev-cli/remediation-notes/2026-05-21-oya-verify-full-ci-mirror.md`** — implementation-PR REMEDIATION-NOTES authored under Wave 15-ZA (if the µservice carries that path; the dev-cli is currently a crate, not a µservice).

<!-- completion-report
adr_id: ADR-0346
title: oya verify --ci-required MUST locally mirror the full CI matrix and block on exit-0 of EACH step before returning success
status: Proposed
date: 2026-05-21
owners: [council-architecture, ops-platform, axis-dev-cli, ops-sre-reliability]
line_floor: 600
substance_bar_compliant: true
bespoke_authoring: true
template_stamped: false
amends: [ADR-0212, ADR-0221, ADR-0322, ADR-0324]
new_lanes_count: 5
new_lane_names: [oya-governance-oya-verify-ci-mirror-coverage, oya-governance-oya-verify-ci-step-exit-semantics, oya-governance-oya-verify-skip-flag-allowlist, oya-governance-oya-submit-calls-verify, oya-governance-oya-verify-exit-code-contract]
mandatory_mirror_steps: 5
advisory_steps: 2
exit_code_enum_closed: true
exit_code_enum_values: [0, 1, 2]
skip_flag_allowlist_closed: true
skip_flag_allowlist: [--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates]
sunset_window_days: 14
sunset_anchor: wave-15-za-implementation-pr-merge
implementation_wave: Wave 15-ZA-oya-verify-full-ci-mirror
implementation_wave_in_scope: false
implementation_wave_files_to_change:
  - crates/oya-dev-cli/src/commands/verify.rs
  - crates/oya-dev-cli/src/commands/submit.rs (call-chain validation only)
  - /specs/oya-verify-ci-mirror.json (new spec file)
  - /specs/master-plan-sequencing.json (Wave 15-ZA entry)
  - crates/oya-dev-cli/tests/verify_full_mirror.rs (new integration test)
related_adrs_count: 20
named_pressures_count: 6
rejected_alternatives_count: 5
multispectrum_facets_covered: 13
multispectrum_evidence_path: evidence/debate/ADR-0346/
named_incident: PR #177 surfaced 7 CI failures on 2026-05-21
named_incident_failures: [cargo-fmt, cargo-clippy, cargo-nextest, oya-vcs-admission, oya-governance-dependency-seam, oya-governance-fitness-aspirational-enforcement, oya-governance-fitness-honest-claims]
canonical_primitives_alignment: true
hyperscaler_precedent_named: [google-presubmit, microsoft-azure-pipelines-task-lib, meta-arc-presubmit, amazon-brazil-build, apple-xcodebuild-test, netflix-nebula-test]
out_of_scope:
  - actual Rust code changes to crates/oya-dev-cli/src/commands/verify.rs
  - actual extension of oya submit
  - actual spec file authoring at /specs/oya-verify-ci-mirror.json
  - actual integration test at crates/oya-dev-cli/tests/verify_full_mirror.rs
  - changes to .github/workflows/pr-tests.yml
  - changes to scripts/hooks/pre-push.sh
  - changes to branch protection ruleset
file_path: docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md
-->
