---
id: ADR-0563
title: "Rename-aware path-keyed CI baseline relabel at the scm-facts emitter — the systemic productization that unblocks strangler moves of already-accepted residue files"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-14
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amended_by: []
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

## Consequences

- Neutral: the firewall, the producer, and every other gate are unchanged; the relabel is pure
  data-over-data at the emitter boundary.
- Positive: strangler moves of already-accepted residue files no longer read as new debt;
  move-3 (and every future capability move) is unblocked with no manual signoff door.
- Reachability + justification: the committed move-manifest face
  `specs/reorg/move-manifest.generated.json` is justified by this ADR and reachable via the
  `specs/reorg/` reachability-registry entry (ADR-0555 born-accounting).

## Files

- `cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/src/main.rs` — the relabel pass.
- `tools/oya-reorg-codemod-app/src/model.rs` and `tools/oya-reorg-codemod-app/src/main.rs` — the move-manifest emitter.
- `specs/reorg/move-manifest.generated.json` — the committed bijection face.
- `registry/generated-artifact-control-plane.json` — the anti-forgery declaration.
- `libs/oya-check-brand-residue/src/forbidden_vocab.rs` — the occurrence-identity SSOT.
- `infra/ci/materialize-cloud-ci-generated-faces.sh` — the codemod->emitter->producer ordering.
- `cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs` — the committed==regenerated coverage for the move-manifest face.

The firewall-invariance fixtures pinning the relabeled-frozen-face behavior (the firewall is
content-blind, so they use opaque residue codes):

- `specs/fixtures/cloud-ci-firewall/tc-FW-good-relabeled-residue-move-tolerated.json` — a relabeled pure move is GREEN.
- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-relabel-refused-add-residue-still-red.json` — a refused (add-residue) move stays RED.
- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-relabel-stem-swap-new-residueb-red.json` — a per-code-scoped stem-swap stays RED under the second code.
