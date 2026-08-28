---
doc_class: Owner-SPEC
owner: build
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - build/ADR.md
  - build/PRD.md
---

# Build technical specification

<landed_contract>

## Current declaration path

The root Cargo workspace/lock resolve packages. `reindeer.toml` targets that
workspace, emits `third-party/BUCK`, enables build scripts/Cargo variables, and
fails unresolved fixups. Package decisions live in
`third-party/fixups/<crate>/fixups.toml`; Buck2 loads the emitted rules.

This path is not reproducible: configured and checked headers name different
deleted wrappers; PSM describes a post-generation OS rewrite; and clean
reconnaissance found unresolved build-script packages. The provenance inventory
does not preserve the historical overlay by default.

</landed_contract>

<toolchain_dependency_evolution>

## Current facts and lifecycle model

Root declarations and hosted jobs name stable 1.98.0 (`rustc` `88d9e12ae`,
Cargo `797e8a9bc`); Buck, the image builder, and standards still narrate 1.97.1.
The observed nightly is `bff8e12ff`/Cargo `e8cb624d5`. Floating channel names
are discovery inputs, never receipt identities.

Typed intake retains exact MSRV/stable/beta/nightly tools, components, targets,
release items, owner dispositions, dependency source/version/checksum/role/
feature/audit/closure, and canonical advisory aliases and provenance. Discovery
may use network adapters; qualification consumes an immutable bound mirror,
refuses conflicting ranges, and takes Security decisions through a port.

One candidate transaction inventories every pin, preserves the declared MSRV,
regenerates Cargo/Reindeer/Buck declarations, computes affected closure, and
emits neutral ChangeSet, rollback, and qualification receipts. It evaluates
build scripts, features, duplicates, source ownership, publication age, yanks,
and audits; owned quarantine replaces unstable Cargo minimum-age behavior.

Stable qualification runs MSRV and production matrices separately. Exact beta
and nightly identities shadow compile/test/lint/format, representative Buck,
declarations, symbols, unsafe/FFI, WASM, macros/scripts, and platforms without
mutating production pins. The safe `arrayref` 0.3.9 selection and current
future-incompatibility findings are remediation inputs, not edit authority.
Every release item needs an owner disposition; “no code change” needs an
affected-graph reason. Rust 1.99/1.100 observations remain provisional.

</toolchain_dependency_evolution>

<reconciliation_model>

## Pure transaction

The semantic core accepts values, never ambient state. `ReconciliationRequest`
binds repository correlation; exact manifest/lock/config/fixup/source/platform
inputs; generator source/build/binary, Cargo/rustc and renderer identities; the
closed environment/sandbox; and graph/parser/grammar/Buck consumer profiles. It
produces generator graph, raw bytes, parsed projection, validated generation,
and stable identity values. A separate publication request binds generation,
destination preimage, and publisher profile; its attempt receipt adds outcome.

Repository revision is correlation, not a substitute for content digests. The
same generation request values and generator bytes produce the same output and
generation identity regardless of checkout path, temporary path, wall clock,
locale, user, or host environment. A publication-attempt receipt is separately
deterministic for its generation, destination preimage, publisher profile, and
actual outcome; distinct success, failure, and indeterminate outcomes do not
share an identity.

The core orders the transaction as: admit generation request; run A and B from
independent clean roots; compare bytes and exported producer graphs; validate
the semantic round trip; construct the generation identity;
admit a qualified publication request; acquire exclusive destination authority;
compare the destination preimage; publish or report unchanged; construct the
publication-attempt receipt for every attempted outcome. Only core may turn the
producer artifact plus independent projection into a publishable generation.

</reconciliation_model>

<generator_profile_v1>

## Provisional Reindeer qualification candidate

The sole candidate is Reindeer `v2026.08.10.00` at `bb681570d2bc47d1446080c12b8681a50a95f628`, not a qualified/publishable binary. A later alias-changing source is comparison evidence. Revalidation of eleven inherited plus four landed run-only scanner decisions, clean byte/graph equality, and consumer evidence still block promotion.

That source is binary-only and private `do_buckify` returns `BTreeSet<Rule>`, a qualification blocker. A reviewed producer-side patch/API against the exact source, bound by patch/fork/source/binary digests, returns one `ReindeerGeneratedArtifactV1 { graph: ReindeerRuleGraphV1, rendered_buck }` per invocation; bytes come from that same graph instance. The type lives with generator code; upstream acceptance is optional, while Build owns qualification/rollback. Build never introspects private `Rule`, makes a second invocation for another view, or reconstructs graph from text.

The producer refuses duplicate sort keys before `BTreeSet` loss. Equality/digest covers every canonical DTO field, never `Rule::PartialEq` (sort key only). The candidate renderer is locked `serde_starlark` 0.1.19 and its source/checksum join the profile. The bounded lossless graph covers Alias, Sources, Filegroup, ExtractArchive, HttpArchive, GitFetch, Binary, Library, BuildscriptBinary, BuildscriptGenrule, CxxLibrary, PrebuiltCxxLibrary, and RootPackage, retaining order, identity, callee, every attribute/value, aliases, maps/env, labels/edges, paths, platform/select branches, wrappers, and RootPackage position. Duplicate/colliding/unknown/lossy projection refuses.

The adapter invokes that API once per run, locked/offline, with absolute admitted inputs, read-only Cargo sources, empty stage/network and no writable Cargo home. It clears proxy/credential/wrapper/compiler-substitution/target-dir/incremental/rustflags/config/loader variables before a closed allowlist. An isolated transport carries one bounded canonical artifact, never bare BUCK; diagnostics are bounded and it is internal API scaffolding, not a user CLI. Exact inputs, source/build/binary, Cargo/rustc, renderer, environment/sandbox, graph/schema/grammar/platform/bounds are digested; undeclared effects, process failure, or oversized fields refuse.

</generator_profile_v1>

<candidate_platforms_v1>

## Candidate generation platform families

The reviewed execution design freezes Rust triples and Buck mappings for nine families:
Linux GNU and musl on `x86_64`/`aarch64`; macOS on `x86_64`/`aarch64`;
Windows GNU and MSVC on `x86_64`; and `wasm32-unknown-unknown`.

Once ratified, generation and validation evaluate every exact mapping
independently of the host. Adding, removing, or redefining a ratified entry
changes the platform-set digest and requires a reviewed compatibility
transition. This family table alone is not an implementable platform set.

</candidate_platforms_v1>

<fixup_contract>

## Fixup-first semantics

Each package with a build script has an explicit reviewed decision: execute it
under the bounded Reindeer model or replace its effects with declarative fixup
rules. Absence is an error. Obsolete fixups are removed rather than retained as
no-op folklore.

Native and generated semantics live in `third-party/fixups/**` or an upstream
Reindeer configuration primitive. V1 qualification includes:

- AWS-LC build-script environment, link metadata, native inputs, and platform
  behavior without inheriting compiler/linker variables from the caller;
- separate PSM Linux and Darwin native-rule behavior so symbol decoration is
  selected structurally rather than rewritten in emitted text;
- optional alias preservation for every checked first-party Buck consumer;
- generated sources/build-script output and proc-macro dependencies required by
  representative targets.

The validator rejects a generated header or rule that advertises another
post-generation command. There is no patch list, regex rewrite, or AST rewrite
between rendered bytes in `ReindeerGeneratedArtifactV1` and validation.

</fixup_contract>

<validation>

## Candidate validation

`ReindeerRuleGraphV1` is the primary semantic seam, `ParsedBuckProjectionV1` the independent cross-check, and Buck2 consumer/configured authority. Validation is the exact round trip: graph → pinned Reindeer renderer bytes → independently injected maintained-parser port → bounded lossless projection → full equality. Meta `starlark_syntax` is later pinned by reviewed source revision, exact crate version/checksum, dependency graph, dialect/API and source digest; this docs lane selects no dependency. Without evaluation it projects every admitted header/import/callee/attribute/value, order, rule/alias identity, internal/external label, path, platform/select branch, map/env, wrapper, fixup effect, and RootPackage fact exactly once. Unknown/extra/lossy/ambiguous nodes or mismatch map to `InvalidGeneratedGraph`; Oyatie adds no renderer. Two clean runs separately prove raw-byte identity.

`GenerationQualificationProfileV1` binds exact generator API/source/build/binary, Cargo/rustc, renderer, parser, graph schema, grammar/header/import, platform, environment/sandbox/bounds, and Buck2 source/binary/toolchain/cell/config/prelude identities. Any tuple change creates a new profile. Promotion cqueries/builds representative AWS-LC, PSM, optional-alias, proc-macro, generated-source, Windows, Linux GNU/musl, macOS, and WebAssembly consumers; parser equality never substitutes for Buck2 configured authority.

</validation>

<publication>

## Qualified filesystem publication adapter

The filesystem adapter accepts a profile qualified for its filesystem/platform,
holds an exclusive destination lease or genuine compare-and-swap authority,
and uses directory-relative no-follow operations. It verifies the preimage,
writes a collision-resistant same-directory regular temporary file, flushes
bytes, sets the final mode, syncs file contents and metadata, replaces
atomically, and syncs the parent before returning `Replaced`.

Matching digest/mode returns `Unchanged` without rename. Lease loss, CAS
conflict, or missing primitives refuse; read-then-rename alone is not conflict
safe. Before-replacement failure retains the old complete file; directory-sync
failure after replacement reports indeterminate durability, never success.

Startup removes only an abandoned owned-prefix/validated-random-suffix regular
file proved not to be the destination; never a symlink, directory, or foreign
file.

</publication>

<receipt>

## Generation identity and publication-attempt receipt

The versioned generation identity canonically binds repository correlation;
manifest/lock/Reindeer/fixup/source/platform/validation digests; generator tag,
source, asset/build, and binary provenance; Cargo/rustc, environment/sandbox,
renderer, exported graph, parser/projection, grammar, bounds, and Buck consumer
profile identities; and `output_digest` plus `output_length_bytes`. Generated
BUCK content remains outside the identity record.

Fields are canonically ordered and exclude wall clock, hostname, checkout path,
temporary path, username, PID, and unrestricted environment. A release asset
digest is present when a prebuilt binary is used; a reproducible-build receipt
replaces it when the binary is built from source.

A separate publication-attempt receipt contains `generation_id`, destination
preimage digest, publisher capability profile/version, and the actual
`Unchanged`, `Replaced`, typed `Failed`, or `Indeterminate` outcome. Failure and
indeterminate variants bind the failure class, known/maybe replacement state,
and known/unknown durability. Publication outcome does not alter the generation
identity, but it does alter the attempt-receipt identity.

</receipt>

<errors_and_evidence>

## Stable failure classes

Stable failures are `InvalidRequest`, `InputChanged`, `MissingFixup`,
`GeneratorUnavailable`, `GeneratorFailed`, `GeneratorTimedOut`,
`GeneratorOutputTooLarge`, `NondeterministicOutput`, `InvalidGeneratedGraph`,
`UnsupportedPublicationProfile`, `DestinationLeaseUnavailable`, `LeaseLost`,
`DestinationConflict`, `StageWriteFailed`, `StageSyncFailed`, `ReplaceFailed`,
`DirectorySyncFailed`, and `InternalInvariant`.

Byte comparison never trusts digests alone: same digest with different content
is `InternalInvariant`; unequal run bytes/producer graphs are
`NondeterministicOutput`. Parser/refusal details are bounded diagnostics under
`InvalidGeneratedGraph`, never unversioned wire variants.

Unit/property tests cover digest ordering, environment construction, bounds,
double-run comparison, receipt stability, and panic freedom. Adapter tests use
a fake generator and fault-injecting filesystem at every transaction boundary.
Contract tests run the pinned Reindeer on fixture workspaces for all nine
ratified platform mappings. Qualification runs twice from clean staging and
builds/cqueries the representative Buck targets; check-only then reports clean.

</errors_and_evidence>

<first_party_source_relation>

## Closed unconfigured grammar and normalized relation

The engine consumes caller-supplied immutable base/head files, changed-path attribution, snapshot identities, and ownership facts. Only head bytes determine conformance; deltas identify triggers/repair owners, never graph scope.

A versioned grammar profile binds exact Cargo/Starlark parser identities/versions, admitted Cargo forms, BUCK preludes/loads/macros/rules, labels/cells, and bounds. Maintained libraries sit behind ports and yield complete normalized facts or typed refusal; Build writes no parser/interpreter. Cargo facts retain package/path/target and normal/build/dev/optional/target-specific/path semantics. BUCK facts retain target/path/kind/direct edges. Each admitted edge resolves uniquely in source IR and is Cargo-permitted; only profiled target/dependency pairs require coverage, so valid binary/test subsets pass. Unknown, unmapped, malformed, duplicate, ambiguous, or unproved-influence forms refuse.

The engine emits sorted violations and exactly one canonical `DeclarationRepairSetV1` per evaluation, including zero actions/groups. It binds engine/snapshot/profile/caller owner-authority/ownership-fact provenance; complete semantic reads and `semantic_writes`; their exact proposed-path projection; and digest-or-absence plus owner-or-absence on every bound path. `OwnerExpectation::Absent` is valid only on non-write reads.

`semantic_writes` is sole action authority: exactly one concrete-owner `Replacement` per proposed path and no other action. Each `Replacement` alone carries its path's complete present/absent postimage and canonical postimage digest. The set binds typed postconditions, exact group-output digests, and one whole-set digest/identity over every other canonical field. Groups are exactly non-empty groups induced by replacement owners, canonically ordered; every replacement/path appears once, writes are disjoint, and zero actions yield zero groups.

Absent-owner writes; empty/extraneous/missing/duplicate/ambiguous/wrong-owner/cross-owner/incomplete/overlapping groups; or any semantic/owner precondition, postimage, digest, or projection mismatch refuses. Snapshot identity is provenance, not a global lock; a disjoint successor applies only if every bound semantic/owner precondition matches. Equal reordered inputs yield byte-identical violations, V1 bytes, groups, outputs, and identity.

The engine never mutates/applies, invokes SCM/Buck2, resolves owners, evaluates configuration, uses network, or spawns a process. `third-party//` and generated `third-party/BUCK` stay in Reindeer.

## Design, dependency, and qualification gate

Before behavior/root mutation, amend the current owner design to freeze the producer artifact API, distinct parser port in the six-package topology, exact types/bounds/dependency review, and RED-first property/fuzz/differential fixtures. Prove the Reindeer prerequisite without dependency changes; only then may a serialized lane exact-pin `starlark_syntax`, followed by full graph/parser/Buck2 qualification. Only the protected out-of-presubmit harness invokes `cargo metadata --offline --locked --no-deps --format-version 1` and non-building `buck2 uquery`; engine/required check never do. Any bound identity change requalifies. No implementation, dependency, generated bytes, qualification, or readiness is claimed.

</first_party_source_relation>
