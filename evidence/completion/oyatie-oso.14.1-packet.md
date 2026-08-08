# Completion packet — oyatie-oso.14.1 (retrospective, PR #1107)

Retrospective D19/D20 completion receipt for a historical merge. **Evidence repair only.**
This packet does not reopen, re-run, or reimplement the merge it describes.

| Field | Value |
| --- | --- |
| Bead | `oyatie-oso.14.1` — Retrospective completion packet for PR #1107 |
| PR | [#1107](https://github.com/jason931225/oyatie/pull/1107) — `chore(reorg): move metering pipeline into billing` |
| State | `MERGED` (squash) into `dev` |
| PR head SHA | `e74d10bcf5cc269050ce416f3254601be00abd47` (branch `work-hermes-t_0e3c8bcd-reorg-003-pr`) |
| Promoted / merge SHA | `68f477bd9884a4f7b383f0c8f30a9a0f83e9532f` — matches the `68f477bd` named in the bead |
| `mergedAt` | `2026-07-01T08:28:29Z` |
| Packet derived at | `origin/dev` = `3da3bb90930541ed2fbb66f9a68029d2faadebc2` (2026-08-07) |

---

## 1. `oya-ci-required` gate evidence

`oya-ci-required` is a **check-run** (`app_id 15368`), not a legacy status context. All evidence below
was re-derived from the check-runs API against explicit SHAs.

### 1a. Pre-merge admission (PR head SHA)

```
$ gh api repos/jason931225/oyatie/commits/e74d10bc.../check-runs \
    --jq '.check_runs[]|select(.name=="oya-ci-required")|"\(.conclusion) \(.completed_at)"'
success  2026-07-01T08:27:56Z
```

**Join against merge:**

| | |
| --- | --- |
| `oya-ci-required` `completed_at` | `2026-07-01T08:27:56Z` |
| `mergedAt` | `2026-07-01T08:28:29Z` |
| **Margin** | **+33 s — the gate completed BEFORE the merge** |

Run: <https://github.com/jason931225/oyatie/actions/runs/28502892227/job/84488407309>

This is the same run id cited verbatim in the promoted commit message, so the commit's own claim and
the forge record corroborate each other independently.

### 1b. Promoted-SHA gate evidence (the exact promoted commit)

The bead requires gate evidence linked to the **promoted SHA**, not only the PR head. The promoted
commit carries its own post-merge check-run set:

```
$ gh api repos/jason931225/oyatie/commits/68f477bd.../check-runs --jq '.total_count'
47
$ ... --jq '.check_runs[].conclusion' | sort | uniq -c
  47 success
$ ... --jq '.check_runs[]|select(.name=="oya-ci-required")'
conclusion: success   started_at: 2026-07-01T09:01:54Z   completed_at: 2026-07-01T09:01:57Z
```

Promoted-SHA `oya-ci-required` run:
<https://github.com/jason931225/oyatie/actions/runs/28504222744/job/84494714345>

The promoted commit is green on **47/47** check-runs, with zero failures and zero cancellations.
This post-merge run completed 2008 s (33 m 28 s) after `mergedAt`, which is expected: it is the
`dev`-branch verification of the promoted commit, not the admission gate. The admission gate is §1a.

### 1c. Query-method note (a false-alarm trap, recorded deliberately)

The legacy status endpoint returns nothing for this commit **by design**, and reading it as a gate
result produces a false P0:

```
$ gh api repos/jason931225/oyatie/commits/68f477bd.../status --jq '{state,total:(.statuses|length)}'
{"state":"pending","total":0}
```

`state: pending` here means "no legacy status contexts exist", not "the gate did not run". Section 1b
shows 47 green check-runs on the very same SHA. Any future audit of this PR must use
`/check-runs`, never `/status`.

---

## 2. Full check-run history on the merged head — disclosed, not smoothed over

The head SHA `e74d10bc` never changed, yet `oya-ci-required` has **four** recorded attempts against
it. Three failed before the one that admitted the merge:

| Conclusion | `completed_at` | Run |
| --- | --- | --- |
| failure | 2026-07-01T07:39:38Z | [28501494201](https://github.com/jason931225/oyatie/actions/runs/28501494201/job/84480054407) |
| failure | 2026-07-01T07:51:57Z | [28501609728](https://github.com/jason931225/oyatie/actions/runs/28501609728/job/84482159491) |
| failure | 2026-07-01T08:02:41Z | [28502259614](https://github.com/jason931225/oyatie/actions/runs/28502259614/job/84483984756) |
| **success** | **2026-07-01T08:27:56Z** | [**28502892227**](https://github.com/jason931225/oyatie/actions/runs/28502892227/job/84488407309) |

Across all attempts the head SHA tallies 135 success / 5 failure / 10 cancelled. The non-green ones
concentrate in two names — `gate · affected-set (ADR-0554)` (2 failures) and `buck2` plus eight other names — ten cancelled check-runs across nine distinct names, because `buck2` appears twice —
cancelled at 07:39 in what reads as a re-trigger cascade.

**I could not determine why the earlier attempts failed.** The workflow logs for a 2026-07-01 run are
past retention, and I did not attempt to re-run anything (that would violate the evidence-repair-only
scope of this bead). What is established is narrower and is stated plainly: the attempt that precedes
`mergedAt` concluded `success`, and the promoted commit is 47/47 green. Whether the three earlier
failures were infra flake or a real signal that later cleared is **open**, and is listed in §7.

---

## 3. Current disposition at `origin/dev` (2026-08-07)

The merge stands. Verified mechanically:

```
$ git merge-base --is-ancestor 68f477bd... origin/dev ; echo $?
0
```

The change was a pure capability-first relocation (ADR-0562) of six files plus registry bookkeeping:

| Source (pre-merge) | Destination | Present at `origin/dev` today |
| --- | --- | --- |
| `libs/oya-metering-pipeline-kernel/BUCK` | `billing/core/metering-pipeline-kernel/BUCK` | yes |
| `libs/oya-metering-pipeline-kernel/Cargo.toml` | `billing/core/metering-pipeline-kernel/Cargo.toml` | yes |
| `libs/oya-metering-pipeline-kernel/src/lib.rs` | `billing/core/metering-pipeline-kernel/src/lib.rs` | yes |
| `libs/oya-metering-pipeline-kernel/src/conformance.rs` | `billing/core/metering-pipeline-kernel/src/conformance.rs` | yes |
| `libs/oya-metering-pipeline-kernel/src/reference.rs` | `billing/core/metering-pipeline-kernel/src/reference.rs` | yes |
| `libs/oya-metering-pipeline-kernel/tests/reference_sink.rs` | `billing/core/metering-pipeline-kernel/tests/reference_sink.rs` | yes |

`git ls-tree -r origin/dev -- libs/oya-metering-pipeline-kernel` returns **empty** — the old home is
gone, so the move is complete rather than duplicated. `registry/catalog/billing-metering-pipeline-kernel.yaml`
(added by this PR) is still present.

**Workspace membership is live, not orphaned.** `Cargo.toml` at `origin/dev` contains no literal
`metering-pipeline-kernel` entry, which in isolation would look like a dropped crate. It is not: the
workspace uses ADR-0538 shape globs, and `"*/core/*"` selects `billing/core/metering-pipeline-kernel`
by construction. I checked the `exclude` array for a billing/metering carve-out and found none.

**The crate is actively developed at its new home**, which is the strongest available evidence the
relocation did not strand it:

```
$ git log 68f477bd..origin/dev --oneline -- billing/core/metering-pipeline-kernel
b4e6469c9 feat(billing): add idempotent metering batch ingest (#1300)
```

`#1300` (2026-07-10) modified `src/lib.rs`, `src/conformance.rs` and `tests/reference_sink.rs` at the
destination path.

### One disposition delta worth naming

`specs/reorg/billing-move-plan.json`, **added** by this PR, is **absent** from `origin/dev` today. It
was removed by `26fdfed09` (#1184), whose own message says it was done "after replacing the active
move plan". This is the expected single-slot behaviour of the serial reorg strangler — each move PR
installs the active plan and the next one retires it. Note the symmetry inside #1107 itself: it added
`billing-move-plan.json` and deleted its predecessor `calendar-move-plan.json` in the same commit.
**Consistent with the pattern, not a defect.** No defect bead is filed for it.

---

## 4. Rollback

| | |
| --- | --- |
| Revert executed? | No |
| Revert ever needed? | No — no incident, no revert commit, no follow-up remediation found |
| Revert search | `git log origin/dev --grep '68f477bd' --grep '#1107' --grep 'Revert.*metering' -i` → **no matches** |
| Current rollback cost | **Non-trivial and rising** |

The rollback note is that a plain `git revert 68f477bd` would **no longer apply cleanly**: `#1300`
subsequently modified three of the six relocated files in place, so reverting the move now conflicts
with a month of downstream work. The correct rollback for this class is a forward move-back through
the reorg codemod, not a revert.

Boundary: this is derived from the commit overlap shown in §3. **I did not execute or rehearse a
revert** — doing so would dirty a worktree this bead is scoped to keep to a single file. See §7.

---

## 5. Observability and release impact

**Observability — no delta, and none was owed.** The diff touches no `*.openslo.yaml`. That is
correct rather than a co-move miss: SLOs in this repo are owned by services, and `metering-pipeline-kernel`
is a kernel crate. Billing's SLO surface lives at `billing/observability/slos/` and is intact at
`origin/dev` — 12 files including `cloud-billing/metering-event-ingest-latency.openslo.yaml`, which is
the SLO covering this pipeline's runtime behaviour. It was neither moved nor modified by #1107, so the
metering signal's ownership and threshold are unchanged across the relocation.

**Release governance — not applicable.** Per the root `CLAUDE.md` rule that Release Please applies
"only when a live repo config/workflow exists", I checked for one and confirmed the negative:
`git ls-tree -r origin/dev -- .github | grep -i release` returns **nothing**. There is no
release-please config or release workflow in the tree, so no release note is owed for this change. It
is a `chore(reorg)` relocation with no public API or user-visible behaviour change in any case.

---

## 6. Agent-observation harvest disposition

**Disposition: no harvest surface exists; nothing harvested.** I searched the tree for an observation
lane and found no `observations/` store — only `docs/adr-archive/ADR-0620-...-history-only-retirement-observation-surfaces.md`
and an unrelated k8s SLA runbook. There is no durable agent-observation store into which a 2026-07-01
merge could be harvested, retroactively or otherwise.

The only agent-authored observation attached to this merge is the promoted commit's own trailer,
recorded here verbatim as the artifact:

> Closes #1106. Verified by oya-ci-required run
> `https://github.com/jason931225/oyatie/actions/runs/28502892227` and current-head architect review
> `147-Reorg003FinalReview` (CLEAR/APPROVE, no findings).

The CI half of that claim is **independently confirmed** — run `28502892227` is exactly the
successful `oya-ci-required` attempt in §1a. The review half is **not confirmed**; see §7.

---

## 7. What I could NOT verify — read this section before relying on the packet

1. **Reviewer-agent APPROVE is unverifiable.** The `completion_gate` in `CLAUDE.md` requires
   "reviewer-agent APPROVE plus cloud-ci green". The cloud-ci leg is proven (§1). The reviewer leg is
   not: `gh pr view 1107 --json reviews,reviewDecision` returns `reviewDecision: ""` and
   `reviews: []` — **zero review records on the PR**. The commit trailer asserts an architect review
   `147-Reorg003FinalReview` returned CLEAR/APPROVE with no findings, but that artifact exists nowhere
   in the repo (`git ls-tree -r origin/dev | grep -i 'Reorg003\|147-Reorg'` → no matches) and nowhere
   on the PR. The claim is **unsupported by any evidence I could reach** — I am neither endorsing nor
   contradicting it, only recording that it cannot be checked.
2. **Root cause of the three pre-merge `oya-ci-required` failures** (§2) is unknown; run logs are past
   retention and re-running is out of scope for an evidence-repair bead.
3. **Rollback was not rehearsed** (§4). The conflict prediction is derived from commit overlap, not
   from an executed revert.
4. **No runtime, rollout, or deployment verification.** This packet is derived entirely from git and
   the GitHub API. Nothing here observes a running system, and no post-merge rollout verification
   record for this change was found.
5. **No browser or user-story evidence.** The `post_merge_product_gate` calls for it; a `chore(reorg)`
   file relocation has no user-facing surface to exercise, and none was produced at merge time.

## 8. Conclusion

The five acceptance criteria of `oyatie-oso.14.1` are met: PR #1107 and promoted SHA `68f477bd` are
identified (§ header), exact promoted-SHA gate evidence is linked (§1b), current disposition and
rollback posture are verified (§3, §4), observability/release impact and agent-observation disposition
are recorded (§5, §6), and the limits are disclosed rather than papered over (§7).

**This packet makes no product-complete claim.** The merge was validly admitted — `oya-ci-required`
completed `success` 33 seconds before merge, and the promoted commit is 47/47 green — and it still
stands, unreverted and actively built upon. That is an admission-and-disposition receipt, which is
what this bead asks for. It is not a D20 product-complete certification: items 1, 4 and 5 of §7 are
gate legs that were never satisfied for this change and cannot now be satisfied retroactively.

No defect beads are filed. Nothing in current behaviour was found to be wrong.
