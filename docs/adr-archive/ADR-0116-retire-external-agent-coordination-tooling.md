---
id: ADR-0116
status: Superseded
superseded_by: [ADR-0709]
supersedes: [ADR-0054]
amended_by: [ADR-0363]
planning_impact: true
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0116: Retire external agent-coordination tooling (grit, rtk, icm, vox) in favour of the Foundry pipeline

> **Status:** Accepted — 2026-05-16
> **Supersedes:** ADR-0054 (grit scaffold-claim pattern)
> **Owner:** `council-architecture` + `foundry-vcs`
> **Date:** 2026-05-16
> **doc_class:** DecisionRecord
> **purpose:** Formally retire grit, rtk, icm, and vox from the repo's prescribed agent-coordination surface. The in-repo Foundry pipeline (M01-P18 substrate landed in PR #5 + the wave-A webhook-receiver work) is now the sole canonical workflow for concurrent agent work.
> **enforced_by:** branch-pipeline review/CI lanes
> **Related:** ADR-0054 (deprecated by this ADR), ADR-0052/0053 (historical inventories of the retired primitives), ADR-0110/0111/0112/0113 (Foundry pipeline substrate), ADR-0115 (sibling registry consolidation landed via PR #9)

---

## Status

Accepted — 2026-05-16.

---

## Context

Prior ADRs (ADR-0052, ADR-0053, ADR-0054) elevated `grit` and `icm` as "sanctioned coordination primitives" for new-crate scaffolds and concurrent agent work. `rtk` was used as a cargo shim for token-cost accounting, and `vox` was inventoried as an additional coordination surface. Each of these is an out-of-repo tool maintained outside the oyatie tree.

Three forces have superseded that model:

1. **Foundry pipeline (M01-P18) is live in-repo.** PR #5 landed ADR-0110/0111/0112/0113, which together define the end-to-end agentic VCS substrate: webhook intake, router, admission-gate, projected-merge-state, conflict-kernel, changeset-state, review-mergequeue-kernel. Wave-A (webhook-receiver intake + routing + projected-merge-state crates) is merged. The Foundry pipeline is the in-repo replacement for everything `grit` and `icm` previously coordinated outside the repo.
2. **Branch pipeline is live (2026-05-16).** `dev` → `staging` → `production` with `dev` as the default branch and auto-promotion workflows on 30-min/hourly cadence. PR off `dev` with reviewer-agent APPROVE + CI green is the canonical contribution path; merge-queue handles ordering. This removes any need for an external lock primitive.
3. **No-out-of-repo-coordination invariant.** The long-term operating contract (sunset note already present in `CLAUDE.md`) said grit/icm/rtk/vox would all leave the repo once Oya VCS and Foundry went live. Both conditions are now met.

Sibling consolidation: ADR-0115 (PR #9, in flight from a parallel agent) consolidates the registry layout to flat canonical naming with no qualifier subdirs. This ADR and ADR-0115 are non-overlapping but share the same canonical-naming and in-repo-only direction.

---

## Decision

The following external agent-coordination tools are **retired** from the prescribed agent surface in this repo, effective 2026-05-16:

- `grit` (claim/work/done, scaffold-locks)
- `icm` (coordination-lock topics, scaffold-locks-oyatie fallback)
- `rtk` (cargo shim and command rewrites)
- `vox` (inventoried but unused)

The **Foundry pipeline (M01-P18)** is the sole canonical workflow for concurrent agent work in the oyatie tree. Every agent contribution flows: per-agent `git worktree` → branch off `dev` → `gh pr create --base dev` → webhook-receiver → router → admission-gate (ADR-0111 projected-merge-state + conflict-kernel pre-admit) → merge-queue ordering → reviewer-agent APPROVE + CI green → auto-merge.

Direct build invocation uses `~/.rustup/toolchains/<channel>/bin/cargo` — no shim.

This ADR does **not** delete any historical content. ADR-0054 is marked `Deprecated 2026-05-16` (Superseded by this ADR) and retained for history. ADR-0052/0053 inventory ADRs are left untouched as historical record. The `docs/runbooks/sanctioned-primitives/preflight.md` runbook is tombstoned to a 3-line pointer at this ADR.

---

## Naming justification

Filename `ADR-0116-retire-external-agent-coordination-tooling.md` — `<artifact:ADR>-<id:0116>-<verb:retire>-<scope:external-agent-coordination-tooling>`; conforms to v4 BNF for ADRs under `docs/decisions/ADR-####-<kebab-summary>.md`. The verb `retire` is the canonical lifecycle verb for prescribed-surface removal (parallels `Deprecated` status on ADR-0054). The scope token `external-agent-coordination-tooling` cleanly distinguishes the retired surface (out-of-repo coordination tools) from the retained in-repo Foundry pipeline substrate.

---

## Replacement mapping

| Retired | Replaced by | ADR/PR |
|---------|-------------|--------|
| `grit claim/work/done` | **Foundry pipeline** (M01-P18): per-agent worktree (Layer 0 isolation) → PR off `dev` → webhook-receiver → router → admission-gate + merge-queue (ADR-0111 projected-merge-state + conflict-kernel) → auto-merge on review+CI green | ADR-0110/0111/0112/0113 + PR #5 + wave-A webhook-receiver |
| `icm scaffold-locks` (topic `scaffold-locks-oyatie`) | **Foundry pipeline** Layer-0 isolation (per-agent worktree). Concurrent-safe-paths registry handles file-scope coordination at admission-gate time; no shared mutable file scope exists once each agent works in its own worktree | ADR-0111 §"Conflict-avoidance pre-admit gate" + `registry/vcs/concurrent-safe-paths.yaml` |
| `rtk` cargo shim | Direct cargo via `~/.rustup/toolchains/<channel>/bin/cargo`. No Foundry-pipeline involvement; this is a local build-tool concern only | (no replacement needed) |
| `vox` | No active use — formally retired without replacement | (no replacement needed) |

---

## Migration (changes in this PR)

| Category | File | Change |
|---|---|---|
| CLAUDE.md | `CLAUDE.md` | Removed `sanctioned_primitives` / `required_sequence` / `scaffold_fallback` / `sunset_note` blocks; replaced with `required_workflow` + `substrate_adrs` lists prescribing the Foundry pipeline |
| ADR deprecation | `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` | Status → `Deprecated 2026-05-16`; added "Superseded by ADR-0116" line at top; content otherwise preserved |
| Runbook tombstone | `docs/runbooks/sanctioned-primitives/preflight.md` | Replaced with a 3-line pointer to this ADR |
| Hook removal | `.omc/hooks/grit-claim-intent-gate.sh` | Removed — the gate guards `grit claim` invocations that no longer occur |
| ADR (new) | `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md` | This document |

The repo-wide scrub of historical references to grit/rtk/icm/vox is **out of scope** for this PR: ADRs ADR-0052/0053/0054 and prior runbooks/plans/specs/evidence files reference these tools as historical record. Per the brief, history is preserved; only **prescribing** files (CLAUDE.md fenced agent surface + the sanctioned-primitives runbook + the active grit hook) are rewritten. Surviving mentions are enumerated under "Verification" below.

---

## Consequences

### Positive

- One canonical workflow. Every agent contribution path is the same: worktree → PR off `dev` → Foundry pipeline → auto-merge. No branching by tool inventory.
- No out-of-repo dependency in the prescribed surface. Contributors need only `git`, `gh`, and `cargo`.
- The Foundry pipeline is auditable in-repo: admission-gate decisions, projected-merge-state snapshots, conflict-kernel verdicts, and changeset-state transitions are all recorded by crates in this tree.
- Removes the scaffold-race pattern entirely. Layer-0 worktree isolation gives each agent its own working tree; conflicts surface at admission-gate, not during edit.

### Negative

- Until Foundry wave-B lands (deployment + actual webhook on the repo + gh-api post-back path + dispatcher fan-out), the pipeline does not auto-invoke from a real webhook. Agents still need to run `git push` + `gh pr create` manually to enter the pipeline at the PR-creation gate. The pipeline takes over from the merge-queue step onward.
- Historical references to grit/icm/rtk/vox remain in ADRs, plans, evidence files, and registries. These are knowingly preserved as history; a future cleanup pass MAY rewrite parenthetical references but must not delete the historical record.
- The `Bash(grit *)` permission entry that may exist in agent settings is left in place under deny-by-omission (commands will not be issued; the permission is inert).

### Operational

- Agents MUST NOT invoke `grit`, `rtk`, `icm`, or `vox` for any new work as of 2026-05-16.
- New crate scaffolds use plain `git mv` + `cargo check --workspace --locked --offline` inside a per-agent worktree, with the resulting PR entering the Foundry pipeline.
- Direct cargo invocations bypass the (now-retired) rtk shim: `~/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo …`.

---

## Sunset

The Foundry pipeline substrate that replaces the retired tools is already in `dev`:

- ADR-0110 — changeset state machine (PR #5)
- ADR-0111 — merge-queue projected-state + fix-at-any-stage (PR #5)
- ADR-0112 — webhook-driven foundry agent invocation (PR #5)
- ADR-0113 — VCS orchestrator end-to-end (PR #5)
- Wave-A webhook-receiver crate work (subsequent in-flight PR series; the merge-queue and webhook-receiver crates are merged — all RETIRED per ADR-0363)

**Follow-up — Foundry wave-B.** Wave B is pending: deployment of the webhook-receiver, registration of the GitHub webhook on the repo, gh-api post-back paths from dispatchers, and dispatcher fan-out (pr-review, ci-fix-loop, merge-queue). Until wave-B lands, the seam is:

- PR creation: agents still run `git push` + `gh pr create` manually. This is the manual entry into the pipeline.
- PR merge-queue: Foundry pipeline takes over here (admission-gate → conflict-kernel → projected-merge-state → merge ordering → auto-merge on review+CI green).

This seam SHOULD be removed once wave-B lands; contributors today need to understand which portion is automated vs. queued.

---

## Rejected alternatives

- **Keep grit as transitional indefinitely.** Rejected — leaves two coordination surfaces in the prescribed agent contract, doubles the cognitive load, and contradicts the long-standing sunset note in CLAUDE.md that promised both surfaces would not coexist past Foundry/Oya VCS go-live.
- **Phase out one tool at a time.** Rejected — the four retired tools share a single rationale (out-of-repo coordination, superseded by the in-repo Foundry pipeline). Sequencing the retirement adds calendar surface and policy churn for no design benefit; one atomic ADR is the cleaner break.
- **Delete history.** Rejected — ADR-0052/0053/0054 and the existing evidence/plan record document why these tools were sanctioned in the first place and how the substrate evolved. Deleting that history would hide the design rationale of the Foundry pipeline itself.

---

## Verification (this PR)

- `grep -rln '\bgrit\b\|\brtk\b\|\bicm\b\|\bvox\b' . --include='*.md' --include='*.yaml' --include='*.yml' --include='*.json' --include='*.toml' --include='*.rs' --include='*.sh' --include='Makefile'` — surviving mentions are confined to historical ADRs (ADR-0052, ADR-0053, ADR-0054), evidence/plan files, registries, and this ADR. The retired tools are not prescribed by any file under `CLAUDE.md`, `docs/runbooks/sanctioned-primitives/`, or `.omc/hooks/`.
- `cargo build --workspace` — unchanged by this PR; the changes are documentation + hook removal only.
- `cargo fmt --all --check` — unchanged by this PR.

---

## References

- ADR-0054 — grit scaffold-claim pattern (now Deprecated 2026-05-16; superseded by this ADR)
- ADR-0052 — grit/icm tooling inventory (historical)
- ADR-0053 — grit + icm as sanctioned coordination primitives (historical)
- ADR-0110 — changeset state machine
- ADR-0111 — merge-queue projected-state + fix-at-any-stage
- ADR-0112 — webhook-driven foundry agent invocation
- ADR-0113 — VCS orchestrator end-to-end
- ADR-0115 — registry consolidation (sibling in-flight via PR #9)
- PR #5 — ADR-0110/0111/0112/0113 end-to-end agentic VCS pipeline (M01-P18)
- Branch pipeline policy (live 2026-05-16): `dev` default, `dev` → `staging` → `production` auto-promotion
