---
doc_class: Owner-SPEC
owner: pipeline
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - pipeline/ADR.md
  - pipeline/PRD.md
---

# Pipeline repository admission and repair contract

<trust_boundary>

## Trust zones

- Candidate repository bytes, paths, modes, deltas, metadata, and proposed
  postimages are untrusted data. Pipeline parses them with bounds and never
  executes them.
- SCM adapter responses and owner-resolution inputs are untrusted until
  structurally validated and bound to one immutable snapshot.
- A Build engine result is untrusted typed data. Pipeline validates its schema,
  version, bounds, ordering, bindings, cardinality, digests, owner facts, and
  lossless mapping; Build alone proves semantic completeness.
- Ruleset-selected Pipeline/Build source and qualification profiles are
  protected, identity-bound, independently reviewed control inputs.
- The Git executable is today's required adapter capability. The adapter uses
  explicit immutable objects and isolates hooks, mutable checkout content,
  network, and candidate-selected configuration.

The current Git adapter is required for the live repository. Git identity or
command syntax does not cross the repository port. A future owned SCM provides
the same values and compare/publish guarantees; Build receives neither adapter.

</trust_boundary>

<repository_values>

## SCM-neutral immutable values

This section specifies the destination. The landed Git seam exposes raw
NUL-delimited name-status bytes and derives occupied/live path sets; it has no
versioned SCM-neutral, status-rich lossless-delta value.

The Pipeline repository boundary exposes versioned semantic values equivalent
to the following shapes. Names are descriptive; exact Rust placement waits for
the implementation plan.

```text
SnapshotIdentity = { format_version, scm_kind, opaque_identity }
SnapshotEntry = { path, kind: RegularBlob | ExecutableBlob | Tree,
                  storage_content_digest, content_bytes? }
PathDelta =
  Added(path) | Modified(path) | Deleted(path)
  | Renamed(old_path, new_path) | Copied(source_path, new_path)
  | TypeChanged(path)
OwnerAuthorityIdentity = { format_version, opaque_revision }
OwnerExpectation = { path, expected: Owner(owner) | Absent }
OwnerFacts = { authority_identity, expectations: sorted [OwnerExpectation] }
```

`opaque_identity` is equality/provenance, not a SHA-shaped string.
`storage_content_digest` is non-authoritative transport integrity, never a
postimage digest. Content is present exactly where the consumer declares it as a
read. Paths are repository-
relative byte-preserving values at the adapter boundary and become normalized
text only after the closed profile proves the encoding and path rules. Symlink,
gitlink, device, mutable checkout, truncated delta, duplicate/conflicting entry,
and lossy path representations refuse. `OwnerExpectation::Absent` means the
authority resolves no owner for that path; it is permitted only for a non-write
semantic read and is independent of file absence.

The adapter provides base/head identities and lossless delta, changed-endpoint
base bytes/modes, and complete HEAD entry facts/bytes selected by Build's
protected versioned source-surface request. It binds one owner fact for each
declaration/potential write under one authority identity/revision. Pipeline does
not accept caller-authored expected nodes, edges, semantic facts, or conformance
answers, and does not select correctness inputs from the delta/base: Build
derives the graph; deltas only trigger, attribute, and shard.
</repository_values>

<build_engine_exchange>

## Protected Build-engine exchange

Pipeline passes one bounded request containing:

- the exact protected engine and grammar-profile identities;
- opaque immutable base and head snapshot identities;
- immutable base bytes/modes/digests for changed declaration endpoints;
- complete HEAD bytes/modes/digests for the requested first-party Cargo and
  BUCK source surfaces, excluding `third-party//` and generated
  `third-party/BUCK`;
- the lossless base-to-head path delta; and
- caller-resolved owner facts bound to one authority identity/revision.

Build returns one versioned result containing sorted typed violations and exactly
one canonical `DeclarationRepairSetV1`, including zero-action sets. It carries
whole-set digest/identity and canonical non-empty owner groups in canonical
owner order; Pipeline
validates schema/version, bindings, bounds/order, owner facts, cardinality,
digests, and lossless mapping, never Cargo/BUCK semantics or completeness.

The protected layout application invokes this exchange when either source
surface changes. Build still evaluates the complete HEAD relation. No candidate
source controls executable code, parser/profile selection, limits, invocation,
or the allow/refuse decision. The engine has no process, network, SCM, owner
resolver, clock, filesystem-write, candidate-mutation, Cargo, or Buck2
capability.

</build_engine_exchange>

<changeset_contract>

## Canonical ChangeSet wrapping

Pipeline accepts exactly one opaque canonical `DeclarationRepairSetV1` per
evaluation. Build supplies whole-set digest/identity and non-empty owner groups
in canonical owner order; a zero-action set has zero groups. Pipeline validates
and maps each supplied group one-to-one to exactly one ChangeSet, never derives,
invents, or regroups ownership. Every action and proposed-write path is in
exactly one group and writes are pairwise-disjoint. Each semantic/proposed write
has exactly one concrete expected owner; `OwnerExpectation::Absent` is allowed
only for non-write semantic reads. Empty, extraneous, missing, duplicate,
ambiguous, wrong-owner, cross-owner, incomplete, overlapping, or absent-owner
groups/writes refuse before ChangeSet construction.

```text
ChangeSet = { schema_version, changeset_identity, source_snapshot_identity,
  repair_set_identity: WholeSetDigest, owner_group_identity,
  owner_group_output_digest, typed_postconditions: sorted [Postcondition],
  protected_engine_identity, grammar_profile_identity, owner_authority_identity,
  owner_preconditions: sorted [OwnerExpectation], causes: sorted [ViolationId],
  semantic_reads: sorted [ExpectedEntry], semantic_writes: sorted [Replacement] }
ExpectedEntry = { path, expected_preimage: Absent | Blob { preimage_digest, mode } }
Replacement = { path, expected_preimage: Absent | Blob { preimage_digest, mode },
  complete_postimage: Absent { postimage_digest } | Blob { complete_bytes, mode, postimage_digest } }
```

There is exactly one ChangeSet for each supplied owner group. `semantic_writes`
is the canonical proposed-write action sequence: it has exactly one `Replacement` for every proposed-write path and no others, so its sorted `Replacement.path` projection preserves the proposed-write path set losslessly. Its whole-set digest/identity and exact group identity, engine/profile, reads/writes, preconditions, typed
postconditions, deterministic complete postimages, every path-bound canonical postimage digest, exact
owner-group output digest, and owner facts map losslessly from V1.
`postimage_digest` is domain-separated over canonical `(path, Present/Absent tag, bytes/mode)` encoding; `Replacement` is its sole authority and no detached digest list exists. `owner_group_identity` routes/isolates but never supplies semantic authority.
Identity encoding is versioned, domain-separated, length-prefixed, ordered, and
excludes ambient host/time/process state. Patches, span edits, or commands only
explain; they never authorize application.
Before application Pipeline selects a current immutable snapshot, re-reads every
expected entry, owner authority, and bound path. It refuses unless all facts,
regular-file ownership, path-bound canonical postimage digests, and occupancy match. The adapter
constructs one successor tree/commit or none, publishes only an isolated PR, is
idempotent for the same ChangeSet/current snapshot, and reports an indeterminate
commit/PR outcome honestly. A source/current snapshot mismatch alone is not a
conflict; changed declared read/write/mode/owner/authority facts require fresh
Build evaluation. Pipeline never heuristically rebases a postimage.

</changeset_contract>

<qualification_state>

## Qualification and activation state

Pre-activation candidate states are nonblocking:

```text
NeverEnforced:
Unqualified -> Shadowed -> RepairQualified -> RepairClean
                                             -> AdmissionQualified
AdmissionQualified -- atomic first activation --> Enforced(active)
```

- `Unqualified`/`Shadowed` are non-admitting and non-applying evidence;
  `RepairQualified` may produce protected repair PRs but never blocks admission.
- `RepairClean` means current violations merged and complete HEAD is clean;
  `AdmissionQualified` also passes every safety, coverage, determinism, fault,
  resource, differential, and SLO criterion.

Monotonic protected `ever_enforced` prevents post-activation fallback:

```text
Enforced(active) + Shadow(candidate)
  candidate AdmissionQualified -- atomic swap --> Enforced(candidate)
  active invalid/unavailable --> EnforcementBlocked(last_enforced)

EnforcementBlocked(last_enforced) + Shadow(candidate)
  candidate AdmissionQualified -- atomic swap --> Enforced(candidate)
```

`Enforced(active)` remains authoritative while another identity shadows;
candidate failure resets only the candidate. Invalid/unavailable active state
enters `EnforcementBlocked` and refuses relevant changes until an atomic
qualified replacement. A shadow never supplies an admission verdict. State is
protected Pipeline control, not candidate configuration, census, or waiver.

Campaign/query/conformance production uses Pipeline's single versioned API,
declarative desired-state resources, and reconcilers; a CLI is a retirement-
marked diagnostic. Before that surface, ordinary protected PRs are the sole
route: manual migrations become versioned gold fixtures but do not claim an
automated campaign or authorize a one-off analyzer/runner/controller/evidence
plane. Campaign planning consumes declared repair-group dependency/fanout facts,
detects SCCs, makes topologically ordered closure-complete waves, starts with
one canary, and supports explicit halt/repair/rollback. Owner groups are not
dependency closure; scheduling consumes a separately adopted Compute contract.

The out-of-presubmit qualification harness may invoke protected
`cargo metadata --offline --locked --no-deps --format-version 1` and non-building
`buck2 uquery` against isolated immutable snapshots. It compares semantic facts
and refusal behavior, not compilation success; the required declaration path invokes neither tool.

</qualification_state>

<failure_contract>

## Stable refusal classes

- invalid, unavailable, mutable, or inconsistent repository snapshot;
- malformed, lossy, duplicate, or unsupported path delta/entry kind;
- missing, ambiguous, stale, or cross-shard owner fact, owner-authority identity
  mismatch, or per-path expected-owner/absence mismatch;
- protected engine unavailable, timed out, crashed, or returned malformed,
  oversized, unbound, unordered, or unqualified output;
- Build semantic refusal or violation;
- invalid repair-set schema/version/binding/digest, absent-owner semantic/
  proposed write, empty/extraneous/missing/duplicate/ambiguous/wrong-owner/
  cross-owner/incomplete/overlapping group, non-exact coverage, missing typed
  postcondition/path-bound postimage/owner-group-output digest, split whole-set
  digest/identity, missing/invalid complete postimage or digest mismatch,
  incomplete/mismatched proposed-write path or lossless map/read/write set,
  derived/regrouped group, or ChangeSet mismatch;
- semantic precondition, mode, owner-authority, per-path owner, or occupancy
  conflict;
- successor construction failure, publication failure, or indeterminate
  publication outcome; and
- qualification evidence incomplete, failed, or invalidated by identity change;
- post-activation active-profile loss/invalidity, `EnforcementBlocked`, or an
  attempted transition back to nonblocking shadow-only admission.

Errors preserve the stable category, correlation identity, protected profile,
snapshot/repair-set/ChangeSet digests where known, and safe human context. They never
include unrestricted source bytes, secrets, tokens, mutable host paths, or
candidate-controlled terminal escapes. Unknown errors become an internal
refusal, never success.

</failure_contract>

<bounds_and_observability>

## Work, evidence, and data handling

Versioned protected limits bound snapshot entries/bytes, delta records, path and
owner-fact bytes, engine result bytes, violations, owner groups, semantic
reads/writes, individual/aggregate postimage bytes, duration, and diagnostic
cardinality. Limit values live with protected implementation and tests, not in
a candidate census or current-repository count. Processing is deterministic and
linear in admitted declaration bytes plus modeled nodes/edges.

Receipts bind correlation ID; protected source, engine, grammar profile and
limit identities; base/head/current snapshot identities; owner-authority
revision; verdict/refusal class; repair-set/ChangeSet identities; compare outcome;
qualification state, monotonic `ever_enforced`, active/last-enforced and shadow
profile identities; and published branch/PR/commit identity when known.
Metrics report duration, bounded sizes, violations, owner groups, refusal class,
owner group, CAS conflict, qualification transition, and indeterminate
publication. Raw declaration bytes and postimages are excluded.

</bounds_and_observability>

<verification>

## Required test and fault matrix

- **Pure values:** canonical identity/property tests, ordering, maximum and
  maximum+one, arbitrary bytes/Unicode, and panic freedom.
- **Git adapter:** immutable-object reads, hook/config isolation,
  rename/copy/delete/type change, non-UTF-8/truncation, symlink/gitlink, absent
  object, and process failure.
- **Owner facts:** authority identity/revision, concrete expected owner for every
  semantic/proposed write, Absent-only non-write read, missing/ambiguous/stale
  fact, owner move, routing-shard misuse, and root/meta owner handling.
- **Protected admission:** candidate cannot select, replace, or skip the engine;
  complete HEAD follows either delta trigger; engine absence, crash, timeout, or
  malformed result refuses; exactly one verdict reaches fan-in.
- **ChangeSet:** one canonical V1 set, zero-action/zero-group, Build-supplied
  non-empty groups mapped one-to-one without derivation/regrouping, exact-once
  actions/paths, concrete write owners, Absent-only non-write reads,
  pairwise-disjoint writes, proposed-path/Replacement bijection, and every listed
  malformed-group refusal; also typed postconditions, deterministic complete
  postimages, every path-bound postimage/owner-group output digest, conflicts, and retry.
- **Application:** inject failure before and after every compare, tree
  construction, commit, push, and PR boundary; observe no partial success and
  an honest indeterminate outcome.
- **Qualification:** both declaration triggers, admitted/refused grammar,
  protected differential mismatch, legacy-drift clean replay, and profile
  identity invalidation in never-enforced and post-activation regimes.
- **End to end:** one owner canary traverses proposal, independent review,
  merge, clean replay, atomic activation/replacement, active-profile loss into
  `EnforcementBlocked`, and recovery before wider owner fan-out.

</verification>
