---
doc_class: Owner-PRD
owner: pipeline
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - pipeline/ADR.md
---

# Pipeline repository admission and repair requirements

<product_boundary>

Pipeline is the SCM-neutral graph, queue, schedule, and promotion product used
by tenant #0 and sold through the same engine; Compute executes its work. The
current Git/GitHub Actions path is an adapter, not product core.

For first-party Cargo/Buck declaration integrity, Pipeline turns untrusted
repository candidates into protected admission verdicts and reviewable,
preconditioned repair proposals. It acquires immutable repository facts,
resolves ownership, invokes Build's one check-only semantic engine, applies
neutral repair sets through the canonical ChangeSet boundary, and orchestrates
qualification and owner-sharded repair campaigns.

Pipeline does not own Cargo/Buck parsing or conformance semantics, does not
execute a configured Buck graph for this check, and does not create another
compile plane or required context. Git is the required current SCM adapter; the
product contract remains SCM-neutral so an owned SCM can implement it later.

</product_boundary>

<users>

- Authors receive one deterministic, semantic declaration verdict from the
  protected `presubmit` path, with violations attributable to changed inputs.
- Owner teams receive reviewable repair PRs limited to their shard, with exact
  preconditions, complete postimages, and no manual label surgery.
- Reviewers can prove which protected engine/profile, repository facts, owner
  facts, and ChangeSet application produced a candidate.
- Operators can qualify a new parser/grammar profile, measure it, repair all
  current drift, activate it atomically, keep the last-enforced immutable profile
  authoritative while a replacement shadows, and block relevant changes when
  no valid enforced profile is available.
- Future SCM adapters can supply the same snapshot, delta, owner, compare, and
  publish semantics without changing Build's engine.

</users>

<landed_scope>

## Current foundation

The landed Pipeline can derive a merge base, read raw NUL-delimited Git
name-status, reduce that stream to occupied and live layout-candidate path sets,
read commit blobs and entry kinds through a draft repository port, map
recognized paths to owner occupants, run layout checks from protected source,
and feed the result into one `presubmit`. No versioned SCM-neutral,
status-preserving lossless-delta or immutable-snapshot contract, declaration-
engine integration, canonical write/CAS port, neutral ChangeSet application,
qualification state, or repair campaign runtime has landed.

</landed_scope>

<requirements>

## Repository facts and trust

- Represent snapshot identity as an opaque, versioned SCM-neutral value and
  acquire complete HEAD declaration bytes plus base bytes for changed
  declaration endpoints from immutable snapshots, never from a mutable working
  tree.
- Preserve add, modify, delete, rename, copy, and type-change information in a
  bounded lossless delta. Use the delta only for attribution, routing, and
  repair sharding; supply the complete HEAD declaration graph to Build.
- Bind one protected owner-authority identity/revision and an expected owner or
  owner absence for every declaration and proposed write. Refuse missing,
  conflicting, stale, or ambiguous ownership rather than routing to a default
  owner.
- Treat repository bytes, paths, deltas, Build outputs, SCM responses, and
  candidate metadata as untrusted data. Only ruleset-selected protected source,
  qualified profile identities, and configured adapter capabilities may direct
  execution.
- Keep Git process behavior in the current adapter. Core values and Build
  interfaces remain independent of Git command syntax, SHA width, checkout
  layout, hooks, environment, network, and host identity.

## Protected declaration admission

- Invoke exactly one versioned Build engine with immutable base/head identities,
  complete HEAD declaration bytes, lossless path delta, and caller-resolved
  owner facts.
- Preserve the engine's sorted typed violations and deterministic
  `DeclarationRepairSet` values without re-parsing declarations or weakening a
  refusal.
- Preserve the first-party boundary: `third-party//` and generated
  `third-party/BUCK` stay on Build's Reindeer reconciliation path and never
  enter this engine.
- Execute from protected source in the existing layout admission seam and feed
  the existing `presubmit`. Before first activation, shadow results and failures
  are nonblocking qualification evidence. After activation, the active profile
  supplies the verdict; its invalidity or unavailability enters
  `EnforcementBlocked` and refuses relevant declaration changes rather than
  skipping or returning to shadow-only behavior.
- Do not add required `buck2`, a candidate compile, a Cargo/Buck parity build,
  a second protected context, or a stored declaration census.

## ChangeSet application and repair delivery

- Wrap one owner shard at a time in a versioned canonical ChangeSet. Bind the
  protected engine/profile, source provenance, violations, full semantic read
  and write sets, expected digest-or-absence for every set member, complete
  resulting bytes/mode or explicit absence for every write, one owner-authority
  identity/revision, and sorted expected owner-or-absence preconditions for
  every semantic read, semantic write, and proposed-write path. Treat the owner
  shard as routing metadata only.
- Re-read and compare all semantic inputs against the selected current
  snapshot before application. Re-load and compare the bound owner-authority
  identity/revision, re-resolve every bound path, and require every owner-or-
  absence result to match before using routing metadata.
- Construct either one complete successor snapshot/commit or none. A crash,
  conflict, adapter uncertainty, ownership change, or invalid postimage cannot
  expose a partial candidate or be reported as success.
- Permit a repair after unrelated commits when all declared semantic
  preconditions and owner facts still match. Every write carries an expected
  precondition and may also be a semantic read. Refuse when the current or an
  intervening snapshot makes any declared semantic read or write precondition
  mismatch; whole-head inequality alone is not a conflict.
- Deliver repairs as isolated protected PRs against `dev`, with independent
  review and normal occupancy/admission. Never write directly to `dev`, merge
  on CI observation, or combine owner shards into one blast radius.

## Qualification and activation

- Qualify each exact grammar-profile identity first for application-ready repair
  proposals, then for admission, using adversarial fixtures, deterministic
  replay, protected differential evidence, fault injection, and side-effect
  detection. Repair qualification never blocks admission.
- Run differential `cargo metadata --offline --locked` and non-building Buck
  queries only in a protected out-of-presubmit qualification harness. Those
  tools are evidence producers, not calls made by the engine or required check.
- Shadow the candidate engine, reach repair qualification, repair every
  detected current violation through owner-sharded PRs, re-evaluate the complete
  current graph, finish admission qualification, and activate only at zero
  violations. Do not create a baseline, allowlist, count threshold, or
  grandfather rule.
- Invalidate qualification mechanically when any bound parser, grammar,
  prelude, macro, rule contract, engine, or result-schema identity changes.
- Before first activation, invalidation returns only the candidate profile to an
  unqualified nonblocking state. After activation, qualify a changed candidate
  beside the last-enforced immutable active profile and swap atomically. If the
  active profile itself is invalid or unavailable, enter `EnforcementBlocked`;
  do not restore nonblocking admission.

## Operability and security

- Emit correlation-safe receipts for repository snapshot, protected engine and
  grammar profile, owner resolution, verdict, qualification state, ChangeSet
  identity, compare result, and published PR/commit outcome.
- Keep declaration bytes and postimages out of logs and metrics. Record bounded
  counts, digests, duration, refusal class, owner shard, and correlation IDs.
- Bound total input bytes, entries, delta records, owner facts, repair files,
  postimage bytes, execution time, and diagnostic cardinality before work.
- Separate orchestration, implementation, review, and merge authority. An
  engine receipt or green `presubmit` is observation, never review APPROVE.

</requirements>

<service_objectives>

## Qualification targets, not landed SLO claims

- **Safety:** zero false-green outcomes across every admitted/refused grammar
  form and modeled relation; every injected incomplete or mismatched fact
  refuses.
- **Coverage:** 100% of relevant candidate changes trigger one complete-HEAD
  evaluation; delta-only evaluation is never accepted as proof.
- **Determinism:** identical immutable inputs produce byte-identical verdict,
  repair-set, ChangeSet, and receipt identities in 100% of replays.
- **Atomicity:** fault injection at every compare/write/publish boundary exposes
  zero partial successor snapshots or partially published repair commits.
- **Availability semantics:** after first activation, active protected source,
  profile, repository facts, owner facts, or engine-result loss/invalidity
  refuses relevant changes in 100% of cases. Before first activation, the same
  faults remain visible nonblocking qualification failures.
- **Latency:** warm p99 declaration evaluation is at most 30 seconds on the
  qualified current-repository profile. The complete protected layout job stays
  inside its existing timeout with at least 50% timeout headroom.
- **Work bound:** work is linear in admitted declaration bytes plus modeled
  nodes/edges. No scan count, package census, or current path total becomes a
  policy constant.

Before first activation, missing latency/work evidence keeps enforcement off.
After activation, an unqualified replacement does not displace the active
profile; if the active profile cannot remain valid, `EnforcementBlocked`
refuses relevant changes. Targets are not rewritten after measurement.

</service_objectives>

<failure_definition>

## Failure means

- After activation, a relevant change receives no authoritative verdict, a
  delta-only verdict, a verdict from candidate-selected/shadow/unqualified
  source, or a second declaration meaning.
- Missing/ambiguous ownership, malformed SCM facts, unsupported entry kinds, an
  unknown grammar shape, engine crash/timeout, or malformed output is accepted.
- A repair omits a semantic or per-path owner precondition, owner-authority
  identity/revision, or complete postimage; treats `owner_shard` as authority;
  applies after a declared-fact conflict; rejects a truly disjoint commit;
  crosses owners; partially publishes; or bypasses protected PR review.
- Activation occurs with a known current violation, unqualified profile, false
  green/failure, nondeterministic result, stored baseline, or unresolved fault.
- After enforcement has activated, profile invalidation/loss restores a
  nonblocking state, a shadow candidate supplies the authoritative verdict, or
  replacement occurs without an atomic admission-qualified transition.
- Required admission invokes `buck2`, claims configured-graph correctness,
  compiles the candidate again, adds another required context, or revives a
  corpus/check fleet.

</failure_definition>

<non_goals>

- Defining Cargo/Buck grammar or label semantics in Pipeline.
- Replacing Buck2, Cargo metadata, rust-analyzer, Git, or a future owned SCM.
- General AST merging, text-hunk rebasing, repository-wide write locking, or
  direct protected-branch mutation.
- A user-facing repair CLI, compatibility targets, manual repair instructions,
  gate fleets, waiver databases, or declaration census artifacts.

</non_goals>
