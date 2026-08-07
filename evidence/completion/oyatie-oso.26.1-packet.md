---
doc_class: Completion-Receipt
doc_status: published
authority_tier: 3
purpose: |
  Closure receipt for bead oyatie-oso.26.1 — independent review of the final merged
  content of PR #1570 (oya-ci-required cancel-in-progress docs + Tide/merge_group
  safety), patch-equivalence, post-review metadata delta, and promoted-SHA gate evidence.
---

# Completion packet — oyatie-oso.26.1

**Bead:** `oyatie-oso.26.1` — "Review final PR #1570 metadata change and publish closure receipt"
**PR:** [#1570](https://github.com/jason931225/oyatie/pull/1570) — `ci(oya-ci-required): ADR-0639 D6 cancel-in-progress docs + Tide/merge_group safety`
**Author / merged by:** jason931225 · **Base:** `dev` · **Head branch:** `agent/ci-cancel-in-progress-20260805`
**Reviewed by:** this packet's author, independently, against the merged trees (not against the pre-existing critic packets).
**Receipt date:** 2026-08-07

## 1. Identity of the merged artifact

| Fact | Value |
|---|---|
| PR head SHA (subject of the required check) | `574bb16365c8e07650c80c93fd11e9ba7709956b` |
| Promoted / squash-merge SHA on `dev` | `360367b4e848d8667223c5b15e60017143f50830` |
| `mergedAt` | `2026-08-05T20:37:34Z` |
| State | MERGED |
| Files changed | 7 (`+331 / -10`) |

Derived with:

```
gh pr view 1570 --repo jason931225/oyatie \
  --json number,title,state,mergedAt,mergeCommit,headRefOid,baseRefName,headRefName,mergedBy,files
```

## 2. `oya-ci-required` gate evidence (check-run, app_id 15368)

`oya-ci-required` is a **check-run**, not a legacy status context. Queried on the PR **head** SHA
(the commit statuses endpoint returns `contexts=0` for check-runs by design, and a squash merge
commit does not inherit the PR's checks):

```
gh api repos/jason931225/oyatie/commits/574bb16365c8e07650c80c93fd11e9ba7709956b/check-runs \
  --jq '.check_runs[]|"\(.name)|\(.conclusion)|\(.completed_at)|app=\(.app.id)"'
→ oya-ci-required|success|2026-08-05T20:37:07Z|started 2026-08-05T20:37:05Z|app=15368
  https://github.com/jason931225/oyatie/actions/runs/31040344390/job/92437964542
```

**Join:**

| Event | Timestamp (UTC) |
|---|---|
| `oya-ci-required` `completed_at` (head SHA) | `2026-08-05T20:37:07Z` |
| `mergedAt` | `2026-08-05T20:37:34Z` |
| **Margin (completed → merged)** | **+27 s** — the required check completed **before** merge. |

The ordering is correct: the gate did not land after the merge.

### Promoted-SHA (post-merge) gate evidence — required by the post-merge product gate

The promoted commit on `dev` carries its own independent `oya-ci-required` run:

```
gh api repos/jason931225/oyatie/commits/360367b4e848d8667223c5b15e60017143f50830/check-runs
→ total_count=19
→ oya-ci-required|success|2026-08-05T21:03:57Z|app=15368
```

So both the pre-merge head SHA **and** the promoted trunk SHA are green on the single required context.

### Non-required lane observation (reported, not a claimed defect)

`gate · platform smoke (windows-amd64 soft)` concluded `failure` on **both** the head SHA
(`2026-08-05T19:44:40Z`) and the promoted SHA (`2026-08-05T20:43:40Z`). It is a **soft** lane and is
not the required context; it did not gate admission, and it is pre-existing on both SHAs rather than
introduced by this PR. Recorded here as a fact, not diagnosed — diagnosing the windows-amd64 soft
lane is outside this bead's scope.

## 3. Acceptance criteria — what was required, what was verified

### AC-1 — "Independent review covers the exact final merged content"

Verified against the merged trees directly, not against the earlier critic packets.

The **entire** executable delta in the merged commit is in `.github/workflows/oya-ci-required.yml`,
and it is **comment-only**. The three executable lines are unchanged context in the merge diff:

```
concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

Confirmed with `git show 360367b4 -- .github/workflows/oya-ci-required.yml`: every `+`/`-` line in
that hunk pair is a `#` comment line; `concurrency:`, `group:` and `cancel-in-progress:` appear as
context (unprefixed) lines. The already-live cancel-in-progress behaviour was therefore **not**
reopened, exactly as the bead required.

Independently re-checked on current trunk — the block is byte-identical today:

```
git show origin/dev:.github/workflows/oya-ci-required.yml | grep -A3 '^concurrency:'
107:concurrency:
108:  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
109:  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

Two invariants asserted by the PR body were checked and hold in the merged content:

- **#1509 non-PR key preserved** — the non-PR fallback is `github.sha`, not `github.ref`. The
  measured trunk pending-eviction class is not reopened.
- **Tide / merge_group isolation** — `cancel-in-progress` evaluates false for anything other than
  `pull_request`; `pull_request.number` is empty on `merge_group`, so a merge-queue run falls back to
  `github.sha` and shares no group key with open-PR `synchronize` runs.

The other six merged files are documentation, ownership and accounting registration:
`docs/programs/hyperscaler-delivery-lanes/OWNERS` (+1), `.../WAVE4-CANCEL-IN-PROGRESS-TIDE-NOTE.md`
(+156), `evidence/ci/pr-1570-critic-a.json` (+43), `evidence/ci/pr-1570-critic-b.json` (+43),
`evidence/ci/pr-1570-dual-critic.json` (+62), `specs/reachability-registry.json` (+4). The
reachability prefix `docs/programs/hyperscaler-delivery-lanes/` is still present on current `dev`
(`specs/reachability-registry.json:473`), so the accounting registration survived promotion.

### AC-2 — "Patch-equivalence and the post-review metadata delta are documented"

**Patch-equivalence, PR head → promoted commit.** Strongest available form:

```
git diff --stat 574bb16365c8e07650c80c93fd11e9ba7709956b 360367b4e848d8667223c5b15e60017143f50830
→ (empty)
```

The promoted squash-merge tree is **byte-identical** to the reviewed PR head tree. Nothing was
introduced between the last CI-verified head and what landed on `dev`.

**Post-review metadata delta.** The dual-critic packets record `subject_head` /
`head = 667d1fa3e5e3ffbdd4f10d61a5841e551e1cf891` — a pre-rebase commit that is not in the final PR
commit list (the branch was rebased; all five final commits share `committedDate 2026-08-05T19:38:13Z`).
Restricting the reviewed-subject → final-head diff to the seven paths this PR touches:

```
git diff --stat 667d1fa3 574bb163 -- .github/workflows/oya-ci-required.yml \
  docs/programs/hyperscaler-delivery-lanes/ evidence/ci/pr-1570-*.json specs/reachability-registry.json
→ docs/programs/hyperscaler-delivery-lanes/WAVE4-CANCEL-IN-PROGRESS-TIDE-NOTE.md |  8 +++
  evidence/ci/pr-1570-critic-a.json                                              | 43 +++++
  evidence/ci/pr-1570-critic-b.json                                              | 43 +++++
  evidence/ci/pr-1570-dual-critic.json                                           | 62 ++++++
  4 files changed, 156 insertions(+)
```

Two findings, both benign:

1. **`.github/workflows/oya-ci-required.yml` does not appear in this diff** — the executable
   workflow content the critics reviewed is identical to the content that merged. This is the
   load-bearing patch-equivalence claim and it holds.
2. **The only non-evidence post-review delta is 8 lines of YAML frontmatter** on the Wave4 design
   note (commit `574bb163`, "declare doc_status on Wave4 design note"):
   `doc_class: Program-Design-Note`, `doc_status: published`, `authority_tier: 3`, `purpose: |…`.
   It was added to clear the `doc-status-lifecycle/stage_not_declared` ratchet
   (1921→1922), mirroring the `docs/programs/k8s-port` pattern. It is document metadata only:
   non-executable, no behaviour, no gate semantics.

   The three `evidence/ci/pr-1570-*.json` entries in the diff are the critic packets themselves —
   necessarily absent from the tree they reviewed, and self-describing (`"dual-critic.head is the
   accounting-fix subject commit; PR tip includes this evidence package"`).

**Conclusion:** the post-review delta is metadata-only. The reviewed content and the merged content
are equivalent on every executable and semantic surface.

### AC-3 — "Promoted-SHA gate evidence is linked"

Linked in §2: promoted SHA `360367b4e848d8667223c5b15e60017143f50830`, `oya-ci-required` =
`success` at `2026-08-05T21:03:57Z` (19 check-runs total on the promoted commit). Pre-merge head-SHA
run linked to job `92437964542` under workflow run `31040344390`.

### AC-4 — "Durable closure receipt exists"

This file, `evidence/completion/oyatie-oso.26.1-packet.md`, committed to the repository.

### AC-5 — "Any defect becomes a separate linked issue"

**No defect found.** The merged content matches its stated intent; the executable concurrency is
unchanged; the #1509 `github.sha` non-PR key and Tide isolation both hold; the accounting
registration survives on trunk; the required context is green on both the head and the promoted SHA.
This criterion is therefore vacuously satisfied — no issue was filed because there is nothing to
file. Nothing was fabricated to populate it.

One **currency note**, not a defect in the PR: the merged docs cite `ADR-0639 D6` as authority. D6
exists and says what the docs claim ("The workflow MAY use a per-PR concurrency group with
`cancel-in-progress: true` … Cancel of a superseded attempt is not a merge-green substitute"). At
merge time (2026-08-05) ADR-0639 was Accepted. It has **since** been archived — it now lives at
`docs/adr-archive/ADR-0639-path-event-optional-legs-under-oya-ci-required.md` with
`status: Superseded`, `superseded_by: [ADR-0700]`, and a HISTORICAL/NON-AUTHORITY banner dated
2026-08-06, i.e. the day *after* this merge. The citation was correct when written; anyone re-reading
the Wave4 note today should follow the redirect to ADR-0700…ADR-0709. That is a docs-currency
follow-up for the note's owner (`cloud-ci-platform`), not a defect in PR #1570.

## 4. What could NOT be verified

- **Reviewer-agent APPROVE on the exact final head.** The dual-critic packets
  (`evidence/ci/pr-1570-*.json`, verdict APPROVE, `reviewed_at 2026-08-05T18:30Z`) are anchored to
  pre-rebase subject `667d1fa3…`, not to the merged head `574bb163…`. I established
  content-equivalence between them (§AC-2) and performed my own independent review of the merged
  content (§AC-1), but I cannot produce an APPROVE record whose recorded SHA is the merged SHA. No
  GitHub PR review objects exist either: `gh pr view 1570 --json reviews,reviewDecision` returns
  `reviews: []` and an empty `reviewDecision`.
- **`mergeStateStatus`** returns `UNKNOWN` post-merge; the merge-queue projected state at admission
  time is not recoverable from the API after the fact. The `completed_at < mergedAt` join in §2 is
  the ordering evidence that is actually obtainable.
- **The windows-amd64 soft-lane `failure`** is reported as observed (§2) but not diagnosed —
  out of scope for this bead, and I will not guess at a cause.
- **Rollout / rollback / browser-user-story evidence** from the broader post-merge product gate is
  not covered here; this bead scopes to review of the final metadata change plus the closure
  receipt. No claim is made about those.

## 5. Verdict

All five acceptance criteria of `oyatie-oso.26.1` are satisfied by evidence obtained and reproduced
above. The merged change is documentation- and accounting-only; the live cancel-in-progress behaviour
was not reopened; the required context was green 27 seconds before merge and green again on the
promoted trunk SHA.

### Reproduction commands

```
gh pr view 1570 --repo jason931225/oyatie --json number,state,mergedAt,mergeCommit,headRefOid,files
gh api repos/jason931225/oyatie/commits/574bb16365c8e07650c80c93fd11e9ba7709956b/check-runs \
  --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
gh api repos/jason931225/oyatie/commits/360367b4e848d8667223c5b15e60017143f50830/check-runs \
  --jq '.check_runs[]|select(.name|test("oya-ci-required"))|"\(.conclusion) \(.completed_at)"'
git diff --stat 574bb16365c8e07650c80c93fd11e9ba7709956b 360367b4e848d8667223c5b15e60017143f50830
git show 360367b4e848d8667223c5b15e60017143f50830 -- .github/workflows/oya-ci-required.yml
git diff --stat 667d1fa3e5e3ffbdd4f10d61a5841e551e1cf891 574bb16365c8e07650c80c93fd11e9ba7709956b \
  -- .github/workflows/oya-ci-required.yml docs/programs/hyperscaler-delivery-lanes/ \
     evidence/ci/pr-1570-critic-a.json evidence/ci/pr-1570-critic-b.json \
     evidence/ci/pr-1570-dual-critic.json specs/reachability-registry.json
```
