---
id: ADR-0563
title: "Rename-aware path-keyed CI baseline relabel at the scm-facts emitter — the systemic productization that unblocks strangler moves of already-accepted residue files"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-14
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amended_by: [ADR-0614]
depends_on: [ADR-0551, ADR-0555, ADR-0562]
related: [ADR-0515, ADR-0532, ADR-0533, ADR-0538, ADR-0552]
related_specs:
  - /specs/reorg/move-manifest.generated.json
  - /specs/reachability-registry.json
  - /registry/generated-artifact-control-plane.json
milestone: W0
---

# ADR-0563: Rename-aware path-keyed CI baseline relabel at the scm-facts emitter

## Status

**Proposed — 2026-06-14 (cloud-ci-platform; door: two-way — pure data-over-data capability, reversible by reverting the emitter relabel pass).**

**Scope of THIS PR (#737): the MACHINERY only.** This PR ships the rename-aware relabel mechanism
— the codemod move-manifest emitter (file-level + crate-DIR + crate-IDENT pairs derived from a
committed plan over the candidate tree), the emitter relabel pass, the `--plan` wiring through the
materialize pipeline + registry-drift, and the anti-forgery declaration. This PR commits NO move
plan, so the move-manifest is the canonical EMPTY manifest, the relabel is a verified strict
NO-OP (the frozen merge-base face is byte-identical before/after the relabel pass), and move-3 is
NOT yet unblocked. A subsequent MOVE PR exercises the machinery by committing its plan at
`specs/reorg/<capability>-move-plan.json`; only THAT PR's relabel fires and unblocks its move.

## Amended stance — move-manifest de-committed (ADR-0614, 2026-07-09)

**Amends the committed-vs-materialized stance ONLY; this ADR's lifecycle status stays Proposed and
its relabel machinery is unchanged.** Decision clause 1 (MANIFEST, below) establishes
`specs/reorg/move-manifest.generated.json` as the AUTHORITATIVE COMMITTED bijection, byte-bound by
the registry-drift `committed==regenerated` coverage. ADR-0614 reverses the *committed* half of
that stance: move-manifest is a pure derivation (a function of the committed move-plan(s) ×
candidate tracked tree), so it is now DE-COMMITTED (`materialization_mode: not-tracked-in-git`),
finishing the ADR-0595 → ADR-0613 pure-derivation strangler for the LAST committed reorg face. It
is materialized on demand by the generated-face materializer
(`//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`, step 1
`materialize_move_manifest`) BEFORE the emitter relabel reads it; registry-drift now validates
regenerate-twice DETERMINISM rather than committed-byte parity. The relabel semantics, the
anti-forgery binding (now a determinism binding), and the fail-closed resolver are otherwise
unchanged — the emitter reads the MATERIALIZED copy instead of a committed one. See ADR-0614 for
the gate / registry-drift / path-resolver consequences and the fail-open → fail-closed hardening
follow-up.

## Context

The firewall ratchet (ADR-0551) freezes today's known-debt as a merge-base baseline keyed
PER PATH (the `cloud-ci-brand-residue` gate keys residue by repo-relative file path; the
`cloud-ci-total-accounting` / `cloud-ci-target-parity` gates key by file/crate-dir path).
Those frozen baselines have NO rename-following: a strangler capability move (ADR-0562) of an
already-accepted-residue file changes its path, so the candidate carries the NEW path while the
frozen baseline still names the OLD path. The firewall then reads the moved file as NEW debt and
goes RED — even though the residue was already accepted at the old path and the move added none.
This blocked strangler move-3 on three observability source files carrying pre-existing
brand-residue vocabulary (a retired-brand stem the shrink-only ratchet tracks).

## Decision

Fix the staleness at the SINGLE sanctioned git boundary — the scm-facts emitter (ADR-0515 D3) —
with a content-aware RELABEL of the FROZEN merge-base snapshot's path-keyed keys, driven by an
AUTHORITATIVE committed move-manifest emitted by the reorg codemod. The firewall stays
byte-for-byte UNCHANGED (pure DATA-over-DATA on opaque string keysets): this is data-over-data,
not a firewall code change.

1. MANIFEST (the bijection, anti-forgery-bound). The codemod
   (`tools/oya-reorg-codemod-app`) emits a committed canonical-JSON generated face,
   `specs/reorg/move-manifest.generated.json` (schema `oya-ci/reorg-move-manifest/v1`),
   carrying both the FILE-level move pairs (derived deterministically from the crate-dir
   `MovePlan` — exact because the codemod does a wholesale `git mv <old_dir> <new_dir>`) and the
   crate-IDENT pairs (for tier-dep edge-endpoint mapping). It is declared in
   `registry/generated-artifact-control-plane.json` (artifact `cloud-ci-reorg-move-manifest`),
   so the registry-drift / freshness committed==regenerated coverage makes a hand-forged row RED
   before the firewall runs. For a no-move PR it is the canonical EMPTY manifest (identity
   relabel).

2. RELABEL PASS (in the emitter, `resolve_merge_base_baseline_snapshot`). After the frozen face
   is obtained at the merge-base and before it is wrapped, a pure `relabel_frozen_face` rewrites
   the PATH-keyed keys per the manifest. Load-bearing corrections: (1) the candidate side is the
   PRODUCER's universe (`git ls-files` existence + on-disk content), NEVER `git show HEAD:`;
   (2) the brand-residue P4 guard is `NEW_OCC ⊆ OLD_OCC` over the de-duplicated set of normalized
   matched-line texts, computed by the brand-residue census SSOT
   (`matched_line_occurrences_with`) under the LIVE VocabPolicy from `oya-ci.toml`; (3) tier-dep
   endpoints map via crate-IDENT pairs with a candidate-edge existence guard; (4) STRICT NO-OP
   when there are no renames, per-(gate,code) injective, fail-closed on collisions.

3. FAIL-CLOSED EVERYWHERE. A missing/malformed manifest, unreadable content, ambiguous pairing,
   or any guard failure yields the identity (no relabel), so the firewall sees the honest stale
   frozen face and goes RED on the moved paths. The relabel can only ever REMOVE a false-RED for
   a proven pure-or-shrinking relocation; it can never manufacture a false-GREEN. No sign-off
   door is used (founder doctrine: a purely mechanical rename needs no signoff).

## Amendment — Section C2: per-FILE total-accounting relabel (2026-06-17, cloud-ci-platform)

The original Section C relabeled `cloud-ci-total-accounting` (and `cloud-ci-target-parity`)
purely on the crate-DIR pairs, treating the gate as member_path-keyed. That is incomplete:
`total-accounting`'s real codes are keyed by **repo-relative FILE path** —
`unjustified` / `unowned` / `unreachable`. `unowned` re-derives from OWNERS and `unreachable`
from the reachability-registry, so a relocated file re-seeds those two automatically. But
`unjustified` has **no re-derivation seed**: its frozen baseline IS the only record that a file
was accepted as unjustified, so relocating an accepted-`unjustified` file with the crate-DIR-only
relabel left the OLD file path frozen and the NEW file path read as fresh debt — firewall RED.
Every prior strangler move relocated ADR/spec-justified files, so this path was never exercised;
the marketplace move's dev-cli (151 files) is the FIRST move of `unjustified`-tolerated crates,
which surfaced the gap.

Fix (data-over-data, no firewall change): a new `relabel_existence_only_file_gate` (Section C2),
dispatched alongside `relabel_existence_only_gate` for `GATE_TOTAL_ACCOUNTING` only. It mirrors
Section C's existence-only P1(frozen-key)+P2(old-absent)+P3(new-present) guard and per-(gate,code)
injectivity, but uses **EXACT** candidate-path membership (these keys ARE tracked file paths) in
place of the directory-aware descendant test. No content guard (same rationale as Section C: the
FILE pairs come from the registry-drift-checked move-plan manifest and the codemod is a
content-preserving mover). The per-DIR relabel (`relabel_existence_only_gate`) is **retained
defensively** for `GATE_TOTAL_ACCOUNTING`: it covers any member_path/crate-DIR-keyed code and is
a harmless no-op against the per-FILE codes otherwise. The two relabels touch disjoint key
classes (a crate-DIR string never equals a FILE pair's old-key and vice versa), so running both
is order-independent and non-overlapping. `GATE_TARGET_PARITY` keeps the dir-only relabel. C2's
anti-laundering safety is **load-bearing on the move-manifest's registry-drift binding**
(committed==regenerated from the codemod's wholesale-git-mv mirror-suffix pairs): a forged
old->new pair REDs at registry-drift BEFORE the firewall runs, so this relabel can only relocate
an already-accepted entry and can never admit new debt. Inert without a committed move-plan
(`file_pairs` empty ⇒ strict byte-identical no-op), so this amendment changes no materialized
face on non-move PRs. `ci/facade/scm-facts-snapshot/src/main.rs`
carries the function + the split dispatch + seven unit pins (marketplace-shaped relabel, P1/P2/P3/
collision fail-closed, mixed per-DIR + per-FILE independence, empty-manifest inert) with a
non-vacuity canary proving the marketplace pin fails without the C2 dispatch line.

## Consequences

- Neutral: the firewall, the producer, and every other gate are unchanged; the relabel is pure
  data-over-data at the emitter boundary.
- Positive: the machinery lets strangler moves of already-accepted residue files stop reading as
  new debt — a move PR that commits its plan unblocks its own move with no manual signoff door.
  THIS PR ships the machinery as a strict no-op (no plan committed); move-3 is unblocked by the
  subsequent move PR that commits the observability move plan.
- Reachability + justification: the committed move-manifest face
  `specs/reorg/move-manifest.generated.json` is justified by this ADR and reachable via the
  `specs/reorg/` reachability-registry entry (ADR-0555 born-accounting). The first capability
  move to exercise the machinery — the ci keystone (`cloud/cloud-ci/gates` → `ci/facade/`,
  ADR-0562) — commits its per-capability plan `specs/reorg/ci-move-plan.json`, likewise
  justified by this ADR and reachable via the same `specs/reorg/` entry. That same move also
  commits its Cargo.lock graph-additions companion `specs/reorg/ci-graph-additions.json` — the
  new local members and dependency edges the move introduces beyond the renames, consumed WITH
  the plan by the owned `oya-xtask-metadata-augment lockfile-move` maintenance (deterministic
  text/graph transform, no cargo, no version resolution) — likewise justified by this ADR and
  reachable via the same `specs/reorg/` entry. The relabel resolves
  every path through the move-stable `PathId` port + manifest adapter (the keystone's proven
  security core): `ci/ports/path-resolver/src/lib.rs`, `ci/ports/path-resolver/Cargo.toml`,
  `ci/ports/path-resolver/BUCK`, `ci/adapters/path-resolver/src/lib.rs`,
  `ci/adapters/path-resolver/Cargo.toml`, and `ci/adapters/path-resolver/BUCK` — all justified
  by this ADR as the load-bearing resolution seam of the rename-aware relabel.

## Files

- `ci/facade/scm-facts-snapshot/src/main.rs` — the relabel pass.
- `tools/oya-reorg-codemod-app/src/model.rs` and `tools/oya-reorg-codemod-app/src/main.rs` — the move-manifest emitter.
- `specs/reorg/move-manifest.generated.json` — the committed bijection face.
- `registry/generated-artifact-control-plane.json` — the anti-forgery declaration.
- `libs/oya-check-brand-residue/src/forbidden_vocab.rs` — the occurrence-identity SSOT.
- `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin` — the codemod->emitter->producer ordering.
- `ci/facade/inventory-registry-drift/tests/registry_drift.rs` — the committed==regenerated coverage for the move-manifest face.

The firewall-invariance fixtures pinning the relabeled-frozen-face behavior (the firewall is
content-blind, so they use opaque residue codes):

- `specs/fixtures/cloud-ci-firewall/tc-FW-good-relabeled-residue-move-tolerated.json` — a relabeled pure move is GREEN.
- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-relabel-refused-add-residue-still-red.json` — a refused (add-residue) move stays RED.
- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-relabel-stem-swap-new-residueb-red.json` — a per-code-scoped stem-swap stays RED under the second code.
