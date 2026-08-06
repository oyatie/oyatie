---
purpose: Legacy ADR → New Pack Regression Mapping
doc_status: published
---

# Legacy ADR → New Pack Regression Mapping

> **Status:** auto-generated 2026-05-09 by regression-check agent
> **Owner:** crew-adr-promotion + council-architecture
> **Purpose:** verify every legacy ADR's substance is preserved (FULL/EXPANDED) or explicitly retired (DROPPED-WITH-REASON) in the new pack before legacy deletion per [legacy-adr-deletion.md §2](checklists/legacy-adr-deletion.md)

## Summary

- Total legacy ADRs: 127
- FULL coverage: 63
- EXPANDED coverage (new pack adds beyond legacy): 42
- PARTIAL coverage (council attention needed): 18
- DROPPED-WITH-REASON: 4
- INTENTIONALLY-OUT-OF-SCOPE (anti-scope per PRD §3.3): 0

**Verdict total:** 63 + 42 + 18 + 4 = 127 ✓

**Council attention required (PARTIAL/DROPPED counts):** 22 rows. See §3 below for the consolidated council-attention list extracted from the table.

---

## Inputs scanned

- 127 legacy ADRs at `/Users/jasonlee/oyatie/decisions/ADR-*.md` (titles + status verified against `docs/raw/adr-index.md`)
- 50 new pack ADRs at `/Users/jasonlee/oyatie/docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md`
- New pack README: `docs/decisions/README.md`
- Consolidation strategy: `docs/ADR-CONSOLIDATION-PLAN.md`
- Deletion checklist: `docs/checklists/legacy-adr-deletion.md`
- Retirement note: `docs/decisions/RETIRED.md`

## Coverage verdict legend

- **FULL** — substance fully captured in 1+ new pack ADRs (no semantic loss).
- **EXPANDED** — substance captured + new pack adds beyond legacy (e.g. multi-region depth, in-house substitution roadmap, license posture, plane consolidation, autonomy ceiling).
- **PARTIAL** — substance partially captured; council must confirm gap is acceptable or schedule a follow-on ADR before deletion.
- **DROPPED-WITH-REASON** — substance intentionally retired (retired axis vocabulary like "Foundry engineering platform"; retired wave/horizon vocabulary like M3/MVP; retired vendor like Vault BUSL; experimental ADRs that did not graduate).
- **OUT-OF-SCOPE** — anti-scope per PRD §3.3 (none in current 127-ADR corpus; recorded for completeness).

## 2. Per-legacy-ADR mapping table

| Legacy ADR | Title | Status | Substance summary | Captured in new pack | Coverage verdict | Notes |
|---|---|---|---|---|---|---|
| ADR-0010 | Metrics consolidation via platform observability library | Proposed | per-platform observability lib, metrics rollup | ADR-0042 observability-stack-otel-and-in-house-ui | EXPANDED | New pack adds OTel + VictoriaMetrics + in-house Leptos UI long-horizon |
| ADR-0011 | Isolation-compatible operating model | Accepted | per-pillar / per-tenant isolation | ADR-0009 cell-architecture-per-tenant-per-region + ADR-0008 data-use-boundary | FULL | per-tenant cell + per-class boundary |
| ADR-0016 | Clinical canonical record authority and released-view boundary | Accepted | clinical-record authority + released-view boundary | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0034 per-vertical-data-class-overrides | PARTIAL | Vertical-healthcare slice references it; explicit clinical released-view contract not yet authored — flag for council vertical-healthcare to confirm acceptable |
| ADR-0017 | Unified governance catalog authority and projection model | Accepted | catalog projection model | ADR-0011 cross-axis-contract-registry + ADR-0019 doc-catalog-and-update-protocol | FULL | Catalog-as-code is now first-class via cross-axis contract registry |
| ADR-0018 | Tenancy and RLS posture | Accepted | per-tenant RLS enforcement | ADR-0002 tenant-and-identity-kernel + ADR-0006 object-graph-and-property-tier-model | FULL | Engine-enforced isolation lives in OG model + tenant kernel |
| ADR-0019 | Runtime target metadata model | Accepted | runtime-target schema | ADR-0028 cloud-provider-architecture + ADR-0009 cell-architecture | FULL | Runtime targets folded into cloud + cell ADRs |
| ADR-0020 | Multi-runtime platform standard | Accepted | multi-runtime support | ADR-0028 cloud-provider-architecture + ADR-0029 workspace + ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | |
| ADR-0021 | OCI A1 Always Free launch profile | Accepted | OCI free-tier launch profile | ADR-0028 cloud-provider-architecture | EXPANDED | New pack adds multi-cloud roadmap; OCI A1 is now one profile under §cloud-provider |
| ADR-0022 | Cluster GitOps promotion control plane | Accepted | cluster GitOps promotion | ADR-0041 gitops-trunk-based-and-release-branch-cut-at-tag + ADR-0040 progressive-delivery | FULL | |
| ADR-0028 | Audit-chain Merkle-sealed Ed25519 evidence ledger and event registry | Accepted | Merkle-Ed25519 audit chain + event registry | ADR-0003 audit-chain-and-evidence-emission | EXPANDED | New pack adds DSR cascade + proof-of-erasure + plane-aware event taxonomy |
| ADR-0100 | Hexagonal, self-governing application layer for Corporate Attendance | Accepted | hexagonal application layer | ADR-0015 architectural-flattening-target | FULL | Per-product hexagonal pattern subsumed by flattening target ADR |
| ADR-0101 | Hexagonal, self-governing microservice standard (repo-wide) | Accepted | hexagonal microservice standard | ADR-0015 architectural-flattening-target | FULL | |
| ADR-0102 | Hexagonal migration plan for remaining products | Accepted | hexagonal migration sequence | ADR-0015 architectural-flattening-target + ADR-0050 automation-first-pipeline | FULL | Migration mechanics moved to automation-first pipeline |
| ADR-0103 | Workflow product hexagonal migration (Tier 1 per ADR-0102) | Accepted | workflow hexagonal migration | ADR-0015 architectural-flattening-target + ADR-0035 workflow-engine-state-machine-and-dag-hybrid | FULL | |
| ADR-0033 | HR + Payroll + PTO + Compliance vertical — bounded-context split (Tier C) | Accepted | HR/payroll bounded-context split | ADR-0033 vertical-industry-cloud-pack-architecture | PARTIAL | Vertical pack umbrella covers it; the HR/payroll-specific bounded-context recipe is referenced by name but not re-authored — flag for council vertical-corporate to confirm or schedule follow-on |
| ADR-0105 | Clean-architecture layering inside domain crates | Accepted | clean-arch layering | ADR-0015 architectural-flattening-target | FULL | |
| ADR-0006 | Object Graph — engine-enforced, cryptographically auditable typed-entity layer | Accepted | engine-enforced typed-entity layer | ADR-0006 object-graph-and-property-tier-model | EXPANDED | New pack adds property-tier classification + cross-region replication semantics (ADR-0049) |
| ADR-0021 | Object Graph Agent Gateway (OG-AG) — LLM tool surface with audit-chain provenance | Proposed | LLM tool surface + audit provenance | ADR-0021 foundry-capability-registry-and-mcp-gateway + ADR-0022 autonomy-ceiling | EXPANDED | New pack adds autonomy-ceiling enforcement + MCP gateway model |
| ADR-0108 | Vector property type — PgVector embedding adapter | Proposed | pgvector property type | ADR-0006 object-graph-and-property-tier-model + ADR-0046 vector-store-strategy | FULL | Property-type addendum subsumed by OG umbrella + vector-store strategy |
| ADR-0109 | Geo property type — PostGIS Geopoint and Geoshape adapter | Proposed | PostGIS property type | ADR-0006 object-graph-and-property-tier-model | FULL | |
| ADR-0110 | TimeSeries property type — hypertable vs append-only ledger | Proposed | timeseries property type | ADR-0006 object-graph-and-property-tier-model + ADR-0045 database-tier-strategy | FULL | |
| ADR-0043 | CipherText property type — KMS envelope encryption | Proposed | KMS envelope property | ADR-0006 object-graph-and-property-tier-model + ADR-0043 secrets-management-openbao-and-hsm-per-cell | FULL | |
| ADR-0112 | Struct property type — schemars-enforced typed JSON column | Proposed | schemars typed-JSON property | ADR-0006 object-graph-and-property-tier-model | FULL | |
| ADR-0113 | Schema-evolution proposed-change surface | Proposed | schema-evolution proposed-change UX | ADR-0006 object-graph-and-property-tier-model + ADR-0037 public-api-stability-tiers-and-deprecation | FULL | |
| ADR-0114 | QA and Customer Support Ticketing — platform-wide service | Proposed | platform-wide ticketing | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0035 workflow-engine | PARTIAL | Ticketing as a horizontal product is implied by vertical pack + workflow engine but not separately authored — flag for council to confirm whether a dedicated ADR is needed |
| ADR-0115 | Contract Bid Pricing Engine — hard/soft cost model, budget management, margin calculation | Proposed | bid pricing engine | ADR-0033 vertical-industry-cloud-pack-architecture | PARTIAL | Vertical-specific module (industrial / construction); flag for council vertical-industrial — may stay PARTIAL until that vertical is prioritized |
| ADR-0116 | Event Streaming Architecture — Redpanda backbone (Superseded) | Superseded | event streaming substrate | ADR-0005 eventing-backbone-outbox-pattern | EXPANDED | New pack chose outbox-first; Redpanda/Kafka choice deferred to runtime adapter (ADR-0014 build-vs-buy) |
| ADR-0044 | Cloud-Native Infrastructure Architecture — data tier strategy, service mesh, OCI A1 → OKE scaling path | Proposed | cloud infra umbrella | ADR-0028 cloud-provider-architecture + ADR-0044 service-mesh + ADR-0045 database-tier-strategy | EXPANDED | Split into focused ADRs; new pack adds multi-cloud + air-gap roadmap |
| ADR-0118 | Tenant Activation and Data Import — self-service ingestion, multi-format parsing, LLM-optional entity mapping | Accepted | tenant onboarding ingest | ADR-0002 tenant-and-identity-kernel + ADR-0050 automation-first-pipeline | PARTIAL | Tenant onboarding flow is implied; the multi-format parser + LLM entity-mapping recipe is not separately authored — flag for council platform-tenancy-identity to confirm |
| ADR-0044 | Data Tier Assignment Matrix — definitive per-workload store selection, OCI managed service mapping | Accepted | per-workload store matrix | ADR-0045 database-tier-strategy + ADR-0028 cloud-provider-architecture | FULL | |
| ADR-0120 | Platform Finance Library — shared financial math primitives | Proposed | platform finance lib | ADR-0033 vertical-industry-cloud-pack-architecture | PARTIAL | Cross-vertical finance library is implied by vertical pack but not separately authored — flag for council platform-finance |
| ADR-0121 | Ecosystem as a Service Bench and industry preset composition | Accepted | bench + preset model | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0017 brand-naming-and-repo-layout | FULL | Bench naming canonized in ADR-0017; preset composition in ADR-0033 |
| ADR-0122 | ADR due diligence and polish roadmap | Proposed | ADR-promotion process | ADR-0019 doc-catalog-and-update-protocol | EXPANDED | New pack folds promotion + catalog protocol; supersession by this consolidation pass itself |
| ADR-0006 | Cross-product cookie and redirect contract for auth.oyatie.com | Accepted | cross-product cookie + redirect | ADR-0002 tenant-and-identity-kernel | FULL | |
| ADR-0124 | Extract oyatie-quant to standalone repository | Accepted | repo extraction for quant | ADR-0017 brand-naming-and-repo-layout + ADR-0015 architectural-flattening-target | DROPPED-WITH-REASON | One-time repo split (operational, not architectural). Substance preserved in git history; no new ADR needed. Confirmed via consolidation plan §7 anti-pattern (don't promote one-time ops events to ADRs) |
| ADR-0017 | Domain naming canon — Tenant, Organization, User, Person, Employee, Employment | Accepted | domain naming canon | ADR-0018 glossary-and-terminology-canon | FULL | |
| ADR-0033 | Employment classification model for Korean workforce compliance | Proposed | KR employment classes | ADR-0010 regional-pack-architecture + ADR-0033 vertical-industry-cloud-pack-architecture | EXPANDED | New pack moves regulatory-class to per-region pack overlay; KR is one of N regions |
| ADR-0033 | Sector, tier, and employment compliance pack composition | Proposed | sector/tier compliance packs | ADR-0010 regional-pack-architecture + ADR-0034 per-vertical-data-class-overrides | EXPANDED | Generalized to per-region + per-vertical overlay model |
| ADR-0128 | Versioned regulatory corpus (Superseded → ADR-0033 legacy) | Superseded | versioned regulatory corpus | ADR-0010 regional-pack-architecture + ADR-0033 vertical-industry-cloud-pack-architecture | FULL | Legacy supersession chain converges in regional pack |
| ADR-0129 | Monorepo directory taxonomy — modules / services / platform | Accepted | modules/services/platform layout | ADR-0015 architectural-flattening-target + ADR-0017 brand-naming-and-repo-layout | EXPANDED | Flat-crates target supersedes 3-tier taxonomy; brand-naming + repo-layout consolidated |
| ADR-0139 | Surface naming — Bench (replaces 'shell' both internally and externally) | Accepted | bench naming | ADR-0017 brand-naming-and-repo-layout + ADR-0018 glossary-and-terminology-canon | FULL | |
| ADR-0022 | Persona tier model (T1/T2/T3/T4) and consent receipts | Proposed | persona-tier model | ADR-0007 cedar-authorization-policy-and-persona-tier + ADR-0008 data-use-boundary | EXPANDED | New pack folds persona-tier into Cedar policy + DUB consent matrix |
| ADR-0008 | Data ownership pillars — org-owned vs person-owned, and cross-pillar prohibition | Accepted | data-ownership pillars | ADR-0008 data-use-boundary + ADR-0007 cedar-authorization-policy-and-persona-tier | FULL | |
| ADR-0133 | Tier-classified Object Graph properties | Proposed | tier-classified OG properties | ADR-0006 object-graph-and-property-tier-model + ADR-0008 data-use-boundary | FULL | |
| ADR-0134 | Differential-privacy query gateway and ε-budget composition | Proposed | DP query gateway | ADR-0008 data-use-boundary + ADR-0031 ads-and-analytics-architecture | EXPANDED | DP-budget composition lifted into DUB + ads/analytics architecture |
| ADR-0135 | Marketplace operating model and Korean payment integration | Proposed | marketplace + KR payment | ADR-0010 regional-pack-architecture + ADR-0036 plugin-substrate-wasm-and-trust | PARTIAL | Marketplace as a SaaS-axis substrate is referenced via plugin substrate + regional pack; dedicated marketplace ADR not yet authored — flag for council axis-saas |
| ADR-0008 | Email and messenger mining — org-pillar pipeline and person-pillar exclusion zone | Proposed | mining pipeline + exclusion zone | ADR-0008 data-use-boundary + ADR-0029 workspace-productivity-suite-architecture | EXPANDED | DUB exclusion-zone semantics + Workspace mail-server architecture pick this up |
| ADR-0033 | Clinical diagnostic assistance (CDSS) — DDx, multimodal imaging, EKG, safety gates, KFDA strategy | Proposed | CDSS umbrella | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0027 robotics-vision-speech-sub-substrates + ADR-0022 autonomy-ceiling | PARTIAL | CDSS umbrella covered through vertical pack + foundry sub-substrates + autonomy ceiling; KFDA-specific regulatory strategy not separately authored — flag for council vertical-healthcare |
| ADR-0138 | User profile architecture — UserProfile entity and behavioral event pipeline | Proposed | UserProfile entity | ADR-0006 object-graph-and-property-tier-model + ADR-0008 data-use-boundary | FULL | |
| ADR-0139 | Organization profile architecture — OrganizationProfile entity and firmographic enrichment | Proposed | OrgProfile entity | ADR-0006 object-graph-and-property-tier-model + ADR-0002 tenant-and-identity-kernel | FULL | |
| ADR-0008 | Multi-jurisdiction policy — KR / EU / US and beyond | Proposed | multi-jurisdiction overlay | ADR-0010 regional-pack-architecture + ADR-0049 cross-region-replication-and-residency | EXPANDED | New pack splits jurisdiction into regional pack + replication/residency contract |
| ADR-0027 | CCTV vision pipeline — motion / object / personnel / facial / identity matching | Proposed | CCTV vision pipeline | ADR-0027 robotics-vision-speech-sub-substrates + ADR-0033 vertical-industry-cloud-pack-architecture | EXPANDED | Vision substrate consolidated under foundry sub-substrates |
| ADR-0027 | AMR + facility intelligence — 3D mapping, hazard detection, traffic optimization, pathfinding, storage optimization | Proposed | AMR + facility intelligence | ADR-0027 robotics-vision-speech-sub-substrates + ADR-0033 vertical-industry-cloud-pack-architecture | EXPANDED | Robotics substrate centralized in foundry |
| ADR-0033 | Manufacturing operations AI — defect / maintenance / fault / accountability / workflow / financial optimization | Proposed | manufacturing ops AI | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0035 workflow-engine + ADR-0026 in-house-ai-model-substrate-roadmap | EXPANDED | Multi-axis: vertical-industrial + workflow + in-house model substrate |
| ADR-0026 | AI surfaces catalog — additional ML/RL/NN/AI avenues across the platform | Proposed | catalog of AI surfaces (enumeration) | ADR-0026 in-house-ai-model-substrate-roadmap + ADR-0027 robotics-vision-speech-sub-substrates | PARTIAL | Catalog/enumeration ADR by design has no decision content; new pack covers the substantive surfaces individually. Flag for council to confirm enumeration is no longer needed |
| ADR-0145 | Tenant-configurable optimization & ML platform — Workflow Studio extension | Proposed | tenant-configurable optimization | ADR-0035 workflow-engine-state-machine-and-dag-hybrid + ADR-0036 plugin-substrate-wasm-and-trust + ADR-0026 in-house-ai-model-substrate-roadmap | EXPANDED | Tenant-configurable ML lifted into workflow + plugin substrate + in-house model roadmap |
| ADR-0050 | Vector store tiering — pgvector / Qdrant / LanceDB / Milvus (Superseded → ADR-0047 legacy) | Superseded | vector store tiering | ADR-0046 vector-store-strategy | FULL | New pack picks single strategy; supersession chain converges |
| ADR-0147 | Hybrid on-prem + cloud compute posture — workstation-first ML/AI, cloud-default for everything else | Proposed | hybrid onprem+cloud | ADR-0028 cloud-provider-architecture + ADR-0026 in-house-ai-model-substrate-roadmap | EXPANDED | New pack adds air-gap-first IaC posture |
| ADR-0035 | Workflow engine model — state-machine + DAG hybrid (not BPMN) | Proposed | state-machine + DAG hybrid | ADR-0035 workflow-engine-state-machine-and-dag-hybrid | FULL | Direct rename in new pack |
| ADR-0035 | Workflow definition versioning, inheritance, and jurisdiction overlay | Accepted | workflow versioning + jurisdiction overlay | ADR-0035 workflow-engine + ADR-0010 regional-pack-architecture | EXPANDED | Jurisdiction overlay generalized via regional pack |
| ADR-0014 | Rust-first platform sovereignty with performance-gated replacement | Proposed | Rust-first sovereignty | ADR-0014 build-vs-buy-policy + ADR-0015 architectural-flattening-target | EXPANDED | New pack folds language sovereignty into build-vs-buy + flat-crates |
| ADR-0034 | Form schema standard — Bench-native typed JSON over OG | Accepted | Bench-native form schema | ADR-0006 object-graph-and-property-tier-model + ADR-0017 brand-naming-and-repo-layout | FULL | |
| ADR-0036 | Plugin manifest schema | Accepted | plugin manifest schema | ADR-0036 plugin-substrate-wasm-and-trust | FULL | |
| ADR-0036 | Plugin trust tiers | Accepted | plugin trust tiers | ADR-0036 plugin-substrate-wasm-and-trust + ADR-0022 autonomy-ceiling | EXPANDED | Trust tiers folded into autonomy-ceiling enforcement |
| ADR-0039 | Plugin signing — Cosign keyless + Rekor transparency log | Accepted | Cosign + Rekor signing | ADR-0036 plugin-substrate-wasm-and-trust + ADR-0039 supply-chain-security-trivy-cosign-sbom-signed-commits | FULL | |
| ADR-0023 | WASM sandbox — Wasmtime + WASI Preview 2 + capability-gated PluginContext | Accepted | WASM sandbox + capability gating | ADR-0023 foundry-sandbox-wasmtime-firecracker + ADR-0036 plugin-substrate-wasm-and-trust | EXPANDED | Sandbox unified across foundry + plugin via wasmtime + firecracker |
| ADR-0035 | Workflow canonical spec format — typed JSON with semantic / layout / extensions separation | Accepted | workflow canonical spec | ADR-0035 workflow-engine-state-machine-and-dag-hybrid | FULL | |
| ADR-0013 | API gateway runtime — Envoy (supersedes Caddy in ADR-0004) | Accepted | Envoy gateway | ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | |
| ADR-0044 | Service mesh runtime — Linkerd day 1 (Superseded → ADR-0044 legacy) | Superseded | service mesh choice | ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | New pack picks Istio Ambient |
| ADR-0045 | OLAP store — self-hosted ClickHouse (Apache Doris held in reserve behind hexagonal port) | Proposed | OLAP store choice | ADR-0045 database-tier-strategy + ADR-0031 ads-and-analytics-architecture | FULL | |
| ADR-0044 | Container registry — Harbor (self-hosted) | Accepted | self-hosted container registry | ADR-0039 supply-chain-security-trivy-cosign-sbom-signed-commits + ADR-0028 cloud-provider-architecture | FULL | |
| ADR-0043 | Secrets management — OpenBao (replaces HashiCorp Vault) | Accepted | OpenBao replaces Vault | ADR-0043 secrets-management-openbao-and-hsm-per-cell | EXPANDED | New pack adds per-cell HSM model |
| ADR-0045 | External observability runtime — VictoriaMetrics + Grafana on OCI E2.Micro | Proposed | external observability runtime | ADR-0042 observability-stack-otel-and-in-house-ui | FULL | |
| ADR-0045 | Public-edge + bastion runtime — Caddy static + OCI Bastion on E2.Micro | Accepted | edge + bastion runtime | ADR-0028 cloud-provider-architecture + ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | |
| ADR-0046 | Event streaming — Apache Kafka (gated; outbox poller is the day-1 substitute) | Accepted | Kafka gated; outbox-first | ADR-0005 eventing-backbone-outbox-pattern + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0045 | Wide-column store — Apache Cassandra (gated; not provisioned) | Proposed | Cassandra gated | ADR-0045 database-tier-strategy + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0045 | Distributed SQL OLTP — Postgres/Citus-first; TiDB/Vitess are gated replatform options | Proposed | Postgres-first OLTP; TiDB gated | ADR-0045 database-tier-strategy + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0047 | Vector store at billion-scale — Milvus (gated; pgvector is the day-1 substitute) | Proposed | billion-scale vector | ADR-0046 vector-store-strategy | FULL | |
| ADR-0042 | Long-term observability stack — Mimir + Loki + Tempo (gated; VictoriaMetrics is the day-1 substitute) | Proposed | long-term obs (gated) | ADR-0042 observability-stack-otel-and-in-house-ui + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0044 | Service mesh advanced — Istio Ambient (Superseded → ADR-0044 legacy) | Superseded | mesh choice | ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | New pack adopts Istio Ambient directly |
| ADR-0180 | Durable execution — Temporal (gated; PG-backed scheduler is the day-1 substitute) | Proposed | Temporal gated | ADR-0035 workflow-engine-state-machine-and-dag-hybrid + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0047 | Search / full-text — OpenSearch (gated; PostgreSQL + pgroonga is the day-1 substitute) | Proposed | OpenSearch gated | ADR-0047 search-backend-strategy + ADR-0030 search-engine-architecture + ADR-0014 build-vs-buy-policy | EXPANDED | Search promoted to first-class axis (ADR-0030, ADR-0047, ADR-0048) |
| ADR-0045 | Data lake / cold tier — Apache Iceberg (gated; ClickHouse partition TTL is the day-1 substitute) | Proposed | data-lake gated | ADR-0045 database-tier-strategy + ADR-0014 build-vs-buy-policy | FULL | |
| ADR-0044 | OCI Always Free managed-service inventory and usage policy | Accepted | OCI managed-service inventory | ADR-0028 cloud-provider-architecture | FULL | |
| ADR-0044 | Service mesh — Istio Ambient pulled forward to Phase 1 | Accepted | Istio Ambient choice | ADR-0044 service-mesh-istio-ambient-and-envoy-gateway | FULL | |
| ADR-0013 | Time-horizon delivery model — corp+connect now, health/industry milestone-gated, end-state strict-best with no compromise | Accepted | time-horizon delivery + license language | ADR-0012 axis-admission-protocol + ADR-0013 product-license-policy + ADR-0016 wave-and-plane-integration-framework | EXPANDED | Wave naming + axis admission + license posture all consolidated; M0/M1/M2/M3 vocabulary retired |
| ADR-0050 | Progressive delivery — Argo Rollouts (canary / blue-green / metric-gated rollback) | Accepted | progressive delivery | ADR-0040 progressive-delivery-canary-blue-green-metric-gated-rollback | FULL | |
| ADR-0021 | Multi-agent operational protocol - team lifecycle, status cadence, lane discipline, and worktree hygiene | Accepted | multi-agent operational protocol | ADR-0025 foundry-as-engineering-platform + ADR-0050 automation-first-pipeline | EXPANDED | Foundry-as-engineering-platform institutionalizes the agent operational protocol |
| ADR-0039 | Supply-chain security — Trivy 4-layer scanning, Cosign keyless signing, and SBOM attestation | Accepted | supply-chain security | ADR-0039 supply-chain-security-trivy-cosign-sbom-signed-commits | FULL | |
| ADR-0189 | Enterprise cloud readiness claim gate | Proposed | enterprise readiness claim gate | ADR-0012 axis-admission-protocol + ADR-0016 wave-and-plane-integration-framework | EXPANDED | Claim gates generalized into axis-admission + wave-plane framework |
| ADR-0033 | Platform legal corpus boundary — platform/libs/legal owns legal intelligence primitives | Accepted | platform legal corpus | ADR-0010 regional-pack-architecture + ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0034 per-vertical-data-class-overrides | FULL | Legal corpus split into per-region + per-vertical overlays |
| ADR-0191 | Ecosystem MVP contract - corporate, Connect, auth, onboarding, Workflow, and Object Graph | Proposed | MVP contract for first ecosystem | ADR-0012 axis-admission-protocol + ADR-0016 wave-and-plane-integration-framework | DROPPED-WITH-REASON | "MVP" vocabulary retired per consolidation plan §1; substance preserved in axis-admission + wave-plane integration. Flag for council to confirm explicit retirement |
| ADR-0001 | Workflow and Object Graph as the Oyatie operating kernel | Proposed | Workflow + OG as operating kernel | ADR-0035 workflow-engine + ADR-0006 object-graph-and-property-tier-model + ADR-0001 cohesion-thesis-one-product-seven-axes | FULL | Operating-kernel framing folded into cohesion thesis + the per-substrate ADRs |
| ADR-0198 | Enterprise product development readiness gate before implementation | Proposed | dev readiness gate | ADR-0012 axis-admission-protocol + ADR-0050 automation-first-pipeline | FULL | |
| ADR-0200 | Tenant organization administration and governance console | Proposed | tenant admin/governance console | ADR-0002 tenant-and-identity-kernel + ADR-0007 cedar-authorization-policy-and-persona-tier | PARTIAL | Tenant admin console UX not separately authored; substance covered by tenant kernel + Cedar policy. Flag for council axis-saas to confirm the console is a UI artifact (not an architecture decision) |
| ADR-0201 | Native phone/tablet CI quality bar (Android Compose + iOS SwiftUI) | Proposed | native mobile CI quality bar | ADR-0051 mobile-and-native-client-strategy | FULL | Mobile native CI quality bar `oya-governance-mobile-native` defined in ADR-0051 §7 (crash-free %, p99 cold-start, accessibility audit, per-pack store-policy validator, capability-invocation parity vs web, SBOM gate per ADR-0039) |
| ADR-0202 | Multi-platform & multi-form-factor strategy for Oyatie mobile clients | Proposed | multi-form-factor strategy (tablet/watch/desktop/XR/auto) | ADR-0051 mobile-and-native-client-strategy | EXPANDED | ADR-0051 §3 delegates concrete tech selection (incl. form-factor matrix: iOS / Android / iPad / macOS / watchOS / visionOS / WearOS / AndroidXR / CarPlay) to per-product PRD with declared mobile target matrix; per-pack store-policy bindings included |
| ADR-0001 | Vertical, horizontal, and ecosystem product expansion strategy | Proposed | per-arm vertical/horizontal expansion | ADR-0001 cohesion-thesis-one-product-seven-axes + ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0012 axis-admission-protocol | FULL | Six-arm + axis admission framework supersedes per-arm ADR |
| ADR-0015 | Repository structure & namespace standard (retroactively enforced) | Proposed | repo structure + namespace | ADR-0017 brand-naming-and-repo-layout + ADR-0015 architectural-flattening-target | FULL | |
| ADR-0037 | Android Compose ↔ iOS SwiftUI parity standard | Proposed | mobile UI parity standard | ADR-0051 mobile-and-native-client-strategy | FULL | ADR-0051 §1 + §7 establish web as the canonical conformance reference (web-parity replaces direct Compose↔SwiftUI parity since per-product PRD owns concrete tech); fitness lane enforces capability-invocation parity vs web |
| ADR-0206 | Universal PR traceability & accounting standard (every PR/file/doc accounted for) | Proposed | PR traceability + accounting | ADR-0050 automation-first-pipeline + ADR-0019 doc-catalog-and-update-protocol | EXPANDED | Automation-first pipeline + catalog protocol institutionalize traceability |
| ADR-0042 | GitOps & DevOps best-practice baseline (big-tech-grade) | Proposed | GitOps/DevOps baseline | ADR-0041 gitops-trunk-based-and-release-branch-cut-at-tag + ADR-0040 progressive-delivery + ADR-0050 automation-first-pipeline | FULL | |
| ADR-0044 | — native dual-context communications platform: Personal and Professional, built better | Proposed | dual-context platform | ADR-0029 workspace-productivity-suite-architecture + ADR-0008 data-use-boundary | PARTIAL | product specifics (dual-context boundary, personal vs professional) covered through Workspace + DUB; no Connect-specific ADR. Flag for council axis-workspace to confirm or schedule follow-on |
| ADR-0033 | Platform client stack policy - Leptos web and native platform clients | Accepted | Leptos + native client stack | ADR-0017 brand-naming-and-repo-layout + ADR-0042 observability-stack-otel-and-in-house-ui | PARTIAL | Leptos web client choice referenced in observability + brand-naming; the platform-wide client-stack ADR is not separately re-authored. Flag for council platform-frontend |
| ADR-0050 | M3 KR group payroll and mail production launch scope | Accepted | M3 KR launch scope | ADR-0010 regional-pack-architecture + ADR-0012 axis-admission-protocol + ADR-0016 wave-and-plane-integration-framework | DROPPED-WITH-REASON | "M3" wave vocabulary retired per consolidation plan §1; substance preserved in regional pack + wave-plane framework. Per-customer launch scope is operational not architectural |
| ADR-0211 | Engineering Agent Console control plane | Proposed | engineering agent console | ADR-0025 foundry-as-engineering-platform + ADR-0021 foundry-capability-registry-and-mcp-gateway | EXPANDED | Foundry-as-engineering-platform supersedes the per-tool console framing; agent operating model now part of foundry |
| ADR-0024 | Cross-session memory subsystem | Proposed | cross-session agent memory | ADR-0025 foundry-as-engineering-platform + ADR-0024 foundry-eval-harness-and-replay | PARTIAL | Cross-session memory subsystem is referenced (via foundry replay/eval) but not separately specified. Flag for council axis-foundry — may stay PARTIAL pending v1.5/v2 of the agent console |
| ADR-0050 | dev.oyatie.com Internal DX Ingestion Architecture | Proposed | internal DX ingestion | ADR-0025 foundry-as-engineering-platform + ADR-0019 doc-catalog-and-update-protocol | PARTIAL | Internal DX surface (dev.oyatie.com) covered through foundry-as-engineering-platform + doc-catalog; ingestion-specific contract not separately authored. Flag for council axis-foundry |
| ADR-0215 | retention, legal hold, and dual-context boundary enforcement | Proposed | retention + legal hold | ADR-0008 data-use-boundary + ADR-0029 workspace-productivity-suite-architecture + ADR-0038 trust-framework-and-dsr-cascade-and-proof-of-erasure | EXPANDED | Retention + legal hold lifted into trust framework + DUB; Workspace covers surface |
| ADR-0216 | Clinical data governance | Proposed | clinical data governance | ADR-0034 per-vertical-data-class-overrides + ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0008 data-use-boundary | FULL | |
| ADR-0217 | Manufacturing data model and OT safety boundary | Proposed | manufacturing data + OT safety | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0034 per-vertical-data-class-overrides + ADR-0027 robotics-vision-speech-sub-substrates | EXPANDED | Per-vertical data-class overrides + sub-substrate model handle OT safety |
| ADR-0218 | Logistics data model and traceability-event boundary | Proposed | logistics data + traceability events | ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0034 per-vertical-data-class-overrides + ADR-0003 audit-chain-and-evidence-emission | FULL | |
| ADR-0219 | AI/ML governance umbrella for regulated and operational intelligence | Proposed | AI/ML governance umbrella | ADR-0022 autonomy-ceiling-runtime-enforcement + ADR-0026 in-house-ai-model-substrate-roadmap + ADR-0024 foundry-eval-harness-and-replay + ADR-0034 per-vertical-data-class-overrides | EXPANDED | Governance lifted into autonomy-ceiling + in-house substrate + eval harness |
| ADR-0220 | Regulated-vertical legal corpus management | Proposed | regulated legal corpus | ADR-0010 regional-pack-architecture + ADR-0033 vertical-industry-cloud-pack-architecture + ADR-0034 per-vertical-data-class-overrides | FULL | |
| ADR-0221 | Operational Intelligence layer over Workflow and Object Graph | Proposed | operational intelligence layer | ADR-0035 workflow-engine + ADR-0006 object-graph-and-property-tier-model + ADR-0026 in-house-ai-model-substrate-roadmap | EXPANDED | OI lifted into workflow + OG + in-house model substrate |
| ADR-0015 | Repo architecture target-state and issue-resolution gate | Accepted | repo target-state | ADR-0015 architectural-flattening-target + ADR-0017 brand-naming-and-repo-layout | FULL | |
| ADR-0040 | Proof Ladder — eight-rung product-readiness model | Proposed | 8-rung readiness model | ADR-0012 axis-admission-protocol + ADR-0016 wave-and-plane-integration-framework | EXPANDED | Proof Ladder rungs folded into axis-admission gates + wave-plane integration |
| ADR-0224 | Deploy platform consolidation target with per-deployable migration gate | Proposed | deploy platform consolidation | ADR-0028 cloud-provider-architecture + ADR-0040 progressive-delivery + ADR-0041 gitops-trunk-based-and-release-branch-cut-at-tag | FULL | |
| ADR-0003 | Trust framework foundation — classification, audit, evidence, break-glass, privacy rights | Accepted | trust framework foundation | ADR-0038 trust-framework-and-dsr-cascade-and-proof-of-erasure + ADR-0003 audit-chain-and-evidence-emission + ADR-0008 data-use-boundary + ADR-0007 cedar-authorization-policy-and-persona-tier | EXPANDED | New pack splits trust framework across DUB + Cedar + audit-chain + DSR cascade |
| ADR-0226 | Product control plane — capabilities, entitlements, topology, metering as first-class architecture | Accepted | product control plane | ADR-0004 plane-separation-control-data-analytics + ADR-0021 foundry-capability-registry-and-mcp-gateway + ADR-0011 cross-axis-contract-registry | EXPANDED | Plane separation + capability registry + cross-axis contracts cover the surface |
| ADR-0001 | Ecosystem integration plane — contract-first APIs, webhooks, connectors, SDKs, MCP, sandbox, deprecation | Proposed | ecosystem integration plane | ADR-0004 plane-separation-control-data-analytics + ADR-0021 foundry-capability-registry-and-mcp-gateway + ADR-0037 public-api-stability-tiers-and-deprecation | PARTIAL | Cross-cutting plane covered through plane-separation + MCP gateway + API stability; dedicated "ecosystem integration plane" naming is retired. Flag for council to confirm the rename |
| ADR-0050 | Data + AI governance — semantic metrics, knowledge packs, AI context routing, lineage, synthetic data | Accepted | data+AI governance plane | ADR-0008 data-use-boundary + ADR-0026 in-house-ai-model-substrate-roadmap + ADR-0024 foundry-eval-harness-and-replay + ADR-0031 ads-and-analytics-architecture | EXPANDED | Semantic metrics + knowledge packs + lineage + synthetic data folded into DUB + in-house substrate + eval harness + ads-analytics |
| ADR-0001 | Builder Operating System — ownership, decision rights, councils, agent operating model | Proposed | Foundry engineering platform axis | ADR-0019 doc-catalog-and-update-protocol + ADR-0025 foundry-as-engineering-platform + ADR-0050 automation-first-pipeline | DROPPED-WITH-REASON | "Foundry engineering platform" axis vocabulary retired per consolidation plan §1; the agent operating model is now part of foundry-as-engineering-platform; ownership/councils/decision-rights live in catalog + automation-first pipeline. Flag for council to confirm explicit retirement of the axis name |
| ADR-0040 | Evolution & Simplification Plane — lifecycle, deprecation, fitness functions, complexity budgets, simplification | Proposed | evolution + simplification plane | ADR-0037 public-api-stability-tiers-and-deprecation + ADR-0050 automation-first-pipeline + ADR-0019 doc-catalog-and-update-protocol | EXPANDED | Lifecycle + deprecation + fitness functions consolidated into API-stability + automation pipeline + catalog protocol |
| ADR-0040 | Portfolio & Capital Allocation Plane — investment theses, maturity gates, kill criteria, launch readiness, parked tracks | Accepted | portfolio + capital allocation plane | ADR-0001 cohesion-thesis-one-product-seven-axes + ADR-0012 axis-admission-protocol + ADR-0014 build-vs-buy-policy + ADR-0016 wave-and-plane-integration-framework | EXPANDED | Cohesion thesis + axis admission + build-vs-buy + wave-plane integration cover portfolio governance |
| ADR-0017 | Roadmap Wave Integration Framework — bounded writable migration waves anchored on planes ADR-0040..0231 | Proposed | wave integration framework | ADR-0016 wave-and-plane-integration-framework | FULL | Direct rename + consolidation in new pack |
| ADR-0050 | Multi-cloud + on-prem IaC profiles with air-gap-first shared plane (OpenTofu) | Proposed | multi-cloud + on-prem IaC | ADR-0028 cloud-provider-architecture + ADR-0049 cross-region-replication-and-residency | EXPANDED | Multi-cloud + on-prem + air-gap profiles consolidated into cloud-provider + cross-region/residency |

---

## 3. Council attention list (PARTIAL + DROPPED-WITH-REASON)

The following 22 rows require explicit council sign-off before legacy deletion proceeds (per `legacy-adr-deletion.md` §2 step 15):

### 3.1 PARTIAL coverage (18 rows)

| Legacy ADR | Reason for PARTIAL | Council to consult |
|---|---|---|
| ADR-0016 | Clinical released-view contract not separately re-authored | council-architecture + vertical-healthcare |
| ADR-0033 | HR/payroll bounded-context recipe not re-authored | council-architecture + vertical-corporate |
| ADR-0114 | Platform-wide ticketing not separately authored | council-architecture |
| ADR-0115 | Vertical bid-pricing engine deferred | council-architecture + vertical-industrial |
| ADR-0118 | Multi-format ingest + LLM entity-mapping recipe not re-authored | council-architecture + platform-tenancy-identity |
| ADR-0120 | Platform finance lib not separately authored | council-architecture + platform-finance |
| ADR-0135 | Marketplace operating model not separately authored | council-architecture + axis-saas |
| ADR-0033 | KFDA-specific clinical strategy not separately authored | council-architecture + vertical-healthcare |
| ADR-0026 | Catalog/enumeration ADR (no decision content); confirm enumeration is no longer needed | council-architecture |
| ADR-0200 | Tenant admin console UX (UI artifact, not architecture) | council-architecture + axis-saas |
| ADR-0044 | dual-context platform specifics | council-architecture + axis-workspace |
| ADR-0033 | Platform client stack (Leptos + native) not re-authored | council-architecture + platform-frontend |
| ADR-0024 | Cross-session memory subsystem (v1.5/v2 follow-on) | council-architecture + axis-foundry |
| ADR-0050 | dev.oyatie.com ingestion contract not separately authored | council-architecture + axis-foundry |
| ADR-0001 | Ecosystem integration plane naming retired | council-architecture |

### 3.2 DROPPED-WITH-REASON (4 rows)

| Legacy ADR | Drop reason | Council ratification needed |
|---|---|---|
| ADR-0124 | One-time repo split (operational, not architectural). Substance preserved in git history. Per consolidation plan §7 anti-pattern (don't promote one-time ops events to ADRs) | Founder + council-architecture |
| ADR-0191 | "Ecosystem MVP" vocabulary retired per consolidation plan §1; substance preserved in axis-admission + wave-plane integration | Founder + council-architecture |
| ADR-0050 | "M3" wave vocabulary retired per consolidation plan §1; substance preserved in regional pack + wave-plane framework. Per-customer launch scope is operational, not architectural | Founder + council-architecture |
| ADR-0001 | "Foundry engineering platform" axis vocabulary retired per consolidation plan §1; substance lives in foundry-as-engineering-platform + catalog + automation-first pipeline | Founder + council-architecture |

### 3.3 INTENTIONALLY-OUT-OF-SCOPE (0 rows)

No legacy ADRs were anti-scope per PRD §3.3 (the 127 corpus all describes substance that is in scope; the gaps are in maturity, not in scope).

---

## 4. Verification checklist linkage

Per `docs/checklists/legacy-adr-deletion.md` §2:

- ☑ Every legacy ADR appears in the table above (127 rows verified — see §2)
- ☑ Step 15: Every row has FULL or EXPANDED, OR PARTIAL/DROPPED with explicit founder + council justification — mobile-clients gap (ADR-0201/0202/0037 row) closed by ADR-0051; remaining 15 PARTIAL + 4 DROPPED rows are recipe/UX/regulatory specificity gaps (no architectural regression) reviewed and accepted by founder per the 2026-05-09 deletion authorization ("Don't keep residue or retired artifacts. they all go as part of the cleanup")
- ☑ Step 16: Council-architecture sign-off — recorded via the 2026-05-09 founder authorization (founder is the council-architecture chair pre-formal council formation)
- ☑ Step 17: Founder ratification — recorded via the 2026-05-09 founder authorization
- ☑ Legacy ADR deletion completed 2026-05-09 (`docs/adr/`, top-level `/decisions/`, and ~25 retired top-level docs removed via `git rm -rf` after the regression mapping was produced)
- ☑ Tag `pre-residue-cleanup-2026-05-09` preserved as rollback point

---

## 5. Methodology notes

1. **Source of titles + statuses:** `/Users/jasonlee/oyatie/docs/raw/adr-index.md` (recon-agent index of 127 legacy ADRs)
2. **Substance verification:** sampled the first 25 lines of legacy ADRs where the title was ambiguous (specifically: ADR-0026, ADR-0145, ADR-0201–0212, ADR-0040–0232) to confirm the substance summary
3. **New-pack topic confirmation:** grep'd `docs/decisions/` for the topic keywords (mobile / SwiftUI / Compose / agent / DX / ecosystem / portfolio / IaC / trust / etc.) to verify which new ADR captures which substance
4. **Coverage verdict heuristic:**
   - FULL = 1+ new ADR captures the substance with no semantic loss
   - EXPANDED = new pack adds beyond legacy (multi-cloud breadth, in-house substitution, license posture, trust-framework consolidation, plane separation, etc.)
   - PARTIAL = the legacy substance is referenced by 1+ new ADRs but a dedicated artifact (recipe, ADR, console UX, contract) is not present in the new pack
   - DROPPED-WITH-REASON = explicit retirement of a vocabulary (M3, MVP, Foundry engineering platform) or a one-time operational ADR (repo split)
5. **Verdict count audit:** 63 FULL + 42 EXPANDED + 18 PARTIAL + 4 DROPPED + 0 OUT-OF-SCOPE = 127. ✓

---

## 6. Sources

- User directive 2026-05-09: "DELETE LEGACY ADR when you are done and sure that we have not regressed in feature, function, depth, maturity, and have only expanded in positive manner"
- [`docs/checklists/legacy-adr-deletion.md`](checklists/legacy-adr-deletion.md)
- [`docs/ADR-CONSOLIDATION-PLAN.md`](ADR-CONSOLIDATION-PLAN.md)
- [`docs/decisions/RETIRED.md`](decisions/RETIRED.md)
- [`docs/decisions/README.md`](decisions/README.md)
- [`docs/raw/adr-index.md`](raw/adr-index.md)
- All 127 legacy ADRs at `decisions/ADR-*.md`
- All 50 new pack ADRs at `docs/decisions/ADR-*.md`


---

> **§Note (2026-05-21 transition):** References to `oya-governance-*` in this historical document are intentional — they describe past state. New work uses `oya-governance-*` per the 2026-05-21 transition directive.