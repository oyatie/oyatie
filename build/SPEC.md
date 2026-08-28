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
deleted wrappers; PSM describes a post-generation OS rewrite; and clean L1a
reconnaissance found unresolved build-script packages. L1b inventories these;
it may not preserve the historical overlay by default.

</landed_contract>

<toolchain_dependency_evolution>

## Current facts and lifecycle model

Root declarations and hosted jobs name stable 1.98.0: rustc `88d9e12ae` and
Cargo `797e8a9bc`. Buck, the distroless builder, and several standards still
narrate 1.97.1. The observed nightly moved from rustc `c656540d6` to
`bff8e12ff`, paired with Cargo `e8cb624d5`; floating channel names are not
receipt identities.

The lock selects safe `arrayref` 0.3.9 rather than malicious, deleted 0.3.10. An
exact `bff8e12ff` all-target offline check passed with the default next trait
solver and Polonius Alpha, but reported future-incompatible
`attribute-derive-macro` 0.10.5 and `proc-macro-error2` 2.0.1 through Leptos/
`application-shell-frontend`, plus `redis` 0.27.6 through the Valkey adapter.
These are preview remediation inputs, not authority to mutate stable pins.

Typed records cover exact MSRV/stable/beta/nightly toolchain and component/
target identities; provenance-bound release items, nonbinding Build
recommendations, and consuming-owner-supplied or accepted
`ADOPT|BENCHMARK|DEFER|REJECT` dispositions; dependency source/version/checksum,
publish/yank/maintainer/role/feature/audit/affected-closure facts; and canonical
vulnerability ranges, aliases, withdrawals, provenance, and Security decision
references.

Discovery may use networked adapters, but qualification consumes an immutable,
source-provenance-bound mirror. It deduplicates advisory aliases, preserves
withdrawals and modifications, and refuses conflicting affected/fixed ranges.
Security-owned severity, exploitability, embargo, disclosure, VEX, and CNA
decisions enter through a port; Build supplies graph impact and transformation
mechanics and does not become a CNA.

One candidate transaction inventories all pin surfaces, resolves with the
declared MSRV policy, regenerates Cargo/Reindeer/Buck declarations, computes
configured-target and semantic fanout, and emits an atomic ChangeSet plus
rollback and qualification receipts. Dependency admission also evaluates
normal/build/dev/proc-macro/native role, new build scripts, feature drift,
duplicates, audits, source ownership, publish age, and yanks. An owned
quarantine rule replaces reliance on unstable Cargo minimum-age behavior.

Production-stable qualification runs MSRV and stable matrices separately. Beta
and an exact dated nightly run shadow differential compilation, tests, lints,
formatting, representative Buck targets, generated declarations, binary/
symbolization checks, unsafe/FFI, WASM, build scripts/proc macros, and platform
targets. Shadow lanes never mutate production pins or `Cargo.lock`.

## Rust 1.96 through 1.100 applicability ledger seed

This table seeds, but does not replace, item-by-item release-note disposition:

| Train | High-impact upstream change | Oyatie-specific disposition work |
|---|---|---|
| 1.96/1.96.1 stable | Copyable `core::range` spans; `assert_matches!`; WASM undefined symbols become link errors; Cargo tarball/auth CVEs plus later libssh2 CVEs and a MIR fix | Adopt spans for future semantic source ranges while public APIs accept `RangeBounds`; use richer match assertions in new fault tests; preserve strict WASM linking and explicit imports; treat point releases as emergency toolchain candidates, not MSRV events. |
| 1.97/1.97.1 stable | v0 symbol mangling; Cargo `build.warnings` and `resolver.lockfile-path`; visible linker diagnostics; integer bit-width/isolation APIs; new NVPTX baseline; LLVM miscompilation fix | Requalify symbolizers, profiles, backtraces, binary provenance, native linkers, and any GPU target; use alternate lockfile paths for read-only snapshot analysis; evaluate cache-neutral warning denial while retaining Clippy policy; property-prove any bit-helper simplification; rebuild affected artifacts after a compiler-fix rollout. |
| 1.98 stable | source-subrange and prefix/suffix APIs; buffered integer formatting; algebraic floating point; mutable-slice atomic views; endian-specific UTF-16 decoding; `CommandArgs` thread traits; `ManuallyDrop<Box<_>>` guarantee; runtime-symbol/FFI lints and stricter layout checks | Use subrange APIs in semantic/codemod provenance and evaluate `strip_circumfix` for exact parser chains; replace formatting crates only when direct usage and benchmarks justify it (`itoa` is currently transitive); flag algebraic floats as unsuitable for exact/deterministic surfaces and require the consuming owner to accept error budgets elsewhere; adopt remaining APIs only at evidenced call sites; prioritize unsafe/FFI/layout and rustfmt-delta qualification. |
| 1.99 projected | Cargo `debug` profile; CI incremental compilation off by default; edition-2024 workspace dependency `default-features` override; lint-name and resolver changes | Keep provisional until release; Oyatie already sets `CARGO_INCREMENTAL=0`, so prove no drift; model `dev`/`debug` explicitly; diff Cargo/Reindeer/Buck feature closure before using member overrides. |
| 1.100 nightly observed | next trait solver and Polonius alpha previews; Cargo build analysis, new cache layout, fine-grained locking, SBOM, section timing, profile-hint, trim-path, and minimum-publish-age experiments; removal of `update-breaking` | Run pinned-nightly compiler/performance/diagnostic differentials and file upstream regressions; treat the three observed future-incompatible transitive packages as owned dependency candidates; consume Cargo JSON messages rather than scrape `target/`; benchmark cache concurrency/rebuild causality and prototype SBOM/path hygiene behind non-authoritative adapters; keep breaking upgrades and publication quarantine in owned policy. |

The base has `assert!(matches!(...))` in 214 Rust files, repeated
`strip_prefix(...).and_then(...strip_suffix(...))` parser shapes, and one
security-sensitive Merkle split using `leading_zeros`. Those are candidate
corpora for 1.96 match diagnostics, 1.98 `strip_circumfix`, and 1.97 bit APIs,
respectively—not bulk-edit authorization. No direct Rust/Cargo use was found
for `itoa`, UTF-16 conversion, mutable atomic-slice APIs, `ManuallyDrop`, or
`CommandArgs`; those items receive an evidence-backed defer/revisit trigger
rather than a ceremonial rewrite.

Every other Rust, Cargo, rustfmt, and Clippy release item requires a recorded
consuming-owner disposition before the train can be called fully evaluated.
“No code change” is valid only with an affected-graph query and reason.

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

The initial comparison candidate is Reindeer `v2026.08.10.00` at
`bb681570d2bc47d1446080c12b8681a50a95f628`; a later candidate changes alias
emission while three checked `any_spawner` consumers exist. L1b must reproduce
both, and L1c ratifies one only after fixup and consumer evidence. Qualification
binds reviewed source and executable digests; a tag is insufficient.

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

L1c freezes exact Rust triples and Buck mappings for nine candidate families:
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

<placement_boundary>

## Implementation placement gate

Build behavior stays separate from `build/port-engine`. ADR-0719's meta-root
grammar makes exact placement structural; prefer
`build/dependency-declarations/{core,ports,adapters,facade}`, but authorize no
code/root mutation until an independent plan names paths, targets, packages,
the shared-file writer, and required reviews.

Build's reusable provider role is adopted; compilation-unit, semantic-fact,
conformance, and recipe schemas/homes remain nonbinding `ADR.md` details.

</placement_boundary>
