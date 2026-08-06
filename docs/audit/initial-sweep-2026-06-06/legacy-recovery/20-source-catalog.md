---
doc_class: LegacyRecoveryCatalog
title: Source Product / Capability Catalog (LIVE, extracted from source monorepo)
status: extracted
date: 2026-06-06
source_root: ~/Developer/source
extracted_from:
  - source/oya/                                  # 87 service dirs
  - source/cloud/                                # 25 service dirs
  - source/docs/products/                        # per-product PRDs
  - source/docs/machine-readable/products.json   # STALE axis/vertical mirror
  - source/docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md     # product-domain ADRs
  - source/docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
purpose: >
  LIVE product/capability inventory of the source ("oyatie") monorepo so it can be
  diffed against the legacy inventory. Captures (a) every service in oya/ + cloud/,
  (b) the product-domain ADRs and what they cover, (c) the substrate primitives.
---

# Source Product / Capability Catalog — LIVE (oyatie monorepo)

## 0. How the catalog is organized (read this first)

The source repo is **one product, expressed as a FLAT catalog of single-concern
microservices**. There is NO product/suite/family/bundle/vertical grouping as an
architecture artifact. This is the load-bearing doctrine and it changed over time:

- **ADR-0001 / ADR-0058 / ADR-0131 / ADR-0132 → ADR-0362**: "flat-only catalog."
  Healthcare / Enterprise / FinTech / Social etc. are **sales & marketing
  segmentation only** — never a directory, code, or deploy boundary. ADR-0362
  ("full grouping retirement") removed the last grandfathered grouping wrappers
  and made the `no-grouping` CI gate real.
- **ADR-0245 (substrate-vs-product layering)**: every µservice declares a manifest
  `tier`: `substrate` (audience-neutral capability), `product` (tenant-scoped
  end-user surface), `service-cell` (peer dedicated-function cell), or `reserved`
  (forward-declared, certification-gated, NOT deployed). Cross-tier dependency
  direction is CI-enforced; substrates carry a stricter SLO floor (99.99%) than
  products (99.9%).
- **ADR-0316 (capability-tier over product fragmentation)**: enterprise product
  surfaces (CRM, ITSM, HR, marketing, CLM, LMS, FP&A…) are **tenant activation
  bundles** (Cedar permit set + ontology projection + workflow templates + UX
  shell + compliance overlay + cost/audit metadata) over shared substrate — NOT
  new microservices. A new flat µservice is created ONLY when a candidate proves a
  *distinct operational concern* (distinct write authority / scale axis / failure
  mode / regulatory license / runtime). ADR-0316 is superseded-in-text by ADR-0329
  (tier system → tenant-class) but the projection-over-substrate doctrine stands.

**IMPORTANT — `docs/machine-readable/products.json` is STALE.** It still describes
the retired "7 axes / 14 verticals / 10 regional packs" model (generated 2026-05-09).
The live catalog is the flat `oya/` + `cloud/` service tree. The coverage matrix
(`enterprise-software-coverage-matrix-2026-05-21.md`) self-reports
**56 microservices post-ERP**, target **69+ after B2B-leader coverage**, with a
**244-vendor-row capability-tier registry**. The on-disk tree has since grown to
**87 dirs in oya/ + 25 in cloud/** (some are crate-only WIP scaffolds, some are
newer bespoke-substrate services not yet in the matrix count).

Counts on disk (2026-06-06): **oya/ = 87 dirs, cloud/ = 25 dirs**.

---

## 1. Products / services in `oya/` (the application + substrate plane)

Tier per ADR-0245 §D-3. "product" = tenant-scoped surface; "substrate" =
audience-neutral capability; "service-cell" = peer cell; ✚ = newer bespoke service
or scaffold not in the 56-count matrix.

### 1.1 Workspace / productivity products (Google-Workspace-class surfaces)
| Service | Tier | Inferred purpose |
|---|---|---|
| `mail` | product | Email client UI (sends via comms-email substrate) |
| `calendar` | product | Calendar / scheduling |
| `drive` | product | File storage + management UI |
| `docs` | product | Document editor |
| `sheets` | product | Spreadsheet editor |
| `slides` | product | Presentation editor |
| `notes` | product | Notes app |
| `tasks` | product | Task management |
| `forms` | product | Forms / surveys |
| `sites` | product | Web-publishing |
| `meet` | product | Video conferencing |
| `recordings` | product | Meeting recordings (paired with meet) |
| `messenger` | product | Chat / messaging |
| `whiteboard` | product | Collaborative whiteboard |
| `design-collaboration` | product | Design collaboration (Figma-class) |
| `translate` | product | Translation tool |
| `connect` ✚ | product | Dual-context (Personal B2C + Professional B2B) comms+community meta-surface; Workspace/M365/Naver-Works/Kakao-Work replacement (ADR-0029). Being dissolved into 8 flat µservices via ADR-0237/0238 strangler. On disk: crate-only scaffold (address-book domain). |
| `community` | product | Community Q&A / KB threads; absorbs anonymity-posting modes |
| `social` | product | Social network; **absorbed `shorts`** (TikTok-style short video) per ADR-0334 |

### 1.2 Workplace / business-app surfaces (capability-tier hosts)
These exist as µservices because the coverage doctrine gave them a distinct concern;
many also act as capability-tier projection owners (CRM/ITSM/HR/etc. labels).
| Service | Tier | Inferred purpose |
|---|---|---|
| `crm` | product | CRM (Sales/Service-Cloud-class customer graph) |
| `marketing-automation` | product | Segments, campaigns, journeys, attribution |
| `contact-center` | product | Telephony/voice/queue (named distinct-runtime new service) |
| `hr` | product | Human-capital surface (HCM) |
| `payroll` | product | Payroll |
| `performance-management` | product | Goals / reviews / calibration |
| `learning-management` | product | LMS — courses / completions / training compliance |
| `contract-lifecycle-management` | product | CLM — contract intake / review / signature |
| `itsm` | product | IT service management |
| `incident-management` | product | Incident / on-call workflows |
| `financial-planning` | product | FP&A — budgets / forecasts / scenarios |
| `accounting` | product | GL / accounting (ERP-finance) |
| `treasury` | product | Treasury / liquidity / cash mgmt |
| `global-trade` | product | Global trade / sanctions / customs screening |
| `workplace-integration` | product | Employee directory / workplace integration surface |
| `real-estate` | product | Real-estate / lease accounting |
| `plant-maintenance` | product | EAM / plant maintenance (ERP) |
| `production-planning` | product | Production planning (ERP/MES) |
| `quality-management` | product | Quality management (ERP) |
| `supply-chain-planning` | product | Supply-chain planning |
| `warehouse` | product | Warehouse execution (WMS) |

### 1.3 Healthcare domain (ADR-0332 decomposition of healthcare-integration)
| Service | Tier | Inferred purpose |
|---|---|---|
| `healthcare-integration` | substrate | Narrowed to FHIR-R4 / HL7v2 / DICOM integration substrate only |
| `emr` | product | Electronic medical record (foundational clinical µservice) |
| `diagnostics` | product | Lab + pathology |
| `imaging` | product | Medical imaging (DICOM/PACS) |
| `pharmacy` | product | Medication catalog / formulary / ePrescribe / drug-interaction |
| `patient-monitoring` | product | Continuous physiologic surveillance (ICU/CCU/ED + RPM) |
| `emergency` | product | Emergency / life-safety services |
| *(planned by ADR-0332)* | product | `clinical-decision-support`, `care-management` (named, not yet present on disk) |

### 1.4 Platform / data / analytics products
| Service | Tier | Inferred purpose |
|---|---|---|
| `workflow-studio` | product | Visual workflow editor (n8n-class canvas) |
| `analytics` | product | Per-tenant / per-cohort analytics dashboards |
| `data-pipeline` | product/substrate | ETL / streaming data pipeline |
| `data-warehouse` | product/substrate | OLAP warehouse (distinct scale axis) |
| `finops-portal` | product | Per-tenant FinOps cost-attribution + chargeback portal |
| `feature-flags` | product | Feature-flag authoring UI (Cedar fragments) |
| `marketplace` | service-cell | Marketplace surface — ingestion/indexing/ranking/discovery backbone (universal deal-settlement; ADR-0249/0314) |
| `plugin-app-store` | product | Third-party plugin discovery + install (first marketplace category) |
| `developer-sdk` | product | Developer SDK catalog + docs + samples |
| `application` | product | Application Shell — B2B host that embeds the product surfaces (tenant-class model ADR-0330) |
| `app-shell-frontend` ✚ | product | Frontend app shell (Leptos; ADR-0393) |
| `ops-dashboard-control-center` | product | Ops/SRE control center (oyatie-internal) |

### 1.5 Substrate primitives (the shared spine — see §3)
`intelligence`, `ontology`, `policy` (+ `policy`-engine), `audit-chain`, `identity`,
`tenancy`/`tenant-rbac`, `observability`, `governance`, `api-gateway`, `comms-email`,
`consent-graph`, `compliance`, `detection`, `eventing`, `search`, `workflow-engine`,
`connector`. (Detailed in §3.)

### 1.6 Detection / risk substrate (ADR-0307 / 0309)
| Service | Tier | Inferred purpose |
|---|---|---|
| `detection` | substrate | Streaming+batch Detection Substrate — 8 detection families (payment fraud, ATO, synthetic identity, AML+sanctions, content abuse incl. CSAM, fake-reviews, insider risk, policy violation); the "D" in Detection→Risk→Mitigation→Prevention. Fairness/civil-rights invariants per ADR-0309. |

### 1.7 Bespoke substrate services (ADR-0476–0482 "bespoke-over-OSS")
Single-crate-per-service (ADR-0509). Replace named OSS with bespoke Rust.
| Service | Tier | Inferred purpose | Replaces |
|---|---|---|---|
| `oya-identity` ✚ | substrate | Bespoke Rust human-identity substrate (Wave C) | (human IdP) |
| `oya-billing` ✚ | substrate | Bespoke Rust billing engine (ADR-0478) | Lago |
| `oya-meter` ✚ | substrate | Bespoke Rust usage metering (ADR-0479) | (metering) |
| `oya-cost` ✚ | substrate | Bespoke Rust K8s cost-allocation (ADR-0480) | OpenCost |
| `oya-flags` ✚ | substrate | Bespoke Rust feature-flag server (OpenFeature remote eval, ADR-0481) | flagd/Unleash |
| `oya-authn-device-firmware` ✚ | substrate | Reference firmware for oyatie hardware security key (WebAuthn/OpenSK; ADR-0506-0508) | (HW key) |

### 1.8 CI/CD & SCM dogfood platform (oya-ci / Prow-in-Rust; ADR-0513 / 0511 / 0374)
| Service | Tier | Inferred purpose |
|---|---|---|
| `ci-controller` ✚ | substrate | Bespoke-Rust Prow-class CI controller (Forgejo + K8s adapters) |
| `ci-tide` ✚ | substrate | Tide-class merge automation (Forgejo adapter) |
| `ci-webhook-gateway` ✚ | substrate | First-hop gated change-coordination webhook gateway (Forgejo→Jenkins/CI) |
| `ops` ✚ | substrate | Ops docs-portal + workspace-shell crates (internal ops tooling) |
| `eventing` ✚ | substrate | Eventing backbone / outbox (ADR-0005) — crate-only scaffold |

> Retired/merged (tombstones, do NOT expect as live services): `foundry` (→ absorbed
> by `intelligence`, ADR-0335; "retired external agent harness" brand dropped), `cell` (→ pattern not service,
> ADR-0333; rebalancing/lifecycle carved into cloud `cell-rebalancer`/`cell-lifecycle`),
> `shorts` (→ merged into `social`, ADR-0334), `anonymous` (→ folded into `community`),
> `tenant-rbac` grouping wrapper (→ deprecated tombstone, ADR-0362).
> A `foundry/` dir + `tenant-rbac/` dir still exist on disk as legacy residue.

---

## 2. Services in `cloud/` (the cloud-provider / control plane)

Oyatie's own hyperscaler-equivalent. Substrate-tier; compute trajectory
phase-1 OCI+AWS → phase-2 hybrid colo → phase-3 own mega-DC (products.json).

### 2.1 Cloud compute / cluster / cell substrate
| Service | Purpose |
|---|---|
| `cloud-k8s` | Kubernetes cluster bootstrap, node lifecycle, control plane wrapper |
| `cloud-compute` | Compute scheduling primitive (crate-only) |
| `cloud-capacity` | Capacity model / planning (crate-only) |
| `cloud-cell` | Cell substrate primitive (crate-only) |
| `cell-lifecycle` | Logical Cell aggregate state machine (register→activate→promote→drain→decommission; ADR-0276/0351) |
| `cell-rebalancer` | Tenant migration across cells as long-running stateful workflow (ADR-0276/0351) |
| `tenancy` | Tenant registration / sub-scope hierarchy substrate |
| `cloud-tenancy` | Cloud-side tenancy primitive (crate-only) |

### 2.2 Cloud IAM / security / secrets
| Service | Purpose |
|---|---|
| `cloud-iam` | Cloud IAM (machine/service identity) |
| `cloud-kms` | CMK/KEK/DEK lifecycle, envelope encryption, HSM custody, rotation, cryptoshred, signing |
| `cloud-secrets` | SecretReference resolution, OpenBao namespace isolation, rotation, HSM, audit |

### 2.3 Cloud network / storage / data
| Service | Purpose |
|---|---|
| `cloud-network` | Tenant-scoped VPC-equivalent networking, ingress/egress policy, mTLS, flow telemetry, isolation |
| `cloud-network-dns` | Authoritative+recursive DNS, DNSSEC, health checks, routing, encrypted DNS, anycast |
| `cloud-storage` | Object/block storage primitive |
| `cloud-data` | Data-plane primitive |
| `cloud-iac` | Helm/Terraform/Kustomize module registry + IaC validation |

### 2.4 Cloud billing / finops / marketplace
| Service | Purpose |
|---|---|
| `cloud-billing` | Canonical source-of-truth for commercial state; owns `tenant_class ∈ {demo_trial, paid}` (ADR-0330) |
| `cloud-billing-tax` | Tax calc, jurisdiction evidence, filing handoffs |
| `cloud-finops` | Cloud FinOps primitive (crate-only) |
| `cloud-marketplace` | Cloud marketplace primitive (crate-only) |

### 2.5 Cloud intelligence / LLM gateway
| Service | Purpose |
|---|---|
| `cloud-intelligence` | Clean-room Rust LLM key-pool reverse-proxy gateway; OpenAI-compatible REST surface; Bedrock-on-Talos cloud primitive (`oya-invoke` capability port); multi-provider OAuth pool (Anthropic/OpenAI/Gemini) — ADR-0373/0384/0389/0390 |

### 2.6 Managed Kubernetes product surface (ADR-0376 — Oyatie's GKE/EKS/OKE)
Two-tier: hosted control plane (Kamaji pods, DEFAULT) + dedicated sovereign Talos spoke (PREMIUM).
| Service | Purpose |
|---|---|
| `managed-k8s-control-plane-host` | Hosts tenant control planes as pods (Kamaji/CAPI) |
| `managed-k8s-cluster-lifecycle` | Cluster provisioning / lifecycle |
| `managed-k8s-tenant-quota` | Per-tenant quota enforcement |
| `managed-k8s-sla-observability` | SLA + observability for managed clusters |

---

## 3. Substrate primitives (the shared spine every product composes on)

Per ADR-0245 substrate DAG (leaf → meta). These are audience-neutral capabilities;
products and capability-tiers project over them. Cross-tier rule: substrates MUST NOT
depend on products.

| Substrate | Concern | Notes / hyperscaler analogue |
|---|---|---|
| `identity` / `oya-identity` | OIDC, service principals, OAuth, WebAuthn/passkey | Zitadel-class → bespoke (ADR-0394/0476) |
| `tenancy` / `tenant-rbac` | Tenant rows, sub-scope hierarchy, reserved-namespace; tenant = universal scoping primitive | ADR-0242/0244 |
| `policy` / policy-engine | Cedar v4.x evaluation; universal authz/routing/retention gate | ADR-0150/0243/0246 |
| `audit-chain` | Merkle-sealed audit emission + evidence | ADR-0003/0263 (leaf substrate) |
| `ontology` | Object types, projections, cross-µservice entity reads; canonical data substrate | Palantir-Foundry-class (ADR-0006) |
| `consent-graph` | Consent state authoring + DSAR/DSR cascade | ADR-0038/0244 |
| `compliance` | Per-pack fragment registry + admission gate (regional/regulatory packs) | ADR-0010/0251 |
| `governance` | ~50 `oya-check-*` CI fitness lanes; substance-bar enforcement | ADR-0131/0322 |
| `comms-email` | Transactional email sending substrate (no UI; behind every product) | ADR-0245 |
| `api-gateway` | Tier-0 north-south edge admission, routing, rate-limit | Envoy/Cilium |
| `observability` | OTel + Mimir/Loki/Tempo/Grafana rollup, per-tenant metrics | ADR-0042/0383 |
| `workflow-engine` | Step-Functions-class durable orchestration (state-machine + DAG hybrid) | ADR-0035/0145 |
| `intelligence` | 2-layer AI substrate — inference, embeddings, RAG, agentic toolchains, eval/RLHF/red-team/model-registry (absorbed foundry) | ADR-0220/0255/0335 |
| `search` | Crawler + inverted/vector index + parser + query + RAG + rank + SERP | ADR-0030/0047 (crate-only) |
| `detection` | Streaming+batch fraud/abuse/risk detection (8 families) | ADR-0307/0309 |
| `connector` | Cross-system connector / integration surface | (connect meta) |
| `eventing` | Event backbone / outbox pattern | ADR-0005 |
| Marketplace substrates (ADR-0249, 8) | `marketplace-{catalog,inventory,orders,fulfillment,reviews,discovery,pricing,trust-safety}` — universal commerce/deal-settlement spine. **NOT separate top-level dirs on disk** (live under `marketplace/`); listed by doctrine. |
| Cloud substrates | compute/network/storage/secrets/kms/iam/iac (see §2) |
| **Reserved (ADR-0245 §D-3.D, NOT deployed)** | `payments`*, `identity-verification`, `tax-engine`, `deidentification`, `breach-notification`, `encryption-substrate`, `consent` — each certification-gated (PCI-DSS, ISO-18295, HIPAA, FIPS-140-3, GDPR/PIPA). `payments/` exists on disk as a populated dir but is reserved-tier. |

---

## 4. Product-domain ADRs and what they cover

### 4.1 Catalog shape & flat-catalog doctrine
| ADR | Covers |
|---|---|
| ADR-0001 | Cohesion thesis — one product, flat catalog, 6 shared substrates |
| ADR-0058 | Flat microservice catalog — Product Groups / Arm / Vertical retired |
| ADR-0131 | Per-microservice flat folder layout (universal artifact set) — AWS/Google/Stripe convention |
| ADR-0132 | Product/platform/bundle dissolution (no-grouping forward-policy) |
| ADR-0362 | FULL grouping retirement — flat-only catalog; removes grandfather; `no-grouping` gate made real |
| ADR-0245 | Substrate-vs-product layering (tier field: substrate/product/service-cell/reserved + SLO bars + dep-direction) |
| ADR-0509 | Hyperscaler service-decomposition pattern (single-crate-per-service + mod-based subsystems) |
| ADR-0512 | Canonical monorepo pattern |
| ADR-0357 | Vertical-slice monorepo nesting |
| ADR-0011 | Cross-microservice contract registry |
| ADR-0145 | Inter-microservice communication reform (3 invariants: audit/tracing/ontology; direct gRPC) |
| ADR-0028 | Cloud microservice architecture |
| ADR-0509/0510 | Hyperscaler decomposition + SCM destination (bespoke monorepo-VCS; Forgejo transitory) |

### 4.2 Coverage doctrines (how product breadth is achieved without sprawl)
| ADR | Covers |
|---|---|
| ADR-0315 | ERP coverage doctrine — SAP S/4HANA module parity WITHOUT a monolithic ERP platform |
| ADR-0316 | Capability-tier OVER product fragmentation — CRM/HR/ITSM/marketing/CLM/LMS/FP&A as tenant activation bundles over substrate (superseded-in-text by ADR-0329) |
| ADR-0321 | B2B SaaS industry-leader coverage beyond SAP (Salesforce/Workday/ServiceNow/Atlassian/Okta/CrowdStrike…) via capability tiers; 244 vendor rows; target 69+ services |
| ADR-0249 | Multi-category marketplace doctrine — Amazon+FB-Marketplace+App-Store+Upwork+Substack under one brand; 8 shared substrates + 4 category contexts |
| ADR-0314 | Marketplace as universal deal-settlement substrate (every tenant↔tenant / tenant↔consumer exchange) |
| ADR-0332 | Healthcare domain decomposition — split healthcare-integration (215 features/14 domains) into 8 domain µservices |
| ADR-0307 | Detection substrate (streaming+batch) — 8 detection families; 8 substrate primitives |
| ADR-0309 | Detection fairness + civil-rights compliance (EU AI Act / ECOA / NY AEDT / 4-5ths rule) |

### 4.3 Connect / super-app / social
| ADR | Covers |
|---|---|
| ADR-0029 | Connect dual-context architecture (Professional B2B + Personal B2C); Workspace/M365/Naver-Works replacement |
| ADR-0234 | Connect social-expansion planning contract (planning, not production claim) |
| ADR-0235 | Connect core public contracts (Workflow/Ontology-mediated) |
| ADR-0237 | Connect dissolution — Strangler-pattern migration to 8 flat µservices |
| ADR-0238 | Connect super-app expansion — the 8-µservice topology + retirement trigger |
| ADR-0334 | shorts merged into social (TikTok-style short-video flavor) |

### 4.4 Tenant-class / pricing / packaging
| ADR | Covers |
|---|---|
| ADR-0013 | Product license policy |
| ADR-0325 | Capability-tier pricing anchors (public) |
| ADR-0329 | Tier system retired → replaced by tenant-class |
| ADR-0330 | tenant-class (demo_trial vs paid) + composable billing components |
| ADR-0331 | Cross-microservice tenant-class adoption template |
| ADR-0313 | Conglomerate tenant hierarchy (sovereign children) |
| ADR-0311 | Dual-tenant identity (personal vs work boundary) |

### 4.5 Retirement / merge ADRs (services that are NOT live)
| ADR | Covers |
|---|---|
| ADR-0333 | cell µservice retired — cellular architecture kept as a pattern |
| ADR-0335 | foundry retired — absorbed by intelligence; "retired external agent harness" dropped |
| ADR-0334 | shorts merged into social |
| ADR-0351 | cell-rebalancer + cell-lifecycle carved out of the cell absorption |
| ADR-0363 | Agentic VCS foundry → intelligence + Forgejo substrate |

### 4.6 Bespoke-substrate roadmap (build-everything ambition)
| ADR | Covers |
|---|---|
| ADR-0482 | Bespoke substrate roadmap — multi-decade kernel+OS ambition, phased timeline + bridges |
| ADR-0476 | oya-identity — bespoke human identity |
| ADR-0478 | oya-billing — bespoke billing engine (supersedes Lago) |
| ADR-0479 | oya-meter — bespoke usage metering |
| ADR-0480 | oya-cost — bespoke K8s cost allocation (supersedes OpenCost) |
| ADR-0481 | oya-flags — bespoke feature-flag server (OpenFeature) |
| ADR-0394 | Bespoke-Rust IDP central hub (Leptos portal + ops-BFF) |
| ADR-0506/0507/0508 | aws-lc-rs crypto / webauthn-rs RP / OpenSK authenticator |

### 4.7 Cloud-provider & managed-K8s product surface
| ADR | Covers |
|---|---|
| ADR-0028 | Cloud microservice architecture |
| ADR-0032 | DCIM software for own DC ops |
| ADR-0376 | Managed Kubernetes product surface (two-tier: Kamaji hosted + Talos sovereign) |
| ADR-0389 | cloud-intelligence — Bedrock-on-Talos as a cloud primitive |
| ADR-0390 | cloud-intelligence v1 request pipeline + proof layer |
| ADR-0373 | LLM gateway production design (OpenAI-compatible surface) |
| ADR-0384 | LLM gateway OAuth subscription-pool redesign |
| ADR-0509/0510/0513/0511 | Hyperscaler decomposition / SCM destination / oya-ci Prow-in-Rust / Argo supersede Jenkins |

### 4.8 Search & ads/analytics
| ADR | Covers |
|---|---|
| ADR-0030 | Search microservice architecture |
| ADR-0031 | Ads + analytics microservice architecture |
| ADR-0046/0047 | Vector store / search backend strategy |

### 4.9 Foundry agent-platform (now folded into intelligence)
ADR-0020–0027, 0136, 0293, 0347 — multi-provider adapter model, capability registry +
MCP gateway, autonomy ceiling, wasmtime/firecracker sandbox, eval harness + replay,
engineering-platform, in-house AI model substrate roadmap, robotics/vision/speech
sub-substrates. (Operational role preserved inside `intelligence` per ADR-0335.)

---

## 5. Cross-cutting catalog entities (not "products" but tracked)

- **Regional packs** (`docs/regional-packs/`): one pack per locale — KR (full), JP/US/EU
  (skeleton), IN/BR/KSA/UAE/AU/SG (planned). Each: i18n + regulatory + payment-rails +
  identity + tax. (ADR-0010 regional-pack architecture; ADR-0240 sovereign cloud per pack.)
- **Compliance packs**: PCI-DSS, HIPAA/FHIR/HL7, GDPR/EU-AI-Act, KR CSAP/PIPA/FSS,
  FedRAMP-High/IL5, SOX/GLBA, PIPL/CSL/DSL, LGPD, PDPA/MAS (ADR-0251).
- **Capability-tier registry**: 244 vendor rows mapping SaaS incumbents → tiers over
  substrate (coverage matrix).
- **Engineering teams** (`docs/teams/`): one charter per team (axis-*/vertical-*/council-*).

---

## 6. STALE vs LIVE reconciliation (for the diff)

| Artifact | Model it asserts | Status |
|---|---|---|
| `products.json` | 7 axes / 14 verticals / 10 regional packs | **STALE** (2026-05-09; axis/vertical retired) |
| `docs/products/*/PRD.md` | per-axis + per-vertical PRDs | partially stale (axis/vertical framing); slice content still referenced |
| coverage matrix (2026-05-21) | 56 µservices, 69+ target, 244 vendor rows, flat+capability-tier | **LIVE doctrine** |
| `oya/` + `cloud/` on-disk tree | 87 + 25 dirs, flat single-concern | **LIVE ground truth** (some crate-only WIP + bespoke + tombstone residue) |

Diff guidance for the legacy inventory: treat the **flat `oya/`+`cloud/` tree as
authoritative for "what exists"**, the **coverage-matrix + ADRs as authoritative for
"what is intended / how it's organized"**, and flag any legacy entry that maps to a
retired/merged service (foundry, cell, shorts, anonymous, the axis/vertical labels) as
a rename/absorption rather than a deletion.
