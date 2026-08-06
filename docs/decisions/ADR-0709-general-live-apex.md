---
id: ADR-0709
title: "Live general architecture and remaining accepted doctrine"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0003, ADR-0004, ADR-0006, ADR-0008, ADR-0014, ADR-0016, ADR-0018, ADR-0020, ADR-0022, ADR-0023, ADR-0024, ADR-0025, ADR-0039, ADR-0045, ADR-0048, ADR-0051, ADR-0055, ADR-0060, ADR-0061, ADR-0062, ADR-0063, ADR-0064, ADR-0065, ADR-0067, ADR-0069, ADR-0092, ADR-0096, ADR-0098, ADR-0100, ADR-0105, ADR-0108, ADR-0109, ADR-0115, ADR-0116, ADR-0119, ADR-0122, ADR-0133, ADR-0144, ADR-0146, ADR-0147, ADR-0149, ADR-0150, ADR-0151, ADR-0152, ADR-0153, ADR-0154, ADR-0156, ADR-0161, ADR-0164, ADR-0168, ADR-0169, ADR-0171, ADR-0173, ADR-0174, ADR-0178, ADR-0179, ADR-0181, ADR-0184, ADR-0189, ADR-0192, ADR-0193, ADR-0195, ADR-0196, ADR-0200, ADR-0202, ADR-0203, ADR-0205, ADR-0207, ADR-0209, ADR-0211, ADR-0212, ADR-0216, ADR-0217, ADR-0219, ADR-0221, ADR-0235, ADR-0237, ADR-0238, ADR-0239, ADR-0250, ADR-0252, ADR-0254, ADR-0257, ADR-0272, ADR-0292, ADR-0296, ADR-0298, ADR-0299, ADR-0304, ADR-0308, ADR-0317, ADR-0318, ADR-0324, ADR-0337, ADR-0339, ADR-0350, ADR-0362, ADR-0364, ADR-0365, ADR-0368, ADR-0371, ADR-0382, ADR-0388, ADR-0389, ADR-0390, ADR-0391, ADR-0393, ADR-0397, ADR-0478, ADR-0481, ADR-0506, ADR-0507, ADR-0508, ADR-0518, ADR-0541, ADR-0542, ADR-0555, ADR-0557, ADR-0558, ADR-0568, ADR-0610, ADR-0619, ADR-0622, ADR-0623, ADR-0625, ADR-0626]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0709-D1
    description: "Live apex source-of-truth for topic general: Live general architecture and remaining accepted doctrine."
    exit_criteria: "docs/decisions/ADR-0709-general-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0709: Live general architecture and remaining accepted doctrine

## Status

**Accepted** — live consolidated source-of-truth entry for topic `general` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **126** Accepted ADRs in the `general` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `general` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-3** (ADR-0003-audit-chain-and-evidence-emission): We adopt a single **append-only, hash-chained audit-event log** as the tamper-evident record-keeping surface for every regulated event in every axis. The kernel is `crates/oya-audit-chain-kernel`; the application layer is `crates/oya-audit-chain-app`; per-tenant shards live behind `crates/oya-audit-chain-adapter-postgres-*` with optional cold-tier 
- **ADR-4** (ADR-0004-plane-separation-control-data-analytics): Every surface in every axis declares one of three planes, validated at the catalog layer and enforced in CI. ```rust // crates/oya-foundation-plane-kernel #[derive(Clone, Copy, PartialEq, Eq)] pub enum Plane { /// Low-frequency, high-trust, audit-heavy. Configures and gates. Control, /// High-frequency, latency-bounded, fan-out scaled. Executes req
- **ADR-6** (ADR-0006-ontology-typed-entity-layer): We adopt the **Ontology** as Oyatie's single typed-entity layer. The kernel is `oya-ontology-entity-kernel`; per-property-tier adapters live in `oya-ontology-adapter-{scalar,vector,timeseries,geo,ciphertext,struct}-*`. Every entity carries a `TenantId`, an `ObjectId`, a `PropertyTier` per declared property, a `data_class` per property (per ADR-0008
- **ADR-8** (ADR-0008-data-use-boundary): We adopt the **Data Use Boundary** as the contract that governs which tenant data may flow across axes under what consent, for what purpose, to what subject class, in what jurisdiction. The boundary is enforced at six structural layers (compile-time first), uses an *orthogonal* subject-class attribute (not a 13th data class), uses a *purpose-permis
- **ADR-14** (ADR-0014-build-vs-buy-policy): We adopt a **per-microservice build-vs-buy matrix**, a **decision flow chart**, **per-dep metadata** in the catalog, and a CI lane that enforces the matrix. ### Per-axis matrix (in-house obligatory / external acceptable / requires-review) | Axis surface | Default | Rationale | |---|---|---| | **Foundation kernels** (Tenant, Identity, Audit chain, C
- **ADR-16** (ADR-0016-wave-and-plane-integration-framework): We adopt **descriptive wave names**, **per-wave gate criteria**, **`preview / stable / GA` status labels**, and explicitly forbid `M0..M3 / minimum-shippable-tier` vocab (ADR-0018 enforces this in the glossary fitness lane). ### Wave names (canonical) | Wave | Description | |---|---| | **W-Foundation** | Foundation correctness: tenancy + identity k
- **ADR-18** (ADR-0018-glossary-and-terminology-canon): We adopt the **glossary canon** with five rules, an industry-aligned vocabulary list, Oyatie-specific terms, a Korean-English parity table, and a CI lane that hard-fails forbidden tokens. ### The five vocabulary rules 1. **Industry-standard term wins** when one exists and is unambiguous. 2. **Oyatie-specific term** is reserved for genuinely new con
- **ADR-20** (ADR-0020-intelligence-multi-provider-adapter-model): We introduce a single normalized provider contract in `oya-intelligence-adapter-kernel` and wire every concrete provider through it. The runtime — not the capability author — chooses which adapter handles a given invocation. ### Trait surface (`oya-intelligence-adapter-kernel`) ```rust // crates/oya-intelligence-adapter-kernel/src/lib.rs pub trait 
- **ADR-22** (ADR-0022-autonomy-ceiling-runtime-enforcement): We enforce the autonomy ceiling at `oya-intelligence-policy-app` on **every** capability invocation. The effective ceiling is the minimum of four sources; agents inherit (and cannot exceed) tenant permissions; healthcare and fintech tenant classes force T1/T2 maxima for regulated capabilities; agentic ad-buying defaults to recommend-only. ### Effec
- **ADR-23** (ADR-0023-intelligence-sandbox-wasmtime-firecracker): We adopt a two-tier sandbox: **Wasmtime + WASI Preview 2** for short-lived deterministic tools and **Firecracker microVMs** for tools that require a full kernel surface. The capability declares its sandbox class in the registry; the runtime selects the substrate; both substrates share a uniform per-spawn audit emission and a uniform resource-cap co
- **ADR-24** (ADR-0024-intelligence-eval-harness-and-replay): Every capability publishes with a golden eval set; the eval harness gates publish, runs nightly, runs A/B against routing decisions, and replays against past production traces for regression detection. Adversarial and regional linguistic cohorts are mandatory. ### Eval kernel (`oya-intelligence-eval-kernel`) ```rust // crates/oya-intelligence-eval-
- **ADR-25** (ADR-0025-intelligence-as-engineering-platform): We consolidate the engineering platform surfaces into the intelligence. The axis owns: `repoctl`, the catalog, the claim-ceiling validator, the foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, supply-chain (Trivy / Cosign / SBOM), plugin substrate trust gates, plugin
- **ADR-39** (ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits): We adopt **Trivy 4-layer scanning** (filesystem + container + IaC + dep) on every PR + nightly; **Cosign keyless signing** for every release artifact; **Rekor transparency log** for signature inclusion; **SBOM in SPDX 2.3 + CycloneDX 1.5** per artifact; **signed commits and tags** repo-wide; **merge-governance ruleset** at the GitHub level; **Kyver
- **ADR-45** (ADR-0045-database-tier-strategy): We adopt **PostgreSQL + Citus** (Apache-2) as the canonical OLTP engine; **per-tenant per-cell shard topology**; **ClickHouse Apache-2 fork** as the canonical OLAP engine (with explicit fork-license verification per License Policy); **Iceberg + DataFusion** (Apache-2) as the canonical lakehouse format + query engine; backup orchestration per ADR-00
- **ADR-48** (ADR-0048-korean-morphology-and-multilingual-tokenization): We adopt a **`Tokenizer` trait per language family** under `crates/oya-search-tokenizer-*`; **mecab-ko + khaiii via FFI day-1** (with mecab-ko legal-isolation analysis per License Policy + Apache-2 khaiii as the cleaner option for tenants who can use it); **in-house Rust port** of the KR morphology engine long-horizon; **per-pack tokenizer impl** f
- **ADR-51** (ADR-0051-mobile-and-native-client-strategy): ### 1. Web is the canonical surface Every Oyatie product surface ships web-first. Web (Leptos for engineering surfaces; SvelteKit for tenant-facing UIs per per-product PRD) is the **canonical** rendering of every capability and the conformance reference for every native client. A capability is **not** considered shipped on a native platform until w
- **ADR-55** (ADR-0055-object-graph-renamed-to-ontology): All "Object Graph" terminology is renamed to "Ontology" in all oyatie artifacts. ### Scope of rename | Was | Now | Location | |---|---|---| | "Object Graph" | "Ontology" | All ADRs, plans, docs, code | | `oya-*-object-graph-*` | `oya-ontology-*` | All crates (Shard 1 atomic rename) | | `oya-platform-object-graph-kernel` | `oya-ontology-entity-kerne
- **ADR-60** (ADR-0060-bominal-inheritance-precedence): Two-tier precedence for all architectural decisions in oyatie: 1. **Default (lower precedence):** Adopt Bominal ADR architecture decisions 1:1, translating Bominal terminology to oyatie canonical glossary (per ADR-0018). 2. **Override (higher precedence):** Anything decided in the 2026-05-13 /deep-interview session overrides Bominal when they confl
- **ADR-61** (ADR-0061-application-b2b-unified-shell): We adopt **Application** as the name for the B2B unified shell. Application is a microservice in the flat catalog registered as `application` in `[workspace.metadata.oya.microservices]`. ### Core model Application implements the Bominal ADR-0121 model (inherited) with glossary translation: - Tenants sign in via the identity substrate (ADR-0002; Bom
- **ADR-62** (ADR-0062-quality-performance-scalability-bar): ### Quality bar — Industry Leaders Every µservice must benchmark against the industry leader for its domain before graduating from Proof-Ladder L4 → L5: | Dimension | Reference standard | |---|---| | API design | Stripe (REST/gRPC contracts, idempotency, pagination, error model) | | Data layer | Palantir Ontology (typed entities + provenance + audi
- **ADR-63** (ADR-0063-documentation-set-coverage): ### 1. The canonical artifact set For every µservice registered in `[workspace.metadata.oya.microservices]`, the following artifacts MUST exist before the µservice's introducing-phase can pass its exit gate: | Artifact | Path convention | Template | |---|---|---| | Microservice record | `docs/microservices/<microservice>.md` | `docs/templates/micro
- **ADR-64** (ADR-0064-canonical-base-and-localization-packs): ### 1. Canonical global base + three overlay forms Every customer-facing µservice has a **canonical global base** (jurisdiction-agnostic) and zero or more **localization overlays**. The overlay form is chosen per-concern — three forms exist, all valid: | Form | Definition | When to use | Naming (BNF v4.1) | |---|---|---|---| | **Seam** | A port (tr
- **ADR-65** (ADR-0065-docs-as-leptos-webapp-with-machine-readable-coemit): ### 1. Triple-output documentation pipeline Every doc artifact ships in three forms: | Form | Audience | Path | |---|---|---| | **Markdown** (source) | Authors (humans + agents writing docs) | `docs/**/*.md` (current state preserved) | | **Leptos web pages** (rendered) | Human readers; search; cross-linking; live diff vs commit | served by `oya-doc
- **ADR-67** (ADR-0067-ops-oyatie-com-hyperscaler-operations-console): ### 1. µservice rename: `docs` → `ops` (catalog entry) Replace the `docs` µservice declared in ADR-0065 §2 with the parent µservice `ops`. The `docs` surface becomes one BC within `ops` (alongside dashboard / database / schema / tech-stack / architecture / health / tenant-mgmt / user-mgmt / observability / deployments / capacity / finops / on-call 
- **ADR-69** (ADR-0069-active-machine-readable-artifact-contract): Adopt the **active machine-readable artifact contract** v3.0.0 with three load-bearing artifacts and one validator crate. The contract is format-agnostic (applies to JSON, TOML, YAML, Cedar, SQL, OpenAPI, GitHub Actions YAML, Cargo.toml, etc.) and registry-based (control plane in registry; data plane in artifacts). ### Components | Component | Path
- **ADR-92** (ADR-0092-workspace-dependency-seam-policy): ### D1 — Canonical 12-layer enum The dependency-seam policy uses ADR-0056 v4.1's **canonical 12-value enum**: `{kernel, domain, application, adapter, infrastructure, cli, rest, grpc, graphql, worker, app, sdk}`. IP-002's 5-value enum is REJECTED as inconsistent with the canon. Layer is derived from the crate-name suffix per the BNF; no parallel `[p
- **ADR-96** (Supervisor language: Rust, not Node (build-vs-adopt Siigari/claude-heartbeat)): **Build in Rust.** The upstream Node implementation is rejected as a runtime dependency.
- **ADR-98** (Supervisor dependency policy Branch Y — zero net-new external Cargo deps + best-): **Branch Y — zero net-new external Cargo deps, sync I/O on tokio blocking pool, best-effort durability.** Concrete shape: ```rust // SessionDriver trait — synchronous; no async_trait dependency pub trait SessionDriver: Send + Sync { fn start_session(&self, ticket: &SessionTicket) -> Result<SessionHandle, SessionError>; fn stop_session(&self, handle
- **ADR-100** (Intelligence Supervisor Public Contract (Lean-a10)): The Intelligence Supervisor will expose zero new public APIs on existing kernels. Instead: 1. All supervisor-specific types (`SessionTicket`, `MessageId`, etc.) live in the new `oya-intelligence-supervisor-kernel`. 2. Existing kernel primitives are composed as pure ports. 3. The `AccountSnapshotProvider` port lives inside `oya-intelligence-supervisor-ke
- **ADR-105** (13-value canonical layer enum + check-family + backend-suffix patterns (amends A): ### Amendment 1 — Extend the canonical enum from 12 to 13 values: add `api`. The 13-value canonical layer enum: | Group | Values | |---|---| | Inner / pure (4) | `kernel`, `domain`, `application`, `app` | | Outer / external (2) | `adapter`, `infrastructure` | | Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, **`a
- **ADR-108** (Sunset → deprecation → removal lifecycle automation schema (machine-readable)): ### Machine-readable sunset schema Every sunset clause MUST be representable as a `SunsetClause` record with the following fields. The schema is identical across all three surfaces (ADR frontmatter YAML, spec JSON `_sunset` object, `[package.metadata.oya.sunset]` Cargo manifest section): | Field | Type | Required | Default | Description | |---|---|
- **ADR-109** (Lifecycle-automation framework (generic kernel + per-lifecycle configs)): 1. **One generic kernel.** `oya-governance-lifecycle-kernel` exposes the canonical `LifecycleConfig`, `LifecycledArtifact`, `Stage`, `Transition`, `Violation`, and `evaluate()` function. Every lifecycle lane is data — a JSON config under `specs/lifecycle-configs/`. Adding a new lifecycle is a config-file + thin dev-CLI commit, not a new kernel. 2. 
- **ADR-115** (ADR-0115-registry-consolidation-flat-singular): `registry/` (flat, singular) is the canonical machine-readable registry root. Every direct child of `registry/` is a semantic class (`catalog/`, `quality/`, `glossary/`, `vcs/`, `adr/`, `accounts/`, `capabilities/`, `cells/`, `audit-chain/`, `placeholder-debt/`, `graph/`, `claim-matrix/`, plus the flat-file cross-cutting registries that landed at t
- **ADR-116** (ADR-0116-retire-external-agent-coordination-tooling): The following external agent-coordination tools are **retired** from the prescribed agent surface in this repo, effective 2026-05-16: - `grit` (claim/work/done, scaffold-locks) - `icm` (coordination-lock topics, scaffold-locks-oyatie fallback) - `rtk` (cargo shim and command rewrites) - `vox` (inventoried but unused) The **Intelligence pipeline (M01-P18
- **ADR-119** (ADR-0119-specs-flat-root-topology): `specs/` is the canonical flat root for machine-readable specifications. The former nested spec scope directory is retired. All prior children of that retired directory are hoisted to `specs/`, while the typed lifecycle-config family remains grouped at `specs/lifecycle-configs/`. All live references to the retired nested path are rewritten to `spec
- **ADR-122** (ADR-0122-ontology-crate-rename-from-object-graph): Rename: | Current crate name | New crate name | Reason | |---|---|---| | `oya-platform-object-graph-kernel` | `oya-ontology-kernel` | Match Bominal-ADR-0106 Ontology naming + `feedback_glossary_ontology_not_object_graph` | | (already correct: `oya-ontology-api`, `oya-ontology-domain`) | n/a | sanity-check — these already use `ontology` | Plus the p
- **ADR-133** (ADR-0133-industry-best-practice-conformance-program): Adopt the 6-axis continuous industry-best-practice + hyperscaler-grade conformance program. Each axis carries: - **Industry baseline** (named primary sources) - **Audit cadence** (quarterly minimum; on-change for new µservices) - **Findings schema** (per `/specs/industry-best-practice-conformance.json`) - **Enforcement lane** (`oya-governance-indus
- **ADR-144** (ADR-0144-eu-ai-act-graduated-risk-tier-model): The risk classification is upgraded to a **5-tier graduated model** matching the EU AI Act 2024/1689 structure: | Tier | Risk class | EU AI Act anchor | Deployment status | |------|-----------------------|------------------------|-------------------| | 0 | Minimal-risk | Art. 50 (when AI-generated content is shown) | PERMITTED with Art. 50 disclosu
- **ADR-146** (ADR-0146-container-base-image-distroless-nonroot): The canonical base image for all Rust binary containers is **`gcr.io/distroless/static-debian12:nonroot`** (with the `:debug-nonroot` variant accepted only for explicit dev builds). Every µservice's `microservices/<ms>/iac/build/Dockerfile*` MUST: - Use the canonical base on the final stage. - Declare `USER 65532:65532` (or `USER 65532`) on the fin
- **ADR-147** (ADR-0147-container-sandboxing-runtime-ladder): oyatie adopts a **workload-class-tiered container sandboxing runtime ladder**. The canonical mapping below replaces the universal-gVisor default: | Workload class | Default runtime | Sovereign-tenant override | |--------------------------------------------------------|------------------------------------------------------------------|--------------
- **ADR-149** (ADR-0149-idempotency-keys-canonical): Adopt the canonical `Idempotency-Key` header as MANDATORY on every state-changing REST operation in every oyatie microservice. 1. The canonical specification is `docs/standards/idempotency-keys-canonical.md`. 2. The trait surface lives in `crates/oya-shared-idempotency-key-kernel/`. 3. Every µservice OpenAPI 3.2.0 document declares the canonical `I
- **ADR-150** (ADR-0150-cursor-pagination-canonical): Adopt opaque cursor pagination as MANDATORY on every list endpoint in every µservice; offset pagination is BANNED. 1. The canonical spec is `docs/standards/cursor-pagination-canonical.md`. 2. The trait surface lives in `crates/oya-shared-cursor-pagination-kernel/`. 3. Every µservice OpenAPI 3.2.0 list path declares `cursor` + `page_size` parameters
- **ADR-151** (ADR-0151-request-id-propagation): Adopt the canonical `X-Request-Id` header (ULID) propagated alongside OpenTelemetry `traceparent` on every inter-µservice call. 1. The canonical spec is `docs/standards/request-id-canonical.md`. 2. Every µservice's edge middleware GENERATES a fresh ULID if the header is absent, and PROPAGATES it on every outbound call. 3. Every µservice's outbound 
- **ADR-152** (ADR-0152-rpo-rto-canonical): Adopt a five-tier RTO model declared per-µservice and aggregated in `specs/microservices/rpo-rto-targets.json`. | Tier | Name | RTO | RPO | |------|----------------|-----------|---------| | R0 | realtime | < 5 min | 0 s | | R1 | hot | < 1 h | 5 min | | R2 | warm | < 4 h | 15 min | | R3 | cold | < 24 h | 1 h | | R4 | best-effort | best-eff | 24 h | 
- **ADR-153** (ADR-0153: Outbox Pattern): Adopt the transactional outbox pattern as the ONLY canonical way for a µservice to emit an event accompanying a state change. 1. The canonical spec is `docs/standards/outbox-pattern-canonical.md`. 2. The trait surface lives in `crates/oya-shared-outbox-pattern-kernel/`. 3. Every µservice with event-emission requirements creates one outbox table per
- **ADR-154** (ADR-0154-event-schema-versioning): Adopt explicit per-event `version` field as MANDATORY on every event emitted across every channel (WebSocket, AMQP, NATS, Kafka). 1. The canonical spec is `docs/standards/event-schema-versioning-canonical.md`. 2. Every AsyncAPI 3.1.0 message envelope MUST declare the `version` header and `event_id` (ULID per ADR-0156). 3. Backward-compatibility rul
- **ADR-156** (ADR-0156-pii-registry-canonical): Adopt a cross-cutting PII registry consolidating per-µservice data-class processing. 1. Every µservice's `manifest.json` gains a top-level `data_classes_processed` array (the UNION of per-BC `data_classes_owned`). 2. `specs/microservices/pii-registry.json` aggregates the per-µservice `data_classes_processed` into a cross-µservice index by data-clas
- **ADR-161** (ADR-0161-csi-storage-class-canonical): Oyatie adopts a canonical StorageClass naming scheme `oya-<kind>-<tier>` where: - `<kind>` ∈ `{pg, s3, redis, object}` — the storage primitive: - `pg` = PostgreSQL-backing block storage (RWO, filesystem ext4). - `s3` = S3-compatible object storage (no native StorageClass; mapped via CSI for `s3fs` workloads OR via direct S3-API for cloud-native pat
- **ADR-164** (ADR-0164-sovereign-cloud-air-gapped): Each sovereign pack declares `air_gap: true|false` in its pack manifest. When true, the following overlay applies: ### (a) On-prem container registry - Each cell deploys Harbor 2.x (CNCF graduated) as its in-cell container registry. - Image pull policy: `imagePullPolicy: IfNotPresent` + image references are rewritten to `registry.{cell}.svc.cluster
- **ADR-168** (ADR-0168-public-status-page): Oyatie deploys a public status page at `status.oya.dev` (and per-pack subdomains: `status.kr.oya.dev`, `status.eu.oya.dev`, etc. per ADR-0010 regional packs) automatically derived from SLO state per ADR-0139, with incident narrative pushed from the incident-response µservice. ### Architecture ``` ┌─────────────────────┐ ┌──────────────────────┐ │ A
- **ADR-169** (ADR-0169-webhook-dlq-retry): Oyatie introduces a SHARED webhook-delivery kernel (`crates/oya-shared-webhook-delivery-kernel/`) that every µservice with outbound-webhook needs integrates. The kernel owns: 1. **Delivery trait** — `WebhookDeliveryClient::deliver(endpoint, event, idempotency_key) -> DeliveryReceipt`. 2. **Retry schedule** — exponential backoff `1s, 2s, 4s, 8s, 16s
- **ADR-171** (ADR-0171-multi-cluster-federation): Oyatie adopts a three-component multi-cluster federation substrate: ### Component 1: ArgoCD ApplicationSets (application deployment across N clusters) Every µservice's `iac/helm/<ms>/` chart is referenced from a single ArgoCD `ApplicationSet` declaration. The ApplicationSet uses a cluster-list generator (`generators.list[]`) or a cluster-decision-r
- **ADR-173** (ADR-0173-vendor-lock-in-avoidance-and-stack-ownership): ### Default posture **OWN-the-stack via OSS substrate with permissive license is the default.** Vendor adoption requires an ADR-tracked exception that satisfies all four of: 1. Concrete business or quality benefit that an OSS substrate cannot currently match (cite the gap — capability, performance, cost). 2. Explicit phase-out plan (target replacem
- **ADR-174** (ADR-0174-finops-cost-attribution-chargeback): ### D-1. Canonical cost-tag block Every cloud resource provisioned by `microservices/cloud-iac/` MUST carry the following labels: | Tag | Type | Cardinality | Source of truth | | --- | --- | --- | --- | | `tenant_id` | UUID | per resource | tenancy µservice | | `cell_id` | UUID | per resource | cell registry (ADR-0009) | | `microservice` | enum | p
- **ADR-178** (ADR-0178-layered-throttling-tiers): ### D-1. Four layers, evaluated outermost-first ``` Request → [per-IP throttle] ↓ (allow) [per-API-key throttle] ↓ (allow) [per-user throttle] ↓ (allow) [per-tenant throttle] ↓ (allow) handler ``` Any layer's denial short-circuits subsequent evaluation. ### D-2. Per-layer policy | Layer | Counter store | Window | Default budget | Denial code | Head
- **ADR-179** (ADR-0179-postgres-connection-pooling-pgcat): Oyatie adopts **pgcat** (Rust pgbouncer-compatible, multi-tenant aware) as the canonical Postgres connection pooler for every µservice with a Postgres dependency. ### Operational shape 1. **Topology** — per-cell pgcat service (DaemonSet) handles fleet-wide pooling; per-µservice sidecar pgcat permitted ONLY when the µservice declares a tenant-isolat
- **ADR-181** (ADR-0181-container-image-promotion-pipeline): Oyatie declares a **three-tier container image promotion ladder**: `dev` → `staging` → `production`. Each tier has a distinct Cosign signing identity (Sigstore Fulcio OIDC-bound). Each cluster's pull policy restricts pulls to images carrying the appropriate-tier signature. ### Promotion ladder ``` dev signer staging signer prod signer (OIDC: dev) (
- **ADR-184** (ADR-0184-storage-tier-layering): Oyatie adopts a **four-tier storage layering** in which each tier owns exactly one access pattern: ### Tier 1 — OLTP write (PostgreSQL 18.4 primary) - Per-µservice Postgres 18.4 primary instance (one per bounded context; multi-tenant via row-level security). - Citus 14.0 for logical sharding by tenant where multi-tenant scale demands it (configured
- **ADR-189** (ADR-0189-step-up-authentication-acr-classes): **Four ACR classes, named `routine`, `elevated`, `sensitive`, `critical`. Each declares min-factor count, accepted factor mix, max session age. Cedar policies attach an `acr_required` to every action; ext_authz returns `step_up_required` when the principal's ACR is below the floor. The OIDC ID-token carries `acr` as a string-enum claim per RFC 9068
- **ADR-192** (ADR-0192-vector-database-canonical-milvus): Oyatie adopts **Milvus 2.6.x** (latest stable: 2.6.15 as of 2026-05-18; Apache-2.0; CNCF Graduated) as the canonical vector-database substrate fleet-wide. Milvus runs as a disaggregated cluster owned by the `intelligence` µservice (since embedding retrieval is a Intelligence AI-workload primitive) and is consumed by all µservices through the `oya-shared-vect
- _…plus 66 additional members listed in supersedes frontmatter; full text in git history / archive._

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-196 residual

**ADR-0196-object-storage-canonical-seaweedfs-primary-ceph-scale-up** — ### D-1. SeaweedFS 4.22 is the primary object store - **License:** Apache 2.0. - **Why primary:** simple operational model (master + volume + filer + S3 gateway), strong erasure-coding support, S3-compatible API, filer-backed POSIX/WebDAV side surfaces, active maintainer, broad adoption (Sina Weibo with billions of objects, multiple sovereign-cloud deployments in Asia). - **Scale envelope** valida

### ADR-205 residual

**ADR-0205-code-editor-canonical-codemirror** — **CodeMirror 6** is canonical for every in-product web code surface. Native shells use their platform-native text system. Same edit model + same language grammars where possible. | Surface | Library | |---|---| | SvelteKit / Leptos web | **CodeMirror 6** (`@codemirror/state` + `@codemirror/view` + per-language `@codemirror/lang-*`) | | SwiftUI (Apple) | `SwiftUI TextEditor` (TextKit 2 backend) wit

### ADR-171 residual

**ADR-0171-multi-cluster-federation** — Oyatie adopts a three-component multi-cluster federation substrate: ### Component 1: ArgoCD ApplicationSets (application deployment across N clusters) Every µservice's `iac/helm/<ms>/` chart is referenced from a single ArgoCD `ApplicationSet` declaration. The ApplicationSet uses a cluster-list generator (`generators.list[]`) or a cluster-decision-resource generator to fan out to each target cluste

### ADR-219 residual

**ADR-0219-no-code-first-ux-with-optional-ai-assist** — Most professional tasks must be possible without technical knowledge. Visual deterministic builders are the primary UX. AI assist through `microservices/intelligence/` is an opt-in accelerator for fuzzy or semantic tasks. ### Deterministic no-code patterns The primary path should be visual and deterministic for: - workflow building through a drag-and-drop canvas; - approval flow configuration thro

### ADR-203 residual

**ADR-0203-documentation-engine-three-tier** — Use three presentation tiers over one repository-owned Markdown and contract source of truth. ### Tier 1 — In-repository technical documentation - **Engine:** `mdbook`. - **Sources:** `docs/`, capability-local ADRs, runbooks, SLOs, and ownership documents. - **Audience:** engineers, reviewers, and air-gapped operators. - **Output:** a reproducible static artifact; rendered output is not committed

### ADR-115 residual

**ADR-0115-registry-consolidation-flat-singular** — `registry/` (flat, singular) is the canonical machine-readable registry root. Every direct child of `registry/` is a semantic class (`catalog/`, `quality/`, `glossary/`, `vcs/`, `adr/`, `accounts/`, `capabilities/`, `cells/`, `audit-chain/`, `placeholder-debt/`, `graph/`, `claim-matrix/`, plus the flat-file cross-cutting registries that landed at the root). `registries/cross-cutting/` is retired.

### ADR-51 residual

**ADR-0051-mobile-and-native-client-strategy** — ### 1. Web is the canonical surface Every Oyatie product surface ships web-first. Web (Leptos for engineering surfaces; SvelteKit for tenant-facing UIs per per-product PRD) is the **canonical** rendering of every capability and the conformance reference for every native client. A capability is **not** considered shipped on a native platform until web parity is verified. ### 2. Native mobile is in

### ADR-179 residual

**ADR-0179-postgres-connection-pooling-pgcat** — Oyatie adopts **pgcat** (Rust pgbouncer-compatible, multi-tenant aware) as the canonical Postgres connection pooler for every µservice with a Postgres dependency. ### Operational shape 1. **Topology** — per-cell pgcat service (DaemonSet) handles fleet-wide pooling; per-µservice sidecar pgcat permitted ONLY when the µservice declares a tenant-isolation constraint requiring per-pod identity binding.

### ADR-391 residual

**N-lane parallel safety proof and unified DevOps console** — Two gaps block production confidence for cloud-intelligence v1 and the broader N-lane parallel agent model:

### ADR-169 residual

**ADR-0169-webhook-dlq-retry** — Oyatie introduces a SHARED webhook-delivery kernel (`crates/oya-shared-webhook-delivery-kernel/`) that every µservice with outbound-webhook needs integrates. The kernel owns: 1. **Delivery trait** — `WebhookDeliveryClient::deliver(endpoint, event, idempotency_key) -> DeliveryReceipt`. 2. **Retry schedule** — exponential backoff `1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 512s, 1024s, 2048s, 4096s`

### ADR-211 residual

**ADR-0211-in-house-tech-stack-policy** — **Adopt a three-class classification for every external dependency** declared in the workspace (Rust crates, K8s controllers/operators, SaaS products, runtimes, model providers, payment processors): ### Class A — Community-standard KEEP The dependency is a CNCF-graduated or Linux-Foundation-hosted community standard with hyperscaler-reference adoption. KEEP indefinitely. Wrap behind a thin adapter

### ADR-168 residual

**ADR-0168-public-status-page** — Oyatie deploys a public status page at `status.oya.dev` (and per-pack subdomains: `status.kr.oya.dev`, `status.eu.oya.dev`, etc. per ADR-0010 regional packs) automatically derived from SLO state per ADR-0139, with incident narrative pushed from the incident-response µservice. ### Architecture ``` ┌─────────────────────┐ ┌──────────────────────┐ │ ADR-0139 SLO engine │───▶│ statuspage projector │ │

### ADR-152 residual

**ADR-0152-rpo-rto-canonical** — Adopt a five-tier RTO model declared per-µservice and aggregated in `specs/microservices/rpo-rto-targets.json`. | Tier | Name | RTO | RPO | |------|----------------|-----------|---------| | R0 | realtime | < 5 min | 0 s | | R1 | hot | < 1 h | 5 min | | R2 | warm | < 4 h | 15 min | | R3 | cold | < 24 h | 1 h | | R4 | best-effort | best-eff | 24 h | Each µservice's `backfill-replay.md` declares its

### ADR-507 residual

**ADR-0507-webauthn-rs-canonical-relying-party** — 1. **webauthn-rs is the canonical Phase-1 WebAuthn RP** across oyatie. All WebAuthn registration and authentication ceremonies are handled via webauthn-rs until oya-webauthn parity gates pass. 2. **Consumed via oya-identity's `oya-identity-webauthn-*` use-case crates**, following the PR #289 canonical clean-architecture pattern: `domain` / `usecase` / `api` / `adapter-postgres` / `rest` / `grpc` /

### ADR-368 residual

**ADR-0368-self-governing-agentic-platform-north-star** — Oyatie is a **self-improving, self-healing, self-governing agentic platform.** 1. **Masterplan = work for agents.** The masterplan is the generated (ADR-0364) collection of plans/tasks. It is the fleet's work queue. 2. **Maximum agents, always.** Deploy the maximum *safe* number of agents at all times against open masterplan deliverables; idle capacity is a defect (D1). Safety = the ADR-0366 confl

### ADR-299 residual

**ADR-0299-account-recovery-resilience** — ### §B. Decision summary **Decision 1: Multi-factor recovery is the canonical path.** Every account-recovery flow requires ≥2 factors verified before granting recovered state. The canonical factor set: - F1: Passkey backup (per ADR-0188 WebAuthn passkey-backup protocol). - F2: Recovery code (one-time-use, generated at account-creation + at recovery-state-change). - F3: Delegated trusted contact (p

### ADR-174 residual

**ADR-0174-finops-cost-attribution-chargeback** — ### D-1. Canonical cost-tag block Every cloud resource provisioned by `microservices/cloud-iac/` MUST carry the following labels: | Tag | Type | Cardinality | Source of truth | | --- | --- | --- | --- | | `tenant_id` | UUID | per resource | tenancy µservice | | `cell_id` | UUID | per resource | cell registry (ADR-0009) | | `microservice` | enum | per resource | µservice manifest (`microservices/<m

### ADR-337 residual

**Apache Iceberg is the canonical OLAP table-format write path (Delta + Hudi demoted to migration adapters; ClickHouse com** — ### B.1 Decision statement Apache Iceberg 1.7+ (Apache Software Foundation Apache-2.0) is the canonical Oyatie OLAP table-format write path corpus-wide. Apache Delta Lake (Linux Foundation Apache-2.0) and Apache Hudi (Apache Software Foundation Apache-2.0) are demoted to migration-adapter-only substrates: tenants ingesting Delta- or Hudi-formatted data are served by adapters that convert to Iceber

### ADR-6 residual

**ADR-0006-ontology-typed-entity-layer** — We adopt the **Ontology** as Oyatie's single typed-entity layer. The kernel is `oya-ontology-entity-kernel`; per-property-tier adapters live in `oya-ontology-adapter-{scalar,vector,timeseries,geo,ciphertext,struct}-*`. Every entity carries a `TenantId`, an `ObjectId`, a `PropertyTier` per declared property, a `data_class` per property (per ADR-0008), and an audit-chain emission hook (ADR-0003) on

### ADR-178 residual

**ADR-0178-layered-throttling-tiers** — ### D-1. Four layers, evaluated outermost-first ``` Request → [per-IP throttle] ↓ (allow) [per-API-key throttle] ↓ (allow) [per-user throttle] ↓ (allow) [per-tenant throttle] ↓ (allow) handler ``` Any layer's denial short-circuits subsequent evaluation. ### D-2. Per-layer policy | Layer | Counter store | Window | Default budget | Denial code | Header emitted | | --- | --- | --- | --- | --- | --- |

### ADR-626 residual

**Resolve fixup-ledger merges structurally instead of by hand** — Ship a structural three-way merge driver, following the two drivers already in the repo (`tools/oya-cargo-lock-merge-driver-app`, `tools/oya-friction-ledger-merge-driver-app`). **D1 — Pure kernel, thin binary.** The merge is a pure function over parsed rows with zero I/O, so it is fixture-drivable; the binary is only the `%O %A %B` git contract plus an atomic write. Same split as both sibling driv

### ADR-4 residual

**ADR-0004-plane-separation-control-data-analytics** — Every surface in every axis declares one of three planes, validated at the catalog layer and enforced in CI. ```rust // crates/oya-foundation-plane-kernel #[derive(Clone, Copy, PartialEq, Eq)] pub enum Plane { /// Low-frequency, high-trust, audit-heavy. Configures and gates. Control, /// High-frequency, latency-bounded, fan-out scaled. Executes requests. Data, /// Read-mostly on materialized proje

### ADR-365 residual

**ADR-0365-automated-adr-lifecycle-and-propagation** — ### 1. ADR authoring is an automated pipeline A decision flows: **`best-practice-research` → `planning-and-task-breakdown` / `ralplan` / `plan` (multi-perspective consensus) → ADR** in the ADR-0364 generative template. The ADR records its provenance: an `evidence[]` block (research citations) + a consensus record (planner/architect/critic verdicts). For a *new* service/lane, a PR-FAQ (does this be

### ADR-173 residual

**ADR-0173-vendor-lock-in-avoidance-and-stack-ownership** — ### Default posture **OWN-the-stack via OSS substrate with permissive license is the default.** Vendor adoption requires an ADR-tracked exception that satisfies all four of: 1. Concrete business or quality benefit that an OSS substrate cannot currently match (cite the gap — capability, performance, cost). 2. Explicit phase-out plan (target replacement, readiness gate, owner, review cadence). 3. Po

### ADR-156 residual

**ADR-0156-pii-registry-canonical** — Adopt a cross-cutting PII registry consolidating per-µservice data-class processing. 1. Every µservice's `manifest.json` gains a top-level `data_classes_processed` array (the UNION of per-BC `data_classes_owned`). 2. `specs/microservices/pii-registry.json` aggregates the per-µservice `data_classes_processed` into a cross-µservice index by data-class. 3. DSR cascade machinery queries the registry t

### ADR-181 residual

**ADR-0181-container-image-promotion-pipeline** — Oyatie declares a **three-tier container image promotion ladder**: `dev` → `staging` → `production`. Each tier has a distinct Cosign signing identity (Sigstore Fulcio OIDC-bound). Each cluster's pull policy restricts pulls to images carrying the appropriate-tier signature. ### Promotion ladder ``` dev signer staging signer prod signer (OIDC: dev) (OIDC: staging) (OIDC: prod) │ │ │ git tag rc-X ───

### ADR-292 residual

**ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification** — ### D-1. Per-jurisdiction age thresholds (canonical) Define the per-jurisdiction age threshold matrix canonically at `/specs/minor-user-doctrine.json` and ingest it as Cedar context attributes for evaluation. | Jurisdiction | "Child" threshold (under) | "Teen" upper bound (under) | "Adult" age | Authority | |---|---|---|---|---| | **US (federal, COPPA)** | 13 | n/a | 13 | 15 USC §6501-6506; 16 CFR

### ADR-623 residual

**Keep the pre-roadmap Stage-1 evidence epoch mechanism-neutral** — ### 1. One exact Stage-1 program The proposed canonical program identity is `correct-way-forward-before-roadmap`. If separately accepted, its instance could live at `/specs/masterplan.json#masterplan_v2.planning_entry_contract.stage1_closure_program`, with closed schemas and a pure semantic evaluator. No such masterplan field, schema, library, registry row, root-hub pointer, or cloud-CI gate exist

### ADR-119 residual

**ADR-0119-specs-flat-root-topology** — `specs/` is the canonical flat root for machine-readable specifications. The former nested spec scope directory is retired. All prior children of that retired directory are hoisted to `specs/`, while the typed lifecycle-config family remains grouped at `specs/lifecycle-configs/`. All live references to the retired nested path are rewritten to `specs/`. Historical prose may still use "cross-cutting

### ADR-250 residual

**ADR-0250-build-ahead-of-certification-doctrine** — ### D-1. Three states for certification-gated capabilities Every capability listed in §D-4 progresses through three distinct states. The state is declared at the capability level (per the `/specs/capability-certification-matrix.json` spec) and tracked per market (per the `/specs/capability-launch-roadmap.json` spec). | State | Meaning | Artifacts required to enter | Exit gate | |---|---|---|---| |

### ADR-296 residual

**ADR-0296-library-first-credential-sidecar** — The keystone establishes eight decision sub-sections, D-1 through D-8. ### D-1. Per-cell credential sidecar — definition The `oyatie.intelligence.credential-sidecar` is a per-cell pod deployed alongside every workload that performs LLM dispatch + audit emission. The sidecar holds: | Key class | Scope | Sidecar role | |---|---|---| | **Audit-signing key** (Ed25519) | Per-cell, per-tenant | The side

### ADR-161 residual

**ADR-0161-csi-storage-class-canonical** — Oyatie adopts a canonical StorageClass naming scheme `oya-<kind>-<tier>` where: - `<kind>` ∈ `{pg, s3, redis, object}` — the storage primitive: - `pg` = PostgreSQL-backing block storage (RWO, filesystem ext4). - `s3` = S3-compatible object storage (no native StorageClass; mapped via CSI for `s3fs` workloads OR via direct S3-API for cloud-native paths). - `redis` = Redis-backing block storage (RWO,

### ADR-200 residual

**ADR-0200-wasm-runtime-canonical-wasmtime** — The canonical WASM runtime for oyatie is **Wasmtime** — a BytecodeAlliance project, CNCF graduated, run in production at hyperscaler edge (notably Fastly Compute@Edge, whose engineering team is a primary upstream contributor and whose deployment profile is the reference operational model for our edge / Envoy filter use case). Footnoted floor: Wasmtime 30.x LTS line as of 2026-05-18 — workspace `Ca

### ADR-100 residual

**Intelligence Supervisor Public Contract (Lean-a10)** — The Intelligence Supervisor will expose zero new public APIs on existing kernels. Instead: 1. All supervisor-specific types (`SessionTicket`, `MessageId`, etc.) live in the new `oya-intelligence-supervisor-kernel`. 2. Existing kernel primitives are composed as pure ports. 3. The `AccountSnapshotProvider` port lives inside `oya-intelligence-supervisor-kernel` to keep the supervisor I/O-free without chan

### ADR-216 residual

**ADR-0216-open-integration-and-migration-out-policy** — Every customer-facing microservice that owns portable business data must ship an explicit open-integration surface: 1. first-party importer from the top three competitors or incumbent systems for that product surface; 2. first-party exporter to the top three competitors or a neutral standards-based archive; 3. OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts as the canonical integration surface

### ADR-364 residual

**ADR-0364-generative-adr-template-and-masterplan-generation** — ### 1. The masterplan is a GENERATED projection of the ADR log `oya gen masterplan` reads accepted `planning_impact: true` ADRs, topo-sorts by `depends_on`/`supersedes`, groups by `milestone`, and emits each `deliverable` as a roadmap line. `specs/masterplan.json` becomes build output, never hand-authored. A **drift gate** (committed == regenerated) is the inspection mechanism (Amazon "mechanisms,

### ADR-558 residual

**Friction-ledger structural merge driver: id-aware union + second-author conversion** — Ship `tools/oya-friction-ledger-merge-driver-app`, a structural three-way merge driver for the friction ledger, registered as `/.omc/ultragoal/friction-ledger.jsonl merge=friction-ledger` in `.gitattributes`, with these pinned semantics: - **Id-aware union.** Base rows preserved in base order (append-only doctrine: a side-deleted base row is preserved; legitimate redaction is a linearised commit o

### ADR-23 residual

**ADR-0023-intelligence-sandbox-wasmtime-firecracker** — We adopt a two-tier sandbox: **Wasmtime + WASI Preview 2** for short-lived deterministic tools and **Firecracker microVMs** for tools that require a full kernel surface. The capability declares its sandbox class in the registry; the runtime selects the substrate; both substrates share a uniform per-spawn audit emission and a uniform resource-cap contract. ### Sandbox kernel (`oya-intelligence-sand

### ADR-116 residual

**ADR-0116-retire-external-agent-coordination-tooling** — The following external agent-coordination tools are **retired** from the prescribed agent surface in this repo, effective 2026-05-16: - `grit` (claim/work/done, scaffold-locks) - `icm` (coordination-lock topics, scaffold-locks-oyatie fallback) - `rtk` (cargo shim and command rewrites) - `vox` (inventoried but unused) The **Intelligence pipeline (M01-P18)** is the sole canonical workflow for concurrent

### ADR-146 residual

**ADR-0146-container-base-image-distroless-nonroot** — The canonical base image for all Rust binary containers is **`gcr.io/distroless/static-debian12:nonroot`** (with the `:debug-nonroot` variant accepted only for explicit dev builds). Every µservice's `microservices/<ms>/iac/build/Dockerfile*` MUST: - Use the canonical base on the final stage. - Declare `USER 65532:65532` (or `USER 65532`) on the final stage. - Be validated by the `oya gate validate

### ADR-557 residual

**Migrate Kafka to Pulsar via KoP wire-compat** — 1. **Standalone Kafka is retired.** The cluster runs Pulsar 4.x + Oxia (per ADR-0397) as the sole canonical event-bus and log-broker substrate. 2. **KoP proxy fronts Pulsar for all Kafka clients.** The Kafka-on-Pulsar wire-compat layer provides a Kafka-protocol endpoint. Existing producers and consumers connect without any code changes. Kafka topics are mapped to Pulsar persistent topics under a `

### ADR-3 residual

**ADR-0003-audit-chain-and-evidence-emission** — We adopt a single **append-only, hash-chained audit-event log** as the tamper-evident record-keeping surface for every regulated event in every axis. The kernel is `crates/oya-audit-chain-kernel`; the application layer is `crates/oya-audit-chain-app`; per-tenant shards live behind `crates/oya-audit-chain-adapter-postgres-*` with optional cold-tier mirror. ### Chain structure ```rust // crates/oya-

### ADR-625 residual

**Commit OpenTofu provider dependency locks for every deployable root** — ### D1 — Every deployable root commits its lock Locks are generated with ```shell tofu providers lock -platform=linux_amd64 -platform=darwin_arm64 ``` which records checksums **without** contacting a backend, reading state, or using credentials. Both platforms are recorded because CI runs linux and development runs darwin. ### D2 — The review root is not a deployment lock, and is unchanged `cloud/

### ADR-14 residual

**ADR-0014-build-vs-buy-policy** — We adopt a **per-microservice build-vs-buy matrix**, a **decision flow chart**, **per-dep metadata** in the catalog, and a CI lane that enforces the matrix. ### Per-axis matrix (in-house obligatory / external acceptable / requires-review) | Axis surface | Default | Rationale | |---|---|---| | **Foundation kernels** (Tenant, Identity, Audit chain, Capability registry, Plane, Eventing, Policy/Cedar,

### ADR-25 residual

**ADR-0025-intelligence-as-engineering-platform** — We consolidate the engineering platform surfaces into the intelligence. The axis owns: `repoctl`, the catalog, the claim-ceiling validator, the foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, supply-chain (Trivy / Cosign / SBOM), plugin substrate trust gates, plugin marketplace authoring, and the customer-facing bu

### ADR-16 residual

**ADR-0016-wave-and-plane-integration-framework** — We adopt **descriptive wave names**, **per-wave gate criteria**, **`preview / stable / GA` status labels**, and explicitly forbid `M0..M3 / minimum-shippable-tier` vocab (ADR-0018 enforces this in the glossary fitness lane). ### Wave names (canonical) | Wave | Description | |---|---| | **W-Foundation** | Foundation correctness: tenancy + identity kernel (ADR-0002), audit chain (ADR-0003), plane se

### ADR-324 residual

**ADR-0324-anti-script-anti-template-doctrine** — The following are CATEGORICALLY forbidden as authoring mechanisms for substantive content artifacts: - **AP-1** Shell loops (bash `for`, `while`) over filenames where the loop body writes a content file whose body is template-driven. - **AP-2** `jq` invocations that combine a constant template with a per- artifact substitution and write the result to disk as substantive content. - **AP-3** `awk`/`

### ADR-252 residual

**ADR-0252-time-coordination-distributed-consistency** — ### D-1. Hybrid Logical Clocks (HLC) as the default clock primitive The platform-wide canonical clock primitive is the **Hybrid Logical Clock** as defined in Demirbas + Kulkarni 2014 ("Logical Physical Clocks and Consistent Snapshot Isolation," OPODIS). Every µservice's kernel layer accepts an HLC parameter; every audit-chain entry carries an HLC timestamp; every saga step's persistence row carrie

### ADR-508 residual

**ADR-0508-opensk-canonical-authenticator-reference** — 1. **OpenSK is the canonical Phase-1 authenticator-side reference** for oyatie. It is the firmware substrate from which `oya-authn-device` will be forked and eventually replaced. 2. **Initial use (Phase-1, NOW–12mo)**: - Declare OpenSK reference metadata in `tools/opensk-vendored/README.md`, `tools/opensk-vendored/UPSTREAM-CONFIG.json`, and `tools/opensk-vendored/OWNERS` (follow-up implementation

### ADR-207 residual

**ADR-0207-accessibility-wcag-2-2-aa** — **WCAG 2.2 AA** is the minimum for every shipped surface. **AAA** for: - Healthcare surfaces under HIPAA (per `microservices/healthcare-portal/`). - EU AI Act high-risk surfaces (per `microservices/governance/` Annex III refusal). - Government / public-sector packs (per ADR-0064 per-pack overlay). ### Per-stack test runner table | Stack | Automated runner | Manual audit tool | |---|---|---| | Svel

### ADR-518 residual

**Bespoke SCM = the 10-stage AST work-area change pipeline (native-only, leases-not-locks); defines the deferred ADR-0510 ** — The bespoke SCM destination is a **10-stage hyperscaler change pipeline**: DECLARE → ADMIT → LEASE → ISOLATE(virtual) → AUTHOR → GATE(buck2 + AST gates + auto-remediate) → ATTEST → INTEGRATE → PROPAGATE(CD) → OBSERVE. It is Sapling / Mononoke / EdenFS / CommitCloud-inspired, owned in Rust, and **native-only**. The grit-essence claim/work/done model is re-framed as these native pipeline stages (no

### ADR-237 residual

**ADR-0237-connect-dissolution-strangler-migration** — The → 8-flat-µservice dissolution is migrated via the **Strangler Pattern** as defined in the agent-skills deprecation-and-migration skill (SKILL.md §"Strangler Pattern"). The migration proceeds through **6 sequential phases**, each gated by a concrete verification command. ### Phase 1 — New µservices ship in parallel *(current state, 2026-05-17)* `microservices/{mail,messenger,calendar}/` are sto

### ADR-541 residual

**Corpus Liveness Graph: one content-addressed corpus graph with per-class decay invariants** — Adopt the Corpus Liveness Graph (CLG) as the W2-milestone decay substrate, with the following binding shape: ### D1. One graph, owned substrate One content-addressed corpus graph: nodes at every granularity (ADR/doc → file → folder → module/crate → symbol → statement → token/format projection), edges derived by parse (never hand-maintained), faces CI-materialized (never hand-rotted; the ADR-0539 f

### ADR-393 residual

**Leptos canonical app-shell frontend (Rust/WASM SSR+hydration; supersedes ADR-0372 SolidJS)** — 1. **Leptos is the canonical app-shell / portal-shell frontend.** Full-stack Rust/WASM with **SSR + hydration** (`leptos` 0.8.x, `csr`/`hydrate`/`ssr` features as already present in `crates/oya-application-shell-frontend-prototype`). SolidJS is NOT a canonical target and is NOT retained as an evaluation track. 2. **Promote the Leptos prototype to the production portal-shell.** `crates/oya-applicat

### ADR-67 residual

**ADR-0067-ops-oyatie-com-hyperscaler-operations-console** — ### 1. µservice rename: `docs` → `ops` (catalog entry) Replace the `docs` µservice declared in ADR-0065 §2 with the parent µservice `ops`. The `docs` surface becomes one BC within `ops` (alongside dashboard / database / schema / tech-stack / architecture / health / tenant-mgmt / user-mgmt / observability / deployments / capacity / finops / on-call / incident / audit-view / ICM-browser / grit-statu

### ADR-622 residual

**Define a nonbinding FixupTask v2 successor foundation** — If separately accepted under qualified authority, the existing cloud-CI Rust lane could enforce a durable FixupTask v2 contract. This proposal does not amend or supersede ADR-0363, ADR-0515, ADR-0544, or ADR-0558 and does not create a binding lifecycle edge. The bounded design is: 1. A pure evaluator compares a protected merge-base snapshot with the candidate; only byte-identical legacy rows are g

### ADR-339 residual

**Shared IaC module library (`cloud/cloud-iac/modules/<context>/<primitive>/` is canonical; per-µservice `iac/<context>/ma** — ### B.1 Decision statement The canonical home for Oyatie reusable OpenTofu IaC primitives is `cloud/cloud-iac/modules/<context>/<primitive>/` where `<context>` is one of `{aws-guest, oci-guest, oci-guest/always-free, on-prem, colo, oyatie-as-cloud-provider}` and `<primitive>` is the canonical primitive name per §D-4 below. Every Oyatie µservice that declares an `iac/` directory ships **thin invoca

### ADR-60 residual

**ADR-0060-bominal-inheritance-precedence** — Two-tier precedence for all architectural decisions in oyatie: 1. **Default (lower precedence):** Adopt Bominal ADR architecture decisions 1:1, translating Bominal terminology to oyatie canonical glossary (per ADR-0018). 2. **Override (higher precedence):** Anything decided in the 2026-05-13 /deep-interview session overrides Bominal when they conflict. ### Glossary translation table (always applie

### ADR-63 residual

**ADR-0063-documentation-set-coverage** — ### 1. The canonical artifact set For every µservice registered in `[workspace.metadata.oya.microservices]`, the following artifacts MUST exist before the µservice's introducing-phase can pass its exit gate: | Artifact | Path convention | Template | |---|---|---| | Microservice record | `docs/microservices/<microservice>.md` | `docs/templates/microservice-template.md` | | Product Requirements (can

### ADR-62 residual

**ADR-0062-quality-performance-scalability-bar** — ### Quality bar — Industry Leaders Every µservice must benchmark against the industry leader for its domain before graduating from Proof-Ladder L4 → L5: | Dimension | Reference standard | |---|---| | API design | Stripe (REST/gRPC contracts, idempotency, pagination, error model) | | Data layer | Palantir Ontology (typed entities + provenance + audit) | | UI/UX craft | Linear / Stripe / Superhuman

### ADR-45 residual

**ADR-0045-database-tier-strategy** — We adopt **PostgreSQL + Citus** (Apache-2) as the canonical OLTP engine; **per-tenant per-cell shard topology**; **ClickHouse Apache-2 fork** as the canonical OLAP engine (with explicit fork-license verification per License Policy); **Iceberg + DataFusion** (Apache-2) as the canonical lakehouse format + query engine; backup orchestration per ADR-0040 release management; per-store retention + DSR c

### ADR-481 residual

**oya-flags: bespoke Rust feature flag server superseding flagd** — Replace flagd with **oya-flags**, a bespoke Rust feature flag server built on Axum for public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2, speaking the OpenFeature flag-evaluation protocol. ADR-0428 SDK adoption is preserved; only the server provider changes. - **Server**: `microservices/oya-flags/` — Rust and Axum. It exposes public HTTPS REST and implements the OpenFeature flag-evaluat

### ADR-122 residual

**ADR-0122-ontology-crate-rename-from-object-graph** — Rename: | Current crate name | New crate name | Reason | |---|---|---| | `oya-platform-object-graph-kernel` | `oya-ontology-kernel` | Match Bominal-ADR-0106 Ontology naming + `feedback_glossary_ontology_not_object_graph` | | (already correct: `oya-ontology-api`, `oya-ontology-domain`) | n/a | sanity-check — these already use `ontology` | Plus the planned-but-not-yet-scaffolded crates per `/specs/m

### ADR-217 residual

**ADR-0217-vertical-slice-rollout-order** — Plan all microservices, but promote production-GA claims only through service evidence and tenancy/RBAC packaging. A packaging axis is not "done" until the participating services reach hyperscaler-grade depth for workflows, compliance posture, integrations, import/export adapters, tenant controls, SLOs, runbooks, and audit evidence. Canonical packaging rollout order: | Order | Packaging axis | Rat

### ADR-235 residual

**ADR-0235-connect-core-public-contracts** — Accept the six contracts as **planning-stage public contracts** for core PRDs with these constraints: - Contract use is advisory until the corresponding implementation crates, contract schemas, and validators land. - All cross-product behavior must route through Workflow and Ontology mediation. Direct child-to-child calls are not allowed. - Any contract that can reveal personal/work state must pre

### ADR-371 residual

**ADR-0371-secure-control-plane-access-cloudflare-tunnel-access** — **Cloudflare Tunnel as an L4 TCP route, fronted by Cloudflare Access (Zero Trust):** 1. **Tunnel (L4 TCP).** A remotely-managed named tunnel (`oyatie-k8s`) with ingress `k8s.oyatie.dev -> tcp://10.211.55.240:6443`; a proxied DNS CNAME points the hostname at the tunnel. The in-cluster `cloudflared` connector dials **outbound only** — no inbound ports, no public IP needed (NAT is a non-issue). 2. **

### ADR-154 residual

**ADR-0154-event-schema-versioning** — Adopt explicit per-event `version` field as MANDATORY on every event emitted across every channel (WebSocket, AMQP, NATS, Kafka). 1. The canonical spec is `docs/standards/event-schema-versioning-canonical.md`. 2. Every AsyncAPI 3.1.0 message envelope MUST declare the `version` header and `event_id` (ULID per ADR-0156). 3. Backward-compatibility rules follow SemVer: - MINOR — additive (consumers to

### ADR-318 residual

**Adopt collar-color and workspace universality doctrine** — > Status: Proposed > Date: 2026-05-20 > Owner: council-architecture > Binding theme: one platform, many projections, no workforce forks.

### ADR-189 residual

**ADR-0189-step-up-authentication-acr-classes** — **Four ACR classes, named `routine`, `elevated`, `sensitive`, `critical`. Each declares min-factor count, accepted factor mix, max session age. Cedar policies attach an `acr_required` to every action; ext_authz returns `step_up_required` when the principal's ACR is below the floor. The OIDC ID-token carries `acr` as a string-enum claim per RFC 9068.** ### ACR enum | ACR | Factors required | Accept

### ADR-202 residual

**ADR-0202-gitops-iac-cluster-lifecycle-three-tier** — Three tools, three tiers, zero overlap. ### Tier A — GitOps app deployment: ArgoCD ArgoCD is the canonical Tier-A engine (Intuit-origin, donated to the CNCF; graduated 2022; Apache-2.0). Per ADR-0171 it is also the federation engine. - Owns: K8s app manifests, Helm releases, Kustomize overlays, ArgoCD `Application` and `ApplicationSet` CRs. - Does NOT own: cloud-side primitives (VPC, IAM, RDS-equi

### ADR-92 residual

**ADR-0092-workspace-dependency-seam-policy** — ### D1 — Canonical 12-layer enum The dependency-seam policy uses ADR-0056 v4.1's **canonical 12-value enum**: `{kernel, domain, application, adapter, infrastructure, cli, rest, grpc, graphql, worker, app, sdk}`. IP-002's 5-value enum is REJECTED as inconsistent with the canon. Layer is derived from the crate-name suffix per the BNF; no parallel `[package.metadata.oyatie.layer]` declaration is requ

### ADR-350 residual

**UUIDv7 canonical ID primitive across Oyatie** — ### D-1: UUIDv7 Is The Single Canonical ID Scheme UUIDv7 is the single canonical ID scheme for Oyatie. This applies to every ID surface. The rule covers event IDs. The rule covers audit-chain row IDs. The rule covers VCS changeset IDs. The rule covers tenant IDs. The rule covers cell IDs. The rule covers principal IDs. The rule covers resource IDs. The rule covers request IDs. The rule covers idem

### ADR-144 residual

**ADR-0144-eu-ai-act-graduated-risk-tier-model** — The risk classification is upgraded to a **5-tier graduated model** matching the EU AI Act 2024/1689 structure: | Tier | Risk class | EU AI Act anchor | Deployment status | |------|-----------------------|------------------------|-------------------| | 0 | Minimal-risk | Art. 50 (when AI-generated content is shown) | PERMITTED with Art. 50 disclosure | | 1 | Limited-risk | Art. 50 + Art. 52 (for G

### ADR-193 residual

**ADR-0193-olap-analytics-warehouse-clickhouse** — Oyatie adopts **ClickHouse 26.3 LTS** (Apache-2.0) as the canonical OLAP analytics warehouse for tenant-facing dashboards, telemetry rollups, audit-log query, and billing aggregation across the fleet. ClickHouse is deployed cell-locally; per-cell clusters are coordinator-free via ClickHouse Keeper (the Raft-based replacement for ZooKeeper). The `observability` µservice owns the cell-wide observabi

### ADR-212 residual

**ADR-0212-buildability-doctrine** — **Every artifact in this codebase MUST satisfy the per-kind bar below.** The buildability gate (advisory in PR #143; BLOCKER once 33+ µservices pass) verifies the bar mechanically; reviewer-agent verifies substance qualitatively. ### Per-artifact buildability bar | Artifact | Substance requirement | | --- | --- | | **PRD** | ≥5 user stories with measurable acceptance criteria; explicit scope-in /

### ADR-192 residual

**ADR-0192-vector-database-canonical-milvus** — Oyatie adopts **Milvus 2.6.x** (latest stable: 2.6.15 as of 2026-05-18; Apache-2.0; CNCF Graduated) as the canonical vector-database substrate fleet-wide. Milvus runs as a disaggregated cluster owned by the `intelligence` µservice (since embedding retrieval is a Intelligence AI-workload primitive) and is consumed by all µservices through the `oya-shared-vector-store-kernel` port. ### Cluster shape — disaggr

### ADR-506 residual

**ADR-0506-aws-lc-rs-canonical-crypto-provider** — 1. **aws-lc-rs is the canonical Phase-1 crypto backend** across all oyatie Rust services. It replaces `ring` as the workspace-level crypto primitive. 2. **rustls/hyper-rustls configured with the `aws-lc-rs` provider feature**: `features = ["ring", ...]` → `features = ["aws-lc-rs", ...]` on all `hyper-rustls` (and future `rustls`) dep declarations. 3. **Direct `ring` deps in prod code migrated to `

### ADR-153 residual

**ADR-0153: Outbox Pattern** — Adopt the transactional outbox pattern as the ONLY canonical way for a µservice to emit an event accompanying a state change. 1. The canonical spec is `docs/standards/outbox-pattern-canonical.md`. 2. The trait surface lives in `crates/oya-shared-outbox-pattern-kernel/`. 3. Every µservice with event-emission requirements creates one outbox table per bounded context. 4. The handler write-path append

### ADR-390 residual

**cloud-intelligence v1: request pipeline and proof layer** — ADR-0384 establishes the OAuth-pool kernel redesign for cloud-intelligence (formerly llm-gateway). That ADR specifies the kernel state machine (`SubscriptionPool`, `SeatLease`, `SeatOutcome`) and the OAuth token-refresh strategy.

### ADR-147 residual

**ADR-0147-container-sandboxing-runtime-ladder** — oyatie adopts a **workload-class-tiered container sandboxing runtime ladder**. The canonical mapping below replaces the universal-gVisor default: | Workload class | Default runtime | Sovereign-tenant override | |--------------------------------------------------------|------------------------------------------------------------------|-------------------------------------------| | App-tier µservice

### ADR-150 residual

**ADR-0150-cursor-pagination-canonical** — Adopt opaque cursor pagination as MANDATORY on every list endpoint in every µservice; offset pagination is BANNED. 1. The canonical spec is `docs/standards/cursor-pagination-canonical.md`. 2. The trait surface lives in `crates/oya-shared-cursor-pagination-kernel/`. 3. Every µservice OpenAPI 3.2.0 list path declares `cursor` + `page_size` parameters via the canonical `Cursor` + `PageSize` component

### ADR-48 residual

**ADR-0048-korean-morphology-and-multilingual-tokenization** — We adopt a **`Tokenizer` trait per language family** under `crates/oya-search-tokenizer-*`; **mecab-ko + khaiii via FFI day-1** (with mecab-ko legal-isolation analysis per License Policy + Apache-2 khaiii as the cleaner option for tenants who can use it); **in-house Rust port** of the KR morphology engine long-horizon; **per-pack tokenizer impl** for JP / ZH / EN / Indic / Arabic. ### `Tokenizer`

### ADR-238 residual

**ADR-0238-connect-super-app-expansion** — The legacy super-app is **dissolved** into first-class flat µservices and community-hosted posting modes, each owning one user-facing concern per ADR-0131: | µservice | Concern | Folder | Crate prefix (BNF v4.1) | |---|---|---|---| | `mail` | Email (SMTP/IMAP/JMAP, mailbox, search, retention, legal-hold, eDiscovery) | `microservices/mail/` | `oya-mail-*` | | `messenger` | Real-time messaging (chan

### ADR-610 residual

**Policy-IR benchmark stage-0: pre-registered frozen rubric + fixture suite as governed data** — Pre-register the stage-0 benchmark artifacts as frozen, machine-readable governed data: - `specs/policy-ir-benchmark-rubric.json` (`/specs/policy-ir-benchmark-rubric.json`) — the pre-registered grading rubric (`POL-IR-BENCH-RUBRIC`, `_meta.status: Frozen`), sole grade authority for the benchmark harvest matrix; amendments only via its embedded amendment log. - `specs/policy-ir-benchmark-fixture-su

### ADR-39 residual

**ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits** — We adopt **Trivy 4-layer scanning** (filesystem + container + IaC + dep) on every PR + nightly; **Cosign keyless signing** for every release artifact; **Rekor transparency log** for signature inclusion; **SBOM in SPDX 2.3 + CycloneDX 1.5** per artifact; **signed commits and tags** repo-wide; **merge-governance ruleset** at the GitHub level; **Kyverno (or OPA-equivalent) admission policy** at every

### ADR-64 residual

**ADR-0064-canonical-base-and-localization-packs** — ### 1. Canonical global base + three overlay forms Every customer-facing µservice has a **canonical global base** (jurisdiction-agnostic) and zero or more **localization overlays**. The overlay form is chosen per-concern — three forms exist, all valid: | Form | Definition | When to use | Naming (BNF v4.1) | |---|---|---|---| | **Seam** | A port (trait) inside the canonical base where a jurisdictio

### ADR-149 residual

**ADR-0149-idempotency-keys-canonical** — Adopt the canonical `Idempotency-Key` header as MANDATORY on every state-changing REST operation in every oyatie microservice. 1. The canonical specification is `docs/standards/idempotency-keys-canonical.md`. 2. The trait surface lives in `crates/oya-shared-idempotency-key-kernel/`. 3. Every µservice OpenAPI 3.2.0 document declares the canonical `IdempotencyKey` parameter component AND references

### ADR-308 residual

**ADR-0308-ml-model-lifecycle-ai-act-compliance** — ### §B. Eight-stage ML model lifecycle Establish the canonical eight-stage lifecycle for every ML model serving production traffic in the detection substrate (per ADR-0307) or any other product-facing AI surface: ``` ┌──────────────────────┐ │ 1. TRAINING │ │ - per-tenant data │ │ residency │ │ - cross-tenant │ │ consent │ │ - Iceberg snapshot │ └──────────┬───────────┘ │ ▼ ┌──────────────────────

### ADR-96 residual

**Supervisor language: Rust, not Node (build-vs-adopt Siigari/claude-heartbeat)** — **Build in Rust.** The upstream Node implementation is rejected as a runtime dependency.

### ADR-98 residual

**Supervisor dependency policy Branch Y — zero net-new external Cargo deps + best-effort durability** — **Branch Y — zero net-new external Cargo deps, sync I/O on tokio blocking pool, best-effort durability.** Concrete shape: ```rust // SessionDriver trait — synchronous; no async_trait dependency pub trait SessionDriver: Send + Sync { fn start_session(&self, ticket: &SessionTicket) -> Result<SessionHandle, SessionError>; fn stop_session(&self, handle: &SessionHandle) -> Result<(), SessionError>; fn

### ADR-542 residual

**Cloud-Intelligence XPROXY External-Proxy Parity Lane: commissioning and governance path** — Commission the XPROXY external-proxy parity lane under `cloud/cloud-intelligence/` with the following governance constraints: 1. **BNF-canonical crate naming**: all new crates carry a BNF role suffix from the approved registry (`kernel|domain|usecase|app|adapter|infrastructure|cli|rest|grpc|graphql|worker|sdk|api`). - `oya-cloud-intelligence-worker` (plural corrected; role: `worker` — K8s deployme

### ADR-109 residual

**Lifecycle-automation framework (generic kernel + per-lifecycle configs)** — 1. **One generic kernel.** `oya-governance-lifecycle-kernel` exposes the canonical `LifecycleConfig`, `LifecycledArtifact`, `Stage`, `Transition`, `Violation`, and `evaluate()` function. Every lifecycle lane is data — a JSON config under `specs/lifecycle-configs/`. Adding a new lifecycle is a config-file + thin dev-CLI commit, not a new kernel. 2. **Per-lifecycle dev-CLI wrappers.** Each lifecycle

### ADR-389 residual

**cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive** — Oyatie runs its own Talos-based substrate (ADR-0378). Cloud-intelligence v1 ships a pure OAuth-pool proxy (Anthropic + OpenAI + Gemini passthrough) via the 8-stage pipeline (ADR-0390). The open question is: how do we position the Bedrock Converse / InvokeModel surface as a **cloud primitive** — i.e., an abstraction that any tenant or internal workload can call without knowing which underlying prov

### ADR-382 residual

**ADR-0382-bare-metal-talos-zero-day-sidero** — ### Substrate: Sidero Metal **Choice**: Sidero Metal (github.com/siderolabs/sidero) as the bare-metal Talos provisioning layer; cluster-api-provider-sidero as the CAPI InfraProvider that maps `Cluster` / `Machine` to provisioned bare-metal nodes. **Hyperscaler-lens validation** (per memory `hyperscaler-lens-architectural-filter`): - **(a) Active upstream**: Sidero Labs releases quarterly; v0.7.x a

### ADR-221 residual

**ADR-0221-agentic-development-pipeline-hardening** — Adopt: 1. **Pre-dispatch validation templates** — dispatch briefs MUST carry §Audience + §Abstraction-rationale + §Catalog-collision-check. 2. **Per-step CI verification gates** — 4 new CI gates (below) detect the high-leverage mistake patterns. 3. **Doctrine intake automation** — decisions in conversation emit ADR scaffolds nightly so they don't live only in agent memory. 4. **PR-charter scope lo

### ADR-304 residual

**ADR-0304-cross-jurisdiction-conflict-resolution** — ### §B. Five core primitives at three layers The cross-jurisdiction conflict resolution baseline is **five core primitives** (data-residency hard-stop; per-tenant jurisdictional preference; per-pack regulator floor; higher-restriction-wins precedence; transparency-report) wired at **three layers** (Tier-0 shared crate, per-µservice gate, Cedar policy fragment). The 5×3 matrix produces fifteen cell

### ADR-619 residual

**Zero-live-context retirement of an external agent-harness brand** — ### 1. Absolute active-tree absence The current protected branch must contain zero case-insensitive occurrences of the forbidden token in tracked pathnames or tracked blob bytes. There are no exemptions for ADRs, audit evidence, archives, binary blobs, symlink payloads, fixtures, or generated projections. A born-blocking frozen-empty cloud-ci rule scans the complete tracked candidate tree before h

### ADR-362 residual

**ADR-0362-full-grouping-retirement-flat-only-catalog** — 1. **Flat single-concern µservices are the only architecture unit.** No grouping artifact of any kind — `suite`, `family`, `bundle`, `platform`, `vertical` — may exist as a spec, folder, or binding. This extends ADR-0132's prohibitions to *existing* artifacts, not just new ones. 2. **The grandfather clause is removed.** Existing grouping wrappers (`../tenant-rbac-packaging.json`, `tenant-rbac.json

### ADR-184 residual

**ADR-0184-storage-tier-layering** — Oyatie adopts a **four-tier storage layering** in which each tier owns exactly one access pattern: ### Tier 1 — OLTP write (PostgreSQL 18.4 primary) - Per-µservice Postgres 18.4 primary instance (one per bounded context; multi-tenant via row-level security). - Citus 14.0 for logical sharding by tenant where multi-tenant scale demands it (configured per-µservice; see `manifest.json` `lts_pins.citus

### ADR-388 residual

**Doc-axis convention to prevent doc sprawl** — ### Seven canonical doc axes | Axis | Canonical home | Auto-gen | Lifecycle rule | |---|---|---|---| | `DECISIONS` | `docs/decisions/ADR-NNNN-*.md` | no | Authoritative. Status field MUST be one of `Accepted`, `Amended`, `Proposed`, `Superseded`, `Deprecated`, or `Rejected` (exact case). | | `PLANS` | `docs/machine-readable/masterplan.generated.json` | yes (`oya gen masterplan`) | Derived from ADR

### ADR-61 residual

**ADR-0061-application-b2b-unified-shell** — We adopt **Application** as the name for the B2B unified shell. Application is a microservice in the flat catalog registered as `application` in `[workspace.metadata.oya.microservices]`. ### Core model Application implements the Bominal ADR-0121 model (inherited) with glossary translation: - Tenants sign in via the identity substrate (ADR-0002; Bominal ADR-0123 two-cookie + PKCE + nonce). - Tenant

### ADR-105 residual

**13-value canonical layer enum + check-family + backend-suffix patterns (amends ADR-0056)** — ### Amendment 1 — Extend the canonical enum from 12 to 13 values: add `api`. The 13-value canonical layer enum: | Group | Values | |---|---| | Inner / pure (4) | `kernel`, `domain`, `application`, `app` | | Outer / external (2) | `adapter`, `infrastructure` | | Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, **`api`** | **`api` semantics.** Protocol-neutral cont

### ADR-133 residual

**ADR-0133-industry-best-practice-conformance-program** — Adopt the 6-axis continuous industry-best-practice + hyperscaler-grade conformance program. Each axis carries: - **Industry baseline** (named primary sources) - **Audit cadence** (quarterly minimum; on-change for new µservices) - **Findings schema** (per `/specs/industry-best-practice-conformance.json`) - **Enforcement lane** (`oya-governance-industry-best-practice-conformance`; BLOCKER on dev) -

### ADR-22 residual

**ADR-0022-autonomy-ceiling-runtime-enforcement** — We enforce the autonomy ceiling at `oya-intelligence-policy-app` on **every** capability invocation. The effective ceiling is the minimum of four sources; agents inherit (and cannot exceed) tenant permissions; healthcare and fintech tenant classes force T1/T2 maxima for regulated capabilities; agentic ad-buying defaults to recommend-only. ### Effective-ceiling resolution ```rust // crates/oya-inte

### ADR-257 residual

**ADR-0257-ontology-object-type-versioning-deprecation-handshake** — ### D-1. Every Object Type schema carries a `schema_revision` (semver) **Rule.** Every `ObjectTypeSchema` registered in the `object-type-registry` BC carries a non-optional field `schema_revision: SemVer`. The field is part of the schema's wire shape; it appears in every Function read response, every Action receipt, every audit chain entry, every schema-evolution event, and every consumer pin. **F

### ADR-254 residual

**ADR-0254-deployment-model-spectrum** — ### D-1. The five deployment models The oyatie platform supports exactly five deployment models. The set is closed; additions require a new ADR. #### D-1.1. Shared-cloud (multi-tenant SaaS) **Definition.** oyatie operates the cell; multiple tenants share the cell via shuffle sharding (per ADR-0248); cell substrate is one of oyatie's contracted cloud providers per the cell's regional pack (per ADR-

### ADR-195 residual

**ADR-0195-stream-processing-tier** — ### Default: ClickHouse Materialized Views + Kafka Engine For class A workloads (~95%), the canonical stream-processing path is: 1. **Source.** Events land in the log-broker substrate (Apache Pulsar 4.2.x; supports Kafka wire protocol via Pulsar's Kafka-on-Pulsar proxy). 2. **Ingest.** ClickHouse `Kafka` engine connects to Pulsar's Kafka-protocol endpoint as a consumer. 3. **Materialized View.** `

### ADR-20 residual

**ADR-0020-intelligence-multi-provider-adapter-model** — We introduce a single normalized provider contract in `oya-intelligence-adapter-kernel` and wire every concrete provider through it. The runtime — not the capability author — chooses which adapter handles a given invocation. ### Trait surface (`oya-intelligence-adapter-kernel`) ```rust // crates/oya-intelligence-adapter-kernel/src/lib.rs pub trait ProviderAdapter: Send + Sync { fn id(&self) -> Pro

### ADR-209 residual

**ADR-0209-compliance-evidence-automation** — ### SOC 2 Type II continuous evidence pipeline Continuous evidence collectors emit to SeaweedFS + audit-chain seal: | Collector | Source | Cadence | |---|---|---| | CI artifact hash | every CI build (per ADR-0181) | per build | | Deploy receipt | every prod deploy | per deploy | | Access-review snapshot | Cedar policy + Zitadel role-bindings | weekly | | Backup restore drill receipt | per ADR-0180

### ADR-317 residual

**ADR-0317-role-based-projection-unified-ux-shell** — > **Disposition light-edit (2026-08-06):** Context re-triage Accept: Role-based projection + unified UX shell

### ADR-108 residual

**Sunset → deprecation → removal lifecycle automation schema (machine-readable)** — ### Machine-readable sunset schema Every sunset clause MUST be representable as a `SunsetClause` record with the following fields. The schema is identical across all three surfaces (ADR frontmatter YAML, spec JSON `_sunset` object, `[package.metadata.oya.sunset]` Cargo manifest section): | Field | Type | Required | Default | Description | |---|---|---|---|---| | `sunset_at` | RFC3339 date `YYYY-MM

### ADR-55 residual

**ADR-0055-object-graph-renamed-to-ontology** — All "Object Graph" terminology is renamed to "Ontology" in all oyatie artifacts. ### Scope of rename | Was | Now | Location | |---|---|---| | "Object Graph" | "Ontology" | All ADRs, plans, docs, code | | `oya-*-object-graph-*` | `oya-ontology-*` | All crates (Shard 1 atomic rename) | | `oya-platform-object-graph-kernel` | `oya-ontology-entity-kernel` | Primary kernel crate | | `oya-shared-object-g

### ADR-298 residual

**ADR-0298-emergency-services-bypass-life-safety** — ### §B. Decision summary **Decision 1: Three-layer Tier-0 edge bypass primitive.** Every internet-facing surface (Edge Gateway, API Gateway, AsyncAPI broker, SMTP MTA, SIP gateway, WebRTC gateway) routes through the canonical attestation-verifier at the Tier-0 edge. Verification results are forwarded as the `X-Oya-Emergency-Attestation` + `X-Oya-Emergency-Pack` headers (per the naming-justificatio

### ADR-568 residual

**born-accounting register_crate: the pure registrar kernel (RegisterCrateRequest → RegistrationPlan)** — Introduce **`libs/oya-crate-registrar-kernel`**: the PURE planner half of `register_crate` (G011 pipeline-as-product, slice 1). It composes a [`RegisterCrateRequest`] with a [`CurrentState`] snapshot of the born-accounting SSOTs and computes an ordered, typed [`RegistrationPlan`] — the set of edits that make a new crate fully born-accounted. It is a diff/upsert: re-planning against an already-regi

### ADR-397 residual

**Pulsar 4.x + Oxia canonical event-bus (reconstructed record)** — > **RECONSTRUCTION.** This record was cited before it was written. Seven governed surfaces > cite "ADR-0397 Pulsar 4.x + Oxia canonical event-bus" (ADR-0476, ADR-0478, ADR-0479, > ADR-0481, ADR-0482, ADR-0557, and `specs/master-plan-sequencing.json` wave 15-ZG), but no > decision file ever existed at this number — audit register H-19 > (`docs/audit/initial-sweep-2026-06-06/00-MASTER-CONTRADICTION-

### ADR-164 residual

**ADR-0164-sovereign-cloud-air-gapped** — Each sovereign pack declares `air_gap: true|false` in its pack manifest. When true, the following overlay applies: ### (a) On-prem container registry - Each cell deploys Harbor 2.x (CNCF graduated) as its in-cell container registry. - Image pull policy: `imagePullPolicy: IfNotPresent` + image references are rewritten to `registry.{cell}.svc.cluster.local/oya/<ms>:<tag>`. - A pre-flight job (per-ce

### ADR-24 residual

**ADR-0024-intelligence-eval-harness-and-replay** — Every capability publishes with a golden eval set; the eval harness gates publish, runs nightly, runs A/B against routing decisions, and replays against past production traces for regression detection. Adversarial and regional linguistic cohorts are mandatory. ### Eval kernel (`oya-intelligence-eval-kernel`) ```rust // crates/oya-intelligence-eval-kernel/src/lib.rs pub struct EvalSet { pub capabil

### ADR-69 residual

**ADR-0069-active-machine-readable-artifact-contract** — Adopt the **active machine-readable artifact contract** v3.0.0 with three load-bearing artifacts and one validator crate. The contract is format-agnostic (applies to JSON, TOML, YAML, Cedar, SQL, OpenAPI, GitHub Actions YAML, Cargo.toml, etc.) and registry-based (control plane in registry; data plane in artifacts). ### Components | Component | Path | Role | |---|---|---| | Contract schema | `/spec

### ADR-272 residual

**ADR-0272-cookie-consent-per-purpose-analytics-opt-in** — > **Disposition light-edit (2026-08-06):** Cookie consent / purpose analytics — privacy substrate

### ADR-239 residual

**ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18** — - **Status:** Accepted (amendment) - **Date:** 2026-05-18 - **Owner:** council-architecture - **Amends:** ADR-0136 (Intelligence consolidation 6→1) - **Related:** ADR-0220 (Consumer Intelligence Substrate — `microservices/intelligence/`) - **PR:** #143 close-out

### ADR-555 residual

**Unaccounted artifacts are unmergeable: advisory→blocking accounting conversion + the structural accounting model** — ### D1 — Convert the exists-but-unaccounted codes to blocking, grandfathered at the merge-base The disposition table (`libs/oya-ci-config/src/bundled/gate-disposition.json` — DATA, not code) flips: | gate | code | was | now | live keys grandfathered | |---|---|---|---|---| | cloud-ci-total-accounting | `unowned` | advisory-until-infra | **baseline-block-on-new** | 16,924 (pre-seed) | | cloud-ci-to

### ADR-8 residual

**ADR-0008-data-use-boundary** — We adopt the **Data Use Boundary** as the contract that governs which tenant data may flow across axes under what consent, for what purpose, to what subject class, in what jurisdiction. The boundary is enforced at six structural layers (compile-time first), uses an *orthogonal* subject-class attribute (not a 13th data class), uses a *purpose-permission matrix* (not a linear ladder), and uses a *fo

### ADR-151 residual

**ADR-0151-request-id-propagation** — Adopt the canonical `X-Request-Id` header (ULID) propagated alongside OpenTelemetry `traceparent` on every inter-µservice call. 1. The canonical spec is `docs/standards/request-id-canonical.md`. 2. Every µservice's edge middleware GENERATES a fresh ULID if the header is absent, and PROPAGATES it on every outbound call. 3. Every µservice's outbound HTTP/gRPC client adapter INJECTS the request-id on

### ADR-478 residual

**oya-billing — bespoke Rust billing engine superseding Lago** — Build `oya-billing` — a bespoke Rust billing engine — as the canonical billing plane. ### D1 — New µservice `microservices/oya-billing/` Rust workspace. Axum serves the public HTTPS REST/OpenAPI surface; internal-only gRPC/proto3 over HTTP/2 serves sibling services. PostgreSQL (ADR-0406) for durable billing state (subscriptions, invoices, line items, credits). Apache Pulsar (ADR-0397) for billable

### ADR-18 residual

**ADR-0018-glossary-and-terminology-canon** — We adopt the **glossary canon** with five rules, an industry-aligned vocabulary list, Oyatie-specific terms, a Korean-English parity table, and a CI lane that hard-fails forbidden tokens. ### The five vocabulary rules 1. **Industry-standard term wins** when one exists and is unambiguous. 2. **Oyatie-specific term** is reserved for genuinely new concepts or for renames the brand has explicitly chos
