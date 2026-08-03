# Approach B design — de-commit gate-baseline, regenerate frozen baseline from merge-base SOURCE

Founder authorized B (2026-07-09, "go with your recommendation"). This is the DESIGN, pre-code.
Gate before implementation: adversarial design review (hostile-security + cost/feasibility).

## Core idea

Today the emitter's `--merge-base-baseline` mode reads the frozen reference as a COMMITTED blob:
`git show <merge_base>:<frozen_policy.face_path>` (`scm-facts-snapshot/src/main.rs:738,921`).
That forces the face to stay committed (ADR-0596). B replaces that read with REGENERATION of the
baseline from the merge-base SOURCE tree.

**Why it's sound (answers ADR-0596's "history only exists for committed paths"):** de-committing
the generated FACE does not remove the SOURCE that produces it. The producer inputs (gate source,
`oya-ci.toml`, tracked-paths) remain committed history at the merge-base. The producer
(`artifact-inventory-registry/src/main.rs`) is a PURE function of `--repo-root` (no ambient git,
`main.rs:3-9`), so `HEAD-producer(merge-base source)` is deterministic and reads only immutable
merge-base history. Frozen content becomes `f(immutable merge-base source, HEAD producer code)`;
HEAD producer/emitter is review-visible code = the same trust tier ADR-0551 already relies on.

## Mechanism

Emitter `--merge-base-baseline` (the single git boundary) changes the FACE acquisition:
1. `merge_base = git merge-base <out-of-band bootstrap> HEAD`   [unchanged]
2. `frozen_policy = ratchet-policy.json @ merge_base` via resolver   [unchanged — frozen-policy-wins]
3. **NEW** frozen face = regenerate:
   a. materialize merge-base tree (`git worktree add <tmp> <merge_base>`, or `git archive|extract`)
   b. run emitter at `<tmp>` → merge-base scm-facts face (tracked_paths for that tree)
   c. run HEAD producer `--repo-root <tmp> --scm-facts <tmp-scm-facts> --stdout --face baseline`
   d. parse stdout → face
4. `build_merge_base_baseline_snapshot(frozen_policy, source, merge_base, face)`   [unchanged wrapper]

## Invariants preserved (must hold under review)

- **Fail-closed**: worktree/regeneration failure = HARD ERROR (never empty/candidate fallback);
  unresolvable merge-base = hard error (unchanged).
- **Frozen-policy-wins**: policy still read from merge-base via out-of-band bootstrap (unchanged).
- **Frozen-mode-wins**: modes/dispositions come from merge-base SOURCE (candidate cannot influence —
  regeneration reads `<tmp>`=merge-base only; producer=HEAD review-visible code).
- **Anti-laundering**: frozen ref is merge-base-derived, PR-uncontrollable (immutable source).

## Safe strangler order (never a dev-deadlock window)

- **PR-1 — add regenerate path, face STILL committed.** Emitter regenerates the frozen face from
  merge-base source AND transitionally cross-checks it against `git show <merge_base>:<face>`.
  Cross-check is KEYSET-parity (NOT byte — tolerates producer/`config_digest` evolution across the
  range); divergence beyond keyset = hard error. Proves regeneration ≡ the live committed reference.
  Face stays committed; zero policy change. RED/GREEN fixtures for regenerate correctness + fail-closed.
- **PR-2 — de-commit.** Remove the git-show cross-check (regeneration is sole source; code lands
  BEFORE the git rm so no window reads a de-committed blob). Flip
  `registry/generated-artifact-control-plane.json` gate-baseline → de-commit class; `git rm --cached`;
  `.gitignore` remove the `!` negation. **Invert the ADR-0596 gate**: rule changes from
  "frozen ref must stay COMMITTED" to "frozen ref must be REGENERABLE-FROM-MERGE-BASE-SOURCE
  (emitter must not `git show <merge_base>:<committed-face>`)". Convert registry-drift + freshness
  gate-baseline checks byte-parity→determinism-class (regenerate-twice), mirroring ADR-0604 scm-facts.
- **ADR**: supersede ADR-0596 + amend ADR-0604. Land Proposed with ratified positions
  (adr-accepted-must-propagate-gate); formal Accepted rides propagation.

## OPEN QUESTIONS for the adversarial design review

1. **Cost**: worktree/archive of the merge-base + a second emitter+producer run per CI lane. Big
   monorepo → expensive. Cacheable by merge-base tree hash (buck2 CAS)? Acceptable per-run cost, or
   does it need a shared materialized-once artifact (like the candidate scm-facts upload)?
2. **Producer-evolution drift**: HEAD-producer(merge-base source) ≠ the merge-base's OWN committed
   face when producer/`oya-ci.toml` changed across the range. Is KEYSET-parity the right transition
   check (PR-1), and post-de-commit is "both sides use HEAD keying" actually MORE correct than the
   old git-show (which compared merge-base's OLD keying vs HEAD's)? Confirm no laundering opened.
3. **Mode-provenance**: can a candidate PR influence which per-code modes the regenerated frozen
   baseline carries (frozen-mode-wins)? Trace that modes derive from merge-base source stamps only.
4. **Worktree as a new git surface**: `git worktree add` inside the emitter — hermeticity/cleanup/
   concurrency (parallel gate lanes)? Or centralize in the materializer (`generated-artifact-freshness`)
   which already orchestrates emitter+producer for the candidate side.
5. **Simpler alternative**: feed the producer a "virtual tree" (content provider backed by
   `git show <merge_base>:<path>`) instead of a physical worktree — bigger producer refactor but no
   checkout. Worth it, or is worktree fine?

## ADVERSARIAL DESIGN REVIEW (opus architect, 2026-07-09) — verdict SOUND-WITH-CHANGES

**The honest trust delta (my design doc OVERSTATED invariant-preservation):** `git show` gives EXACT
immutable frozen bytes — the frozen mode/keys of EXISTING codes are un-authorable by the evaluated PR.
B replaces this with `frozen = HEAD-producer(merge-base source)`, so the SAME PR's producer-code edit
recomputes its OWN frozen reference. Concrete new vector: a review-visible edit to `build_gate_baseline`
that (a) downgrades an existing code's frozen mode block-on-new→advisory, or (b) collapses key
granularity so new debt folds into a pre-existing key — `git show` structurally REDs both (immutable old
bytes / phantom regression); B passes both green. So B trades a STRUCTURAL guarantee → a REVIEW-VISIBILITY
guarantee on frozen mode/keys. B DOES preserve the CENTRAL anti-laundering property (frozen inputs =
immutable merge-base SOURCE, PR-uncontrollable) + frozen-policy-wins + frozen-mode-wins at the DATA level
(modes derive from merge-base `oya-ci.toml`). Producer is `--repo-root`-pure + ambient-git-free → regen is
mechanically sound, zero refactor.

**ADR-0613 does NOT generalize:** it de-commits by materialize-on-demand from the PRESENT candidate tree;
the frozen ref is read across the merge-base HISTORY boundary — can't be materialized from the present tree.
B is a genuinely new mechanism (materialize merge-base SOURCE + run producer there).

**Five required hardenings:**
- H1 — superseding ADR states the trust delta honestly (revised weaker-but-acceptable invariant), does NOT
  claim 0551/0596 preservation; supersede 0596, amend 0604, explicitly override 0613's same-day reaffirmation.
- H2 — steady-state tamper-evidence: after PR-2 there's no git-show tripwire; gate the census kernel
  (`build_gate_baseline` + disposition/key-format path) behind heightened-review/provenance, or a monotonic
  frozen-keyset-cardinality invariant. At min, name the ceiling in the ADR.
- H3 — PR-1 cross-check compares the FULL ratchet projection `{keys, mode, frozen_empty}` per (gate,code),
  NOT just keyset (keyset-parity misses a mode downgrade). Keyset-tolerance for config_digest byte noise is
  right otherwise. Land the 2 PRs on a range with NO producer key-format change.
- H4 — PR-1 proves blob-INDEPENDENCE (regen with the committed face HIDDEN/absent), not just equivalence;
  emitter makes regen the SOLE path, fail-closed, NEVER git-show fallback (else PR-2 → #828 empty-frozen deadlock).
- H5 — KEEP the existing `relabel_frozen_face` step (design omitted it); B's merge-base-tree keys are already
  in merge-base naming so the relabel applies unchanged — dropping it gives moved paths phantom regressions.

**Cost:** do NOT `git worktree add` per gate-leg (concurrency/cleanup/hermeticity). CENTRALIZE the frozen
regen in the existing materializer (`ci/facade/generated-artifact-freshness`) that already orchestrates
emitter+producer + uploads faces ONCE/run, downloaded by every leg; key by merge-base sha (buck2-CAS-friendly).
One regen/run, LESS new code than per-leg worktree. Use physical worktree/git-archive (reuses `--repo-root`),
NOT virtual-tree (big refactor, marginal benefit).

**A remains the lower-TRUST-risk option** and is ALREADY the manifest's declared mode
(`main-branch-materialized`/`controller-owned-main-materialization`, `registry/generated-artifact-control-plane.json:129-137`)
— A = "finish the unwired controller ADR-0596 specifies," not new design. The break (731fd5669) was an
INCOMPLETE A, not a refutation of A. B and A don't dominate: B buys north-star completion (no committed
surface, kills the relocation fragility that caused the keystone break, no main-writing controller); B sells
structural frozen-byte immutability → review-visibility + per-run regen cost. A buys structural trust +
declared machinery; A retains the last committed surface + needs a privileged dev-writing controller.

**SYNTHESIS (cleanest end-state):** de-commit the face (B) BUT have the materializer compute-AND-SIGN the
frozen snapshot with controller/merge-base provenance the candidate PR can't forge — restores "frozen
un-authorable by the evaluated PR" WITHOUT committing. Review rec: ship B + H1-H5 now, add the signer only if
a real laundering attempt surfaces (name the ceiling, don't build speculatively).
