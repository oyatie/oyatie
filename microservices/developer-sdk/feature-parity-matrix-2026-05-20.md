# developer-sdk Feature Parity Matrix

- Date: 2026-05-20.
- Wave: 3.
- Batch: 3.2.
- Microservice: `developer-sdk`.
- Deliverable: 2 of 3.
- Counterpart 1: Stainless.
- Counterpart 2: Speakeasy.
- Counterpart 3: Fern.
- Retired comparison set: Stripe SDK, Twilio helper libraries, Auth0 SDK, AWS SDK v3, Apple App Store, VS Code Marketplace, AWS Marketplace, Shopify, Stripe Connect, Salesforce AppExchange.
- Product target: SDK generator and publisher for Oyatie API surfaces.
- Non-target: developer portal runtime, marketplace submission, KYC onboarding, payout ledger, tax package, and generic plugin review.
- Tenant-class note: capability quality is uniform across `demo_trial`, `paid`, and `revenue_share`; the matrix does not create capability ladders.

## Source Anchors

1. Developer-sdk product correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-121`.
2. Chat counterpart correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15754`.
3. Stainless product/docs: `https://www.stainless.com/products/sdks/` and `https://app.stainless.com/docs`.
4. Speakeasy SDK docs: `https://www.speakeasy.com/docs/sdks/core-concepts` and `https://www.speakeasy.com/docs/sdks/customize/runtime/retries`.
5. Fern SDK product page: `https://buildwithfern.com/sdks`.
6. Current Oyatie PRD evidence: `PRD.md:21-25`, `PRD.md:31-38`, `PRD.md:106-123`, `PRD.md:127-157`.
7. Current Oyatie contract evidence: `contracts/openapi/developer-sdk.yaml:147-171`, `contracts/openapi/developer-sdk.yaml:161`, `contracts/asyncapi/developer-sdk-events.yaml:121`, `contracts/proto/developer-sdk.proto:126-133`.
8. Current Oyatie plan evidence: `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20`, `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80`, `sdk-plan.md:15-24`.
9. Current wrong-counterpart evidence: `competitor-parity-matrix.md:15-27`, `performance-bench.md:36-40`, `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1`.
10. Tenant-class retirement migration constraint: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_ladder_2026_05_20.md:10-44`.

## Counterpart 1 — Stainless Capability Surface

1. Stainless is the strongest reference for OpenAPI-driven SDK generation with generated packages presented as production-grade developer artifacts.
2. Stainless source: `https://www.stainless.com/products/sdks/`.
3. Stainless source: `https://app.stainless.com/docs`.
4. Surface: OpenAPI input as the primary contract source.
5. Surface: generated SDKs across multiple mainstream languages.
6. Surface: TypeScript SDK generation.
7. Surface: Python SDK generation.
8. Surface: Go SDK generation.
9. Surface: Java SDK generation.
10. Surface: Kotlin SDK generation.
11. Surface: C# SDK generation.
12. Surface: Swift SDK generation.
13. Surface: Ruby and PHP are shown in Stainless public language lists, but Oyatie's developer-sdk directive rejects those as target outputs.
14. Surface: Terraform provider generation appears in Stainless docs and product navigation, but Oyatie's current target is SDK generation, not provider generation.
15. Surface: CLI generation appears in Stainless docs and product navigation, but the developer-sdk directive says this µservice is not a CLI generator.
16. Surface: SDK Studio and generation debugging workflows.
17. Surface: compile confidence and generated test harness posture.
18. Surface: idiomatic SDK shape rather than raw client stubs.
19. Surface: pagination configuration.
20. Surface: authentication modeling.
21. Surface: retry behavior.
22. Surface: error modeling.
23. Surface: streaming support.
24. Surface: uploads and downloads.
25. Surface: webhook helpers.
26. Surface: generated examples.
27. Surface: generated documentation surfaces.
28. Surface: changelog and versioning workflows.
29. Surface: generated package publication workflows.
30. Surface: customer-controlled OpenAPI configuration.
31. Surface: per-language customization.
32. Surface: SDK release quality checks.
33. Surface: package examples tied to API reference docs.
34. Surface: regenerated SDKs as API definitions evolve.
35. Surface: support for difficult API shapes through configuration rather than handwritten forks.
36. Oyatie parity target: match the generator quality bar, not the exact full Stainless portfolio.
37. Oyatie exclusion: do not adopt Stainless CLI/provider scope unless a separate product decision adds it.
38. Oyatie gap: current contracts do not expose `/sdks/generate`, `/publish`, configurations, docs, or fixture APIs; see `contracts/openapi/developer-sdk.yaml:147-171`.
39. Oyatie gap: current language set is six families, not the ten-family directive; see `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20`.
40. Oyatie gap: current docs over-index on marketplace, KYC, payout, and portal surfaces; see `PRD.md:21-25`.

## Counterpart 2 — Speakeasy Capability Surface

1. Speakeasy is the strongest reference for workflow-driven SDK generation and customization across language targets.
2. Speakeasy source: `https://www.speakeasy.com/docs/sdks/core-concepts`.
3. Speakeasy source: `https://www.speakeasy.com/docs/sdks/customize/runtime/retries`.
4. Surface: SDK generation from API descriptions.
5. Surface: SDK workflow configuration.
6. Surface: TypeScript generation.
7. Surface: Python generation.
8. Surface: Go generation.
9. Surface: Java generation.
10. Surface: C# generation.
11. Surface: PHP generation in public language list, excluded from Oyatie target.
12. Surface: Ruby generation in public language list, excluded from Oyatie target.
13. Surface: Swift generation.
14. Surface: Terraform generation in public language list, outside this µservice target.
15. Surface: package customization knobs.
16. Surface: authentication customization.
17. Surface: pagination customization.
18. Surface: retry customization.
19. Surface: custom retry status codes.
20. Surface: retry connection-error policy.
21. Surface: exponential backoff configuration.
22. Surface: server URL configuration.
23. Surface: error handling customization.
24. Surface: generated SDK usage documentation.
25. Surface: publishing support.
26. Surface: registry workflow integration.
27. Surface: schema linting and overlay-style API transformation concepts.
28. Surface: SDK behavior shaped by contract metadata.
29. Surface: runtime configuration exposed to SDK consumers.
30. Surface: CI-oriented generation workflows.
31. Surface: contract-first maintenance.
32. Surface: per-language generator settings.
33. Surface: source-of-truth driven regeneration.
34. Surface: generated SDKs as installable packages.
35. Surface: generated code quality through configurable rules.
36. Oyatie parity target: match Speakeasy's generator configuration depth for retries, auth, pagination, errors, server selection, and package publication.
37. Oyatie exclusion: do not import non-target language outputs just because Speakeasy supports them.
38. Oyatie gap: current retry/idempotency behavior is described in the directive but not first-class in current contracts.
39. Oyatie gap: current AsyncAPI events publish SDK events but do not model generator configuration lifecycle; see `contracts/asyncapi/developer-sdk-events.yaml:12-40`.
40. Oyatie gap: current migration playbook focuses on migrating from downstream SDKs, not operating a generator platform; see `migration-playbooks/from-stripe-and-twilio-sdks.md:1-4`.

## Counterpart 3 — Fern Capability Surface

1. Fern is the strongest reference for broad multi-language SDK generation plus generated docs and unified API definitions.
2. Fern source: `https://buildwithfern.com/sdks`.
3. Surface: generate SDKs in more than ten languages from OpenAPI.
4. Surface: TypeScript SDK support.
5. Surface: Python SDK support.
6. Surface: Java SDK support.
7. Surface: Go SDK support.
8. Surface: C# SDK support.
9. Surface: PHP SDK support in public matrix, excluded from Oyatie target.
10. Surface: Ruby SDK support in public matrix, excluded from Oyatie target.
11. Surface: Kotlin SDK support.
12. Surface: Swift SDK support.
13. Surface: Rust SDK support in public matrix.
14. Surface: automatic regeneration.
15. Surface: multi-package release support.
16. Surface: self-hosted GitHub or GitLab workflow support.
17. Surface: docs pages with SDK snippets.
18. Surface: unified intermediate representation.
19. Surface: advanced authentication.
20. Surface: pagination.
21. Surface: idempotency.
22. Surface: server-side code generation.
23. Surface: API definition generation.
24. Surface: changelog generation.
25. Surface: in-memory mock server support for several language families.
26. Surface: OAuth2 support across supported families.
27. Surface: auto-pagination support across supported families.
28. Surface: websocket support in language-specific matrices.
29. Surface: SSR-safe client behavior for relevant web runtimes.
30. Surface: error discrimination.
31. Surface: webhook verification.
32. Surface: version compatibility flags.
33. Surface: custom code integration.
34. Surface: stable release signals per language.
35. Surface: generated package distribution.
36. Surface: generated docs tied to SDK snippets.
37. Surface: OpenAPI-first public positioning.
38. Oyatie parity target: match Fern's broad language and generated-doc integration depth for the Oyatie-approved output set.
39. Oyatie exclusion: do not treat server-side generation as owned by developer-sdk unless separately approved.
40. Oyatie gap: current docs include generated SDKs but not unified IR as a product contract.
41. Oyatie gap: current contracts do not model mock server generation or fixture replay.
42. Oyatie gap: current codegen plan does not include Go, Java, C, or C++ despite those being in the developer-sdk directive.
43. Oyatie gap: current docs include generated docs and portal plans, but the owner boundary is blurred with portal runtime ownership.
44. Oyatie gap: current package publication coverage focuses on npm, cargo, NuGet, and PyPI, while the directive also names Maven, Go modules, SPM, vcpkg, Conan, GitHub Releases, and Homebrew; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:48-60`.
45. Oyatie gap: current per-language feature support is not normalized into a matrix with compile, fixture, publish, and docs gates.

## UNION-Coverage Matrix

| # | Capability | Stainless | Speakeasy | Fern | Oyatie current state | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | OpenAPI input | Covered | Covered | Covered | Partial | `contracts/openapi/developer-sdk.yaml:1-3` |
| 2 | AsyncAPI input | Not primary in public source | Not primary in public source | Not primary in public source | Present | `contracts/asyncapi/developer-sdk-events.yaml:1-3` |
| 3 | proto3 input | Not primary in public source | Not primary in public source | Not primary in public source | Present but wrong service surface | `contracts/proto/developer-sdk.proto:1-4`, `:126-133` |
| 4 | Contract ingestion API | Covered | Covered | Covered | Missing as first-class generator API | `contracts/openapi/developer-sdk.yaml:147-171` |
| 5 | Generate SDK run API | Covered | Covered | Covered | Missing canonical `/sdks/generate` | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:92-100` |
| 6 | Generation configuration API | Covered | Covered | Covered | Missing | `contracts/openapi/developer-sdk.yaml:147-171` |
| 7 | Publish SDK API | Covered | Covered | Covered | Missing canonical publish surface | `contracts/openapi/developer-sdk.yaml:147-171` |
| 8 | SDK version API | Covered | Covered | Covered | Partial through artifacts, not lifecycle | `contracts/openapi/developer-sdk.yaml:147-171` |
| 9 | Generated docs API | Covered | Covered | Covered | Blurred with portal ownership | `implementation-plans/IP-008-dev-portal-backstage-extension.md` |
| 10 | Test fixture API | Covered | Covered | Covered | Missing | `contracts/proto/developer-sdk.proto:126-133` |
| 11 | TypeScript output | Covered | Covered | Covered | Present | `PRD.md:33`, `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` |
| 12 | Python output | Covered | Covered | Covered | Present | `contracts/openapi/developer-sdk.yaml:161` |
| 13 | Go output | Covered | Covered | Covered | Inconsistent | `ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:33`, `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` |
| 14 | Java output | Covered | Covered | Covered | Missing | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 15 | Kotlin output | Covered | Partial/public matrix dependent | Covered | Present | `contracts/openapi/developer-sdk.yaml:161` |
| 16 | Swift output | Covered | Covered | Covered | Present | `contracts/asyncapi/developer-sdk-events.yaml:121` |
| 17 | Rust output | Not a primary public Stainless list item from observed page | Not primary from observed page | Covered | Present | `PRD.md:33` |
| 18 | C#/.NET output | Covered | Covered | Covered | Present | `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` |
| 19 | C output | Not public primary | Not public primary | Not public primary | Missing but required by directive | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 20 | C++ output | Not public primary | Not public primary | Not public primary | Missing but required by directive | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 21 | Ruby output | Covered by counterparts | Covered by counterparts | Covered by counterparts | Should remain excluded | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 22 | PHP output | Covered by counterparts | Covered by counterparts | Covered by counterparts | Should remain excluded | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 23 | Elixir output | Not union-critical from observed public sources | Not union-critical from observed public sources | Not union-critical from observed public sources | Should remain excluded | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` |
| 24 | Idiomatic naming | Covered | Covered | Covered | Not contractual | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:34-47` |
| 25 | Type mapping | Covered | Covered | Covered | Not contractual | `contracts/openapi/developer-sdk.yaml:147-171` |
| 26 | Pagination helpers | Covered | Covered | Covered | Missing as generator feature | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:34-47` |
| 27 | Retry helpers | Covered | Covered | Covered | Missing as generator feature | Speakeasy retry source, `feedback_developer_sdk...:34-47` |
| 28 | Streaming helpers | Covered | Partial | Partial | Missing as generator feature | `feedback_developer_sdk...:34-47` |
| 29 | Idempotency helpers | Covered | Partial | Covered | Missing as generator feature | `feedback_developer_sdk...:34-47` |
| 30 | Timeout configuration | Covered | Covered | Covered | Missing as generator feature | `feedback_developer_sdk...:34-47` |
| 31 | Telemetry hooks | Covered | Covered | Partial | Missing as generator feature | `feedback_developer_sdk...:34-47` |
| 32 | Error taxonomy | Covered | Covered | Covered | Missing as generator feature | `feedback_developer_sdk...:34-47` |
| 33 | Mocking support | Partial | Partial | Covered | Missing | Fern source and `feedback_developer_sdk...:34-47` |
| 34 | Multipart upload support | Covered | Partial | Partial | Missing | `feedback_developer_sdk...:34-47` |
| 35 | Serialization edge cases | Covered | Covered | Covered | Missing as explicit gate | `feedback_developer_sdk...:34-47` |
| 36 | Webhook verification helpers | Covered | Partial | Covered | Not generator-first | `contracts/asyncapi/developer-sdk-events.yaml:12-40` |
| 37 | Generated examples | Covered | Covered | Covered | Partial through tutorials | `tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:23-43` |
| 38 | Generated API snippets | Covered | Covered | Covered | Blurred with portal | `implementation-plans/IP-008-dev-portal-backstage-extension.md` |
| 39 | Generated docs publication | Covered | Covered | Covered | Blurred with portal | `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` |
| 40 | Changelog generation | Covered | Partial | Covered | Missing | Fern source, Stainless source |
| 41 | Semantic versioning policy | Covered | Covered | Covered | Partial | `feedback_developer_sdk...:62-68` |
| 42 | API diff classification | Covered | Covered | Covered | Missing | current contracts lack diff API |
| 43 | Breaking-change gate | Covered | Covered | Covered | Missing as generator gate | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` |
| 44 | Two-run determinism gate | Partial | Partial | Partial | Present in ADR | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80` |
| 45 | Reference fixtures | Covered | Covered | Covered | Partial in ADR, not contract | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` |
| 46 | Compile fixtures | Covered | Covered | Covered | Partial in ADR, not OS matrix | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` |
| 47 | Runtime behavior fixtures | Covered | Covered | Covered | Missing | current contract surface |
| 48 | Package publication proof | Covered | Covered | Covered | Partial | `implementation-plans/IP-012-package-registry-vendored.md:20` |
| 49 | npm publication | Covered | Covered | Covered | Present | `feedback_developer_sdk...:48-60` and current registry plans |
| 50 | PyPI publication | Covered | Covered | Covered | Present | `feedback_developer_sdk...:48-60` and current registry plans |
| 51 | Maven publication | Covered | Covered | Covered | Missing | `feedback_developer_sdk...:48-60` |
| 52 | crates.io publication | Partial | Partial | Partial | Present in current packaging ideas | `feedback_developer_sdk...:48-60` |
| 53 | NuGet publication | Covered | Covered | Covered | Present | `feedback_developer_sdk...:48-60` |
| 54 | Go modules publication | Covered | Covered | Covered | Missing or inconsistent | `feedback_developer_sdk...:48-60` |
| 55 | Swift Package Manager publication | Covered | Covered | Covered | Missing as explicit channel | `feedback_developer_sdk...:48-60` |
| 56 | vcpkg publication | Not counterpart core | Not counterpart core | Not counterpart core | Missing but required by directive | `feedback_developer_sdk...:48-60` |
| 57 | Conan publication | Not counterpart core | Not counterpart core | Not counterpart core | Missing but required by directive | `feedback_developer_sdk...:48-60` |
| 58 | GitHub Releases | Covered | Covered | Covered | Missing as explicit channel | `feedback_developer_sdk...:48-60` |
| 59 | Homebrew release | Partial | Partial | Partial | Missing but required by directive | `feedback_developer_sdk...:48-60` |
| 60 | Registry signing | Covered | Covered | Partial | Partial | `tutorials/...:147-164`, `implementation-plans/IP-012...:20` |
| 61 | Package provenance | Covered | Covered | Covered | Partial | `evidence-emission.md` |
| 62 | SBOM per generated artifact | Covered | Covered | Partial | Missing as explicit generator contract | current docs |
| 63 | Reproducible generated artifact hash | Covered | Covered | Covered | Partial in ADR | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80` |
| 64 | Generator run audit event | Covered | Covered | Covered | Partial | `contracts/asyncapi/developer-sdk-events.yaml:12-40` |
| 65 | Failure event taxonomy | Covered | Covered | Covered | Partial | `failure-modes.md:15-73` |
| 66 | Spec validation error API | Covered | Covered | Covered | Missing | current OpenAPI path set |
| 67 | Generator config validation | Covered | Covered | Covered | Missing | current OpenAPI path set |
| 68 | Generator template versioning | Covered | Covered | Covered | Missing | `sdk-plan.md:15-24` |
| 69 | Language feature matrix | Covered | Covered | Covered | Missing | current docs scatter language claims |
| 70 | Per-language quality flags | Covered | Covered | Covered | Missing | current docs |
| 71 | OAuth helper generation | Covered | Covered | Covered | Missing as explicit SDK capability | Fern source |
| 72 | API key helper generation | Covered | Covered | Covered | Partial through signing docs but wrong owner | `decisions/ADR-SDK-0001-ed25519-signing-keys-via-openbao-transit-engine-only;-privat.md:48-75` |
| 73 | Bearer token helper generation | Covered | Covered | Covered | Missing as explicit SDK capability | current contracts |
| 74 | Multi-server URL support | Covered | Covered | Covered | Missing | Speakeasy source |
| 75 | Environment selection | Covered | Covered | Covered | Partial through sandbox docs but wrong owner | `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:48` |
| 76 | Generated SDK retries | Covered | Covered | Covered | Missing | Speakeasy retry source |
| 77 | Generated SDK rate-limit handling | Covered | Covered | Covered | Missing | current contracts |
| 78 | Generated SDK idempotency keys | Covered | Partial | Covered | Missing | directive |
| 79 | Generated SDK pagination iterators | Covered | Covered | Covered | Missing | directive |
| 80 | Generated SDK streaming abstractions | Covered | Partial | Partial | Missing | directive |
| 81 | Generated SDK upload abstractions | Covered | Partial | Partial | Missing | directive |
| 82 | Generated SDK download abstractions | Covered | Partial | Partial | Missing | directive |
| 83 | Generated SDK webhook signing | Covered | Partial | Covered | Missing as SDK artifact | directive |
| 84 | Generated SDK mock transport | Partial | Partial | Covered | Missing | Fern source |
| 85 | Generated SDK in-memory server | Partial | Partial | Covered | Missing | Fern source |
| 86 | Generated SDK SSR-safe mode | Partial | Partial | Covered | Missing where web runtime applies | Fern source |
| 87 | Generated SDK websocket support | Partial | Partial | Covered | Missing | Fern source |
| 88 | Custom code insertion | Covered | Covered | Covered | Missing | Fern source |
| 89 | Generated client examples by endpoint | Covered | Covered | Covered | Partial through tutorial | `tutorials/...:23-43` |
| 90 | Endpoint-level snippet generation | Covered | Covered | Covered | Missing as contract | current docs |
| 91 | Generated docs diff | Covered | Partial | Covered | Missing | current docs |
| 92 | SDK release notes | Covered | Covered | Covered | Missing | current docs |
| 93 | Changelog from API diff | Covered | Partial | Covered | Missing | Fern source |
| 94 | Monorepo output mode | Covered | Covered | Covered | Missing | current docs |
| 95 | Multi-package output mode | Covered | Covered | Covered | Missing | Fern source |
| 96 | Self-hosted generation workflow | Covered | Covered | Covered | Missing as six-context contract | Fern source |
| 97 | Cloud generation workflow | Covered | Covered | Covered | Missing as context contract | current docs |
| 98 | On-prem generation workflow | Partial | Partial | Covered | Missing as context contract | `docs/decisions/ADR-0328...:1882-1895` |
| 99 | Colo generation workflow | Not explicit | Not explicit | Self-hosted adjacent | Missing as context contract | `docs/decisions/ADR-0328...:1932-1945` |
| 100 | OCI guest generation workflow | Not explicit | Not explicit | Self-hosted adjacent | Missing | no `iac/oci-guest/` |
| 101 | OCI Always Free profile generation limits | Not counterpart-specific | Not counterpart-specific | Not counterpart-specific | Missing | no `iac/oci-guest/always-free/` |
| 102 | API spec size limits | Covered | Covered | Covered | Missing | current docs |
| 103 | Endpoint-count limits | Covered | Covered | Covered | Missing | current docs |
| 104 | Schema-count limits | Covered | Covered | Covered | Missing | current docs |
| 105 | Language fanout limits | Covered | Covered | Covered | Partial | `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` |
| 106 | Parallel generation workers | Covered | Covered | Covered | Missing as target | current docs |
| 107 | Queue fairness | Covered | Covered | Covered | Missing | current docs |
| 108 | Tenant usage caps | Not counterpart-visible | Not counterpart-visible | Not counterpart-visible | Missing target tenant classes | §3.4.C in coherence audit |
| 109 | Contractual SLO scale | Covered | Covered | Covered | Not generator-specific | `performance-bench.md:19-28` |
| 110 | Usage metering | Covered | Covered | Covered | Blurred with payout/cost | `cost-budget.md:17-24` |
| 111 | Billing integration | Covered | Covered | Covered | Blurred with payout/tax | `PRD.md:21-25` |
| 112 | Revenue-share tenant model | Not counterpart core | Not counterpart core | Not counterpart core | Fragment only as payout kind | `contracts/openapi/developer-sdk.yaml:323` |
| 113 | Per-seat paid model | Not counterpart core | Not counterpart core | Not counterpart core | Missing as tenant_class | `cost-budget.md:19` |
| 114 | Demo-trial model | Not counterpart core | Not counterpart core | Not counterpart core | Missing | no `demo_trial` string found |
| 115 | Compliance evidence pack | Covered | Covered | Covered | Blurred with non-generator compliance | `compliance.md:15-29` |
| 116 | DPIA for generator artifacts | Partial | Partial | Partial | Blurred with payout PII | `dpia.md:21-25` |
| 117 | Credential leak scanning | Covered | Covered | Covered | Partial through signing/security docs | `threat-model.md`, `ADR-SDK-0004...` |
| 118 | Generated secret redaction | Covered | Covered | Covered | Missing as explicit gate | current docs |
| 119 | Dependency vulnerability scan | Covered | Covered | Covered | Partial through Trivy failure mode | `failure-modes.md:15-73` |
| 120 | Generated SDK license metadata | Covered | Covered | Covered | Missing | current docs |
| 121 | Generated package README | Covered | Covered | Covered | Partial through docs plan | `IP-007...` |
| 122 | Generated package examples | Covered | Covered | Covered | Partial | `tutorials/...:23-43` |
| 123 | Generated package tests | Covered | Covered | Covered | Partial in ADR | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` |
| 124 | Generated package compile check | Covered | Covered | Covered | Partial in ADR | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` |
| 125 | Generated package publish dry run | Covered | Covered | Covered | Missing | current docs |
| 126 | Generated package rollback | Covered | Covered | Covered | Missing | current docs |
| 127 | Generated package deprecation | Covered | Covered | Covered | Partial | `deprecation-plan.md` |
| 128 | Generated package support window | Covered | Covered | Covered | Blurred with retired tier language | `faqs/sdk-engineer-faq.md:83-86` |
| 129 | Customer template override | Covered | Covered | Covered | Missing | current docs |
| 130 | Oyatie extension metadata | Covered | Covered | Covered | Missing | current docs |
| 131 | Cross-service API ingestion | Covered | Covered | Covered | Partial | `contracts/openapi/oya-ecosystem.yaml` |
| 132 | Marketplace/plugin ownership | Not generator target | Not generator target | Not generator target | Wrong owner in current docs | `PRD.md:21-25` |
| 133 | KYC onboarding ownership | Not generator target | Not generator target | Not generator target | Wrong owner in current docs | `IP-002...` |
| 134 | Payout ledger ownership | Not generator target | Not generator target | Not generator target | Wrong owner in current docs | `IP-010...` |
| 135 | Tax package ownership | Not generator target | Not generator target | Not generator target | Wrong owner in current docs | `IP-010...` |
| 136 | Developer portal runtime | Not primary generator target | Not primary generator target | Docs adjacent | Wrong owner if runtime | `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` |
| 137 | Signing-key custody | Not generator target | Not generator target | Not generator target | Wrong owner unless artifact signing | `ADR-SDK-0004...` |
| 138 | Sandbox provisioning | Not generator target | Not generator target | Not generator target | Wrong owner unless fixture sandbox | `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:48` |
| 139 | Plugin trust scoring | Not generator target | Not generator target | Not generator target | Wrong owner | `ADR-SDK-0006...` |
| 140 | Backstage portal deployment | Not generator target | Not generator target | Not generator target | Wrong owner and language-risk | `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` |
| 141 | Per-context OpenTofu modules | Not counterpart-visible | Not counterpart-visible | Self-hosted adjacent | Missing | no canonical context dirs |
| 142 | Supported OS manifest | Not counterpart-visible | Not counterpart-visible | Not counterpart-visible | Missing | no `supported-oses.json` |
| 143 | Rust generator runtime | Not counterpart-visible | Not counterpart-visible | Not counterpart-visible | Required but not expressed | `docs/decisions/ADR-0328...:3047-3067` |
| 144 | Generated-output provenance | Covered | Covered | Covered | Partial | `evidence-emission.md` |
| 145 | Uniform quality across tenant classes | Not counterpart-visible | Not counterpart-visible | Not counterpart-visible | Missing | tenant-class retirement migration directive |
| 146 | No capability-ladder scaffolding | Not counterpart-visible | Not counterpart-visible | Not counterpart-visible | Violated by existing docs | `capability-ladders/tier-matrix.md:11` |
| 147 | Product purpose statement | Covered | Covered | Covered | Incorrectly broad | `PRD.md:21-25` |
| 148 | Counterpart list | Covered | Covered | Covered | Incorrect in existing parity docs | `competitor-parity-matrix.md:15-27` |
| 149 | Generator performance model | Covered | Covered | Covered | Partial and wrong baseline | `performance-bench.md:19-40` |
| 150 | Generator incident model | Covered | Covered | Covered | Partial and wrong emphasis | `failure-modes.md:15-73` |

## Family Summary

1. Input-contract family: Oyatie has OpenAPI, AsyncAPI, and proto files, but the service methods point at developer operations instead of generator runs.
2. Input-contract family status: partial.
3. Generator-run family: canonical generate, publish, version, configuration, docs, and fixture APIs are missing.
4. Generator-run family status: missing.
5. Output-language family: Oyatie has six-family documentation, but the canonical directive requires ten outputs.
6. Output-language family status: partial with P1 inconsistency.
7. Language-boundary family: generated SDK outputs are allowed if provenance-scoped; portal/runtime language claims remain risky.
8. Language-boundary family status: partial with P1 documentation risk.
9. Quality-behavior family: idiomatic naming, type mapping, pagination, retries, streaming, idempotency, timeouts, telemetry, error taxonomy, mocks, multipart, and serialization are named in the directive but not first-class in contracts.
10. Quality-behavior family status: missing from API contract.
11. Publication family: current docs cover several registries, but Maven, Go modules, SPM, vcpkg, Conan, GitHub Releases, and Homebrew are not coherently represented.
12. Publication family status: partial.
13. Versioning family: ADR-SDK-0002 has useful determinism and verification ideas, but semantic release channels and changelog generation are not complete.
14. Versioning family status: partial.
15. Fixture family: compile and determinism tests are discussed, but cross-language runtime fixtures and fixture APIs are missing.
16. Fixture family status: partial.
17. Documentation family: docs and tutorials exist, but runtime portal ownership conflicts with canonical product scope.
18. Documentation family status: partial and over-owned.
19. Deployment-context family: all six contexts are required, but none have canonical OpenTofu directories.
20. Deployment-context family status: missing.
21. OCI Always Free profile family: required profile directory is absent.
22. OCI Always Free profile family status: missing.
23. OS-support family: required manifest is absent.
24. OS-support family status: missing.
25. Tenant-class family: no `demo_trial`, `paid`, or first-class `revenue_share` tenant_class contract exists.
26. Tenant-class family status: missing.
27. Tenant-class retirement migration family: existing docs still include direct retired capability-ladder names and older tier metadata.
28. Tenant-class retirement migration family status: active retirement candidate.
29. Operational-evidence family: SLO, capacity, runbook, and failure docs exist, but most are not generator-first.
30. Operational-evidence family status: partial.
31. Security family: credential, signing, OpenBao, and policy docs exist, but some may belong to other owners.
32. Security family status: needs boundary split.
33. Data-protection family: DPIA exists but emphasizes developer PII and payout data rather than generated artifact risk.
34. Data-protection family status: partial.
35. Handoff family: cross-microservice handoff docs exist, but several paths and ownership claims are stale.
36. Handoff family status: partial with correction required.
37. Counterpart-family summary: Stainless sets the OpenAPI-driven quality and packaging bar.
38. Counterpart-family summary: Speakeasy sets the workflow and customization bar.
39. Counterpart-family summary: Fern sets the broad-language, generated-doc, and multi-package bar.
40. Oyatie-family summary: the service currently contains useful seeds but needs a product-boundary reset before it can claim parity.

## Headline Gap Analysis

1. Gap 1: wrong product center.
2. Gap 1 evidence: `PRD.md:21-25` and `PHASE-01-DEVELOPER-SDK-SUBSTRATE.md:37` center onboarding, signing, sandbox, marketplace, payout, and tax.
3. Gap 1 counterpart impact: Stainless, Speakeasy, and Fern are generator platforms, so the current product center misdirects parity work.
4. Gap 1 remediation: rewrite PRD purpose around generator lifecycle and convert non-generator features to handoffs.
5. Gap 2: missing canonical generator API.
6. Gap 2 evidence: `contracts/openapi/developer-sdk.yaml:147-171` lacks generate/publish/configuration/docs/fixtures APIs.
7. Gap 2 counterpart impact: all three counterparts expose generation configuration and artifact lifecycle as their core product.
8. Gap 2 remediation: add a generator-first OpenAPI surface and align proto/AsyncAPI events to that lifecycle.
9. Gap 3: incomplete language set.
10. Gap 3 evidence: `contracts/openapi/developer-sdk.yaml:161`, `contracts/asyncapi/developer-sdk-events.yaml:121`, and `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` show six output families.
11. Gap 3 counterpart impact: Fern and the developer-sdk directive require broader language coverage; Go and Java are especially basic parity omissions in current active plans.
12. Gap 3 remediation: normalize to the directive's ten families and define staged delivery gates without feature tiers.
13. Gap 4: missing behavior contract for generated SDK quality.
14. Gap 4 evidence: the directive names pagination, retries, streaming, idempotency, timeouts, telemetry, error taxonomy, mocks, multipart, and serialization, but current contracts do not model these.
15. Gap 4 counterpart impact: Speakeasy and Fern publicly emphasize these customization and behavior features.
16. Gap 4 remediation: add per-language capability schema and fixture gates for each behavior.
17. Gap 5: publication matrix incomplete.
18. Gap 5 evidence: directive publication channels at `feedback_developer_sdk...:48-60` exceed current package registry plans.
19. Gap 5 counterpart impact: counterpart products treat package release as a first-class output, not a sidecar registry deployment.
20. Gap 5 remediation: define per-channel release tasks, credentials, dry-run, signing, rollback, and provenance.
21. Gap 6: generated docs conflated with portal runtime.
22. Gap 6 evidence: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` uses Backstage-style portal architecture.
23. Gap 6 counterpart impact: generator products produce SDK docs and snippets, but this does not require developer-sdk to own a portal runtime.
24. Gap 6 remediation: own generated documentation artifacts and hand off UI runtime ownership.
25. Gap 7: wrong counterpart history still present.
26. Gap 7 evidence: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1` and `competitor-parity-matrix.md:15-27`.
27. Gap 7 counterpart impact: comparing downstream SDKs or marketplaces hides generator-platform gaps.
28. Gap 7 remediation: retire or replace the old comparison docs.
29. Gap 8: no OpenTofu context surface.
30. Gap 8 evidence: no canonical context directories under `iac/`.
31. Gap 8 counterpart impact: Oyatie-specific deployment doctrine cannot be validated by counterpart comparison alone.
32. Gap 8 remediation: create context-specific OpenTofu modules through cloud-iac.
33. Gap 9: no OS manifest.
34. Gap 9 evidence: no `supported-oses.json`.
35. Gap 9 counterpart impact: generated SDK fixture platforms and generator workers cannot be evaluated.
36. Gap 9 remediation: define runtime and generated-output fixture OS rows.
37. Gap 10: tenant-class semantics absent.
38. Gap 10 evidence: only sandbox tenant class and payout-kind revenue-share fragments exist.
39. Gap 10 counterpart impact: tenant class is Oyatie-specific, but required for current batch doctrine.
40. Gap 10 remediation: add tenant_class contract after the shared doctrine is finalized.
41. Gap 11: tenant-class retirement migration not complete.
42. Gap 11 evidence: `capability-ladders/tier-matrix.md:11` and other cited lines in the coherence audit.
43. Gap 11 counterpart impact: tier language risks recreating a capability ladder that the current directive retired.
44. Gap 11 remediation: replace with tenant-class, billing, usage cap, and deployment-context overlays.
45. Gap 12: no generator-specific incident model.
46. Gap 12 evidence: `failure-modes.md:15-73` emphasizes non-generator systems.
47. Gap 12 counterpart impact: generator customers care about bad generated code, broken packages, incorrect auth, and unsafe retries.
48. Gap 12 remediation: add generator-specific failure modes and runbooks.
49. Gap 13: no fixture API.
50. Gap 13 evidence: proto and OpenAPI surfaces do not expose fixture lifecycle.
51. Gap 13 counterpart impact: compile and runtime fixtures are required to claim generated SDK quality.
52. Gap 13 remediation: make fixture sets first-class resources.
53. Gap 14: no package rollback model.
54. Gap 14 evidence: registry plans exist but rollback is not modeled.
55. Gap 14 counterpart impact: package publication mistakes are customer-visible and need reversal paths.
56. Gap 14 remediation: define yank/deprecate/rollback semantics per registry.
57. Gap 15: current generated SDK examples are too narrow.
58. Gap 15 evidence: `tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:23-43` focuses on three families.
59. Gap 15 counterpart impact: generated examples must prove all mandatory families and behavior features.
60. Gap 15 remediation: make examples generated artifacts derived from fixtures, not handwritten tutorial drift.

## Additive Surface for Oyatie

1. Additive surface 1: generator-worker runtime in Rust with deterministic generation and signed output evidence.
2. Additive surface 2: OpenAPI 3.2.0 ingestion with validation error taxonomy.
3. Additive surface 3: AsyncAPI 3.1.0 ingestion for event-driven SDK surfaces.
4. Additive surface 4: proto3 ingestion for gRPC and message-schema SDK surfaces.
5. Additive surface 5: canonical `/sdks/generate` API with asynchronous run state.
6. Additive surface 6: `/sdks/configurations` API for per-language generator settings.
7. Additive surface 7: `/sdks/publish` API for registry release orchestration.
8. Additive surface 8: `/sdks/versions` API for semver, diff, and compatibility records.
9. Additive surface 9: `/sdks/docs` API for generated docs and snippets as artifacts.
10. Additive surface 10: `/sdks/fixtures` API for generated compile and runtime fixtures.
11. Additive surface 11: TypeScript SDK output with generated tests, docs, examples, package release, and provenance.
12. Additive surface 12: Python SDK output with generated tests, docs, examples, package release, and provenance.
13. Additive surface 13: Go SDK output with generated tests, docs, examples, module release, and provenance.
14. Additive surface 14: Java SDK output with generated tests, docs, examples, Maven release, and provenance.
15. Additive surface 15: Kotlin SDK output with generated tests, docs, examples, Maven release, and provenance.
16. Additive surface 16: Swift SDK output with generated tests, docs, examples, SPM release, and provenance.
17. Additive surface 17: Rust SDK output with generated tests, docs, examples, crate release, and provenance.
18. Additive surface 18: C#/.NET SDK output with generated tests, docs, examples, NuGet release, and provenance.
19. Additive surface 19: C SDK output with generated tests, docs, examples, vcpkg/Conan release, and provenance.
20. Additive surface 20: C++ SDK output with generated tests, docs, examples, vcpkg/Conan release, and provenance.
21. Additive surface 21: no non-target language expansion unless a future directive changes the output set.
22. Additive surface 22: generated pagination fixtures per language.
23. Additive surface 23: generated retry fixtures per language.
24. Additive surface 24: generated idempotency fixtures per language.
25. Additive surface 25: generated streaming fixtures per language.
26. Additive surface 26: generated timeout fixtures per language.
27. Additive surface 27: generated telemetry fixtures per language.
28. Additive surface 28: generated error taxonomy fixtures per language.
29. Additive surface 29: generated mock transport fixtures per language.
30. Additive surface 30: generated multipart fixtures per language.
31. Additive surface 31: generated serialization edge-case fixtures per language.
32. Additive surface 32: generated webhook verification fixtures where protocol scope requires them.
33. Additive surface 33: contract diff classification that gates generator releases.
34. Additive surface 34: changelog generation from API diff.
35. Additive surface 35: package signing and provenance bundle per generated output.
36. Additive surface 36: rollback/deprecation action per registry.
37. Additive surface 37: package publication dry-run per registry.
38. Additive surface 38: package visibility and artifact integrity check per registry.
39. Additive surface 39: OCI Always Free profile with explicit demo-trial usage caps and worker sizing.
40. Additive surface 40: paid tenant scale controls tied to contractual SLO and billing.
41. Additive surface 41: revenue-share tenant semantics according to the current prompt, or paid billing-component semantics if the later memory is promoted.
42. Additive surface 42: all six deployment contexts represented through OpenTofu modules or explicit N/A records.
43. Additive surface 43: `supported-oses.json` that separates generator runtime support from generated SDK fixture support.
44. Additive surface 44: Rust-only generator runtime and command tooling.
45. Additive surface 45: generated-output exception records for every emitted SDK language.
46. Additive surface 46: explicit non-ownership handoffs for KYC, payout, tax, marketplace, plugin review, and portal runtime.
47. Additive surface 47: generator-specific SLOs for API latency, queue latency, generation duration, compile pass rate, publish latency, and rollback latency.
48. Additive surface 48: generator-specific failure modes for unsafe generated auth, incorrect retry, bad pagination, broken package publish, leaked credential, stale docs, and fixture failure.
49. Additive surface 49: generator-specific capacity model based on spec size, endpoint count, schema count, output language fanout, fixture count, and registry fanout.
50. Additive surface 50: replacement parity docs that cite Stainless, Speakeasy, and Fern and do not resurrect capability ladders.
