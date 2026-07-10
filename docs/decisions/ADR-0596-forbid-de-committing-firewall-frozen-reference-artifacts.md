---
id: ADR-0596
title: "Forbid de-committing firewall frozen-reference artifacts — make the #828 ratchet-baseline deadlock class impossible"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0616]
amends: [ADR-0595]
depends_on: [ADR-0515, ADR-0539, ADR-0551, ADR-0552, ADR-0595]
related: [ADR-0111, ADR-0363, ADR-0541, ADR-0558]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0596: Forbid de-committing firewall frozen-reference artifacts

## Status

**Superseded by [ADR-0616](ADR-0616-de-commit-firewall-frozen-reference-baseline.md) - 2026-07-09.**
ADR-0616 keeps the #828 deadlock class IMPOSSIBLE but INVERTS the mechanism: instead of forbidding
de-commit of the frozen reference, it REGENERATES the frozen reference from the merge-base source
(so there is no committed blob to empty), and a frozen reference MAY be de-committed IFF its ratchet
policy declares `frozen_reference.source: regenerate-from-merge-base-source`. A committed-git-blob
frozen reference de-committed WITHOUT that declaration still RED-blocks (this ADR's guard, preserved).

Originally **Proposed - 2026-06-23** (door: one-way — the control-plane guard that a firewall
frozen-reference baseline must stay committed; ADR-0616 is the deliberate one-way reversal to the
regenerate-from-source model).

## Context

### The #828 → #830 incident (root cause)

The firewall ratchet (`ci/facade/baseline-ratchet`) is a shrink-only
merge-base baseline ratchet (ADR-0551). It materializes its FROZEN reference — the enumerated
set of currently-failing keys — from the committed git blob at the merge-base:

```
git show <merge_base>:<frozen_reference.face_path>
```

where `frozen_reference.face_path` is declared as DATA in
`ci/facade/baseline-ratchet/ratchet-policy.json`. Today that path is
`ci/facade/artifact-inventory-registry/gate-baseline.generated.json`.

ADR-0595 de-committed six pure-derivation accounting faces (they are content-addressed
projections of the candidate tree, re-derived at gate-time, so they are not contributor merge
surfaces). PR #828 went one face too far: it set the `gate-baseline.generated.json` face to
`materialization_mode: not-tracked-in-git` in
`registry/generated-artifact-control-plane.json` — treating the frozen baseline as if it were
one of the pure-view faces.

But the gate-baseline face is **categorically different** from the five pure-view faces. It is
the firewall's FROZEN reference. De-committing it removed the git blob at the merge-base, so:

```
git cat-file -e <merge_base>:<face_path>   ->  fails
missing_at_merge_base = true               ->  frozen baseline is EMPTY (frozen_empty)
```

An empty frozen baseline means the ratchet has nothing to subtract. Every pre-existing
repo-wide debt key reads as a **NEW regression** on every broad-affected-set PR. The result
was a dev-wide merge deadlock: the firewall went red on every PR for debt those PRs did not
introduce. PR #830 hotfixed it by re-committing the gate-baseline face and re-marking it
`merge-candidate-regenerated`.

### Why a control plane that allowed this is the defect

The generated-artifact control plane (`registry/generated-artifact-control-plane.json`,
enforced by `oya-cloud-ci-generated-artifact-control-plane-app`) is the SSOT for "which
generated artifacts exist and how they materialize." It had a `not-tracked-in-git` mode
(legitimate for pure-view faces) but **no rule preventing that mode from being applied to a
frozen-reference baseline**. A human edit to a single JSON field could (and did) empty the
ratchet baseline with zero gate resistance. Per the friction-is-process-failure doctrine, the
fix is not "re-commit the face once" (#830 already did that) — it is to make the deadlock
**class** impossible at the control-plane policy layer.

### The frozen-reference vs pure-view artifact-class distinction

This ADR records the durable distinction the gate now enforces:

| Class | Materialization | May be de-committed? | Consumer boundary |
|---|---|---|---|
| **Frozen-reference / baseline** (e.g. the firewall `gate-baseline` face) | `git show <merge_base>:<face_path>` — read from the committed git blob at the merge-base | **NO** — must stay a committed git blob on the integration branch | The ratchet reads HISTORY (the merge-base), which only exists for committed paths |
| **Pure-view** (accounting-registry, decision-crosswalk, enforcement-inventory, enforcement-liveness, ttl-policy) | re-derived from the checked-out candidate tree at gate-time | **YES** (ADR-0595) | Consumers re-derive content from the present tree; no history read |

The defining property: a frozen reference is consumed across the **merge-base git boundary**
(it needs the path to exist in history), whereas a pure view is consumed from the **present
candidate tree**. De-committing a path is safe iff no gate reads it from history.

## Decision

The generated-artifact control-plane gate gains a new rule:

> **`frozen_reference_artifact_must_stay_committed`** — RED when a declared artifact whose
> `path` is a firewall FROZEN-REFERENCE is declared with a de-commit `materialization_mode`
> (`not-tracked-in-git`, or any future mode whose semantics is "absent from the committed tree
> / derive-on-demand").

The frozen-reference set is derived **universally from DATA, not hardcoded paths**:

- **Primary signal (authoritative, repo-agnostic):** the path appears as
  `frozen_reference.face_path` in the repo's `ratchet-policy.json` (the firewall ratchet
  policy, ADR-0551). The gate reads the ratchet-policy JSON and extracts the frozen-reference
  set; it tolerates either a single `frozen_reference` object or an array of them so adopters
  can declare multiple frozen baselines.
- The de-commit-mode set is also DATA (a named constant list), so adding a new
  derive-on-demand materialization mode extends the list, not the predicate.
- The live gate-baseline manifest row may use `materialization_mode: main-branch-materialized`
  with `merge_policy: controller-owned-main-materialization`. That mode still means the baseline
  stays committed on the integration branch for merge-base reads, but contributor PRs do not own
  generated baseline byte churn; cloud-ci/controllers materialize it from source.

The gate carries **no hardcoded oyatie face paths**. It works on any repo that ships its own
`ratchet-policy.json` + control-plane manifest: the same predicate forbids de-committing
whatever that repo declares as its frozen reference.

### Properties (7-property bar)

- **UNIVERSAL** — the frozen-reference set is pure data from `ratchet-policy.json`; zero
  hardcoded paths in the predicate.
- **HERMETIC** — a pure `serde_json` predicate over the committed control-plane manifest + the
  committed ratchet-policy JSON. No shell, network, clock, rand, or VCS call in the verdict.
- **AUTOMATED (flag policy)** — this is a human-decision guard, not a mechanical mis-formatting:
  whether a frozen reference should be de-committed is an architectural decision that requires
  re-pointing the ratchet policy first. There is no safe auto-fix (silently re-committing a
  face the author deliberately de-committed, or silently flipping the mode, would mask intent).
  `no_autofix_reason`: **the remediation is an architectural decision (keep the face committed,
  OR re-point `ratchet-policy.json` to a different frozen reference first); an auto-fix would
  guess the author's intent and could re-introduce the very deadlock by flipping the wrong
  field.** The gate ships RED with a remediation message instead.
- **Wired into oya-ci-required** — the live corpus gate
  (`oya-cloud-ci-generated-artifact-control-plane-app-gate`) reads the real ratchet-policy +
  manifest and runs the guard; it is already in the `oya-ci-required` gate matrix.

## Consequences

- **Positive:** the #828 deadlock class is structurally impossible. A future PR that tries to
  de-commit the frozen baseline (the exact #828 edit) is RED at presubmit with a message
  citing the incident, instead of green-then-deadlock.
- **Positive:** the frozen-reference vs pure-view distinction is now machine-enforced policy,
  not tribal knowledge, and is portable to any oya-ci adopter.
- **One-way door:** once shipped, "a frozen reference stays committed" is a committed policy;
  legitimately de-committing a frozen reference requires first re-pointing `ratchet-policy.json`
  (a reviewed change) so the path is no longer a frozen reference.
- **Neutral:** the rule is inert when no ratchet policy declares a frozen reference (empty set),
  so adopters without a frozen-baseline ratchet are unaffected.

## Verification

- RED fixture (`frozen_reference_decommitted_is_red_the_828_class`) + a live synthetic-#828
  manifest run: marking the gate-baseline `not-tracked-in-git` fires
  `frozen_reference_artifact_must_stay_committed` and the verdict is RED.
- GREEN fixture (`frozen_reference_committed_with_decommitted_pure_views_is_green`) + the live
  current-dev manifest: frozen baseline committed + five pure-view faces de-committed → GREEN.
- The frozen-reference-unaware legacy entry point stays inert
  (`frozen_reference_guard_is_inert_without_a_ratchet_policy`), preserving the diff-policy
  bridge's behavior.
