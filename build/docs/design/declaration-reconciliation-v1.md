---
doc_class: Owner-Design
owner: build
status: Proposed
date: 2026-08-27
base: 505828b377dbd4b6705f50af6369fc1a8a98b21a
---

# Declaration reconciliation v1 execution map

This is the L1c execution map, not an ADR fork, second plan authority, evidence
dump, or claim of landed behavior. Authority remains
[`build/ADR.md`](../../ADR.md), [`build/PRD.md`](../../PRD.md),
[`build/SPEC.md`](../../SPEC.md), and [`build/PLAN.md`](../../PLAN.md), inheriting
[ADR-0719](../../../docs/decisions/ADR-0719-eac-serving-control-north-star.md)
D-27/D-43. Evidence is the exact base above, L1b's measured blocker at
`build/PLAN.md:70-95`, and the pinned Reindeer source named below.

## Rulings and closed shape

`build/` is a meta-root, while declaration reconciliation is a Build-owned
subsystem. Therefore its admitted shape is
`build/dependency-declarations/<face>/<crate>`, distinct from and with no edge
to frozen `build/port-engine/**`. Current admission does not recognize this
nested owner grammar and the root workspace admits only the port-engine
exception; the Pipeline prerequisite below lands first.

The six packages are irreducible: two capability-agnostic effect traits, one
pure reconciliation core, two implementations, and one real process/composition
root. Validation is core behavior. No seventh orchestration, CLI, graph, or
storage package is admitted.

| Path; Cargo package; entry point | Owned files and Buck targets | Dependency direction |
|---|---|---|
| `core/reconcile`; `dependency-declarations-reconcile`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{canonical,digest,error,platform,receipt,request,validation,reconcile}.rs`, `tests/{request,determinism,validation,publication,refusals}.rs`; `:dependency-declarations-reconcile` plus `:dependency-declarations-reconcile-request-test`, `:dependency-declarations-reconcile-determinism-test`, `:dependency-declarations-reconcile-validation-test`, `:dependency-declarations-reconcile-publication-test`, and `:dependency-declarations-reconcile-refusals-test` | generation port + publication port + workspace `sha2`; no IO crate |
| `ports/generation`; `dependency-declarations-generation`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `src/lib.rs`, `tests/contract.rs`; `:dependency-declarations-generation`, `:dependency-declarations-generation-contract-test` | `std` only; owns `GenerationPort<Request, Output, Error>` |
| `ports/publication`; `dependency-declarations-publication`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `src/lib.rs`, `tests/contract.rs`; `:dependency-declarations-publication`, `:dependency-declarations-publication-contract-test` | `std` only; owns `PublicationPort<Request, Output, Error>` |
| `adapters/generation-reindeer`; `dependency-declarations-generation-reindeer`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{environment,process,sandbox,snapshot}.rs`, `tests/{process,nine_platforms}.rs`, `tests/fixtures/nine-platforms/{workspace,fixups}.txt`; `:dependency-declarations-generation-reindeer`, `:dependency-declarations-generation-reindeer-process-test`, and `:dependency-declarations-generation-reindeer-nine-platforms-test` | core + generation port; exact Reindeer process only |
| `adapters/publication-filesystem`; `dependency-declarations-publication-filesystem`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{capability,lease,publish,recovery}.rs`, `tests/{faults,old_or_new,recovery}.rs`; `:dependency-declarations-publication-filesystem`, `:dependency-declarations-publication-filesystem-faults-test`, `:dependency-declarations-publication-filesystem-old-or-new-test`, and `:dependency-declarations-publication-filesystem-recovery-test` | core + publication port + workspace `rustix` |
| `facade/reconciler-app`; `dependency-declarations-reconciler-app`; `src/main.rs` (+ `src/lib.rs`) | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/{lib,main}.rs`, `src/items/{codec,resource,service}.rs`, `tests/{resource_status,process,freshness,dependency_direction}.rs`; `:dependency-declarations-reconciler-app`, `:dependency-declarations-reconciler-app-bin`, `:dependency-declarations-reconciler-app-resource-status-test`, `:dependency-declarations-reconciler-app-process-test`, `:dependency-declarations-reconciler-app-freshness-test`, and `:dependency-declarations-reconciler-app-dependency-direction-test` | core, both ports, both adapters; composition only |

Every multi-item crate's immutable `build.rs` sorts direct `src/items/*.rs`
and writes module membership to `OUT_DIR`; later item lanes add unique files and
never hand-edit an index. Cargo uses canonical library discovery and only the
facade uses canonical `src/main.rs`. BUCK rules name the paths above explicitly
or with bounded `src/**/*.rs`/`tests/*.rs` globs; there is no parent BUCK
index. All package path dependencies are direct, so no new root
`[workspace.dependencies]` entry is needed. Libraries are visible only within
`//build/dependency-declarations/...`; the binary is private until a consumer
adopts the API. `dependency_direction.rs` parses all six Cargo/BUCK declarations
and admits only the edges above, including no port-engine/Pipeline label. Thus
path shape, Cargo/Buck target kind, entrypoint, dependency direction,
visibility, OWNERS and the tested serving relationship jointly prove role; an
empty `main.rs` never does, and structure never claims production readiness.
The serialized structural writer owns any `Cargo.lock` normalization.

## Pipeline-owned prerequisite

A separate protected Pipeline PR precedes Build scaffolding and changes only:

- `pipeline/core/admission/src/layout.rs` and `layout/inner.rs`: recognize only
  `build/dependency-declarations/`, require the four faces and canonical crate
  leaves, and reject subsystem law/docs, `facade/proto`, and every other nested
  Build Cargo shape;
- `pipeline/core/admission/src/layout/manifest.rs`: bind the subsystem package
  prefix `dependency-declarations` (the same established rule as
  `port-engine-*`), library/facade target kind, canonical
  entrypoint, and paths below each crate to its manifest;
- `pipeline/core/admission/src/layout/workspace.rs`: admit the optional pair
  member `build/dependency-declarations/*/*/src/..` and literal-fallback exclude
  `build/dependency-declarations/*/*`; absent-or-both-once is valid, while an
  unpaired, reordered, duplicate, broader, or recursive form is red;
- `pipeline/core/admission/tests/layout_adversarial.rs` and
  `tests/layout_change.rs`: prove the exact accepted/rejected paths, six package
  names/entrypoints, and optional closed workspace pair.

Pipeline owns that grammar and its review; it implements no Build behavior.
The optional pair lets this PR land while root `Cargo.toml` remains unchanged.
The later serialized Build structural lane adds both root entries together.

## Frozen Rust contract

The port signatures are generic and dependency-free:

```rust
pub trait GenerationPort<Request, Output, Error> {
    fn generate(&self, request: &Request) -> Result<Output, Error>;
}
pub trait PublicationPort<Request, Output, Error> {
    fn publish(&self, request: &Request) -> Result<Output, Error>;
}
```

Core binds them to `GenerationInvocationV1 -> RawGenerationV1 /
GenerationPortErrorV1` and `PublicationRequestV1 -> PublicationObservationV1 /
PublicationPortErrorV1`. `reconcile<G, P>(&ReconciliationRequestV1, &G, &P) ->
ReconciliationResultV1` admits inputs, invokes A/B, compares bytes, validates,
constructs the generation identity, then optionally publishes. Only core can
construct `ValidatedGenerationV1` or a publishable request. Fake generator and
publisher implementations live in integration-test support, not production.

`ReconciliationRequestV1` owns `RepositoryCorrelationV1`, logical
`InputFileV1` values for manifest/lock/config, `InputTreeV1` for fixups and the
read-only Cargo source snapshot, `PlatformSetV1`, `GeneratorIdentityV1`, exact
Cargo/rustc `ToolIdentityV1`s, and environment/sandbox/validation profiles.
Physical checkout/stage paths belong only to `GenerationInvocationV1` and are
excluded from identity. `RawGenerationV1` is bytes plus bounded stderr;
`ValidatedGenerationV1` adds output digest/length and validation-profile ID.

`ReconciliationResultV1` is exactly `Refused { request_id, failure }`,
`Generated { generation }`, or `Published { generation, attempt }`.
`PublicationAttemptReceiptV1` outcomes are `Unchanged`, `Replaced`,
`Failed { class, replacement: No }`, or `Indeterminate { replacement:
Maybe, durability: Unknown }`; publication failure never mutates the generation
identity. The stable classes are exactly those in `build/SPEC.md`:
`InvalidRequest`, `InputChanged`, `MissingFixup`, `GeneratorUnavailable`,
`GeneratorFailed`, `GeneratorTimedOut`, `GeneratorOutputTooLarge`,
`NondeterministicOutput`, `InvalidGeneratedGraph`,
`UnsupportedPublicationProfile`, `DestinationLeaseUnavailable`, `LeaseLost`,
`DestinationConflict`, `StageWriteFailed`, `StageSyncFailed`, `ReplaceFailed`,
`DirectorySyncFailed`, and `InternalInvariant`.

Canonical v1 encoding is a domain tag, one-byte enum tags, big-endian lengths,
exact UTF-8 bytes (no Unicode normalization), normalized repository-relative
`/` paths, and lists sorted
by their documented key with duplicates rejected. Digests are SHA-256 spelled
`sha256:<64 lowercase hex>` using workspace `sha2`. `request_id` hashes
`build.declaration-request.v1\0` plus all semantic input/tool/profile fields;
`generation_id` hashes `build.declaration-generation.v1\0`, request ID, output
digest and length; `attempt_id` hashes `build.declaration-publication.v1\0`,
generation ID, destination preimage, publisher profile, and actual outcome.
Wall clock, host, user, PID, checkout/temp paths, and unrestricted environment
never enter these encodings.

`ValidationBoundsV1` freezes: 32 MiB per declared file, 16,384 fixup files and
64 MiB fixup bytes, 16 GiB source snapshot, 64 MiB generator stdout, 1 MiB
stderr, 120 seconds/run, 4,096-byte path, 100,000 rules, 512 attributes/rule,
131,072 list entries/attribute, 1 MiB string, 1,000,000 reference edges, and
8 KiB rendered diagnostic. Limit+1 refuses with the phase-specific class.

`EnvironmentProfileV1("reindeer-hermetic-v1")` supplies fixed `LANG=C`,
`LC_ALL=C`, `TZ=UTC`, `CARGO_NET_OFFLINE=true`, `CARGO_INCREMENTAL=0`, exact
tool paths, a read-only Cargo home/source snapshot, and an empty writable stage.
It clears proxies, credentials, wrappers, rustflags, target-dir/config,
compiler/linker substitution, and loader variables. `SandboxProfileV1` permits
only declared reads, the stage write, process execution and no network.
`ValidatorProfileV1("reindeer-buck-v1")` parses the whole file and enforces the
syntax, unique names, resolved edges, aliases, platform/fixup effects, canonical
header/imports, bounded shape, and absence of absolute/stage/legacy-overlay
text. Publisher profiles are `linux-local-v1` (ext4/xfs) and `macos-local-v1`
(APFS): directory-handle no-follow operations, an exclusive sibling lease,
same-directory regular temp, flush/mode/file sync, atomic rename, parent sync,
and owned-prefix regular-file recovery. Any other platform/filesystem refuses.

The facade defines `ReconciliationResourceV1 { resource_id, generation, spec,
publish }` and `ReconciliationStatusV1 { observed_generation, phase,
generation_id, attempt_id, failure }`; phases are `Pending|Running|Succeeded|
Failed|Indeterminate`. `serve_framed<R: Read, W: Write>` consumes one bounded,
length-prefixed canonical resource stream and emits statuses with backpressure.
`main.rs` calls that service over inherited stdin/stdout handles: no argv,
subcommands, prompts, shell, network, or diagnostic CLI. Process tests prove a
real request/status transition and check-only freshness; this structural/API
proof explicitly does not claim production ingress, authn, deployment, or SLO
readiness.

## Reindeer, fixups, and platforms

The sole source candidate is Reindeer `v2026.08.10.00`, commit
`bb681570d2bc47d1446080c12b8681a50a95f628`. It is qualification-blocked, not a
qualified or publishable binary. A later source build binds source-tree and
lock digests, exact builder toolchain/target/flags, reproducible-build receipt
and binary SHA-256; a reviewed release asset may instead bind its asset digest.
Promotion then requires two clean equal runs, perturbation detection, alias,
native/generated-source and Buck-consumer evidence.

One serialized declaration writer owns `Cargo.toml`, `Cargo.lock`, all six
package manifests/OWNERS/BUCK/entrypoints, `reindeer.toml`,
`third-party/PACKAGE`, fifteen explicit fixup files, and tool-materialized
`third-party/BUCK`. It never hand-edits generated output. L1b's eleven measured
decisions are:

- `iam-pdp-app`: run the build script with declared `proto/iam-pdp.proto` and
  vendored protoc inputs; `intelligence-supervisor-app`: `run=false` while its
  base script emits only rerun markers;
- `storage-block-http-legacy-draft`: run `src/items`;
  `storage-domain`: run `src/{items,cas_items,test_items,cas_test_items}`;
  `storage-object-http-legacy-draft`: run `src/{items,test_items}` and
  `tests/items`; `storage-oci-adapter`: run `src/{items,test_items}`;
- `storage-provider-block-oci-draft`: run `src/items` and `tests/items`;
  `storage-provider-draft`: run `src/{items,test_items}`;
  `storage-provider-object-oci-draft`: run `src/items` and `tests/items`;
  `storage-provider-object-s3-draft` and `storage-s3-adapter`: run
  `src/{items,test_items}`.

Each maps to its package-named `third-party/fixups/` directory; a source/effect change
invalidates the decision. The four new scanner scripts add `run=true` fixups at
`third-party/fixups/dependency-declarations-{reconcile,generation-reindeer,
publication-filesystem,reconciler-app}/fixups.toml`, each declaring its direct
`src/items` input; they land with the scaffold so unresolved-fixup count cannot
grow silently.

AWS-LC DEP metadata must become supported fixup/generator behavior. PSM 0.1.31
gets an explicit nine-platform matrix: x86_64 Linux GNU/musl and macOS use
`src/arch/x86_64.s`; aarch64 Linux GNU/musl and macOS use
`src/arch/aarch_aapcs64.s`; Windows GNU uses
`src/arch/x86_64_windows_gnu.s`; Windows MSVC binds the qualified execution-host
choice between that GAS source and `src/arch/x86_64_msvc.asm`; wasm32 archives
`src/arch/wasm32.o`. Every branch binds exact `asm`/`link_asm`/
`switchable_stack`, OS/arch/env defines and target compatibility. If Reindeer
cannot express AWS-LC, any PSM branch, or the Windows host/toolchain choice
without output rewriting, Q refuses; no wrapper or second candidate replaces it.

`reindeer.toml` retains default names to minimize generated-key churn and binds:
`linux-x86_64`=`x86_64-unknown-linux-gnu`,
`linux-arm64`=`aarch64-unknown-linux-gnu`,
`linux-x86_64-musl`=`x86_64-unknown-linux-musl`,
`linux-arm64-musl`=`aarch64-unknown-linux-musl`,
`macos-x86_64`=`x86_64-apple-darwin`, `macos-arm64`=`aarch64-apple-darwin`,
`windows-gnu`=`x86_64-pc-windows-gnu`,
`windows-msvc`=`x86_64-pc-windows-msvc`, and
`wasm32`=`wasm32-unknown-unknown`. Only Linux GNU x86_64/arm64 have
`execution-platform=true`, matching protected x86_64 Linux and nightly arm64
Linux runners; the other seven are target-only.

The generation adapter `BUCK` owns paired `config_setting` and `platform`
labels using the bundled Prelude constraints shown here (`none` OS for wasm):

| Reindeer name | Prelude cpu/os/abi | Exact select label suffix | Exact platform label suffix |
|---|---|---|---|
| `linux-x86_64` | `x86_64/linux/gnu` | `select-linux-x86-64-gnu` | `platform-linux-x86-64-gnu` |
| `linux-arm64` | `arm64/linux/gnu` | `select-linux-arm64-gnu` | `platform-linux-arm64-gnu` |
| `linux-x86_64-musl` | `x86_64/linux/musl` | `select-linux-x86-64-musl` | `platform-linux-x86-64-musl` |
| `linux-arm64-musl` | `arm64/linux/musl` | `select-linux-arm64-musl` | `platform-linux-arm64-musl` |
| `macos-x86_64` | `x86_64/macos` | `select-macos-x86-64` | `platform-macos-x86-64` |
| `macos-arm64` | `arm64/macos` | `select-macos-arm64` | `platform-macos-arm64` |
| `windows-gnu` | `x86_64/windows/gnu` | `select-windows-x86-64-gnu` | `platform-windows-x86-64-gnu` |
| `windows-msvc` | `x86_64/windows/msvc` | `select-windows-x86-64-msvc` | `platform-windows-x86-64-msvc` |
| `wasm32` | `wasm32/none` | `select-wasm32-unknown-unknown` | `platform-wasm32-unknown-unknown` |

Every suffix is under
`root//build/dependency-declarations/adapters/generation-reindeer:`.
`third-party/PACKAGE` calls `set_reindeer_platforms` from
`@prelude//rust:cargo_package.bzl`, maps the nine exact select labels to the
names above, and maps `DEFAULT` to `None`. These commands must each select one
mapping; zero or multiple matches refuse promotion:

```text
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-x86-64-gnu 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-arm64-gnu 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-x86-64-musl 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-arm64-musl 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-macos-x86-64 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-macos-arm64 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-windows-x86-64-gnu 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-windows-x86-64-msvc 'third-party//:psm-0.1'
buck2 cquery --target-platforms root//build/dependency-declarations/adapters/generation-reindeer:platform-wasm32-unknown-unknown 'third-party//:psm-0.1'
```

## Reversible delivery waves

| Wave | Red/green proof and success | Refusal/fault and rollback |
|---|---|---|
| P: Pipeline prerequisite | `cargo test --locked --offline -p pipeline-admission --test layout --test layout_adversarial --test layout_change`; absent/paired glob and exact paths pass | broad meta face, wrong name/entrypoint, half-pair fail; revert Pipeline PR |
| S: serialized structure | add exact six crates and paired root entries; `cargo metadata --locked --offline --format-version 1 --no-deps`, `cargo fmt --all --check`, and `buck2 targets 'root//build/dependency-declarations/...'` resolve every named target | missing member/owner/target, port-engine/Pipeline edge fail; revert unconsumed scaffold + root pair |
| K: pure core | package tests start red, then cover canonical IDs, bounds, A/B mismatch, validation, stable failures and fake ports; `cargo test --locked --offline -p dependency-declarations-reconcile` | ambient IO, panic, limit+1, order/path drift fail; revert only K item/test files |
| A: adapters | `cargo test --locked --offline -p dependency-declarations-generation-reindeer -p dependency-declarations-publication-filesystem`; tests cover exact process/env/sandbox and every lease/write/sync/rename/recovery fault | network/env leak, timeout orphan, partial visibility or dishonest durability fail; withdraw adapters, retain core |
| F: reconciler/freshness | `cargo test --locked --offline -p dependency-declarations-reconciler-app`; resource/status, process and freshness tests pass with no CLI parser or network/forge transport dependency | vacant main, mutation in check-only, missing status/backpressure fail; withdraw facade |
| D: declaration conversion | `cargo test --locked --offline -p dependency-declarations-reconciler-app --test freshness`; resolve the eleven inherited decisions plus four scanner decisions, all nine platforms, AWS-LC/PSM, and materialize only through the engine; two immediate check-only runs are clean | missing fixup, overlay/header residue, hand edit or unequal output fails; restore prior complete config/fixup/BUCK tuple |
| Q: qualification | two clean roots produce equal bytes/IDs; cquery all nine and build `third-party//:{aws-lc-rs,aws-lc-sys-0.41,psm-0.1,psm-0.1-psm_asm,any_spawner,syn-2,quote-1,wasm-bindgen-futures,web-sys}` on qualified hosts; perturb each input and observe red | alias/native/generated/platform/consumer or p95 evidence gap blocks promotion; republish last qualified tuple |

Every wave also runs `git diff --check`, `cargo fmt --all --check`, locked
offline metadata, its package tests, and the protected path-layout application.
Build supplies neutral machinery and receipts; product owners supply semantic
intent, postconditions and acceptance; Pipeline alone owns dependency-closed
campaign waves and protected review. Manual migrations become gold fixtures,
never the automated campaign. L1 owns no Git/GitHub/merge state, CNA or security
severity, database/schema or customer-data migration, traffic shift, deployment
or cell evacuation; it may only require receipts from those systems.

L2 consumes this seam only after Q: declared MSRV, qualified production stable,
beta, and exact dated nightly remain independent identities. Rust/dependency and
CVE/CNA candidates enter through L1 ChangeSets/receipts without moving MSRV or
security ownership. Cargo `min-publish-age` and `update-breaking` are unstable
observations, never durable APIs.
