# PR-drain playbook (review → fix → merge a PR queue)

Status: referenced guidance (gitignored-local). Promotion rule at bottom.
First data row: 2026-07-02 drain, session 01V1bLB3RysbmF22vYN7k1v5.

## 1. Preflight
- `git fetch origin dev` + fetch every PR branch. The canonical checkout may be a preserved dirty tree — never work in it; one scratch worktree per lane.
- Enumerate PRs: `gh pr list --json number,title,headRefName,mergeable` + per-PR `gh pr checks`. Classify every failing check from its ACTUAL run log before touching anything (`gh run view --log-failed`); check names lie, logs don't.
- Read the admission contract first: merge admission = PR body (5 H2 sections in order + `## Code Review` with reviewer/verdict APPROVE/resolved/deferred; see memory `pr-merge-admission-mechanics`). A recently merged PR body is the template.
- Identify excluded/protected PRs up front and never touch them (this drain: 1181/1182/1189).

## 2. Reviewer fan-out protocol
- One adversarial reviewer agent per PR, dispatched in parallel, READ-ONLY, with: how to fetch truth (`gh pr diff`, `git show origin/<branch>:path`), repo doctrine lenses, and a structured verdict format (VERDICT / WHAT-IT-DOES / FINDINGS w/ severity+file+fix / RESOLVED / DEFERRED).
- Extra lenses by surface: authz PRs get fail-closed + dep-direction + move-fidelity checks; tenant-isolation PRs get blast-radius (grep ALL sibling parsers/callers); fixture PRs get RED/GREEN falsifiability ("if the check were stubbed to return empty, does a test fail?").
- REQUEST_CHANGES → fixer agent in an isolated worktree with the findings as spec → push → the ORIGINAL reviewer re-verdicts at the new head. Never self-approve; never stamp APPROVE from narration.

## 3. Stamp-then-rerun procedure
- After an earned APPROVE: append `## Code Review` (reviewer-agent / verdict: APPROVE / resolved items / deferred items) via `gh pr edit --body-file`.
- The workflow triggers on `edited` too: the stamp itself spawns a fresh run. Branch protection judges the NEWEST check suite on the head — so after stamping, find the newest run (`gh api repos/:o/:r/actions/runs?head_sha=<sha>`) and let IT finish or `rerun --failed` IT. Rerunning an older run leaves the newest suite red and blocks automerge even though an older suite is green (bit #1202: three suites on one head, merge blocked until the newest went green).
- Post-#1196 (live-body preflight): reruns re-read the LIVE body — no commit needed. Proven end-to-end on #1202: RED (CodeReviewRequired 18:41Z) → stamp → newest-suite rerun → GREEN → merged 19:40Z, zero empty commits.
- Pre-#1196 fallback (or any workflow step still reading `github.event.*`): reruns see the frozen payload — mint a fresh event with an empty commit instead.
- Arm `gh pr merge --squash --auto` as soon as the APPROVE is stamped; it only fires on required-context green. If everything looks green but the PR stays BLOCKED, audit ALL check suites on the head (paginate! `gh pr checks` and default per_page truncate) for a red duplicate of the required context.

## 4. Serial-merge conflict protocol
- Sibling PRs appending to the same registration list (ADR-0515 evidence blocks, ADR-0562 path lists, Cargo.lock) conflict pairwise: expect the first merge to dirty the rest.
- On DIRTY/CONFLICTING: worktree → rebase onto dev → resolution is almost always keep-both-bullets → verify `git diff origin/dev --stat` equals exactly the PR's own delta → force-with-lease push.
- Expect non-fast-forward races on actively-shepherded branches (other workers push mid-fix): fetch + rebase + push, never force over unseen commits.

## 5. Settlement monitoring by run-ID
- Monitor CONCRETE run IDs (`gh run list --commit <head-sha>`), never `gh pr checks` buckets — buckets mix check-runs from cancelled/superseded attempts and read stale-fail as settled (bit us twice).
- A job inside an in-progress run can already be failed — pull its log via the job id without waiting for the run.
- New-file CI failures decode as: `unjustified` regression = missing exact-path ADR anchor (fix: ADR-0562 born-accounting list); new top-level dir = root-hygiene/capability-membership (fix: relocate, never allowlist); generated-face modification = close or drop the face (delete-only is legal).

## 6. Post-mortem metrics (row per drain)

| Metric | 2026-07-02 row |
|---|---|
| PRs processed | 7 (6 merged, 1 closed-justified) |
| Verdicts | 4 APPROVE / 2 REQUEST_CHANGES / 1 RECOMMEND_CLOSE; 0 overturned |
| Defects found+fixed | ~19 across 2 PRs + 2 found by CI class-decode (accounting anchors ×2) |
| CI-signal misreads | 2 (stale check-bucket read as settled; rerun cancelled fresh edit-run) |
| Wall-clock | ~90 min first-triage→last-merge; 14 agent passes, max 7-way parallel |
| Defect classes | doc self-contradiction after rename; missing accounting anchors; illegal top-level dir; non-hermetic fixture loading; missing RED tests; branded ids; deprecated dep for new data; dangling version pin; hand-maintained hash drift; stale pointers |
| Follow-ups | live-body preflight (shipped externally as #1196); drift-repair controller card; CRATEADR harness card |

## 7. Promotion rule
This stays referenced guidance. If a SECOND drain session actually consumes this playbook, promote it to a governed `/specs/` contract with an anchoring ADR — with the reuse evidence in hand. Do not pre-build the gate.
