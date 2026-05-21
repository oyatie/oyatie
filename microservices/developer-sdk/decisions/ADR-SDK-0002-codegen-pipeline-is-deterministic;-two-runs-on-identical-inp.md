---
id: ADR-SDK-0002
title: "Developer SDK code generation is byte-deterministic"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0258
  - ADR-0263
decision_owner: axis-ecosystem + council-architecture
---

# ADR-SDK-0002: Developer SDK code generation is byte-deterministic

## Context

- The named pressure is `reproducible-sdk-distribution`: developers must be able to regenerate SDKs from the same OpenAPI/AsyncAPI/proto inputs and get byte-identical output.
- The prior incident class is `phantom-sdk-diff`: a codegen batch changed map ordering and created a large diff with no semantic change.
- The second prior incident class is `timestamp-poisoned-generated-file`: generated files included wall-clock timestamps and broke SLSA reproducibility.
- The third prior incident class is `locale-dependent-template-output`: local developer locale changed decimal and date formatting in generated examples.
- Developer SDKs are public artifacts and carry higher trust load than internal generated clients.
- ADR-0213 makes developer-sdk the public Ecosystem-as-a-Service interface for third-party builders.
- ADR-0258 requires SDKs to encode the API versioning model without hidden drift.
- ADR-0173 requires stack ownership; deterministic generation must not depend on opaque SaaS codegen.
- ADR-0263 requires generation runs to emit metrics and audit events so reproducibility failures are observable.
- ADR-0243 requires release and generation gates to be Cedar-authorized.
- ADR-0244 requires sandbox and tenant-specific SDK generation to preserve tenant scoping.
- SDKs are generated for Rust, TypeScript, Swift, Kotlin, Python, and Go.
- The same input spec must produce the same bytes on macOS, Linux, OCI CI, and self-hosted sovereign runners.
- The same input spec must produce the same bytes across repeated CI attempts.
- The same input spec must produce the same bytes when file discovery order differs.
- Generated package lockfiles must not float dependencies.
- Generated examples must not include nondeterministic request ids.
- Generated examples may include fixed ULID fixtures only when the fixture seed is named.
- Generated docs may include a generation date only when it is the API release date from the spec, not wall-clock now.
- The build must detect nondeterminism before publication.
- The build must expose a deterministic diff artifact for reviewers.
- The build must fail closed when the generator cannot prove deterministic output.

## Decision

- We choose `two-run byte-identical code generation` as the release gate for every developer-sdk generated artifact.
- The named pattern is `reproducible builds with deterministic input graph`, following Bazel/Nix-style hermeticity without adopting those systems as product dependencies.
- The generator implementation is a Rust CLI crate named `oya-developer-sdk-codegen-app`.
- Template rendering uses `Tera 1.20.x` with deterministic filters only.
- JSON input normalization uses RFC 8785 JSON Canonicalization Scheme.
- YAML input normalization parses to typed Rust structs and serializes through canonical JSON before template context construction.
- Protobuf input normalization uses `buf.build` image descriptors pinned by digest.
- File discovery sorts by normalized UTF-8 path bytes.
- Map iteration sorts by canonical key bytes before rendering.
- Enum variants sort by declared order from the input contract, not by hash-map order.
- Generated package metadata sorts dependency keys lexicographically.
- Generated documentation sections sort endpoint groups by explicit `x-oyatie-order`, then path.
- Generated examples use fixed fixture seeds named in `codegen_fixture_seed`.
- Generated ULIDs use deterministic seed `sdk-codegen:{artifact_id}:{language}:{api_version}`.
- Generated timestamps are forbidden except `api_released_at`, which is a field in the source spec.
- The generator runs twice in CI in isolated temp directories.
- The CI gate compares byte digests of every output file.
- The CI gate refuses publication on any byte mismatch.
- The CI gate stores the mismatch diff as an audit evidence artifact.
- The generation environment sets `TZ=UTC`, `LC_ALL=C`, and `SOURCE_DATE_EPOCH` to the API release timestamp.
- Dependency versions are pinned per language.
- Rust uses `Cargo.lock` committed for generated examples.
- TypeScript uses `pnpm-lock.yaml` with `frozen-lockfile`.
- Swift uses `Package.resolved`.
- Kotlin uses Gradle dependency locking.
- Python uses `uv.lock`.
- Go uses `go.sum` with proxy checksum verification.
- Public SDK release manifests include `codegen_input_digest`, `codegen_binary_digest`, `template_bundle_digest`, and `output_tree_digest`.
- Public SDK release manifests include `determinism_proof_ref`.
- The p95 generation latency target for a standard OpenAPI contract is 45 seconds per language.
- The p99 generation latency target is 90 seconds per language.
- The byte mismatch budget for release branches is zero.
- The mismatch budget for local sandbox generation is zero by default and may be warning-only only under `--unsafe-preview`.

## Alternatives Considered

### Best-effort deterministic generation

- Pro: easier to implement.
- Pro: fewer CI cycles.
- Pro: local generators can keep wall-clock comments.
- Con: reproducibility is a claim without proof.
- Con: phantom diffs will keep reaching review.
- Con: SDK consumers cannot verify official output independently.
- Con: supply-chain attestations are weaker because output bytes are unstable.
- Tradeoff: less CI cost but persistent trust debt.
- Rejected because public SDK artifacts need evidence, not intent.

### Use OpenAPI Generator as an opaque binary

- Pro: mature multi-language generator ecosystem.
- Pro: broad language support.
- Pro: lower initial implementation cost.
- Con: template and dependency behavior are harder to make deterministic across languages.
- Con: generated idioms may not match Oyatie API semantics.
- Con: codegen bugs would be upstream-dependent.
- Con: ADR-0173 stack ownership is weaker.
- Tradeoff: faster language coverage but less control over reproducibility.
- Rejected as canonical; selected snippets may be studied as compatibility references.

### Language-specific handwritten SDKs

- Pro: idiomatic SDKs in every language.
- Pro: fewer generator bugs.
- Pro: bespoke developer experience.
- Con: API drift across languages is likely.
- Con: every API version change becomes six manual edits.
- Con: reproducibility of generated artifacts no longer applies.
- Con: release cadence slows.
- Tradeoff: better local idiom but worse cross-language contract consistency.
- Rejected for generated core SDKs; handwritten ergonomic wrappers may sit above generated cores.

### Nix-only hermetic build

- Pro: excellent reproducibility.
- Pro: mature content-addressed build model.
- Pro: strong local/CI parity.
- Con: raises contributor onboarding complexity.
- Con: not every downstream SDK consumer uses Nix.
- Con: the problem is codegen determinism, not full workspace build hermeticity.
- Tradeoff: stronger build substrate but broader scope than this ADR.
- Deferred; we implement deterministic generator discipline first.

## Consequences

- Positive: SDK releases become reproducible from public inputs.
- Positive: reviewers stop seeing phantom diffs caused by file ordering or timestamps.
- Positive: generated SDKs can attach strong SLSA provenance.
- Positive: customers can rerun generation and verify tree digests.
- Positive: API versioning changes are visible as semantic spec deltas, not generator noise.
- Negative: CI cost doubles for codegen lanes because every generation runs twice.
- Negative: templates must avoid convenience helpers that inspect wall clock or filesystem order.
- Negative: language-specific package managers require pinned lockfile discipline.
- Negative: contributors must learn the deterministic fixture seed model.
- Neutral: the chosen Rust generator can still emit idiomatic language wrappers.
- Neutral: OpenAPI Generator remains useful for compatibility tests, not as canonical output.
- Follow-up work: implement `SDK-IP-002-deterministic-codegen-runner`.
- Follow-up work: add `codegen-determinism` dashboard panels.
- Follow-up work: add a public reproducibility tutorial in developer-sdk onboarding.
- Follow-up work: register `DeveloperSdkCodegenMismatch` in the audit-event registry.

## Implementation Notes

- Data shape `CodegenRunV1` records one generation execution.
- Field `run_id` is a ULID prefixed by `sdk_codegen_`.
- Field `input_contract_refs` is a sorted array of contract digests.
- Field `template_bundle_digest` is SHA-256 over canonical tar contents.
- Field `generator_binary_digest` is SHA-256 over the release binary.
- Field `language` is one of `rust`, `typescript`, `swift`, `kotlin`, `python`, or `go`.
- Field `source_date_epoch` is an integer Unix timestamp from the API release record.
- Field `fixture_seed` is mandatory for examples.
- Field `output_tree_digest` is SHA-256 Merkle root over path and bytes.
- Field `determinism_passed` is boolean.
- Field `mismatch_paths` is a sorted array and must be empty for publication.
- API endpoint `POST /v1/sdk/codegen/runs` starts a generation run.
- API endpoint `GET /v1/sdk/codegen/runs/{run_id}` returns run metadata.
- API endpoint `GET /v1/sdk/codegen/runs/{run_id}/diff` returns mismatch diff when any exists.
- API endpoint `POST /v1/sdk/codegen/runs/{run_id}/promote` promotes generated output into release packaging.
- API endpoint `GET /v1/sdk/releases/{release_id}/determinism-proof` exposes the public proof artifact.
- Cedar principal is `Oyatie::Principal::Service("developer-sdk.codegen-worker")`.
- Cedar action `developer-sdk.codegen.run` applies to `DeveloperSdk::CodegenRun`.
- Cedar action `developer-sdk.codegen.promote` applies to `DeveloperSdk::SdkRelease`.
- Cedar context field `api_version` must match the source contract.
- Cedar context field `input_digest` must match the admitted contract digest.
- Cedar context field `release_branch` must be true for publication.
- Cedar context field `determinism_passed` must be true for promotion.
- Cedar context field `environment` must not be `local-preview` for promotion.
- Example permit: principal `developer-sdk.codegen-worker`, action `developer-sdk.codegen.promote`, resource `DeveloperSdk::SdkRelease::"sdk_rel_01HY..."`, context `{determinism_passed:true, api_version:"2026-05-18", environment:"ci"}`.
- Example forbid: same principal, action `developer-sdk.codegen.promote`, context `{determinism_passed:false}`.
- Template filter `stable_case` maps names through a versioned case-conversion table.
- Template filter `stable_json` serializes through RFC 8785.
- Template filter `stable_doc_anchor` lowercases ASCII and percent-encodes non-ASCII.
- Template filter `now` is forbidden.
- Template filter `random` is forbidden.
- Template filter `uuid` is forbidden.
- Template filter `ulid_from_seed` is allowed only with explicit seed.
- File mode for generated files is fixed at `0644`.
- Directory mode for generated directories is fixed at `0755`.
- Archive member order is lexicographic by normalized path.
- Archive mtime is `SOURCE_DATE_EPOCH`.
- Archive uid and gid are zero.
- Archive user and group names are empty.
- Generated comments say `Generated from api_version <version>` without wall-clock time.
- OpenTelemetry span name is `developer_sdk.codegen.run`.
- Span attribute `sdk.codegen.language` carries the language.
- Span attribute `sdk.codegen.input_digest` carries the contract digest.
- Metric `oya_developer_sdk_codegen_duration_seconds` records generation runtime.
- Metric `oya_developer_sdk_codegen_mismatch_total` records mismatches by language and path class.
- Metric `oya_developer_sdk_codegen_output_files_total` records output file count by language.
- Dashboard `developer-sdk-codegen-determinism.json` shows mismatch count, runtime, output size, and language split.
- SLO `developer-sdk-codegen-determinism.openslo.yaml` sets mismatch budget zero on release branches.
- SLO `developer-sdk-codegen-runtime.openslo.yaml` sets p95 <= 45 seconds per language.
- Failure mode `unordered_map_iteration` is detected by two-run mismatch.
- Failure mode `wall_clock_timestamp` is detected by stable-grep CI and two-run mismatch.
- Failure mode `locale_dependent_output` is detected by C locale and non-US locale replay.
- Failure mode `package_manager_floating_dependency` is detected by frozen-lockfile checks.
- Failure mode `template_bundle_drift` is detected by template digest mismatch.
- Rollback path is to mark the generated release `blocked`, publish no package, fix template/input, and rerun both generation passes.
- Reproducibility proof stores both run ids, both output tree digests, and the comparison result.

## Verification

- Test `codegen_openapi_same_input_same_bytes` runs OpenAPI generation twice and compares bytes.
- Test `codegen_asyncapi_same_input_same_bytes` runs AsyncAPI generation twice and compares bytes.
- Test `codegen_proto_same_input_same_bytes` runs proto generation twice and compares bytes.
- Test `codegen_rejects_wall_clock_filter` verifies forbidden template filters fail CI.
- Test `codegen_sorts_map_keys` verifies randomized map insertion order produces same output.
- Test `codegen_source_date_epoch_controls_archives` verifies archive mtimes are deterministic.
- Test `codegen_lockfiles_are_frozen` verifies each language package manager uses locked dependencies.
- Test `codegen_fixture_seed_controls_ulids` verifies generated examples are stable.
- Test `codegen_promote_requires_determinism_passed` verifies Cedar blocks failed runs.
- Test `codegen_public_proof_verifies_tree_digest` verifies external verification of release outputs.
- Metric `oya_developer_sdk_codegen_mismatch_total` must remain zero for release branches.
- Metric `oya_developer_sdk_codegen_duration_seconds` must meet p95 <= 45 seconds per language on reference contracts.
- Metric `oya_developer_sdk_codegen_output_files_total` warns on 25 percent output-count drift.
- Dashboard `developer-sdk-codegen-determinism.json` must show one panel per language.
- Dashboard `supply-chain-release-integrity.json` must join determinism proof to signing manifest.
- CI check `sdk-codegen-two-run-byte-compare` runs on every SDK PR.
- CI check `sdk-codegen-forbidden-nondeterminism` scans for wall-clock, random, and filesystem-order helpers.
- CI check `sdk-codegen-lockfile-frozen` validates language package locks.
- CI check `sdk-codegen-public-proof` verifies proof files before release publication.
- CI check `oya-governance-api-versioning --microservice developer-sdk` validates ADR-0258 fields.
- CI check `oya-governance-observability-emission --microservice developer-sdk` validates ADR-0263 telemetry.
- Load test generates all six language SDKs from a 500-endpoint OpenAPI contract and requires p95 <= 45 seconds per language.
- Cross-platform test runs generation on Linux and macOS and compares tree digests.
- Locale test runs generation under `LC_ALL=C`, `ko_KR.UTF-8`, and `de_DE.UTF-8` and compares tree digests.
- Release gate refuses package publication when any mismatch diff exists.
- Audit query verifies `DeveloperSdkCodegenCompleted` event count equals promoted release count.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- RFC 8785: JSON Canonicalization Scheme.
- Reproducible Builds project documentation.
- SLSA v1.0 provenance specification.
- in-toto attestation framework.
- Bazel remote execution and hermetic action guidance.
- Nix derivation reproducibility guidance.
- Tera template engine documentation.
- Buf build image descriptor documentation.
- OpenAPI 3.2.0 specification.
- AsyncAPI 3.1.0 specification.
- Protocol Buffers language guide, proto3.
