---
id: ADR-0620
title: "De-commit the active-artifact-contract graph projection and make controller regeneration its only live form"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-07-21
door: one-way
owner: council-architecture + cloud-ci-platform
supersedes: []
superseded_by: []
amends: [ADR-0613]
amended_by: []
depends_on: [ADR-0069, ADR-0515, ADR-0613, ADR-0619]
related: [ADR-0539, ADR-0552, ADR-0595, ADR-0597]
related_specs:
  - /specs/active-machine-readable-artifact-contract.json
  - /specs/artifact-profile-defaults.json
  - /registry/artifact-capabilities-registry.json
  - /registry/generated-artifact-control-plane.json
milestone: W0
---

# ADR-0620: De-commit the active-artifact-contract graph projection

## Status

**Accepted — 2026-07-21.** The founder's standing technical delegation authorizes this narrow
pipeline-integrity repair. It does not approve roadmap planning, implementation dispatch,
production readiness, or any qualified legal, custody, operations, affected-party, council, or
dissent claim. `HOLD(Planning)` remains controlling.

Acceptance is atomic with the control-plane, materializer, consumer-delivery, freshness, and
hand-edit-rejection changes described below. A status-only decision or an unpropagated
implementation is invalid evidence.

## Context

`registry/graph/active-artifact-contract-edges.json` is a deterministic projection of every row in
`registry/artifact-capabilities-registry.json`: one ordered `artifact_id -> artifact_profile`
`declares` edge per row. The active-artifact-contract gate has long been able to emit the file, but
the path was neither declared in `registry/generated-artifact-control-plane.json` nor classified by
any generated-path rule. It remained a tracked merge surface and could be edited directly without
the generated-output diff policy identifying the edit as generated output.

A byte comparison against a committed copy is not an adequate repair. The candidate could change
the source and generated copy together, while concurrent changes would continue to serialize on a
shared derived file. ADR-0613 already established the stronger pattern for pure projections:
remove the derived bytes from Git, register the face, materialize it from the candidate tree before
consumers run, and prove regeneration plus regenerate-twice determinism.

## Decision

1. Stop tracking `registry/graph/active-artifact-contract-edges.json`. Git history remains its sole
   committed provenance; no readable archive or tombstone copy is created.
2. Register the exact path in the generated-artifact control plane as
   `not-tracked-in-git` with `never-manual-merge-regenerate-from-source-tree` and the Buck-built
   `//marketplace/facade/dev-cli:oya` producer operation
   `emit-active-artifact-contract-graph-edges`.
3. Add an exact generated-path rule for the legacy non-`.generated.json` filename. A direct add,
   modify, rename, or copy is therefore rejected by the generated-output diff policy; deletion is
   the one allowed retirement transition.
4. Do not add the face to `registry/artifact-capabilities-registry.json`. That registry requires
   its artifact paths to be HEAD-tracked; presenting this de-committed projection as a tracked
   active authority would make its own producer fail and would recreate the authority ambiguity.
5. Extend the one official generated-face materializer to invoke the existing producer after its
   candidate-tree accounting inputs are available. Required CI transports the materialized path to
   mere-reader gate jobs before cross-artifact consumers run.
6. Replace committed-copy freshness with three independent checks:
   - exact semantic projection equality for every registry row;
   - exact canonical output bytes derived from the complete ordered row population; and
   - independent regenerate-twice byte stability in the generated-artifact freshness gate.
7. Missing, malformed, incomplete, reordered, or extra graph rows fail closed. Producer failure,
   artifact-delivery failure, and re-tracking the path also fail closed.

## Consequences

- Agents and humans can no longer establish graph truth by editing a tracked JSON face.
- Registry changes do not create a shared generated merge surface. CI derives the graph from the
  exact candidate tree and passes those bytes to consumers.
- The local bridge command remains non-authoritative. Protected-PR admission and the single
  `oya-ci-required` context remain the only merge authority under ADR-0515.
- The graph can be inspected after explicit materialization, but it is not an ordinary repository
  authority surface and is absent from a clean checkout by design.
- No feature, migration wave, implementation roadmap, or production rollout is authorized by this
  decision.

## Rejected alternatives

- **Keep the graph tracked and merely add a producer test.** Rejected because the no-generated-
  merge-surface policy would correctly reject the repaired face's own PR diff, and future source
  changes would continue to serialize on derived bytes.
- **Declare the face but leave the broad materializer unchanged.** Rejected because a declared
  producer that the official controller never invokes is an aspirational control, not closure.
- **Allow a special hand-edited or generated-only PR exemption.** Rejected because provenance
  cannot be inferred from a diff label; derive-on-demand removes the authority ambiguity entirely.

## Verification and rollback

Verification requires the control-plane live-corpus test, direct-diff rejection test, exact
registry-to-graph semantic/byte test, materializer invocation test, regenerate-twice freshness
test, materialize-first cross-artifact gate, and protected `oya-ci-required` admission.

This is a one-way authority cleanup. Operational rollback means reverting the controller or
consumer implementation while preserving fail-closed absence; it does not mean restoring a
tracked generated copy. A future reversal requires a new Accepted ADR and an independently
reviewed replacement integrity mechanism.

## Lifecycle

ADR-0620 amends ADR-0613 by applying its accepted pure-projection de-commit invariant to one
additional, explicitly named face. ADR-0613 records the reciprocal `amended_by: [ADR-0620]` edge.
