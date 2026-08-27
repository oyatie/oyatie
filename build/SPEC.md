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

The root Cargo workspace and `Cargo.lock` resolve Rust packages. Root
`reindeer.toml` points Reindeer at `Cargo.toml`, emits `third-party/BUCK`,
enables build-script execution, injects Cargo package variables, and fails on an
unresolved build-script fixup. `third-party/fixups/<crate>/fixups.toml` provides
package-local generation decisions. Buck2 loads the resulting rules through
the repository prelude.

This path is not reproducible as checked. The configured generated header names
the deleted `ci/facade/dependency-automation` overlay, while the checked output
names deleted `scripts/ci/regen-third-party.sh`. The PSM fixup explicitly
describes a post-generation per-OS rewrite. Clean-generation reconnaissance at
the L1a base also found unresolved local build-script packages. These are
inputs to L1b inventory, not permission to preserve the historical overlay.

</landed_contract>

<reconciliation_model>

## Pure transaction

The semantic core accepts values, never ambient repository state:

```text
ReconciliationRequest {
  repository_revision_correlation
  manifest_digest
  lock_digest
  reindeer_config_digest
  fixup_tree_digest
  platform_set_digest
  generator_source_identity
  generator_binary_digest
  cargo_identity
  rustc_identity
  validation_profile_version
}

RawGeneration { bytes, bounded_diagnostics }
ValidatedGeneration { bytes, output_digest, validation_summary }
PublicationOutcome = Unchanged | Replaced
```

Repository revision is correlation, not a substitute for content digests. The
same request values and generator bytes produce the same output and receipt
identity regardless of checkout path, temporary path, wall clock, locale, user,
or host environment.

The core orders the transaction as: admit request; run A; run B; compare bytes;
validate the common bytes; compare the destination preimage; publish or report
unchanged; construct the final receipt. No adapter can publish a
`RawGeneration`.

</reconciliation_model>

<generator_profile_v1>

## Initial Reindeer qualification

The initial candidate is Reindeer tag `v2026.08.10.00`, source commit
`bb681570d2bc47d1446080c12b8681a50a95f628`. The pin is chosen because the
later 2026-08-24 candidate no longer emits optional aliases currently
consumed as `any_spawner` and `http`, while the initial candidate retains those
aliases and the required fixup features. Qualification records both a reviewed
source digest and the exact executable digest; a tag alone is insufficient.

The process adapter invokes the pinned `buckify` operation with equivalent
`--locked`, `--offline`, and `--stdout` behavior. It supplies absolute paths to
the admitted executable and inputs, runs in a newly created empty staging
directory, disables network access, and does not inherit a writable Cargo home.
It clears proxy, credential, wrapper, compiler-substitution, target-dir,
incremental, rustflags, config, and dynamic-loader variables before applying a
closed allowlist required by Reindeer and the pinned toolchain.

Stdout alone carries candidate BUCK bytes. Stderr is bounded diagnostic data.
The adapter rejects destination writes, undeclared file reads when sandbox
support is available, timeout, signal, nonzero exit, invalid UTF-8 where the
BUCK parser requires text, and output above the declared byte ceiling.

</generator_profile_v1>

<platforms_v1>

## Closed generation platform set

The v1 platform-set digest covers exactly these nine Rust target families:

| Architecture | Operating system / environment |
|---|---|
| `x86_64` | Linux GNU |
| `aarch64` | Linux GNU |
| `x86_64` | Linux musl |
| `aarch64` | Linux musl |
| `x86_64` | macOS |
| `aarch64` | macOS |
| `x86_64` | Windows GNU |
| `x86_64` | Windows MSVC |
| `wasm32` | unknown / unknown |

Generation and validation evaluate all nine independently of the host. Adding,
removing, or redefining an entry changes the platform-set digest and requires a
reviewed compatibility transition.

</platforms_v1>

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

## Atomic filesystem adapter

The filesystem adapter opens the destination directory without following an
untrusted replacement path, verifies the expected destination preimage, and
creates a collision-resistant temporary regular file in that same directory.
It writes the validated bytes, flushes file contents/metadata, sets the declared
mode, and atomically replaces the destination. It then flushes the parent
directory before returning `Replaced`.

If the destination already has the validated digest and declared mode, the
adapter returns `Unchanged` without a rename. If the destination changes after
preimage capture, publication returns a typed conflict. A failure before or
during rename never reports success; a directory-sync failure reports an
indeterminate durability error and never fabricates a receipt claiming durable
publication.

Temporary files are recognized only by an owned prefix plus validated random
suffix. Startup may remove an abandoned matching regular file after proving it
is not the destination; it never follows or removes a symlink, directory, or
foreign file.

</publication>

<receipt>

## Deterministic provenance receipt

The receipt uses a versioned semantic schema and contains:

```text
schema_version
repository_revision_correlation
manifest_digest, lock_digest, reindeer_config_digest, fixup_tree_digest
platform_set_digest, validation_profile_version
generator_tag, generator_source_commit, generator_source_digest
generator_asset_digest?, generator_binary_digest
cargo_identity, rustc_identity
output_digest, output_bytes
publication_outcome
```

Fields are canonically ordered and exclude wall clock, hostname, checkout path,
temporary path, username, PID, and unrestricted environment. A release asset
digest is present when a prebuilt binary is used; a reproducible-build receipt
replaces it when the binary is built from source. The publication outcome does
not alter the semantic generation identity.

</receipt>

<errors_and_evidence>

## Stable failure classes

Stable failures are `InvalidRequest`, `InputChanged`, `MissingFixup`,
`GeneratorUnavailable`, `GeneratorFailed`, `GeneratorTimedOut`,
`GeneratorOutputTooLarge`, `NondeterministicOutput`, `InvalidGeneratedGraph`,
`DestinationConflict`, `StageWriteFailed`, `StageSyncFailed`, `ReplaceFailed`,
`DirectorySyncFailed`, and `InternalInvariant`.

Unit/property tests cover digest ordering, environment construction, bounds,
double-run comparison, receipt stability, and panic freedom. Adapter tests use
a fake generator and fault-injecting filesystem at every transaction boundary.
Contract tests run the pinned Reindeer on fixture workspaces for all nine
platforms. Qualification runs twice from clean staging and builds/cqueries the
representative Buck targets; check-only then reports clean.

</errors_and_evidence>

<placement_boundary>

## Implementation placement gate

The behavior belongs to Build and stays separate from `build/port-engine`.
ADR-0719's current meta-root grammar and shared workspace/build declarations
make the exact package path a structural decision. The preferred shape is a
capability-style `build/dependency-declarations/{core,ports,adapters,facade}`
tree, but no code path or root/workspace mutation is authorized until the
independent implementation plan names the admitted paths, Buck targets, package
names, shared-file writer, and required cross-owner reviews.

Compilation units, semantic facts, conformance queries, and transformation
recipes remain the nonbinding proposal recorded in `ADR.md`; this specification
does not define their schemas or implementation homes.

</placement_boundary>
