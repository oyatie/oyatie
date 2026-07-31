---
id: ADR-0631
title: "Amend the closed capability registry: admit infra/ as a non-crate meta directory, repair the ten dead absorb prefixes, and record policy/ as a forward declaration"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-31
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0562, ADR-0615, ADR-0628]
amends: [ADR-0562, ADR-0615]
related: [ADR-0280, ADR-0536, ADR-0555, ADR-0627]
related_specs:
  - /specs/capability-registry.json
milestone: W3
---

# ADR-0631: Amend the closed capability registry

## Status

**Proposed — 2026-07-31.** Amends `specs/capability-registry.json`, which is `closed: true` with
`placement_authority: ADR-0562`, previously amended by ADR-0615. A closed registry can only be
changed by a decision record; this is that record.

## Context

Three defects were measured against `origin/dev@96da99d14`.

### 1. `infra/` exists and the registry never authorized it

`infra/` is a tracked top-level directory holding 81 files across 16 subdirectories. It appears in
neither `capabilities[]` nor `meta_directories[]`, and no `absorbs_current_dirs` prefix claims it.

The membership lint nevertheless passes it, because
`ci/facade/module-membership/capability-membership-policy.json` carries a **separate,
hand-maintained** `non_crate_top_level_dirs` list (15 entries, `infra` among them) that is unioned
into `allowed_top_level_dirs`. Nothing derives that list from the closed registry — unlike
capability roots and meta directories, which
`ci/facade/cross-artifact-agreement/src/registry_policy_sync.rs` re-derives and cross-checks. So the
ADR-0562 §6 "closed top-level set" has a 15-entry hole that the closed-set authority never saw.

`infra/` is the largest and most consequential entry in that hole: it is the fleet substrate.

### 2. Ten `absorbs_current_dirs` prefixes are dead

Of 97 declared prefixes, 86 resolved and **11 did not**. Every one of the eleven is explained, and
the explanations split into two different classes that call for two different repairs:

| dead prefix | class | evidence |
|---|---|---|
| `cloud/cloud-cell` | vacated | `f2a3243ac` homed cell |
| `cloud/cloud-capacity` | vacated | `f2a3243ac` homed cell |
| `cloud/cloud-observability` | vacated | observability move |
| `cloud/cloud-compute` | vacated | compute move |
| `cloud/cloud-ci` | vacated | ci move |
| `cloud/cloud-finops` | vacated | `31aa56ec6` move-21 homed billing |
| `cloud/cloud-marketplace` | vacated | marketplace move |
| `oya/search` | vacated | `463d125f9` move-16 homed data |
| `oya/ops` | vacated | `6fb63de57` move-11 homed console |
| `oya/policy` | vacated | `07c17470c` move-18 homed iam |
| `policy` | **forward** | never created; the ADR-0615-extracted capability's destination root |

The widely-circulated figure of "69 of 73 dead" is **false** and is corrected here: the measurement
is 11 of 97 (86 live).

A dead prefix maps no crate, so it is not a live false-green today. It is a correctness defect of a
different kind: it asserts a source directory still exists, which is exactly the reading a later
migration lane will act on. `oya/policy` is the sharpest case — its crates went to `iam/` at
`07c17470c`, so the `policy` capability appears to own a source it does not.

### 3. Two premises that drove this wave were false, and are recorded so they stop propagating

- **"`oya/application` (8 crates) and `oya/workplace-integration` (1) are named nowhere."** False.
  Both are `membership_lint_coverage.app_products.current_dirs` entries (registry lines 485 and 517
  pre-amendment), which the membership lint reads as `meta:app/` homes. Confirmed by two
  differently-shaped probes: structural JSON walk, and raw `grep` of the file text.
- **"Nothing can move into or out of an unregistered root while the registry is closed,"** applied
  to `infra/`. False as a mover-blocker. `infra/` contains **zero** `Cargo.toml`
  (`git ls-tree -r origin/dev infra/ | grep -c Cargo.toml` = 0), and the registry closes the
  *capability* (crate) axis only, as `app_products_note` states verbatim. Every destination the
  later waves need — `intelligence`, `app/`, `base/`, `build/` — is already registered. No crate
  move was blocked by the registry. Admitting `infra/` closes a governance hole, not a mover
  blocker, and this ADR does not claim otherwise.

## Decision

### `infra/` is a META directory with `owns_crates: false`

Justified from the charters, not from name similarity:

- It is not a capability. A capability in this registry is an *engine* with `core/ports/adapters/
  facade` faces. `infra/` has no crates at all, so it has no engine and no faces.
- It is not absorbed by the `iac` capability, despite the obvious name pull. `iac` owns the Rust
  code that **authors and reconciles** declarations. `infra/` is the **rendered fleet-level output**
  those engines act on. Folding the output into the engine's absorb list would make the membership
  lint treat rendered manifests as capability crates the moment either grows one, and would erase
  the port seam ADR-0562 exists to preserve.
- It matches the existing meta charter shape used by `governance/`, `build/`, and `third-party/`:
  off the runtime ladder, owns zero crates.

`owns_crates: false` is declared **explicitly**, not omitted. `registry_policy_sync` defaults the
field to `true` when undeclared so a new meta dir fails *closed* into scan coverage; an omitted
field would therefore demand `infra` in `module-membership.scan_roots` and turn the gate red.

### The ten vacated prefixes are removed; `policy` is retained and declared forward

The repair differs by class, because the two classes mean opposite things.

- The **ten vacated** prefixes are deleted. Provenance is not lost: each vacating commit is named in
  the table above and `git log -- <path>` recovers the rest. Adding a parallel provenance array
  would duplicate what version control already stores.
- The **one forward** prefix, bare `policy`, is retained and given an explicit
  `forward_declared_root` block with a reason and a retirement condition. This mirrors the
  distinction ADR-0628 already draws in
  `ci/facade/scan-root-liveness/scan-root-liveness-policy.json`, which carries `policy` as a
  `forward_declarations` entry for four other gate policies. Deleting it would also narrow the
  `capability_top_level_roots` set that `registry_policy_sync` derives, quietly removing `policy`
  from the roots three policies are *required* to carry.

All ten deletions are nested paths containing `/`. None is a top-level root, so
`capability_top_level_roots` is unchanged and no derived policy list moves.

### `infra/`'s 16 subdirectories are NOT dispositioned here

Recorded as `infra_subdir_disposition` with `decided: false` and `blocks_crate_moves: false`, so the
closed registry does not imply that admitting the root settled its contents.

The measured cost of deciding it: **26 distinct `docs/decisions/*.md` files name an `infra/<subdir>`
path verbatim**. Moving such a path turns each into a dangling pointer. Ten of the 26 are
Accepted-class — ADR-0117, ADR-0148, ADR-0370, ADR-0371, ADR-0374, ADR-0375, ADR-0378, ADR-0379,
ADR-0380 (`Accepted (amendment)`), ADR-0515 — and editing an Accepted body implies an `Amended`
status flip plus cross-artifact propagation. That is a separate, expensive lane and it does not
belong in a registry amendment. The remaining 16 are 10 `Proposed`, 5 `Superseded`, 1 `Rejected`.

Two subdirs, `cilium` and `observability`, carry **zero** ADR path anchors and are therefore the
cheapest candidates whenever that lane runs.

## Consequences

**Zero-delta by construction.** The amendment was designed so every consumer's finding set is
unchanged:

- `registry_policy_sync` — `infra` is already present in `module-membership.allowed_top_level_dirs`
  and in `repo-root-hygiene.allowed_root_dirs`, which is the entire requirement for a
  non-crate-owning meta dir. Meta dirs are deliberately not cross-required in tier-dependency. The
  behaviour is pinned by the pre-existing unit test
  `a_non_crate_owning_meta_dir_is_not_required_in_scan_roots`.
- `module-membership` — its `parse_mapping` reads `capabilities[].absorbs_current_dirs` and the
  `membership_lint_coverage` block. It does **not** read top-level `meta_directories`, so `infra/`
  is invisible to it. The ten removed prefixes matched no crate, so no crate changes home.
- `crate-registration` — its `load_capability_set` reads only `membership_lint_coverage`
  (`absorbs_current_crate_globs`, `app_products.meta_dir`, `meta_directory_absorbs[].meta_dir`),
  none of which this ADR touches, so the accepted-slug set is byte-identical.
- No consumer uses `deny_unknown_fields`, so the new `forward_declared_root` and
  `infra_subdir_disposition` keys are inert to every reader.
- `tools/oya-reorg-codemod-app` does not read the registry at all; its only mention is a doc
  comment.

**What this does not fix.** `non_crate_top_level_dirs` remains hand-maintained and
un-registry-derived; `infra/` is now authorized, but the other 14 entries are not. Making that list
registry-derived is a `registry_policy_sync` change and belongs with the scan-root-drift wave, not
here.

**Known residual.** `specs/capability-registry.json` is outside the `scan-root-liveness` collector's
universe, which walks `ci/facade/*/*.json` only, and `absorbs_current_dirs` is not one of its
`coverage_bearing_keys`. So this repair is a one-time correction with no detector: the prefixes can
rot again. Registering the registry with that gate is the recurrence-prevention and is recorded here
as follow-up rather than smuggled into this amendment.

## Alternatives considered

- **Amend ADR-0615 in place.** Rejected. ADR-0615 is `Accepted`; editing its body implies an
  `Amended` flip plus propagation — the same expense this ADR declines to pay for the `infra/`
  subdir anchors. A new Proposed ADR is the cheaper, reversible door.
- **Register `infra/` as a capability.** Rejected. It has no crates, no engine, and no faces, and it
  would then be required in `tier-dependency` roots and `scan_roots`, making the amendment
  non-zero-delta for no coverage gain.
- **Fold `infra/` into the `iac` capability's absorbs.** Rejected. Conflates the authoring engine
  with its rendered output and erases the port seam.
- **Keep the ten dead prefixes and add a `historical_absorbed_dirs` mirror.** Rejected as
  duplicated provenance; `git log -- <path>` already answers it, and each vacating commit is named
  above.

## Governed surfaces

```
specs/capability-registry.json
docs/decisions/ADR-0631-amend-the-closed-capability-registry-admit-infra-and-repair-dead-absorb-prefixes.md
```
