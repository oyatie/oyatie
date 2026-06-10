# Oya Verticals Inventory — Service Dirs n–z (source-backed)

Scope: every directory under `/Users/jasonlee/Developer/source/oya/` whose name starts n–z.
Method: `ls` of each service's `crates/` (or root `Cargo.toml` search where no `crates/`). Tree = `oya/` for every dir in scope (this whole subtree is the PRODUCT tree; `cloud/` is a sibling platform tree not in scope). Layer suffixes read as clean-arch lanes: `kernel`/`domain`/`usecase`/`app`/`api`/`rest`/`grpc`/`sdk`/`worker`. `-adapter-<x>` crates = swappable PORT impls (mobility seams). Cohesion = single-BC crate cluster; Detachment = multi-BC, independently scalable.

NO SILENT CAPS — every n–z dir is accounted for below, including the 6 that have NO Rust crates (design/docs-only or stubs).

---

## A. SERVICES WITH CRATES (clean-arch implemented)

### notes — oya/ (product, Workspace/productivity)
- crates: `oya-notes-domain`
- LAYER: domain only. BC: notes (1). PORTS/ADAPTERS: none.
- COHESION: single-BC, domain-only skeleton (vertical seeded, not yet built out).
- Role: G-Suite-style "Keep/Notes" surface in the workspace product axis.

### observability — oya/ (product OBSERVABILITY vertical; note it bridges into platform)
- crates: `oya-cloud-observability-api`, `oya-cloud-observability-domain`, `oya-cloud-observability-kernel`, `oya-observability-domain`, `oya-observability-tracing-adapter`
- LAYER: kernel + domain + api (the `oya-cloud-observability-*` trio = a fuller stack), plus a second `oya-observability-domain`. PORTS/ADAPTERS: `oya-observability-tracing-adapter` (swappable tracing backend seam; `publish=false`, Apache-2.0).
- BC: two parallel slices — a "cloud-observability" (kernel/domain/api) slice and a plain "observability" domain + tracing-adapter slice. Mild detachment (2 slices).
- Role: Datadog/Grafana-class observability — the product-facing telemetry plane (distinct from internal SLO docs).

### ontology — oya/ (product, data/knowledge layer)
- crates: `oya-ontology-api`, `oya-ontology-domain`, `oya-ontology-kernel`, `oya-ontology-query-engine-domain`, `oya-ontology-query-engine-usecase`, `oya-resolve-scorecards-app`
- LAYER: kernel→domain→api (core ontology) + query-engine domain→usecase (sub-BC) + `oya-resolve-scorecards-app` (app).
- BC: ontology core + query-engine (2 BCs) → DETACHMENT (query engine separable from the ontology model). PORTS/ADAPTERS: none yet.
- Role: Palantir-Foundry-style ontology/semantic layer; `resolve-scorecards-app` = a consumer app over it.

### ops — oya/ (product, internal-ops / operator workspace)
- crates: `oya-ops-docs-portal-adapter`, `oya-ops-docs-portal-kernel`, `oya-ops-docs-portal-rest`, `oya-ops-docs-portal-usecase`, `oya-ops-workspace-shell-adapter`, `oya-ops-workspace-shell-app`, `oya-ops-workspace-shell-kernel`, `oya-ops-workspace-shell-rest`, `oya-ops-workspace-shell-usecase`
- LAYER: two FULL clean-arch stacks — docs-portal (kernel/usecase/rest/adapter) and workspace-shell (kernel/usecase/app/rest/adapter).
- BC: docs-portal + workspace-shell (2 BCs) → DETACHMENT. PORTS/ADAPTERS: `oya-ops-docs-portal-adapter`, `oya-ops-workspace-shell-adapter` (un-suffixed adapter seams — outbound integration ports).
- Role: operator/admin shell + ops docs portal (the runnable counterpart to the docs-only ops-dashboard-control-center).

### payments — oya/ (product, FinTech core)
- crates: `oya-payments-adapter-adyen`, `oya-payments-adapter-stripe`, `oya-payments-charge-{app,domain,grpc,kernel,rest,usecase}`, `oya-payments-dispute-{domain,usecase}`, `oya-payments-kyc-kyb-{domain,usecase}`, `oya-payments-payout-{domain,usecase}`, `oya-payments-refund-{domain,usecase}`, `oya-payments-settlement-{domain,worker}`, `oya-payments-subscription-{domain,usecase}`
- LAYER: charge is the flagship full stack (kernel/domain/usecase/app/rest/grpc); the other BCs are domain(+usecase/worker) slices.
- BC: charge, dispute, kyc-kyb, payout, refund, settlement, subscription = 7 BCs → strong DETACHMENT (each independently scalable; settlement runs a `worker`).
- PORTS/ADAPTERS: `oya-payments-adapter-stripe` + `oya-payments-adapter-adyen` (PSP gateway seam — swap payment processors; classic mobility). (stripe adapter has BUCK+Cargo+src, real impl.)
- Role: Stripe/Adyen-class payments platform — the FinTech product axis.

### payroll — oya/ (product, HR/FinTech)
- crates: `oya-payroll-run-api`, `oya-payroll-run-app`, `oya-payroll-run-domain`, `oya-payroll-run-runtime`, `oya-payroll-run-storage-adapter-inmemory`
- LAYER: domain→app→api + runtime. PORTS/ADAPTERS: `oya-payroll-run-storage-adapter-inmemory` (storage port; inmemory impl — swap for prod store).
- BC: payroll-run (1) → COHESION, single-BC full stack with storage seam.
- Role: Gusto/Workday-payroll engine; HR product axis.

### performance-management — oya/ (product, HR)
- crates: `oya-performance-management-review-calibration-service`
- LAYER: a single `-service` crate (composed service, not split into lanes). BC: review-calibration (1) → COHESION. PORTS/ADAPTERS: none.
- Role: Workday/Lattice performance-review + calibration; HR product axis.

### plant-maintenance — oya/ (product, ERP/manufacturing)
- crates: `oya-plant-maintenance-domain`, `oya-plant-maintenance-work-order-app`
- LAYER: domain + work-order app. BC: plant-maintenance core + work-order (1 BC, 1 app) → COHESION. PORTS/ADAPTERS: none.
- Role: SAP PM (EAM) — ERP/manufacturing product axis.

### policy — oya/ (product/platform-shared, authorization)
- crates: `oya-policy-cedar-api`, `oya-policy-cedar-domain`
- LAYER: domain→api. BC: cedar policy (1) → COHESION. PORTS/ADAPTERS: none (Cedar IS the policy engine here).
- Role: AWS-Cedar authorization service — cross-cutting authz primitive surfaced as a product/platform crate.

### production-planning — oya/ (product, ERP/manufacturing)
- crates: `oya-production-planning-domain`, `oya-production-planning-mrp-app`
- LAYER: domain + mrp app. BC: production-planning + MRP (1 BC) → COHESION. PORTS/ADAPTERS: none.
- Role: SAP PP / MRP planning; ERP product axis.

### quality-management — oya/ (product, ERP/manufacturing)
- crates: `oya-quality-management-domain`, `oya-quality-management-inspection-app`
- LAYER: domain + inspection app. BC: QM + inspection (1 BC) → COHESION. PORTS/ADAPTERS: none.
- Role: SAP QM; ERP product axis.

### real-estate — oya/ (product, vertical SaaS)
- crates: `oya-real-estate-lease-app`, `oya-real-estate-portfolio-domain`
- LAYER: portfolio domain + lease app. BC: portfolio + lease (could read as 2) → mild detachment but really one vertical → COHESION. PORTS/ADAPTERS: none.
- Role: real-estate / lease-portfolio management; industry-vertical product axis.

### recordings — oya/ (product, Meet/comms adjacency)
- crates: `oya-recordings-domain`
- LAYER: domain only. BC: recordings (1) → COHESION. PORTS/ADAPTERS: none.
- Role: meeting/call recordings store (companion to meet/contact-center); communications product axis.

### search — oya/ (product, data/search platform)
- crates: `oya-search-crawler-domain`, `oya-search-index-inverted-domain`, `oya-search-index-vector-domain`, `oya-search-parser-domain`, `oya-search-query-domain`, `oya-search-rag-domain`, `oya-search-rank-domain`, `oya-search-serp-domain`
- LAYER: all `-domain` (pure model lanes, no app/api yet). BC: crawler, index-inverted, index-vector, parser, query, rag, rank, serp = 8 BCs → strong DETACHMENT (each pipeline stage independently scalable). PORTS/ADAPTERS: none yet (inverted vs vector index are sibling BCs, not adapters of one port — a design seam to watch).
- Role: Elastic/Algolia + RAG search platform — the search/AI-retrieval product axis.

### sheets — oya/ (product, Workspace)
- crates: `oya-sheets-domain` — domain only, BC=sheets(1), COHESION, no adapters.
- Role: Google-Sheets-class spreadsheet; workspace product axis.

### sites — oya/ (product, Workspace)
- crates: `oya-sites-domain` — domain only, BC=sites(1), COHESION, no adapters.
- Role: Google-Sites-class site builder; workspace product axis.

### slides — oya/ (product, Workspace)
- crates: `oya-slides-domain` — domain only, BC=slides(1), COHESION, no adapters.
- Role: Google-Slides-class presentations; workspace product axis.

### supply-chain-planning — oya/ (product, ERP/SCM)
- crates: `oya-supply-chain-planning-domain`, `oya-supply-chain-planning-network-app`
- LAYER: domain + network app. BC: SCP + network (1 BC) → COHESION. PORTS/ADAPTERS: none.
- Role: SAP IBP / Kinaxis supply-chain planning; ERP/SCM product axis.

### tasks — oya/ (product, Workspace/productivity)
- crates: `oya-tasks-domain` — domain only, BC=tasks(1), COHESION, no adapters.
- Role: Asana/Google-Tasks; productivity product axis.

### tenant-rbac — oya/ (product, MULTI-TENANCY platform substrate)  ★ richest service in scope
- crates (39): `oya-tenant-rbac-api`, `-application`, `-audit-chain-emission`, `-audit-chain-runtime-evidence`, `-auth-runtime`, `-cloud-deployment-evidence`, `-cloud-deployment-manifest`, `-cloud-readiness-gate`, `-disbursement-evidence`, `-domain`, `-erp-parity-map`, `-identity-provider-runtime-evidence`, `-identity-provider-verification`, `-listener-gateway`, `-listener-runtime-evidence`, `-local-inmemory-harness`, `-local-runtime-composition`, `-postgres-rls-runtime-evidence`, `-postgres-rls-storage`, `-postgres-rls-transaction-contract`, `-postgres-rls-write-contract`, `-runtime`, `-slo-evidence`, `-statutory-filing-evidence`, `-storage-adapter-inmemory`, `-tenant-admission-policy`, `-tenant-autoscaling-contract`, `-tenant-availability-contract`, `-tenant-cost-allocation-contract`, `-tenant-egress-policy-contract`, `-tenant-image-provenance-contract`, `-tenant-residency-contract`, `-tenant-resource-quota-contract`, `-tenant-secret-boundary-contract`, `-tenant-workload-identity-contract`, `-tenant-workload-manifest`, `-tenant-workload-runtime-evidence`, `-workflow-adapter-inmemory`, `-workflow-runtime-evidence`
- LAYER: domain→application→api + runtime + auth-runtime + listener-gateway/runtime + local-runtime-composition + local-inmemory-harness.
- PORTS/ADAPTERS: `-storage-adapter-inmemory` (storage port), `-workflow-adapter-inmemory` (workflow port), plus the postgres-RLS storage family (`-postgres-rls-storage` is the real backing impl behind the storage port; transaction/write *-contract* crates pin the seam). Strong mobility design: inmemory vs postgres-rls = the swap.
- DETACHMENT: very high — a wall of `*-contract` crates (admission, autoscaling, availability, cost-allocation, egress, image-provenance, residency, resource-quota, secret-boundary, workload-identity) + `*-evidence` crates. These are not separate product BCs but a contract+evidence lattice over the one tenancy BC — i.e. cohesion of purpose, detachment of verification surfaces.
- Role: the multi-tenancy / RBAC / tenant-isolation control substrate (hyperscaler "cell/tenant" boundary). Functions as platform infra exposed in the product tree; the `-cloud-deployment-*`, `-cloud-readiness-gate`, `erp-parity-map`, `statutory-filing-evidence`, `disbursement-evidence` crates show it doubling as the cross-cutting compliance/cloud-readiness anchor.

### translate — oya/ (product, AI/Workspace)
- crates: `oya-translate-domain` — domain only, BC=translate(1), COHESION, no adapters.
- Role: Google-Translate-class translation; AI/workspace product axis.

### treasury — oya/ (product, FinTech)
- crates: `oya-treasury-cash-app`, `oya-treasury-cash-domain`
- LAYER: cash domain + cash app. BC: treasury-cash (1) → COHESION. PORTS/ADAPTERS: none.
- Role: corporate treasury / cash management; FinTech product axis.

### warehouse — oya/ (product, ERP/logistics)
- crates: `oya-warehouse-fulfillment-app`, `oya-warehouse-inventory-domain`
- LAYER: inventory domain + fulfillment app. BC: inventory + fulfillment (1 vertical) → COHESION. PORTS/ADAPTERS: none.
- Role: SAP EWM / WMS warehouse mgmt; ERP/logistics product axis.

### whiteboard — oya/ (product, collaboration)
- crates: `oya-whiteboard-canvas-collaboration-app`
- LAYER: single collaboration app crate. BC: canvas-collaboration (1) → COHESION. PORTS/ADAPTERS: none.
- Role: Miro/FigJam collaborative whiteboard; collaboration product axis.

### workflow-engine — oya/ (product/platform, WORKFLOW substrate)  ★ canonical DETACHMENT example
- crates (43): four BCs each with their own clean-arch stack + a SaaS shell:
  - SaaS shell: `oya-saas-workflow-app`, `oya-saas-workflow-domain`, `oya-saas-workflow-kernel`
  - **event-bus** BC: `-event-bus-{kernel,domain,usecase,app,api,rest,sdk,worker}` + adapter seam `-event-bus-adapter` and impls `-event-bus-adapter-{kafka,nats,postgres,pulsar,redpanda,valkey}`
  - **execution-engine** BC: `-execution-engine-{kernel,domain,usecase,app,api,rest,sdk,worker}` + `-execution-engine-adapter` and impl `-execution-engine-adapter-postgres`
  - **state-machine** BC: `-state-machine-{kernel,domain,usecase,api}` + `-state-machine-adapter` and impl `-state-machine-adapter-postgres`
  - **trigger-orchestrator** BC: `-trigger-orchestrator-{kernel,domain,usecase,app,api,rest,sdk,worker}`
- PORTS/ADAPTERS: the marquee mobility surface — event-bus port has SIX swappable broker impls (kafka / nats / pulsar / redpanda / valkey / postgres); execution-engine + state-machine each have a postgres adapter behind their port.
- DETACHMENT: maximal — 4 independently scalable BCs (event-bus, execution-engine, state-machine, trigger-orchestrator) each with full kernel→domain→usecase→app→api→rest→sdk→worker lanes. This is the textbook "multi-BC, independently scalable" service.
- Role: Temporal/Argo-Workflows-class durable workflow engine — the orchestration/workflow product+platform axis.

### workflow-studio — oya/ (product, low-code authoring over workflow-engine)
- crates: `oya-workflow-studio-dsl-emitter-domain`, `oya-workflow-studio-dsl-loader-domain`, `oya-workflow-studio-policy-preview-domain`, `oya-workflow-studio-visual-canvas-kernel`
- LAYER: three domain crates (dsl-emitter, dsl-loader, policy-preview) + one kernel (visual-canvas).
- BC: dsl-emitter, dsl-loader, policy-preview, visual-canvas = 4 BCs → DETACHMENT (authoring pipeline stages). PORTS/ADAPTERS: none yet.
- Role: the visual/low-code authoring front-end that emits DSL for workflow-engine (Temporal-UI / n8n-canvas analog).

### workplace-integration — oya/ (product, integration)
- crates: `oya-workplace-integration-doc-set-scaffold`
- LAYER: a single scaffold crate (doc-set scaffold — early/seed). BC: workplace-integration (1) → COHESION. PORTS/ADAPTERS: none.
- Role: Slack/Teams/Workspace integration connector hub (seeded, not built out).

### oya-authn-device-firmware — oya/ (product, security/hardware authn)
- crates: `oya-authn-device-firmware` (single crate; `crates/oya-authn-device-firmware/`)
- src modules: `attestation/`, `ctap2/`, `transport/`, `storage/`, `observability/`, `config.rs`, `lib.rs`
- LAYER: single firmware crate (no clean-arch suffix split; module-internal layering). BC: device-authn-firmware (1) → COHESION. PORTS/ADAPTERS: none as crates (internal `transport/`, `storage/` modules are the swap seams).
- Role: FIDO2/CTAP2 hardware authenticator firmware (YubiKey-class); security/identity hardware product axis.

---

## B. SERVICES WITH NO RUST CRATES (design/docs-only or stubs) — explicitly accounted, not capped

- **ops-dashboard-control-center** — oya/ (product). DOCS-ONLY: no `crates/`, no `Cargo.toml` anywhere. Rich design corpus (PRD, ARCH, ~30 IP-journey-* files, manifest.json, threat-models, cedar, iac). It is the spec for an internal operator/control-plane console; the runnable counterpart is the `ops` service. Hyperscaler role: AWS-Console / GCP-Cloud-Console operator surface.
- **plugin-app-store** — oya/ (product). DOCS-ONLY: no `crates/`, no `Cargo.toml`. Large design corpus (PRD, packs/, ~25 IP-journey-* incl. marketplace/publish/install/quarantine, sdk-plan, deprecation-plan). Hyperscaler role: AWS-Marketplace / Salesforce-AppExchange plugin & app marketplace substrate.
- **social** — oya/ (product). DESIGN-ONLY: no `crates/`, no `Cargo.toml`. Full IP set (IP-001..IP-018 covering user-profile/follow-graph/post-composition/feed-timeline/reactions/mentions/trending/notifications/moderation BCs + DSA/minor-protection overlays). Hyperscaler role: a social-network product (Twitter/Threads-class) fully specced, awaiting implementation.
- **patient-monitoring** — oya/ (product, HealthTech). DESIGN-ONLY: no `crates/`, no `Cargo.toml`; has implementation-plans/, design-spec-maturity/, supported-oses.json, PRD/ARCH. Role: ICU/remote patient-monitoring (Epic/Philips-class); HealthTech product axis.
- **pharmacy** — oya/ (product, HealthTech). DESIGN-ONLY: identical shape to patient-monitoring (implementation-plans/, design-spec-maturity/, supported-oses.json, no crates/Cargo). Role: pharmacy/e-prescribing (Epic-Willow-class); HealthTech product axis.
- **oya-billing / oya-cost / oya-flags / oya-identity / oya-meter** — oya/ (product). STUBS: each has only `BUCK`, `catalog.yaml`, `README.md`, `slos/availability.openslo.yaml` — NO crates, NO src. Newer (May 31) catalog/SLO placeholders for billing, cost-allocation, feature-flags, identity, and metering planes (likely the consolidated successors to the larger `billing`/`feature-flags`/`identity` services that live in the a–m half). Counted here as 5 stub entries.

---

## C. PATTERN SUMMARY (n–z half)

- **Tree**: 100% `oya/` (product). No `cloud/` platform dirs fall in n–z scope.
- **Cohesion (single-BC)** majority: notes, payroll, performance-management, plant-maintenance, policy, production-planning, quality-management, real-estate, recordings, sheets, sites, slides, supply-chain-planning, tasks, translate, treasury, warehouse, whiteboard, workplace-integration, oya-authn-device-firmware.
- **Detachment (multi-BC, independently scalable)**: workflow-engine (4 BCs, 6-way event-bus adapter swap — the exemplar), payments (7 BCs + dual PSP adapters), search (8 pipeline BCs), tenant-rbac (1 BC but a vast contract/evidence + inmemory↔postgres-rls adapter lattice), ops (2 BCs), ontology (2 BCs), workflow-studio (4 authoring BCs), observability (2 slices).
- **Mobility seams (`-adapter-*` ports) present in**: workflow-engine (event-bus: kafka/nats/pulsar/redpanda/valkey/postgres; execution-engine & state-machine: postgres), payments (PSP: stripe/adyen), tenant-rbac (storage: inmemory/postgres-rls; workflow: inmemory), payroll (storage: inmemory), observability (tracing-adapter), ops (docs-portal & workspace-shell outbound adapters). Everything else has no externalized port yet.
- **Maturity gradient**: full clean-arch stacks (workflow-engine, payments, tenant-rbac, ops, ontology, payroll) → domain+app pairs (ERP family: plant-maintenance/production-planning/quality-management/supply-chain-planning/warehouse/treasury/real-estate) → domain-only seeds (notes/sheets/sites/slides/tasks/translate/recordings + the 8 search domains) → design-only (social, patient-monitoring, pharmacy, ops-dashboard-control-center, plugin-app-store) → stubs (oya-billing/cost/flags/identity/meter).
