---
doc_class: Owner-Design
owner: build
status: Proposed
date: 2026-08-27
base: 505828b377dbd4b6705f50af6369fc1a8a98b21a
revision: 2
amended: 2026-08-28
---

# Declaration reconciliation v1 execution map

This is the L1c execution map, not an ADR fork, second plan authority, evidence dump, or landed-behavior claim. Authority remains [`build/ADR.md`](../../ADR.md), [`build/PRD.md`](../../PRD.md), [`build/SPEC.md`](../../SPEC.md), and [`build/PLAN.md`](../../PLAN.md), inheriting [ADR-0719](../../../docs/decisions/ADR-0719-eac-serving-control-north-star.md) D-27/D-43. Evidence is the exact base above, L1b's blocker at `build/PLAN.md:70-95`, and the pinned source below.

## Rulings and closed shape

`build/` is a meta-root; declaration reconciliation is a Build-owned subsystem at
`build/dependency-declarations/<face>/<crate>`, distinct from and with no edge to frozen `build/port-engine/**`. Current admission recognizes neither that nested grammar nor its workspace pair, so the Pipeline prerequisite lands first.

The irreducible six packages are two effect traits, one pure core, two implementations, and one real process/composition root. Validation stays in core; no seventh orchestration, CLI, graph, or storage package is admitted.

| Path; Cargo package; entry point | Owned files and Buck targets | Dependency direction |
|---|---|---|
| `core/reconcile`; `dependency-declarations-reconcile`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{canonical,digest,error,platform,receipt,request,validation,reconcile}.rs`, `tests/{request,determinism,validation,publication,refusals,module_membership}.rs`; `:dependency-declarations-reconcile`, `:dependency-declarations-reconcile-request-test`, `:dependency-declarations-reconcile-determinism-test`, `:dependency-declarations-reconcile-validation-test`, `:dependency-declarations-reconcile-publication-test`, `:dependency-declarations-reconcile-refusals-test`, `:dependency-declarations-reconcile-module-membership-test` | generation port + publication port + workspace `sha2`; no IO crate |
| `ports/generation`; `dependency-declarations-generation`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `src/lib.rs`, `tests/contract.rs`; `:dependency-declarations-generation`, `:dependency-declarations-generation-contract-test` | `std` only; owns `GenerationPort<Request, Output, Error>` |
| `ports/publication`; `dependency-declarations-publication`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `src/lib.rs`, `tests/contract.rs`; `:dependency-declarations-publication`, `:dependency-declarations-publication-contract-test` | `std` only; owns `PublicationPort<Request, Output, Error>` |
| `adapters/generation-reindeer`; `dependency-declarations-generation-reindeer`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{environment,process,sandbox,snapshot}.rs`, `tests/{process,nine_platforms,module_membership}.rs`, `tests/fixtures/nine-platforms/{workspace,fixups}.txt`; `:dependency-declarations-generation-reindeer`, `:dependency-declarations-generation-reindeer-process-test`, `:dependency-declarations-generation-reindeer-nine-platforms-test`, `:dependency-declarations-generation-reindeer-module-membership-test` | core + generation port; exact Reindeer process only |
| `adapters/publication-filesystem`; `dependency-declarations-publication-filesystem`; `src/lib.rs` | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/lib.rs`, `src/items/{capability,lease,publish,recovery}.rs`, `tests/{faults,old_or_new,recovery,module_membership}.rs`; `:dependency-declarations-publication-filesystem`, `:dependency-declarations-publication-filesystem-faults-test`, `:dependency-declarations-publication-filesystem-old-or-new-test`, `:dependency-declarations-publication-filesystem-recovery-test`, `:dependency-declarations-publication-filesystem-module-membership-test` | core + publication port + workspace `rustix` |
| `facade/reconciler-app`; `dependency-declarations-reconciler-app`; `src/main.rs` (+ `src/lib.rs`) | `Cargo.toml`, `OWNERS`, `BUCK`, `build.rs`, `src/{lib,main}.rs`, `src/items/{codec,resource,service}.rs`, `tests/{resource_status,process,freshness,dependency_direction,module_membership}.rs`; `:dependency-declarations-reconciler-app`, `:dependency-declarations-reconciler-app-bin`, `:dependency-declarations-reconciler-app-resource-status-test`, `:dependency-declarations-reconciler-app-process-test`, `:dependency-declarations-reconciler-app-freshness-test`, `:dependency-declarations-reconciler-app-dependency-direction-test`, `:dependency-declarations-reconciler-app-module-membership-test` | core, both ports, both adapters; composition only |

Each multi-item crate's immutable `build.rs` sorts direct `src/items/*.rs` into `OUT_DIR`; item lanes add unique files, never an index. Cargo uses canonical library discovery; only the facade has `src/main.rs`. BUCK names these paths explicitly or with bounded `src/**/*.rs`/`tests/*.rs` globs; there is no parent BUCK; direct path dependencies need no new root `[workspace.dependencies]` entry.
Libraries are visible only within `//build/dependency-declarations/...`; the binary stays private until API adoption. `dependency_direction.rs` parses all six Cargo/BUCK declarations and admits only the edges above, including no port-engine/Pipeline label. Path, Cargo/Buck target kind, entrypoint, dependency direction, visibility, OWNERS and tested serving relationship jointly prove role; empty `main.rs` never does, structure never proves production readiness, and the serialized writer owns `Cargo.lock` normalization.

## OVERRULE — stage tracked scanner inputs with their first real item

- **achieves:** every intermediate declaration tuple remains Reindeer-resolvable and Buck-executable without placeholder behavior.
- **origin:** exact Reindeer candidate `bb681570d2bc47d1446080c12b8681a50a95f628` treats an empty tracked `src/items/*.rs` glob as unused and fatal; target listing does not execute OUT_DIR wiring.
- **rule:** S MUST land four empty-safe scanners and `run=true`-only fixups, testing sorted empty output, run decision, and no premature tracked input. K MUST atomically add core's first real item plus exact `extra_srcs`; A MUST do so for both adapters; F MUST do so for the facade. Each is a serialized shared-declaration sublane; later item lanes add unique files without editing scanner, parent/index, or fixup. S MUST Buck-build six libraries and the private facade binary, not merely target-list them.
- **ensure:** exact-candidate source audit, package tests, rejection of premature or absent post-first-item input, explicit Buck builds, path-layout, and independent review.
- **overturn_when:** a reviewed, qualified Reindeer primitive supports empty tracked globs (or an equally exact no-placeholder input model) and its same-wave replacement proves clean generation and Buck execution.

## Pipeline-owned prerequisite

A separate protected Pipeline PR precedes Build scaffolding and changes only:

- `pipeline/core/admission/src/layout.rs` and `layout/inner.rs`: recognize only `build/dependency-declarations/`, require four faces/canonical crate leaves, and reject subsystem law/docs, `facade/proto`, and every other nested Build Cargo shape;
- `pipeline/core/admission/src/layout/manifest.rs`: bind prefix `dependency-declarations` (as `port-engine-*` does), target kind, entrypoint, and crate path to manifest;
- `pipeline/core/admission/src/layout/workspace.rs`: admit optional member `build/dependency-declarations/*/*/src/..` plus literal-fallback exclude `build/dependency-declarations/*/*`; absent or both-once is valid, every unpaired/reordered/duplicate/broader/recursive form red;
- `pipeline/core/admission/tests/layout_adversarial.rs` and `tests/layout_change.rs`: prove exact paths, six names/entrypoints, and the closed optional pair.

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

The v1 fields, in canonical declaration order, are:

```rust
struct RepositoryCorrelationV1 { repository_id: String, revision: String }
struct CanonicalPathV1(String)
enum InputFileRoleV1 { Manifest, Lock, Config, TreeManifest } enum TreeRoleV1 { Fixups, CargoSource }
struct InputFileV1 { role: InputFileRoleV1, path: CanonicalPathV1, length_bytes: u64, sha256: DigestV1, bytes: Box<[u8]> }
struct TreeEntryV1 { path: CanonicalPathV1, length_bytes: u64, sha256: DigestV1 }
struct InputTreeV1 { role: TreeRoleV1, manifest: InputFileV1, root_sha256: DigestV1, file_count: u64, total_bytes: u64 }
struct PlatformIdentityV1 { name: String, target_triple: String, select_label: String, platform_label: String, execution_platform: bool }
struct PlatformSetV1 { entries: Box<[PlatformIdentityV1]> }
struct GeneratorIdentityV1 { name: String, version: String, source_revision: String, source_tree_sha256: DigestV1, binary_sha256: DigestV1, binary: GeneratorBinaryV1 }
struct ToolIdentityV1 { name: String, version: String, commit: String, host_triple: String, binary_sha256: DigestV1 }
enum GeneratorBinaryV1 { ReproducibleBuild { receipt_sha256: DigestV1 }, ReleaseAsset { asset_sha256: DigestV1 } }
enum EnvironmentProfileV1 { ReindeerHermeticV1 } enum SandboxProfileV1 { DeclaredReadStageWriteNoNetworkV1 } enum ValidatorProfileV1 { ReindeerBuckV1 }
enum PublisherProfileV1 { LinuxExt4V1, LinuxXfsV1, MacosApfsV1 }
struct GenerationRequestV1 { repository: RepositoryCorrelationV1, manifest: InputFileV1, lock: InputFileV1, reindeer_config: InputFileV1, fixups: InputTreeV1, cargo_sources: InputTreeV1, platforms: PlatformSetV1, generator: GeneratorIdentityV1, cargo: ToolIdentityV1, rustc: ToolIdentityV1, environment: EnvironmentProfileV1, sandbox: SandboxProfileV1, validator: ValidatorProfileV1 }
struct ReconciliationRequestV1 { generation: GenerationRequestV1, publish: Option<PublicationIntentV1> }
struct GenerationInvocationV1 { request_id: DigestV1, request: GenerationRequestV1 }
struct RawGenerationV1 { bytes: Box<[u8]>, stderr: Box<[u8]> }
enum GenerationPortErrorV1 { InputChanged, MissingFixup, GeneratorUnavailable, GeneratorFailed, GeneratorTimedOut, GeneratorOutputTooLarge, InternalInvariant }
type PublicationPortErrorV1 = core::convert::Infallible;
struct ValidatedGenerationV1 { request_id: DigestV1, generation_id: DigestV1, output_sha256: DigestV1, output_length_bytes: u64, bytes: Box<[u8]>, validator: ValidatorProfileV1 }
struct PublicationIntentV1 { expected_preimage: Option<DigestV1>, publisher: PublisherProfileV1 }
struct PublicationRequestV1 { generation: ValidatedGenerationV1, intent: PublicationIntentV1 }
struct PublicationObservationV1 { outcome: PublicationOutcomeV1 }
enum FailureClassV1 { InvalidRequest, InputChanged, MissingFixup, GeneratorUnavailable, GeneratorFailed, GeneratorTimedOut, GeneratorOutputTooLarge, NondeterministicOutput, InvalidGeneratedGraph, UnsupportedPublicationProfile, DestinationLeaseUnavailable, LeaseLost, DestinationConflict, StageWriteFailed, StageSyncFailed, ReplaceFailed, DirectorySyncFailed, InternalInvariant }
struct FailureV1 { class: FailureClassV1 }
enum ReplacementStateV1 { No, Maybe } enum DurabilityStateV1 { Unknown }
enum PublicationOutcomeV1 { Unchanged, Replaced, Failed { failure: FailureV1, replacement: ReplacementStateV1 }, Indeterminate { failure: FailureV1, replacement: ReplacementStateV1, durability: DurabilityStateV1 } }
struct PublicationAttemptReceiptV1 { attempt_id: DigestV1, generation_id: DigestV1, expected_preimage: Option<DigestV1>, publisher: PublisherProfileV1, outcome: PublicationOutcomeV1 }
enum ReconciliationResultV1 { Refused { request_id: Option<DigestV1>, failure: FailureV1 }, Generated { generation: ValidatedGenerationV1 }, Published { generation: ValidatedGenerationV1, attempt: PublicationAttemptReceiptV1 } }
enum ReconciliationPhaseV1 { Pending, Running, Succeeded, Failed, Indeterminate }
```

`GenerationPortErrorV1` is an adapter-origin Rust enum with no independent wire encoding: its variants map one-to-one to same-named `FailureV1` classes/tags `1/2/3/4/5/6/17`; core returns `Refused { request_id: Some(request_id), failure }`, and no other generation error conforms. `PublicationPortErrorV1` is `Infallible`: an invalid `PublicationRequestV1` or unsupported profile/capability is refused before `publish` and is not an attempt. Once invoked, the adapter always returns `Ok(PublicationObservationV1)`; post-invocation failure classes are only `DestinationLeaseUnavailable` through `DirectorySyncFailed` or `InternalInvariant`, and adapter/process/filesystem faults are `Failed` with replacement `No` only when no replacement is proven, otherwise `Indeterminate` with replacement `Maybe` and durability `Unknown`, never panic or `Err`. Core constructs exactly one `PublicationAttemptReceiptV1` from every invocation's `Unchanged`, `Replaced`, `Failed`, or `Indeterminate` observation.

`DigestV1=[u8;32]`; display is `sha256:` plus 64 lowercase hex. Tags are one
byte and closed: file roles manifest/lock/config/tree-manifest=`0/1/2/3`; tree
roles fixups/Cargo-source=`0/1`; generator binary reproducible-build/release-
asset=`0/1` followed by its receipt/asset digest; environment/sandbox/validator
profiles `reindeer-hermetic-v1`/declared-read-stage-write-no-network-v1/
`reindeer-buck-v1` are each tag `0`; publishers Linux-ext4/Linux-xfs/macOS-APFS
are `0/1/2`. Result Refused/Generated/Published=`0/1/2`; publication outcome
Unchanged/Replaced/Failed/Indeterminate=`0/1/2/3`, with Failed then encoding
failure+replacement-No(`0`), and Indeterminate failure+replacement-Maybe(`1`)+
durability-Unknown(`0`). Failure tags `0..17` follow the stable-class order in
`build/SPEC.md`, from `InvalidRequest` through `InternalInvariant`; diagnostics
are bounded non-identity data. Status Pending/Running/Succeeded/Failed/
Indeterminate=`0/1/2/3/4`; any other tag refuses.

Structs concatenate fields above; enum payload follows its tag. `u64` and list/
byte/string lengths are unsigned big-endian; bool is `0|1`; digest is 32 raw
bytes; option is `0` or `1` then value; bytes and exact UTF-8 strings are length
then bytes, with no normalization. Lists are count then items: tree entries sort
by path bytes, platforms by name, and the fixed environment map by key; cargo then rustc is fixed field order. Any
duplicate sort key refuses. A path must be nonempty UTF-8, repository-relative,
`/`-separated, and contain no NUL, backslash, empty, `.` or `..` component;
invalid or duplicate paths refuse before hashing.

`InputTreeV1.manifest` is canonical streamed `TreeEntryV1` records; its root
hash covers `build.input-tree.fixups.v1\0` for role `0` or `build.input-tree.cargo-source.v1\0` for role `1`, then role, manifest digest, count and total. Core never
holds tree contents: the adapter opens only manifest-listed regular files with
no-follow capability handles, streams/digests each against its entry, and
returns `InputChanged` on mismatch. Thus the 16-GiB snapshot is never resident.
Physical checkout/stage/capability paths are adapter state, excluded from IDs.

`request_id` hashes `build.declaration-request.v1\0` plus `GenerationRequestV1`;
publish intent is excluded. `generation_id` hashes
`build.declaration-generation.v1\0`, request ID, output digest/length and
validator. `attempt_id` hashes `build.declaration-publication.v1\0` then every
receipt field except itself. Wall clock, host, user, PID and physical paths are
excluded. Refusal is `Refused { request_id: Option<DigestV1>, failure }`;
Generated/Published carry validated generation, with Published also its attempt.

`ValidationBoundsV1` freezes: 32 MiB per declared/manifest file; fixups 16,384
files/64 MiB; Cargo sources 1,000,000 files/16 GiB; stdout 64 MiB; stderr 1 MiB;
120 seconds/run; path 4,096 bytes; 100,000 rules; 512 attributes/rule; 131,072
list entries/attribute; string 1 MiB; 1,000,000 edges; diagnostic 8 KiB.
Limit+1 refuses with the phase-specific class.

The process calls `env_clear()`; its exact final map is `CARGO_HOME=@cargo-home`,
`CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`, `CARGO_TERM_COLOR=never`,
`HOME=@empty-home`, `LANG=C`, `LC_ALL=C`, `PATH=` (empty), `RUSTC=@rustc`,
`TMPDIR=@stage/tmp`, `TZ=UTC`. Tokens map only to absolute no-symlink capability
paths; `@reindeer`, `--cargo-path @cargo`, and `--rustc-path @rustc` must be
regular executable files in the read-only tool snapshot whose SHA-256 equals
the bound identities. No search/fallback is permitted. Sandbox tag `0` permits
only declared reads, stage/TMPDIR writes, those processes, and no network.
Validator tag `0` parses the whole output and enforces syntax, names, edges,
aliases, platform/fixup effects, header/imports, bounds and path absence.
Publisher tags `0/1/2` use directory-handle no-follow, exclusive sibling lease,
same-directory regular temp, flush/mode/file sync, atomic rename, parent sync
and owned-prefix recovery; every other platform/filesystem refuses.

The facade defines `ReconciliationResourceV1 { resource_id: String, generation: u64, spec: GenerationRequestV1, publish: Option<PublicationIntentV1> }`
and `ReconciliationStatusV1 { observed_generation: u64, phase: ReconciliationPhaseV1, generation_id: Option<DigestV1>, attempt_id: Option<DigestV1>, failure: Option<FailureV1> }`.
`serve_framed<R: Read, W: Write>` consumes one bounded,
length-prefixed canonical resource stream and emits statuses with backpressure.
`main.rs` calls that service over inherited stdin/stdout handles: no argv,
subcommands, prompts, shell, network, or diagnostic CLI. Process tests prove a
real request/status transition and check-only freshness; this structural/API
proof explicitly does not claim production ingress, authn, deployment, or SLO
readiness.

## Reindeer, fixups, and platforms

The sole source candidate is Reindeer `v2026.08.10.00`, commit `bb681570d2bc47d1446080c12b8681a50a95f628`. It is qualification-blocked, not a qualified or publishable binary. A later source build binds source-tree and lock digests, exact builder toolchain/target/flags, reproducible-build receipt and binary SHA-256; a reviewed release asset may instead bind its asset digest. Promotion then requires two clean equal runs, perturbation detection, alias, native/generated-source and Buck-consumer evidence.

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

Each maps to its package-named `third-party/fixups/` directory; a source/effect change invalidates the decision. S alone adds and tests `run=true`-only fixups at `third-party/fixups/dependency-declarations-{reconcile,generation-reindeer,publication-filesystem,reconciler-app}/fixups.toml`; the four package `module_membership` tests prove sorted empty output, the run decision, and no premature tracked input. K atomically adds core's first real item and exact `src/items/*.rs` input; A does so for each adapter and F for the facade. After that binding, item lanes add unique files without editing scanner, parent/index, or fixup, so the unresolved count never grows silently.

AWS-LC DEP metadata must become supported fixup/generator behavior. PSM 0.1.31
gets an explicit nine-platform matrix: x86_64 Linux GNU/musl and macOS use
`src/arch/x86_64.s`; aarch64 Linux GNU/musl and macOS use
`src/arch/aarch_aapcs64.s`; Windows GNU uses
`src/arch/x86_64_windows_gnu.s`; Windows MSVC uses
`src/arch/x86_64_msvc.asm` iff the executor is Windows and its compiler is
MSVC-like; every other qualified pair (including cross-host) uses the GNU `.s`; wasm32 archives
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
`wasm32`=`wasm32-unknown-unknown`. `execution-platform=true` is frozen for
`linux-x86_64`, `linux-arm64`, and `macos-arm64`: protected Ubuntu x86_64,
nightly Ubuntu arm64, and local `aarch64-apple-darwin` rustc `1.98.0/88d9e12ae178fab0fb5cc050a94da85685d449ea` plus Cargo `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`.
The other six are false. Reindeer must agree on build/proc-macro closure across
all three true entries; a target/host pair without an exact qualified executor,
compiler, assembler, archiver and linker receipt refuses rather than falling back.

The generation adapter `BUCK` owns paired `config_setting` and `platform`
labels using the bundled Prelude constraints shown here (`none` OS for wasm):

| Reindeer name | Prelude cpu/os/abi | Exact select label suffix | Exact platform label suffix | exec |
|---|---|---|---|---|
| `linux-x86_64` | `x86_64/linux/gnu` | `select-linux-x86-64-gnu` | `platform-linux-x86-64-gnu` | true |
| `linux-arm64` | `arm64/linux/gnu` | `select-linux-arm64-gnu` | `platform-linux-arm64-gnu` | true |
| `linux-x86_64-musl` | `x86_64/linux/musl` | `select-linux-x86-64-musl` | `platform-linux-x86-64-musl` | false |
| `linux-arm64-musl` | `arm64/linux/musl` | `select-linux-arm64-musl` | `platform-linux-arm64-musl` | false |
| `macos-x86_64` | `x86_64/macos` | `select-macos-x86-64` | `platform-macos-x86-64` | false |
| `macos-arm64` | `arm64/macos` | `select-macos-arm64` | `platform-macos-arm64` | true |
| `windows-gnu` | `x86_64/windows/gnu` | `select-windows-x86-64-gnu` | `platform-windows-x86-64-gnu` | false |
| `windows-msvc` | `x86_64/windows/msvc` | `select-windows-x86-64-msvc` | `platform-windows-x86-64-msvc` | false |
| `wasm32` | `wasm32/none` | `select-wasm32-unknown-unknown` | `platform-wasm32-unknown-unknown` | false |

Every suffix is under
`root//build/dependency-declarations/adapters/generation-reindeer:`.
`third-party/PACKAGE` calls `set_reindeer_platforms` from
`@prelude//rust:cargo_package.bzl`, maps the nine exact select labels to the
names above, and maps `DEFAULT` to `None`. On an executor qualified for each
pair, this retirement-marked local loop must expose the PSM source/defines and
build PSM/AWS-LC/consumer closure. AWS-LC must either build when cquery reports
compatible or carry an explicit non-applicability receipt; ambiguity or failure refuses:

```sh
for p in root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-x86-64-gnu root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-arm64-gnu root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-x86-64-musl root//build/dependency-declarations/adapters/generation-reindeer:platform-linux-arm64-musl root//build/dependency-declarations/adapters/generation-reindeer:platform-macos-x86-64 root//build/dependency-declarations/adapters/generation-reindeer:platform-macos-arm64 root//build/dependency-declarations/adapters/generation-reindeer:platform-windows-x86-64-gnu root//build/dependency-declarations/adapters/generation-reindeer:platform-windows-x86-64-msvc root//build/dependency-declarations/adapters/generation-reindeer:platform-wasm32-unknown-unknown; do
  buck2 cquery --json --output-attribute '^(srcs|preprocessor_flags|target_compatible_with)$' --target-platforms "$p" 'deps(set(third-party//:psm-0.1-psm_asm third-party//:psm-0.1 third-party//:aws-lc-rs third-party//:aws-lc-sys-0.41 third-party//:any_spawner third-party//:syn-2 third-party//:quote-1 third-party//:wasm-bindgen-futures third-party//:web-sys root//app/application/facade/application-shell-frontend:application-shell-frontend root//app/application/facade/application-shell-frontend:application-shell-frontend-unittest root//app/application/facade/application-shell-frontend:application-shell-frontend-bin))'
  buck2 build --target-platforms "$p" third-party//:psm-0.1-psm_asm third-party//:psm-0.1 third-party//:aws-lc-rs third-party//:aws-lc-sys-0.41 third-party//:any_spawner third-party//:syn-2 third-party//:quote-1 third-party//:wasm-bindgen-futures third-party//:web-sys root//app/application/facade/application-shell-frontend:application-shell-frontend root//app/application/facade/application-shell-frontend:application-shell-frontend-unittest root//app/application/facade/application-shell-frontend:application-shell-frontend-bin
done
```

## Reversible delivery waves

| Wave | Red/green proof and success | Refusal/fault and rollback |
|---|---|---|
| P: Pipeline prerequisite | `cargo test --locked --offline -p pipeline-admission --test layout --test layout_adversarial --test layout_change`; absent/paired glob and exact paths pass | broad meta face, wrong name/entrypoint, half-pair fail; revert Pipeline PR |
| S: serialized structure | add six crates, paired root entries, four empty-safe scanners, four run-only fixups and four `module_membership` tests; run their Cargo test, metadata/fmt, `buck2 targets 'root//build/dependency-declarations/...'`, and the explicit Buck build below | premature tracked glob, nonempty placeholder, missing run decision, missing Buck target/OUT_DIR, half tuple, or forbidden edge fails; revert six-crate/root-pair/four-fixup tuple together |
| K: pure core | atomically add core's first real item and exact tracked `src/items/*.rs` input, then start red package tests for canonical IDs, bounds, A/B mismatch, validation, stable failures and fake ports; `cargo test --locked --offline -p dependency-declarations-reconcile` | ambient IO, panic, limit+1, order/path drift, or separated item/input binding fails; rollback removes both the core item and its input |
| A: adapters | atomically add each adapter's first real item and exact tracked input; `cargo test --locked --offline -p dependency-declarations-generation-reindeer -p dependency-declarations-publication-filesystem` covers exact process/env/sandbox and every lease/write/sync/rename/recovery fault | network/env leak, timeout orphan, partial visibility, dishonest durability, or separated adapter item/input binding fails; withdraw adapter items/input bindings together while retaining S |
| F: reconciler/freshness | atomically add the facade's first real item and exact tracked input; `cargo test --locked --offline -p dependency-declarations-reconciler-app` covers resource/status, process and freshness with no CLI parser or network/forge transport dependency | vacant main, check-only mutation, missing status/backpressure, or separated facade item/input binding fails; withdraw facade item/input binding while retaining S |
| D: declaration conversion | `cargo test --locked --offline -p dependency-declarations-reconciler-app --test freshness`; resolve 11 inherited decisions, revalidate all 15, bind platforms/AWS-LC/PSM, materialize through the engine, then obtain two clean checks | missing inherited fixup, residue, hand edit or unequal output fails; restore the pre-D config/11-fixup/PACKAGE/BUCK tuple while preserving S's four scanner fixups/scaffold, now with their exact tracked inputs |
| Q: qualification | two clean roots yield equal bytes/IDs; run the exact loop above on recorded qualified executors and perturb every input | any executor, alias/native/generated/platform/consumer or p95 gap blocks promotion; republish last qualified tuple |

Every wave also runs `git diff --check`, `cargo fmt --all --check`, locked
offline metadata, its package tests, and the protected path-layout application.
S's explicit Buck execution check is:

```text
buck2 build root//build/dependency-declarations/core/reconcile:dependency-declarations-reconcile root//build/dependency-declarations/ports/generation:dependency-declarations-generation root//build/dependency-declarations/ports/publication:dependency-declarations-publication root//build/dependency-declarations/adapters/generation-reindeer:dependency-declarations-generation-reindeer root//build/dependency-declarations/adapters/publication-filesystem:dependency-declarations-publication-filesystem root//build/dependency-declarations/facade/reconciler-app:dependency-declarations-reconciler-app root//build/dependency-declarations/facade/reconciler-app:dependency-declarations-reconciler-app-bin
```

Build supplies neutral machinery and receipts; product owners supply semantic
intent, postconditions and acceptance; Pipeline alone owns dependency-closed
campaign waves and protected review. Manual migrations become gold fixtures,
never the automated campaign. L1 owns no Git/GitHub/merge state, CNA or security
severity, database/schema or customer-data migration, traffic shift, deployment
or cell evacuation; it may only require receipts from those systems.

L2a read-heavy MSRV/Rust/dependency/CVE/CNA analysis continues now under `build/PLAN.md`; declared MSRV, qualified stable, beta and exact dated nightly remain independent identities. Only L2 behavior, candidate rendering and publication wait for Q and consume L1 ChangeSets/receipts without moving MSRV or Security ownership. Cargo `min-publish-age` and `update-breaking` remain unstable observations, never durable APIs.
