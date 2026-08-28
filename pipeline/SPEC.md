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
- A Build engine result is untrusted typed data. Pipeline validates its version,
  identity, bounds, ordering, ownership, preconditions, and postimages.
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
                  content_digest, content_bytes? }
PathDelta =
  Added(path) | Modified(path) | Deleted(path)
  | Renamed(old_path, new_path) | Copied(source_path, new_path)
  | TypeChanged(path)
OwnerAuthorityIdentity = { format_version, opaque_revision }
OwnerExpectation = { path, expected: Owner(owner) | Absent }
OwnerFacts = { authority_identity, expectations: sorted [OwnerExpectation] }
```

`opaque_identity` is equality/provenance, not a SHA-shaped string. Content is
present exactly where the consumer declares it as a read. Paths are repository-
relative byte-preserving values at the adapter boundary and become normalized
text only after the closed profile proves the encoding and path rules. Symlink,
gitlink, device, mutable checkout, truncated delta, duplicate/conflicting entry,
and lossy path representations refuse. `OwnerExpectation::Absent` means the
authority resolves no owner for that path; it is independent of file absence.

The adapter provides base and head identities, their lossless delta, base bytes
and modes for changed declaration endpoints, complete HEAD declaration entries
selected by Build's versioned source-surface request, and exactly one owner fact
for each declaration and potential write, all under one protected owner-
authority identity/revision. Pipeline does not select correctness inputs from
the delta or base. It uses them to decide whether to invoke, attribute
violations, and shard repair output.

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

Build returns one versioned result containing sorted typed violations and zero
or more deterministic `DeclarationRepairSet` owner shards. Pipeline validates
the result version, engine/profile binding, input-snapshot binding, ordering,
bounds, one-owner sharding, semantic read/write closure, expected digest-or-
absence preconditions, and complete postimages. It does not reinterpret Cargo,
BUCK, targets, labels, dependency kinds, or the reason for a Build refusal.

The protected layout application invokes this exchange when either source
surface changes. Build still evaluates the complete HEAD relation. No candidate
source controls executable code, parser/profile selection, limits, invocation,
or the allow/refuse decision. The engine has no process, network, SCM, owner
resolver, clock, filesystem-write, candidate-mutation, Cargo, or Buck2
capability.

</build_engine_exchange>

<changeset_contract>

## Canonical ChangeSet wrapping

Pipeline translates a valid Build repair shard one-to-one into its canonical
ChangeSet application contract:

```text
ChangeSet = {
  schema_version, changeset_identity, source_snapshot_identity,
  protected_engine_identity, grammar_profile_identity,
  owner_shard,  # routing only
  owner_authority_identity,
  owner_preconditions: sorted [OwnerExpectation],
  causes: sorted violation identities,
  semantic_reads: sorted [ExpectedEntry], semantic_writes: sorted [Replacement],
}
ExpectedEntry = { path, expected: Absent | Blob { content_digest, executable } }
Replacement = {
  path, expected: Absent | Blob { content_digest, executable },
  postimage: Absent | Blob { complete_bytes, content_digest, executable },
}
```

Identity encoding is versioned, domain-separated, length-prefixed, and ordered.
It includes the owner-authority identity/revision and sorted per-path owner
preconditions for the union of every semantic read, semantic write, and
proposed-write path; `owner_shard` is routing metadata only. Identity excludes
wall clock, hostname, checkout path, temporary path, PID, username, and ambient
environment. `semantic_reads` includes every declaration fact that can change
the proposed repair, including entries not written. Every write also appears in
the read set or carries an equivalent expected precondition. A patch, span edit,
or command is explanatory only and never application authority; application
uses complete postimages.

Before application, Pipeline selects one current immutable snapshot, reads every
expected entry, reloads the owner authority, and re-resolves every bound path.
It refuses unless the owner-authority identity/revision, every semantic
expectation, and every owner-or-absence expectation match; every write remains
a regular-file operation within the resolved owner represented by the routing
shard; all postimage digests recompute; and no active path occupancy conflicts.
The adapter then constructs one successor tree/commit from the current snapshot
and publishes it only as an isolated PR branch. Publication is idempotent for
the same ChangeSet/current snapshot and never reports success for an
indeterminate commit or PR outcome.

`source_snapshot_identity != current_snapshot_identity` is not itself a
conflict. If all semantic expectations and owner facts still match, commits
whose changes are disjoint from the declared semantic sets preserve
applicability. A changed semantic read, destination, mode, owner, or authority
revision refuses and requires fresh Build evaluation; Pipeline never rebases a
postimage heuristically.

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

The out-of-presubmit qualification harness may invoke protected
`cargo metadata --offline --locked` and non-building Buck queries against
isolated immutable snapshots. It compares semantic facts and refusal behavior,
not compilation success. The required declaration path invokes neither tool.

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
- incomplete semantic read/write set, missing/invalid complete postimage, or
  ChangeSet identity mismatch;
- semantic precondition, mode, owner-authority, per-path owner, or occupancy
  conflict;
- successor construction failure, publication failure, or indeterminate
  publication outcome; and
- qualification evidence incomplete, failed, or invalidated by identity change;
- post-activation active-profile loss/invalidity, `EnforcementBlocked`, or an
  attempted transition back to nonblocking shadow-only admission.

Errors preserve the stable category, correlation identity, protected profile,
snapshot/ChangeSet digests where known, and safe human context. They never
include unrestricted source bytes, secrets, tokens, mutable host paths, or
candidate-controlled terminal escapes. Unknown errors become an internal
refusal, never success.

</failure_contract>

<bounds_and_observability>

## Work, evidence, and data handling

Versioned protected limits bound snapshot entries/bytes, delta records, path and
owner-fact bytes, engine result bytes, violations, repair shards, semantic
reads/writes, individual/aggregate postimage bytes, duration, and diagnostic
cardinality. Limit values live with protected implementation and tests, not in
a candidate census or current-repository count. Processing is deterministic and
linear in admitted declaration bytes plus modeled nodes/edges.

Receipts bind correlation ID; protected source, engine, grammar profile and
limit identities; base/head/current snapshot identities; owner-authority
revision; verdict/refusal class; ChangeSet identity; compare outcome;
qualification state, monotonic `ever_enforced`, active/last-enforced and shadow
profile identities; and published branch/PR/commit identity when known.
Metrics report duration, bounded sizes, violations, repair shards, refusal
class, owner shard, CAS conflict, qualification transition, and indeterminate
publication. Raw declaration bytes and postimages are excluded.

</bounds_and_observability>

<verification>

## Required test and fault matrix

- **Pure values:** canonical identity/property tests, ordering, maximum and
  maximum+one, arbitrary bytes/Unicode, and panic freedom.
- **Git adapter:** immutable-object reads, hook/config isolation,
  rename/copy/delete/type change, non-UTF-8/truncation, symlink/gitlink, absent
  object, and process failure.
- **Owner facts:** authority identity/revision, exact expected owner/absence for
  every semantic/proposed path, missing/ambiguous/stale fact, owner move,
  routing-shard misuse, and root/meta owner handling.
- **Protected admission:** candidate cannot select, replace, or skip the engine;
  complete HEAD follows either delta trigger; engine absence, crash, timeout, or
  malformed result refuses; exactly one verdict reaches fan-in.
- **ChangeSet:** complete semantic reads/writes/postimages, deterministic
  identity, applicable disjoint commit, refused intervening mismatch affecting
  either declared semantic set, internal read/write precondition coverage, and
  idempotent retry.
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
