---
doc_class: Owner-ADR
owner: pipeline
status: Accepted
date: 2026-08-28
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Pipeline decisions in force

This file specializes ADR-0719 for `pipeline/`. It defines Pipeline's
repository trust, admission, repair-application, and orchestration boundaries;
it does not claim that source-declaration conformance, canonical ChangeSet
application, qualification, or enforcement has landed.

<current_state>

## Evidence at owner-law adoption

- `core/admission` has pure repository-layout, path-delta, owner-occupant,
  occupancy, and fan-in rules. It has no first-party Cargo/Buck declaration
  integration or canonical ChangeSet application.
- `ports/draft/repository` and `adapters/draft/repository-git` expose the
  Pipeline-local read port and required current Git adapter for merge base,
  raw NUL-delimited Git name-status bytes, immutable blob bytes, tree
  enumeration, and entry kind. `core/admission` derives occupied and live
  layout-candidate path sets from that stream. No versioned SCM-neutral,
  status-rich lossless-delta contract, snapshot value, write/CAS port, or
  owner-fact contract has landed.
- `facade/path-layout-app` reads candidate objects through the repository port
  and emits one fail-closed layout verdict. It does not invoke Build's
  declaration engine.
- `.github/workflows/presubmit.yml` builds that application from
  ruleset-selected protected source, treats the candidate checkout as data, and
  feeds one `presubmit` fan-in. It has no qualified declaration verdict, repair
  campaign, or activation state.

These are evidence statements at the adoption base, not maturity claims for the
destination described below.

</current_state>

<product_charter>

## Decision: one execute engine, with hosted SCM/CI as temporary adapters

- **achieves:** tenant #0 and sold Pipeline converge on one graph, queue, and
  scheduler instead of preserving a repository-specific CI product.
- **origin:** Pipeline currently has repository admission kernels and hosted
  workflow adapters, while ADR-0719 D-18 rejects treating workflow YAML,
  GateRun/Tide, Cargo, or a Pipeline-owned worker fleet as the product.
- **rule:** `pipeline/` MUST own one SCM-neutral execute engine for graph,
  queue, schedule, and promotion orchestration, with internal and sold facades
  over the same semantics. Pipeline consumes a separately adopted Compute
  execution contract; Git/GitHub Actions and GitHub merge queue MUST remain
  replaceable current adapters. Tenant #0 MUST
  retain one required `presubmit`; Cargo nextest remains its temporary execute
  path until Pipeline serves the Buck2 graph under the ADR-0716 cutover bar.
- **ensure:** owner review rejects workflow YAML or hosted-provider types in
  core, a second worker cluster/execute engine, per-language CI products,
  GateRun/Tide-as-core, and dual Cargo/Buck merge proof.
- **overturn_when:** a founder-accepted five-field amendment changes D-18's
  product/adapter/cutover split and migrates tenant #0 without parallel truth.

</product_charter>

<ownership_boundary>

## Decision: Pipeline owns trust and orchestration, not declaration semantics

- **achieves:** one declaration-integrity meaning while Pipeline can schedule it
  across Git today and another owned SCM later.
- **origin:** recurring Cargo/Buck drift needs protected admission and automated
  repair, but copying declaration parsing or conformance rules into Pipeline
  would create a second graph authority beside Build.
- **rule:** Product owners own semantic intent, postconditions, and acceptance;
  Build owns neutral Cargo/Buck analysis and transformation. Pipeline MUST own
  SCM-neutral immutable snapshot acquisition, path deltas, caller-resolved owner
  facts, protected execution, admission fan-in, ChangeSet application,
  qualification, campaigns, retries, protected review/admission, and merge.
  Pipeline consumes, but MUST NOT prescribe or reinterpret, the separately
  adopted Build contract: its typed violations and `DeclarationRepairSetV1`.
- **ensure:** dependency review permits one opaque Build check-only contract and
  rejects a Pipeline Cargo/Buck parser, label resolver, configured Buck graph
  oracle, declaration corpus, duplicate repair generator, or semantic claim.
- **overturn_when:** a founder-accepted five-field amendment reallocates both
  sides atomically while preserving one semantic engine, one protected verdict,
  and SCM neutrality.

</ownership_boundary>

<repository_facts>

## Decision: immutable repository facts cross an SCM-neutral port

- **achieves:** candidate evaluation and repair application bind exact bytes and
  identities rather than mutable checkouts, ambient filesystem state, or
  Git-only concepts.
- **origin:** the live Git adapter already reads commit objects, but its string
  SHAs, byte streams, and owner lookup are not yet one versioned repository-fact
  contract suitable for another SCM.
- **rule:** Pipeline MUST provide versioned immutable base/head snapshot
  identities, exact regular-blob entry facts/bytes/modes for both changed
  declaration endpoints, a lossless path delta, and complete protected-requested
  source-surface bytes, plus one protected owner-authority identity/revision and
  owner-or-absence CAS facts for bound paths. Absence is only for non-write
  semantic reads; every semantic/proposed write has one concrete expected owner.
  Git is the required current adapter behind that port, not a Build dependency
  or permanent value model.
  Missing, conflicting, lossy, mutable, symlink, gitlink, or unsupported facts
  MUST refuse. Pipeline never accepts caller-authored expected nodes, edges,
  semantic facts, or conformance answers: Build derives the complete HEAD graph;
  deltas/base bytes only trigger, attribute, or shard.
- **ensure:** adapter contract tests replay immutable objects, renames, copies,
  deletes, non-UTF-8 and malformed records, unsafe entry kinds, owner gaps and
  ambiguity; SCM-neutral kernel tests contain no Git command, SHA grammar,
  checkout path, network, clock, or process dependency.
- **overturn_when:** a five-field owner decision proves an equally immutable,
  lossless, fail-closed repository-fact model and migrates every adapter and
  consumer in one reviewed sequence.

</repository_facts>

<protected_admission>

## Decision: declaration admission extends the existing protected layout seam

- **achieves:** a candidate cannot weaken the engine that judges it, and merge
  still has one proof plane and one required context.
- **origin:** the live layout job already compiles Pipeline from
  ruleset-selected protected source while the candidate is data; a candidate-
  compiled checker or separate workflow would either be self-bypassable or
  become a second admission product.
- **rule:** Pipeline MUST invoke ruleset-selected protected Build source through
  the existing trusted layout application and feed one verdict into the existing
  `presubmit`. Before first activation, unqualified/shadow candidate results and
  failures MUST remain nonblocking qualification evidence. After any activation,
  the immutable active profile MUST remain authoritative while a replacement
  shadows; if that active profile is invalid or unavailable, Pipeline MUST enter
  `EnforcementBlocked` and relevant declaration changes MUST refuse until an
  admission-qualified replacement activates atomically. Pipeline MUST NOT fall
  back to pre-activation nonblocking behavior, add a required context,
  standalone Cargo/Buck compile lane, required `buck2` invocation,
  candidate-supplied checker, or compatibility target.
- **ensure:** protected-policy tests prove candidate edits cannot select engine
  source/profile or skip invocation; fan-in tests prove exactly one verdict;
  state tests distinguish never-enforced shadow failure from post-activation
  `EnforcementBlocked`; fault injection covers unavailable, panicking,
  timing-out, malformed, and unqualified engines without candidate mutation.
- **overturn_when:** a founder-accepted five-field amendment replaces the
  protected-source mechanism while retaining non-bypassability, one
  `presubmit`, and no second compile or configured-graph proof.

</protected_admission>

<repair_application>

## Decision: neutral repair sets become atomic, preconditioned ChangeSets

- **achieves:** owner-grouped mechanical repair remains safe across unrelated
  commits and can never partially overwrite a changed semantic input.
- **origin:** binding a repair only to one repository head serializes disjoint
  work, while applying text hunks or checking only destination preimages misses
  semantic read dependencies and permits stale repair.
- **rule:** Build returns exactly one canonical `DeclarationRepairSetV1` per
  evaluation, carrying its whole-set digest/identity and canonical non-empty
  owner groups in canonical owner order; a zero-action set has zero groups.
  Pipeline MUST validate and
  map every supplied group one-to-one to exactly one canonical ChangeSet, never
  derive, invent, or regroup ownership. Each preserves whole-set digest/identity
  and exact-group identity, engine/profile binding, reads/writes, preconditions,
  typed postconditions, deterministic complete postimages, every path-bound
  canonical postimage digest, exact owner-group output digest, and owner facts.
  Every action/proposed-write path occurs exactly once; group writes are
  pairwise-disjoint. Semantic/proposed writes require a concrete expected owner;
  `OwnerExpectation::Absent` is only a non-write-read CAS fact. Empty,
  extraneous, missing, duplicate, ambiguous, wrong-owner, cross-owner,
  incomplete, overlapping, absent-owner, split whole-set digest/identity,
  missing/invalid complete postimage, or digest mismatch groups/writes MUST
  refuse. Before a new snapshot Pipeline CAS-compares every
  bound fact; mismatch refuses with no partial state. A later disjoint commit applies when declared facts still match;
  application publishes only an isolated protected PR against `dev`.
- **ensure:** model/property tests cover zero-action/zero-group, concrete-owner
  write, absent-owner rejection, exact-once path/action coverage, non-empty
  canonical groups, typed postconditions, deterministic complete postimages,
  every path-bound postimage/group-output digest, every listed refusal, conflicts, retries,
  crashes, and byte-identical replay.
  Every failure sees old state or no published commit.
- **overturn_when:** a five-field Pipeline decision proves an alternative with
  equal complete-read validation, atomic complete postimages, disjoint-commit
  applicability, owner isolation, retry safety, and protected review.

</repair_application>

<qualification_and_activation>

## Decision: qualify, repair clean, then enforce without a baseline

- **achieves:** declaration admission starts with measured correctness and zero
  grandfathered drift instead of institutionalizing known violations.
- **origin:** nonblocking Buck smoke discovered stale labels after merge;
  immediately blocking an unqualified parser would trade false greens for broad
  false failures and manual exceptions.
- **rule:** Pipeline MUST orchestrate adversarial/protected-differential
  qualification outside required presubmit, then campaigns, before activation.
  Its planner uses declared repair-group dependency/fanout facts, not Cargo/Buck
  semantics, detects SCCs, and emits topologically ordered closure-complete
  waves; an incomplete or unresolved closure halts. Owner groups are distinct
  from dependency closure. One canary completes before wider waves; explicit
  halt, repair, and rollback remain available. Pipeline schedules only through a
  separately adopted Compute contract. Pipeline MUST invoke
  `cargo metadata --offline --locked --no-deps --format-version 1` and
  non-building `buck2 uquery` only in protected out-of-presubmit qualification,
  and MUST NOT invoke them from required admission. First activation MUST require
  an admission-qualified profile and zero current violations without a baseline,
  count/path allowlist, census artifact, or waiver fleet. After activation, a
  new parser/grammar/prelude/macro/rule-contract identity MUST shadow separately
  while the last-enforced immutable profile remains active. A replacement MUST
  activate atomically only after full admission qualification. Loss or
  invalidation of the active profile MUST enter `EnforcementBlocked`, never a
  nonblocking pre-activation state.
- **ensure:** a qualification state-machine test exercises never-enforced
  unqualified/shadow/repair states, atomic first activation, active-plus-shadow
  replacement, atomic replacement, and fail-closed `EnforcementBlocked`;
  protected differential fixtures cover every admitted/refused shape and
  current-drift repair; activation evidence is independently reviewed.
- **overturn_when:** a founder-accepted five-field amendment demonstrates a
  safer activation protocol with equal protected differential evidence, no
  grandfathered drift or census, and fail-closed protection after first
  activation.

</qualification_and_activation>

## Rejected destinations

- Git commands, Git object IDs, checkout paths, owner resolution, ChangeSet
  application, or repair-campaign behavior inside the Build core.
- Cargo/Buck syntax parsing, label semantics, grammar policy, or an independent
  declaration graph inside Pipeline.
- Candidate-compiled admission, candidate-selected profiles, or candidate
  execution of Starlark.
- A second workflow fleet, required context, configured Buck graph claim, Cargo
  versus Buck compile matrix, census JSON, learned baseline, path/count freeze,
  waiver registry, or compatibility target.
- Whole-snapshot locks that invalidate semantically disjoint repairs, text-hunk
  application without complete postimages, or destination-only preconditions.
- Direct writes to `dev`, invisible working-tree mutation, partial application,
  self-approval, or treating green CI as review APPROVE.
- A declaration-specific campaign API, one-off analyzer/runner/controller/evidence
  plane, or a production CLI that is not a retirement-marked diagnostic.
