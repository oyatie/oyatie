# Post-merge completion packet — oyatie-oso.2 (PR #1562)

- **Bead:** `oyatie-oso.2` — "R2 path-filter live-postgres PR #1562" (P1, labels: `ci`,
  `evidence-repair`, `implementable`, `lane:evidence-packet`, `r2`)
- **PR:** [#1562](https://github.com/jason931225/oyatie/pull/1562) — `ci: path-filter live-postgres
  gates on oya-ci-required`
- **Repo:** `jason931225/oyatie` · base `dev` · head branch `agent/r2-premerge-shape-20260805`
- **PR head SHA:** `adfad9eaaf0ddc6382b168909c0b7151e9650a9b`
- **Promoted (squash merge) SHA:** `215db3402adb34a090346836a4389b2efb849c77`
- **mergedAt:** `2026-08-05T13:27:20Z` · **mergedBy:** `jason931225` · **state:** `MERGED`
- **Change surface:** 1 file, +33 / -7 — `.github/workflows/oya-ci-required.yml` only
- **Packet author position:** worktree `wf_034a880f-ac8-2`, `head_at_start =
  3da3bb90930541ed2fbb66f9a68029d2faadebc2`, which equals `origin/dev` at time of writing.

---

## VERDICT: acceptance criteria NOT fully met

The bead's primary criterion is *"Exact promoted merge SHA `215db340` has `oya-ci-required`
SUCCESS."* **It does not, and no re-derivation can make it so.** The promoted SHA carries **no
`oya-ci-required` check-run at all**, and its workflow run was cancelled before the fan-in job
could exist.

This packet records that plainly rather than substituting the (real, green) pre-merge evidence for
the promoted-SHA attestation the criterion actually demands. Everything else the criterion asks the
packet to record — rollout/current-state, rollback, observability, release impact,
observation-harvest, regression disposition — **is** satisfied and evidenced below.

---

## 1. The criterion that failed: promoted-SHA `oya-ci-required`

```
$ gh api repos/jason931225/oyatie/commits/215db3402adb34a090346836a4389b2efb849c77/check-runs \
    --jq '.total_count as $t | "total_count=\($t)", (.check_runs[]|"\(.name)\t\(.conclusion)\t\(.completed_at)")'
total_count=8
cache-writer identity (trusted dev push only)                              skipped     2026-08-05T13:27:24Z
producer-regen (accounting-registry)                                       cancelled   2026-08-05T15:08:16Z
gate · affected-set (ADR-0554, binding workspace coverage)                 failure     2026-08-05T15:06:02Z
generated-output-diff-policy (no generated merge surfaces)                 cancelled   2026-08-05T15:08:16Z
cloud-ci-firewall (baseline ratchet + gate-registration meta-test)         success     2026-08-05T15:00:35Z
freshness (lock + generated faces, ADR-0539)                               success     2026-08-05T14:49:31Z
registry-drift (materialized == regenerated)                               success     2026-08-05T15:03:57Z
buck2 (hermetic build + affected gate tests)                               success     2026-08-05T14:41:06Z
```

**`oya-ci-required` is absent from that list.** It is the zero-command fan-in job; it is `needs:`-gated
on every constituent lane, so when the run died the fan-in never scheduled and no check-run was
created. There is nothing to grade.

```
$ gh api "repos/jason931225/oyatie/actions/runs?head_sha=215db3402adb34a090346836a4389b2efb849c77"
total_count=1
id=31010277244  name=oya-ci-required  event=push  status=completed  conclusion=cancelled
                created=2026-08-05T13:27:24Z  updated=2026-08-05T15:08:17Z  run_attempt=1
```

This confirms the bead's own AUDIT REOPEN note. **Criterion 1 is unobtainable by query and can only
be repaired by action** (see §8).

### 1a. The `affected-set` "failure" is a kill artifact, not a gate verdict

Do not read the `failure` above as a substantive gate failure — I checked, because a failing
ADR-0554 lane on trunk would be a genuine P1:

```
$ gh api repos/jason931225/oyatie/check-runs/92320366937/annotations
failure  .github    The operation was canceled.
failure  .github    Process completed with exit code 130.
```

Exit **130** is SIGINT. The lane was terminated mid-flight at 15:06:02Z by the same event that
cancelled the run; GitHub recorded `failure` rather than `cancelled` only because the step process
exited non-zero before the cancellation propagated. There is **no substantive gate failure** on the
promoted SHA.

### 1b. Why the run was cancelled — bounded honestly

Four consecutive `dev` push runs were cancelled at the **identical second**:

```
id=31009564936  sha=010c132ec5  cancelled  created=13:18:15Z  updated=2026-08-05T15:08:17Z
id=31010277244  sha=215db3402a  cancelled  created=13:27:24Z  updated=2026-08-05T15:08:17Z   <-- ours
id=31013688156  sha=5d62f6a364  cancelled  created=14:08:40Z  updated=2026-08-05T15:08:17Z
id=31018312576  sha=ee000cb7d1  cancelled  created=15:03:01Z  updated=2026-08-05T15:08:17Z
```

I can **rule out** the repo's two known cancellation classes:

- Not `cancel-in-progress`. At `215db340` the setting is
  `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`, which evaluates **false** for `push`.
- Not trunk pending-eviction. At `215db340` the group is
  `${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}` — **per-SHA**, so
  these four runs sat in four *different* concurrency groups and could not evict one another. (This
  is the CI-CLASS-FIX documented in-file, measured 2026-08-02.)

An identical-second bulk cancel across four independent groups is consistent with an **external
cancellation** — manual bulk-cancel, or account-level quota/billing exhaustion. **I did not confirm
which, and I am not asserting one.** The Actions REST API exposes no cancelling-actor field; that
attribution needs the org audit log, which I did not have access to. Recorded as unverified in §9.

---

## 2. Pre-merge evidence — re-derived, green, correctly ordered

The exact-head evidence the bead credits does exist. Re-derived independently:

```
$ h=$(gh pr view 1562 --repo jason931225/oyatie --json headRefOid --jq .headRefOid)
$ echo $h
adfad9eaaf0ddc6382b168909c0b7151e9650a9b

$ gh api repos/jason931225/oyatie/commits/$h/check-runs \
    --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
success 2026-08-05T13:26:44Z

$ gh pr view 1562 --repo jason931225/oyatie --json mergedAt,mergeCommit,state
mergedAt = 2026-08-05T13:27:20Z
mergeCommit.oid = 215db3402adb34a090346836a4389b2efb849c77
state = MERGED
```

**JOIN:** `completed_at 13:26:44Z` precedes `mergedAt 13:27:20Z`.

> **Margin: +36 seconds** (check completed *before* merge — correct ordering, no
> merged-ahead-of-CI defect).

All 12 check-runs on the PR head were terminal and non-red — 11 `success`, 1 `skipped`
(`cache-writer identity`, correctly skipped on a non-trusted-push context). Both live-postgres legs
ran and passed on the head, which is the designed self-matching behavior: the path filter includes
`\.github/workflows/oya-ci-required\.yml`, so a PR editing that workflow exercises both legs.

---

## 3. Rollout / current-state verification

The promoted commit is in the live trunk lineage:

```
$ git rev-parse HEAD           -> 3da3bb90930541ed2fbb66f9a68029d2faadebc2
$ git rev-parse origin/dev     -> 3da3bb90930541ed2fbb66f9a68029d2faadebc2   (identical)
$ git merge-base --is-ancestor 215db3402adb34a090346836a4389b2efb849c77 origin/dev; echo $?
0    # YES — promoted SHA is an ancestor of current dev
```

**Trunk health since the merge** (`dev`, `event=push`, `created > 2026-08-05T13:27:20Z`):

| conclusion | runs |
|---|---|
| success | 26 |
| cancelled | 3 |
| in-flight (null) | 1 |

**Current dev tip `3da3bb9093` (= this worktree's base) is fully green:**

```
oya-ci-required                                    success   2026-08-07T09:27:51Z
gate-live-postgres-adapters (…RLS / CDC / SCIM)    success   2026-08-07T09:08:30Z
gate-live-postgres-facades (…tenant lifecycle)     success   2026-08-07T09:11:53Z
```

### 3a. Both branches of the merged logic verified in production

I verified the change's actual behavior, not merely that CI is green afterwards. The PR made the two
live-postgres legs path-optional **on `pull_request` only**, and relaxed the fan-in to accept
`success OR skipped` for those two legs alone.

- **Non-PR arm — both legs must always run** (`push`/`merge_group`/`workflow_dispatch` set
  `adapters=true; facades=true` unconditionally). Verified on dev push `e409b104ef` and on the
  current tip `3da3bb9093`: both legs `success` in each case. Durable trunk proof is intact —
  the Chesterton's-Fence concern the PR comment raises is honored in practice.
- **PR arm — non-matching change cone skips both legs, and fan-in stays green.** Verified on
  PR #1614 (`docs(brand)…`, head `fd8a8a69bd`, docs-only):

  ```
  gate-live-postgres-adapters   skipped
  gate-live-postgres-facades    skipped
  oya-ci-required               success
  ```

  This is the load-bearing proof that the relaxed fan-in (`success || skipped`) does not redden and
  does not false-green: the other lanes remain success-only.

**No false-green risk introduced:** the fan-in relaxation is scoped to exactly the two live-postgres
legs; every other lane still requires literal `success`. The filter also fails **open**
(`adapters=true; facades=true`) when the merge-base diff cannot be resolved.

---

## 4. Rollback

- **Method:** `git revert 215db3402adb34a090346836a4389b2efb849c77` — single-file, workflow-only.
- **Blast radius:** confined to `.github/workflows/oya-ci-required.yml`. No crates, no schema, no
  data migration, no runtime service, no persisted state. Nothing to un-migrate.
- **Effect of reverting:** both live-postgres legs return to unconditional execution on every PR and
  the fan-in returns to `success`-only for them. Strictly *more* gating, never less — so rollback
  cannot open a firewall hole. Cost, not safety, regresses (the FinOps/`maxRunners=1` queue
  pressure this PR was written to relieve returns).
- **Rollback exercised?** **No — and not needed.** No condition warranting rollback arose; 26/26
  decided trunk runs since the merge are green. This is a stated, reasoned procedure, not a drill
  I performed. Recorded as such in §9.

---

## 5. Observability

For a change whose entire surface is the required-CI workflow, the observability substrate *is* the
check-run stream on `dev`, which is queryable and was queried above:

- **Signal:** per-SHA `oya-ci-required` conclusion on `dev` pushes — 26 success / 3 cancelled / 1
  in-flight since merge (§3).
- **Per-lane signal:** the two live-postgres legs are individually observable as named check-runs, so
  a `skipped` leg is distinguishable from a `success` leg at query time. This is what let me verify
  both arms in §3a, and it means a future path-filter regression (legs wrongly skipping on a
  matching cone) is detectable from the check-run stream alone.
- **Gap:** there is **no dedicated dashboard, alert, or SLO** on live-postgres leg skip-rate or on
  trunk cancellation rate. The 3 post-merge cancellations were found by ad-hoc query, not surfaced
  by an alarm. The identical-second quadruple cancellation of §1b went unalarmed for two days —
  which is precisely how this bead came to need an evidence-repair lane. Recorded as a gap, not a
  finding against the PR.

---

## 6. Release-governance / release-note impact

**None — and Release Please does not apply.** Verified there is no live config or workflow in the
repo:

```
$ ls .release-please* release-please* .github/release-please*   -> no matches
$ ls .github/workflows/ | grep -iE "release|please"             -> (empty)
```

Per the root `CLAUDE.md` post-merge gate ("Release Please applies only when a live repo config/
workflow exists"), this criterion is satisfied vacuously. The change is CI-internal: no public API,
no user-visible product surface, no ADR status transition, nothing requiring a release note.

---

## 7. Observation-harvest disposition

Harvested from this verification pass. All are **process/evidence observations**, none is a defect in
the merged change:

1. **A green PR head does not attest the promoted SHA.** #1562 merged 36s after its exact-head check
   went green — textbook-correct — yet the promoted SHA has *no* `oya-ci-required` verdict at all.
   Pre-merge green and post-merge attestation are independent obligations; satisfying the first
   silently leaves the second unmet.
2. **`conclusion: failure` can mean SIGINT.** The `affected-set` lane graded `failure` purely because
   its process exited 130 under an external kill. Grading a promoted SHA on the conclusion string
   alone would have manufactured a phantom ADR-0554 trunk failure — the false-alarm shape this
   lane exists to prevent. Always read the annotations.
3. **Fan-in absence reads as silence, not red.** Because `oya-ci-required` is `needs:`-gated, a
   killed run produces *no* required check-run rather than a failing one. Any monitor that asks
   "is `oya-ci-required` red on trunk?" will answer "no" for a SHA that was never verified at all.
   Absence of the proxy is not absence of the problem.
4. **The two known cancellation classes were already fixed here, and this still happened.** Both
   `cancel-in-progress` and trunk pending-eviction are ruled out by the config *as it stood at the
   promoted SHA* (§1b). The in-file comment's own aside — "the 4 cancelled runs that DID schedule
   jobs are a separate, unrelated cause" — is unresolved, and this incident is four such runs.

## 8. Regression disposition

**No regression discovered. No implementation issue is linked, because inventing one would be
fabrication.**

The merged behavior was positively verified on both arms (§3a), the trunk is green through the
current tip (§3), and the single red mark on the promoted SHA is a kill artifact (§1a). The
outstanding defect is in *evidence*, not in *code*.

The bead's repair — an exact promoted-SHA `oya-ci-required` SUCCESS — remains **available but
unperformed**: the workflow retains `workflow_dispatch`, so a maintainer can dispatch
`oya-ci-required` pinned to `215db3402adb34a090346836a4389b2efb849c77` and produce the missing
attestation directly. I did not do this: it requires write authority this lane does not hold, and my
instructions forbid `git push` / `gh pr create` / `bd` mutation. **This is the one action that would
flip criterion 1 to met.**

---

## 9. What I could NOT verify, and why

Stated plainly, per the honest-incompleteness requirement:

1. **`oya-ci-required` SUCCESS on promoted SHA `215db340` — THE PRIMARY CRITERION. NOT MET.**
   No such check-run exists (`total_count=8`, name absent); run `31010277244` concluded `cancelled`.
   Unobtainable by any query. Requires a `workflow_dispatch` re-run pinned to that SHA — an action
   outside this lane's authority (§8).
2. **The cause of the 15:08:17Z quadruple cancellation.** Ruled out `cancel-in-progress` and
   concurrency eviction from the config at that commit. Cannot attribute further: the Actions REST
   API exposes no cancelling actor, and the org audit log — which would show a manual bulk-cancel or
   a quota/billing stop — was not accessible to me. **Leading hypothesis (external bulk cancel or
   quota exhaustion) is explicitly UNCONFIRMED and must not be cited as fact.**
3. **Rollback was not exercised.** §4 is a reasoned procedure derived from the diff's blast radius,
   not a performed drill. No revert was executed or dry-run.
4. **No browser / user-story evidence.** The root `CLAUDE.md` post-merge gate lists it; it is not
   applicable here — the change has no user-facing surface whatsoever (one CI workflow file).
   Recorded as N/A rather than silently dropped.
5. **Runner-fleet and cost effect not measured.** The PR's stated motivation is FinOps relief on the
   `maxRunners=1` live-postgres queue. I verified the legs *do* skip on non-matching PRs (§3a) but
   did **not** measure realized queue-time or runner-minute savings; no before/after figure is
   claimed anywhere in this packet.

---

## Summary table

| Acceptance element | Status | Evidence |
|---|---|---|
| Promoted SHA `215db340` has `oya-ci-required` SUCCESS | **NOT MET** | check-run absent; run `31010277244` cancelled (§1) |
| Durable completion packet exists | MET | this file |
| Rollout / current-state verification | MET | ancestor of `origin/dev`; tip green; both arms verified (§3, §3a) |
| Rollback recorded | MET (not exercised) | §4, §9.3 |
| Observability recorded | MET (gap noted) | §5 |
| Release impact recorded | MET (N/A — no live config) | §6 |
| Observation-harvest disposition | MET | §7 |
| Discovered regression linked as separate issue | N/A — none discovered | §8 |

**`criteria_met = false`**, on the strength of item 1 alone. Pre-merge admission for #1562 was sound
(+36s margin, correctly ordered); the merged behavior is verified correct in production on both
arms; the promoted-SHA attestation the bead demands does not exist and was not manufactured here.

---

*Packet generated 2026-08-07 from `origin/dev@3da3bb9093`. Every command shown was executed against
`jason931225/oyatie`; check-run facts were re-derived via the `/commits/{sha}/check-runs` endpoint,
never via the legacy `/status` endpoint (which returns `contexts=0` for check-runs by design).*
