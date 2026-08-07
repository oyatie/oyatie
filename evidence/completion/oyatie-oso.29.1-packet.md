# Post-merge completion packet — oyatie-oso.29.1

Bead: `oyatie-oso.29.1` — "Publish durable PR #1614 reviewer and post-merge receipt" (P2, type task).
Parent: `oyatie-oso.29` (CLOSED) — "P2 brand follow-on historical merge PR #1614".
Packet authored: 2026-08-07. All forge facts below were re-derived from the GitHub API in this session; none
were carried over from the bead text or from any prior lane's assertion.

**Verdict: criteria NOT fully met.** Five of the seven acceptance elements are satisfied by durable evidence.
Two are not, and are stated plainly in [Could not be verified](#could-not-be-verified). This packet does not
claim the exact-head independent review exists.

---

## 1. Identity and merge facts

| Fact | Value |
| --- | --- |
| Bead | `oyatie-oso.29.1` |
| PR | [#1614](https://github.com/jason931225/oyatie/pull/1614) — `docs(brand): keep-with-sunset dispose .omc/ultragoal brand residue` |
| Author | `jason931225` |
| Head branch → base | `agent/hygiene-brand-20260807` → `dev` |
| PR head SHA (exact head) | `fd8a8a69bd81399ed975707c6cadf4d0d4902dea` |
| Promoted / merge SHA | `3da3bb90930541ed2fbb66f9a68029d2faadebc2` (`3da3bb909`) |
| State | `MERGED` |
| createdAt | `2026-08-07T08:31:16Z` |
| mergedAt / closedAt | `2026-08-07T08:59:53Z` |
| Squash | single commit `fd8a8a69` → merge commit `3da3bb909` |

Verification commands and output:

```
$ gh pr view 1614 --repo jason931225/oyatie \
    --json number,headRefOid,state,mergedAt,mergeCommit,baseRefName,headRefName
{"baseRefName":"dev","closedAt":"2026-08-07T08:59:53Z","createdAt":"2026-08-07T08:31:16Z",
 "headRefName":"agent/hygiene-brand-20260807",
 "headRefOid":"fd8a8a69bd81399ed975707c6cadf4d0d4902dea",
 "mergeCommit":{"oid":"3da3bb90930541ed2fbb66f9a68029d2faadebc2"},
 "mergedAt":"2026-08-07T08:59:53Z","number":1614,"state":"MERGED",
 "title":"docs(brand): keep-with-sunset dispose .omc/ultragoal brand residue"}

$ git merge-base --is-ancestor 3da3bb90930541ed2fbb66f9a68029d2faadebc2 origin/dev; echo $?
0                       # promoted SHA is genuinely on trunk, not merely claimed
```

Packet-authoring position, recorded before any other action (the lane could have started anywhere):

```
$ git rev-parse HEAD
3da3bb90930541ed2fbb66f9a68029d2faadebc2      # == the promoted SHA under audit
$ git merge-base --is-ancestor origin/dev HEAD; echo $?
0
```

---

## 2. `oya-ci-required` on the exact head — the pre-merge join

`oya-ci-required` is a **check-run** (app_id `15368`), not a legacy status context. The commit *status* API
returns `contexts=0` for it by design, and the squash merge commit does not carry the PR's pre-merge checks
at all. Both errors were avoided; the check-runs API was queried against the **PR head SHA**.

```
$ h=$(gh pr view 1614 --repo jason931225/oyatie --json headRefOid --jq .headRefOid)
$ echo $h
fd8a8a69bd81399ed975707c6cadf4d0d4902dea

$ gh api repos/jason931225/oyatie/commits/$h/check-runs \
    --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
success 2026-08-07T08:59:34Z
```

**The join:**

| | |
| --- | --- |
| `oya-ci-required` conclusion (exact head `fd8a8a69`) | `success` |
| `completed_at` | `2026-08-07T08:59:34Z` |
| `mergedAt` | `2026-08-07T08:59:53Z` |
| **Margin (mergedAt − completed_at)** | **+19 seconds** |

The margin is **positive**: the required context completed green **19 seconds before** the merge landed. This
merge did *not* front-run its gate. That is the material question and it is answered in the affirmative.

Total check-runs on the exact head: 20, all `success` or `skipped`, none `failure`:

```
$ gh api repos/jason931225/oyatie/commits/$h/check-runs \
    --jq '.check_runs[]|"\(.name) | \(.conclusion) | \(.completed_at)"'
oya-ci-required                                              | success | 2026-08-07T08:59:34Z
gate · affected-set (ADR-0554, binding workspace coverage)   | success | 2026-08-07T08:59:26Z
buck2 (hermetic build + affected gate tests)                 | success | 2026-08-07T08:41:51Z
cloud-ci-firewall (baseline ratchet + gate-registration)     | success | 2026-08-07T08:38:22Z
registry-drift (materialized == regenerated)                 | success | 2026-08-07T08:36:55Z
producer-regen (accounting-registry)                         | success | 2026-08-07T08:35:43Z
generated-output-diff-policy (no generated merge surfaces)   | success | 2026-08-07T08:35:32Z
freshness (lock + generated faces, ADR-0539)                 | success | 2026-08-07T08:35:37Z
gate · ADR census epoch receipt (linux-amd64 binding)        | success | 2026-08-07T08:37:04Z
gate · platform smoke (linux-arm64 / macos-arm64 / win-amd64)| success | 08:37:03 / 08:38:00 / 08:41:23
Analyze (actions / python / c-cpp / rust) + CodeQL           | success | 08:32:05 … 08:36:53
gate-live-postgres-adapters / -facades (#901)                | skipped | 2026-08-07T08:35:43Z
cache-writer identity (trusted dev push only)                | skipped | 2026-08-07T08:31:20Z
```

Note `gate · affected-set` completed at `08:59:26` — 8 seconds before `oya-ci-required` rolled up. The
aggregate context is genuinely gated on the slowest constituent, not stamped early.

---

## 3. Promoted-SHA (trunk) run `31164028155`

```
$ gh api repos/jason931225/oyatie/actions/runs/31164028155 \
    --jq '{id,name,head_sha,head_branch,event,status,conclusion,run_started_at,updated_at}'
{"conclusion":"success","event":"push","head_branch":"dev",
 "head_sha":"3da3bb90930541ed2fbb66f9a68029d2faadebc2",
 "id":31164028155,"name":"oya-ci-required","run_started_at":"2026-08-07T08:59:57Z",
 "status":"completed","updated_at":"2026-08-07T09:27:52Z"}

$ gh api repos/jason931225/oyatie/commits/3da3bb909.../check-runs \
    --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
success 2026-08-07T09:27:51Z
```

Confirmed: run `31164028155` is a **`push`-event run on `dev` at head_sha `3da3bb909`** — i.e. genuinely the
promoted-SHA run named in the acceptance criteria — and its conclusion is `success`. All 19 constituent
check-runs on the promoted SHA are `success`, with only `cache-writer identity` skipped. Notably the two
`gate-live-postgres-*` gates that were `skipped` on the PR head ran and passed on trunk.

**Trunk-green timing, stated honestly:** the run *started* at `08:59:57Z`, four seconds **after** `mergedAt`,
and completed at `09:27:51Z` — **27 min 58 s after the merge**. This is the expected shape for a post-merge
push run and is not a defect. But it is the precise reason the parent bead `oyatie-oso.29` was "closed before
trunk green": its close reason cites the `08:59:53Z` merge, which predates trunk green by ~28 minutes. The
sequencing is now durably recorded here rather than inferred.

---

## 4. What actually merged — scope for the disposition below

```
$ gh pr view 1614 --repo jason931225/oyatie --json files,additions,deletions,changedFiles
changedFiles=4  +47 −56   (net −9 lines)
  .gitignore                                                  +6  −3
  .omc/ultragoal/TEAMMATE-PREAMBLE.md                        +33 −45
  .omc/ultragoal/premise.txt                                  +3  −3
  ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json  +5  −5
```

Content, read from the merge commit (`git show 3da3bb909`): prose tombstoning of two `.omc/ultragoal` files
under ADR-0709, comment-only changes to `.gitignore` freezing the four-path exception, and `reason` **string**
edits to four existing allowlist entries in the hygiene policy JSON. The allowlist *rule set* is unchanged —
same four `id`/`kind`/`value` triples before and after; only the human-readable `reason` fields and one
`product_contract` prose field were reworded. No Rust, no BUCK, no workflow, no `*.generated.json`, no
`Cargo.lock`, no service or deployment surface. Net-negative prose.

---

## 5. Release / rollback / observability disposition

| Dimension | Disposition | Basis |
| --- | --- | --- |
| **Release governance / release note** | **Not applicable.** | Per root `CLAUDE.md`, Release Please applies only when a live repo config/workflow exists. `git ls-files \| grep -i release-please` on `origin/dev` returns **zero** tracked files. No release artifact was due and none is missing. |
| **Rollout verification** | **No rollout.** | The diff ships no deployable unit: zero Rust crates, zero BUCK targets, zero Helm/kustomize, zero workflow files. There is nothing to roll out and therefore no rollout to verify. The trunk-green run `31164028155` is the whole of the promotion verification. |
| **Rollback** | **Single-commit revert, blast radius bounded to one CI data file.** | `git revert 3da3bb909` restores four files. The only behaviour-bearing path is `ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json`, and because only `reason` strings changed while the four allowlist `value` entries are byte-identical, a revert cannot change which paths the hygiene gate admits. `.gitignore` changes are comment-only. Rollback risk is effectively nil; no data migration, no state, no compatibility window. |
| **Observability** | **No observability surface touched; nothing to check.** | No `slos/*.openslo.yaml`, no metric, no log line, no trace, no alert is added, removed, or renamed by this diff. The applicable "observability check" for a prose+CI-data change is the CI gate corpus itself, which is green on the promoted SHA — specifically `cloud-ci-firewall`, `registry-drift`, `generated-output-diff-policy`, and `gate · affected-set` all `success` at `3da3bb909`. |

This disposition is derived from the merged diff read in full, not assumed from the PR title.

---

## 6. Observation-harvest outcome

**Outcome: no observation was harvested for PR #1614.** Recorded here as a negative result, durably.

```
$ git show origin/dev:.omc/ultragoal/friction-ledger.jsonl | grep -c "1614"
0
```

The CI-load-bearing friction ledger (`ledger_path`, the very file this PR's commit message calls out as
deliberately left unchanged) contains **zero** entries referencing PR #1614. No friction, defect, or process
observation from this lane was promoted into the durable ledger.

Assessment: for this change that is a defensible outcome rather than a miss. The PR is a net −9-line prose and
CI-data disposition that merged with a clean +19 s gate margin and a fully green trunk run; it generated no
gate failure, no retry, and no pipeline friction worth a ledger row. The one genuine process observation this
lane *did* produce is not about PR #1614's content at all — it is the evidence-durability defect recorded in
§7 below, which is what bead `oyatie-oso.29.1` exists to capture.

---

## 7. Could not be verified

Both items below are stated as failures, not smoothed over. `criteria_met = false` because of §7.1.

### 7.1 Exact-head independent review — **NO DURABLE EVIDENCE EXISTS, AND NONE WAS FOUND LOCALLY EITHER**

The acceptance criterion requires durable evidence of an "exact-head independent review". It is not satisfied.

```
$ gh api repos/jason931225/oyatie/pulls/1614/reviews --jq '.[]|"\(.user.login) \(.state) \(.commit_id)"'
                                    # (empty — zero reviews)
$ gh api repos/jason931225/oyatie/pulls/1614/comments --jq 'length'
0                                   # zero inline review comments
$ gh api repos/jason931225/oyatie/issues/1614/comments
                                    # (empty — zero issue comments)
$ git ls-tree -r --name-only origin/dev | grep -E "pr-1614"
                                    # (no match — rc=1)
$ find . -path ./.git -prune -o -name "*1614*" -print
                                    # (no match in this worktree)
```

PR #1614 carries **zero** reviews, **zero** review comments, and **zero** issue comments on the forge. It was
merged unreviewed as far as the forge record is concerned.

The repository has an established durable-critic convention — `.grok/programs/*/evidence/pr-NNNN-critic-{a,b}.json`,
`pr-NNNN-dual-critic.json`, and `evidence/ci/pr-NNNN-dual-critic.json` (89 such files tracked on `dev`, most
recently `pr-1595-dual-critic.json` and `evidence/ci/pr-1570-dual-critic.json`). **No `pr-1614` artifact exists
under any of these paths.**

The bead description asserts the exact-head critic artifact "is only local/untracked". I searched the tracked
`origin/dev` tree and this entire worktree and **could not locate it in either**. I therefore cannot promote it,
cannot cite its verdict, and will not paraphrase a review I have not read. Scope limit stated precisely: my
search covered `origin/dev` as tracked and the worktree at
`/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_034a880f-ac8-7`. It does **not** cover the main checkout's
untracked working tree or any other machine — absence here is not proof of absence everywhere, and a
differently-shaped confirmation would be needed to close that out.

**Consequence:** this criterion stays open. Satisfying it requires either (a) locating the local critic artifact
and committing it as `evidence/ci/pr-1614-dual-critic.json` per the existing convention, or (b) running a fresh
independent review pinned to exact head `fd8a8a69bd81399ed975707c6cadf4d0d4902dea` and committing that. Option
(b) is the sounder path: a rediscovered local file has no provenance binding it to the exact head, which is
precisely the failure mode this bead was opened against.

### 7.2 The parent's closure authority is a local-only artifact — **CONFIRMED PRESENT AS A DEFECT**

The criterion "no local-only artifact is treated as authority" is currently **violated by the parent bead's
existing close reason**, which this packet does not inherit and does not repeat as authority.

`bd show oyatie-oso.29` closes the parent with:

> "VERIFIED gh: PR #1614 MERGED 2026-08-07T08:59:53Z merge=3da3bb909305 … fleet-babysit-merge-4 cycle evidence
> **babysit-cycle.latest.json**."

```
$ git ls-tree -r --name-only origin/dev | grep -i babysit
.grok/programs/delivery-fabric/BABYSIT-SINGLE-FLIGHT.md
.grok/workflows/pr-babysit-lanes.rhai
$ find . -path ./.git -prune -o -name "*babysit*" -print
./.grok/workflows/pr-babysit-lanes.rhai
```

`babysit-cycle.latest.json` is **not tracked on `origin/dev` and not present in this worktree**. The parent bead
was closed citing an artifact that has no durable existence — exactly the local-only-authority pattern the
acceptance criterion forbids.

This packet's remedy is structural, not rhetorical: **every** merge fact in §1–§3 was re-derived from live forge
API calls with the commands and raw output quoted inline, so nothing here depends on `babysit-cycle.latest.json`,
on the parent bead's close reason, or on the task description that spawned this lane. What that artifact cannot
supply — the exact-head review of §7.1 — is left explicitly open rather than back-filled from it.

I did not modify or reopen bead `oyatie-oso.29`; correcting its close reason is outside this lane's write scope.

---

## 8. Criteria roll-up

| # | Acceptance element | Status |
| --- | --- | --- |
| 1 | Durable evidence identifies PR #1614 | ✅ verified (§1) |
| 2 | Promoted SHA `3da3bb909` | ✅ verified, and confirmed an ancestor of `origin/dev` (§1) |
| 3 | Successful promoted-SHA run `31164028155` | ✅ verified `success`, push-event on `dev` at `3da3bb909` (§3) |
| 4 | Exact-head independent review | ❌ **NOT SATISFIED** — zero forge reviews, no durable artifact, none found locally (§7.1) |
| 5 | Release / rollback / observability disposition | ✅ recorded, derived from the merged diff (§5) |
| 6 | Observation-harvest outcome | ✅ recorded — outcome is *no observation harvested*, ledger `grep -c 1614 == 0` (§6) |
| 7 | No local-only artifact treated as authority | ⚠️ **honoured by this packet; violation found in the parent's close reason** (§7.2) |

**`criteria_met = false`.** The merge itself is sound and should be preserved — it gated green with a +19 s
margin and went trunk-green on the promoted SHA. What is missing is the independent review, and this packet
says so rather than implying a verification that did not happen.
