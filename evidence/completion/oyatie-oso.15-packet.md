# Post-merge completion packet — oyatie-oso.15 (PR #1565, RR-FACE-DECOMMIT)

Author: evidence-packet lane agent. Compiled 2026-08-07. All forge facts re-derived in this
session with the commands quoted inline; nothing below is carried over from a prior claim.

**Verdict: criteria NOT fully met.** Acceptance criterion 1 is *unobtainable as literally written*
and is reported as such in [§7](#7-what-could-not-be-verified). Criteria 2–4 are met, with the
scoping caveats stated. Do not read this packet as a green light on criterion 1.

---

## 1. Identity

| Field | Value |
| --- | --- |
| Bead | `oyatie-oso.15` — RR-FACE-DECOMMIT verify executable reorg promotion packet |
| PR | [#1565](https://github.com/jason931225/oyatie/pull/1565) — `docs(reorg): RR-FACE-DECOMMIT residual inventory + stale prose (ADR-0613–0616)` |
| State | `MERGED` (squash), base `dev`, head branch `agent/rr-face-decommit-20260805` |
| PR head SHA | `8374a19bcd9bcb352d7bc794835fef1aa055ad0e` |
| Promoted / merge SHA | `5d62f6a364d1de9a13827a49a3b6197077bb0d27` |
| `mergedAt` | `2026-08-05T14:08:36Z` |
| Trunk tip at packet time | `3da3bb90930541ed2fbb66f9a68029d2faadebc2` (`origin/dev` == packet-lane `HEAD` at start) |

```
$ gh pr view 1565 --repo jason931225/oyatie --json number,state,mergedAt,mergeCommit,headRefOid,baseRefName
{"baseRefName":"dev","headRefOid":"8374a19bcd9bcb352d7bc794835fef1aa055ad0e",
 "mergeCommit":{"oid":"5d62f6a364d1de9a13827a49a3b6197077bb0d27"},
 "mergedAt":"2026-08-05T14:08:36Z","number":1565,"state":"MERGED"}
```

---

## 2. `oya-ci-required` — the join, on both SHAs

`oya-ci-required` is a **check-run** (app id 15368), not a legacy status context. Queried via
`/commits/<sha>/check-runs`, never via `/commits/<sha>/status` (that endpoint returns
`contexts=0` for check-runs by design and would have produced a false negative).

### 2a. PR head SHA — SUCCESS, 46 s before merge

```
$ gh api repos/jason931225/oyatie/commits/8374a19bcd9bcb352d7bc794835fef1aa055ad0e/check-runs \
    --jq '.check_runs[]|select(.name=="oya-ci-required")|"\(.conclusion) \(.started_at) \(.completed_at)"'
success 2026-08-05T14:07:47Z 2026-08-05T14:07:50Z
  html_url: https://github.com/jason931225/oyatie/actions/runs/31002825202/job/92327613665

$ gh api repos/jason931225/oyatie/actions/runs/31002825202 --jq '{event,head_branch,conclusion}'
{"conclusion":"success","event":"pull_request","head_branch":"agent/rr-face-decommit-20260805"}
```

**Join.** `completed_at = 2026-08-05T14:07:50Z` precedes `mergedAt = 2026-08-05T14:08:36Z`.

**Margin = +46 seconds** (CI finished 46 s before the merge). This is the correct ordering: the
merge did not race ahead of the required context.

All 12 check-runs on the head SHA (the fan-in plus its 11 constituent lanes) are `success`, except
`cache-writer identity (trusted dev push only)` which is `skipped` by design on a PR event.
Constituent lanes and conclusions: `gate · ADR census epoch receipt` success · `registry-drift`
success · `gate · affected-set (ADR-0554)` success · `gate-live-postgres-facades` success ·
`gate-live-postgres-adapters` success · `buck2 (hermetic build + affected gate tests)` success ·
`generated-output-diff-policy` success · `producer-regen (accounting-registry)` success ·
`freshness (lock + generated faces, ADR-0539)` success · `cloud-ci-firewall` success.

### 2b. Promoted merge SHA — NO `oya-ci-required` check-run exists; the run was CANCELLED

```
$ gh api repos/jason931225/oyatie/commits/5d62f6a364d1de9a13827a49a3b6197077bb0d27/check-runs \
    --jq '.check_runs[]|"\(.name) | \(.conclusion) | \(.completed_at)"'
cache-writer identity (trusted dev push only)          | skipped   | 2026-08-05T14:08:41Z
cloud-ci-firewall (baseline ratchet + meta-test)       | cancelled | 2026-08-05T15:08:16Z
freshness (lock + generated faces, ADR-0539)           | cancelled | 2026-08-05T15:08:16Z
gate · affected-set (ADR-0554)                         | cancelled | 2026-08-05T15:08:16Z
generated-output-diff-policy                           | cancelled | 2026-08-05T15:08:16Z
registry-drift (materialized == regenerated)           | cancelled | 2026-08-05T15:08:16Z
buck2 (hermetic build + affected gate tests)           | cancelled | 2026-08-05T15:08:16Z
producer-regen (accounting-registry)                   | cancelled | 2026-08-05T15:08:16Z

$ gh api repos/jason931225/oyatie/actions/runs/31013688156 \
    --jq '{event,head_branch,head_sha,conclusion,created_at,updated_at}'
{"conclusion":"cancelled","created_at":"2026-08-05T14:08:40Z","event":"push","head_branch":"dev",
 "head_sha":"5d62f6a364d1de9a13827a49a3b6197077bb0d27","updated_at":"2026-08-05T15:08:17Z"}
```

Two facts, stated plainly:

1. **There is no check-run named `oya-ci-required` on `5d62f6a3` at all.** `oya-ci-required` is a
   zero-command *fan-in* job that is green IFF every constituent lane is green; because the lanes
   were cancelled, the fan-in never ran and never published a check-run.
2. The eight constituent lanes that did get scheduled were cancelled 59 m 35 s after start, all at
   the same instant.

The bead's AUDIT REOPEN note ("promoted-SHA run 31013688156 was CANCELLED") is **confirmed
correct**.

Root cause of the cancellation is **NOT DETERMINED by this packet**. Two candidate explanations
exist in the repo and neither was proven here: (a) the trunk concurrency / pending-eviction class
documented at length in the `concurrency:` comment of `.github/workflows/oya-ci-required.yml` *as
of `5d62f6a3`* — which, per that comment's own measurement, was already keyed on `github.sha` per
trunk commit and so should NOT have evicted; and (b) whatever ADR-0639 D6 later addressed in
`360367b4e` (`ci(oya-ci-required): ADR-0639 D6 cancel-in-progress docs + Tide/merge_group safety`,
#1570). Attributing the cancel to either without a run-log read would be invention. See §7.

### 2c. Substitute (NOT equivalent) evidence — the promoted content is green on trunk

Labelled explicitly as substitute; it does **not** satisfy criterion 1.

```
$ git merge-base --is-ancestor 5d62f6a364d1de9a13827a49a3b6197077bb0d27 HEAD ; echo $?
0
$ git rev-parse origin/dev
3da3bb90930541ed2fbb66f9a68029d2faadebc2      # == HEAD of this packet lane at start

$ gh api repos/jason931225/oyatie/commits/3da3bb90930541ed2fbb66f9a68029d2faadebc2/check-runs \
    --jq '.check_runs[]|select(.name=="oya-ci-required")|"\(.conclusion) \(.completed_at)"'
success 2026-08-07T09:27:51Z
  run 31164028155, event=push, branch=dev
```

The first trunk `push` run to go green after the promotion was on `360367b4e` (#1570):

```
$ gh api repos/jason931225/oyatie/commits/360367b4e848d8667223c5b15e60017143f50830/check-runs \
    --jq '.check_runs[]|select(.name=="oya-ci-required")|"\(.conclusion) \(.completed_at)"'
success 2026-08-05T21:03:57Z      # run 31044962390, event=push, branch=dev
```

So the promoted commit is an ancestor of a trunk tip that carries a green `oya-ci-required`, and
of every green trunk run since. That is *current-state* verification. It is not exact-promoted-SHA
verification, because the trees differ.

---

## 3. What actually landed — and the load-bearing correction to the bead's premise

```
$ gh pr view 1565 --repo jason931225/oyatie --json files
   9+   6-  .gitignore
   1+   1-  ci/adapters/path-resolver/Cargo.toml
  15+  12-  ci/adapters/path-resolver/src/lib.rs
   5+   5-  ci/facade/crate-registration/src/lib.rs
  12+  11-  ci/facade/scm-facts-snapshot/src/lib.rs
   1+   1-  ci/ports/path-resolver/Cargo.toml
  19+  14-  ci/ports/path-resolver/src/lib.rs
 383+   0-  evidence/reorg/rr-face-decommit-residual-inventory-20260805.json
   1+   1-  registry/generated-artifact-control-plane.json
```

The bead's AUDIT REOPEN says the PR "changed executable Rust path-resolver/SCM/crate-registration
code." **At file granularity that is true. At behavior granularity it is false, and the difference
decides what regression evidence is owed.**

```
$ git show 5d62f6a364d1de9a13827a49a3b6197077bb0d27 -- '*.rs' \
    | grep -E '^[+-][^+-]' | grep -vE '^[+-] *(//|/\*|\*)' | wc -l
0
```

**Zero non-comment lines changed in any `.rs` file in the promotion.** Every added/removed Rust
line is a `//!`, `///`, or `//` comment. The two `Cargo.toml` edits (1 line each) are likewise `#`
comment lines inside the crate-description block — no version, dependency, feature, or metadata
key moved. Manual read of all four Rust diffs confirms the grep: the edits replace the phrase
"committed move-manifest" with "de-committed move-manifest (ADR-0614)" and correct one doc claim
that the path-resolver `load` fails *open* to identity on absence, when the shipped code fails
*closed*. No signature, literal, control-flow branch, or constant value changed.

Non-Rust content changes and their consumers:

- **`.gitignore`** (3 hunks): comment prose only in the de-commit-class blocks; **no pattern line
  added, removed, or negated**. Verified by reading the full diff — every changed line begins `#`.
- **`registry/generated-artifact-control-plane.json`**: exactly one string field,
  `final_tree_validation` on the SCM-facts artifact, rewritten to stop claiming gate-baseline is a
  committed face (ADR-0616 de-committed it). This field *does* have executable consumers, so it
  was checked rather than assumed:
  - `ci/facade/generated-artifact-policy/src/lib.rs:1424` — presence/non-empty check only; a
    rewritten non-empty string still satisfies it.
  - `ci/facade/generated-artifact-policy/tests/generated_artifact_control_plane.rs:229–237` — the
    only *content* assertion on this field, and it is scoped to the **ADR-census-epoch** artifact
    (required substrings `"selects the immutable revision"`, `"excluded from the squash-stable P2
    receipt core"`), not the SCM-facts artifact this PR edited. No other assertion site exists.
  This test target was run in §4 and passes 8/8 against the live manifest.

---

## 4. Targeted regression evidence (acceptance criterion 2)

Run in this packet worktree at `HEAD = 3da3bb909`, hermetically under buck2 (no cargo, no shell
harness). Faces were materialized first via the sanctioned boundary.

| Target | Result |
| --- | --- |
| `//ci/ports/path-resolver:ci-path-resolver-ports-unittest` | **1 passed**, 0 failed |
| `//ci/adapters/path-resolver:ci-path-resolver-adapters-unittest` | **18 passed**, 0 failed |
| `//ci/facade/crate-registration:ci-crate-registration-unittest` | **29 passed**, 0 failed, 1 ignored |
| `//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot-unittest` | **104 passed**, 0 failed |
| `//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot-integration` | **49 passed**, 0 failed |
| `//ci/facade/generated-artifact-policy:ci-generated-artifact-policy-unittest` | **74 passed**, 0 failed |
| `//ci/facade/generated-artifact-policy:ci-generated-artifact-policy-gate` | **8 passed**, 0 failed (after materialize) |

**273 tests passed, 0 failed, 1 ignored** across the exact four crates the promotion touched plus
the gate that consumes the edited manifest.

The one ignored test is named, not hidden: `ci/facade/crate-registration/src/tests.rs:1341`,
`#[ignore = "requires buck2 + the full candidate tree; run explicitly when buck2 is available"]`.
It is a pre-existing marker, not introduced by this promotion, and it is not a stub — but it did
not execute here, so it contributes no evidence.

**Scoping caveat, stated because it matters.** Two of these crates are no longer byte-identical to
the promoted tree; later commits changed them:

```
$ git diff --stat 5d62f6a3 origin/dev -- ci/adapters/path-resolver ci/ports/path-resolver
(empty — byte-identical)

$ git diff --stat 5d62f6a3 origin/dev -- ci/facade/crate-registration ci/facade/scm-facts-snapshot
 ci/facade/crate-registration/src/lib.rs          | 26 +++--
 ci/facade/crate-registration/src/tests.rs        |  8 +--
 ci/facade/scm-facts-snapshot/src/retirement.rs   | 66 +++++++++++--
 .../tests/snapshot_integration.rs                | 35 ++++----
```

So: the **path-resolver** results verify exactly the promoted code. The **crate-registration** and
**scm-facts-snapshot** results verify current trunk, which is a superset containing the promotion's
(comment-only) delta but also later work. Given the promotion's Rust delta is provably empty of
behavior, the mechanical proof in §3 is the primary evidence for criterion 2 and these runs are
corroboration, not the load-bearing artifact.

---

## 5. Generated-face ban, non-vacuously enforced (acceptance criterion 4)

**Ban holds — at the promoted SHA and now:**

```
$ git ls-tree -r --name-only 5d62f6a364d1de9a13827a49a3b6197077bb0d27 | grep -c 'generated\.json'
0
$ git ls-tree -r --name-only origin/dev | grep -c 'generated\.json'
0
```

**Non-vacuity was demonstrated, not asserted.** The live control-plane gate was run twice against
a clean worktree, and the first run *hard-failed on absence* — proving the gate reads the real
filesystem and cannot pass by finding nothing:

```
$ buck2 test //ci/facade/generated-artifact-policy:ci-generated-artifact-policy-gate
test result: FAILED. 6 passed; 2 failed
  live_firewall_frozen_reference_is_regenerate_from_source_adr_0616  FAILED
  live_generated_artifacts_are_declared_in_the_control_plane         FAILED
  panicked at .../generated_artifact_control_plane.rs:71:
  read .../ci/facade/artifact-inventory-registry/scm-facts.generated.json: No such file or directory
  "This is a GENERATED face (ADR-0604 de-commit class): it is not tracked in git and is absent
   in a clean worktree. Materialize it, then re-run this gate."

$ buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
BUILD SUCCEEDED - starting your binary

$ buck2 test //ci/facade/generated-artifact-policy:ci-generated-artifact-policy-gate
test result: ok. 8 passed; 0 failed
```

That RED→materialize→GREEN transition is the non-vacuity proof: the face genuinely is not in git,
the gate genuinely reads it, and absence is fail-closed rather than silently identity/green. It is
also the direct executable counterpart of the doc correction this PR shipped ("the path-resolver
`load` is FAIL-CLOSED on absence").

After materialization the working tree remained clean of tracked changes (`git status --short`
returned nothing), confirming the materialized faces are gitignored and did not re-enter git.

---

## 6. Post-merge product gate items

**Rollout / current-state verification.** The promotion is an ancestor of `origin/dev` tip
`3da3bb909`, which carries `oya-ci-required` = `success` (run 31164028155, `push`, completed
2026-08-07T09:27:51Z). The path-resolver crates on trunk today are byte-identical to the promoted
tree. There is no service rollout dimension: the artifacts are CI gate crates and repo policy
data, not a deployed workload — nothing was released to a cluster by this merge.

**Rollback.** Reverting `5d62f6a3` is low-risk and mechanical: `git revert -m 1 5d62f6a3`. Blast
radius is bounded by §3 — reverting restores stale doc prose (re-asserting that move-manifest and
gate-baseline are committed faces, and that the path-resolver load fails open), restores one
`final_tree_validation` string that no test asserts on, and deletes one evidence JSON. **It changes
no executable behavior**, because the promotion changed none. No data migration, no schema change,
no consumer to coordinate. Rollback would *reintroduce* the documentation/implementation
contradiction ADR-0614 shipped against, so it is not recommended absent a specific defect.

**Observability.** The observable surface for this change class is the gate check-run stream
itself. Post-merge observation is what produced this packet's central negative finding: the
promoted SHA has no `oya-ci-required` verdict. That is a *gap in the observability of the trunk
promotion path*, not merely a stale run — a cancelled promoted-SHA run leaves no signal that
anything is missing, and nothing alerted on it for two days. Recorded as a finding; see §7 item 2.
No runtime metrics, logs, traces, or SLOs are in scope: no service was touched.

**Release impact.** None. No release configuration is live in this repo — no `release-please`
config and no release workflow exists under `.github/workflows/`. Per repo policy, Release Please
applies only when a live config/workflow exists, so no release note, version bump, or changelog
entry is owed. The change ships no public API, CLI surface, or user-visible behavior.

**Observation-harvest disposition.** Three observations harvested; all are recorded here and none
is silently dropped:
1. *A cancelled promoted-SHA run is invisible.* Merge admission correctly gated on the exact head
   SHA, but the trunk verdict for the promoted SHA was never produced and nothing noticed. Worth a
   follow-up bead (trunk-promotion verdict liveness), filed by the orchestrator — this lane is
   scoped to evidence and does not create or mutate beads.
2. *"Touched an executable file" ≠ "changed executable behavior."* The audit reopen was right to
   flag the file set and wrong about the delta. The cheap discriminator — count non-comment
   changed lines — should be the first step of any docs-only-claim audit, before regression work is
   scoped.
3. *The live gate's absence error is a good failure.* Its message names the exact materialize
   command, which is why the local RED was resolved in one step instead of being misread as a
   regression. This is the pattern other gates should copy.

---

## 7. What could NOT be verified, and why

1. **Acceptance criterion 1 is NOT met and is unobtainable by this lane as literally written.**
   The criterion demands that "exact promoted merge SHA `5d62f6a3` has `oya-ci-required` SUCCESS."
   It does not. There is no check-run of that name on that SHA in any state, and the workflow run
   that would have produced one (31013688156) is `cancelled` with 7 of its 8 lanes cancelled.
   Satisfying this literally requires re-running `oya-ci-required` on `5d62f6a3` (e.g.
   `gh run rerun 31013688156`), which mutates forge state and is outside this lane's authority —
   this lane is read-only against the forge by instruction. It is also not guaranteed to succeed on
   re-run: that workflow's candidate-tree materializer and event parser fail closed on event
   tuples they do not recognise, and a two-day-old re-run is not the same evaluation the original
   push would have been. **Do not treat §2c as satisfying this criterion.**
2. **Why run 31013688156 was cancelled is not established.** I recorded the observable facts
   (event=push on dev, all lanes cancelled at the same instant 59 m 35 s in) and deliberately did
   not assign a cause. I did not read the run logs or the runner-fleet state for that window, and
   the two available hypotheses (trunk concurrency eviction; whatever #1570/ADR-0639 D6 fixed)
   would both be speculation from where I stand.
3. **Runtime/production behavior was not exercised.** Evidence here is buck2 unit/integration/gate
   tests plus mechanical diff analysis. No end-to-end CI run was performed against the promoted
   tree from this lane, and no cluster, browser, or user-story evidence exists — nor is any
   applicable, since the promotion deploys nothing.
4. **`crate-registration` and `scm-facts-snapshot` were tested at trunk, not at `5d62f6a3`.** See
   the §4 caveat. Their sources have since changed. A verification of the *exact* promoted tree for
   those two crates would require checking out `5d62f6a3` and re-running, which this lane did not
   do (the promotion's Rust delta being provably comment-only made it low value; that is a
   judgement, and it is stated so it can be overruled).
5. **`#[ignore]`d test at `crate-registration/src/tests.rs:1341` did not execute.** Pre-existing,
   not introduced here, but it contributes zero evidence and is not counted as passing.
6. **Dual-critic APPROVE on the draft PR was not re-verified.** The bead notes it; I did not
   re-derive it from the forge, because the acceptance criteria do not require it and I did not
   want to restate an unverified claim as though I had checked it.

---

## 8. Bottom line

| Criterion | Verdict |
| --- | --- |
| Exact promoted SHA `5d62f6a3` has `oya-ci-required` SUCCESS | **NOT MET — unobtainable here.** No such check-run exists; run 31013688156 cancelled. Pre-merge exact-head success (+46 s margin) and descendant-trunk green offered only as substitute. |
| Targeted regression evidence for path-resolution / SCM / crate-registration behavior | **MET.** Zero non-comment `.rs` lines changed (mechanical proof); 273 buck2 tests pass across all four crates + the manifest gate. Scoping caveat in §4. |
| Durable packet: rollout/current-state, rollback, observability, release impact, observation-harvest | **MET.** §6. |
| Generated-face ban remains non-vacuously enforced | **MET.** 0 tracked `*.generated.json` at both SHAs; gate proven live by RED-on-absence → GREEN-after-materialize (§5). |

**Overall: `criteria_met = false`**, on criterion 1 alone. The promotion itself carries no
behavioral risk that this packet could find; the unmet criterion is an evidence gap in the trunk
promotion path, not a defect in the promoted change.
