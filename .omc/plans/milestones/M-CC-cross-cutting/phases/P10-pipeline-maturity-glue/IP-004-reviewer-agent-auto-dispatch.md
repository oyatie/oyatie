---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P10-IP-004
title: Reviewer-agent auto-dispatch on PR open + merge-queue integration
status: scaffolded
tier: M
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #4: no reviewer-agent auto-dispatch on PR open"
upstream_kernel: oya-foundry-vcs-review-mergequeue-kernel
purpose: Wire `.github/workflows/pr-review.yml` to dispatch the multispectrum subagent panel from `feedback_consensus_debate_spectrum_lens_subagents.md` on PR open + PR update, post APPROVE/REJECT as a required-check, and emit a merge-queue admission event on APPROVE.
---

# M-CC-P10-IP-004 — Reviewer-agent auto-dispatch on PR open

## Scope

`oya-foundry-vcs-review-mergequeue-kernel` exists per M-CC-P00-IP-007 but no GitHub workflow fans out a reviewer panel automatically. Today's PR #3 had its reviewer hand-dispatched. This IP closes Layer-2 gate 1 by:

- Authoring `.github/workflows/pr-review.yml` triggered on `workflow_run: {workflows: [pr-tests], types: [completed], conclusion: success}` — i.e. review only fires AFTER CI converges green. This matches the canonical state machine (`push → CI → fix-loop until green → review → fix-loop until APPROVE → merge`) and avoids burning review cycles on broken builds. Sub-trigger on `workflow_run` from `pr-tests` + `oya-foundry-fitness-supply-chain` workflows; both must be `success`.
- On review REJECT / CHANGES_REQUESTED, dispatcher emits a `pr-review-fix-requested` event consumed by IP-005's fix-loop (review-side fix-loop, parallel to CI-side fix-loop; same bounded-retry budget pool to prevent runaway).
- Workflow invokes a `tools/oya-pr-review-dispatcher` adapter that fans out per-facet subagents (F1–F9 + M1+M2 + A1–A7 per `feedback_multispectrum_review_v22` + `feedback_multispectrum_adherence_facets`).
- Each subagent posts a sub-finding; the dispatcher rolls up into a single APPROVE / REJECT / CHANGES_REQUESTED Check Run.
- On APPROVE, dispatcher emits a `pr-review-approved` event into the merge-queue admission stream (consumed by IP-006).
- Required-check `oya-pr-review` is added to `.github/branch-protection.yaml` (deployed via IP-001).

## Dependencies

- IP-001 (branch-protection deploy) — to actually enforce the new required-check server-side.
- IP-002 (`oya` CLI) — dispatcher invokes `oya pr-review fan-out` rather than re-implementing subagent orchestration.

## Acceptance

- A test PR opened against `main` triggers `pr-review.yml` within 30s; the workflow completes and posts a single rollup Check Run.
- APPROVE outcome emits a verifiable `pr-review-approved` event in `registries/cross-cutting/merge-queue-admission-log.json`.
- REJECT outcome blocks merge (gated via `.github/branch-protection.yaml` required-check `oya-pr-review`).
- Per-facet subagent findings are stored at `/evidence/pipeline-maturity-glue/ip-004-pr-review/<pr-number>/<facet-id>.json`.
- Evidence rollup at `/evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json`.

## Symbols to grit-claim

- `.github/workflows/pr-review.yml::*`
- `tools/oya-pr-review-dispatcher/Cargo.toml::package`
- `tools/oya-pr-review-dispatcher/src/main.rs::main`
- `tools/oya-pr-review-dispatcher/src/fanout.rs::fan_out_facets`
- `tools/oya-pr-review-dispatcher/src/rollup.rs::rollup_verdict`
- `.github/branch-protection.yaml::required_status_checks` (add `oya-pr-review`)
- `registries/cross-cutting/merge-queue-admission-log.json::*` (new)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json`
- `/evidence/pipeline-maturity-glue/ip-004-pr-review/<pr-number>/` (per-PR fan-out trace)
