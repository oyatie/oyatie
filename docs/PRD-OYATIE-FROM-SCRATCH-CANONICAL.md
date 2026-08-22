---
doc_class: ProductRequirements
product: oyatie
status: Draft
date: 2026-05-22
owner: founder + product-council + architecture-council
canonicality: from-scratch-self-contained
doc_status: draft
---

# Oyatie From-Scratch Canonical PRD

## 1. Purpose

This PRD is the standalone product and implementation handoff for building
Oyatie from scratch. It is complete on its own: an implementation team should
not need to read any existing repository document, decision record, roadmap, or
specification to understand the intended product, architecture constraints,
technology stack, delivery sequence, and acceptance gates.

Oyatie is an integrated ecosystem-as-a-service for regulated organizations. It
combines business applications, workspace collaboration, workflow automation,
ontology-backed data modeling, agent execution, cloud infrastructure, search,
analytics, and operational control under one tenancy, identity, policy, audit,
and regional-pack model.

The product bet is cohesion. Customers can buy separate tools, cloud services,
workflow engines, search products, and AI assistants elsewhere. Oyatie wins by
making those surfaces share the same tenant boundary, evidence trail, identity
model, policy engine, cloud cells, workflow substrate, and agent runtime.

## 2. Product North Star

Build one system where a regulated organization can run its work, automate its
processes, govern its data, operate its infrastructure, and deploy agents
without stitching together many vendor contracts and inconsistent data
boundaries.

The first production release must prove four things:

1. A tenant can be onboarded, isolated, governed, billed, and audited.
2. A user can sign in once and use the application entry surface, workspace tools,
   workflow studio, ontology data, and agent assistance without crossing product
   seams.
3. Operators can deploy, observe, recover, and audit the system from one control
   surface.
4. Every external API, event, policy, data model, and deployment primitive is
   versioned and source-checkable.

## 3. Product Principles

- One tenant boundary everywhere. Tenant identity is not a convention; it is a
  mandatory input to authorization, data partitioning, audit, metering, search,
  workflow, and agent execution.
- One policy surface. Application authorization, feature activation, agent
  autonomy ceilings, and sensitive operation approvals use Cedar-backed policy
  decisions. Kubernetes admission policy is separate and handled by Kyverno.
- One audit chain. Every mutation, capability invocation, policy decision,
  evidence emission, data egress approval, and privileged operation produces a
  tamper-evident audit event.
- One ontology and workflow substrate. Cross-product state is modeled in the
  ontology. Cross-product action is routed through workflow. Direct hidden
  service-to-service coupling is forbidden.
- One regional-pack model. Geography-specific law, tax, identity, payment,
  localization, and regulatory evidence are overlays. The core architecture must
  stay locale-neutral.
- Agents are bounded executors, not unchecked operators. Every agent run starts
  from a registered capability, runs under a declared autonomy ceiling, and
  emits evidence.
- The cloud is both internal substrate and external product. Oyatie must be able
  to run itself on the same primitives it sells: cells, compute, storage,
  network, IAM, billing, observability, KMS, and deployment operations.
- Source contracts beat prose. APIs, events, policies, schemas, tests, and gates
  are the implementation truth. Prose exists to explain why and how.

## 4. Target Users

| Persona | Primary need | Product value |
|---|---|---|
| Tenant executive | Run regulated operations without many disconnected vendors | One accountable ecosystem with integrated compliance evidence |
| Tenant admin | Configure users, products, policy, billing, residency, and audit | Unified admin and control plane |
| Employee or member | Complete daily work across communication, documents, workflows, approvals, and search | One sign-in, one workspace, fewer context switches |
| Tenant builder | Model business objects, build workflows, install integrations, and automate repetitive work | Workflow Studio, ontology, plugins, connectors, and agent assistance |
| Developer or ISV | Build integrations and publish capabilities safely | Stable APIs, SDKs, webhooks, marketplace path, sandboxed plugins |
| Operator or SRE | Deploy, observe, recover, and govern production cells | Ops Control Center, GitOps, evidence, SLOs, runbooks, DR drills |
| Security or compliance officer | Prove access, consent, retention, residency, and incident handling | Audit exports, policy evidence, data lineage, DSR and retention reports |
| Regional compliance owner | Adapt Oyatie to a jurisdiction without forking the product | Regional packs for law, tax, identity, payment, language, and evidence |
| Agent operator | Run multi-provider agents under human and policy control | Capability registry, autonomy ceilings, provider routing, evidence chain |

## 5. Product Scope

### 5.1 In Scope

| Surface | First production scope |
|---|---|
| Application entry surface | Tenant sign-in, tenant admin, product enablement, app switcher, billing entry points, audit viewer |
| Tenancy and identity | Tenant lifecycle, org hierarchy, workspaces, OIDC/SAML, WebAuthn/MFA, sessions, service identity |
| Policy and data boundary | Cedar authorization, feature activation, autonomy ceilings, consent tiers, egress controls, DSR cascade |
| Audit chain | Append-only event evidence, sealing, verification, retention, export, recovery replay |
| Workspace and | Mail, chat, calendar, drive, docs, sheets, slides, forms, meet, tasks, notes, translate, recordings |
| Workflow | Workflow engine, workflow studio, state machines, DAGs, approvals, retries, idempotency, human tasks |
| Ontology | Object types, link types, action types, functions, entity history, provenance, query APIs |
| Foundry | Agent runtime, capability registry, provider adapters, run/step evidence, RAG retrieval, engineering automation |
| Cloud | Region, availability zone, cell, compute, Kubernetes, functions, object/block/file storage, VPC, load balancer, DNS, IAM, KMS, billing, FinOps, observability |
| Search | Tenant-private search, public search path, inverted index, vector index, morphology hooks, RAG endpoint |
| Ads and analytics | Privacy-gated sponsored slots, attribution, aggregate analytics, no regulated sensitive-data targeting |
| Ops Control Center | Live documentation, dashboards, deployments, incidents, backups, restore drills, capacity, cost, carbon, evidence packs |
| Developer surface | OpenAPI, AsyncAPI, proto3, SDK generation, webhooks, signing, sandboxed plugins, marketplace workflow |
| Regional packs | KR first-class pack, plus a repeatable model for JP, US, EU, IN, BR, KSA, UAE, ANZ, and SEA |
| Deployment modes | Shared cloud, dedicated cell, on-prem, air-gapped, hybrid, and future owned/colocated capacity |

### 5.2 Out Of Scope

- Frontier foundation-model research as a standalone AI lab.
- Custom chip design.
- Selling raw tenant data.
- Targeting ads with PHI, PCI, regulated credit data, sensitive personal data,
  or tenant-private content.
- Recreating Redis as a dependency choice; Valkey is the cache/KV choice.
- Standalone feature-flag SaaS; feature activation is policy-backed.
- Direct microservice-to-microservice product coupling outside declared workflow,
  ontology, or declared shared contracts.
- Unbounded autonomous agents or agents operating outside the capability
  registry.
- Shared tenant workloads without cell-isolation evidence.
- Unversioned public APIs, events, proto services, SDKs, policies, or database
  migrations.

## 6. First Production Release Requirements

### 6.1 Tenant Onboarding

| ID | Requirement | Acceptance |
|---|---|---|
| TEN-01 | Create a tenant with legal name, region, residency class, billing account, admin user, enabled products, and regional pack | Tenant can sign in and access enabled products within 5 minutes |
| TEN-02 | Assign every tenant to a cell and shard key at creation time | Every request resolves to a cell and tenant partition before business logic |
| TEN-03 | Enforce tenant isolation in application code, database policy, search indexes, cache keys, event topics, object storage prefixes, and metrics labels | Cross-tenant read/write fuzz tests fail closed |
| TEN-04 | Support suspension, deletion request, retention hold, and DSR workflow | Evidence export shows every lifecycle step |

### 6.2 Identity And Access

| ID | Requirement | Acceptance |
|---|---|---|
| IAM-01 | Support OIDC, SAML, passwordless/WebAuthn, MFA, service identity, and short-lived tokens | Auth flows are covered by integration tests and audit events |
| IAM-02 | Every user and service request carries tenant, actor, workspace, capability, plane, and data-class context | Middleware rejects missing context |
| IAM-03 | Cedar policy decisions gate application permissions, feature activation, and agent autonomy | Policy decision latency meets hot-path target and emits evidence |
| IAM-04 | Privileged actions require step-up auth and break-glass evidence | Unauthorized and stale-step-up requests fail closed |

### 6.3 Application And Workspace

| ID | Requirement | Acceptance |
|---|---|---|
| APP-01 | Provide a tenant admin console for users, roles, products, billing, audit, and regional settings | Admin can enable and disable products with audit proof |
| APP-02 | Provide an application switcher and workspace navigation surface | User can move between enabled products without new sign-in |
| APP-03 | Provide workspace communication and productivity tools for first release tenants | Mail, chat, calendar, drive, docs, forms, meet, tasks, notes, and recordings have tenant-isolated APIs |
| APP-04 | Provide global search across enabled tenant content according to consent and data class | Search never indexes denied classes and emits indexing evidence |

### 6.4 Workflow And Ontology

| ID | Requirement | Acceptance |
|---|---|---|
| WFO-01 | Model business entities as ontology object types, link types, action types, and functions | Entity changes are versioned and queryable by tenant |
| WFO-02 | Route cross-product work through workflow events, state machines, DAGs, and human approvals | Workflow replay reproduces state transitions |
| WFO-03 | Support idempotency, retries, compensation, dead-letter queues, and operator recovery | Duplicate commands do not double-apply mutations |
| WFO-04 | Expose workflow and ontology contracts through stable HTTP, event, and proto surfaces | Contract compatibility gates pass before release |

### 6.5 Foundry Agents

| ID | Requirement | Acceptance |
|---|---|---|
| FND-01 | Register every agent capability with schema, autonomy ceiling, provider allowance, data class, cost profile, and evidence requirements | Unregistered capabilities cannot run |
| FND-02 | Support provider adapters for OpenAI/Codex-class, Anthropic/Claude-class, Google/Gemini-class, and regional providers | Provider selection is policy-gated and auditable |
| FND-03 | Store run, step, tool-call, output, cost, and review evidence | Every run can be reconstructed from evidence |
| FND-04 | Support RAG retrieval over tenant-approved search data only | Retrieval returns citations and denies disallowed data classes |

### 6.6 Cloud And Operations

| ID | Requirement | Acceptance |
|---|---|---|
| CLD-01 | Provide cell-based compute, storage, network, IAM, KMS, billing, observability, and deployment primitives | Internal Oyatie workloads run on the same cell model |
| CLD-02 | Deploy server-side workloads on Kubernetes unless an explicit edge exception is approved | All workloads declare cell, tenant, plane, data class, and SLO |
| CLD-03 | Use OpenTofu for infrastructure as code and Argo CD for GitOps deployment | Desired state, drift, and deployment evidence are auditable |
| CLD-04 | Provide backup, restore, DR pairing, restore drills, capacity planning, cost, and carbon reporting | Restore drills produce evidence and meet RTO/RPO targets |

## 7. Technical Architecture Requirements

### 7.1 Layering

Every bounded context uses the same layered structure:

| Layer | Responsibility | Forbidden |
|---|---|---|
| Kernel | Value objects, invariants, pure traits, sealed domain primitives | Network, filesystem, database, clocks, randomness, framework types |
| Domain | Use cases, aggregates, policy-free business rules | HTTP handlers, SQL clients, queue clients, provider SDKs |
| Application | Command orchestration, transactions, sagas, retries, idempotency | Vendor-specific business logic |
| Adapter | Database, queue, cache, cloud, provider, filesystem, and external API clients | Owning domain invariants |
| API | HTTP/gRPC handlers, request/response mapping, auth middleware binding | Business rules not expressed in domain/application |
| Worker | Event consumers, scheduled jobs, background processors | Hidden synchronous product coupling |
| Runtime | Composition root, dependency wiring, process lifecycle | Domain logic |

### 7.2 Crate And Service Naming

Use flat service naming:

`oya-<service>[-<bounded-context>]-<layer>`

Examples:

- `tenancy-kernel`
- `identity-api`
- `workflow-engine-domain`
- `ontology-function-api`
- `intelligence-capability-kernel`
- `cloud-compute-adapter-oci`
- `search-index-vector-domain`
- `ops-docs-portal-rest`

Do not add artificial grouping slots such as shared, vertical, module, or
infrastructure-tier names inside package identity. A package name should reveal
the owned service, optional bounded context, and layer.

### 7.3 Boundary Code Shape

Kernel crates are pure. They define types and ports; adapters implement ports.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId([u8; 16]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataClass {
    Public,
    Internal,
    Confidential,
    Regulated,
}

pub trait TenantContextPort {
    fn tenant_id(&self) -> &TenantId;
    fn data_class(&self) -> &DataClass;
    fn capability(&self) -> &str;
}
```

Rules:

- API layers map transport into commands and queries.
- Domain/application layers own behavior.
- Adapter layers own external systems.
- Policy checks happen before mutation and before data egress.
- Audit events are emitted for every mutation and policy decision.

## 8. Canonical Technology Stack

### 8.1 Language And Backend Runtime

| Area | Decision |
|---|---|
| Primary language | Rust |
| Rust edition | 2024 |
| Minimum supported Rust version | 1.97.1 unless implementation source verification requires a later version |
| Workspace model | Cargo workspace, resolver 2, shared dependency and lint baselines |
| Async runtime | Tokio 1 |
| HTTP backbone | Hyper 1 plus hyper-util 0.1 |
| Serialization | Serde 1, serde_json 1, serde_yaml 0.9 where YAML is required |
| CLI | clap 4.5 |
| Tracing | tracing 0.1.44 and tracing-subscriber 0.3.23 |
| Error handling | anyhow for boundary tooling; typed errors for product domain code |

### 8.2 Web And Client

| Area | Decision |
|---|---|
| New web surfaces | Leptos in Rust |
| Rendering | SSR-first with selective WASM hydration |
| Admin and ops UX | Dense, utilitarian, keyboard-friendly, audit-heavy |
| Mobile | Native clients only when parity, accessibility, and store-policy gates are satisfied |
| Design posture | No marketing page as the first screen for tools; users land in the actual working surface |

### 8.3 API And Contract Standards

| Contract type | Decision |
|---|---|
| REST and public HTTP | OpenAPI 3.2.0 |
| Events and channels | AsyncAPI 3.1.0 |
| Internal and external gRPC | proto3 |
| Public API versioning | Date-based external version carrier plus SDK semver |
| SDKs | Generated from contracts; manually written SDK code must match generated contract tests |
| Webhooks | Signed, versioned, idempotent, replay-protected |
| Deprecation | Support at least three public versions and at least 180 days after deprecation notice unless a security emergency requires shorter handling |

### 8.4 Data And Search

| Area | Decision |
|---|---|
| Primary OLTP | PostgreSQL |
| Horizontal OLTP scale | Citus per cell |
| Cache, KV, pub/sub-like cache duties | Valkey |
| Event backbone | Kafka in KRaft mode plus outbox pattern |
| Search | OpenSearch plus tenant-private index lifecycle |
| Vector search | Milvus for vector retrieval where a dedicated vector database is justified |
| Object storage | S3-compatible object storage; SeaweedFS-compatible implementation path is acceptable when evidence supports it |
| OLAP table format | Apache Iceberg as canonical write path |
| OLAP query | ClickHouse may query/compute over Iceberg, but must not become the canonical write path |
| IDs | Stable typed IDs; use native UUID where required and preserve tenant partition keys |

### 8.5 Policy, Security, And Identity

| Area | Decision |
|---|---|
| Application authorization | Cedar policy language and engine |
| Cedar compatibility target | Start from the pinned product semantics level, then verify against current Cedar reference before implementation |
| Kubernetes admission policy | Kyverno |
| Secrets and KMS | OpenBao plus HSM/BYOK/HYOK integration where required |
| Service identity | SPIFFE/SPIRE |
| Service proxy | Envoy where L7 proxying is required |
| Network policy and CNI | Cilium |
| Plugin sandbox | Wasmtime for WASM capabilities |
| Tenant-customer untrusted code | Kata Containers plus Cloud Hypervisor-class VM isolation |
| Container images | OCI images, signed and provenance-attested |
| Supply chain | SLSA posture, SBOMs, cosign/Sigstore signing, license policy gate |

### 8.6 Infrastructure And Delivery

| Area | Decision |
|---|---|
| Runtime orchestration | Kubernetes for server-side workloads |
| Infrastructure as code | OpenTofu only |
| GitOps CD | Argo CD |
| CI | GitHub Actions for hosted workflows; Jenkins LTS for self-hosted, air-gapped, on-prem, and colo parity |
| Image registry | OCI registry with signed image enforcement |
| Deployment modes | Shared cloud, dedicated cell, hybrid, on-prem, air-gapped, and future owned/colocated capacity |
| Agent-safe repository workflow | Isolated plain-git branch, PR against `dev`, Jenkins required checks, `oya gate` / `oya verify`, reviewer/governance approval |

### 8.7 Retired Or Rejected Choices

| Choice | Replacement |
|---|---|
| Redis | Valkey |
| Standalone feature flag service | Cedar-backed policy and feature activation |
| Product coupling through hidden direct calls | Workflow, ontology, and declared shared contracts |
| Unbounded agent tools | Capability registry and autonomy ceiling |
| Hand-maintained API drift | Source contracts plus generated SDKs and compatibility tests |
| Infrastructure drift by manual console edits | OpenTofu plan/apply and Argo CD reconciliation |
| Unsigned images | Signed OCI images with verification |

## 9. Product Surfaces

### 9.1 Application Entry Surface

The Application entry surface is the B2B entry point. It owns sign-in, app
switching, tenant administration, product enablement, billing entry points,
notifications, preferences, audit viewing, and navigation.

Acceptance requirements:

- First paint and entry-frame p99 <= 500 ms under normal regional conditions.
- Tenant admin can enable a product and see the resulting workflow and audit
  evidence.
- Product-specific business logic is not embedded in the entry surface.
- A user sees only products enabled for the tenant and allowed by policy.

### 9.2 Workspace And Connect

Workspace includes professional communication and productivity tools: mail,
calendar, drive, messenger, meet, docs, sheets, slides, forms, sites, tasks,
notes, translate, recordings, retention, DLP, e-discovery, and address book.

Acceptance requirements:

- All workspace content carries tenant, workspace, owner, data class, retention
  class, residency, and search-indexing eligibility.
- Real-time collaboration uses explicit transport contracts and presence
  evidence.
- Mail and messaging support retention, legal hold, export, DLP, and audit.
- Recordings and transcripts are governed by consent and regional pack rules.

### 9.3 Workflow

Workflow is both substrate and hero product. It executes cross-product actions
through typed state machines, DAGs, human approval tasks, retries,
compensation, timers, idempotency, and event routing.

Acceptance requirements:

- Every workflow execution has replayable history.
- Every external side effect has an idempotency key.
- Human approval tasks have policy, role, timeout, escalation, and audit.
- Failed workflows can be inspected and resumed without database surgery.

### 9.4 Ontology

Ontology is the canonical semantic layer for tenant business objects. It owns
object types, link types, action types, functions, provenance, and change
history.

Acceptance requirements:

- Object types are versioned.
- Link types preserve provenance.
- Action types route through workflow or domain commands.
- Functions are tenant-filtered and data-class-filtered.
- Search and RAG derive from approved ontology projections, never from raw
  tenant sprawl.

### 9.5 Foundry

Foundry is the agent runtime and engineering automation surface. It runs
capabilities, routes providers, applies autonomy ceilings, emits evidence,
supports RAG, and operates engineering lanes.

Acceptance requirements:

- No unregistered capability can execute.
- No provider route can bypass residency, data class, cost ceiling, or policy.
- Run and step evidence is complete enough for replay and audit.
- Agents can modify Oyatie only through policy-gated change workflows.

### 9.6 Cloud

Cloud is an internal substrate and external product: regions, availability
zones, cells, compute, Kubernetes, functions, object/block/file storage, VPC,
load balancing, DNS, IAM, KMS, billing, FinOps, observability, backup, and DR.

Acceptance requirements:

- Every workload declares cell, plane, tenant relation, data class, and SLO.
- Every privileged control-plane mutation emits audit evidence.
- Every cell has backup, restore, DR pairing, and capacity evidence.
- Customers can understand cost, quota, carbon, and residency posture.

### 9.7 Search And Analytics

Search includes tenant-private search, public search path, RAG retrieval,
indexing, ranking, and consent-aware extraction. Analytics includes product
analytics, operational analytics, tenant reporting, and privacy-preserving ads
measurement.

Acceptance requirements:

- Tenant-private data never enters a public index.
- Sensitive regulated classes are excluded from ad targeting.
- Indexing eligibility is explicit and auditable.
- Analytics uses aggregate and consented data with data-boundary evidence.

### 9.8 Ops Control Center

Ops Control Center is the operational cockpit. It covers live system docs,
deployments, incidents, SLOs, logs, traces, metrics, backups, restore drills,
capacity, cost, carbon, compliance evidence, and release gates.

Acceptance requirements:

- Operators can trace a production issue from alert to deployment, service,
  tenant, cell, logs, traces, metrics, recent changes, and rollback plan.
- Every page names stale or missing evidence instead of implying certainty.
- Restore drills and incident reports are visible and exportable.
- Operational docs are generated from live system state where possible.

## 10. Non-Functional Requirements

### 10.1 Availability And Recovery

| Surface | Availability target | RTO | RPO |
|---|---:|---:|---:|
| Tenant identity and tenancy | 99.99% | 10 seconds | 1 second |
| Application entry surface | 99.95% | 15 seconds | 5 seconds |
| Workflow execution | 99.95% | 60 seconds | 5 seconds |
| Audit emission | 99.99% | 10 seconds | 1 second |
| Cloud control plane | 99.95% preview, 99.99% stable | 60 seconds | 5 seconds |
| Object storage metadata | 99.99% | 60 seconds | 1 second |
| Ops Control Center | 99.9% | 5 minutes | 15 minutes |

### 10.2 Performance

| Metric | Target |
|---|---:|
| Hot-path tenant context validation | p99 <= 5 ms |
| Cedar authorization decision for hot-path request | p99 <= 10 ms |
| Application entry surface SSR response | p99 <= 500 ms |
| Workflow command acceptance | p99 <= 200 ms |
| Workflow event propagation | p99 <= 1 second |
| Audit event accepted | p99 <= 200 ms |
| Audit segment sealed | p99 <= 1 second |
| Search query over approved tenant index | p99 <= 250 ms for normal tenant corpus |
| RAG retrieval over approved tenant corpus | p99 <= 500 ms before model call |
| Cloud control-plane read | p99 <= 200 ms |
| Cloud control-plane mutation accepted | p99 <= 500 ms before async provisioning |

### 10.3 Security And Compliance

- Deny by default on network egress, data export, provider routing, and
  privileged mutation.
- Every sensitive action requires tenant, actor, capability, policy decision,
  data class, reason, and audit event.
- Secrets never appear in logs, traces, events, prompts, generated code, or
  evidence payloads.
- PII, PHI, PCI, regulated credit data, and sensitive personal data are separate
  data classes with stricter indexing, RAG, analytics, and ad rules.
- DSR requests cascade through databases, indexes, caches, object storage,
  derived data, model memories, workflow histories, and exports.
- Evidence must distinguish completed controls from planned controls.

### 10.4 Observability

- Every request has trace context from edge to database and event publication.
- Metrics use bounded cardinality labels.
- Logs are structured JSON with trace and span correlation.
- Tenant labels are allowed only where cardinality and privacy rules permit.
- SLO dashboards must show current burn, historical burn, and owner.
- Every critical alert links to evidence, runbook, owner, rollback path, and
  recent changes.

### 10.5 Scalability

- All tenant-bound state is partitionable by tenant or cell.
- All event topics are partitioned and replayable.
- All caches include tenant-aware keys and explicit TTL.
- All long-running jobs are resumable.
- All search and vector indexes are rebuildable from canonical source data.
- All OLAP tables use Iceberg-compatible schema evolution and partitioning.
- All public APIs support pagination and bulk-safe patterns.

## 11. Regional Pack Requirements

Regional packs adapt the canonical product to local law, tax, identity,
payments, language, and evidence without forking core product behavior.

The first pack is KR. The pack must cover:

- Language and locale.
- Identity and account conventions.
- Business registration and KYB/KYC hooks.
- Tax invoice and payroll-relevant rails.
- Residency and retention classes.
- Privacy and consent requirements.
- Regulator evidence exports.
- Payment and billing rails.
- Store policy where mobile is used.
- Search morphology and language processing.

Future packs use the same seams for JP, US, EU, IN, BR, KSA, UAE, ANZ, and SEA.

Acceptance requirements:

- A regional pack can add validation, evidence, language, tax, payment, and
  identity behavior without changing core tenant isolation.
- A tenant declares one or more regional packs at onboarding.
- Data residency and provider routing enforce regional pack constraints.
- Every regional exception is machine-readable, testable, and owned.

## 12. Delivery Sequence

### 12.1 Foundation

Build tenancy, identity, policy, audit, data boundary, cell model, service
identity, observability, contracts, and source verification first. No customer
surface can claim production readiness until its foundation dependencies are
real and tested.

Exit criteria:

- Tenant isolation tests pass across database, cache, events, object storage,
  search, metrics, and logs.
- Policy decisions are enforced and audited.
- Contract compatibility gates exist.
- Audit events can be emitted, sealed, queried, and replayed.
- Deployment evidence exists for at least one development cell.

### 12.2 Foundry And Workflow Preview

Build capability registry, agent run model, provider adapters, workflow engine,
workflow studio, ontology primitives, and evidence pipeline.

Exit criteria:

- A registered agent capability can execute a workflow step under autonomy
  ceiling and emit complete evidence.
- Workflow can call product APIs through declared contracts.
- Ontology can model and query tenant entities.
- RAG retrieval uses only approved tenant data.

### 12.3 Application And Workspace Preview

Build the tenant-facing application entry surface and the first workspace
surfaces.

Exit criteria:

- Tenant admin can onboard users, enable products, manage billing entry points,
  and view audit logs.
- Users can use mail, chat, calendar, drive, documents, forms, tasks, and notes
  baseline surfaces under tenant isolation.
- Search and workflow integrations are visible in normal work.

### 12.4 Cloud And Ops Preview

Build internal cloud cells, OpenTofu modules, Argo CD deployment, Kubernetes
runtime, KMS/secrets, observability, backup, restore, DR, capacity, FinOps, and
Ops Control Center.

Exit criteria:

- Oyatie workloads run in a cell with signed images and GitOps deployment.
- Operators can trace incidents from alert to rollback.
- Restore drill evidence exists.
- Infrastructure drift is detected and reconciled.

### 12.5 First Commercial Release

Ship a focused regulated organization release. Corporate operations is the
recommended first vertical because it exercises identity, admin, workspace,
workflow, billing, audit, regional pack, and employee lifecycle without starting
from the highest clinical or financial risk surface.

Exit criteria:

- At least three design-partner tenants can run real workflows.
- Product-critical SLOs are measured, not guessed.
- Tenant evidence exports are usable by security/compliance buyers.
- Support, incident, backup, restore, and billing workflows are operational.

## 13. Acceptance Gates

| Gate | Must pass |
|---|---|
| Source verification | Every framework, protocol, cloud primitive, security tool, and API standard claim is checked against official docs before implementation |
| Contract gate | OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts lint and generate clients |
| Tenancy gate | Cross-tenant fuzz and RLS tests fail closed |
| Policy gate | Cedar policies cover sensitive actions, feature activation, and agent autonomy |
| Audit gate | Mutations and policy decisions emit evidence and can be replayed |
| Data boundary gate | Search, RAG, analytics, ads, exports, and provider routing enforce data class and consent |
| Layer gate | Kernel/domain/application/adapter/API dependencies point inward only |
| Supply-chain gate | Images are signed, SBOMs exist, provenance is attached, forbidden licenses fail |
| Deployment gate | OpenTofu plan, Argo CD sync, drift detection, rollback, and restore evidence exist |
| Ops gate | SLOs, alerts, runbooks, owners, incident workflow, and restore drills exist |
| Localization gate | Regional pack obligations are declared, tested, and evidenced |

Recommended local command vocabulary:

```bash
git worktree add -b <branch> <isolated-worktree> origin/dev
git status --short --branch
oya verify --ci-required
oya gate run-all
git push -u origin <branch>
gh pr create --base dev --head <branch>
```

## 14. Success Metrics

### 14.1 Product

| Metric | Target |
|---|---:|
| Tenant onboarding time | p99 <= 5 minutes |
| Tenant activation failure rate | < 1% after retry |
| Weekly active users in first tenant cohort | >= 70% of provisioned users |
| Workflow completion without manual operator intervention | >= 95% |
| Admin tasks completed without support ticket | >= 90% |
| Search result permission violations | 0 |
| DSR completion evidence coverage | 100% |

### 14.2 Engineering And Operations

| Metric | Target |
|---|---:|
| Cross-tenant violations on mainline | 0 |
| Unversioned public API changes | 0 |
| Unsigned production images | 0 |
| Restore drill success | 100% for critical cells |
| SLO owner coverage | 100% for production services |
| Evidence emission coverage for regulated actions | 100% |
| Agent runs without capability registration | 0 |
| Provider calls without policy and data-boundary evidence | 0 |

## 15. Competitive Benchmark

Oyatie should be benchmarked against integrated and best-of-breed products, but
it should not copy their architecture blindly.

| Category | Products to benchmark | Oyatie parity target |
|---|---|---|
| Workspace | Google Workspace, Microsoft 365, Naver Works | Professional communication and productivity baseline with stronger tenant evidence |
| Workflow | n8n, Zapier, Temporal-backed internal systems, ServiceNow workflows | Typed workflow authoring, replay, approval, idempotency, and governance |
| Ontology | Palantir Foundry Ontology-style modeling | Business object layer with workflow and audit integration |
| Cloud | AWS, Azure, GCP, Naver Cloud, KT Cloud, NHN Cloud | Sovereignty-aware cloud primitives and integrated tenant audit |
| Search | Google Search, OpenSearch-based enterprise search, Elastic | Tenant-private search, public search path, and RAG-safe retrieval |
| Agent runtime | OpenAI/Codex-class, Claude-class, Gemini-class, enterprise agent platforms | Multi-provider capability runtime with autonomy ceiling and evidence |
| Ops | Datadog, Grafana stack, Backstage, incident platforms | One operational control plane tied to deploy, evidence, SLO, cost, and recovery |

Benchmark acceptance:

- Every benchmark must name the user job, not just a feature checklist.
- Every copied pattern must survive tenancy, policy, audit, and regional-pack
  review.
- A product may lag on breadth in first release, but must not lag on isolation,
  evidence, policy, recovery, or contract quality.

## 16. Risks And Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Product scope is too broad for first release | High | Sequence foundation, Foundry/workflow, application/workspace, cloud/ops, then first vertical |
| Tenant data leaks into search, RAG, provider prompts, analytics, or ads | Critical | Data-boundary gate, deny-by-default egress, data-class tests, audit evidence |
| Agents bypass autonomy limits | Critical | Capability registry, Cedar decision, tool allowlist, run evidence, human approval for high-risk capabilities |
| Architecture becomes prose-only | High | Source contracts, lints, generated SDKs, gates, and CI evidence |
| Regional packs fork core product behavior | High | Locale-neutral core with pack-specific overlays and machine-readable pack tests |
| Cloud becomes an internal-only abstraction and cannot be sold | Medium | Build internal cells using the same primitives exposed externally |
| Open-source stack drift or licensing changes | Medium | Source verification, vendor recency checks, license gates, replacement seams |
| Operational evidence is incomplete | High | Evidence on hot path, restore drills, SLO ownership, incident closeout requirements |
| Performance targets are aspirational | Medium | Load tests and p50/p99/p999 budgets before release claims |
| Teams reintroduce hidden product coupling | High | Dependency gates, workflow/ontology integration rule, code review checklist |

## 17. Open Product Questions

These do not block architecture, but they must be answered before broad public
launch:

1. Which exact first vertical ships commercially after the foundation release:
   corporate operations, healthcare, industrial, or fintech?
2. What is the first paid packaging model: per-seat SaaS, per-workflow usage,
   per-resource cloud billing, or a bundle?
3. Which regional pack follows KR: JP, US, or EU?
4. When does public search become externally visible rather than tenant-private?
5. When does ads graduate from internal tenant-facing inventory to external
   advertiser onboarding?
6. Which agent provider modes are acceptable for strict-residency tenants?
7. Which cloud primitives must be built in-house immediately versus wrapped
   behind stable adapters while demand is validated?

## 18. External Source Baseline

The following official sources define the stack facts that must be rechecked at
implementation time. This PRD remains complete without reading them, but
implementation must not rely on stale assumptions when a source has changed.

| Area | Official source |
|---|---|
| Rust manifest, edition, and MSRV metadata | https://doc.rust-lang.org/cargo/reference/manifest.html |
| Cargo workspaces | https://doc.rust-lang.org/cargo/reference/workspaces.html |
| OpenAPI 3.2.0 | https://spec.openapis.org/oas/v3.2.0.html |
| AsyncAPI 3.1.0 | https://www.asyncapi.com/docs/reference/specification/v3.1.0 |
| Protocol Buffers and proto3 | https://protobuf.dev/overview/ and https://protobuf.dev/programming-guides/proto3/ |
| Leptos SSR and islands | https://book.leptos.dev/ssr/ and https://book.leptos.dev/islands.html |
| Kubernetes | https://kubernetes.io/docs/concepts/overview/ |
| OpenTofu | https://opentofu.org/docs/intro/ |
| Argo CD | https://argo-cd.readthedocs.io/en/stable/ |
| Jenkins Pipeline | https://www.jenkins.io/doc/book/pipeline/ |
| Cedar policy language | https://docs.cedarpolicy.com/ |
| OpenTelemetry | https://opentelemetry.io/docs/ |
| PostgreSQL | https://www.postgresql.org/docs/current/intro-whatis.html |
| Citus | https://docs.citusdata.com/ |
| Kafka KRaft | https://kafka.apache.org/40/operations/kraft/ |
| Apache Iceberg | https://iceberg.apache.org/spec/ |
| OpenBao | https://openbao.org/docs/ |
| Cilium | https://docs.cilium.io/en/stable/ |
| SPIFFE/SPIRE | https://spiffe.io/docs/latest/spire-about/ |
| Envoy | https://www.envoyproxy.io/docs |
| Wasmtime | https://docs.wasmtime.dev/introduction.html |
| Kata Containers | https://katacontainers.org/docs/ |
| Cloud Hypervisor | https://www.cloudhypervisor.org/ |
| OpenSearch | https://docs.opensearch.org/docs/latest/ |
| Milvus | https://milvus.io/docs/overview.md |
| SeaweedFS | https://seaweedfs.com/docs/ |
| Kyverno | https://kyverno.io/docs/introduction/ |
| Sigstore/cosign | https://docs.sigstore.dev/cosign/signing/overview/ |
| SLSA | https://slsa.dev/ |
| GitHub Actions | https://docs.github.com/en/actions |

## 19. Implementation Prompt

Use this prompt when handing the project to an implementation agent:

```text
You are implementing Oyatie from scratch.

Build one integrated ecosystem-as-a-service for regulated organizations:
application entry surface, tenancy, identity, policy, audit chain, workspace tools,
workflow studio, ontology, Foundry agent runtime, cloud infrastructure, search,
analytics, ads path, Ops Control Center, developer surface, and regional packs.

Use Rust 2024 with Cargo workspace resolver 2. Backend services use Tokio,
Hyper, Serde, structured tracing, and strict layer boundaries. New web surfaces
use Leptos SSR-first with selective WASM hydration. REST contracts use OpenAPI
3.2.0, event contracts use AsyncAPI 3.1.0, and gRPC uses proto3. External APIs
use date-based version carriers plus SDK semver.

Use PostgreSQL plus Citus for OLTP, Valkey for cache/KV, Kafka KRaft plus
outbox for eventing, OpenSearch for search, Milvus where dedicated vector
search is justified, S3-compatible object storage, Apache Iceberg as the OLAP
write path, and ClickHouse only as query/compute over Iceberg. Use Cedar for
application authorization, feature activation, and agent autonomy ceilings;
Kyverno for Kubernetes admission policy; OpenBao for secrets/KMS; SPIFFE/SPIRE
for service identity; Cilium for network policy; Envoy for L7 proxying where
needed; Wasmtime for WASM plugin sandboxing; Kata plus Cloud Hypervisor-class
isolation for untrusted tenant code.

Deploy server-side workloads to Kubernetes. Use OpenTofu for infrastructure as
code, Argo CD for GitOps, signed OCI images, SBOMs, cosign/Sigstore provenance,
SLSA posture, GitHub Actions for hosted CI, and Jenkins LTS for self-hosted or
air-gapped parity. Use the plain-git branch, PR, Jenkins, `oya gate` / `oya verify`, and reviewer/governance lifecycle for
agent-safe changes.

Hard rules:
- One tenant boundary everywhere.
- One policy surface for app authorization, feature activation, and agent
  autonomy.
- One audit chain for mutations, policy decisions, provider calls, and evidence.
- Cross-product action goes through workflow.
- Cross-product shared state goes through ontology.
- No unregistered agent capability can run.
- No data leaves a tenant boundary, region, or data class without explicit
  policy and audit evidence.
- No public API, event, proto service, SDK, policy, or migration ships
  unversioned.
- No production workload ships without SLO, owner, observability, backup,
  restore, rollback, and deployment evidence.

Delivery order:
1. Foundation: tenancy, identity, policy, audit, data boundary, cell model,
   service identity, observability, contracts, and verification gates.
2. Foundry plus workflow: capability registry, agent runtime, provider adapters,
   workflow engine, workflow studio, ontology, and evidence.
3. Application plus workspace: tenant admin, app switcher, product enablement,
   billing entry points, audit viewer, mail, chat, calendar, drive, docs, forms,
   meet, tasks, notes, and search.
4. Cloud plus ops: internal cells, compute, storage, network, KMS, billing,
   OpenTofu, Argo CD, Kubernetes, backup, restore, DR, capacity, cost, carbon,
   and Ops Control Center.
5. First commercial vertical: prefer corporate operations unless product council
   selects a different regulated vertical.

Before writing code for any stack component, verify the current official source
for that component. If the source conflicts with this prompt, pause, record the
delta, and update the product decision before implementation.
```
