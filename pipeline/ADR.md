---
doc_class: Owner-ADR
owner: pipeline
status: Active
date: 2026-08-28
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md#ADR-0719-D17a
---

# Pipeline decisions in force

This owner law specializes ADR-0719 D17a for `pipeline/`. Its status describes
the law after this PR lands, not a landed declaration-integrity implementation.

<current_state>

## Current foundation

- `pipeline/core/admission` has layout, path-delta, owner-occupant, occupancy,
  and fan-in rules.
- `pipeline/ports/draft/repository` and its Git adapter provide a current
  read-only seam for merge bases, name-status records, immutable blob bytes,
  tree entries, and kinds.
- `pipeline/facade/path-layout-app` is built from protected source and admits
  candidate repository objects into the existing `presubmit` fan-in.

No versioned SCM-neutral snapshot/owner-fact port, Build declaration call,
canonical ChangeSet application, qualification runtime, repair campaign, or
declaration enforcement has landed.

</current_state>

<repository_facts>

## Decision: Pipeline owns repository facts, not declaration semantics

- **achieves:** Build evaluates the same immutable first-party head for every
  required check without becoming coupled to Git or a future SCM.
- **origin:** the current Git seam is useful evidence but exposes adapter-shaped
  strings and bytes rather than an owner-independent fact contract.
- **rule:** Pipeline MUST provide versioned SCM-neutral immutable snapshot
  identities, complete requested HEAD entries, lossless base-to-head deltas,
  and caller-resolved owner facts bound to one owner-authority identity and
  revision. Base and delta facts are attribution and repair-sharding inputs;
  they never reduce Build's complete-HEAD correctness boundary. Git is the
  required current adapter behind this port, never a Build-core dependency or
  permanent value model. Missing, mutable, lossy, conflicting, or ambiguous
  snapshot or owner facts refuse closed.
- **ensure:** later port and adapter tests cover immutable objects, lossless
  path changes, unsupported entry kinds, owner absence, and owner ambiguity;
  core tests contain no Git command, SHA, checkout, network, or process model.
- **overturn_when:** a five-field Pipeline amendment migrates every adapter and
  consumer to an equally immutable, lossless, SCM-neutral fact boundary.

</repository_facts>

<protected_admission>

## Decision: protected source invokes one Build contract into `presubmit`

- **achieves:** an untrusted candidate cannot select, replace, or skip the
  declaration checker that judges it.
- **origin:** the existing repository-layout admission path already builds
  trusted source while treating candidate content as data and contributes to
  the sole protected `presubmit` context.
- **rule:** after its separate Build contract and qualification land, Pipeline
  MUST invoke that versioned contract only from the existing protected-source
  admission path and feed exactly one result into existing `presubmit`.
  Pipeline never interprets Cargo, Starlark, labels, dependency kinds, or the
  Build relation; it preserves Build refusals and typed output. It MUST NOT add
  a workflow, required context, candidate-controlled executable, candidate
  compile/test proof, native Cargo/Buck invocation, stored corpus, or build
  graph authority.
- **ensure:** protected-source and fan-in tests prove candidate edits cannot
  select an engine/profile or bypass invocation, and that one result reaches
  the existing fan-in.
- **overturn_when:** a five-field amendment retains non-bypassable protected
  evaluation, one `presubmit`, and no second execution or graph-proof plane.

</protected_admission>

<changeset>

## Decision: Pipeline wraps neutral repairs in canonical ChangeSets

- **achieves:** owner-routed repairs remain safe across disjoint successor
  commits without allowing stale partial mutation.
- **origin:** whole-head locks serialize unrelated work, while text hunks and
  destination-only checks omit semantic dependencies.
- **rule:** Pipeline MUST losslessly wrap every source
  `DeclarationRepairSetV1` in canonical ChangeSets. Every ChangeSet preserves
  its source RepairSet identity, whole-set digest, and exact owner-group
  identity, plus complete semantic reads and writes, digest-or-absence
  preconditions, deterministic complete postimages, postconditions, output
  digests, and owner facts. The full set partitions completely and exactly once
  into owner groups with pairwise-disjoint writes; missing, duplicate,
  ambiguous, or overlapping groups refuse. A group routes review only; it never
  grants application authority. Before a future application, Pipeline re-reads
  every declared semantic and owner fact against one current immutable snapshot.
  Any mismatch refuses with no partial state; a disjoint successor remains
  applicable only while all declared facts match.
- **ensure:** later model and fault tests cover read/write mismatch, owner move,
  disjoint successor, overlap, retry, and every compare/publish boundary.
- **overturn_when:** a five-field Pipeline amendment proves equal complete-read
  validation, deterministic complete postimages, atomicity, and owner isolation.

</changeset>

## Rejected destinations

- Cargo or Starlark parsing, label resolution, or configured-graph semantics in
  Pipeline.
- A new SCM product, direct protected-branch mutation, or a general codemod.
- Native-query execution, candidate executable execution, a second workflow or
  context, path/count inventory, baseline, waiver registry, or gate fleet.
