# 00 — Verticals Map (Cloud Platform + Oya Product)

> **READ-ONLY synthesis.** Source-backed from the REAL tree under
> `/Users/jasonlee/Developer/source` (service dirs, `crates/` listings, `Cargo.toml`
> package names, `BUCK` files, `manifest.json` bounded-contexts/tier) — **NOT from ADRs**.
> Aggregated from the six lane files in this dir:
> `10-cloud-verticals.md`, `10-oya-verticals-a.md`, `10-oya-verticals-d.md`,
> `10-oya-verticals-n.md` (+ tech-stack context from `10-techstack-now.md`,
> `10-techstack-roadmap.md`).
>
> **Clean-arch lens.** Crate role suffixes = layers (inner→outer):
> `kernel` (pure invariants) · `domain` (entities/VOs) · `usecase` (application
> services) · `app` (composition root) · `api` (typed port surface) ·
> `rest`/`grpc`/`graphql`/`sse`/`websocket` (transport drivers) · `sdk` (client) ·
> `worker` (async runner) · `runtime` (process host). **`-adapter-<x>` + `-api`/`-rest`
> = the PORTS/ADAPTERS = the mobility seams** (swap impl without touching the domain).
> One crate-set per bounded-context = **COHESION**; multiple independently-scalable
> BCs in one service dir = **DETACHMENT-for-scale**.

---

## 0. Counts (no silent caps)

| Tree | Total dirs | With Rust crates | Spec/doc-only or non-Rust / stub |
|---|---|---|---|
| **CLOUD platform** (`cloud/`) | **25** | 22 | 3 (`cell-lifecycle`, `cell-rebalancer`, `cloud-k8s`) |
| **OYA product** (`oya/`) — a–c | 19 | 15 | 4 (api-gateway, comms-email, consent-graph doc-only; app-shell-frontend = TS) |
| **OYA product** (`oya/`) — d–m | 30 | 24 | 6 spec-only (detection, diagnostics, finops-portal, governance, healthcare-integration, imaging) |
| **OYA product** (`oya/`) — n–z | 37 | 26 | 11 (5 docs/design-only + 5 catalog stubs + 1 dup span) |
| **OYA total** | **86** | **65** | **21** (15 doc/design-only · 5 catalog stubs · 1 TS frontend) |
| **GRAND TOTAL** | **111 verticals** | **87 with crates** | **24 without** |

> Spec-only / stub dirs are **listed, not dropped** (see §3 and §4 end-notes). They
> carry design contracts (OpenAPI/AsyncAPI/proto), IP-journey plans, PRDs, threat-models,
> or catalog/SLO placeholders — i.e. specified but not yet code.

---

## (a) CLOUD VERTICALS (`cloud/`, by hyperscaler substrate class)

| Service | Bounded contexts | Clean-arch layers present | Ports/adapters (mobility seams) | Cohesion / Detachment (why) |
|---|---|---|---|---|
| **cloud-iam** | iam (1) | domain · app · api | `adapter-oci` ↔ `adapter-selfhosted` | **COHESION** — single IAM control-plane BC |
| **cloud-kms** | kms (1) | domain · api | `adapter-oci` ↔ `adapter-openbao` | **COHESION** — single KMS BC |
| **cloud-secrets** | secrets (1) | domain | `secrets-file-adapter` (only built; openbao+HSM PLANNED IP-006/011) | **COHESION** — secret-resolution BC |
| **cloud-compute** | vm · k8s · functions (+dcops, resource) | domain ×3 · api ×3 | `adapter-aws` ↔ `adapter-oci` | **DETACHMENT** — 3 API surfaces (IaaS VM / managed-k8s / FaaS) over shared domain; real home of cloud-k8s code |
| **cloud-cell** | cell · region · regional-pack (3) | domain/app + domain/api ×2 | — | **DETACHMENT** — cellular-isolation topology, 3 BC api/domain pairs |
| **cloud-capacity** | capacity (1) | kernel · domain | — (inner-only) | **COHESION** — capacity/headroom substrate |
| **cell-lifecycle** *(spec-only, 0 crates)* | cell-lifecycle (planned) | — (contracts + ARCH only) | — | planned cell lifecycle controller |
| **cell-rebalancer** *(spec-only, 0 crates)* | cell-rebalance (planned) | — (contracts + runbooks) | — | planned cross-cell placement/rebalance |
| **cloud-k8s** *(no crates dir)* | → owner `axis-cloud-compute` | — (OpenAPI/proto + IP-* planning) | (PLANNED kubeadm/istio/envoy/csi adapters) | code lives in `cloud-compute/`; cloud-k8s crate names are PLANNED IP-* targets only |
| **managed-k8s-cluster-lifecycle** | cluster-lifecycle (1) | kernel · api · app | — (no external adapter yet) | **COHESION** — managed-cluster create/upgrade/delete |
| **managed-k8s-control-plane-host** | control-plane-host (1) | kernel · api · app | `adapter-capi` ↔ `adapter-inmemory` | **COHESION** — hosted control-plane provisioning |
| **managed-k8s-sla-observability** | sla-observability (1) | kernel · api · app | `adapter-inmemory` (real sink planned) | **COHESION** — managed-cluster SLA/telemetry |
| **managed-k8s-tenant-quota** | tenant-quota (1) | kernel · api · app | `adapter-cedar` ↔ `adapter-inmemory` | **COHESION** — per-tenant quota (policy-enforced) |
| **cloud-storage** | object · block (2) | domain · api ×2 | `adapter-s3` ↔ `adapter-oci` | **DETACHMENT** — object (S3-style) + block (EBS-style) over shared domain |
| **cloud-data** | data (1) | kernel · domain | — (inner-only) | **COHESION** — managed-data substrate invariants |
| **cloud-network** | vpc · lb · residency (3) | domain · api ×2 (+ residency-domain) | `adapter-oci` ↔ `adapter-selfhosted` | **DETACHMENT** — VPC + LB + co-located residency BC |
| **cloud-network-dns** | dns (1) | api | — (thin/early) | **COHESION** — managed DNS (Route53-style), split out for scale |
| **cloud-billing** | billing · metering (2) | kernel · domain (+ bench app) | — | **DETACHMENT** — metering (usage) separable from billing (invoicing) |
| **cloud-billing-tax** | tax (1) | app | — | **COHESION** — tax calc, detached from core billing |
| **cloud-finops** | finops (+cost) (1+) | kernel · domain · api | — | **COHESION** — FinOps/cost (budgets, showback/chargeback) |
| **cloud-marketplace** | cloud-marketplace · saas-plugin-marketplace (2) | kernel · domain | — | **DETACHMENT** — cloud listing vs in-product plugin marketplace |
| **cloud-iac** | iac (1) | domain · app · api · **rest** · runtime | `-rest` (built); renderer-trio argocd/flux/opentofu PLANNED IP-005 | **COHESION** — GitOps renderer/applier; only `cloud/` svc with a present `-rest` |
| **cloud-tenancy** | tenancy-cli (1) | (cli) | — | **COHESION** — thin operator CLI to `tenancy` |
| **tenancy** ★ | core + tenant-lifecycle, isolation-policy, cell-assignment, dsr-cascade, lifecycle-locks, sub-scope-registry, kyb-kyc, dr-pairing, per-tenant-quota, reserved-namespace (10+ feature BCs) | api · domain · kernel + 6 feature-kernels + kyb-kyc domain + 3 usecases | `data-residency-enforcer-adapter` (+ postgres adapters PLANNED IP-005/020/023) | **DETACHMENT (canonical exemplar)** — 14 crates, fan of independently-evolving feature kernels/usecases |
| **cloud-intelligence** ★ | intelligence (1, rich adapter fan) | kernel · app · rest | `authz-cedar-adapter` · `codex-adapter` · `eventsink-clickhouse-adapter` ↔ `eventsink-valkey-adapter` · `openbao-adapter` (5 ports) | **COHESION** of BC, **richest cloud adapter fan** — swappable eventsink = event-bus-style seam |

**Cloud substrate grouping:** IAM/KMS/Secrets · Compute/K8s/Cell (+ managed-k8s quartet) ·
Storage/Data · Network/DNS · Billing/FinOps/Marketplace · IaC/Tenancy · Intelligence.

---

## (b) PRODUCT VERTICALS (`oya/`, by product axis)

### saas-substrate / shared substrate (governance · integration · eventing · identity · CI · AI · flags · search · workflow · tenancy)

| Service | Bounded contexts | Layers present | Ports/adapters | Cohesion / Detachment |
|---|---|---|---|---|
| **audit-chain** | emission · query · retention-cascade · sealing · verification (5) | kernel+domain+api per BC, shared domain/usecase | `file-adapter` (audit sink) | **DETACHMENT** — 5 BCs, crate-per-BC |
| **compliance** | dlp · dsr · ediscovery · retention · trust-portal (5) | domain ×5 (+1 usecase) | — (early) | **DETACHMENT** — 5 compliance BCs |
| **connector** | 10 vendor adapters (no core crates) | all-adapter ring | adp · epic-fhir · gusto · netsuite · quickbooks · rippling · salesforce · slack · teams · workday | **DETACHMENT** — 10 independent vendor seams (richest a–c) |
| **consent-graph** *(doc-only, 0 crates)* | consent/privacy (planned) | — | — | spec/cedar/arch only |
| **api-gateway** *(doc-only, 0 crates)* | routing/rate-limit/auth/abuse (planned) | — | — | IP-002..008 ladder planned, none built |
| **identity** ★ | identity-core · identity-workload (2) | core: oya-identity/domain/usecase/api + oidc-issuer-kernel; workload: domain/api/app/rest | `workload-authz-cedar-adapter` · `workload-oidc-adapter` | **DETACHMENT** — human vs workload identity (T0) |
| **policy** | cedar-policy (1) | domain · api | — (Cedar IS the engine) | **COHESION** — Cedar authz primitive |
| **eventing** | eventing (1) | domain + adapter | `file-adapter` (1-adapter stage) | **COHESION** — early domain↔adapter split |
| **detection** *(spec-only, 0 crates)* | streaming/batch/feature-store/rules/scorer/graph (planned) | — | — | fraud/abuse substrate, design stage |
| **feature-flags** | flags (1) | umbrella crate | intra-crate; SDK fan-out is the port | **COHESION** (roadmap implies later detachment) |
| **governance** *(spec-only, 0 crates)* | governance (mid-migration) | — | — | IP-002/003 migrating check-crates IN |
| **intelligence** ★★ | ~25+ BCs (assist-draft, eval, attribution, context-aware-retrieval, subagent-runtime, rag-endpoint, model-routing, guardrails, …) | kernel×36 · domain×27 · adapter×25 · app×12 · usecase×10 · api×8 · worker×5 · rest×1 (128 crates) | LLM-provider mesh (anthropic/openai/gemini × api+subscription) · account adapters (claude/codex/gemini/inmemory) · 4 transport drivers (rest/graphql/sse/websocket) · file/infra adapters | **DETACHMENT, extreme** — the textbook hyperscaler ports/adapters control plane |
| **search** | crawler · index-inverted · index-vector · parser · query · rag · rank · serp (8) | domain ×8 | — (inverted vs vector are sibling BCs, not adapters — seam to watch) | **DETACHMENT** — 8 pipeline-stage BCs |
| **tenant-rbac** ★ | tenancy (1 BC; vast contract/evidence lattice) | domain→application→api + runtime + auth-runtime + listener + harness (39 crates) | `storage-adapter-inmemory` ↔ `postgres-rls-storage`; `workflow-adapter-inmemory` | cohesion-of-purpose + detachment-of-verification; cloud-readiness/compliance anchor |
| **workflow-engine** ★★ | event-bus · execution-engine · state-machine · trigger-orchestrator (4) + SaaS shell | full kernel→domain→usecase→app→api→rest→sdk→worker per BC (43 crates) | **event-bus port: kafka/nats/pulsar/redpanda/valkey/postgres (6 swappable)**; execution-engine + state-machine: postgres | **DETACHMENT (canonical exemplar)** — 4 independently-scalable BCs |
| **workflow-studio** | dsl-emitter · dsl-loader · policy-preview · visual-canvas (4) | domain ×3 + kernel | — | **DETACHMENT** — low-code authoring stages over workflow-engine |

### workspace (collaboration suite)

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **application** | chat · drive · forms · meet (4 surfaces) | domain + app + 4 `-api` | — (api = delivery ports) | **DETACHMENT** — suite aggregation seam |
| **app-shell-frontend** *(TS/pnpm, non-Rust)* | UI shell | — | — | unified app shell / launcher UI |
| **calendar** | calendar (1) | domain | — | **COHESION** — skeleton |
| **comms-email** *(doc-only, 0 crates)* | email/comms (planned) | — | — | Draft PRD |
| **connect** | address-book (1) | domain | — | **COHESION** — contacts directory skeleton |
| **docs** | docs (1) | domain | — (postgres+s3, valkey-CRDT PLANNED) | **COHESION** — Notion/Confluence-class |
| **drive** | file-store (1) | domain | — | **COHESION** — Drive/Dropbox-class |
| **forms** | forms (1) | domain | — | **COHESION** — T0 forms/data-capture |
| **mail** | mailbox-store (1) | domain · usecase · api · app · rest · grpc | `mailbox-store-adapter-postgres` | **COHESION** in crate-per-ring shape |
| **meet** | meet (1) | domain | — | **COHESION** — Zoom/Meet-class (T0) |
| **messenger** | message-stream (1) | domain · app · usecase · api · rest · grpc | `message-stream-adapter-postgres` | **COHESION** in crate-per-ring shape |
| **notes** | notes (1) | domain | — | **COHESION** — Keep/Notes skeleton |
| **recordings** | recordings (1) | domain | — | **COHESION** — meeting recordings store |
| **sheets** | sheets (1) | domain | — | **COHESION** — Sheets-class |
| **sites** | sites (1) | domain | — | **COHESION** — Sites-class |
| **slides** | slides (1) | domain | — | **COHESION** — Slides-class |
| **tasks** | tasks (1) | domain | — | **COHESION** — Asana/Tasks-class |
| **translate** | translate (1) | domain | — | **COHESION** — Translate-class |
| **whiteboard** | canvas-collaboration (1) | app | — | **COHESION** — Miro/FigJam-class |
| **workplace-integration** | workplace-integration (1) | scaffold | — | **COHESION** — seeded connector hub |
| **community** | post-store · social (2) | full domain→usecase→app→api→rest→grpc ×2 | `post-store-adapter-postgres`, `social-post-composition-adapter-postgres` | **DETACHMENT** — dual-protocol per BC |
| **design-collaboration** | creative-artifact (1) | modular-monolith (`adapter/domain/usecase`) | intra-crate `adapter/` (valkey realtime) | **COHESION** — Figma-class |
| **social** *(design-only, 0 crates)* | profile/follow/feed/reactions/… (planned) | — | — | Twitter/Threads-class, IP-001..018 specced |

### vertical-industry (fintech · ERP · CRM · CX · CLM · health · real-estate)

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **accounting** | journal (1) | domain · app · api · runtime | `storage-adapter-inmemory` | **COHESION** — general-ledger |
| **payments** ★ | charge · dispute · kyc-kyb · payout · refund · settlement · subscription (7) | charge full stack; others domain(+usecase/worker) | `adapter-stripe` + `adapter-adyen` (PSP) | **DETACHMENT** — 7 BCs, dual PSP |
| **payroll** | payroll-run (1) | domain · app · api · runtime | `storage-adapter-inmemory` | **COHESION** — Gusto/Workday-payroll |
| **treasury** | treasury-cash (1) | domain · app | — | **COHESION** — cash/treasury mgmt |
| **crm** | customer-engagement · revenue · procurement-source-to-pay (3) | domain/app across BCs | — | **DETACHMENT** — CRM + co-located SRM seam (flagged) |
| **contact-center** | voice-routing (1) | app | — | **COHESION** — CX scaffold |
| **contract-lifecycle-management** | contract-obligation (1) | app | — | **COHESION** — CLM scaffold |
| **real-estate** | portfolio · lease (1 vertical) | domain + app | — | **COHESION** — lease-portfolio mgmt |
| **plant-maintenance** | plant-maintenance + work-order (1) | domain + app | — | **COHESION** — SAP-PM/EAM |
| **production-planning** | production-planning + MRP (1) | domain + app | — | **COHESION** — SAP-PP/MRP |
| **quality-management** | QM + inspection (1) | domain + app | — | **COHESION** — SAP-QM |
| **supply-chain-planning** | SCP + network (1) | domain + app | — | **COHESION** — SAP-IBP/Kinaxis |
| **warehouse** | inventory + fulfillment (1) | domain + app | — | **COHESION** — SAP-EWM/WMS |
| **performance-management** | review-calibration (1) | `-service` | — | **COHESION** — Workday/Lattice |
| **hr** | employment (1) | domain · api · app · runtime | `employment-storage-adapter-inmemory` | **COHESION** in crate-per-ring shape |
| **global-trade** | trade-compliance (1) | domain | — | **COHESION** — trade-compliance |
| **learning-management** | course-progress (1) | modular-monolith | intra-crate `adapter/` | **COHESION** — Canvas/Coursera-class |
| **financial-planning** | forecast-scenario (1) | modular-monolith; `adapter/{http,grpc,asyncapi}.rs` | intra-crate transport adapters | **COHESION** — Anaplan/EPM-class |
| **marketing-automation** | campaign-journey (1) | modular-monolith; `adapter/{http,grpc,asyncapi}.rs` | intra-crate transport adapters | **COHESION** — Marketo/HubSpot-class |
| **patient-monitoring** *(design-only, 0 crates)* | ICU/remote-monitoring (planned) | — | — | Epic/Philips-class HealthTech |
| **pharmacy** *(design-only, 0 crates)* | e-prescribing (planned) | — | — | Epic-Willow-class HealthTech |
| **healthcare-integration** *(spec-only, 0 crates)* | HL7/FHIR interop (planned) | — | — | healthcare-interop |
| **imaging** *(spec-only, 0 crates)* | imaging/PACS (planned) | — | — | medical/doc imaging |

### intelligence / data / knowledge

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **analytics** | analytics (1) | domain · usecase · app · api (+tenant-bootstrap) | — | **COHESION** — OLAP analytics |
| **data-pipeline** | lineage-replay (1) | modular-monolith | intra-crate `adapter/` | **COHESION** — Fivetran/dbt-class ELT/lineage |
| **data-warehouse** | tenant-olap (1) | modular-monolith + `lake_engine/` | intra-crate `adapter/`+`lake_engine/` | **COHESION** — Snowflake/BigQuery-class |
| **ontology** | ontology-core · query-engine (2) | kernel→domain→api + query-engine domain/usecase | — | **DETACHMENT** — Palantir-Foundry-style |

### ci-cd-tooling

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **ci-controller** | CI-control (1) | kernel + app + 2 adapters | `github-adapter` · `k8s-adapter` | **COHESION** — dogfooded control plane |
| **ci-tide** | merge-queue (1) | kernel + app + 1 adapter | `github-adapter` | **COHESION** |
| **ci-webhook-gateway** | webhook-ingress (1) | kernel + app + 4 adapters | `github` · `jenkins` · `ed25519` · `authz-cedar` | **COHESION** with rich adapter ring |

### ops / observability / security / integration

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **ops** | docs-portal · workspace-shell (2) | full kernel/usecase/rest/adapter ×2 | `docs-portal-adapter` · `workspace-shell-adapter` | **DETACHMENT** — operator/admin shell + docs |
| **observability** | cloud-observability + observability (2 slices) | kernel/domain/api + domain | `observability-tracing-adapter` | mild **DETACHMENT** — Datadog/Grafana-class |
| **incident-management** | sre-incident-command (1) | modular-monolith | intra-crate `adapter/` | **COHESION** — PagerDuty-class (overlaps itsm) |
| **itsm** | escalation-policy · incident-room · on-call-schedule · postmortem · status-update · service-management (6) | aggregator = monolith; others lib-per-BC | intra-crate `adapter/` | **DETACHMENT** — 6 ITSM BCs |
| **oya-authn-device-firmware** | device-authn-firmware (1) | single firmware crate (`attestation/ctap2/transport/storage`) | internal `transport/`,`storage/` modules | **COHESION** — FIDO2/CTAP2 (YubiKey-class) |
| **developer-sdk** | dev-cli (1) | sdk/CLI | it IS the SDK port | **COHESION** — external dev surface |

### marketplace / app-store

| Service | BCs | Layers | Ports/adapters | Cohesion/Detachment |
|---|---|---|---|---|
| **marketplace** | doc-set-scaffold (pre-BC) | scaffold | — | early scaffold |
| **plugin-app-store** *(docs-only, 0 crates)* | publish/install/quarantine (planned) | — | — | AWS-Marketplace/AppExchange-class |
| **ops-dashboard-control-center** *(docs-only, 0 crates)* | operator console (planned) | — | — | AWS-Console-class (runnable = `ops`) |
| **finops-portal** *(spec-only, 0 crates)* | billing-presentation (planned) | — | — | chargeback/showback portal |
| **diagnostics** *(spec-only, 0 crates)* | device/host diagnostics (planned) | — | — | pre-implementation |

### catalog/SLO stubs (oya-billing · oya-cost · oya-flags · oya-identity · oya-meter)

5 dirs with only `BUCK` + `catalog.yaml` + `README.md` + `slos/availability.openslo.yaml`
— **no crates, no src**. Newer (May-31) consolidated successors to the larger
`billing`/`feature-flags`/`identity` planes. Listed, not dropped.

---

## (c) ARCHITECTURE — cohesion, detachment, and mobility

### Where COHESION matters (shared kernels / contracts pull inward)

- **Single-BC services keep one crate-cluster** so the domain stays the single source of
  truth: the entire **ERP family** (plant-maintenance, production-planning,
  quality-management, supply-chain-planning, warehouse, treasury, real-estate) is a
  domain+app pair — cohesion until scale forces a split.
- **Permanent shared contracts** are the cohesion anchors that everything binds to:
  **Cedar** is wired as the authz contract across 5+ adapters (`managed-k8s-tenant-quota`,
  `cloud-intelligence`, `identity-workload`, `ci-webhook-gateway`, `oya/policy`); the
  **transactional-outbox** (`oya-shared-transactional-outbox-*`) is the one real eventing
  path today. These are deliberately cohesive: one engine/contract, many consumers.
- **`tenant-rbac`** is the instructive hybrid: **cohesion of purpose** (one tenancy BC) but
  **detachment of verification** (a wall of `*-contract` / `*-evidence` crates). The BC
  stays cohesive; the proof surfaces fan out.

### Where DETACHMENT enables scale (multi-BC, independently scalable)

- **CLOUD:** `tenancy` (10+ feature BCs) · `cloud-compute` (vm/k8s/functions) ·
  `cloud-cell` (cell/region/regional-pack) · `cloud-storage` (object/block) ·
  `cloud-network` (vpc/lb/residency) · `cloud-billing` (billing/metering) ·
  `cloud-marketplace` (cloud/plugin) · the **managed-k8s quartet** (one product split
  across 4 deployable service dirs).
- **PRODUCT:** `workflow-engine` (4 BCs) · `intelligence` (~25+ BCs, 128 crates) ·
  `payments` (7 BCs) · `search` (8 pipeline BCs) · `audit-chain` (5) · `compliance` (5) ·
  `itsm` (6) · `identity` (2) · `community` (2) · `application` (4 surfaces) ·
  `ontology` (2) · `ops` (2) · `workflow-studio` (4) · `connector` (10 adapters).
- The detachment payoff: each BC ships and scales on its own timeline (e.g. `payments`
  settlement runs a dedicated `worker`; metering scales apart from invoicing; managed-k8s
  control-plane-host vs tenant-quota are separately ownable).

### How PORTS + ADAPTERS give architecture mobility (concrete, cited)

The mobility seam is always a **`-adapter-<x>` crate in front of an owned `-kernel`/`-domain`,
exposed through an `-api`/`-rest` port** — swap the adapter, keep the domain:

- **Event-bus, 6 adapters** — `workflow-engine` event-bus port has swappable
  `kafka/nats/pulsar/redpanda/valkey/postgres` impls behind one `-event-bus-api`. The
  marquee multi-backend seam (today the impls are trait-stubs; the *shape* is the proof).
- **cloud-iam: `oci ↔ selfhosted`** — managed-cloud vs on-prem IAM behind one domain.
- **cloud-intelligence: 5 ports** — `authz-cedar` · `codex` · `openbao` · and a swappable
  **eventsink `clickhouse ↔ valkey`** (the cloud-side event-bus-style seam).
- **PSP swap** — `payments` `adapter-stripe ↔ adapter-adyen`.
- **Provider/persistence swaps** — `cloud-compute` aws↔oci · `cloud-storage` s3↔oci ·
  `cloud-kms` oci↔openbao · `cloud-network` oci↔selfhosted · `tenant-rbac`
  inmemory↔postgres-rls · managed-k8s `capi↔inmemory` / `cedar↔inmemory`.
- **LLM-provider mesh** — `intelligence` swaps anthropic/openai/gemini behind a common
  `adapter-domain`, each in API-key *and* subscription auth modes.

This is the **infra-sovereignty ratchet** made structural: a vendored bridge today
(`adapter-oci`, `adapter-stripe`, `adapter-kafka`) is swapped for an owned engine tomorrow
without touching the kernel — see `00-TECH-STACK.md` for the NOW→IDEAL trajectory per seam.

### Maturity spread (a signal, not a verdict)

Full clean-arch stacks (workflow-engine, payments, intelligence, audit-chain, community,
tenancy, ops, tenant-rbac) → crate-per-ring single-BC (hr, mail, messenger) → modular
monoliths (data-pipeline, financial-planning, marketing-automation, learning-management) →
domain+app ERP pairs → domain-only seeds (notes/sheets/sites/slides/tasks/translate +
the 8 search domains) → design-only specs → catalog stubs. Only `cloud-iac` and the
crate-per-ring services ship a present `-rest`/`-grpc` surface in `cloud/`; most cloud
verticals are inner-only (`cloud-data`, `cloud-capacity` = kernel+domain).

---

## (d) Pure-split conformance note

**Rule:** `oya/` = product plane; `cloud/` = platform (hyperscaler substrate) plane.

**Conformant:** the 25 `cloud/` dirs are all platform substrate (IAM/KMS/compute/storage/
network/tenancy/intelligence/k8s/cell). The crate package prefix is `oya-` everywhere
(org/workspace prefix), so **placement under `source/cloud/` — not the prefix — is what
marks platform**; this is consistent across all 25.

**Sprawl / boundary smells to flag (cited):**

1. **Platform-class services living in `oya/`.** `audit-chain`, `compliance`, `connector`,
   `consent-graph`, `api-gateway` are `cloud/`-style shared-substrate services physically
   under `oya/` (the lane files tag them "cloud/-style platform svc (in oya/)"). They are
   substrate, not end-user product — candidates to migrate to `cloud/` or a dedicated
   `substrate/` tree.
2. **Substrate tier inside the product tree.** `detection`, `feature-flags`, `intelligence`,
   `marketing-automation` carry `tier: substrate` yet live in `oya/`.
3. **AI engine home is in flux.** The 128-crate AI engine is in `oya/intelligence` while
   `cloud/cloud-intelligence` is the thin egress broker — the roadmap re-homes the engine
   *down* to `cloud/` (see `00-TECH-STACK.md` AI row). Until then, the heaviest substrate
   sits on the product side.
4. **Duplicate/consolidation spans.** `oya-billing`/`oya-cost`/`oya-flags`/`oya-identity`/
   `oya-meter` (catalog stubs) appear to be consolidated successors to `billing`/
   `feature-flags`/`identity`; and `cloud-billing` (platform) vs `oya-billing` (product
   stub) overlap. `incident-management` (1 app) and `itsm` (6 BCs) cover the same
   incident/on-call space with opposite decomposition — a future merge/seam decision.
5. **`cloud-k8s` placeholder dir** has no crates (code in `cloud-compute/`); kept "for
   compatibility" — a boundary artifact to resolve.

Net: the split is **broadly honored** (cloud=platform, oya=product), with one consistent
sprawl pattern — **substrate-class services parked in `oya/`** — and a handful of
in-flight consolidation/relocation seams (AI engine re-home, billing/identity stubs,
cloud-k8s placeholder).
