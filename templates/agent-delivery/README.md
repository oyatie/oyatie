# Workflow suite

Five canonical workflows. Each is self-contained (workflow scripts cannot `import`), so shared
doctrine is duplicated deliberately rather than factored out — a workflow that depends on a file it
cannot import is a workflow that breaks silently.

| workflow | input | question it answers |
|---|---|---|
| **`deliver`** | a goal or bead | *build it* — scout, map, trial, scale, converge, claim, land ONE pull request per `integ/<root>` |
| **`verify`** | a PR, branch or working tree | *is this change right* — review, QA, GitOps drift, maintainability |
| **`audit`** | the merged codebase | *what is already wrong* — in capability chunks, filed to the backlog |
| **`restack`** | the base moved | *make every open PR follow it* — rebase, re-derive pinned numbers, push |
| **`respond`** | someone else's findings | *disposition them* — accept, refute with evidence, or leave honestly open |

They compose: `deliver` builds and lands, `verify` judges before merge, `audit` finds what neither
looked at because it was already merged, and `restack` runs immediately after anything merges —
it is the second half of merging, not optional cleanup.

`restack` earns its own file by the input test: its input is "the base moved", which none of the
other three take. Measured cost of not having it: a stale base fails HARD rather than degrading —
one commit behind red-lights eight checks, so the symptom reads as a broad code failure rather than
"you are one commit behind". Observed four times in three days, each costing a CI round trip plus
diagnosis, and quadratic in open PRs because `maxRunners=1` serialises every re-run.

---

## Why five and not one

A monolith would force a barrier where none belongs. Their **inputs differ** (a goal, a diff, a
tree, a moved base), their **cadence differs** (per task, per PR, periodic, per merge), and their
**failure modes differ**. `verify` must be fast enough to run before every merge; `audit` is
deliberately slow and budgeted; `restack` must run within minutes of a merge or the cost compounds.
Fusing them would make the fast ones wait on the slow one.

**The split is by INPUT, not by topic**, and that is the test for adding a sixth: if it takes an
input none of these five take, it is a new workflow; if it takes the same input, it is a lens inside
an existing one. `maintainability` is a lens in `verify` rather than a workflow for exactly that
reason, while `restack` is a workflow because "the base moved" is an input nothing else consumes.

`respond` passes the same test from the other direction: `verify` PRODUCES findings, and nothing
CONSUMED externally-authored ones. That gap was load-bearing — six PRs sat green-but-blocked on
`required_conversation_resolution` with dozens of unresolved threads, which no other workflow
touches. Its discipline is symmetrical: blanket-accept ships a defect to unblock a merge and
resolves the thread that would have caught it, while blanket-refuse closes a real finding with a
confident paragraph. Refuting is first-class and carries the same evidentiary standard as fixing.

---

## Shared invariants

Every workflow here obeys these. They are not style; each was paid for.

**Evidence over assertion.** A claim without a runnable command is not a result. buck2 output must
include its `Commands:` line — a run reporting `Commands: 1` is a cache-only no-op that proves
nothing. Measured: a stale daemon once reported 276 against a true 338.

**Adversarial by construction.** Findings face refuters told to kill them; implementations face
counterparts who see only the diff. An explanation is precisely what stops a reviewer seeing the
bug, so reviewers do not get the author's reasoning.

**Counterparts, not clones.** Redundancy catches the same class twice; diversity catches different
classes. Reviewer slots are bound to named subsets of the repo's 16 reasoning lenses
(correctness, operability, security, contract, maintainability, economics) so coverage is auditable
— you can ask which lens went unapplied.

**Prevent, then catch locally, then PR.** A defect that reaches CI costs a 30–70 minute round trip
to learn what a local gate answers in seconds. Nothing opens a pull request while red.

**One PR per run.** Units commit to a single integration branch. Measured cost of the alternative:
23 draft PRs for one logical change, every one stale and conflicting, all superseded by a single
consolidation PR. `deliver` also refuses to start when the forge is already over budget.

**Honest gaps.** "I could not determine this" is a success. Silence about what was not covered reads
as coverage that was never achieved, and every schema here has a required field for it.

**Negatives must be provable.** Before asserting "there are no X", show the pattern can match
something. `git grep -E "\b..."` uses POSIX ERE, which has no word-boundary atom — it matches
nothing, silently, `rc=1`, indistinguishable from a true negative.

**The suite is subject to its own rules.** `audit` is budgeted because an audit that files unbounded
findings *is* the accretion problem. Every check added to the lane runner ships with a preflight case
proving it fires — a rule that cannot be shown to fire is the false green it exists to prevent.

**Hard gates before forge paths:**
- `node templates/agent-delivery/auth-preflight.mjs` — required before push/merge/babysit/restack (`preflight.mjs` runs this automatically).
- Equality-pinned census merge protocol — mandatory re-derive after rebase even when git auto-merges clean (oyatie-o90; see `deliver.js` / `restack.js`).
- Two-round rule — same failure class twice → fix process/oracle, not output (`deliver.js` Converge).
- Claim phase (ADR-0711) — envelope verify + `git merge-tree` preflight + hub exclusivity before any integ push (`deliver.js` Claim; policy in `specs/integ-branch-envelopes.json`).
- Land upsert — at most one open PR per durable `integ/<root>`; after squash-merge, server-side reset via `git push --force-with-lease origin origin/dev:refs/heads/integ/<root>` (no local `git reset`).
- Bead counters are not live state — prove merge/remaining work with `git merge-base` / `gh pr view` before claim or close.

---

## Provenance

Mechanisms and where they came from, so a future reader can judge whether they still apply.

**From Bun's Zig→Rust rewrite** (`https://bun.com/blog/bun-in-rust`), adopted as *discipline*, not
mechanics:
- write the pattern mapping BEFORE any code — this is what let 64 parallel agents converge instead
  of diverge, and it is `deliver`'s Map phase
- trial on 2–3 units and prove the loop before spending on scale
- 1 implementer : 2+ adversarial reviewers, reviewers seeing only the diff
- code needing a paragraph-long justification is wrong until simplified — the comment is the smell
- errors as the work queue, sharded
- tests written so they cannot be co-broken with the thing they test
- no `git stash`/`reset`; commit named files; no slow commands in the inner loop
- "everything all at once" — an incremental rewrite adds temporary code you only hope gets deleted

What did **not** transfer: Bun's compiler-error work queue assumes one compilation unit and a
mechanical source-to-source mapping. Neither holds for general delivery, so `deliver` uses gates and
tests as the queue instead.

**From this repo, measured:**
- PR #1620 took 48 commits: 16 of actual work, 15 lane merges, 6 hotfile re-anchors, 10 repairs.
  67% overhead. Lane merges and 5 of 6 re-anchors are pure ceremony and are designed out; 6 of the
  10 repairs were locally catchable and are now preconditions.
- Five PRs shipped red on one equality-pinned corpus census that fires on any added or deleted file.
- 7 of 9 lifecycle lanes enforce nothing — a gate whose scan root is moved away does not fail, it
  observes zero artifacts and reports green.
- ADRs declare an end-of-life and 95% have retired; docs mostly do not and 5,351 accumulate.
  **Sprawl is caused by adding things with no defined end-of-life, not by adding things.**
- Project hooks referenced their scripts by relative path, so every command run from a worktree
  silently skipped them — including the no-cargo guard and the trust-boundary scanner. A guard that
  cannot run fails open and looks identical to one that is not needed.

---

## Polishing them

These are meant to accrete evidence, not features. When something goes wrong that one of these
should have caught:

1. Find the **class**, not the instance. One fix that removes a class beats N symptom fixes, and
   this repo has measured that repeatedly.
2. Add the check where it fires **earliest** — preferably in `lane-runner.js` where it blocks before
   a push, not in a workflow prompt where it depends on an agent reading carefully.
3. Add a preflight case proving it fires, driven by the real shape that broke. `lane-runner.test.mjs`
   must print ALL PASS before any dispatch.
4. Record the measurement in the comment. A comment recording WHY survives a rewrite; one explaining
   WHAT does not, and should be deleted in favour of clearer code.
5. Then ask the maintainability question of your own addition: what deletes this check? A rule that
   has never fired is a candidate for removal, and process that only ever accretes is the failure
   mode these workflows exist to catch.
