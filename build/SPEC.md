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

<toolchain_dependency_evolution>

## Current toolchain and update facts

At the L1a base, `rust-toolchain.toml`, workspace `rust-version`, and hosted
Rust jobs name 1.98.0. The installed host reports rustc 1.98.0 commit
`88d9e12ae` and Cargo 1.98.0 commit `797e8a9bc` on
`aarch64-apple-darwin`; Buck and the distroless builder still name 1.97.1.
`docs/standards/{code-style-rust,dependency-policy,lts-versions-verified,
observability-slo}.md` also contain stale or coupling claims that must be
reconciled by their owner; none overrides the live root declarations.
The floating nightly moved from rustc 1.100.0-nightly commit `c656540d6`
(2026-08-21) to `bff8e12ff` (2026-08-26), paired locally with Cargo commit
`e8cb624d5`. These are observed identities, not a production pin.

The lockfile selects `arrayref` 0.3.9, not the malicious/yanked 0.3.10 release
reported by the Rust Security Response Team on 2026-08-20. This near miss shows
why version-range CVE scans alone are incomplete: registry deletion/yank,
malicious-package, maintainer, build-script, and publication-age facts also
belong in candidate admission.

An exact `bff8e12ff` nightly `cargo check --workspace --all-targets --locked
--offline` passed on 2026-08-27 and produced future-incompatibility warnings for
`attribute-derive-macro` 0.10.5, `proc-macro-error2` 2.0.1, and `redis` 0.27.6.
That nightly enables the next trait solver and Polonius Alpha by default, so
the check exercised both preview compiler paths without an opt-in flag.
The current inverse graph reaches the first two through Leptos into
`application-shell-frontend`, and the third through
`intelligence-eventsink-valkey-adapter` into `intelligence-app`. These are
preview remediation inputs, not stable failures or authority to mutate pins.

## Typed intake and disposition model

```text
ToolchainIdentity {
  role = DeclaredMsrv | ProductionStable | BetaCandidate | NightlyObservation
  release, channel_date?, host, rustc_commit, rustc_binary_digest
  cargo_commit, cargo_binary_digest, rustfmt_digest, clippy_digest
  llvm_version, component_set_digest, target_set_digest, dist_manifest_digest
}
ReleaseItem {
  source_digest, upstream_item_id, release_status, compatibility_class
}
ApplicabilityDisposition {
  release_item_id, ADOPT | BENCHMARK | DEFER | REJECT
  owner, rationale, affected_units, msrv_effect, evidence, revisit_trigger
}
DependencyCandidate {
  package, source, version, checksum, publish_time, yank_state
  maintainer_provenance, dependency_role, feature_graph_digest
  license_audit_refs, affected_unit_closure
}
VulnerabilityFact {
  canonical_id, aliases, source, modified, withdrawn
  affected_ranges, fixed_ranges, provenance_digest, security_decision_ref?
}
```

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
| 1.98 stable | source-subrange and prefix/suffix APIs; buffered integer formatting; algebraic floating point; mutable-slice atomic views; endian-specific UTF-16 decoding; `CommandArgs` thread traits; `ManuallyDrop<Box<_>>` guarantee; runtime-symbol/FFI lints and stricter layout checks | Use subrange APIs in semantic/codemod provenance and evaluate `strip_circumfix` for exact parser chains; replace formatting crates only when direct usage and benchmarks justify it (`itoa` is currently transitive); allow algebraic floats only in approximate, error-budgeted kernels and forbid them in billing/accounting, hashes, receipts, tests, and deterministic transforms; adopt the remaining APIs only at evidenced call sites; prioritize unsafe/FFI/layout and rustfmt-delta qualification. |
| 1.99 projected | Cargo `debug` profile; CI incremental compilation off by default; edition-2024 workspace dependency `default-features` override; lint-name and resolver changes | Keep provisional until release; Oyatie already sets `CARGO_INCREMENTAL=0`, so prove no drift; model `dev`/`debug` explicitly; diff Cargo/Reindeer/Buck feature closure before using member overrides. |
| 1.100 nightly observed | next trait solver and Polonius alpha previews; Cargo build analysis, new cache layout, fine-grained locking, SBOM, section timing, profile-hint, and trim-path experiments; removal of `update-breaking` and minimum-publish-age experiments | Run pinned-nightly compiler/performance/diagnostic differentials and file upstream regressions; treat the three observed future-incompatible transitive packages as owned dependency candidates; consume Cargo JSON messages rather than scrape `target/`; benchmark cache concurrency/rebuild causality and prototype SBOM/path hygiene behind non-authoritative adapters; keep breaking upgrades and publication quarantine in owned policy. |

The base has `assert!(matches!(...))` in 214 Rust files, repeated
`strip_prefix(...).and_then(...strip_suffix(...))` parser shapes, and one
security-sensitive Merkle split using `leading_zeros`. Those are candidate
corpora for 1.96 match diagnostics, 1.98 `strip_circumfix`, and 1.97 bit APIs,
respectively—not bulk-edit authorization. No direct Rust/Cargo use was found
for `itoa`, UTF-16 conversion, mutable atomic-slice APIs, `ManuallyDrop`, or
`CommandArgs`; those items receive an evidence-backed defer/revisit trigger
rather than a ceremonial rewrite.

Every other Rust, Cargo, rustfmt, and Clippy release item receives a recorded
disposition before the train can be called fully evaluated. “No code change” is
valid only with an affected-graph query and reason.

</toolchain_dependency_evolution>

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
  cargo_source_snapshot_digest
  platform_set_digest
  generator_source_identity
  generator_binary_digest
  cargo_binary_digest
  rustc_binary_digest
  effective_environment_digest
  sandbox_profile_version
  validator_binary_digest
  validation_profile_version
}

RawGeneration { bytes, bounded_diagnostics }
ValidatedGeneration { bytes, output_digest, validation_summary }
GenerationIdentity { generation_id, admitted_input_digests, output_digest }
PublicationRequest {
  generation_id
  expected_destination_preimage_digest
  publisher_profile_version
}
PublicationOutcome = Unchanged | Replaced
PublicationAttemptReceipt {
  generation_id
  destination_preimage_digest
  publisher_profile_version
  publication_outcome
}
```

Repository revision is correlation, not a substitute for content digests. The
same generation request values and generator bytes produce the same output and
generation identity regardless of checkout path, temporary path, wall clock,
locale, user, or host environment. A publication-attempt receipt is separately
deterministic for its generation, destination preimage, publisher profile, and
actual outcome; `Replaced` and `Unchanged` attempts do not share an identity.

The core orders the transaction as: admit generation request; run A; run B;
compare bytes; validate the common bytes; construct the generation identity;
admit a qualified publication request; acquire exclusive destination authority;
compare the destination preimage; publish or report unchanged; construct the
publication-attempt receipt. No adapter can publish a `RawGeneration`.

</reconciliation_model>

<generator_profile_v1>

## Provisional Reindeer qualification candidate

The initial candidate is Reindeer tag `v2026.08.10.00`, source commit
`bb681570d2bc47d1446080c12b8681a50a95f628`. Upstream comparison suggests that
the later 2026-08-24 candidate changes public-alias emission, while this
repository has three checked `third-party//:any_spawner` consumer references.
That is a hypothesis for L1b reproduction, not proof that either candidate
produces the required graph. L1c ratifies the pin only after clean comparison,
fixup compatibility, and representative consumer evidence. Qualification
records a reviewed source digest and exact executable digest; a tag alone is
insufficient.

The process adapter invokes the pinned `buckify` operation with equivalent
`--locked`, `--offline`, and `--stdout` behavior. It supplies absolute paths to
the admitted executable, inputs, and read-only Cargo registry/vendor source
snapshot; runs in a newly created empty staging directory; disables network
access; and does not inherit a writable Cargo home. It clears proxy, credential,
wrapper, compiler-substitution, target-dir, incremental, rustflags, config, and
dynamic-loader variables before applying a closed allowlist required by
Reindeer and the pinned toolchain. The source snapshot, Cargo/rustc binaries,
effective environment, and sandbox profile are all digested inputs.

Stdout alone carries candidate BUCK bytes. Stderr is bounded diagnostic data.
The adapter refuses a sandbox profile that cannot enforce its declared read and
network boundary. Under a qualified profile it rejects destination writes,
undeclared file reads, timeout, signal, nonzero exit, invalid UTF-8 where the
BUCK parser requires text, and output above the declared byte ceiling.

</generator_profile_v1>

<candidate_platforms_v1>

## Candidate generation platform families

L1c must freeze exact Rust triples and Buck configuration mappings. The current
candidate covers these nine target families:

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

The filesystem adapter accepts only a declared capability profile qualified for
the target filesystem and platform. It obtains an exclusive destination lease
or genuine compare-and-swap authority, opens the destination directory, and
uses directory-relative no-follow operations. Under that authority it verifies
the expected destination preimage and creates a collision-resistant temporary
regular file in the same directory. It writes validated bytes, flushes file
contents/metadata, sets the declared mode, atomically replaces the destination,
and flushes the parent directory before returning `Replaced`.

If the destination already has the validated digest and declared mode, the
adapter returns `Unchanged` without a rename. A lease loss or compare-and-swap
conflict returns a typed refusal; a read followed by rename alone is never
represented as conflict-safe publication. A profile lacking the required
primitives returns `UnsupportedPublicationProfile` before staging. On qualified
profiles, a failure before replacement retains the prior file and observers see
only old or new complete bytes. A directory-sync failure after replacement
reports indeterminate durability and never fabricates a durable-success receipt.

Temporary files are recognized only by an owned prefix plus validated random
suffix. Startup may remove an abandoned matching regular file after proving it
is not the destination; it never follows or removes a symlink, directory, or
foreign file.

</publication>

<receipt>

## Generation identity and publication-attempt receipt

The stable generation identity uses a versioned semantic schema and contains:

```text
schema_version
repository_revision_correlation
manifest_digest, lock_digest, reindeer_config_digest, fixup_tree_digest
cargo_source_snapshot_digest
platform_set_digest, validation_profile_version
generator_tag, generator_source_commit, generator_source_digest
generator_asset_digest?, generator_binary_digest
cargo_binary_digest, rustc_binary_digest
effective_environment_digest, sandbox_profile_version, validator_binary_digest
output_digest, output_bytes
```

Fields are canonically ordered and exclude wall clock, hostname, checkout path,
temporary path, username, PID, and unrestricted environment. A release asset
digest is present when a prebuilt binary is used; a reproducible-build receipt
replaces it when the binary is built from source.

A separate publication-attempt receipt contains `generation_id`, destination
preimage digest, publisher capability profile/version, and the actual
`Unchanged` or `Replaced` outcome. Publication outcome does not alter the
generation identity, but it does alter the attempt-receipt identity.

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

The behavior belongs to Build and stays separate from `build/port-engine`.
ADR-0719's current meta-root grammar and shared workspace/build declarations
make the exact package path a structural decision. The preferred shape is a
capability-style `build/dependency-declarations/{core,ports,adapters,facade}`
tree, but no code path or root/workspace mutation is authorized until the
independent implementation plan names the admitted paths, Buck targets, package
names, shared-file writer, and required cross-owner reviews.

Build's reusable analysis/transformation provider role is adopted, but the
compilation-unit, semantic-fact, conformance, and recipe contracts remain the
nonbinding details recorded in `ADR.md`; this specification does not define
their schemas or implementation homes.

</placement_boundary>
