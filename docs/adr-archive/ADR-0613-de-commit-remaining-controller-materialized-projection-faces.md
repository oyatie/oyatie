---
id: ADR-0613
title: "De-commit the remaining controller-materialized projection faces (masterplan + product-graph) — finish the pure-derivation strangler"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-07-09
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amends: [ADR-0364]
depends_on: [ADR-0515]
related: [ADR-0539, ADR-0595, ADR-0596, ADR-0597, ADR-0364, ADR-0066, ADR-0563]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0613: De-commit the remaining controller-materialized projection faces

## Status

**Accepted - 2026-07-13** (ratified under the founder's 2026-07-08 autonomous-drive delegation and
the 2026-07-09 scope-refinement approval; door: one-way — same policy class as ADR-0595). The
manifest/gitignore/controller propagation landed in PR #1222. The 2026-07-13 lifecycle promotion is
paired with direct, independent masterplan and product-graph regeneration/determinism enforcement;
the prior transitive dashboard-only claim was a false-green and is not acceptance evidence.
ADR-0539, ADR-0595, and ADR-0597 have frontmatter status `Proposed` and are nonbinding
implementation provenance; this
Accepted decision depends only on the Accepted ADR-0515 admission authority.

## Context

ADR-0595 de-committed the six pure-derivation accounting faces under
`ci/facade/artifact-inventory-registry/`; ADR-0597 completed that strangler by de-committing the
last shared accounting face (`scm-facts.generated.json`) and standardising the owned Rust/Buck2
materializer `//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`.
Two controller-materialized **projection** faces were left behind as committed merge surfaces:

| Face | Content | Prior mode |
|---|---|---|
| `docs/machine-readable/masterplan.generated.json` | projection of `specs/masterplan.json` × `specs/master-plan-sequencing.json` × `docs/decisions/**` | `branch-committed-regenerated-until-controller-materialization` (an explicitly transitional mode) |
| `docs/architecture/product-graph.html` | arch-graph dashboard generated from `architecture-graph.json` × masterplan × `product-graph.template.html` | `main-branch-materialized` (already recognised as de-commit-class by the freshness gate via `CONTROLLER_MATERIALIZED_ARTIFACT_PATHS`, yet still git-tracked) |

Both are pure deterministic functions of committed sources; neither carries human bits. Leaving
them committed forced every capability move / SSOT-touching PR to re-materialize multi-hundred-line
blobs, reproducing the ADR-0595 merge-conflict friction one layer up. The motivating incident: a
strangler-move executor ran the masterplan materializer with partial inputs and committed a
**corrupt** projection (`adr_count` 106 → 64, `deliverable_count` 108 → 75, net −500 lines from
moving a single leaf crate) that nearly landed. The corruption was possible **only because the face
was a committed merge surface a local tool could overwrite**. Committed-byte-parity against a copy a
mis-invoked local tool can silently poison is exactly the anti-pattern the ADR-0595 de-commit
retires.

## Decision

STOP committing `docs/machine-readable/masterplan.generated.json` and
`docs/architecture/product-graph.html`. Both are declared `materialization_mode:
not-tracked-in-git` in `registry/generated-artifact-control-plane.json`, removed from git
(`git rm --cached`), and covered by `.gitignore` (masterplan by the existing `**/*.generated.json`
rule; product-graph.html needs an explicit line — it is a `.html`, not a `*.generated.json`). They
are derived on demand by the existing materializer entrypoint (masterplan first, then the
arch-graph dashboard) and materialized out-of-graph for consumers before gates run.

### Historical scope boundary — move manifest (later amended by ADR-0614)

At this decision's acceptance, ADR-0595 still special-cased move-manifest as "stays committed" and
this ADR deliberately left it outside the two-face de-commit. Accepted ADR-0614 later amended that
boundary. The original reasons remain useful incident context:

1. **It conflicted with a proposed design.** ADR-0563 is Proposed, not Accepted; it proposed making
   move-manifest the authoritative committed rename-aware move-bijection that the path-keyed CI
   baseline relabel consumes. This rationale did not make ADR-0563 binding. ADR-0614 later accepted
   and implemented de-commitment with independent lifecycle propagation.
2. **It introduced a silent failure mode.** `ci/adapters/path-resolver` formerly failed **open to
   identity** on an absent manifest. De-committed, any emitter leg not preceded by
   a full materialize would silently relabel to identity — turning every pre-existing renamed-path
   debt item into a phantom "new regression" (a silent false-RED on move PRs), a worse failure than
   the one this ADR fixes.
3. **It was never the corruption source.** move-manifest is codemod-deterministic (a pure function
   of the committed move-plan × candidate tree); it regenerated correctly in the same incident that
   corrupted masterplan. It is the *reviewed* artifact — a human reads the bijection.

ADR-0614 de-commits the move manifest only after making relabel fail closed and requiring materialize
before consume. That amendment is separate from ADR-0613's original two-face scope.

### Gate teaching (no flag day)

1. **Control-plane / policy gate** (`ci/facade/generated-artifact-policy`): no code change — the
   ADR-0595 `not-tracked-in-git` machinery already applies. The existing
   `generated_artifact_not_tracked_path_is_tracked` finding is the one-way re-tracking guard for
   the two newly de-committed paths; `generated_path_rules` already classifies both (the
   `*.generated.json` suffix rule for masterplan, the explicit `docs/architecture/product-graph.html`
   path rule for the dashboard).
2. **Freshness gate** (`ci/facade/generated-artifact-freshness`): the acceptance closure explicitly
   registers both de-committed faces as independent regenerate-twice outputs. Each pass generates a
   temporary masterplan projection and then generates product-graph from that projection; the gate
   compares the masterplan bytes and product-graph bytes independently between the two passes. A
   masterplan mismatch is therefore RED even when product-graph is stable. This direct required-CI
   enforcement replaces the earlier false-green transitive claim. The `masterplan-drift` lane
   (`registry/quality/lanes.yaml`, a dev-cli local bridge) remains feedback-only and checks successful
   regeneration rather than committed-byte parity; it is not merge authority.
3. **`.gitignore`**: documents the de-commit; adds the explicit `docs/architecture/product-graph.html`
   line and records the move-manifest deferral rationale inline.
4. **`docs-graph-drift.yml`** (feedback-only, NOT branch-protection-required): re-taught — it
   materializes the de-committed masterplan input before building/testing the generator, and its
   retired byte-parity-vs-committed `git diff --exit-code` step (which would go silently vacuous on
   an untracked file) is removed in favour of the required freshness gate's determinism canary.

## Consequences

- The two projection faces are never in a PR diff again; the move-corruption class is structurally
  impossible (a strangler move performs a pure structural rename+rewire and never touches a
  materializer-derived face). This is the fix for the incident above.
- Direct, independent regenerate-twice determinism is the load-bearing integrity invariant for both
  masterplan and product-graph. Byte parity against a committed copy is retired for both; successful
  local-bridge regeneration is supplementary feedback, not a substitute for required-CI enforcement.
- Consumer safety (verified): no BUCK `srcs` reference either face, so `buck2 build` is unaffected.
  The CI-required masterplan consumers materialize-first or are tolerant (masterplan-drift gate
  short-circuit; cross-artifact-agreement reads masterplan in a leg that runs the materializer
  first). The `board_masterplan_consistency` / `board-sync` consumers are dev-cli `oya gate` / `oya
  gen` LOCAL BRIDGE commands, not wired into `oya-ci-required.yml` or `registry/quality/lanes.yaml`
  — feedback-only per the CLI-retirement doctrine, never merge authority.

## Alternatives considered

- **De-commit all three faces (include move-manifest).** Rejected for this ADR because the then-live
  relabel path failed open when the manifest was absent. Accepted ADR-0614 later addresses this as
  a separate fail-closed amendment.
- **Hand-revert the corrupt face and keep committing.** Rejected: it leaves the identical trap
  armed for the next move; it treats a symptom, not the class (the friction-is-process-failure
  doctrine).

## Born-accounting

This ADR adds one decision node (ADR-0613) consumed by the accounting-registry producer's ADR
front-matter projection and the decision-crosswalk. It adds no new crate. The PR edits two entries
in `registry/generated-artifact-control-plane.json`, adds one `.gitignore` line (+ documentation),
re-teaches `.github/workflows/docs-graph-drift.yml`, and `git rm --cached`s the two faces. The
de-committed faces leave the tracked-output set but remain DECLARED in the control-plane manifest,
so declared-artifact accounting is unchanged in shape. `decisions.json` / `ADR-INDEX.md` are
land-time materialized, not hand-edited in this PR.

## Supersedes / feeds

- Completes (does NOT supersede) ADR-0595 and ADR-0597 for the two remaining controller-materialized
  projection faces.
- Amends ADR-0364's committed-surface stance for the masterplan projection and the product-graph
  dashboard (they become derive-on-demand, not committed).
- Leaves Proposed ADR-0563's broader move-manifest design outside this ADR. Accepted ADR-0614
  separately governs the de-committed move-manifest disposition. At acceptance ADR-0613 reaffirmed
  ADR-0596's committed frozen-reference rule; Accepted ADR-0616 later reverses that rule for the
  provenance-bound merge-base regeneration design.
