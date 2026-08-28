---
doc_class: Owner-SPEC
owner: pipeline
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md#ADR-0719-D17a
  - pipeline/ADR.md
  - pipeline/PRD.md
---

# Pipeline declaration-integrity boundary

<status>

## Contract status

This specifies the future Pipeline boundary required by D17a. It does not
rename the current draft Git read port, declare its values compatible, or claim
an implementation, qualification profile, ChangeSet publisher, or enforcement
state exists.

</status>

<repository_facts>

## SCM-neutral facts

The future repository capability supplies versioned opaque values for:

- immutable base and HEAD snapshot identities;
- repository-relative entry facts: path, supported regular-file mode, complete
  bytes when requested, and content digest;
- a bounded lossless base-to-HEAD path delta, including add, modify, delete,
  rename, copy, and entry-kind change; and
- one owner-authority identity/revision plus sorted expected owner-or-absence
  facts for every semantic read, semantic write, and proposed-write path.

The identity is equality/provenance, not a Git-SHA-shaped field. Git translates
to these values in its adapter; core and Build receive neither Git command
syntax nor checkout paths. Symlinks, gitlinks, nonregular input, duplicate or
lossy paths, incomplete data, and conflicting or ambiguous owner facts refuse.

Build requests the relevant complete HEAD surface. Pipeline supplies it with
the delta and base facts, but does not select correctness facts from the delta
or infer declaration meaning from any entry.

</repository_facts>

<build_exchange>

## Protected Build exchange

A future protected invocation binds:

- versioned protected Build-contract and profile identities;
- immutable base/HEAD snapshot identities and requested complete HEAD facts;
- base facts for changed requested entries and the lossless delta; and
- caller-resolved owner facts under their protected authority revision.

Pipeline accepts a versioned Build result only when its identity, input binding,
bounds, canonical ordering, owner grouping, and complete repair envelope are
valid. It transports sorted typed violations and `DeclarationRepairSetV1`
values without interpreting Cargo, Starlark, target, label, dependency, parser,
or configured-graph semantics.

The invocation is selected by protected source in the existing layout-admission
seam. Candidate content can supply data but cannot choose executable code,
profile, limits, invocation, or success. The required invocation has no native
Cargo/Buck query or candidate executable capability.

</build_exchange>

<changeset>

## Canonical ChangeSet

Every source `DeclarationRepairSetV1` has one source identity, whole-set digest,
and exact owner groups. Pipeline maps it losslessly to exactly one versioned
ChangeSet per owner group. Each ChangeSet preserves that RepairSet identity,
whole-set digest, and owner-group identity. The groups jointly contain every
repair exactly once and have pairwise-disjoint writes; missing, duplicate,
ambiguous, or overlapping groups refuse.

Each canonical identity is domain-separated, length-prefixed, and ordered; it
includes source snapshot, Build/profile provenance, sorted violation causes,
complete semantic reads and writes, expected digest-or-absence preconditions,
complete deterministic postimages, postconditions, output digests, owner
authority identity/revision, and sorted owner-or-absence preconditions. It
excludes clock, host, checkout, PID, username, and ambient environment.

Every write has an expected precondition; every fact that can affect the repair
is a semantic read. Text hunks, commands, and explanatory patches have no
application authority. An owner shard is routing metadata only.

Before a future apply, Pipeline selects one current immutable snapshot and
re-reads every declared semantic and owner fact. It refuses unless every
precondition, owner fact, mode, and postimage digest matches. It then produces
one complete successor or no successor and publishes only an isolated protected
PR. A source/current snapshot identity difference is not itself conflict; a
disjoint successor is applicable only while all declared facts still match.

</changeset>

<qualification>

## Qualification boundary

Pipeline records the protected qualification identity and outcome necessary to
orchestrate repair and activation. It neither defines the grammar nor executes
the native differential tools: a separate protected out-of-presubmit harness
produces that evidence. Any parser, grammar, prelude, macro, rule-contract,
Build-contract, result-schema, or profile identity change invalidates the
corresponding qualification.

Before first activation, an invalid or incomplete qualification is nonblocking
evidence only. Later activation requires a clean complete-head replay after
all detected drift is repaired; it has no stored baseline or exception list.

</qualification>

<refusal>

## Stable refusal boundary

Pipeline refuses invalid/mutable snapshots; malformed/lossy paths or deltas;
missing, stale, conflicting, or ambiguous owner facts; unbound, unordered,
oversized, malformed, or unavailable Build output; incomplete repairs;
precondition or postimage mismatch; and indeterminate publication. Unknown
failure is never converted to success.

</refusal>
