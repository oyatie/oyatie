# Post-merge completion packet — oyatie-oso.1

**Bead:** `oyatie-oso.1` — R1 verify dual-worker maxRunners promotion packet
**PR:** [#1564](https://github.com/jason931225/oyatie/pull/1564) — feat(ci): CI-heavy ARC capacity (maxRunners=4, 120Gi, Talos CPU/RAM profile)
**State:** MERGED · base `dev` · head branch `agent/r1-unlock-runners-20260805`
**Promoted (squash merge) SHA:** `ee000cb7d107fe2a9ab85eb9653b29eee396e9bb`
**Pre-merge PR head SHA:** `ce6db39bf176eb6793682f42b67a35e13dd96e7d`
**mergedAt:** `2026-08-05T15:02:58Z`
**Packet author:** agent lane, 2026-08-07. Evidence re-derived from the forge; nothing carried over from the bead notes on trust.

> **Headline: the bead's first acceptance criterion is NOT satisfied.** The exact promoted merge SHA
> `ee000cb7` does **not** have an `oya-ci-required` SUCCESS. Its `oya-ci-required` run was **CANCELLED**.
> The pre-merge head SHA `ce6db39b` *does* carry a genuine `oya-ci-required` success that completed
> 25 s before merge. Details, and the distinction between the two, are below. This packet does not
> assert promoted-SHA green.

---

## 1. Acceptance criteria — required vs. verified

| # | Criterion (from bead) | Verdict |
|---|---|---|
| 1 | Exact promoted merge SHA `ee000cb7` has `oya-ci-required` SUCCESS | **NOT MET** — run cancelled; no success conclusion exists at that SHA |
| 2 | Durable completion packet records promoted SHA, rollout/current-state verification, rollback, observability, release impact, observation-harvest disposition | **PARTIALLY MET** — this file is the durable record; rollout *current-state* is declared-state only, live read-back unobtainable (§5) |
| 3 | `maxRunners=4` scope is accurate | **MET** — verified in tree at promoted SHA and at current `origin/dev` (§4) |
| 4 | External capacity application remains `oyatie-oso.28` | **MET** — `oyatie-oso.28` is OPEN, human-only, and owns the apply (§7) |

Overall: **criteria_met = false**, on criterion 1.

---

## 2. CI evidence — the two SHAs, queried as check-runs

`oya-ci-required` is a check-run (app_id `15368`), not a legacy status context. The legacy
`/commits/<sha>/status` endpoint returns `contexts=0` for check-runs by design and was not used here.

### 2a. Pre-merge PR head `ce6db39b` — oya-ci-required SUCCESS

```console
$ h=$(gh pr view 1564 --repo jason931225/oyatie --json headRefOid --jq .headRefOid)
$ echo $h
ce6db39bf176eb6793682f42b67a35e13dd96e7d

$ gh api repos/jason931225/oyatie/commits/$h/check-runs \
    --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
success 2026-08-05T15:02:33Z
```

Full check-run set at `ce6db39b` (all app_id 15368):

```
buck2 (hermetic build + affected gate tests)                              success   2026-08-05T13:36:17Z
cache-writer identity (trusted dev push only)                             skipped   2026-08-05T13:28:58Z
cloud-ci-firewall (baseline ratchet + gate-registration meta-test)        success   2026-08-05T14:46:08Z
freshness (lock + generated faces, ADR-0539)                              success   2026-08-05T14:01:20Z
gate · ADR census epoch receipt (P2 active; P3 dormant)                   success   2026-08-05T15:01:51Z
gate · affected-set (ADR-0554, binding workspace coverage)                success   2026-08-05T14:59:23Z
gate-live-postgres-adapters (durable adapters: RLS / CDC / SCIM, #901)    skipped   2026-08-05T14:26:03Z
gate-live-postgres-facades (durable facades: tenant lifecycle / SCIM)     skipped   2026-08-05T14:26:03Z
generated-output-diff-policy (no generated merge surfaces)                success   2026-08-05T14:23:26Z
oya-ci-required                                                           success   2026-08-05T15:02:33Z
producer-regen (accounting-registry)                                      success   2026-08-05T14:26:03Z
registry-drift (materialized == regenerated)                              success   2026-08-05T14:31:58Z
```

No `failure` conclusion in the set; the three `skipped` entries are conditional lanes, not suppressed failures.

### 2b. The join — completed_at precedes mergedAt

```console
$ gh pr view 1564 --repo jason931225/oyatie --json mergedAt,mergeCommit,state
mergedAt   = 2026-08-05T15:02:58Z
mergeCommit= ee000cb7d107fe2a9ab85eb9653b29eee396e9bb
state      = MERGED
```

| Event | Timestamp (UTC) |
|---|---|
| `oya-ci-required` completed `success` on head `ce6db39b` | `2026-08-05T15:02:33Z` |
| PR #1564 `mergedAt` | `2026-08-05T15:02:58Z` |
| **Margin** | **+25 s — the check completed BEFORE the merge** |

The ordering is correct: admission was not certified by a check that finished after the fact.
The head that merged is also the head that was tested — `ce6db39b` is the final head (`run_attempt: 1`,
run `31010352098`, the only `oya-ci-required` run at that SHA; no re-run masking an earlier red).

### 2c. Promoted SHA `ee000cb7` — oya-ci-required CANCELLED (the gap)

```console
$ gh api repos/jason931225/oyatie/commits/ee000cb7d107fe2a9ab85eb9653b29eee396e9bb/check-runs \
    --jq '.check_runs[]|"\(.name)\t\(.conclusion)\t\(.completed_at)"'
buck2 (hermetic build + affected gate tests)                        cancelled  2026-08-05T15:08:16Z
cache-writer identity (trusted dev push only)                       skipped    2026-08-05T15:03:02Z
cloud-ci-firewall (baseline ratchet + gate-registration meta-test)  cancelled  2026-08-05T15:08:16Z
freshness (lock + generated faces, ADR-0539)                        cancelled  2026-08-05T15:08:16Z
gate · affected-set (ADR-0554, binding workspace coverage)          cancelled  2026-08-05T15:08:16Z
generated-output-diff-policy (no generated merge surfaces)          cancelled  2026-08-05T15:08:16Z
producer-regen (accounting-registry)                                cancelled  2026-08-05T15:08:16Z
registry-drift (materialized == regenerated)                        cancelled  2026-08-05T15:08:16Z
```

There is **no `oya-ci-required` check-run at all** at the promoted SHA — only its constituent gate
jobs, all cancelled. The rollup never concluded, so it could not conclude `success`.

```console
$ gh api repos/jason931225/oyatie/actions/runs/31018312576 \
    --jq '"\(.name) status=\(.status) concl=\(.conclusion) head=\(.head_sha) event=\(.event) attempt=\(.run_attempt)"'
oya-ci-required status=completed concl=cancelled head=ee000cb7... event=push attempt=1
# created 2026-08-05T15:03:01Z · updated 2026-08-05T15:08:17Z · actor jason931225
```

Run `31018312576` is the only workflow run at the promoted SHA. Jobs *started* (`started_at`
15:03:02Z) and ran ~5 minutes before cancellation at 15:08:16Z — this was not a zero-job pending
eviction.

**Cause: not established.** Recording the observation without asserting a mechanism:

- The successor `dev` push run `31018722164` (SHA `e409b104`) was created `2026-08-05T15:07:55Z`;
  the promoted run was cancelled **21 s later** at `15:08:16Z`. The correlation is suggestive of
  supersession.
- But the workflow config **as it stood at `ee000cb7`** should have prevented cross-SHA
  supersession for push events:
  ```yaml
  concurrency:
    group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
    cancel-in-progress: ${{ github.event_name == 'pull_request' }}
  ```
  For a `push` event the group key includes `github.sha` (per-SHA isolation) and
  `cancel-in-progress` evaluates to `false`. Auto-cancellation by this config is therefore **not**
  a sufficient explanation.
- A manual cancel, a runner-capacity action, or an eviction path not visible in the config all
  remain open. The GitHub API does not expose a cancellation actor for the run. **I did not
  determine why it was cancelled and do not claim to have.**

---

## 3. What the merge actually contained

```console
$ git show --stat --format='' ee000cb7
 infra/arc/CAPACITY-PROFILE-CI-HEAVY.md                     | 108 +++
 infra/arc/README.md                                        |  58 +-
 infra/arc/RUNBOOK-scale-runners.md                         | 142 ++++
 infra/arc/ci-workspace-storage.yaml                        |   6 +-
 infra/arc/runner-scale-set-arm64-values.yaml               |  40 +-
 infra/arc/tests/ci_workspace_capacity.rs                   | 756 +++++++++++------
 infra/talos/local/patches/ci-workspace-worker-1.yaml       |  12 +-
 infra/talos/local/patches/ci-workspace-worker-2.yaml       |  13 +-
 specs/reachability-registry.json                           |   8 +
 9 files changed, 813 insertions(+), 330 deletions(-)
```

Declarative/infra + tests + docs only. No runtime service code, no workflow edits, no cluster mutation
in the diff.

---

## 4. Scope accuracy — maxRunners=4 (criterion 3, MET)

The bead was originally written as `maxRunners=2` and was corrected to `=4` in the 2026-08-07 audit.
The merged artifact confirms **4** is the accurate figure:

```console
$ git show ee000cb7:infra/arc/runner-scale-set-arm64-values.yaml | grep -nE 'maxRunners|minRunners|maxSkew|whenUnsatisfiable'
25:# maxRunners=4 on two general workers (≈2 per node) after expanding each
31:maxRunners: 4
32:minRunners: 0
73:#   per hostname (maxRunners=4 across 2 workers) while still preferring spread.
74:#   DoNotSchedule maxSkew=1 on 4 pods / 2 nodes => 2+2. See ADR-0630.
85:      - maxSkew: 1
87:        whenUnsatisfiable: DoNotSchedule
```

Dual-worker containment is real in the declared artifact: hard `topologySpreadConstraints`
(`maxSkew: 1`, `whenUnsatisfiable: DoNotSchedule`) caps ~2 general runners per
`kubernetes.io/hostname` at max 4. Live-postgres stays at its own `maxRunners: 1`.

---

## 5. Rollout / current-state verification

**Declared state — VERIFIED.** The promoted change is on `dev` and has not been reverted or drifted
in the 26 commits since:

```console
$ git merge-base --is-ancestor ee000cb7 origin/dev ; echo $?
0
$ git rev-list --count ee000cb7..origin/dev
26
$ git show origin/dev:infra/arc/runner-scale-set-arm64-values.yaml | grep -nE 'maxRunners|minRunners'
25:# maxRunners=4 on two general workers (≈2 per node) after expanding each
31:maxRunners: 4
32:minRunners: 0
```

**Live cluster state — NOT VERIFIED, unobtainable from this lane.** The Talos/ARC API is unreachable
from this worktree:

```console
$ kubectl config current-context
admin@oya-talos
$ kubectl get autoscalingrunnerset -A --request-timeout=15s
E... couldn't get current server API group list: Get "https://10.5.0.1:6443/api?timeout=15s":
     context deadline exceeded (Client.Timeout exceeded while awaiting headers)
Unable to connect to the server: context deadline exceeded
```

The API endpoint `10.5.0.1:6443` requires an active port-forward that is not present. No effective
`maxRunners`, pod-per-node distribution, PVC binding, or Talos UserVolume size was read back.

This is expected rather than a defect of this packet: **PR #1564 is a git-side change only.** Its own
body lists the Helm/Argo apply, the Talos 120 GiB volume re-apply, and the optional QEMU CPU/RAM
recreate as *human/operator residual*. Live effect therefore cannot exist until `oyatie-oso.28`
executes. This packet asserts declared state only and makes **no claim of production mutation**.

---

## 6. Rollback, observability, release impact, observation harvest

**Rollback — documented, not exercised.** `infra/arc/RUNBOOK-scale-runners.md` §"Safe scale-down /
rollback" ships in this very merge:

1. Land a git change setting general `maxRunners: 0`–`2`; live-postgres stays under its own plan.
2. Wait until surplus general Pods and ephemeral PVCs are gone.
3. Only then revert `nodePathMap` or Talos volume sizes if desired.
4. Never wipe disks as part of a runner scale rollback.

Git-side rollback is a plain revert of `ee000cb7` (declarative artifacts only, no migration, no
irreversible step). Rollback was **not** rehearsed — nothing was applied to roll back from.

**Observability — declared verification steps exist; no telemetry read.** Runbook §4 defines the
post-apply checks (≤2 general Pods per hostname at max 4; PVC binds on the same node under
WaitForFirstConsumer; no overcommit past 120 GiB; live-postgres remains at one runner; no
DiskPressure/eviction, watch CPU steal at 5 vCPU). These are operator-executed at apply time and are
part of `oyatie-oso.28`'s readback obligation. **I collected no metrics, logs, or dashboards** — see §5.

**Release impact — none / not applicable.** Repo-scoped check: no Release Please configuration or
release workflow is live (`.github/workflows/` contains no release workflow; no
`release-please*.json` at repo root). Per the root `CLAUDE.md` rule, release-governance obligations
apply only when a live repo config/workflow exists. The merge ships infra YAML, a Rust test file,
docs, and a registry entry — no published artifact, no version bump, no consumer-visible API change.

**Observation-harvest disposition — harvested here.** Two durable observations from this lane:

1. **A pre-merge-green PR can still land with a cancelled promoted-SHA run.** The `oya-ci-required`
   evidence lived on head `ce6db39b`; the post-merge `dev` push run at `ee000cb7` was cancelled and
   never re-run. Branch protection was satisfied (head green, 25 s before merge) while the promoted
   commit carries no rollup verdict. Anyone auditing only the merge commit sees no green — and
   anyone querying it with `/commits/<sha>/status` sees `contexts=0` and may mistake a check-run
   surface for a red gate. That double trap is exactly the class this bead was reopened over.
2. **Cancelled ≠ failed, and neither is green.** The workflow's own header comments already record
   the measured dev-branch pathology (60 push runs: 2 success / 27 failure / 31 cancelled, 27 with
   zero jobs) and state the rule directly: *"cancel is never a merge-green substitute."* This packet
   applies that rule to itself and declines to score the promoted SHA green.

Both are recorded in this packet rather than filed as new beads (this lane may not create or modify
beads). Neither is a security finding; nothing about this merge was found to be exploitable.

---

## 7. Boundary — external capacity application (criterion 4, MET)

```console
$ bd show oyatie-oso.28
○ oyatie-oso.28 · Day-2 execute and read back authorized production capacity change [P2 · OPEN]
  labels: capacity, day2, human, lane:capacity-apply, production-authority
  AC: ...authorized operator records target, timestamp, applied/effective capacity, readback,
      rollback readiness, and outcome... No agent claims production mutation.
```

`oyatie-oso.28` is OPEN and human-gated; it owns target/timestamp/applied capacity/readback/rollback
evidence, contingent on an exact authorization or defer decision in `oyatie-3xu`. This packet does
not consume, pre-empt, or discharge it. No cluster mutation was performed by this lane.

---

## 8. Could NOT be verified — stated plainly

1. **`oya-ci-required` SUCCESS at the promoted SHA `ee000cb7`.** It does not exist. The run
   (`31018312576`) is `cancelled`; no rollup check-run is present at that SHA. The bead's first
   acceptance criterion is unmet, and no query I can run from this lane will make it met — only a
   re-run of `oya-ci-required` against `ee000cb7`, or an explicit governance decision to accept the
   exact-head pre-merge green (`ce6db39b`, +25 s margin) as the admission record, can close it. This
   lane has no authority to trigger either.
2. **Why the promoted-SHA run was cancelled.** Correlates with the successor run's creation 21 s
   earlier, but the concurrency config in force at that SHA gives push events per-SHA isolation with
   `cancel-in-progress: false`, so supersession is not established. Cancellation actor is not exposed
   by the API. Undetermined — deliberately not guessed.
3. **Live/effective cluster capacity.** Kubernetes API unreachable (§5). No effective `maxRunners`,
   no pods-per-node distribution, no PVC binding, no Talos UserVolume size, no node CPU/RAM readback.
   Belongs to `oyatie-oso.28`.
4. **Rollback rehearsal and post-apply observability signals.** Both are apply-time activities; the
   apply has not happened. Documented, not exercised.
5. **The "7 passed" capacity-test figure in the bead notes.** Not re-run in this lane. The
   corresponding CI evidence I *did* re-derive is the `buck2 (hermetic build + affected gate tests)`
   check-run at head `ce6db39b`: `success` at `2026-08-05T13:36:17Z`. I did not independently execute
   `buck2 test //infra/arc:ci-workspace-capacity-test`.

---

## Summary

PR #1564 merged with a legitimate, correctly-ordered pre-merge gate: `oya-ci-required` concluded
`success` on the exact merged head `ce6db39b` 25 seconds before `mergedAt`. The `maxRunners=4` scope
is accurate and still intact on `dev`. Rollback and verification procedures are documented in the
runbook the merge itself shipped, and the external capacity apply correctly remains with the
human-gated `oyatie-oso.28`.

The evidence gap the bead was reopened for is **real and remains open**: the promoted SHA `ee000cb7`
carries a cancelled `oya-ci-required` run, not a success. This packet records that as an unmet
criterion rather than papering over it with the head-SHA green.

**criteria_met: false** (criterion 1).
