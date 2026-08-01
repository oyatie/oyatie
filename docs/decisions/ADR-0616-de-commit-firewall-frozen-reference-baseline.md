---
id: ADR-0616
title: "De-commit the firewall frozen-reference baseline — regenerate it from merge-base SOURCE (reverses ADR-0596)"
status: Accepted
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

# ADR-0616: De-commit the firewall frozen-reference baseline

## Status

**Accepted — 2026-08-01.** The founder selected deterministic regeneration from merge-base source
during the north-star authority interview. This reverses ADR-0596's committed-face requirement and
narrows ADR-0613's earlier reaffirmation accordingly. The trust delta in Decision §Trust remains
binding: the baseline must be regenerated twice from the immutable merge-base source and bound to
that source by provenance. Door: one-way.

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

1. **De-commit `gate-baseline.generated.json`** (manifest `not-tracked-in-git` + merge_policy
   `never-manual-merge-regenerate-from-source-tree`; `git rm --cached`; `.gitignore` drop the `!` negation). Its
   freshness/registry-drift checks fall to the regenerate-twice determinism class (mirroring ADR-0604/0613), the
   same class as every other de-committed face.
2. **Regenerate the frozen baseline from the merge-base SOURCE tree** (the emitter's production frozen-read,
   REPLACING `git show <merge_base>:<face>`). The emitter's `--merge-base-baseline` mode still computes
   `merge_base` and reads the merge-base ratchet POLICY (frozen-policy-wins, unchanged); the frozen FACE is now the
   `--regen-baseline-face` regeneration handed in by the materializer, which materializes the merge-base tracked
   tree (`git worktree add --detach <merge_base>`) and runs the accounting producer there. `relabel_frozen_face`
   (rename-aware move-manifest re-keying) and every fail-closed guard are PRESERVED. **Regen is the SOLE face
   source — the `git show` committed-blob fallback is removed, which is exactly what makes a de-committed frozen
   reference unable to empty-frozen-deadlock (the #828 defect).**
3. **Validate by DETERMINISM (the hyperscaler trust mechanism).** The materializer regenerates the frozen baseline
   TWICE over the same merge-base source and the emitter asserts the two project IDENTICALLY (`{keys, mode,
   frozen_empty}` per `(gate, code)`); a non-deterministic producer is a hard error. Factored as a reusable
   projection-level determinism canary.
4. **Provenance (in-toto materials/subject).** The snapshot carries a `provenance` object: `base_tree_sha`
   (`git rev-parse <merge_base>^{tree}`), the analyzer identity (producer/emitter buck labels), and `computed_by`.
   The firewall parser VERIFIES provenance is present, `base_tree_sha` is a well-formed tree id, and the provenance
   is bound to the snapshot's own `merge_base` (fail-closed; the firewall never calls git). Cryptographic signing
   of this provenance is a fleet-wide follow-on (the ceiling below), NOT built here.
5. **Centralize the regen** in the materializer (`ci/facade/generated-artifact-freshness`) that already
   orchestrates emitter+producer ONCE per CI run; the emitter owns the merge-base (single git boundary), the
   materializer materializes exactly that tree. No per-gate-leg worktree.
6. **Invert the ADR-0596 gate** `frozen_reference_artifact_must_stay_committed` → a frozen reference MAY be
   de-committed IFF its ratchet policy declares `frozen_reference.source: regenerate-from-merge-base-source`
   (data-driven, no hardcoded paths). A committed-git-blob frozen reference de-committed WITHOUT that declaration
   still RED-blocks, so #828 stays impossible.

### Decision criterion (founder 2026-07-09): what would a hyperscaler do

ADR reversal is not a cost; the sole criterion is the hyperscaler-grade answer. Hyperscalers
DERIVE-DON'T-COMMIT the ratchet base and trust it via ATTESTATION, not committed bytes:
Bazel affected-target-set computes the base state from base-revision source (never a committed
failing-set); Google Tricorder/Critique recompute analysis from source at the base revision;
SLSA / in-toto / Binary Authorization trust derived artifacts via signed build provenance from
trusted CI. A committed baseline + a trunk-writing controller (Approach A) is the small-repo OSS
shape (Betterer's committed `.betterer.results`) and a shared-mutable-state anti-pattern at monorepo
scale (the merge-conflict/drift that broke dev twice). This ADR adopts the SYNTHESIS: de-commit +
regenerate-from-source in trusted CI + ATTEST the computed frozen snapshot (provenance now, signer later).
Structural immutability of committed bytes is the crutch this replaces with the hyperscaler trust model
(reviewed analyzer code + deterministic compute + attestation) — a malicious producer edit is a
review-visible code change AND fails the determinism canary / rewrites attestation.

### Regen scope: producer over the merge-base TRACKED tree (NOT the full materialize chain)

An earlier framing feared "regenerate from source" meant running the FULL materialize chain (controller generators
+ emitter + producer) at the merge-base, because the producer censuses `not-tracked-in-git` controller artifacts
(masterplan, product-graph — ADR-0613). A real-binary attempt disproved it: the producer censuses the git-TRACKED
path universe (`scm-facts = git ls-files`, `artifact-inventory-registry/src/main.rs`), NOT the on-disk filesystem —
so materializing controller artifacts on disk changes nothing (they are untracked at the merge-base). The regen is
therefore the exact Bazel affected-set shape and CHEAP: `git worktree add --detach <merge_base>` (tracked files) +
the merge-base scm-facts + the accounting producer. Controller generators are irrelevant to the frozen baseline.

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

Compensating controls (shipped):
- **Determinism canary** — the frozen baseline is regenerated TWICE over the same merge-base source and asserted
  projection-identical (`{keys, mode, frozen_empty}` per `(gate, code)`, NOT just keyset — keyset-parity misses a
  mode downgrade), tolerating `_provenance.config_digest` byte noise. A non-deterministic producer is a hard error,
  so the regenerated reference is trustworthy because it is reproducible.
- **Provenance** — the snapshot binds the regeneration to the immutable merge-base tree (`base_tree_sha`), verified
  fail-closed by the firewall parser. This is the in-toto materials/subject a signer will later bind.
- **Review-visibility** — the census kernel (`build_gate_baseline`) is workflow-class code; a mode downgrade / key
  collapse to launder debt is a conspicuous, review-visible edit (the same tier ADR-0551 already trusts for the
  emitter/gate and the candidate side).
- **Ceiling / synthesis (follow-on, NOT built here):** have the materializer COMPUTE-AND-SIGN the provenance with a
  key the candidate PR cannot forge — restoring "frozen un-authorable by the evaluated PR" WITHOUT committing the
  face. Fleet-wide; sequenced after this core proves out (do not build the signer speculatively).

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

Owned-Rust, buck2 (unit + integration), plus a real-binary run over the actual merge-base:
- **Emitter** (`ci/facade/scm-facts-snapshot`): the frozen FACE comes from `--regen-baseline-face`, the `git show`
  face read is removed. RED pins: with the policy present at the merge-base a missing regeneration is a hard error
  (no empty/`git show` fallback); the determinism canary hard-fails a mode downgrade / key divergence between the
  two regenerations; frozen-policy-wins (the #698 repoint attack) and the rename-aware relabel fixtures still pass.
- **Ratchet** (`ci/facade/baseline-ratchet`): `FrozenBaseline::from_value` REJECTS a snapshot with missing
  provenance, a malformed `base_tree_sha`, or a `base_tree_sha`/`merge_base` bound to a different merge-base.
- **Materializer** (`ci/facade/generated-artifact-freshness`): the regeneration is a hard error on producer
  failure (fail-closed) and blob-INDEPENDENT (runs `--face baseline`, never references the committed blob path).
- **Gate** (`ci/facade/generated-artifact-policy`): a frozen reference declaring
  `source: regenerate-from-merge-base-source` MAY be de-committed (GREEN); one that does not still REDs on
  de-commit. The live control-plane corpus is GREEN with `gate-baseline.generated.json` de-committed.
- **Real merge-base run:** over `git merge-base origin/dev HEAD` the two regenerations are byte-identical, the
  frozen baseline is a sane non-empty reference (15 gates / 88 `(gate, code)` entries), and the provenance
  `base_tree_sha` equals `git rev-parse <merge_base>^{tree}`.

A de-committed frozen with the emitter still reading `git show` would empty-frozen-deadlock — impossible here
because the `git show` face read is removed (regen is the sole path).
