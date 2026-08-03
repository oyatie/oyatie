# Context Snapshot: Trusted Delivery Pipeline and MPV2 Wave 0

## Task statement

Produce a deliberate `$ralplan` consensus plan for closing Oyatie's planning-entry hold, proving a reproducible and safely parallel delivery pipeline on recovery lanes, replacing draft PR #1524 with protected serial recovery PRs, hardening trusted admission, and completing the first authorized Masterplan v2 wave. Preserve durable lessons from Bun's Rust rewrite, the Gaebal agentic-development archive, and the pinned `it-legal` drafting method. Make the exact 16 Oyatie engineering/review lenses operational.

## Desired outcome

- A decision-complete PRD and test specification under `.omx/plans/`.
- Sequential Planner -> Architect -> Critic consensus, with deliberate-mode pre-mortem and expanded unit/integration/e2e/observability testing.
- A durable handoff that permits a later `$ultragoal` plus `$team` execution lane but performs no implementation during ralplan.

## User-selected scope and rollout decisions

- Close planning entry, prove the pipeline on recovery lanes, then complete the first authorized MPV2 wave.
- Compliance framework first, followed by phased multi-region expansion.
- New validators and trusted admission use shadow-then-blocking promotion.
- Exact review lenses: Cartesian doubt; Essentialism/YAGNI; Chesterton's Fence; contrarian/outside-the-box; Socratic; pragmatism; Red Team; Systems Thinking; Operability/Day-2; Opportunity Cost; blast-radius/cell-based; constant-work/anti-fragility; shared-nothing/eventual consistency; FinOps/unit-cost; telemetry-first; zero-trust/defense-in-depth.

## Repository authority and current evidence

- `specs/root-hub-pointers.json` makes `specs/masterplan.json` the live plan entry surface and `docs/AGENTS.md` the operating contract until explicit PHASE-5 promotion.
- `origin/dev` was `d11567a1a` during the 2026-08-03 analysis. Execution must refresh it and must not use the dirty canonical working tree.
- `origin/dev:specs/masterplan.json:9008-9107` defines five waves and shows none dispatched.
- `origin/dev:specs/masterplan.json:9110-9117` keeps planning entry open with binding approval and dispatch disabled.
- `origin/dev:specs/masterplan.json:9085-9095` protects the founder-ratified sequencing digest; changing wave IDs, membership, dependency edges, or order voids ratification.
- `origin/dev:specs/masterplan.json:9122-9139` records the historical closure candidate as bot-approved without qualified-human proof.
- `origin/dev:AGENTS.md:60-96` already contains the exact 16 review/hyperscale lenses and the three-layer instruction -> automation -> CI enforcement model. The ignored `.worktrees/t_03877337-calendar-e2e/AGENTS.md` is corroboration only, not authority.
- `origin/dev:docs/AGENTS.md:175-224,262-267` requires isolated worktree, signed commit/push, PR to `dev`, independent review, one `oya-ci-required`, squash merge, and a post-merge completion packet.
- Live GitHub inspection on 2026-08-03 found PR #1524 as the only open PR: draft, dirty/conflicting, no approval, and canceled/failed checks. Its W0-C and W0-D content survives only on the branch and must be recovered serially.
- Live branch protection required only `oya-ci-required` from GitHub Actions, with `strict=false`, no required reviews, and admins not enforced. No provider merge queue/ruleset was enabled.
- Recent `oya-ci-required` history showed many cancellations/failures and post-merge failures, so green-on-one-head cannot be treated as pipeline completion.

## Existing implementation surfaces to extend

- `specs/fabric-drive-loop-state.json` and `tools/oya-fabric-loop-state-app` already define loop state, claims, heartbeats, run records, flow metrics, and disjointness checks. The `tools` implementation and filesystem adapters are retirement bridges.
- Current capability topology places the destination in top-level `ci/{core,ports,adapters,facade}`; stale `cloud/cloud-ci` destination language must not create a second topology.
- `plan/fabric-loop/cards/MPV2-0000.C001..C007` contains existing durable cards, including lossy reviewer verdicts, invalid filename fail-closed behavior, and justification gaps.
- `registry/mistakes-ledger.json`, `docs/MISTAKES-LEDGER.md`, `libs/oya-governance-mistakes-ledger-kernel`, and the postmortem template are the existing learning surfaces; do not introduce a parallel observation ledger.
- `specs/hyperscaler-production-readiness-claim-contract.json` already defines claim tiers and evidence domains and should be refined/promoted rather than replaced.
- `specs/compliance-pack-schema.json` already contains source revision and effective/sunset concepts but needs stronger authority, applicability, control mapping, lifecycle, and legal-review semantics. Generated compliance projections must never be hand-edited.
- `.github/workflows/oya-ci-required.yml`, `ci/facade/affected-target-set`, the generated-artifact controller, `libs/oya-ci-gate-contract`, and gate disposition data are existing control-plane primitives.

## Recovery facts

- #1524 contains W0-B `b04328f84` (already merged elsewhere), W0-C `3f5c4b8a0`, and W0-D `b1c4664d0`.
- Correct recovery is W0-C on fresh `origin/dev`, then W0-D on the merged W0-C base. Producer-owned ADR projections are regenerated, not authored or conflict-resolved by hand.
- Small candidates available for current-state revalidation include hook wiring, registry drift, census self-check, reorganization park/selector deletion, multi-lockfile supply-chain coverage, init-app tests plus a sibling flake, and a documentation/count correction.
- The E-shard candidate is verified but no longer justified and should not land.

## External method evidence

- Bun: https://bun.com/blog/bun-in-rust — mapping artifacts before mass work, three-file pilot, isolated worktrees, compiler failures as a work queue, adversarial reviewers, compile-not-proof, canaries, and resource/IOPS measurement.
- Gaebal: https://blog.gaebal-gajae.dev/archive.html — current authority/source checks, durable handoff, exact-head evidence, verified no-op/blocked states, no phantom completion, and turning postmortems into gates/tests/validators.
- it-legal pinned method: https://github.com/jclab-joseph/it-legal/blob/5624ff14e673863ec3b5645155742691a74ef152/README.md — primary-source corpus, authored/effective dates, applicability first, threshold triggers, obligation-to-control/evidence mapping, limitations, and qualified legal review.

## Constraints

- Ralplan is planning-only. Write only `.omx/context`, `.omx/plans`, `.omx/specs`, `.omx/tmp`, `.omx/drafts`, and required `.omx/state` artifacts.
- No implementation edits, pushes, PRs, merges, branch-protection mutation, or cluster mutation.
- No new dependency or revived retired CLI/VCS authority.
- Never hand-edit generated faces.
- Authoring and review remain separate; no self-approval.
- Public claims remain capped by evidence. Planning closure, ordinary PR delivery, trusted admission, production readiness, and hyperscaler readiness are distinct boundaries.

## Open questions resolved by defaults

- Current execution tranche ends after all 23 Wave 0 items and the first compliance regional proof batch are verified. Later regional batches are successor lanes.
- Trusted CI uses the canonical `ci` capability and one protected required context; legacy Actions is a temporary comparison adapter.
- Provider merge queue is transitional behind an owned merge-admission port.
- Concurrency begins with two author lanes plus a serialized integrator and ratchets upward only from measured collision/resource/reviewer evidence.

## Likely planning touchpoints

- `specs/masterplan.json` and cross-artifact agreement closure evaluator.
- Canonical `ci` loop-state, admission, gate contract, affected-target, and generated-artifact surfaces.
- `specs/decision-principles.json` for stable lens IDs and review assessment references.
- Existing readiness, compliance, mistakes-ledger, PR-template, postmortem, and agent-operating-contract surfaces.
