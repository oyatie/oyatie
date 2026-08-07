# Post-merge completion packet — oyatie-oso.5 (CAS Lane 3A NativeLink rehome)

| Field | Value |
| --- | --- |
| Bead | `oyatie-oso.5` — CAS Lane 3A NativeLink rehome (external `gh-1563`) |
| PR | [#1563](https://github.com/jason931225/oyatie/pull/1563) — `reorg(storage): rehome NativeLink CAS provider to storage/adapters (Lane 3A)` |
| Head SHA (reviewed/tested) | `e8495b28fdfdbaf96a1b9d53909f68d985a60005` |
| Promoted merge SHA | `010c132ec5aa0b1f07f4687c0288913eed122c52` |
| Base | `dev` (squash merge, on `origin/dev`, ancestry confirmed) |
| mergedAt | `2026-08-05T13:18:12Z` |
| mergedBy | `jason931225` |
| Packet author position | `head_at_start=3da3bb90930541ed2fbb66f9a68029d2faadebc2`, `origin/dev` is ancestor (exit 0) |

## Verdict

**criteria_met = false.**

The bead's acceptance criterion is a conjunction. Its *first* conjunct — "Exact promoted merge SHA
`010c132e` has oya-ci-required SUCCESS" — is **verifiably not satisfied**. This is not an
"unverifiable" case: the run exists, was located, and its conclusion is `cancelled`. Every other
conjunct (current-state, rollback, observability, release impact, harvest disposition, authority,
no-secret) was verified and is recorded below.

---

## 1. oya-ci-required — the load-bearing finding

`oya-ci-required` is a check-run (app_id 15368, `github-actions`), not a legacy status context. The
legacy `/status` endpoint was deliberately **not** used; it returns `contexts=0` for check-runs by
design and would have produced a false reading.

### 1a. On the PR head SHA — SUCCESS, pre-merge

```
$ gh api "repos/jason931225/oyatie/commits/e8495b28fdfdbaf96a1b9d53909f68d985a60005/check-runs?check_name=oya-ci-required" \
    --jq '.check_runs[]|{name,conclusion,started_at,completed_at,app_id:.app.id}'
{"app_id":15368,"completed_at":"2026-08-05T13:17:46Z","conclusion":"success",
 "name":"oya-ci-required","started_at":"2026-08-05T13:17:43Z","status":"completed"}
```

**Join:** completed_at `2026-08-05T13:17:46Z` → mergedAt `2026-08-05T13:18:12Z`.
**Margin = +26 seconds.** The required check completed **before** the merge, in the correct
direction. Head-SHA admission is clean.

### 1b. On the promoted merge SHA — NO SUCCESS CHECK-RUN EXISTS

```
$ gh api ".../commits/010c132ec5aa0b1f07f4687c0288913eed122c52/check-runs?check_name=oya-ci-required" --jq .total_count
0
$ gh api ".../commits/010c132ec5aa0b1f07f4687c0288913eed122c52/check-runs?per_page=100"       --jq .total_count
11
$ gh api ".../commits/010c132ec5aa0b1f07f4687c0288913eed122c52/check-runs?app_id=15368&per_page=100" --jq .total_count
11
```

11 check-runs exist on the promoted SHA; **none of them is named `oya-ci-required`.** The
aggregating check-run was never emitted, because the workflow run that would emit it did not
finish successfully:

```
$ gh api repos/jason931225/oyatie/actions/runs/31009564936 \
    --jq '{name,head_sha,event,status,conclusion,created_at,updated_at}'
{"conclusion":"cancelled","created_at":"2026-08-05T13:18:15Z","event":"push",
 "head_branch":"dev","head_sha":"010c132ec5aa0b1f07f4687c0288913eed122c52",
 "name":"oya-ci-required","status":"completed","updated_at":"2026-08-05T15:08:17Z"}
```

This is the only workflow run against the promoted SHA. **Conclusion `cancelled`, not `success`.**

### 1c. Why it was cancelled — evidenced, not inferred

Per-job breakdown of run `31009564936`:

| Job | Conclusion |
| --- | --- |
| buck2 (hermetic build + affected gate tests) | success |
| producer-regen (accounting-registry) | success |
| generated-output-diff-policy | success |
| gate-live-postgres-facades (#901) | success |
| gate-live-postgres-adapters (#901) | success |
| registry-drift (materialized == regenerated) | success |
| cloud-ci-firewall (baseline ratchet) | success |
| freshness (lock + generated faces, ADR-0539) | success |
| gate · affected-set (ADR-0554) | success |
| cache-writer identity (trusted dev push only) | skipped (by design — not a dev push) |
| **gate · ADR census epoch receipt (P2 active; P3 dormant)** | **cancelled** |

10 of 11 substantive jobs passed. The single cancelled job never ran:

```
$ gh api ".../actions/runs/31009564936/jobs" --jq '.jobs[]|select(.conclusion=="cancelled")'
{"name":"gate · ADR census epoch receipt (P2 active; P3 dormant)",
 "started_at":"2026-08-05T14:19:15Z","completed_at":"2026-08-05T15:08:17Z",
 "runner_name":"","labels":["oya-arm64"],"steps":[]}
```

`runner_name` is empty and `steps` is an empty array: the job **queued for 49 minutes waiting for
an `oya-arm64` self-hosted runner and was cancelled without executing a single step.** This is
runner starvation on the scarce ARC arm64 pool, not a gate failure and not a defect in the Lane 3A
change. One never-scheduled job carried the whole run's conclusion to `cancelled`, and with it the
`oya-ci-required` check-run.

**A hypothesis I tested and discarded:** I first suspected concurrency-group eviction, because
PR #1561 merged at `15:07:52Z`, 25 seconds before this run ended at `15:08:17Z`. I checked the
workflow as it stood *at the merge commit* rather than as it stands today, and the group was
already per-SHA:

```
$ git show 010c132e:.github/workflows/oya-ci-required.yml | grep -A3 '^concurrency:'
concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

Per-SHA grouping plus `cancel-in-progress: false` for `push` events rules out group eviction. The
timing correlation with #1561 is real but **is not evidence of causation**, and I am not claiming
it as such. The runner-starvation evidence above stands on its own.

### 1d. Standing tension worth recording (not an excuse)

The workflow's own header documents this failure class as systemic, measuring "the last 60 `push`
runs on dev: 2 success / 27 failure / 31 cancelled". The same header states the admission contract
explicitly: *"cancel is never a merge-green substitute — only a successful oya-ci-required on the
current head admits."* By that contract, #1563's admission (head-SHA green, +26s pre-merge) was
correct. The bead's acceptance criterion asks for something **stronger** than the repo's own
admission contract — promoted-SHA green — and that stronger bar is not met. Both statements are
true simultaneously; I am not using the second to discharge the first.

---

## 2. Rollout / current-state verification — VERIFIED

The change is a pure path rehome. Shape of the squash commit (10 files, `-M` rename detection):
2 × `R100` (byte-identical renames), 7 × `M`, 1 × `A`.

```
{infra => storage/adapters}/nativelink/OWNERS               | 0
{infra => storage/adapters}/nativelink/nativelink-cas.k8s.yaml | 0
Cargo.toml                                                  | 13 +++++-----
ci/facade/operator-secret-rbac/operator-secret-bootstrap-policy.json | 4 +--
infra/arc/runner-scale-set-arm64-values.yaml                | 2 +-
infra/arc/tests/ci_workspace_capacity.rs                    | 4 +--
infra/external-secrets/RUNBOOK.md                           | 2 +-
registry/fixuptasks.jsonl                                   | 2 +-
specs/reachability-registry.json                            | 4 +--
specs/reorg/nativelink-storage-move-plan.json               | 11 ++++++++
```

Current state on `origin/dev` today:

- `git ls-tree -r origin/dev -- infra/nativelink` → **empty**. Old path deleted, no symlink, no
  copy, no compatibility alias.
- `git ls-tree -r origin/dev -- storage/adapters/nativelink/` → `OWNERS`,
  `nativelink-cas.k8s.yaml`. Both present, content byte-identical (R100).
- Live machine-consumed consumers rewritten to the new path and still correct today:
  `Cargo.toml`, `ci/facade/operator-secret-rbac/operator-secret-bootstrap-policy.json`,
  `infra/arc/runner-scale-set-arm64-values.yaml`, `infra/external-secrets/RUNBOOK.md`,
  `registry/fixuptasks.jsonl`, `specs/reachability-registry.json`.
- Residual `infra/nativelink` mentions outside scratch dirs are 4 files in `docs/adr-archive/`
  plus `docs/decisions/ADR-0700` — historical ADR prose, not machine-consumed activation paths.
  Consistent with the lane's own stated boundary.

### 2a. Post-merge regression found — attributed to a LATER PR, not to #1563

`specs/reachability-registry.json` on `origin/dev` today carries **two** nativelink prefix rows:

```
217:      "prefix": "storage/adapters/nativelink/",
477:      "prefix": "infra/nativelink/",
```

Line 477 points at a path that no longer exists. **#1563 was clean** — at the promoted SHA the file
had exactly one such row:

```
$ git show 010c132e:specs/reachability-registry.json | grep -n '"prefix"' | grep -i nativelink
210:      "prefix": "storage/adapters/nativelink/"
```

and the merge diff shows a clean in-place rewrite of that single row (`-infra/nativelink/` →
`+storage/adapters/nativelink/`). The stale row was reintroduced afterwards:

```
$ git log --oneline -S'"prefix": "infra/nativelink/"' 010c132e..origin/dev -- specs/reachability-registry.json
e409b104e feat(k8s): admit deterministic Go-to-Rust port W0-A (#1561)
```

PR #1561 merged `2026-08-05T15:07:52Z`, ~1h50m **after** #1563. Its own critic record
(`.grok/programs/k8s-port/evidence/pr-1561-critic-b.json`) reasoned that its prefixes "coexist with
#1558 CAS prefixes (`infra/nativelink/`, …). Prefixes are additive and non-overlapping" — i.e. the
critic evaluated against a pre-#1563 tree and thereby blessed a prefix that #1563 had already
deleted. This is a concurrent-lane staleness defect in #1561's admission, **not** a Lane 3A defect,
and it is out of scope for this bead. Recorded here because the criterion asks for current-state
verification and current state is genuinely non-clean. It warrants its own bead.

### 2b. Move plan is now PARKED and stale-but-inert

#1563 added `specs/reorg/nativelink-storage-move-plan.json` as the active plan. PR #1576
(`940899863`, `chore(reorg): singleton live move-plan + park nativelink (W0)`) renamed it to
`nativelink-storage-move-plan.PARKED.json`. The parked file still declares the
`infra/nativelink → storage/adapters/nativelink` artifact move as pending, with
`unpark_when: "G039 terminal packet + this plan is the single live rehome lane"` — while the move is
already executed and promoted. Inert (marked `PARKED (not executable)`), but factually stale.

---

## 3. Rollback — VERIFIED as a stated path, NOT rehearsed

- **Door:** the governing ADR-0560 is `door: two-way`. The change is a path rename with no content,
  identity, digest, or topology change, so it is mechanically reversible.
- **Procedure:** `git revert 010c132ec5aa0b1f07f4687c0288913eed122c52` restores
  `infra/nativelink/` and reverts the seven consumer rewrites.
- **Measured conflict risk (revert is no longer clean):** 8 of the 10 touched paths have changed on
  `dev` since the merge — `Cargo.toml` (#1610), `specs/reachability-registry.json` (#1561, #1564,
  #1570, #1574, #1575, #1582, #1584), `registry/fixuptasks.jsonl` (#1610),
  `infra/arc/runner-scale-set-arm64-values.yaml` and `infra/arc/tests/ci_workspace_capacity.rs`
  (#1564, #1573), and the move plan (#1576, renamed). Only the two moved artifacts themselves are
  untouched. A revert today requires manual conflict resolution on the registry and Cargo files.
- **Blast radius if rolled back: nil at runtime.** The manifest is dark (see §4) — nothing
  reconciles it, so a revert moves bytes and changes no running system.
- **Not rehearsed.** I did not execute a trial revert; doing so would have modified the tree beyond
  the single file this task permits. The conflict assessment above is derived from commit history,
  not from an attempted merge.

---

## 4. Observability — VERIFIED (no obligation triggered, and why)

- The rehomed artifact is a **dark declaration**. `specs/cache-warm-license.json` on `origin/dev`
  still reads `"warm_reads_licensed": false`, with reason "no live CAS endpoint is reachable from
  any executing lane and the cold integrity-canary has never run GREEN". Lane 3A did not flip it —
  consistent with the PR body ("warm_reads_licensed remains false; no RE scheduler/worker").
- The reachability row for the new prefix carries its own disclaimer: "activation requires a later
  reviewed GitOps Application plus #1551/#1534 live proof, so this reachability row grants no
  deployment claim."
- **No SLO file exists for the nativelink adapter.** `storage/observability/slos/` is populated
  (availability, latency, etc.) but contains no entry for the CAS adapter, and
  `storage/adapters/nativelink/` holds only `OWNERS` and the manifest. Under the repo's
  observability substrate policy, SLO authoring is mandatory **before a service promotes past dev**;
  this artifact has not been promoted past dev and is not reconciled by anything, so no SLO
  obligation is triggered by this move. **This is a deferred obligation, not a discharged one** —
  it becomes blocking at activation, not at rehome.
- There is therefore no runtime signal to check post-merge. A path rename of an unreconciled
  manifest emits no telemetry. Stating this plainly is the honest form of "observability check" for
  this change; claiming a green dashboard would be fabrication.

---

## 5. Release-governance / release-note impact — VERIFIED: none

Release Please applies only where a live repo config/workflow exists. Neither does:

```
$ git ls-tree -r --name-only origin/dev | grep -iE "release-please|\.release-please"   → (empty)
$ git ls-tree -r --name-only origin/dev -- .github/workflows | grep -iE "release"      → (empty)
```

No release configuration and no release workflow exist in the repository. **Release impact: N/A.**
Independently, the change is a path-only move of a dark artifact with no user-visible or API
surface, so it would carry no release note even if the machinery existed.

---

## 6. Observation-harvest disposition — VERIFIED ABSENT

No friction-ledger row or observation record exists for this lane:

```
$ git grep -c "1563" origin/dev -- .omc/ultragoal/friction-ledger.jsonl   → (no match)
$ git grep -o -h "cas-3a[a-z0-9-]*" origin/dev -- .omc/ultragoal/friction-ledger.jsonl → (no match)
```

**Disposition: nothing was harvested from this lane.** The lane did surface a harvestable
observation — the promoted-SHA `oya-ci-required` run dying on a 49-minute `oya-arm64` runner-queue
starvation with zero steps executed (§1c) — and it was never written to the ledger. Recorded here
as the disposition; the harvest itself remains outstanding.

What *does* exist is lane review evidence under `.grok/programs/cas-fabric/evidence/`:
`3a-nativelink-rehome.md`, `pr-1563-critic-a.json`, `pr-1563-critic-b.json`,
`pr-1563-dual-critic.json`.

---

## 7. Authority + no-secret constraints

### Dual-critic — VERIFIED APPROVE, both bound to the exact merged head

Both critic records carry `"head": "e8495b28fdfdbaf96a1b9d53909f68d985a60005"`, which matches the
merged head SHA exactly — the review is not stale relative to what merged.

| Critic | Lenses | Verdict | Blockers | severity_max |
| --- | --- | --- | --- | --- |
| A | cartesian_doubt, red_team | APPROVE | `[]` | low |
| B | contrarian, systems_thinking | APPROVE | `[]` | low |

### Authority chain — VERIFIED PRESENT, but every cited ADR is now Superseded

The bead's authority note cites ADR-0562 / 0614 / 0615 (plus ADR-0560 for the artifact). All four
now live in `docs/adr-archive/` with frontmatter `status: Superseded`:

| ADR | Status today | superseded_by |
| --- | --- | --- |
| ADR-0560 (NativeLink CAS slice 1) | Superseded | ADR-0700 |
| ADR-0562 (capability-first organization) | Superseded | ADR-0701 |
| ADR-0614 (de-commit move-manifest bijection) | Superseded | ADR-0701 |
| ADR-0615 (capability boundary rulings) | Superseded | ADR-0701 |

The bead's note required re-verifying the ADR table on each force-push **before merge**; at merge
time these were the governing Accepted decisions and the lane cited them correctly. The
supersession is later drift. I did **not** trace whether ADR-0700/ADR-0701 preserve the
`storage/` capability-first placement that Lane 3A implemented — see `unobtainable`. The rehome's
current authority basis is therefore **presumed-but-unconfirmed** under the successor ADRs.

### No-secret constraint — VERIFIED SATISFIED

Full scan of the 416-line moved manifest for secret material. Every hit is a **reference**, never a
value:

- Key material is projected via `ExternalSecret` → `ClusterSecretStore` from OpenBao
  (`oya/ci/nativelink-cas-tls`, KV v2), pulling `server-cert`, `server-key`,
  `writer-client-ca`, `reader-client-ca` by `remoteRef`.
- The manifest states its own invariant: *"Key material comes from OpenBao via ExternalSecret
  (below) — never from this manifest"* and *"No key bytes live in git."* Verified true by scan —
  no literal certificate, key, token, or password bytes appear.
- Defense-in-depth posture preserved byte-for-byte by the R100 rename: keyed mTLS boundary
  (writer CA `:50051` AC-rw, reader CA `:50052` AC `read_only=true`),
  `automountServiceAccountToken: false`, NetworkPolicy beneath, fork PRs receive no secrets.

Because both artifacts moved as `R100` byte-identical renames, the move **cannot** have altered the
security posture. No new secret surface was created.

---

## 8. What I could NOT verify, and why

1. **Promoted-SHA `oya-ci-required` SUCCESS — cannot be satisfied as written.** The run exists
   (`31009564936`) and its conclusion is `cancelled`. No amount of querying turns this green. It
   could only be satisfied by *re-running* CI against `010c132e`, which is a mutating forge action
   outside this task's authority (no push, no re-dispatch). **This alone sets `criteria_met=false`.**
2. **Root cause of the cancellation beyond runner starvation.** I established the job never
   acquired an `oya-arm64` runner (empty `runner_name`, zero steps, 49-minute queue). I could not
   establish *who or what* issued the cancel at `15:08:17Z` — the GitHub API exposes no cancelling
   actor for a run. The correlation with #1561's merge 25s earlier is noted and explicitly **not**
   claimed as cause, since per-SHA concurrency grouping rules out the obvious mechanism.
3. **Whether successor ADR-0700 / ADR-0701 preserve Lane 3A's placement.** All four cited authority
   ADRs are now Superseded. Confirming the successors still ratify
   `storage/adapters/nativelink/` requires reading both successor ADRs in full; I did not do that,
   so I do not assert current authority — only authority-at-merge.
4. **Rollback rehearsal.** Not executed; would require modifying files beyond the one file this task
   permits. Conflict risk was derived from history, not from an attempted revert.
5. **Live rollout / browser / user-story evidence.** Structurally unobtainable: the artifact is a
   dark, unreconciled Kubernetes manifest with `warm_reads_licensed=false` and no GitOps
   Application. There is no running system to observe and no user-facing surface to exercise. This
   is a property of the change, not a gap in my investigation.

---

## Summary

PR #1563 itself is **sound**: a clean R100 path rehome, dual-critic APPROVE against the exact merged
head, consumers correctly rewritten, no secrets introduced, no behavior change, head-SHA
`oya-ci-required` green 26 seconds before merge.

The bead nevertheless **fails its acceptance criterion**, for one reason that is real and one that
is inherited:

- The promoted SHA `010c132e` has **no** `oya-ci-required` SUCCESS — the run was `cancelled` after a
  gate job starved 49 minutes waiting on an `oya-arm64` runner. 10/11 jobs passed; the aggregate
  never emitted.
- Current state on `origin/dev` is not clean: a stale `infra/nativelink/` reachability row was
  reintroduced by **#1561** (a later, concurrent lane), and the executed move plan sits PARKED as
  though pending.

Neither residual defect belongs to Lane 3A. Both should be filed against their own lanes rather
than resolved by re-litigating this bead.
