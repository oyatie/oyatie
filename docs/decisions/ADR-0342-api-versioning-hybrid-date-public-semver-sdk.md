---
id: ADR-0342
title: API versioning HYBRID model — date-based (YYYY-MM-DD) for public APIs (OpenAPI 3.2.0 endpoints, AsyncAPI 3.1.0 channels, proto3 services) + semver (major.minor.patch) for SDK packages (TS / Python / Go / Java / Kotlin / Swift / Rust / .NET-C# / C / C++); N=3 supported public versions; ≥180-day post-deprecation window; per-tenant version pinning
status: Proposed
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - axis-developer-sdk
  - axis-api-gateway
  - ops-platform
  - council-security
owners:
  - council-architecture
  - axis-developer-sdk
  - axis-api-gateway
  - ops-platform
  - council-security
supersedes: []
superseded_by: []
amends:
  - ADR-0145-inter-microservice-communication-reform.md (the 3-invariant inter-µservice gRPC contract gains the Oyatie-Version protobuf field + Oyatie-Version metadata header as canonical version carriers; the contract surface of "direct gRPC over HTTP/3" is amended to declare the version-carrier triplet — header + URL prefix + protobuf field — as binding on every public-facing endpoint)
  - ADR-0211-in-house-tech-stack-preference.md (the SDK substrate-class allow-list gains the 10-language semver discipline; SDK release engineering for the 10 canonical languages is the canonical reuse vehicle for public-API consumption; hand-rolled HTTP/gRPC client substrate inside developer-sdk-generated SDKs is non-canonical going forward)
  - ADR-0212-buildability-doctrine.md (per-µservice manifest `api_versioning` block becomes the canonical declaration of the µservice's date-version surface area, deprecation calendar, and tenant-version-pinning posture)
  - ADR-0216-deployment-context-iac-layout.md (the api-gateway µservice's per-context iac/<context>/ wrappers gain version-routing primitive invocations from the shared library; per-version URL-prefix routes are materialized by api-gateway not by per-µservice plumbing)
  - ADR-0218-opentofu-not-terraform.md (no change to OpenTofu-only rule; informational citation; version-routing OpenTofu primitives are authored under ADR-0339 shared library shape)
  - ADR-0244-tenant-as-universal-scoping-primitive.md (every tenant's effective Oyatie-Version is a tenant-scoped resolved value; tenant version pinning lives on the tenant manifest, audit-chain emits tenant_id + resolved_version on every privileged request)
  - ADR-0263-observability-emission-contract.md (every audit-chain emit + metric + trace gains the `oyatie_version` label; cardinality bounded at N=3 supported versions × ~30 days deprecation overlap)
  - ADR-0316-tier-system-canonical-bronze-silver-gold-platinum.md (RETIRED; this ADR's API-versioning posture does not rely on the retired capability-tier ladder; tenant-class governs feature ceilings per ADR-0330 and is orthogonal to API version selection)
related:
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0108-deprecation-and-sunset-discipline.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0216-deployment-context-iac-layout.md
  - ADR-0218-opentofu-not-terraform.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0316-tier-system-canonical-bronze-silver-gold-platinum.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-3.md
  - ADR-0339-shared-iac-module-library.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
  - ADR-0341-cellular-promotion-gates-explicit-per-tier.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/api-gateway.json
  - /specs/microservices/developer-sdk.json
  - /specs/tenant-manifest-schema.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_six_candidate_adrs_2026_05_21
  - feedback_developer_sdk_stainless_generator_2026_05_20
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_bominal_inheritance_precedence
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_drift_too_big_2026_05_20
  - feedback_api_first
companion_docs:
  - docs/standards/api-versioning.md
  - docs/standards/sdk-release-engineering.md
  - docs/standards/deprecation-policy.md
  - microservices/api-gateway/ARCHITECTURE.md
  - microservices/api-gateway/manifest.json
  - microservices/developer-sdk/ARCHITECTURE.md
  - microservices/developer-sdk/manifest.json
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-version-router-and-sdk-pipeline-land
enforced_by:
  - oya-check-public-api-date-version (new lane; advisory until crate lands; planned to refuse contract authoring that omits a YYYY-MM-DD Oyatie-Version anchor on every public OpenAPI 3.2.0 path / AsyncAPI 3.1.0 channel / proto3 service)
  - oya-check-public-api-version-triplet (new lane; advisory until crate lands; planned to refuse public-facing handlers that do not honor all three carriers — header `Oyatie-Version`, URL prefix `/v/<YYYY-MM-DD>/...`, protobuf field `oyatie_version`)
  - oya-check-public-api-supported-window (new lane; advisory until crate lands; planned to refuse contract drops below N=3 supported versions and refuses removal earlier than 180 days post-deprecation)
  - oya-check-public-api-sunset-adr (new lane; advisory until crate lands; planned to refuse a tenant-affecting breaking change without a paired sunset-class ADR + audit-chain `api.version.sunset` event emission)
  - oya-check-sdk-semver-bump (new lane; advisory until crate lands; planned to refuse SDK releases that do not honor the major-on-breaking / minor-on-additive / patch-on-fix rule; refuses SDK packages that fail to pin a date-version under-the-hood)
  - oya-check-sdk-language-coverage (new lane; advisory until crate lands; planned to refuse SDK release trains that omit any of the 10 canonical languages without an exception ADR)
  - oya-check-tenant-version-pinning (new lane; advisory until crate lands; planned to refuse tenant manifests that omit `api_version_pinning` block; refuses pinning to an unsupported / dropped version)
  - oya-governance-version-routing-canonical-carriers (refuses any non-canonical version carrier — e.g., a `?api_version=` query param, a custom `X-API-Version` header, a separate sub-domain `v20260521.api.oyatie.dev`)
purpose: >
  Establish a HYBRID API versioning model that separates the public-API
  surface (OpenAPI 3.2.0 endpoints, AsyncAPI 3.1.0 channels, proto3
  services exposed externally per `tools/hooks/_canonical-primitives.md`
  contracts section) from the SDK distribution surface (developer-sdk
  Stainless-class generator producing 10 idiomatic SDKs per
  `feedback_developer_sdk_stainless_generator_2026_05_20`). The public-API
  surface uses date-based versions (YYYY-MM-DD; same pattern Stripe,
  Anthropic, OpenAI ship) carried through three canonical channels —
  the `Oyatie-Version` request header, the `/v/<YYYY-MM-DD>/...` URL
  prefix, and a `oyatie_version` protobuf field on every proto3 message
  envelope. The SDK distribution surface uses semantic versioning
  (major.minor.patch) per package; each SDK release pins a specific
  public-API date version under the hood so SDK consumers never have
  to author a date string. Last N=3 public versions remain supported
  for ≥180 days post-deprecation; per-tenant version pinning is
  declared in the tenant manifest under `api_version_pinning`; every
  tenant-affecting breaking change requires a paired ADR + sunset
  calendar + audit-chain `api.version.sunset` emission. Specify the
  per-µservice manifest field shape, the api-gateway routing layer
  shape, the developer-sdk release-engineering pipeline shape, the
  version-discovery surface (`GET /v/versions` + `oyatie.versions.v1`
  proto service), the deprecation-warning carrier (`Sunset` +
  `Deprecation` + `Link` headers per RFC 8594 + RFC 9745), and the
  eight new CI lanes that enforce the boundary. Do NOT author the
  actual `Oyatie-Version` carriers across every µservice contract in
  this ADR; that authoring is sequenced as a follow-on sub-wave under
  ADR-0328 batch discipline. Do NOT migrate any existing public
  contract surface in this ADR; per-µservice migration follows the
  canonical-build phase order.
---

# ADR-0342: API versioning HYBRID model — date-based (YYYY-MM-DD) for public APIs + semver (major.minor.patch) for SDK packages

## Status

Proposed on 2026-05-21.

This ADR is one of six approved on 2026-05-21 via the `/idea-refine` two-round interview cycle captured in `feedback_six_candidate_adrs_2026_05_21.md`. The sibling ADRs in the same approval batch are ADR-0340 (capacity model per µservice), ADR-0341 (cellular promotion gates explicit per-Tier), ADR-0343 (DR + RTO/RPO matrix per-µservice + per-pack), ADR-0344 (sustainability + finops dimensional model), and ADR-0345 (talent + OSS contribution policy). This ADR is the third in that six-ADR queue, prioritized to author after the 2026-05-21 doctrine cluster of ADR-0336 (Valkey), ADR-0337 (Iceberg), ADR-0338 (Pod runtime tier), and ADR-0339 (shared IaC module library) landed.

This ADR is the canonical API-versioning-shape decision establishing the date-version-for-public + semver-for-SDK split as binding on every Oyatie µservice that ships a public API surface and every SDK release from the developer-sdk generator. It directly amends ADR-0145 (inter-µservice communication reform) by carving the version-carrier triplet — header + URL prefix + protobuf field — into the canonical gRPC-over-HTTP/3 contract for public-facing endpoints. It directly amends ADR-0212 (buildability doctrine) by declaring a per-µservice manifest `api_versioning` block as the canonical declaration of the µservice's date-version surface area and deprecation calendar. It is binding on every µservice that declares an OpenAPI 3.2.0 path, AsyncAPI 3.1.0 channel, or proto3 service exposed beyond the internal mesh.

Enforcement transitions from `advisory-until-version-router-and-sdk-pipeline-land` to `BLOCKER` per the lane sequence in §E below: at landing of the version-router subsurface in `microservices/api-gateway/` and the SDK release-engineering pipeline in `microservices/developer-sdk/`, the lanes promote to BLOCKER for new authoring; per-µservice migrations of existing contract surfaces follow the µservice's canonical-build phase order under ADR-0328.

The decision does not delete any existing public-contract content. The decision does not change the OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 canonical version pins from `tools/hooks/_canonical-primitives.md`. The decision does not change the inter-µservice communication invariants from ADR-0145 (direct gRPC over HTTP/3 remains the canonical internal transport; this ADR adds the version carrier on top, it does not change the transport). The decision does not assert a particular SDK distribution channel (npm, PyPI, crates.io, Maven Central, Go modules, NuGet, vcpkg, Conan) beyond the existing developer-sdk responsibility per `feedback_developer_sdk_stainless_generator_2026_05_20`; channel-by-channel publish discipline lives in `docs/standards/sdk-release-engineering.md` companion-doc.

## Date

2026-05-21.

## Context

### A.1 Named pressure: Oyatie has no canonical public-API versioning shape today

Oyatie ships ~77 active µservices (47 baseline + 9 ERP + 13 B2B-leader + the in-flight 8 healthcare/marketing splits captured by the realignment effort per `specs/master-plan-sequencing.json#realignment_wave_sequence`). Of those, ~25 expose public-facing APIs that any external tenant, developer, or SDK consumer interacts with: cloud-iam, cloud-kms, cloud-secrets, cloud-billing, cloud-billing-tax, cloud-finops, cloud-marketplace, cloud-compute-* (vm/k8s/functions), cloud-data-*, cloud-iac, audit-chain, messenger, mail, drive, docs, sheets, slides, calendar, crm, marketing-automation, contract-lifecycle-management, itsm, intelligence, workflow-studio, developer-sdk, api-gateway.

Today the OpenAPI 3.2.0 surfaces, AsyncAPI 3.1.0 channels, and proto3 services across those ~25 µservices have **no canonical version carrier**. Some µservices ship `openapi: "1.0.0"` in their `info` block; some ship semver under different keys; some carry a `X-Service-Version` response header populated from the running build hash; some carry no version at all on the public surface. There is no shared rule for what a "v1" means, when a breaking change becomes "v2", how long an old version remains live, what the tenant-affecting deprecation calendar looks like, or how an SDK consumer pins to a version.

Two failure modes follow from this ambiguity:

- **Silent regression on the public boundary.** Per `feedback_no_silent_regression`, every public contract change is supposed to be ADR-traceable + sunset-gated. Today there is no canonical version surface to ADR-trace against; an OpenAPI 3.2.0 path that drops a field is indistinguishable from one that adds a field; no carrier makes the change observable to tenants until their integration breaks at runtime.
- **No SDK release engineering bar.** Per `feedback_developer_sdk_stainless_generator_2026_05_20`, the developer-sdk µservice is supposed to generate Stainless-class SDKs across 10 languages from OpenAPI/AsyncAPI/proto3 sources. Without a versioned input surface, the generator cannot reproducibly emit a semver-incremented SDK across the 10 languages; release cadence becomes ad-hoc and consumer-pinning becomes impossible.

### A.2 Named pressure: hyperscaler precedent uses date versions for public APIs

The hyperscaler precedent for public-API versioning at scale is uniform:

- **Stripe.** `Stripe-Version: 2024-06-20` request header (date string) plus per-key default version configurable via the Stripe Dashboard. SDK consumers do not pass the date — the SDK pins it under the hood; SDK semver describes the SDK package, the date describes the API. Stripe has supported every dated API version they ever shipped going back to at least 2011-01-01.
- **Anthropic.** `anthropic-version: 2023-06-01` request header (date string). Successive versions on 2023-06-01, 2024-10-22, 2025-03-01 carry breaking and additive changes; SDK consumers receive the version pin baked into the SDK release.
- **OpenAI.** `OpenAI-Beta: ...` header for opt-in features + `Azure OpenAI api-version: 2024-10-21` query/header for the Azure-flavored surface (Azure's exposure of OpenAI uses date versions explicitly). Mainline OpenAI ships date-anchored snapshots of completions / chat / embeddings endpoints.
- **AWS.** Every AWS service published its `Version: 2010-12-01` (Query API) or `?Version=2015-04-08` (REST) date-stamped version since the EC2 era. Internal AWS service-to-service traffic still carries date versions on every Query / JSON-1.1 payload.
- **Google Cloud.** Every Google Cloud REST API publishes per-version date suffixes (`v1`, `v1beta1`, `v1alpha1`) plus a per-service `version: 2024-12-01`-style date in the discovery document; SDKs pin to a specific discovery date.
- **GitHub.** `X-GitHub-Api-Version: 2022-11-28` header on the REST API since 2022; supported window of ~6 months between version bumps.

The hyperscaler convention is the same across all of them: **date strings on the public boundary, semver on the SDK boundary, multiple versions supported in parallel, a deprecation calendar.** Oyatie's public-API surface is in the wrong shape if it ships any other vocabulary.

### A.3 Named pressure: SDK release engineering needs semver, not dates

The developer-sdk µservice per `feedback_developer_sdk_stainless_generator_2026_05_20` is responsible for generating SDKs in 10 idiomatic languages: TypeScript, Python, Go, Java, Kotlin, Swift, Rust, .NET (C#), C, C++. The release vehicles for those languages are:

- TS → npm
- Python → PyPI (sdist + wheel)
- Go → Go modules (Git-tag-pinned)
- Java → Maven Central
- Kotlin → Maven Central
- Swift → Swift Package Manager + CocoaPods
- Rust → crates.io
- .NET-C# → NuGet
- C → vcpkg + Conan
- C++ → vcpkg + Conan

Every one of these distribution channels speaks semver natively. npm rejects non-semver tags. crates.io rejects non-semver tags. Maven Central rejects non-semver coordinates. Go modules pin via Git tags that follow the `vMAJOR.MINOR.PATCH` rule. NuGet, vcpkg, Conan all rely on semver for upgrade resolution.

An SDK release of `oyatie-sdk-typescript 3.2.1` means:
- Consumers can safely upgrade from 3.2.0 to 3.2.1 (patch — bug fix).
- Consumers can safely upgrade from 3.1.0 to 3.2.0 (minor — additive features).
- Consumers MUST review the changelog before upgrading from 2.x.x to 3.0.0 (major — breaking change).

This is the lingua franca that every SDK consumer (developer ecosystem, partner integrations, internal Oyatie teams) already speaks. Using date strings on the SDK side fights every package manager's contract; using semver on the public-API side fights every hyperscaler convention. The HYBRID model — date strings outside, semver inside the SDK release — matches both contracts simultaneously.

### A.4 Named pressure: tenant-affecting breaking change without sunset breaks tenants

Per `feedback_no_silent_regression`, Oyatie protects public contracts from silent change. Per ADR-0108 (deprecation-and-sunset discipline), every breaking change carries a sunset calendar and a per-tenant migration window. Per ADR-0244 (tenant scoping universal primitive), every audit-chain emit carries `tenant_id` so per-tenant impact can be measured.

In practice, a tenant-affecting breaking change without a sunset calendar produces:

- Tenant-side integration breakage with no advance notice.
- No SDK upgrade path because the SDK pinned to a now-removed version.
- An audit-chain gap (no `api.version.sunset` event was emitted because no sunset existed).
- An auditor-visible "did Oyatie give tenants reasonable notice?" question that fails the basic SOC2 + ISO 27001 change-management posture.

The hybrid versioning model with the supported window (N=3 versions for ≥180 days) and the per-tenant pinning declaration converts every breaking change into:

1. A new date-version `YYYY-MM-DD` introduced.
2. The prior date-versions remain live; tenants pinned to them continue receiving the old behavior.
3. A sunset-class ADR is opened announcing the deprecation.
4. The audit-chain emits `api.version.deprecated` immediately and `api.version.sunset` 180 days later.
5. Tenants pinned to the dropped version receive `Sunset:` + `Deprecation:` + `Link:` headers per RFC 8594 + RFC 9745 on every response.
6. At sunset, the dropped version returns `410 Gone` with a `Link: <new-version>; rel="successor-version"` pointer.

That is the canonical hyperscaler-grade migration carrier; the named pressure this ADR resolves is the absence of that carrier in Oyatie today.

### A.5 Named pressure: per-tenant version pinning is a feature, not a default

Per `feedback_microservice_ownership_coherence_2026_05_20`, each µservice owns its own ABI across versions. Per ADR-0244 (tenant scoping universal primitive), every tenant is the unit of feature gating. The intersection of these two doctrines is: **every tenant pins to a specific public-API date version** as part of its tenant manifest, and µservices serve the tenant's pinned version transparently.

This is the Stripe pattern: each Stripe account has a "default API version" set in the Dashboard; the per-account default applies unless the request explicitly overrides via the `Stripe-Version` header. This is the pattern Oyatie inherits. The tenant manifest field `api_version_pinning` declares:

```json
"api_version_pinning": {
  "default_oyatie_version": "2026-03-15",
  "per_microservice_overrides": {
    "cloud-billing": "2026-05-21",
    "messenger": "2026-01-10"
  },
  "auto_advance_policy": "pinned_until_sunset"
}
```

The `default_oyatie_version` is what the µservice resolver returns when no explicit `Oyatie-Version` header is present on the request. The `per_microservice_overrides` allow per-µservice deviation (e.g., a tenant on the global default 2026-03-15 may need the 2026-05-21 cloud-billing surface for a regulatory feature). The `auto_advance_policy` controls whether the tenant's pin auto-rolls forward on sunset (`auto_advance_at_sunset`) or remains pinned and returns `410 Gone` at sunset (`pinned_until_sunset`); the default is `pinned_until_sunset` per the conservative-by-default principle.

### A.6 Named pressure: ADR-0145 inter-µservice traffic must NOT carry public versions internally

Per ADR-0145 (inter-µservice communication reform), internal µservice-to-µservice traffic uses direct gRPC over HTTP/3 with three invariants (no peer-adapter coupling, ports declared in inner layers, runtime as composition root). The inter-µservice surface is **internal**; it is not a public API. The internal proto3 surface evolves under the proto3 backward-compatibility rules (only-add-new-fields, never-remove-tag-numbers, etc.); it does NOT carry the `Oyatie-Version` triplet on every inter-µservice call.

The `Oyatie-Version` triplet (header + URL prefix + protobuf field) applies only at the **public boundary** — the surface that an external tenant, partner, or SDK consumer touches. The api-gateway µservice is the ingress point that materializes the public-version surface; once inside the mesh, calls are proto3-versioned by tag-number compatibility per the ADR-0145 invariants.

This separation is critical because internal mesh calls happen ~10^4–10^6 per public request; carrying the date string on every internal hop would be measurable overhead. Restricting the triplet to the public boundary keeps the internal mesh at proto3-native efficiency while giving the public boundary the date-version carrier it needs.

### A.7 Named pressure: ADR-0328 substance-bar discipline forbids template stamping a "version" field

Per ADR-0322 (substance bar as doctrine and CI enforcement) + ADR-0324 (anti-script authoring doctrine) + ADR-0328 (substance bar as canonical sequence and batch discipline), every per-µservice artifact must carry bespoke per-µservice substance, not template-stamped plumbing. The `api_versioning` manifest block declared in §D-1 below is not template-stamped — each µservice's block declares:

- Its specific date-version surface area (which dates it ever shipped).
- Its specific deprecation calendar (which dates are deprecated and which are still supported).
- Its specific tenant-affecting breaking-change history (per-version changelog with sunset-class ADR cross-references).
- Its specific SDK-pinning relationship (which SDK semver release pinned which date version).

Template-stamping the same `api_versioning` block across µservices is a P0 finding under ADR-0324. The Wave 15V-API-Versioning-Adoption sub-wave (queued in §H below) explicitly dispatches per-µservice bespoke authoring, not a single sweep.

### A.8 Named pressure: discovery surface must be machine-readable

Per `feedback_api_first`, Oyatie ships contracts before handlers. The public version-discovery surface must be machine-readable so SDK generation, version-pinning UIs, integration test fixtures, and per-tenant migration tooling can all consume the supported-version list programmatically. The canonical discovery endpoints are:

- `GET /v/versions` (OpenAPI 3.2.0 spec under `microservices/api-gateway/contracts/openapi/version-discovery.yaml`).
- `oyatie.versions.v1.VersionsService.ListVersions` (proto3 service under `microservices/api-gateway/contracts/proto/versions.proto`).
- Both return: list of supported versions, the current default version, the deprecated versions with sunset dates, and the successor version pointers.

This is the same shape Stripe's `https://api.stripe.com/v1/api/versions` discovery surface provides (mostly via the Dashboard but consumable by API too); the AWS service-discovery `endpoints.json`; the Google discovery document. Oyatie ships the discovery surface as a first-class µservice (api-gateway responsibility), not as a static documentation page.

### A.9 Counterpart precedent

- **Stripe.** Date-versioned API. `Stripe-Version: 2024-06-20`. Multi-year supported window — every date version ever shipped is still routable. Per-account default pinned in the Dashboard. SDK semver pins under-the-hood. Reference: stripe-node, stripe-python, stripe-go all expose `apiVersion: '2024-06-20'` configuration knob defaulting to the SDK's baked-in date.
- **Anthropic.** Date-versioned API. `anthropic-version: 2023-06-01`. SDK semver. Reference: `@anthropic-ai/sdk` and `anthropic-python` pin the date string in the client constructor.
- **OpenAI / Azure OpenAI.** Mixed; Azure surface uses date versions explicitly (`api-version=2024-10-21`), mainline OpenAI uses semver-flavored endpoint paths (`/v1/chat/completions`) plus dated snapshot model identifiers (`gpt-4o-2024-08-06`).
- **AWS.** Date-versioned per-service since 2006-04-10 (EC2 launch). Continues to ship date strings on every Query / JSON-1.1 payload.
- **Google Cloud.** Per-service date versions in discovery documents.
- **GitHub.** `X-GitHub-Api-Version: 2022-11-28` since 2022.
- **Microsoft Graph.** `v1.0` + `beta` channels; channel-level versioning rather than date-stamped, but the SDK semver pins to a channel.
- **Cloudflare API.** `/client/v4/` URL prefix + dated breaking changes documented in `https://developers.cloudflare.com/api/`; no in-band date header.

Every hyperscaler-grade precedent operates a date-version-on-the-boundary + semver-on-the-SDK split. Oyatie is in the wrong shape if it picks any other vocabulary.

### A.10 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_six_candidate_adrs_2026_05_21.md` ADR-0342 section — "HYBRID API versioning: date-based (YYYY-MM-DD) for public APIs (Stripe/Anthropic/OpenAI pattern; N=3 version support window ≥180 days) + semver for SDK packages".
- Anchor 2: ADR-0145 (inter-µservice communication reform). The `Oyatie-Version` triplet is binding on the public boundary only; internal mesh calls follow proto3 tag-number backward compatibility.
- Anchor 3: ADR-0212 (buildability doctrine). Per-µservice manifest declarations describe the µservice's public-API version surface area.
- Anchor 4: ADR-0244 (tenant scoping universal primitive). Per-tenant version pinning lives in the tenant manifest.
- Anchor 5: ADR-0108 (deprecation-and-sunset discipline). Every tenant-affecting breaking change carries a sunset calendar.
- Anchor 6: ADR-0263 (observability emission contract). Audit-chain emits `api.version.*` events on deprecation / sunset / migration.
- Anchor 7: ADR-0322 + ADR-0324 + ADR-0328. Per-µservice substance-bar discipline applies to the `api_versioning` block.
- Anchor 8: ADR-0181 (cosign-signed artifacts and modules). SDK packages carry cosign attestations per ADR-0181; the SDK release pipeline emits provenance.
- Anchor 9: `feedback_developer_sdk_stainless_generator_2026_05_20`. The 10-language SDK matrix is binding on the SDK side of the hybrid.
- Anchor 10: `feedback_quality_performance_scalability_bar`. Hyperscaler-grade precedent (Stripe / Anthropic / OpenAI / AWS / Google / GitHub) is the substance bar.
- Anchor 11: `tools/hooks/_canonical-primitives.md` contracts section. OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3 remain canonical; this ADR does not change those.

### A.11 What this ADR does not assert

- **A.11.1** Does not change the OpenAPI 3.2.0, AsyncAPI 3.1.0, or proto3 canonical pins. Those remain in `tools/hooks/_canonical-primitives.md`.
- **A.11.2** Does not change the ADR-0145 inter-µservice communication invariants. The triplet applies only at the public boundary.
- **A.11.3** Does not retire any existing public contract. Every existing public surface is migrated under the Wave 15V-API-Versioning-Adoption sub-wave; nothing is deleted in this ADR.
- **A.11.4** Does not change SDK distribution channel selection (npm, PyPI, crates.io, Maven Central, Go modules, NuGet, vcpkg, Conan). Channel-by-channel publish discipline lives in `docs/standards/sdk-release-engineering.md`.
- **A.11.5** Does not change tenant_class semantics from ADR-0330. Version pinning is tenant-scoped; tenant_class governs feature ceilings (paid vs demo_trial), not API version.
- **A.11.6** Does not change Cedar evaluation from ADR-0243. Cedar gates remain authorization; version routing sits at admission / dispatch, not authorization.
- **A.11.7** Does not assert a particular hash / digest binding between SDK semver and public-API date version. The mapping is recorded in the SDK release manifest under `microservices/developer-sdk/release-manifests/<sdk-package>-<semver>.json`; the discovery surface returns the mapping but does not enforce a particular cryptographic binding (cosign attestation is sufficient).
- **A.11.8** Does not retire any existing CI lane. Eight new lanes (§E) are added.
- **A.11.9** Does not assert a single SDK release cadence. Per-package release cadence is per-language idiomatic (npm: per-feature; Maven Central: per-quarter; crates.io: per-feature). The discovery surface tracks which SDK semver pins which date.
- **A.11.10** Does not author actual `Oyatie-Version` headers / URL routes / proto fields across the ~25 public-facing µservice contracts. That authoring is sequenced as Wave 15V-API-Versioning-Adoption.

## Decision

### B.1 Decision statement

Oyatie's public-facing API surface (OpenAPI 3.2.0 endpoints, AsyncAPI 3.1.0 channels, proto3 services exposed beyond the internal mesh) uses **date-based versions in the form `YYYY-MM-DD`**, carried through three canonical channels:

1. The `Oyatie-Version: 2026-05-21` request header (every public request).
2. The `/v/<YYYY-MM-DD>/...` URL prefix (every public URL path under api-gateway routing).
3. The `oyatie_version` protobuf field on every public proto3 message envelope.

Oyatie's SDK distribution surface (10 idiomatic SDKs generated by developer-sdk per `feedback_developer_sdk_stainless_generator_2026_05_20`) uses **semantic versioning (major.minor.patch)** per package. Each SDK semver release pins a specific public-API date version under the hood; the SDK consumer never has to author the date string.

The last **N=3** public-API date versions remain supported in parallel for **≥180 days** post-deprecation. Every tenant declares a default Oyatie-Version pin (and optional per-µservice overrides) in its tenant manifest under `api_version_pinning`. Every tenant-affecting breaking change requires a paired sunset-class ADR + audit-chain `api.version.sunset` event emission + RFC 8594 + RFC 9745 deprecation-warning carrier on responses (`Sunset:`, `Deprecation:`, `Link:` headers).

A machine-readable discovery surface (`GET /v/versions` + `oyatie.versions.v1.VersionsService.ListVersions`) enumerates supported versions + default + deprecation calendar.

Per-µservice from-scratch API-versioning shape is non-canonical going forward. Existing public contracts remain compilable until each µservice's migration bucket lands under ADR-0328 canonical-build phase order; new authoring after this ADR is Accepted MUST use the hybrid shape.

The api-gateway µservice owns version routing (header parsing + URL-prefix dispatch + per-tenant resolution + Sunset/Deprecation/Link carrier injection); the developer-sdk µservice owns SDK release engineering (semver bump rule + per-language package emission + cosigned-artifact attestation + per-SDK-release date-version pinning).

### B.2 Numbered decision clauses

B2.001. Public-API version format = `YYYY-MM-DD` (ISO-8601 calendar date; UTC; no time component).

B2.002. The first canonical date-version is `2026-05-21` (the date this ADR is approved); preceding date versions may be declared retroactively for existing contracts in their respective per-µservice migration buckets.

B2.003. The three canonical version carriers are: HTTP request header `Oyatie-Version`, URL prefix `/v/<YYYY-MM-DD>/`, proto3 message field `oyatie_version` (snake_case identifier; string-typed).

B2.004. Every public OpenAPI 3.2.0 path MUST accept `Oyatie-Version` as a required header parameter; the operation's responses MUST also support the `Sunset`, `Deprecation`, and `Link` response headers per RFC 8594 + RFC 9745.

B2.005. Every public OpenAPI 3.2.0 path MUST be reachable under the URL prefix `/v/<YYYY-MM-DD>/...`. The api-gateway routes `<YYYY-MM-DD>` to the µservice handler bound to that date version.

B2.006. Every public proto3 message envelope MUST carry a `string oyatie_version = <reserved tag>` field. The tag number is reserved per `microservices/api-gateway/contracts/proto/_canonical_tags.proto`. (Reserved tag pool: tag numbers 8001..8020; this ADR reserves tag 8001 for `oyatie_version`.)

B2.007. Every public AsyncAPI 3.1.0 channel MUST declare a message header `oyatie-version` (kebab-case for AsyncAPI per-spec convention); the channel server URL MUST include `/v/<YYYY-MM-DD>/` in the path.

B2.008. The api-gateway µservice is the canonical version-routing point. When an incoming request omits the `Oyatie-Version` header, api-gateway resolves the effective version from the tenant manifest's `api_version_pinning.default_oyatie_version` (or per-µservice override).

B2.009. The api-gateway µservice rejects requests with conflicting carriers (e.g., header says `2026-05-21` but URL prefix says `/v/2026-03-15/`). Conflict response: HTTP 400 + body `{"error": "version_carrier_conflict", "header": "...", "url_prefix": "..."}`.

B2.010. The api-gateway µservice rejects requests with an unsupported version (a date string that was never published or has been sunset). Sunset response: HTTP 410 Gone + body `{"error": "version_sunset", "requested": "...", "successor": "...", "successor_link": "..."}` + `Link: </v/<successor>>; rel="successor-version"` header.

B2.011. The api-gateway µservice rejects requests with a deprecated-but-still-supported version with a 200 response carrying `Sunset:` + `Deprecation:` + `Link:` headers per RFC 8594 + RFC 9745.

B2.012. Supported-version window: at any given moment, **N=3** date versions are simultaneously supported (current + 2 prior). The exact N is decided per-µservice as a manifest declaration but MUST be ≥ 3.

B2.013. Deprecation window: a version is marked deprecated for **≥180 days** before sunset. The 180 days is the floor; per-µservice may declare a longer window in its manifest.

B2.014. The `Sunset:` header carries an HTTP-date (RFC 5322 format per RFC 8594) indicating when the version goes to 410 Gone.

B2.015. The `Deprecation:` header per RFC 9745 carries an `@<unixtime>` value indicating when the version was first marked deprecated.

B2.016. The `Link:` header carries `rel="deprecation"` pointing to the per-version migration document under `docs/api-migration/<from-date>-to-<to-date>.md` AND `rel="successor-version"` pointing to the successor version's URL prefix.

B2.017. Every µservice that ships a public API MUST declare a top-level `api_versioning` block in its `manifest.json`. The block enumerates: declared_versions (list of dates), default_version (date), deprecation_calendar (per-deprecated-version sunset date), supported_window_size (integer ≥ 3), supported_window_minimum_days (integer ≥ 180), public_surface_files (per-version OpenAPI / AsyncAPI / proto3 files).

B2.018. The per-µservice OpenAPI 3.2.0 source files are organized as `microservices/<name>/contracts/openapi/<YYYY-MM-DD>.yaml` — one file per supported date version. AsyncAPI: `microservices/<name>/contracts/asyncapi/<YYYY-MM-DD>.yaml`. proto3: `microservices/<name>/contracts/proto/<YYYY-MM-DD>/<service>.proto`.

B2.019. Every public proto3 file MUST be in a `microservices/<name>/contracts/proto/<YYYY-MM-DD>/` subdirectory. The `<YYYY-MM-DD>` segment is the canonical version anchor.

B2.020. SDK package version format = `MAJOR.MINOR.PATCH` (semver 2.0.0 strict).

B2.021. SDK MAJOR bump = breaking interface change (a method signature changed, a type was removed, a previously-required parameter became optional in a non-backward-compat way, the SDK's pinned date-version dropped support for an endpoint the SDK used).

B2.022. SDK MINOR bump = additive change (new methods, new types, new optional parameters; the SDK's pinned date-version added new endpoints).

B2.023. SDK PATCH bump = bug fix (no interface or type change; the SDK's pinned date-version was patched for a non-breaking fix).

B2.024. Every SDK release pins exactly one public-API date version under the hood. The mapping is recorded in `microservices/developer-sdk/release-manifests/<sdk-package>-<semver>.json` (machine-readable). The pinned date is also written as a constant in the generated SDK source (`OYATIE_API_DATE = "2026-05-21"`).

B2.025. SDK consumers MAY override the SDK's baked-in date version at construction time via a client option (`new OyatieClient({ apiVersion: "2026-03-15" })`). The override MUST be a supported (not sunset) date.

B2.026. The 10 canonical SDK languages per `feedback_developer_sdk_stainless_generator_2026_05_20`: TypeScript, Python, Go, Java, Kotlin, Swift, Rust, .NET-C#, C, C++. SDK release trains MUST publish all 10 on every minor or major release; patch releases MAY release a subset of affected languages.

B2.027. Each SDK package carries an ADR-0181 cosign attestation signed by the canonical Oyatie root key. The attestation includes the SDK semver + pinned date-version + content hash.

B2.028. The discovery endpoint `GET /v/versions` (OpenAPI 3.2.0 spec under `microservices/api-gateway/contracts/openapi/version-discovery.yaml`) returns:
```json
{
  "supported_versions": ["2026-05-21", "2026-03-15", "2026-01-10"],
  "default_version": "2026-05-21",
  "deprecated_versions": [
    {
      "version": "2026-01-10",
      "deprecation_date": "2026-05-21",
      "sunset_date": "2026-11-17",
      "successor": "2026-05-21",
      "migration_doc": "/docs/api-migration/2026-01-10-to-2026-05-21.md"
    }
  ],
  "per_microservice": {
    "cloud-billing": {"supported_versions": ["2026-05-21", "2026-04-01", "2026-02-14"], "default": "2026-05-21"},
    "messenger": {"supported_versions": ["2026-05-21", "2026-03-15", "2026-01-10"], "default": "2026-05-21"}
  }
}
```

B2.029. The discovery proto3 service `oyatie.versions.v1.VersionsService.ListVersions` returns the same structure.

B2.030. The discovery surface is UN-versioned (no `Oyatie-Version` header required for `/v/versions`). The surface itself follows a stable schema for ≥3 years; breaking changes to discovery itself require a major-version ADR.

B2.031. Every tenant's manifest gains an `api_version_pinning` block:
```json
"api_version_pinning": {
  "default_oyatie_version": "2026-05-21",
  "per_microservice_overrides": {"cloud-billing": "2026-04-01"},
  "auto_advance_policy": "pinned_until_sunset"
}
```

B2.032. The `auto_advance_policy` enum: `pinned_until_sunset` (default; tenant's pin persists; returns 410 at sunset) | `auto_advance_at_sunset` (tenant's pin auto-rolls forward to the successor on sunset).

B2.033. Tenant version pinning is mutable via the cloud-iam tenant-admin surface; changes emit an audit-chain `api.version.pin_change` event per ADR-0263.

B2.034. Every tenant-affecting breaking change requires:
  (a) A new date-version is introduced.
  (b) A paired sunset-class ADR is opened referencing this ADR (ADR-0342) as the procedural authority.
  (c) The old version is marked deprecated in the deprecation_calendar of every affected µservice's manifest.
  (d) The audit-chain emits `api.version.deprecated` immediately and `api.version.sunset` ≥180 days later.
  (e) Per-tenant migration documents are published under `docs/api-migration/<from-date>-to-<to-date>.md`.

B2.035. Audit-chain emits the canonical event classes `api.version.created`, `api.version.deprecated`, `api.version.sunset`, `api.version.pin_change`, `api.version.carrier_conflict`, `api.version.carrier_missing` per ADR-0263 emission contract.

B2.036. Every audit-chain emit, every metric, every distributed-tracing span on the public boundary gains the `oyatie_version` label (string, one of the declared supported versions). Cardinality bound: ≤ 5 distinct values per µservice (N=3 supported + ≤ 2 in transition).

B2.037. Internal µservice-to-µservice traffic (per ADR-0145) does NOT carry the `Oyatie-Version` triplet. Internal proto3 evolves under tag-number backward-compatibility rules independent of the public boundary.

B2.038. The api-gateway maps the incoming `Oyatie-Version` to the appropriate µservice handler at dispatch. Multiple handlers may co-exist in the same µservice binary (one handler per supported date version). The mapping is declared in the µservice's manifest under `api_versioning.public_surface_files`.

B2.039. SDK release trains follow the developer-sdk release-engineering pipeline. The pipeline:
  (a) Consumes the latest OpenAPI + AsyncAPI + proto3 contracts from every public µservice.
  (b) Generates the 10 idiomatic SDK packages.
  (c) Runs per-package contract-conformance tests against a live api-gateway instance.
  (d) Emits cosign-attested artifacts to the respective distribution channels.
  (e) Records the SDK-semver → date-version mapping in `release-manifests/`.

B2.040. The eight new CI lanes (per §E below) are REPORT-ONLY at this ADR's Acceptance and promote to BLOCKER per the §H sunset schedule.

B2.041. The Wave 15V-API-Versioning-Adoption sub-wave (queued in §H below) authors the manifest declarations + the per-µservice migration of contract files to the `<YYYY-MM-DD>.yaml` layout + the api-gateway routing layer + the discovery endpoints + the developer-sdk pipeline.

B2.042. Existing public contracts remain compilable until each µservice's migration bucket lands. The lanes are advisory until the api-gateway version-router and the developer-sdk release pipeline are operational; lanes promote to BLOCKER per-µservice as each migration bucket lands.

B2.043. New µservices created after this ADR is Accepted MUST author their public-API contracts under the hybrid shape from the first authoring step.

B2.044. The OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 canonical pins from `tools/hooks/_canonical-primitives.md` are preserved verbatim. This ADR ONLY adds the date-version carrier on top of the existing canonical contracts; it does not change the spec version.

B2.045. Six Rejected Alternatives are recorded in §F below: (i) semver-everywhere (semver on public boundary), (ii) date-everywhere (date strings on SDK packages), (iii) URL-prefix-only (no header, no proto field), (iv) header-only (no URL prefix routing), (v) query-parameter (`?api_version=...`), (vi) sub-domain (`v20260521.api.oyatie.dev`).

B2.046. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. The review evidence is at `evidence/debate/ADR-0342/` after this ADR lands in a review-track PR.

B2.047. The 30-day sunset window starts on Acceptance. The eight new lanes (§E) promote from REPORT-ONLY to BLOCKER for new authoring at day 30; per-µservice migration of existing contracts is sequenced under ADR-0328 and may extend the per-µservice-BLOCKER promotion until each µservice's migration bucket lands.

B2.048. The ADR is final on Acceptance. No exception clause is provided for any µservice's from-scratch versioning shape after the api-gateway router + developer-sdk pipeline land.

B2.049. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

B2.050. The ADR's enforcement and sunset run in coordination with the developer-sdk Stainless-class generator authoring per `feedback_developer_sdk_stainless_generator_2026_05_20`. The two work streams share the per-language emission layer.

### B.3 What this decision does not do

- This ADR does not author per-µservice manifest updates; the corpus-wide declaration sub-wave handles that.
- This ADR does not author the api-gateway version-routing handlers; that work belongs in api-gateway under the Wave 15V-API-Versioning-Adoption sub-wave.
- This ADR does not author the developer-sdk release-engineering pipeline; that work belongs in developer-sdk under the same sub-wave.
- This ADR does not change the canonical OpenAPI / AsyncAPI / proto3 spec pins.
- This ADR does not change the internal mesh proto3 evolution rules from ADR-0145.

## Consequences

### C.1 Positive consequences

- **Hyperscaler-grade public-API posture.** Date versions on the public boundary match Stripe / Anthropic / OpenAI / AWS / Google / GitHub precedent. SDK consumers and partner integrations find a familiar shape.
- **SDK release engineering becomes deterministic.** Per-language semver discipline is enforced; the 10 canonical languages release on a predictable cadence; consumers pin via package-manager-native mechanisms.
- **Tenant-affecting breaking changes carry sunset.** Per `feedback_no_silent_regression`, every breakage is ADR-traced + sunset-gated + audit-chain-emitted; the 180-day window gives tenants the canonical migration runway.
- **Per-tenant pinning is first-class.** Each tenant pins to a default version + per-µservice overrides; the tenant manifest is the authority; cloud-iam admin surface exposes mutation.
- **Discovery is machine-readable.** SDK generation, version-pinning UIs, integration tests, per-tenant migration tooling all consume the supported-version list programmatically.
- **Audit-chain coverage is canonical.** `api.version.*` event classes give regulators + auditors + internal post-mortem investigators the canonical evidence of every API evolution event.
- **Substance-bar reinforced.** The per-µservice `api_versioning` block is bespoke per-µservice (declared dates + deprecation calendar + supported window); template-stamping is detected by ADR-0324 lanes.
- **Hybrid model avoids friction.** Semver on the SDK boundary matches every distribution channel's contract; date strings on the public boundary match every hyperscaler precedent. The hybrid keeps both sides honest.
- **Observability cardinality bounded.** `oyatie_version` label cardinality ≤ 5 per µservice (N=3 supported + ≤ 2 in transition); aggregate Prometheus cardinality stays manageable.
- **RFC 8594 + RFC 9745 standards compliance.** `Sunset:`, `Deprecation:`, `Link:` headers per IETF RFCs; consumers can use off-the-shelf RFC-compliant tooling to detect deprecation.

### C.2 Negative consequences

- **Per-µservice migration cost.** ~25 public-facing µservices need:
  - manifest `api_versioning` block declaration.
  - OpenAPI / AsyncAPI / proto3 file restructure into `<YYYY-MM-DD>/` subdirectories.
  - handler routing layer wiring at api-gateway.
  - per-version handler implementations where multiple versions co-exist.
- **api-gateway routing complexity.** The api-gateway µservice gains version routing, per-tenant resolution, conflict detection, deprecation warning injection. Estimated ~5,000 LOC of new Rust code at api-gateway.
- **developer-sdk pipeline authoring cost.** The Stainless-class generator + 10-language emission + cosign attestation + release-manifest tracking is estimated ~20,000 LOC of new Rust code at developer-sdk + per-language template authoring.
- **Tenant manifest schema update.** Every tenant manifest gains `api_version_pinning`. Migration of existing tenants to declare the field is per-tenant under cloud-iam tenant-admin.
- **Cardinality multiplier on observability.** `oyatie_version` adds a label; bounded but real (~5x cardinality on public-boundary metrics).
- **Cross-team coordination.** axis-api-gateway + axis-developer-sdk + axis-tenant-management + ops-platform all participate in the implementation; quarterly tier review for the version surface.
- **Discovery-surface stability constraint.** `/v/versions` is UN-versioned; breaking changes to discovery itself need a major-version ADR amendment.

### C.3 Neutral consequences

- **Internal mesh unchanged.** ADR-0145 inter-µservice direct gRPC over HTTP/3 remains; the triplet applies only at the public boundary.
- **Cedar authorization unchanged.** Cedar evaluates application-layer authorization; version routing is admission / dispatch.
- **Existing OpenAPI / AsyncAPI / proto3 canonical pins unchanged.** This ADR adds the date carrier on top; it does not change the spec version.
- **Tenant_class semantics unchanged.** demo_trial and paid tenants both pin Oyatie-Version the same way; the version dimension is orthogonal to billing-component composition.
- **Compliance pack activation unchanged.** Compliance packs (HIPAA, PCI, GDPR-strict, CSAP, EU AI Act) are independent of the version dimension.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Per-µservice manifest declaration of supported versions + deprecation calendar | `api_versioning` block green across all ~25 public-facing µservices |
| Supply chain | Cosign-attested SDK artifacts; per-SDK-release date-version pinning recorded in release-manifest | release-manifest signatures match cosign attestations |
| Substance bar | Per-µservice `api_versioning` block is bespoke; template-stamping refused by ADR-0324 | substance-bar lane stays green |
| Hyperscaler alignment | Date-on-boundary + semver-on-SDK + supported-window pattern | discovery surface returns the supported window; SDK release manifests show the pinning |
| Performance | api-gateway version routing adds ≤1 hop overhead; carriers do not require parsing per internal hop | p99 latency budget at api-gateway preserved (≤1 ms additional) |
| Resilience | Multi-version coexistence in the same µservice binary; per-version handler isolation | per-version handler test coverage ≥ 95% per µservice |
| Compliance | Audit-chain emits `api.version.*` events on every evolution event; sunset window ≥180 days | audit-chain seal events recorded; regulator-replay reproducible |
| Standards | RFC 8594 (Sunset) + RFC 9745 (Deprecation) + RFC 5322 (HTTP-date) headers | response samples carry the standard headers |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** Stripe (date-versioned since 2011-01-01), Anthropic (since 2023-06-01), OpenAI / Azure OpenAI, AWS (per-service date versions since 2006-04-10 EC2 launch), Google Cloud (discovery-document dates), GitHub (`X-GitHub-Api-Version: 2022-11-28` since 2022). The date-on-boundary + semver-on-SDK split is the canonical hyperscaler pattern at scale.

**Failure-mode tree.** Failure modes:
(1) µservice forgets to declare `api_versioning` block → CI lane REPORT-ONLY signal; promotes to BLOCKER at sunset;
(2) Tenant manifest omits `api_version_pinning` → tenant defaults to the µservice's `default_version`; CI lane emits a warning;
(3) Conflict between header and URL prefix → api-gateway 400 + structured error body; observability emits `api.version.carrier_conflict`;
(4) Tenant requests a sunset version → 410 Gone + `Link: <successor>; rel="successor-version"`; tenant migration runbook is published;
(5) SDK semver-pinned date is sunset → SDK constructor logs deprecation warning; consumer must upgrade SDK or override apiVersion;
(6) Discovery endpoint goes down → consumers fall back to SDK's baked-in date; api-gateway returns its cached version list;
(7) New µservice adds public surface without `api_versioning` declaration → BLOCKER lane refuses at PR;
(8) Auditor asks for evidence of tenant notice on sunset → audit-chain `api.version.deprecated` + `api.version.sunset` events provide the record;
(9) SDK release without cosign attestation → BLOCKER lane refuses at release pipeline;
(10) Internal mesh accidentally carries `Oyatie-Version` triplet → CI lane refuses (the triplet is public-boundary only).

**Capacity math.** Triplet overhead per public request: header bytes (~25 B `Oyatie-Version: YYYY-MM-DD`) + URL prefix (~14 B `/v/YYYY-MM-DD/`) + proto3 field (~12 B varint-length-prefixed string). Aggregate ~50 B overhead per public request; bounded relative to typical 1-100 KB request payloads. Discovery endpoint cache: refresh interval 60 s; consumer-side cache TTL 5 min. SDK release engineering: 10 packages × ~1-50 MB each × monthly cadence ≈ ~5 GB monthly artifact emission.

**Observability hooks.** Every public-boundary request emits metric `api_request_total{microservice, oyatie_version, status}`; every audit-chain row carries `oyatie_version`; every distributed-tracing span on the public boundary carries `oyatie_version` attribute. Cardinality multiplier ≤ 5 per µservice.

**Rollback path.** Per-µservice rollback: a problematic new date-version is removed from the µservice's `supported_versions` list; tenants pinned to it auto-resolve to the prior supported version per `auto_advance_at_sunset`; api-gateway routes the new prefix to 410 Gone. SDK rollback: a problematic SDK release is yanked from the distribution channel (`npm unpublish`, `cargo yank`, `pip yank`); consumers fall back to the prior pinned semver. Aggregate corpus-wide rollback is not provided; rollback is per-µservice or per-SDK-package.

**Multi-region awareness.** Each region's api-gateway instance carries the same supported-version list (replicated via the cell-mesh per ADR-0044). Per-region version drift is forbidden; all regions serve identical supported windows at any given moment.

**Sovereign-cell awareness.** Sovereign cells per ADR-0240 host the same supported versions as non-sovereign cells. The compliance pack (HIPAA / GDPR-strict / CSAP / PCI / IL5) does not constrain the version dimension; it constrains the tenant-class capability ceiling (per ADR-0330) which is orthogonal.

**Versioning + deprecation.** This ADR is itself versioned per ADR-0108. Future amendments may extend the supported-window floor (e.g., to N=5 or to ≥360 days) under a new ADR. The date-on-boundary + semver-on-SDK shape itself is not expected to change.

## D. Detailed mechanics — ten enforcement surfaces

The hybrid versioning mechanism touches ten enforcement surfaces in api-gateway + developer-sdk + every public-facing µservice. Subsections D-1 through D-10 enumerate each surface. Numbering is normative.

### D-1: Per-µservice manifest `api_versioning` block

D-1.1. Every public-facing µservice's `microservices/<name>/manifest.json` MUST declare a top-level `api_versioning` block when the µservice ships any OpenAPI 3.2.0 path / AsyncAPI 3.1.0 channel / proto3 service exposed beyond the internal mesh.

D-1.2. The block schema:
```json
"api_versioning": {
  "declared_versions": ["2026-05-21", "2026-03-15", "2026-01-10"],
  "default_version": "2026-05-21",
  "supported_window_size": 3,
  "supported_window_minimum_days": 180,
  "deprecation_calendar": [
    {
      "version": "2026-01-10",
      "deprecated_on": "2026-05-21",
      "sunset_on": "2026-11-17",
      "successor": "2026-05-21",
      "breaking_change_adr": "ADR-NNNN",
      "migration_doc": "docs/api-migration/2026-01-10-to-2026-05-21.md"
    }
  ],
  "public_surface_files": {
    "openapi": {
      "2026-05-21": "contracts/openapi/2026-05-21.yaml",
      "2026-03-15": "contracts/openapi/2026-03-15.yaml",
      "2026-01-10": "contracts/openapi/2026-01-10.yaml"
    },
    "asyncapi": {
      "2026-05-21": "contracts/asyncapi/2026-05-21.yaml"
    },
    "proto": {
      "2026-05-21": "contracts/proto/2026-05-21/",
      "2026-03-15": "contracts/proto/2026-03-15/"
    }
  }
}
```

D-1.3. CI lane `oya-check-public-api-date-version` step 1 parses the manifest and validates: block presence, schema validity, `default_version` is in `declared_versions`, `supported_window_size` ≥ 3, `supported_window_minimum_days` ≥ 180.

D-1.4. CI lane step 2 validates each `public_surface_files` reference exists at the declared path.

D-1.5. CI lane step 3 validates each `deprecation_calendar` entry has `breaking_change_adr` populated with a real ADR file under `docs/decisions/`.

### D-2: Three-carrier triplet on public surfaces

D-2.1. **Header carrier.** `Oyatie-Version: YYYY-MM-DD` request header. MUST be accepted on every public OpenAPI 3.2.0 path operation. MUST be declared in the path's `parameters` array as a `header` parameter with `name: Oyatie-Version`, `in: header`, `required: false` (api-gateway resolves missing header to tenant default), `schema: { type: string, format: date }`.

D-2.2. **URL-prefix carrier.** Every public URL MUST be reachable under `/v/<YYYY-MM-DD>/...`. The api-gateway routes the date prefix to the corresponding µservice handler. Without the prefix, requests fall through to the tenant default version's handler.

D-2.3. **proto3 field carrier.** Every public proto3 message MUST carry `string oyatie_version = 8001;` at reserved tag 8001. The field is populated by the api-gateway-to-µservice dispatch layer.

D-2.4. **AsyncAPI carrier.** Every public AsyncAPI 3.1.0 channel MUST declare a message header `oyatie-version` (kebab-case per AsyncAPI). Channel-server URLs MUST include `/v/<YYYY-MM-DD>/`.

D-2.5. CI lane `oya-check-public-api-version-triplet` validates every public-facing OpenAPI path declares `Oyatie-Version` parameter; every public proto3 file declares the field at reserved tag 8001; every AsyncAPI channel declares the message header.

### D-3: api-gateway version-routing layer

D-3.1. The api-gateway µservice owns the version-routing layer at `microservices/api-gateway/src/version_router/`.

D-3.2. Routing algorithm:
  1. Parse `Oyatie-Version` header (if present).
  2. Parse URL prefix `/v/<YYYY-MM-DD>/` (if present).
  3. If both present and conflict → 400 + `api.version.carrier_conflict` audit emit.
  4. If neither present → resolve tenant default from tenant manifest.
  5. Look up the resolved version in the target µservice's manifest `api_versioning.declared_versions`.
  6. If version is deprecated → inject `Sunset:` + `Deprecation:` + `Link:` headers on response.
  7. If version is sunset (no longer in `declared_versions`) → 410 Gone + `Link: rel="successor-version"`.
  8. Forward the request to the µservice handler bound to the resolved version (per `public_surface_files.openapi/proto` map).

D-3.3. The routing layer caches the discovery view for 60 s; cache invalidates on `api.version.created` / `api.version.deprecated` / `api.version.sunset` audit events.

D-3.4. The routing layer emits per-request metric `api_gateway_version_resolution_total{tenant_id, resolved_version, source ∈ {header, url_prefix, tenant_default}}`.

D-3.5. The routing layer is BLOCKER-class: a routing layer outage means public API is down. Per-cell HA replica count ≥ 3.

### D-4: SDK release engineering pipeline at developer-sdk

D-4.1. The developer-sdk µservice owns the SDK release-engineering pipeline at `microservices/developer-sdk/src/release_pipeline/`.

D-4.2. Pipeline stages:
  1. **Snapshot.** Consume current public contracts from every µservice's `public_surface_files`.
  2. **Codegen.** Run Stainless-class generator per `feedback_developer_sdk_stainless_generator_2026_05_20` to emit 10 per-language SDKs.
  3. **Pin.** Write the consumed date-version into each SDK as `OYATIE_API_DATE` constant + into `release-manifests/<sdk-package>-<semver>.json`.
  4. **Test.** Run per-package contract-conformance tests against a live api-gateway instance pinned to the consumed date.
  5. **Bump.** Compare against the prior release and apply semver bump rule: major if breaking change detected, minor if additive, patch if no-op (or bug-fix-only).
  6. **Sign.** Generate cosign attestation per ADR-0181 including SDK semver + pinned date + content hash.
  7. **Publish.** Emit to per-language distribution channel.

D-4.3. The semver bump rule is planned to be enforced by CI lane `oya-check-sdk-semver-bump` (advisory until the crate lands): a release with a breaking diff that ships as MINOR or PATCH is refused.

D-4.4. The 10-language coverage is planned to be enforced by CI lane `oya-check-sdk-language-coverage` (advisory until the crate lands): a minor or major release that omits any language without an exception ADR is refused.

D-4.5. The release-manifest schema:
```json
{
  "sdk_package": "oyatie-sdk-typescript",
  "sdk_semver": "3.2.1",
  "pinned_oyatie_version": "2026-05-21",
  "content_hash": "sha256:...",
  "cosign_attestation_digest": "sha256:...",
  "released_on": "2026-05-21T12:00:00Z",
  "supersedes": "3.2.0",
  "bump_class": "patch"
}
```

D-4.6. Per-language distribution channel: npm (TS), PyPI (Python), Go modules (Go), Maven Central (Java + Kotlin), Swift Package Manager + CocoaPods (Swift), crates.io (Rust), NuGet (.NET-C#), vcpkg + Conan (C + C++).

### D-5: Tenant manifest `api_version_pinning` block

D-5.1. Every tenant manifest at `specs/tenant-manifest-schema.json` (the canonical tenant-manifest schema) gains an `api_version_pinning` block.

D-5.2. Schema:
```json
"api_version_pinning": {
  "default_oyatie_version": "2026-05-21",
  "per_microservice_overrides": {
    "cloud-billing": "2026-04-01"
  },
  "auto_advance_policy": "pinned_until_sunset"
}
```

D-5.3. Tenant onboarding (per ADR-0247 self-modification + cloud-iam tenant-creation flow) materializes `default_oyatie_version` as the then-current default at onboarding time.

D-5.4. Tenant admins mutate the pinning via cloud-iam tenant-admin surface. Mutations emit `api.version.pin_change` audit events.

D-5.5. CI lane `oya-check-tenant-version-pinning` validates: tenant manifests declare the block; pinned versions are in the corresponding µservice's `supported_versions` list; `auto_advance_policy` is one of `pinned_until_sunset` or `auto_advance_at_sunset`.

### D-6: Discovery surface (`GET /v/versions` + `oyatie.versions.v1.VersionsService`)

D-6.1. The discovery endpoint `GET /v/versions` is owned by api-gateway. OpenAPI spec at `microservices/api-gateway/contracts/openapi/version-discovery.yaml`.

D-6.2. The discovery proto3 service `oyatie.versions.v1.VersionsService.ListVersions` is owned by api-gateway. Proto3 file at `microservices/api-gateway/contracts/proto/_canonical_versions/versions.proto`.

D-6.3. The discovery surface is UN-versioned (no `Oyatie-Version` header required). The schema is stable for ≥3 years; breaking changes require an amendment ADR.

D-6.4. The discovery response includes: `supported_versions[]`, `default_version`, `deprecated_versions[]` (each with deprecation_date, sunset_date, successor, migration_doc), `per_microservice{}` (per-µservice supported_versions + default).

D-6.5. The discovery cache TTL: 60 s server-side; HTTP `Cache-Control: max-age=300` client-side.

D-6.6. Consumers (SDK clients, version-pinning UIs, integration tests) MUST honor the cache directives.

### D-7: RFC 8594 `Sunset:` + RFC 9745 `Deprecation:` + RFC 5988 `Link:` headers

D-7.1. The api-gateway injects RFC-compliant response headers on every public response served from a deprecated-but-still-supported version:

```
Sunset: Sun, 17 Nov 2026 00:00:00 GMT
Deprecation: @1748800000
Link: </docs/api-migration/2026-01-10-to-2026-05-21.md>; rel="deprecation",
      </v/2026-05-21>; rel="successor-version"
```

D-7.2. `Sunset:` per RFC 8594 carries an HTTP-date (RFC 5322 format) of when the version goes to 410 Gone.

D-7.3. `Deprecation:` per RFC 9745 carries `@<unixtime>` of when the version was first marked deprecated.

D-7.4. `Link:` per RFC 5988 carries:
  - `rel="deprecation"` → migration document URL.
  - `rel="successor-version"` → successor URL prefix.

D-7.5. SDK clients per `feedback_developer_sdk_stainless_generator_2026_05_20` MUST parse these headers and surface them to consumers (e.g., logger.warn). Per-language SDK template includes the header-parsing logic in the canonical client template.

### D-8: Audit-chain event classes

D-8.1. The audit-chain emits the following canonical event classes per ADR-0263 emission contract:

- `api.version.created` — a new date-version is published (emitted on the µservice's manifest update).
- `api.version.deprecated` — a version is marked deprecated (emitted when the deprecation_calendar entry is created).
- `api.version.sunset` — a version transitions from deprecated to sunset (emitted at the sunset_on date).
- `api.version.pin_change` — a tenant's `api_version_pinning` is mutated (emitted by cloud-iam tenant-admin).
- `api.version.carrier_conflict` — a request had conflicting header + URL prefix (emitted by api-gateway).
- `api.version.carrier_missing` — a request omitted the carrier and was resolved from tenant default (emitted by api-gateway at sampled rate; not every request).

D-8.2. Every audit-chain emit on the public boundary carries `oyatie_version` field per ADR-0263.

D-8.3. Every audit-chain emit on tenant version mutation carries the prior + new values per ADR-0263 mutation pattern.

D-8.4. Auditor view per regulator pack (HIPAA / GDPR / SOC2 / PCI): dashboard per-pack rolls up `api.version.deprecated` + `api.version.sunset` events to show notice-period compliance.

### D-9: Per-µservice contract file layout

D-9.1. OpenAPI 3.2.0 files: `microservices/<name>/contracts/openapi/<YYYY-MM-DD>.yaml`. One file per supported date version. Existing single-file layout (e.g., `openapi.yaml`) is migrated to `2026-05-21.yaml` (or appropriate first canonical date) under the per-µservice migration bucket.

D-9.2. AsyncAPI 3.1.0 files: `microservices/<name>/contracts/asyncapi/<YYYY-MM-DD>.yaml`. Same per-version pattern.

D-9.3. proto3 files: `microservices/<name>/contracts/proto/<YYYY-MM-DD>/<service>.proto`. The `<YYYY-MM-DD>` is a subdirectory; proto packages include the date suffix (`oyatie.cloud_billing.v2026_05_21.InvoiceService`).

D-9.4. The migration of existing contract files is per-µservice bespoke under the Wave 15V-API-Versioning-Adoption sub-wave. ADR-0324 anti-template-stamping discipline applies; each migration is bespoke.

D-9.5. The `_canonical_tags.proto` file at `microservices/api-gateway/contracts/proto/_canonical_tags.proto` reserves the version-carrier tag pool (8001..8020) corpus-wide. New µservices import the canonical tag reservations.

### D-10: Cellular + cross-region propagation

D-10.1. The supported-version list is replicated across cells via the cell-mesh per ADR-0044. Per-region drift is forbidden; all api-gateway instances in all regions serve identical supported windows at any moment.

D-10.2. New date-version publication: a control-plane event propagates the manifest update across cells; cells refresh their cached version list within 60 s.

D-10.3. Deprecation calendar advancement: a control-plane event propagates the deprecation marker across cells; cells refresh.

D-10.4. Sunset transition: at the sunset_on date, the control-plane transitions the version from deprecated to sunset; cells start returning 410 Gone for that prefix.

D-10.5. Sovereign-cell awareness per ADR-0240: sovereign cells (HIPAA / GDPR-strict / CSAP / PCI / IL5) follow the same supported-version timeline; the compliance pack does not extend or shorten the deprecation window.

D-10.6. dev-tools-cell-N awareness per ADR-0247: foundry workflow execution under `oyatie.foundry.*` principals consumes the same discovery surface; foundry tooling is a tenant from the version-pinning perspective.

## E. Enforcement-by-lanes

E.1 `oya-check-public-api-date-version` (new) — verifies every public-facing µservice manifest declares the `api_versioning` block with valid schema + ≥3 supported versions + ≥180-day window. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance for new authoring; BLOCKER per-µservice as each migration bucket lands.

E.2 `oya-check-public-api-version-triplet` (new) — verifies every public-facing OpenAPI path declares `Oyatie-Version` parameter; every public proto3 file declares the field at reserved tag 8001; every AsyncAPI channel declares the message header. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.3 `oya-check-public-api-supported-window` (new) — refuses manifest declarations that drop the supported window below N=3 or shrink the deprecation window below 180 days. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.4 `oya-check-public-api-sunset-adr` (new) — refuses additions to a µservice's `deprecation_calendar` without a paired sunset-class ADR file under `docs/decisions/`. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.5 `oya-check-sdk-semver-bump` (new) — refuses developer-sdk SDK releases that violate the major-on-breaking / minor-on-additive / patch-on-fix rule per D-4. REPORT-ONLY at Acceptance; BLOCKER once developer-sdk release pipeline lands.

E.6 `oya-check-sdk-language-coverage` (new) — refuses SDK release trains that omit any of the 10 canonical languages on minor or major releases without an exception ADR. REPORT-ONLY at Acceptance; BLOCKER once developer-sdk release pipeline lands.

E.7 `oya-check-tenant-version-pinning` (new) — refuses tenant manifests that omit the `api_version_pinning` block; refuses pinning to an unsupported / sunset version. REPORT-ONLY at Acceptance; BLOCKER once cloud-iam tenant-admin migration lands.

E.8 `oya-governance-version-routing-canonical-carriers` (new) — refuses any non-canonical version carrier in OpenAPI / AsyncAPI / proto3 (e.g., `?api_version=`, `X-API-Version` custom header, sub-domain). REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

## F. Alternatives Rejected

F.1 **Semver-everywhere (semver on the public boundary).** Use semver `1.2.3` as the public-API version anchor. Rejected because: every hyperscaler precedent (Stripe / Anthropic / OpenAI / AWS / Google / GitHub) uses date strings on the public boundary; semver on the boundary forces consumers to interpret "what does 1.2.3 → 2.0.0 mean for me" without a calendar anchor; aggregate corpus-wide semver coordination across ~25 public µservices is harder than per-µservice date anchors; semver on the boundary fights the tenant-pinning Stripe-pattern that every developer ecosystem already knows.

F.2 **Date-everywhere (date strings on SDK packages).** Use `2026-05-21` as the SDK package version on npm / PyPI / crates.io / Maven Central / etc. Rejected because: every package manager rejects non-semver tags; npm enforces semver; crates.io enforces semver; Maven Central enforces `MAJOR.MINOR.PATCH(-QUALIFIER)`; Go modules enforce `vMAJOR.MINOR.PATCH`; date strings break upgrade-resolution semantics (which is "newer", 2026-05-21 or 2026-05-22?); SDK consumers can't use `npm install oyatie-sdk@^3.2.0` semantics with date strings.

F.3 **URL-prefix-only (no header, no proto field).** Use `/v/<YYYY-MM-DD>/...` URL prefix as the only version carrier. Rejected because: URL prefixes are sufficient for HTTP REST surfaces but proto3 services need an in-message carrier (gRPC metadata is the equivalent of an HTTP header, but proto3 message envelopes also benefit from an explicit field for cross-language SDK ergonomics); AsyncAPI channels need an in-message carrier because the channel itself is the routing primitive, not a URL prefix; URL-prefix-only doesn't compose with the tenant-default-resolution flow because the URL doesn't carry the tenant.

F.4 **Header-only (no URL prefix routing).** Use `Oyatie-Version` header as the only version carrier. Rejected because: URL prefixes are how CDNs and reverse proxies cache + route per-version content; without a URL prefix, api-gateway must inspect every request body / header to route; URL-prefix routing also gives consumers a copy-pasteable URL that includes the version (Stripe's, Anthropic's, AWS's URL prefix patterns); header-only loses the URL anchor that documentation and SDK examples want.

F.5 **Query-parameter version carrier (`?api_version=...`).** Carry the version as a query parameter. Rejected because: query parameters affect cache keys awkwardly; CDN caching is harder with arbitrary query parameters; consumers often build URLs with query parameters reserved for the API's domain semantics; the hyperscaler precedent strongly prefers header or URL-prefix; Azure OpenAI uses `?api-version=...` but that's the outlier, not the canonical pattern.

F.6 **Sub-domain version (`v20260521.api.oyatie.dev`).** Use per-version DNS sub-domains. Rejected because: sub-domain proliferation pollutes DNS; TLS certificate management explodes (per-version wildcards); sub-domain routing per-version is an anti-pattern that no hyperscaler precedent uses; the cell-mesh per ADR-0044 prefers stable internal hostnames and per-version routing at the URL prefix layer.

## G. Multispectrum Review v2.4.0

Per ADR-0322 §D-2 and ADR-0328 §D-4, this ADR is subject to multispectrum-review v2.4.0 evaluation across the F-family critique facets, M-family meta facets, and A-family own-policy-adherence facets. Evidence files land at `evidence/debate/ADR-0342/<facet>.md` after this ADR is opened in a review-track PR.

The expected critique surface:

- **F1 (correctness).** Are the three carriers correctly bound? Is the reserved tag 8001 actually unused corpus-wide? Are the supported-window-floor (N=3 + ≥180 days) numbers correctly defended against tenant migration pressure?
- **F2 (architecture).** Should version routing live in api-gateway, or should each µservice own its own routing? Is the central api-gateway routing the right shape for cell-shaped deployments per ADR-0248?
- **F3 (security).** Does carrying `oyatie_version` in observability create a tenant-identification side-channel? Are the audit-chain emissions correctly scoped?
- **F4 (performance).** Is the ~50 B per-request triplet overhead acceptable on the public boundary? Is the api-gateway routing layer's ≤1 ms latency budget defended?
- **F5 (operability).** Is the quarterly tier review process correctly hooked up to deprecation calendars?
- **F6 (compliance).** Does the audit-chain coverage satisfy SOC2 + ISO 27001 change-management requirements? Does the ≥180-day window meet regulatory notice-period floors?
- **F7 (cost).** Does multi-version coexistence in the same µservice binary inflate cell capacity? Is the developer-sdk pipeline capacity correctly sized?
- **F8 (testability).** How are multi-version handlers tested? Per-version snapshot tests + cross-version contract conformance tests?
- **F9 (failure modes).** Is the failure-mode tree in C.5 complete?
- **M1 (counterpart-precedent calibration).** Are Stripe / Anthropic / OpenAI / AWS / Google / GitHub the right precedents? Is the Cloudflare counter-example correctly handled?
- **M2 (substance bar).** Is the per-µservice `api_versioning` block correctly bespoke and not template-stamped?
- **A1..A7 (own-policy-adherence).** Does this ADR adhere to naming BNF v4 (`oyatie_version`, `api_version_pinning`, `Oyatie-Version` carrier naming), documentation rigor 1.1, structural placement under `docs/decisions/`, architectural boundaries (api-gateway owns routing; developer-sdk owns SDK pipeline), dependency policy (no new external dependencies introduced), schema (manifest field naming + reserved tag pool), and algorithmic invariants (the triplet is binding on the public boundary only)?

## H. Enforcement + Sunset

H.1 **Enforcement transition.** From ADR Acceptance, the eight new lanes (§E) start REPORT-ONLY. They promote per the schedule:

- E.1, E.2, E.3, E.4, E.8 promote to BLOCKER 30 days post-Acceptance (for new authoring).
- E.5, E.6 promote to BLOCKER once the developer-sdk release pipeline is operational (sub-wave Wave 15V-API-Versioning-Adoption deliverable).
- E.7 promotes to BLOCKER once cloud-iam tenant-admin version-pinning UI ships and the per-tenant manifest migration completes.

H.2 **Sunset window.** The 30-day post-Acceptance window is the sunset window for new authoring of from-scratch versioning shapes. After day 30, new public-API authoring MUST use the hybrid shape; existing public contracts remain compilable until their migration bucket lands.

H.3 **Wave 15V-API-Versioning-Adoption sub-wave.** The Wave 15V-API-Versioning-Adoption sub-wave (queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.wave_15.subwaves`) authors:
  - Per-µservice `api_versioning` manifest blocks for ~25 public-facing µservices.
  - Per-µservice OpenAPI / AsyncAPI / proto3 file restructure into `<YYYY-MM-DD>/` layouts.
  - api-gateway version-routing layer (~5,000 LOC of new Rust).
  - api-gateway `/v/versions` discovery endpoint + `oyatie.versions.v1.VersionsService` proto3 service.
  - developer-sdk release-engineering pipeline (~20,000 LOC of new Rust + per-language templates).
  - Tenant manifest schema update + per-tenant `api_version_pinning` migration via cloud-iam.
  - Eight new CI lane implementations.

Sub-wave dispatch follows ADR-0328 batch discipline. Per-µservice bespoke authoring per ADR-0322 substance-bar + ADR-0324 anti-template discipline.

H.4 **Per-µservice migration sub-waves.** Each public-facing µservice's migration bucket is sequenced under ADR-0328 canonical-build phase order. Phase 0 cloud-* substrate µservices migrate first; Phase 4B long-tail B2B SaaS µservices migrate last. The aggregate corpus-wide migration is expected to span multiple realignment waves.

H.5 **Exception clause.** None. No µservice may continue authoring from-scratch versioning shapes after the 30-day sunset window for new authoring (existing surfaces are permitted until each migration bucket lands).

H.6 **Sunset of the prior shape.** From-scratch per-µservice version-carrier authoring is forbidden after day 30 for new authoring. The retirement is recorded in `tools/hooks/_canonical-primitives.md` per the canonical-primitives cheat sheet pattern (the new API Versioning section).

## I. Cross-references

I.1 Memory anchors:

- `feedback_six_candidate_adrs_2026_05_21` — user directive of 2026-05-21 capturing this decision as ADR-0342.
- `feedback_developer_sdk_stainless_generator_2026_05_20` — 10-language SDK matrix (TS / Python / Go / Java / Kotlin / Swift / Rust / .NET-C# / C / C++) + Stainless-class quality bar.
- `feedback_no_silent_regression` — public contracts protected from silent change; ADR + sunset + audit-chain required.
- `feedback_quality_performance_scalability_bar` — hyperscaler-grade precedent (Stripe / Anthropic / OpenAI / AWS / Google / GitHub).
- `feedback_clean_architecture_requirements` — separation of public boundary (api-gateway routing) from internal mesh (ADR-0145 direct gRPC).
- `feedback_microservice_ownership_coherence_2026_05_20` — per-µservice owner remains accountable for the µservice's `api_versioning` declaration.
- `feedback_rust_strict_only_no_python_2026_05_20` — api-gateway routing layer and developer-sdk pipeline are Rust-strict; per-language SDK templates are emitted artifacts not authored code.
- `feedback_bominal_inheritance_precedence` — Bominal corpus inherits the same hybrid versioning pattern.
- `feedback_docs_substance_not_scaffold_2026_05_20` — per-µservice `api_versioning` block is bespoke substance.
- `feedback_drift_too_big_2026_05_20` — versioning ambiguity is part of the drift this ADR closes.
- `feedback_api_first` — contracts before handlers; the discovery surface materializes contracts-as-code for the version dimension.

I.2 ADR anchors:

- ADR-0044 (inter-cell mesh tunnel) — cross-cell supported-version replication.
- ADR-0105 (13-layer canonical enum) — public-boundary version routing sits at api-gateway / dispatch layer.
- ADR-0108 (deprecation-and-sunset discipline) — every breaking change carries a sunset calendar.
- ADR-0145 (inter-µservice communication reform) — amended; the triplet applies only at the public boundary.
- ADR-0150 (Cedar policy engine) — Cedar authorization unchanged.
- ADR-0181 (cosign-signed artifacts and modules) — SDK packages cosign-attested.
- ADR-0183 (policy-engine separation: Cedar app-authz vs Kyverno admission) — version routing sits at dispatch, orthogonal to authorization.
- ADR-0211 (in-house tech stack preference) — Rust-strict; amended for SDK release-engineering substrate.
- ADR-0212 (buildability doctrine) — amended; manifest declaration of `api_versioning` block.
- ADR-0216 (deployment-context iac layout) — amended; api-gateway iac wrappers gain version-routing primitive invocations.
- ADR-0218 (OpenTofu not Terraform) — informational; version-routing OpenTofu primitives authored under ADR-0339 shape.
- ADR-0243 (Cedar as universal gate) — authorization unchanged.
- ADR-0244 (tenant scoping universal primitive) — amended; tenant manifest gains `api_version_pinning`.
- ADR-0245 (substrate vs product layering) — api-gateway is substrate; developer-sdk is substrate.
- ADR-0247 (self-modification doctrine) — foundry workflows consume the discovery surface as a tenant.
- ADR-0248 (Amazon-shape cellular architecture) — version routing replicates across cells.
- ADR-0250 (build ahead of certification) — versioning posture built day one.
- ADR-0251 (compliance pack cell certification levels) — packs do not extend or shorten the deprecation window.
- ADR-0263 (observability emission contract) — amended; `oyatie_version` label + `api.version.*` event classes added.
- ADR-0316 (tier system canonical Bronze/Silver/Gold/Platinum) — RETIRED; informational; version is orthogonal to capability-tier and tenant-class.
- ADR-0322 (substance bar as doctrine and CI enforcement) — per-µservice `api_versioning` block is bespoke.
- ADR-0324 (anti-script authoring doctrine) — template-stamping refused.
- ADR-0328 (substance bar as canonical sequence and batch discipline) — Wave 15V-API-Versioning-Adoption sub-wave.
- ADR-0329 (tier system retired; replaced by tenant_class) — informational.
- ADR-0330 (tenant_class demo_trial vs paid composable billing components) — informational.
- ADR-0331 (cross-microservice tenant_class adoption template) — informational.
- ADR-0335 (foundry retired; absorbed by intelligence) — informational; intelligence inherits the same versioning rules.
- ADR-0336 (Valkey not Redis substrate) — informational; substrate change orthogonal.
- ADR-0337 (Iceberg canonical OLAP) — informational.
- ADR-0338 (Pod runtime tier 0..3) — informational.
- ADR-0339 (Shared IaC module library) — version-routing OpenTofu primitives authored under this shape.
- ADR-0340 (Capacity model per-µservice) — version routing handlers contribute to capacity declarations.
- ADR-0341 (Cellular promotion gates explicit per-tier) — informational.

I.3 Spec anchors:

- `/specs/master-plan-sequencing.json` — adds the Wave 15V-API-Versioning-Adoption sub-wave + queued ADR-0342 entry per H.3.
- `/specs/microservices/manifest-schema.json` — admits the per-µservice `api_versioning` block per D-1.
- `/specs/microservices/api-gateway.json` — manifests-index pointer to the api-gateway version-routing expansion.
- `/specs/microservices/developer-sdk.json` — manifests-index pointer to the developer-sdk release-engineering expansion.
- `/specs/tenant-manifest-schema.json` — admits the `api_version_pinning` block per D-5.
- `/specs/markdown-retirement-policy.json` — informational; this ADR does not retire any prior markdown.

I.4 Companion-doc anchors:

- `docs/standards/api-versioning.md` (new) — canonical companion doc per `tools/hooks/_canonical-primitives.md` API Versioning section.
- `docs/standards/sdk-release-engineering.md` (new) — per-channel publish discipline + per-language template authoring.
- `docs/standards/deprecation-policy.md` — RFC 8594 + RFC 9745 carrier conventions documented; per-version migration document template.
- `microservices/api-gateway/ARCHITECTURE.md` — gains the version-routing layer description at landing.
- `microservices/api-gateway/manifest.json` — gains the `api_versioning` block as part of the migration.
- `microservices/developer-sdk/ARCHITECTURE.md` — gains the release-engineering pipeline description at landing.
- `microservices/developer-sdk/manifest.json` — gains the SDK-release-pipeline scope expansion.
- `tools/hooks/_canonical-primitives.md` — gains an API Versioning section naming this ADR and the hybrid model.

## J. Completion Report

<!--
adr: ADR-0342
status: Proposed
date: 2026-05-21
session: 2026-05-21 /idea-refine six-candidate batch (ADR-0342 of 6)
sibling_adrs: ADR-0340 (capacity model), ADR-0341 (cellular promotion gates), ADR-0343 (DR matrix), ADR-0344 (sustainability + finops), ADR-0345 (talent + OSS contribution policy)
authority_source: feedback_six_candidate_adrs_2026_05_21 (ADR-0342 section)
canonical_path_public_versions: YYYY-MM-DD (date-based; UTC)
canonical_path_sdk_versions: MAJOR.MINOR.PATCH (semver 2.0.0 strict)
canonical_carriers: header (Oyatie-Version) + URL prefix (/v/<YYYY-MM-DD>/) + proto3 field (oyatie_version, reserved tag 8001)
supported_window_floor: N=3 versions × ≥180 days post-deprecation
tenant_pinning_block: api_version_pinning { default_oyatie_version, per_microservice_overrides, auto_advance_policy }
sdk_language_coverage: TS / Python / Go / Java / Kotlin / Swift / Rust / .NET-C# / C / C++ (10 languages per feedback_developer_sdk_stainless_generator_2026_05_20)
discovery_surface: GET /v/versions + oyatie.versions.v1.VersionsService.ListVersions
deprecation_headers: Sunset (RFC 8594) + Deprecation (RFC 9745) + Link (RFC 5988)
audit_event_classes: api.version.created, api.version.deprecated, api.version.sunset, api.version.pin_change, api.version.carrier_conflict, api.version.carrier_missing
new_lanes: 8 (oya-check-public-api-date-version, -version-triplet, -supported-window, -sunset-adr, oya-check-sdk-semver-bump, -language-coverage, oya-check-tenant-version-pinning, oya-governance-version-routing-canonical-carriers)
sunset_window: 30 days post-Acceptance for new authoring; per-µservice migration follows ADR-0328 canonical-build phase order
wave_queue: Wave 15V-API-Versioning-Adoption added to /specs/master-plan-sequencing.json#realignment_wave_sequence.wave_15.subwaves
manifest_expansions: api-gateway gains version-routing scope; developer-sdk gains SDK-release-pipeline scope; ~25 public-facing µservices gain api_versioning block; tenant-manifest-schema gains api_version_pinning block
out_of_scope: authoring actual Oyatie-Version carriers across all µservice contracts (sequenced as Wave 15V-API-Versioning-Adoption); authoring per-µservice contract-file restructure (sequenced per-µservice under ADR-0328); authoring api-gateway routing layer + developer-sdk pipeline (sequenced under same sub-wave)
hyperscaler_precedents: Stripe (Stripe-Version since 2011-01-01); Anthropic (anthropic-version since 2023-06-01); OpenAI / Azure OpenAI (api-version=); AWS (per-service date versions since 2006-04-10 EC2 launch); Google Cloud (discovery-document dates); GitHub (X-GitHub-Api-Version since 2022-11-28)
related_memory_evidence: feedback_six_candidate_adrs_2026_05_21 + feedback_developer_sdk_stainless_generator_2026_05_20 + feedback_no_silent_regression
commits: none required at this ADR's landing
-->
