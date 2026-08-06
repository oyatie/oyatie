---
id: ADR-0604
title: "De-commit the scm-facts boundary snapshot — the last committed pure-derivation face (completes ADR-0595)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-24
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
amended_by: [ADR-0616]
amends: []
depends_on: [ADR-0515, ADR-0539, ADR-0551, ADR-0552, ADR-0555, ADR-0595]
related: [ADR-0111, ADR-0363, ADR-0541, ADR-0558, ADR-0596]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0604: De-commit the scm-facts boundary snapshot

## Status

**Proposed - 2026-06-24 (authored for founder sign-off; door: one-way — once the de-commit and
the re-tracking guard land, "the scm-facts boundary snapshot is not a contributor merge surface"
is a one-way commitment, identical in shape to ADR-0595).**

## Context

ADR-0595 de-committed six pure-derivation cloud-ci faces but explicitly DEFERRED
`ci/facade/artifact-inventory-registry/scm-facts.generated.json`, because it
is a declared hermetic input to ~20 gate tests and its de-commit required a dedicated repoint
pass. It is now the **single remaining committed pure-derivation face** — and the ROOT of the
faces-serialization cascade the founder flagged:

- The scm-facts snapshot is a pure function of the tracked tree (`git ls-files` × last-touch
  commit map × author timestamps). Because it is committed AND lists every tracked path, it lists
  ITSELF, and any PR that changes the tracked set re-materializes it.
- The face-settle protocol regenerates it in every PR that touches a non-generated path, so every
  merge to `dev` rewrites the committed snapshot — which DIRTIES every other open PR, forcing
  endless hand-rebasing. With the other six faces de-committed (ADR-0595), scm-facts is the LAST
  shared committed merge surface, so it alone now sustains the cascade.

De-committing it leaves **zero** shared committed generated-face merge surface, so the cascade is
structurally impossible: a PR can no longer dirty other PRs by re-materializing a generated face.

## Decision

STOP committing `scm-facts.generated.json`. It is declared `materialization_mode:
not-tracked-in-git` (was `main-branch-materialized`) with `merge_policy:
never-manual-merge-regenerate-from-source-tree` in
`registry/generated-artifact-control-plane.json`, removed from git (`git rm --cached`), and
covered by the existing `**/*.generated.json` ignore. It is derived on demand via
`buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin` (which already writes it to the canonical
path) and materialized out-of-graph for consumers before gates run.

This is **NOT** the #828→#830 dev-deadlock class and does **NOT** trip the #831/ADR-0596
frozen-reference guard: the firewall's frozen ratchet reference is `gate-baseline.generated.json`
(declared in `firewall/ratchet-policy.json`), NOT scm-facts. scm-facts is the SCM-facts boundary
snapshot the producer derives FROM (regenerated from `git ls-files` each run), never the firewall's
frozen reference, so de-committing it empties no ratchet baseline. The
`frozen_reference_decommit_findings` guard matches on the full canonical frozen-reference path;
scm-facts is not in that set, so no `frozen_reference_artifact_must_stay_committed` finding fires.

KEEP committed (unchanged): `gate-baseline.generated.json` (frozen ratchet reference, ADR-0551 /
ADR-0596 — stays tracked and negated in `.gitignore`), `firewall/ratchet-policy.json`,
`registry/generated-artifact-control-plane.json`, all upstream sources.

### Gate teaching (the no-flag-day mechanics)

1. **Control-plane gate** (`oya-cloud-ci-generated-artifact-control-plane-app`): the
   `scm-facts-boundary-snapshot` artifact class previously REQUIRED
   `materialization_mode == main-branch-materialized` AND
   `merge_policy == controller-owned-main-materialization`. Both rules are amended to recognise
   the de-commit class as a SECOND valid controller-materialized shape: a boundary snapshot may
   be `not-tracked-in-git` paired with `never-manual-merge-regenerate-from-source-tree` (the same
   merge policy every ADR-0595 de-committed face carries). Both shapes are controller-owned and
   non-source-authored; every other mode/policy still RED. The de-commit class already gets the
   `declared_path_not_tracked` exemption and the `not_tracked_path_is_tracked` one-way-door guard
   from ADR-0595 (no change), so re-committing scm-facts is a hard RED.

2. **Freshness gate** (`oya-cloud-ci-freshness-app`): NO CODE CHANGE. The de-commit-class
   machinery added in ADR-0595 keys de-commit faces off the manifest's `materialization_mode` by
   CANONICAL FULL PATH (`read_decommitted_face_names` requires the path be in
   `GENERATED_FACE_PATHS`, whose first entry is already the scm-facts canonical path). The moment
   the manifest mode flips, scm-facts auto-enrols: byte-parity-to-committed is skipped and it is
   instead validated by regenerate-success + the regenerate-twice determinism canary
   (`evaluate_face_determinism`).

3. **registry-drift** (`registry-drift` test crate): the `committed_scm_facts_equal_regenerated`
   byte-parity test (read committed copy, compare to one regeneration) is converted to the
   determinism class — regenerate the snapshot TWICE and assert byte-identical
   (`scm_facts_regenerates_deterministically`), matching the freshness gate. There is no longer a
   committed copy to compare against.

4. **`.gitignore`**: documents the de-commit; the path is covered by the existing
   `**/*.generated.json` rule and is intentionally NOT negated (unlike gate-baseline).

### Merge-base anchor / non-forgeability

The de-commit exemption is NOT forgeable from the candidate tree. Both the freshness gate
(`read_decommitted_face_names`) and the de-commit machinery match on the **canonical full path**
baked into the compiled gate binaries (`GENERATED_FACE_PATHS`), never the basename — so a
candidate-controlled manifest row at a deceptive path sharing a basename (e.g.
`anything/scm-facts.generated.json`) cannot retire the real face's checks. This mirrors ADR-0595's
guard and the ADR-0551 merge-base-frozen-reference doctrine (the firewall's frozen baseline is
materialized from `git show <merge-base>:<path>`, never the candidate copy).

### Consumer materialization (the deferred repoint from ADR-0595)

Every scm-facts consumer materializes it on demand rather than reading a committed copy:

- The **CI producer-regen job** runs `oya-cloud-ci-materialize-generated-faces-bin` (which writes
  scm-facts to its canonical path) and uploads `accounting-registry-app/*.generated.json` — a
  FILESYSTEM glob that captures the untracked scm-facts — as the `accounting-faces` artifact.
- Every **gate matrix leg** downloads `accounting-faces` into `cloud/cloud-ci/gates` before
  `cargo test`, restoring scm-facts on disk. The ~15 producer-input gate tests, root-workspace-
  hygiene (direct read), and the control-plane live test all `root.join(<canonical path>)` and
  find it there. No test logic changes; the artifact flow is byte-identical to ADR-0595's six
  faces.
- **registry-drift** and the **firewall lane** re-materialize in-job (they are detectors and must
  not consume a shared artifact).
- **Local dev**: run `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .` first, identical to the
  six ADR-0595 faces.

So no CI workflow change is required: the producer-regen → upload → matrix-leg download chain
already materialized scm-facts before gates ran (it is uploaded by the filesystem glob whether or
not it is tracked).

## Consequences

- scm-facts is never in a PR diff, so it can no longer conflict — and with the six ADR-0595 faces
  already de-committed, there is now ZERO shared committed generated-face merge surface. The
  faces-serialization cascade is structurally impossible.
- Determinism is the load-bearing integrity invariant for scm-facts (previously masked by
  byte-equality-to-committed). The regenerate-twice canary in registry-drift + freshness makes a
  nondeterministic emitter hard-fail rather than silently green — the same cold-vs-warm
  integrity-canary doctrine that makes derive-don't-commit sound.
- The de-committed scm-facts no longer lists itself in `tracked_paths` (it is no longer tracked),
  which is the precise mechanism that removes the self-mutating merge surface.
- Born-accounting / register-crate remain fully enforceable: the producer derives the registry
  each PR from the materialized scm-facts; nothing about born-accounting depended on scm-facts
  being committed.
- The materializer now runs through the Rust/Buck2 binary
  `//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`;
  the transitional shell bridge is retired. The remaining destination is a cloud-ci controller
  path for final-tree materialization.

## Alternatives considered

- **Keep scm-facts committed + a merge driver**: leaves the self-mutating multi-MB snapshot in
  every PR diff and re-derives it on every touch. De-committing removes the surface entirely.
- **Keep scm-facts committed but stop listing itself**: a special-case carve-out in the emitter
  would still leave a committed snapshot that mutates whenever any other tracked path changes —
  the cascade persists. De-committing is the only fix that removes the surface.

## Born-accounting

This ADR adds one decision node (ADR-0604) consumed by the accounting-registry producer's ADR
front-matter projection and the decision-crosswalk. It adds no new crate; the changes are edits
to two existing gate crates (control-plane gate amends the boundary-snapshot rules + RED/GREEN
fixtures; registry-drift converts one test to the determinism class) plus the manifest flip and
the `git rm --cached`. scm-facts leaves the registry's tracked-output set but remains DECLARED in
the control-plane manifest, so the registry's declared-artifact accounting is unchanged in shape.

## Supersedes / feeds

- Completes (does NOT supersede) ADR-0595, ADR-0551, and ADR-0552 for the final producer face.
- Does NOT trip ADR-0596 (frozen-reference must stay committed): scm-facts is not a frozen
  reference.
- After this de-commit, the only residual shared committed surface in the faces dir is the
  human-owned `gate-baseline.generated.json` frozen reference and `signoff.json` door; the
  ADR-0558 faces merge driver should be scoped to those only.
