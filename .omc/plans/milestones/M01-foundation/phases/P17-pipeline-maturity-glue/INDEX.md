---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P17
title: Pipeline maturity glue (binary/CLI/workflow surfaces that lift M01-P07 kernels from substrate → operational)
status: scaffolded
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
purpose: Close the partial-maturity gap between the M01-P07 substrate (kernels + fitness lanes scaffolded) and an unsupervised claim → work → verify → done → PR → review → CI → merge → deploy pipeline. This phase ships the binary/CLI/workflow surfaces that lift those kernels from substrate into operational enforcement.
naming_justification: "Slot id M01-P17 is the next free integer slot after the existing P00..P09 stack under `.omc/plans/milestones/M01-foundation/phases/`. Per `feedback_no_exceptions_canonical.md`, canonical extension = append next integer; two-digit inserts (P01a) and renumbering occupied slots are rejected. Slug `pipeline-maturity-glue` describes the layer: binary/CLI/workflow surfaces sitting *above* M01-P07 kernels (substrate) and *below* deploy-time (Layer 3 follow-on lands at P11)."
---

# M01-P17 — Pipeline maturity glue

## Purpose

Lift the M01-P07 substrate (30+ `oya-foundry-*-kernel` crates, 9 declared required-checks, ratchet/promotion-controller kernels) into an enforced runtime pipeline. The 2026-05-15 audit (`evidence/audits/pipeline-maturity-audit-2026-05-15.md`) verdict is PARTIAL with one URGENT critical: branch protection is declared in `.github/branch-protection.yaml` but not deployed on GitHub. This phase closes that gap and the 4 other top-blockers via 8 ChangeSet-sized IPs.

## Entry gate

- M01-P07 (gitops-vcs-replacement) — all 9 IPs status: complete. DONE.
- Audit `evidence/audits/pipeline-maturity-audit-2026-05-15.md` landed (commit `a8b629b` + amendment commit `dded068`).

## Exit gate

- An agent executes `claim → work → verify → done → PR → review → CI → merge → deploy` end-to-end without human intervention.
- Pipeline-maturity audit, re-run on phase-exit, scores **MATURE** (not PARTIAL) across all 10 stages and all 4 layers.
- The 3 net-new constraints from the 2026-05-15 amendment (webhook fix-loop, merge-queue fix-loop integration, surface-all-failures CI) are operational, evidence-backed, and CI-enforced.

## Authority boundary

P10 consumes the M01-P07 kernels and exposes binary/CLI/workflow surfaces that wrap them. No new kernel logic is authored at this phase except where a closing-the-gap requires a thin fitness kernel (e.g. banned-primitives detector in IP-008). Layer-3 staging/canary/rollback is explicitly **out of scope** — deferred to M01-P18.

## ChangeSet contract

Per M01-P07's "ImplementationPlan == ChangeSet" rule, every IP below is a cohesive claim/work/verify/done/promote unit. If an IP discovers unrelated lock scopes during execution, split into child IPs before claiming broader trees.

## Implementation Plans

| IP | Title | Tier | Status | File |
|---|---|---|---|---|
| IP-001 | Branch-protection deploy + auto-merge enablement | S | scaffolded | [`IP-001-branch-protection-deploy.md`](IP-001-branch-protection-deploy.md) |
| IP-002 | `oya` CLI binary + `oya gate run-all` aggregator | M | scaffolded | [`IP-002-oya-cli-gate-aggregator.md`](IP-002-oya-cli-gate-aggregator.md) |
| IP-003 | Mistakes-ledger 5-control stack (preflight + template + fitness lane) | M | scaffolded | [`IP-003-mistakes-ledger-5-control-stack.md`](IP-003-mistakes-ledger-5-control-stack.md) |
| IP-004 | Reviewer-agent auto-dispatch on PR open + merge-queue integration | M | scaffolded | [`IP-004-reviewer-agent-auto-dispatch.md`](IP-004-reviewer-agent-auto-dispatch.md) |
| IP-005 | CI-failure webhook → fix-loop agent dispatch | M | scaffolded | [`IP-005-ci-failure-webhook-fix-loop.md`](IP-005-ci-failure-webhook-fix-loop.md) |
| IP-006 | Merge-queue fix-loop integration (parked-PR semantics + bounded retry + fairness) | L | scaffolded | [`IP-006-merge-queue-fix-loop-integration.md`](IP-006-merge-queue-fix-loop-integration.md) |
| IP-007 | Surface-all-failures CI workflow refactor (relax `needs:`, `if: always()`) | S | scaffolded | [`IP-007-surface-all-failures-ci.md`](IP-007-surface-all-failures-ci.md) |
| IP-008 | Banned-primitives fitness check (catches grit/git bypass; auto-flips when grit retires) | S | scaffolded | [`IP-008-banned-primitives-fitness-check.md`](IP-008-banned-primitives-fitness-check.md) |
| IP-009 | *(deferred to M01-P18)* Layer 3 staging canary + cohort + rollback | XL | deferred | n/a |

## Execution order

Wave 1 (URGENT, can ship same-day, no inter-dep): **IP-001** (branch-protection deploy) + **IP-007** (surface-all-failures CI). These two unblock everything downstream.

Wave 2 (depends on Wave 1): **IP-002** (CLI binary) + **IP-008** (banned-primitives fitness check). IP-002 is the kingpin — it ends the silent `git`/`gh` bypass.

Wave 3 (depends on Wave 2): **IP-003** (mistakes-ledger), **IP-004** (reviewer-agent), **IP-005** (CI-failure webhook).

Wave 4 (depends on Waves 1–3): **IP-006** (merge-queue fix-loop integration) — the convergence layer that ties IP-004 + IP-005 + IP-007 together.

## Concurrent-execution note

IP-001 and IP-007 are already in flight on a parallel implementation agent as of phase-scaffold time. The plan files for those two IPs document the contract that implementation must satisfy; commit ordering coordinates (audit landed before phase files; impl edits to `.github/workflows/pr-tests.yml` may have landed before, during, or after this phase commit).

## Evidence anchors

- Source audit: `evidence/audits/pipeline-maturity-audit-2026-05-15.md` (with 2026-05-15 amendment)
- Each IP declares its own `/evidence/pipeline-maturity-glue/ip-<NNN>-<slug>.json` exit-evidence path.

## Agent-handoff

Phase-exit ChangeBundle promoted by:

```
oya vcs done --agent <id> --changeset <id>
oya vcs promote --changeset <id> --phase M01-P17
```
