---
doc_class: Owner-SPEC
owner: build
status: Active
date: 2026-08-27
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
binds repository correlation plus manifest, lock, Reindeer config, fixup tree,
Cargo source snapshot, platform set, generator source/binary, Cargo/rustc,
effective environment, sandbox, validator, and validation-profile identities.
It produces `RawGeneration`, `ValidatedGeneration`, and stable
`GenerationIdentity` values. A separate `PublicationRequest` binds generation,
expected destination preimage, and publisher profile; its attempt receipt adds
the actual success, typed failure, or indeterminate outcome.

Repository revision is correlation, not a substitute for content digests. The
same generation request values and generator bytes produce the same output and
generation identity regardless of checkout path, temporary path, wall clock,
locale, user, or host environment. A publication-attempt receipt is separately
deterministic for its generation, destination preimage, publisher profile, and
actual outcome; distinct success, failure, and indeterminate outcomes do not
share an identity.

The core orders the transaction as: admit generation request; run A; run B;
compare bytes; validate the common bytes; construct the generation identity;
admit a qualified publication request; acquire exclusive destination authority;
compare the destination preimage; publish or report unchanged; construct the
publication-attempt receipt for every attempted outcome. No adapter can publish
a `RawGeneration`.

</reconciliation_model>

<generator_profile_v1>

## Provisional Reindeer qualification candidate

The sole implementation candidate is Reindeer `v2026.08.10.00` at
`bb681570d2bc47d1446080c12b8681a50a95f628`; the reviewed design ratifies it now,
not a qualified or publishable binary. The later alias-changing source is comparison evidence,
not a second candidate. Binary qualification binds reviewed
source/executable digests and remains blocked on eleven inherited plus four
planned scanner decisions, two clean byte-equal runs, and consumer evidence.

The process adapter invokes pinned `buckify` with locked/offline/stdout
semantics, absolute admitted inputs and read-only Cargo sources, an empty stage,
no network, and no writable Cargo home. It clears proxy, credential, wrapper,
compiler-substitution, target-dir, incremental, rustflags, config, and loader
variables before applying a closed allowlist. Source, Cargo/rustc, environment,
and sandbox identities are digested inputs.

Stdout alone carries BUCK bytes; stderr is bounded diagnostic data. Unsupported
sandbox profiles refuse. Qualified profiles reject undeclared reads/writes,
network, timeout/signal/nonzero exit, invalid required text, and oversized
output.

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
between Reindeer stdout and validation.

</fixup_contract>

<validation>

## Candidate validation

Validation is deterministic and bounded. It parses the full generated file and
checks:

- the canonical generated header and imports;
- unique rule names and one definition per expected crate/alias identity;
- every dependency, source archive, build-script output, and native-rule
  reference resolves;
- no destination, absolute checkout, staging, or host-specific path appears;
- required Cargo feature/cfg/platform branches and fixup effects are present;
- forbidden wrapper/overlay references and hand-edit sentinels are absent;
- representative targets exist for AWS-LC, PSM, optional aliases,
  `wasm-bindgen-futures`, and `web-sys`.

Parser and rule-shape limits bound file bytes, rules, attributes, list entries,
string bytes, and reference edges. Limit values land with the kernel tests and
cannot be inferred from a candidate file.

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
validator identities; and `output_digest` plus `output_length_bytes`. Generated
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

Unit/property tests cover digest ordering, environment construction, bounds,
double-run comparison, receipt stability, and panic freedom. Adapter tests use
a fake generator and fault-injecting filesystem at every transaction boundary.
Contract tests run the pinned Reindeer on fixture workspaces for all nine
ratified platform mappings. Qualification runs twice from clean staging and
builds/cqueries the representative Buck targets; check-only then reports clean.

</errors_and_evidence>

<first_party_source_relation>

## Closed unconfigured grammar and normalized relation

The first-party engine consumes immutable base/head files, changed-path
attribution, snapshot identities, and ownership facts supplied by its caller.
Only head bytes determine conformance. Base/head deltas identify triggers and
repair owners; they do not limit the evaluated first-party graph.

A versioned grammar profile binds exact Cargo and Starlark parser identities,
versions, admitted Cargo forms, BUCK preludes, loaded macros, rule contracts,
label/cell forms, and limits. Maintained syntax libraries sit behind parser
ports. Parsing returns complete normalized facts or typed refusal; partial facts
never enter the relation, and Build owns no hand-written parser/interpreter.

Normalized Cargo facts retain package/path identity, target kind, and
normal/build/dev/optional/target-specific/path dependency semantics.
Normalized BUCK facts retain target identity/kind, declaring path, and direct
first-party edges. Every admitted BUCK edge resolves to a unique declared identity
in the admitted unconfigured source IR and is permitted by Cargo. Coverage applies
only to profiled participating target/dependency pairs; valid binary/test subsets pass,
and a Cargo package with no profiled BUCK participation is not an automatic
violation. Unsupported, unmapped, malformed, duplicate, or ambiguous facts
refuse. Unknown loads, macros, expressions, mutation/reassignment, control flow,
comprehensions, selects/configuration, labels, cells, or Cargo forms refuse until
a profile admits and qualifies their exact source form. Inability to prove an unknown
construct cannot influence target identity or dependencies is itself refusal;
implementation never classifies an unrecognized construct harmless without that proof.

The engine emits canonically sorted typed violations and owner-sharded
`DeclarationRepairSet` values. Each repair set binds snapshot correlation,
grammar-profile identity, the complete semantic read and write sets with an
expected digest or expected absence for every path, and deterministic complete
postimages for every write. The semantic read set includes every fact that
influenced a postimage. Application refuses any mismatch; a disjoint commit is
irrelevant when all declared preconditions still match. Reordered identical
inputs yield byte-identical violations, shards, preconditions, and postimages.

The engine never mutates a snapshot, applies a repair, invokes SCM or Buck2,
resolves owners, evaluates configuration, accesses a network, or spawns a
process. `third-party//` and generated `third-party/BUCK` stay outside this
profile and inside the Reindeer transaction above.

## Design, dependency, and qualification gate

No implementation or root/package-graph mutation starts until an owner design
names the parser ports/adapters, exact packages and targets, public/internal
types, bounds, dependencies and provenance review, serialized root writer, and
red/green/property/fuzz/differential fixtures. Reindeer qualification precedes
new parser dependencies. Differential qualification runs outside required
presubmit against protected Cargo metadata and non-building Buck queries; the
engine itself invokes neither. Parser, grammar, prelude, macro, or rule-contract
identity changes create a new profile and requalify before activation.

</first_party_source_relation>
