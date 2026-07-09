---
id: ADR-0614
title: "De-commit the firewall frozen-reference baseline — regenerate it from merge-base SOURCE (reverses ADR-0596)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-09
door: one-way
owner: council-architecture
supersedes: [ADR-0596]
superseded_by: []
amends: [ADR-0604]
depends_on: [ADR-0515, ADR-0539, ADR-0551, ADR-0552, ADR-0595, ADR-0596, ADR-0604, ADR-0613]
related: [ADR-0111, ADR-0363, ADR-0558, ADR-0563]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0614: De-commit the firewall frozen-reference baseline

## Status

**Proposed - 2026-07-09.** Reverses ADR-0596 ("frozen references must stay committed") and overrides
ADR-0613's same-day reaffirmation of it. Authored for founder sign-off; **records an HONEST trust delta**
(see Decision §Trust) rather than claiming ADR-0551/0596 invariant-preservation. Door: one-way.

## Context

`gate-baseline.generated.json` is the firewall ratchet's FROZEN reference and the LAST committed generated
merge-surface (ADR-0604/0613 de-committed every other face). The emitter reads it as the committed blob at
the merge-base — `git show <merge_base>:<face_path>` (`ci/facade/scm-facts-snapshot/src/main.rs:737`). ADR-0596
forbade de-committing it because the ratchet reads HISTORY, which only exists for committed paths; ADR-0613's
"materialize from the PRESENT candidate tree" pattern does NOT generalize to a merge-base history read.

But keeping it committed is the root of a recurring fragility: a capability MOVE must relocate the committed
face, and the #828→#830 deadlock + the 2026-07-09 keystone break both stemmed from committed-face
staleness/relocation (byte-identical relocate deferred content re-keying to an unwired postsubmit controller).

**Key realization:** de-committing the generated FACE does not remove the SOURCE that produces it. The producer
(`ci/facade/artifact-inventory-registry`) is `--repo-root`-pure and ambient-git-free (`src/main.rs:1-9`), so the
frozen baseline can be REGENERATED deterministically from the merge-base SOURCE tree (committed history) instead
of read from a committed generated blob.

## Decision

1. **De-commit `gate-baseline.generated.json`** (manifest `not-tracked-in-git`; `git rm --cached`; `.gitignore`
   drop the `!` negation). Convert its registry-drift + freshness checks from byte-parity-to-committed to the
   regenerate-twice determinism class (mirroring ADR-0604/0613).
2. **Regenerate the frozen baseline from the merge-base SOURCE tree.** The emitter's `--merge-base-baseline`
   mode computes `merge_base` and the merge-base ratchet policy (frozen-policy-wins, unchanged), then instead of
   `git show <merge_base>:<face>` it materializes the merge-base source (worktree/`git archive`, reusing the
   producer's `--repo-root` port) and runs the producer there to PRODUCE the frozen baseline. The existing
   `relabel_frozen_face` step (rename-aware move-manifest re-keying) is PRESERVED (merge-base-tree keys are in
   merge-base naming, same as `git show` gave).
3. **Centralize the regen** in the materializer (`ci/facade/generated-artifact-freshness`) that already
   orchestrates emitter+producer and uploads faces ONCE per CI run; key the frozen snapshot by merge-base sha.
   No per-gate-leg worktree.
4. **Invert the ADR-0596 gate** `frozen_reference_artifact_must_stay_committed` → the frozen reference must be
   REGENERABLE-FROM-MERGE-BASE-SOURCE (the emitter must not read a committed generated blob; regen is the SOLE
   path, fail-closed — never a `git show` fallback).

### Trust (the HONEST delta — this is what founder ratifies)

B preserves the CENTRAL ADR-0551 anti-laundering property: the frozen reference stays derived from IMMUTABLE
merge-base SOURCE + policy read via the out-of-band bootstrap, so it is un-forgeable by the evaluated PR;
frozen modes derive from the merge-base `oya-ci.toml` DATA (a candidate disposition edit cannot rewrite its own
frozen reference).

B WEAKENS one property `git show` provided structurally: under status quo the frozen mode/keys of EXISTING codes
are un-authorable by the evaluated PR (immutable bytes; a `main-branch-materialized` face may only be
R100-relocated, its content controller-materialized from a PRIOR producer — `generated-artifact-policy/src/lib.rs:571-596`).
B recomputes the frozen reference with HEAD's producer, so a review-visible edit to the census kernel
(`build_gate_baseline`: downgrade an existing code's frozen mode block-on-new→advisory, or collapse key
granularity to fold in new debt) would pass GREEN under B where `git show` structurally REDs it (immutable old
bytes / phantom regression). **B trades a STRUCTURAL guarantee for a REVIEW-VISIBILITY guarantee on frozen
mode/keys.** Both attacks require a conspicuous edit to the census kernel (workflow-class, the same review tier
ADR-0551 already trusts for the emitter/gate code and the candidate side), but the STRUCTURAL foreclosure is lost.

Compensating controls (required):
- **H2** steady-state tamper-evidence: gate the census kernel (`build_gate_baseline` + disposition/key-format
  path) behind heightened-review/provenance, or a monotonic frozen-keyset-cardinality invariant. Ceiling named
  below.
- **H3** the PR-1 transition cross-check compares the FULL ratchet projection `{keys, mode, frozen_empty}` per
  `(gate,code)` (NOT just keyset — keyset-parity misses a mode downgrade), tolerating `config_digest` byte noise.
- **Ceiling / synthesis:** the fully-structural end-state is to have the materializer COMPUTE-AND-SIGN the frozen
  snapshot with controller/merge-base provenance the candidate PR cannot forge — restoring "frozen un-authorable
  by the evaluated PR" WITHOUT committing the face. Sequenced as the next slice after B's core proves out (do not
  build the signer speculatively).

## Consequences

- Removes the LAST committed generated merge-surface → kills the committed-face relocation fragility that caused
  the #828/keystone breaks; no privileged dev-writing controller (unlike Approach A).
- Trust: structural frozen-byte immutability → review-visibility + H2 tripwire, until the signer (ceiling) lands.
- **Alternative not taken — Approach A:** keep committed + wire the `controller-owned-main-materialization` the
  manifest already declares (`registry/generated-artifact-control-plane.json:129-137`). A is lower-TRUST-risk
  (structural immutability preserved) and reuses declared machinery, but retains the last committed surface, needs
  a privileged controller that writes to dev, and does not kill the relocation fragility. A and B do not dominate;
  founder chose B for north-star completion + no-main-write, ratifying the trust delta above.

## Verification (RED/GREEN)

Serial strangler (no de-committed-blob-read window → no #828 deadlock):
- **PR-1** — add regen-from-merge-base-source path; face STILL committed; cross-check the FULL ratchet projection
  (H3) between regen and `git show`; **prove blob-INDEPENDENCE** (H4 — regen with the committed face hidden/absent,
  assert regen never reads its path). RED fixtures: mode-downgrade + key-collapse caught by the projection
  cross-check; regen-failure = hard error (fail-closed).
- **PR-2** — de-commit; invert the ADR-0596 gate; determinism-class registry-drift/freshness; H2 tripwire. RED
  fixture: the census-kernel laundering vectors are caught by H2; a de-committed frozen with the emitter still
  reading `git show` would empty-frozen-deadlock (must not compile — regen is sole path).
