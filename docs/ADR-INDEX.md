# Oyatie — ADR Index

> **Updated:** 2026-05-13 (ADR partition audit + rewrite)
> **Authoritative:** `crew-adr-promotion` owns freshness per [DOC-CATALOG.md](DOC-CATALOG.md).
> **Machine-readable mirror:** [`machine-readable/decisions.json`](machine-readable/decisions.json).

## At-a-glance

- **Total ADRs:** 55
- **Numbering:** ADR-0001 .. ADR-0062 (ADR-0012 deleted; ADR-0033 deleted)
- **Next ADR number:** 0063
- **Status counts:** Accepted 10, Proposed 45
- **Canonical glossary:** `shared` (not platform), `Ontology` (not Object Graph), `Application` (not Shell), `microservice` (not vertical/arm/product-group), `flat catalog` (not Arms), `Connect` (not Workspace-product)

## Full table

| ADR | Status | Title | Owner | File |
|---|---|---|---|---|
| ADR-0001 | Accepted | Adopt the cohesion thesis — one product across a flat microservice catalog joined at six shared substrates | council-architecture | [`ADR-0001-cohesion-thesis-one-product-flat-catalog.md`](decisions/ADR-0001-cohesion-thesis-one-product-flat-catalog.md) |
| ADR-0002 | Proposed | Establish the Tenant and Identity kernel as the single substrate every microservice consumes | council-architecture | [`ADR-0002-tenant-and-identity-kernel.md`](decisions/ADR-0002-tenant-and-identity-kernel.md) |
| ADR-0003 | Proposed | Audit chain and evidence emission as the single tamper-evident record-keeping substrate | council-architecture | [`ADR-0003-audit-chain-and-evidence-emission.md`](decisions/ADR-0003-audit-chain-and-evidence-emission.md) |
| ADR-0004 | Proposed | Plane separation across control / data / analytics with catalog-declared plane class | council-architecture | [`ADR-0004-plane-separation-control-data-analytics.md`](decisions/ADR-0004-plane-separation-control-data-analytics.md) |
| ADR-0005 | Proposed | Eventing backbone on Apache Kafka with outbox pattern, CloudEvents envelope, and per-tenant per-cell partitioning | council-architecture | [`ADR-0005-eventing-backbone-outbox-pattern.md`](decisions/ADR-0005-eventing-backbone-outbox-pattern.md) |
| ADR-0006 | Accepted | Ontology as the engine-enforced typed-entity layer with per-property tier classification | oya-ontology | [`ADR-0006-ontology-typed-entity-layer.md`](decisions/ADR-0006-ontology-typed-entity-layer.md) |
| ADR-0007 | Proposed | Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling (T1–T4) with per-capability runtime enforcement | council-architecture | [`ADR-0007-cedar-authorization-policy-and-persona-tier.md`](decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md) |
| ADR-0008 | Proposed | Data Use Boundary — twelve data classes with HARD_DENY for PHI/PCI/PIPA-Art23/CHILDREN, orthogonal subject_class, purpose-permission matrix, and four-pillar flow matrix | council-privacy | [`ADR-0008-data-use-boundary.md`](decisions/ADR-0008-data-use-boundary.md) |
| ADR-0009 | Proposed | Cell architecture — per-tenant per-region blast-radius cells with cell-routing primitives at edge / mesh / store / event | council-architecture | [`ADR-0009-cell-architecture-per-tenant-per-region.md`](decisions/ADR-0009-cell-architecture-per-tenant-per-region.md) |
| ADR-0010 | Proposed | Regional pack architecture — canonical seams + per-locale plug-ins for regulatory, compliance, i18n, currency, calendar, tax, identity, payment, address, ecosystem partners, content safety, ad policy, industry data models, and vendor partners | council-architecture | [`ADR-0010-regional-pack-architecture.md`](decisions/ADR-0010-regional-pack-architecture.md) |
| ADR-0011 | Accepted | Cross-microservice contract registry — `contracts/microservice-contracts.yaml` source-of-truth, openapi/proto/asyncapi sub-directories, oya-check-contracts CI lane, cross-microservice contract change protocol, auto-generated SDKs | council-architecture | [`ADR-0011-cross-microservice-contract-registry.md`](decisions/ADR-0011-cross-microservice-contract-registry.md) |
| ADR-0013 | Proposed | Product license policy — allowed (Apache-2 / MIT / BSD-2/3 / ISC / 0BSD / MPL-2 / Unicode), forbidden in product code (AGPL / GPL), requires-review tier (LGPL / SSPL / BUSL / Elastic / RSAL / TSL / Confluent / AWS-FSL / Commons Clause), dev-only carve-out, oya-check-license CI lane, per-release SBOM | council-architecture | [`ADR-0013-product-license-policy.md`](decisions/ADR-0013-product-license-policy.md) |
| ADR-0014 | Proposed | Build-vs-buy policy — per-microservice matrix (in-house obligatory / external acceptable / requires-review), decision flow chart, per-dep metadata, oya-check-build-vs-buy CI lane | council-architecture | [`ADR-0014-build-vs-buy-policy.md`](decisions/ADR-0014-build-vs-buy-policy.md) |
| ADR-0015 | Accepted | Architectural flattening target — flat-crates `crates/oya-*`, role taxonomy, dep-direction kernel←domain←app←api/worker/adapter, boundary validator, migration path from legacy modules / services / platform tree | council-architecture | [`ADR-0015-architectural-flattening-target.md`](decisions/ADR-0015-architectural-flattening-target.md) |
| ADR-0016 | Proposed | Wave and plane integration framework — descriptive wave names, per-wave gate criteria, status labels (preview / stable / GA), no M0/M1/M2/M3/MVP vocab | council-architecture | [`ADR-0016-wave-and-plane-integration-framework.md`](decisions/ADR-0016-wave-and-plane-integration-framework.md) |
| ADR-0017 | Accepted | Brand naming and repo layout — Oyatie / oYa logo / oyatie.com domain, oya-\<microservice\>-\<layer\> Cargo prefix per BNF v4.1, repo path / GitHub slug oyatie retained | council-architecture | [`ADR-0017-brand-naming-and-repo-layout.md`](decisions/ADR-0017-brand-naming-and-repo-layout.md) |
| ADR-0018 | Accepted | Glossary and terminology canon — canonical glossary with forbidden terms (Object Graph, Workspace-product, Shell, vertical, arm, Product Group, platform-as-substrate, \<shared\|vertical\> BNF), Korean-English parity table, oya-check-glossary CI lane | council-architecture | [`ADR-0018-glossary-and-terminology-canon.md`](decisions/ADR-0018-glossary-and-terminology-canon.md) |
| ADR-0019 | Proposed | Doc catalog and update protocol — every consolidated doc has owner / trigger / cadence / dependent-docs / validation; pre-flight + authoring + validation + review + publish stages | council-architecture | [`ADR-0019-doc-catalog-and-update-protocol.md`](decisions/ADR-0019-doc-catalog-and-update-protocol.md) |
| ADR-0020 | Proposed | Foundry multi-provider adapter model — `ProviderAdapter` trait, ProviderAuth, capability-level routing | council-foundry | [`ADR-0020-foundry-multi-provider-adapter-model.md`](decisions/ADR-0020-foundry-multi-provider-adapter-model.md) |
| ADR-0021 | Proposed | Foundry capability registry and MCP gateway — `Capability` schema, MCP-compatible discovery, per-tenant endpoint | council-foundry | [`ADR-0021-foundry-capability-registry-and-mcp-gateway.md`](decisions/ADR-0021-foundry-capability-registry-and-mcp-gateway.md) |
| ADR-0022 | Proposed | Autonomy ceiling — runtime enforcement via Cedar policy at every capability invocation | council-foundry | [`ADR-0022-autonomy-ceiling-runtime-enforcement.md`](decisions/ADR-0022-autonomy-ceiling-runtime-enforcement.md) |
| ADR-0023 | Proposed | Foundry sandbox — Wasmtime + WASI Preview 2 for short-lived tools, Firecracker microVMs for full-kernel tools | council-foundry | [`ADR-0023-foundry-sandbox-wasmtime-firecracker.md`](decisions/ADR-0023-foundry-sandbox-wasmtime-firecracker.md) |
| ADR-0024 | Proposed | Foundry eval harness and replay — per-capability golden sets, A/B routing, adversarial cohorts, regional linguistic eval | council-foundry | [`ADR-0024-foundry-eval-harness-and-replay.md`](decisions/ADR-0024-foundry-eval-harness-and-replay.md) |
| ADR-0025 | Proposed | Foundry as the engineering platform — repoctl, catalog, gates, fitness functions, supply chain, customer-facing builder surfaces all under one microservice | council-foundry | [`ADR-0025-foundry-as-engineering-platform.md`](decisions/ADR-0025-foundry-as-engineering-platform.md) |
| ADR-0026 | Proposed | In-house AI model substrate — long-horizon; consume providers until per-microservice eval suite favors in-house | council-foundry | [`ADR-0026-in-house-ai-model-substrate-roadmap.md`](decisions/ADR-0026-in-house-ai-model-substrate-roadmap.md) |
| ADR-0027 | Proposed | Robotics, vision, and speech sub-substrates — vision/speech model crates, robotics control plane, deterministic latency, safety-critical anti-scope | council-foundry | [`ADR-0027-robotics-vision-speech-sub-substrates.md`](decisions/ADR-0027-robotics-vision-speech-sub-substrates.md) |
| ADR-0028 | Accepted | Cloud microservice — compute substrate with stable product surface across three infrastructure phases | oya-cloud | [`ADR-0028-cloud-microservice-architecture.md`](decisions/ADR-0028-cloud-microservice-architecture.md) |
| ADR-0029 | Accepted | Connect microservice — dual-context communications (Professional + Personal) as cohesion-bound replacement for Google Workspace / M365 / Naver Works / Kakao Work | oya-connect | [`ADR-0029-connect-dual-context-architecture.md`](decisions/ADR-0029-connect-dual-context-architecture.md) |
| ADR-0030 | Accepted | Search microservice — crawler / parser / index / ranker / SERP architecture with KR-first morphology and Data-Use-Boundary segregation | oya-search | [`ADR-0030-search-microservice-architecture.md`](decisions/ADR-0030-search-microservice-architecture.md) |
| ADR-0031 | Accepted | Ads + Analytics microservice — singleton tenant-ads-gate sourcing, sub-100ms auction, privacy-preserving attribution, Data-Use-Boundary at runtime | oya-ads | [`ADR-0031-ads-and-analytics-microservice-architecture.md`](decisions/ADR-0031-ads-and-analytics-microservice-architecture.md) |
| ADR-0032 | Proposed | DCIM software for Oyatie-owned DC operations — `oya-cloud-dcops-*` with anti-scope on custom silicon | oya-cloud | [`ADR-0032-dcim-software-for-own-dc-ops.md`](decisions/ADR-0032-dcim-software-for-own-dc-ops.md) |
| ADR-0034 | Accepted | Per-microservice data class overrides — microservice-side hard-deny pack that tenant admin cannot raise | council-architecture | [`ADR-0034-per-microservice-data-class-overrides.md`](decisions/ADR-0034-per-microservice-data-class-overrides.md) |
| ADR-0035 | Proposed | Workflow engine — hybrid state-machine + DAG (not pure BPMN), per-tenant versioning, jurisdiction overlay, agent-authored steps | council-architecture | [`ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md`](decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md) |
| ADR-0036 | Proposed | Plugin substrate — Wasmtime + WASI Preview 2 with capability-gated context, Cosign signing, trust tiers, marketplace economics | council-foundry | [`ADR-0036-plugin-substrate-wasm-and-trust.md`](decisions/ADR-0036-plugin-substrate-wasm-and-trust.md) |
| ADR-0037 | Proposed | Public API stability tiers — preview / stable / GA with semver-diff PR gate, contract-first SDK generation, per-deprecation telemetry | council-architecture | [`ADR-0037-public-api-stability-tiers-and-deprecation.md`](decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md) |
| ADR-0038 | Proposed | Trust framework — cross-microservice lineage, DSR cascade across all microservices, Cosign-signed proof-of-erasure, tenant trust portal | council-architecture | [`ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md`](decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md) |
| ADR-0039 | Proposed | Supply chain security — Trivy 4-layer scan, Cosign keyless signing, SBOM dual-format, signed commits and tags, Kyverno admission | council-foundry | [`ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md`](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) |
| ADR-0040 | Proposed | Progressive delivery — Argo Rollouts canary, blue-green for stateful surfaces, metric-gated rollback at SLO burn-rate ≥ 14.4× | council-foundry | [`ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md`](decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) |
| ADR-0041 | Proposed | GitOps — trunk-based development with release branch cut at tag, merge queue with one-PR-at-a-time root-Cargo-touch | council-foundry | [`ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md`](decisions/ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md) |
| ADR-0042 | Proposed | Observability stack — OpenTelemetry SDK + VictoriaMetrics, in-house Leptos portal long-horizon, gen_ai semconv per capability | council-foundry | [`ADR-0042-observability-stack-otel-and-in-house-ui.md`](decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md) |
| ADR-0043 | Proposed | Secrets management — OpenBao (MPL-2; supersedes Vault BUSL), per-tenant per-cell HSM partition (KCMVP + FIPS 140-3), per-capability SecretProvider | council-foundry | [`ADR-0043-secrets-management-openbao-and-hsm-per-cell.md`](decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md) |
| ADR-0044 | Proposed | Service mesh — Istio Ambient mode for east-west, Envoy as edge gateway, mTLS everywhere, per-cell namespace, audited cross-cell traffic | oya-cloud | [`ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md`](decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md) |
| ADR-0045 | Proposed | Database tier strategy — PostgreSQL + Citus for OLTP, ClickHouse-fork for OLAP, Iceberg + DataFusion for lakehouse | council-architecture | [`ADR-0045-database-tier-strategy.md`](decisions/ADR-0045-database-tier-strategy.md) |
| ADR-0046 | Proposed | Vector store strategy — pgvector day-1, in-house Rust HNSW/IVF at billion-scale long-horizon, FAISS only as adapter | council-architecture | [`ADR-0046-vector-store-strategy.md`](decisions/ADR-0046-vector-store-strategy.md) |
| ADR-0047 | Proposed | Search backend strategy — pgroonga day-1 (LGPL legal isolation), Tantivy in-Rust at scale, OpenSearch as Apache-2 adapter, in-house long-horizon | oya-search | [`ADR-0047-search-backend-strategy.md`](decisions/ADR-0047-search-backend-strategy.md) |
| ADR-0048 | Proposed | Korean morphology + multilingual tokenization — `Tokenizer` trait per language family, mecab-ko + khaiii FFI day-1, in-house Rust port long-horizon | oya-search | [`ADR-0048-korean-morphology-and-multilingual-tokenization.md`](decisions/ADR-0048-korean-morphology-and-multilingual-tokenization.md) |
| ADR-0049 | Proposed | Cross-region replication + residency — per-pack default residency class, opt-in cross-region per consent, immutable post-create | council-architecture | [`ADR-0049-cross-region-replication-and-residency.md`](decisions/ADR-0049-cross-region-replication-and-residency.md) |
| ADR-0050 | Proposed | Automation-first pipeline — Google + Amazon doctrine, sccache + remote execution, affected-graph testing, Foundry-driven PR triage | council-foundry | [`ADR-0050-automation-first-pipeline.md`](decisions/ADR-0050-automation-first-pipeline.md) |
| ADR-0051 | Accepted | Mobile and Native Client Strategy | council-architecture | [`ADR-0051-mobile-and-native-client-strategy.md`](decisions/ADR-0051-mobile-and-native-client-strategy.md) |
| ADR-0052 | Accepted | Canonical inventory ledger for the grit/icm cutover — classifies 211 artifacts across oyatie/ and bominal/ by closed-set action (KEEP/ARCHIVE/DELETE/FLAG-FOR-USER) | council-architecture | [`ADR-0052-inventory-grit-cutover.md`](decisions/ADR-0052-inventory-grit-cutover.md) |
| ADR-0053 | Accepted | grit + icm + oya-tooling-agent-read as sole sanctioned primitives for agentic work | council-architecture | [`ADR-0053-grit-icm-as-sanctioned-primitives.md`](decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md) |
| ADR-0054 | Accepted | Resolve new-crate chicken-and-egg via grit scaffold-claim pattern (icm-coordination-lock fallback) | council-architecture | [`ADR-0054-grit-scaffold-claim-pattern.md`](decisions/ADR-0054-grit-scaffold-claim-pattern.md) |
| ADR-0055 | Accepted | Object Graph renamed to Ontology — all oya-\*-object-graph-\* crates, ADR-0006, docs, plans use "Ontology" going forward | council-architecture | [`ADR-0055-object-graph-renamed-to-ontology.md`](decisions/ADR-0055-object-graph-renamed-to-ontology.md) |
| ADR-0056 | Accepted | Rust Clean Architecture BNF v4.1 — flat microservice grammar `oya-<microservice>-(<bc>-)?<layer>` + 12-layer enum; `<shared\|vertical>` slot2 enum retired | council-architecture | [`ADR-0056-rust-clean-architecture-bnf.md`](decisions/ADR-0056-rust-clean-architecture-bnf.md) |
| ADR-0057 | Accepted | Cutover Mechanics — Rename Plan v4 (Hybrid C): Shard 0 pure-tooling precursor + atomic Shard 1 rename | council-architecture | [`ADR-0057-cutover-mechanics-rename-plan-v4.md`](decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md) |
| ADR-0058 | Accepted | Flat microservice catalog — Product Groups, Arms, and Verticals retired; every product/feature is an independent microservice in a flat catalog | council-architecture | [`ADR-0058-flat-microservice-catalog.md`](decisions/ADR-0058-flat-microservice-catalog.md) |
| ADR-0059 | Accepted | Workflow + Ontology = ecosystem adapter layer — all cross-microservice integration flows through Workflow (action) or Ontology (data); no direct microservice-to-microservice calls | council-architecture | [`ADR-0059-workflow-ontology-ecosystem-adapter-layer.md`](decisions/ADR-0059-workflow-ontology-ecosystem-adapter-layer.md) |
| ADR-0060 | Accepted | Bominal-inheritance precedence — default inherit Bominal ADRs 1:1 with glossary translation; 10 locked overrides from 2026-05-13 session | council-architecture | [`ADR-0060-bominal-inheritance-precedence.md`](decisions/ADR-0060-bominal-inheritance-precedence.md) |
| ADR-0061 | Accepted | Application — B2B unified shell with à-la-carte product enablement; tenants enable products from flat catalog like AWS console; Connect Personal is separate entry path | council-architecture | [`ADR-0061-application-b2b-unified-shell.md`](decisions/ADR-0061-application-b2b-unified-shell.md) |
| ADR-0062 | Accepted | Quality/Performance/Scalability bar — industry leaders + hyperscaler scale; competitive benchmark + perf targets + horizontal scalability mandatory in every µservice PRD and impl plan | council-architecture | [`ADR-0062-quality-performance-scalability-bar.md`](decisions/ADR-0062-quality-performance-scalability-bar.md) |

## Deleted ADRs (2026-05-13 audit)

The following ADRs were deleted because the decisions themselves are superseded by session decisions; stale content is removed rather than marked retired:

| ADR | Reason |
|---|---|
| ADR-0012 (Axis Admission Protocol) | "Axes" framing retired; flat microservice catalog (ADR-0058) replaces axis admission |
| ADR-0033 (Vertical industry cloud pack architecture) | "Vertical" as architectural grouping retired; microservices in flat catalog (ADR-0058) + per-microservice data overrides (ADR-0034) replace this |

## Update protocol

- Per-event + monthly per `doc.adr_index` row in [`DOC-CATALOG.md`](DOC-CATALOG.md).
- New ADRs land via [`templates/adr-template.md`](templates/adr-template.md) and contiguous numbering (next available: 0063).
- Per-ADR amendments preserve the original ADR number.

## Sources scanned

- `decisions/` directory — 55 ADR files (2026-05-13)
- Session decisions 2026-05-13 `/deep-interview` — 10 locked overrides applied
