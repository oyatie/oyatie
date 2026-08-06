---
id: ADR-0352
title: Oyatie from-scratch architecture handoff
status: Rejected
date: 2026-05-22
owner: council-architecture
doc_class: Architecture-Decision-Record
shape: Self-Contained-Synthesis-Handoff
authority_tier: 1
enforcement_status: handoff-synthesis-no-new-lanes
purpose: >
  Define the complete self-contained architecture, product scope, technical
  stack, runtime doctrine, delivery sequence, governance workflow, and
  implementation prompt for building Oyatie from scratch. This file is written
  so an implementation agent can understand the intended system without
  following pointers to any other existing file.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: From-scratch architecture handoff — narrative dump; not a decision unit

# ADR-0352: Oyatie from-scratch architecture handoff

## Status

Proposed on 2026-05-22.

This ADR is a complete handoff artifact. It intentionally contains the operative
architecture and implementation contract inline. An implementation agent should
not need to open another existing source file to understand the target product,
target architecture, target stack, delivery sequence, non-negotiable invariants,
or validation posture.

## Context

Oyatie is an agentic-primary, machine-optimized, programmatically governed
ecosystem-as-a-service platform. It is not a bundle of loosely related products.
It is one coherent tenant-scoped operating substrate with flat microservices,
shared policy, shared ontology, shared workflow, shared evidence, shared
deployment automation, and product surfaces projected over those shared
substrates.

The failure mode this handoff prevents is a greenfield implementer reviving
stale boundaries: standalone Foundry, standalone cell, standalone network,
standalone shorts, Redis, suite services, central mediators, ad hoc versioning,
policy-in-code, manual deployment, silent fallback, or product-specific
substrates. The target shape is explicit below.

## Decision

Adopt this ADR as the self-contained from-scratch architecture handoff.

### D-1. Non-negotiable doctrine

Oyatie development follows these principles:

- Agentic-primary: agents are first-class implementers and operators.
- Machine-optimized: durable truth is structured, typed, and mechanically
  checkable wherever possible.
- Programmatic where possible: manual work needs an exception and an automation
  path.
- Deterministic where it matters: gates, policy, promotion, IDs, evidence, and
  deployment must be reproducible.
- Enforce in everything: architecture claims require validators or evidence.
- Iterate until consensus: important architecture decisions must survive
  adversarial review.
- No silent regression: contract, behavior, evidence, policy, and deployment
  changes must be visible and traceable.
- Canonical base plus localization: ship the global base and localization packs
  deliberately; do not fork the product per region.
- No sprawl: one concept gets one owner, one bounded context, one vocabulary.
- Rust is the default language for new services, tools, coordination code, and
  backend logic.

### D-2. First deliverable

Build the first production deliverable before later sector verticals.

The first deliverable is Tenant RBAC view plus Tenant RBAC view. It is full-depth
production scope, not MVP, not preview, and not reduced scope.

Required surfaces:

- core
- messenger
- mail
- community
- infrastructure
- ops-dashboard-control-center
- intelligence
- workflow
- ontology
- canonical base
- Korea localization pack

Exit bar:

- production-grade customer workflows
- hyperscaler-grade system design
- industry-leading comparative position
- complete PRD, phase, implementation-plan, acceptance, test, and evidence
  coverage
- no placeholders, stubs, thin scaffolds, or deferred scope inside accepted
  first-deliverable scope
- security, privacy, tenant isolation, audit-chain, SLO, runbook, DR, cost,
  FinOps, rollback, import/export, migration, and vendor-exit evidence
- one-command or one-click reproducible Kubernetes deployment
- multi-architecture OCI artifacts for amd64 and arm64
- distroless or scratch production images by default
- secure cluster join, hardened bootstrap, restore evidence, and rollback
  evidence

The Korea localization pack includes:

- regulatory bindings
- Cedar policy fragments
- workflow templates
- Typst document templates
- Mail, Messenger, and Community localization
- Enterprise and SMB operating flows
- audit-chain evidence
- data residency and privacy controls
- import, export, and migration paths
- localized runbooks and SLOs
- Ops Control Center escalation flows

No later regulated or sector-specific vertical may claim production GA until the
first deliverable has evidence for this exit bar.

### D-3. Canonical build sequence

Build in this order.

Phase 0: shared cloud infrastructure.

- cloud IAM
- cloud KMS
- cloud secrets
- cloud IaC
- cloud network and DNS
- cloud data
- cloud storage
- cloud compute functions
- cloud compute Kubernetes
- cloud compute VM
- cloud billing
- cloud billing tax
- cloud capacity
- cloud cell context
- cloud DC operations
- cloud FinOps
- cloud marketplace primitives
- cloud file-system and handoff substrate

Phase 1: foundations and platform substrate.

- identity
- tenancy
- audit-chain
- governance
- compliance
- observability
- payments
- finops-portal
- api-gateway
- application shell
- developer-sdk
- network substrate
- cell-pattern support

Phase 2: core capability substrate.

- intelligence
- ontology
- workflow-engine
- workflow-studio
- consent-graph
- detection

Phase 3: communication and collaboration.

- messenger
- mail
- drive
- calendar
- meet
- recordings
- notes
- docs
- sheets
- slides
- forms
- connect
- comms-email
- community
- analytics
- tasks
- translate
- search

Phase 4: distribution and B2B SaaS surface.

- marketplace
- plugin-app-store
- workplace-integration
- Cedar-backed feature activation
- ops-dashboard-control-center
- brand
- sites
- flat B2B SaaS product services

Serialized lanes:

- root workspace manifest changes
- shared contract changes
- shared data ownership changes
- portable deployment target changes
- shared GitOps and OpenTofu module changes
- bootstrap entrypoints
- secure cluster join
- production hardening baseline
- branch and promotion policy

Parallel product lanes may fan out only after shared contracts, shared policy,
shared data ownership, and shared deployment substrates are locked.

### D-4. Service-boundary reconciliation

The following service-boundary decisions are mandatory in a from-scratch build:

- Foundry is not a standalone microservice.
- Foundry responsibilities live inside intelligence.
- The `oyatie.foundry.*` principal namespace remains only for
  self-modification authorization, operational identity, and audit.
- retired external agent harness terminology is retired.
- `cell` is not a standalone microservice.
- Cellular architecture remains mandatory as a topology and runtime pattern.
- `network` is retired into community.
- LinkedIn-class jobs, professional profile, recruiting, and workplace
  discussion content belongs to community.
- `shorts` is retired into social.
- Short-form video is a social media flavor, not a separate service.
- Redis is not canonical.
- Valkey is canonical for cache, KV, pub/sub, and stream-like cache duties.
- Apache Iceberg is the canonical OLAP table-format write path.
- Delta and Hudi are migration adapters only.
- ClickHouse is compute/query over Iceberg, not the canonical write path.
- Feature activation is Cedar-backed policy, not LaunchDarkly, Flagsmith,
  Unleash, or another feature-flag service.

### D-5. Architecture model

The architecture is flat, API-first, independently scalable, and cleanly
layered.

Rules:

- No grouping microservices.
- No bundle microservices.
- No vertical-grouping microservices.
- No product-specific duplicates of shared substrates.
- One service owns one bounded concern.
- Each service independently deploys.
- Each service independently scales horizontally.
- Contracts exist before handlers.
- Business logic does not live in HTTP, gRPC, queue, worker, or UI handlers.
- Direct product-to-product business coupling is forbidden.
- Typed direct service calls are allowed only when they carry mTLS identity,
  Cedar authorization, audit emission, distributed tracing, versioned contract,
  and ontology projection where data crosses service ownership.
- Workflow and Ontology are composition substrates, not universal data-path
  mediators.
- Policy-engine, Intelligence, and Ontology read paths are library-first with
  network opt-in only where explicitly justified.

Layer model:

- `kernel`: pure invariants, value types, identity types, port traits, no I/O,
  no async runtime, no filesystem, no network, no framework, no clock reads.
- `domain`: business logic over kernel types, calls through kernel ports, no
  direct provider dependencies.
- `app` or `usecase`: use-case composition, one operation per public function,
  holds port bounds but no concrete adapters.
- `api`: inbound HTTP/gRPC/GraphQL adapter; extract, validate, call app, format.
- `worker`: inbound queue, stream, timer, cron, or background adapter.
- `adapter`: outbound implementation of ports for databases, queues, providers,
  stores, files, secrets, model providers, and telemetry.
- `runtime`: composition root; wires concrete adapters to use cases and starts
  processes.

Forbidden dependency edges:

- kernel to any outer layer
- domain to app, api, worker, adapter, or runtime
- app/usecase to api, worker, adapter, or runtime
- adapter to app, api, worker, runtime, or peer adapter
- api to worker or worker to api
- any layer leaking provider-specific errors across its boundary

### D-6. Microservice taxonomy

Every microservice declares one taxonomy class:

- `substrate`: audience-neutral shared infrastructure or capability substrate.
- `product`: tenant-facing product capability.
- `service-cell`: service-cell deployment/control surface where explicitly
  required.
- `reserved`: certification-gated planned service with no live runtime.

Reserved services may contain planning and certification artifacts but cannot
ship live source code, live IaC, live runtime contracts, egress permits, or
SPIFFE issuance before promotion.

Marketplace is a unified brand surface over flat services, not a suite. Its
flat services include catalog, inventory, orders, fulfillment, reviews,
discovery, pricing, and trust-safety. Plugin-app-store ships first; physical,
C2C, services, and subscriptions come later under the same flat-service rule.

### D-7. Tenancy and policy

Oyatie itself is a tenant. There are no internal carve-outs.

Tenant ID and dotted sub-scope are universal primitives for:

- routing
- authorization
- audit
- retention
- residency
- attribution
- cost
- encryption
- compliance
- quota
- rate limits
- evidence

Audience is a call tag or tenant property, not a microservice scope.

Cedar policy rules:

- Cedar v4.2 is the application policy engine.
- Every policy decision defaults to `Forbid`.
- Deny wins in composition.
- Fail closed when policy evaluation is unavailable.
- Policy fragments are signed.
- Policy fragments are versioned.
- Policy fragments are hot-reloadable.
- Fragment scopes are baseline, pack, overlay, reserved, and tenant.
- Per-tenant overlays require signed lifecycle.
- Hot-path policy evaluation target is p99 <= 1 ms.
- Evaluation is library-first in-process by default.
- Distributed fragment snapshots may use Valkey-backed cache/distribution.
- Every Cedar decision emits audit-chain evidence.
- Kyverno owns Kubernetes admission policy.
- Cedar owns application authorization and feature activation.

### D-8. Cellular architecture

Cells are mandatory for blast-radius control. The cell concept is a topology and
runtime pattern, not a general-purpose central cell service.

Responsibility split:

- Tenancy owns tenant placement and tenant-cell assignment.
- Cloud-iac owns topology, provisioning registry, and OpenTofu-owned cell
  infrastructure.
- Observability owns health, blast radius, live utilization, and cell SLO burn.
- Api-gateway owns cell-aware routing.
- Audit-chain owns cell-scoped evidence.
- The shuffle-sharding library owns deterministic within-cell tenant-to-shard
  placement, hot split, and cold merge mechanics.
- `cell-rebalancer` owns cross-cell tenant migration as a long-running workflow.
- `cell-lifecycle` owns the logical cell entity state machine:
  Registered -> Activated -> Promoted -> Drained -> Decommissioned.

Cell invariants:

- cell isolation
- shuffle sharding
- static stability
- no cross-cell traffic unless explicitly permitted
- capacity headroom
- per-cell SLO burn
- multi-region failover
- residency-aware placement
- compliance-pack-aware placement
- audit emission for placement, migration, promotion, drain, and decommission

Cell promotion gates:

- error-budget health
- warm-soak duration
- canary SLO success
- cross-cell mesh health
- tenant-class coverage
- compliance-pack coverage
- quiet window before promotion
- Kyverno admission evidence
- audit-chain promotion event
- emergency override with sealed evidence and rollback plan

Sharding automation:

- Autosharding means control-plane-driven tenant-to-cell and tenant-to-shard
  placement.
- Auto-rebalance means reversible migration from hot cells to eligible cooler
  cells.
- Dynamic sharding means hot split and cold merge within a cell.
- Thresholds must be declared per service; no default-fill for load thresholds.
- Residency and compliance constraints always override load-balancing desire.

### D-9. Time, IDs, and coordination

Clock and ordering:

- Hybrid Logical Clock is the default ordering primitive.
- TrueTime-class coordination is reserved for high-precision regulated cells.
- Leap seconds smear.
- Wall-clock ordering is not a correctness primitive.
- Per-cell cron uses jitter.
- There is no global scheduler.

Coordination:

- Use sagas with compensation.
- Do not use distributed locks for business correctness.
- Do not use two-phase commit across services.
- Use outbox for event publication.
- Model "exactly once" as idempotency plus outbox plus at-least-once delivery
  plus dedupe.
- State-changing public operations carry idempotency keys.

IDs:

- UUIDv7 is canonical.
- UUIDv7 is used for event IDs, audit row IDs, changeset IDs, tenant IDs, cell
  IDs, principal IDs, resource IDs, request IDs, idempotency keys, evidence
  references, and related identifiers.
- UUIDv7 follows RFC 9562.
- Rust generation uses the `uuid` crate v1 line with v7 support and
  `Uuid::now_v7()`.
- Text form is lowercase hyphenated UUID.
- Postgres storage uses native UUID where available.
- SQLite storage uses text.
- Typed newtypes validate UUID version 7.
- UUIDv7 does not replace HLC ordering.
- ULID and Snowflake-style IDs are rejected.

### D-10. API, event, and SDK contract

Contract formats:

- REST uses OpenAPI 3.2.0.
- Event and channel surfaces use AsyncAPI 3.1.0.
- gRPC uses proto3.
- Contract definitions exist before handler implementation.
- Boundary validation is mandatory.
- Error semantics are consistent across services.

Public API versioning:

- Public version values are date strings in `YYYY-MM-DD` format.
- Canonical request header is `Oyatie-Version`.
- Canonical URL prefix is `/v/<YYYY-MM-DD>/`.
- Canonical protobuf field is `oyatie_version`.
- The last three public versions remain supported.
- Deprecation window is at least 180 days after deprecation.
- Tenant manifests pin effective public API version.
- Deprecated versions emit Sunset, Deprecation, and Link response headers.
- Removing a tenant-affecting version requires an explicit sunset calendar and
  audit event.

SDKs:

- SDK packages use semantic versioning.
- Each SDK release pins a public API date version under the hood.
- Canonical SDK languages are TypeScript, Python, Go, Java, Kotlin, Swift,
  Rust, .NET C#, C, and C++.
- SDKs are generated from contract sources.
- Hand-authored SDK drift from contract sources is forbidden.

### D-11. Technology stack

Backend, tools, and coordination:

- Rust by default.
- Shell scripts are limited to thin bootstrap/compatibility entrypoints.
- Python is not a default implementation language.
- Node is not a backend service runtime default.

Web and native clients:

- New web surfaces use Leptos authored in Rust.
- Web rendering is SSR-first.
- Pure SSR is the default for non-interactive sections.
- WASM hydration is selective and scoped to route, component, or island.
- CSR-only first-party web surfaces are forbidden.
- TypeScript is allowed for SDKs and narrow frontend cases.
- Swift/SwiftUI is used for iOS, iPadOS, and macOS native clients.
- Kotlin/Jetpack Compose is used for Android.
- WinUI 3 on C#/.NET 8+ is used for Windows native clients.
- GTK/libadwaita is used for Linux native clients where desktop native is in
  scope.

Identity, secrets, and policy:

- Zitadel/OIDC where identity-provider service is needed.
- OpenBao for secrets.
- BYOK is supported throughout regulated and enterprise surfaces.
- Tenant KMS/HSM integration is supported.
- Bootstrap secrets are externalized.
- SPIFFE/SPIRE identities are mandatory for service identity.
- Cedar v4.2 is the application policy engine.
- Kyverno is Kubernetes admission policy.

Data and storage:

- Primary OLTP is Postgres plus Citus per cell.
- Tenant isolation keys on `tenant_id`.
- Cache, KV, pub/sub, and stream-like cache duties use Valkey.
- Eventing uses Kafka with outbox.
- Search uses OpenSearch per cell.
- Vector storage uses Milvus.
- Object storage uses SeaweedFS plus S3-compatible stores such as R2, GCS, and
  managed S3-compatible stores.
- OLAP write path uses Apache Iceberg.
- ClickHouse may query/compute over Iceberg.
- Schema registry and contract registry are canonical platform substrates.

Observability:

- OpenTelemetry and OTLP
- Prometheus/OpenMetrics
- Mimir
- Loki
- Tempo
- Alertmanager
- Grafana
- OpenSLO
- structured logs
- distributed traces
- metrics
- postmortems

Runtime isolation:

- First-party application services use runc by default.
- Tenant-customer untrusted code uses Kata plus Cloud Hypervisor.
- Substrate workloads touching tenant data plane use Kata plus Cloud Hypervisor.
- Dedicated edge or performance-critical workloads may use runc-edge on
  dedicated node pools.
- WASM extensibility uses Wasmtime and WASI Preview 2 Component Model.
- WASM has no ambient filesystem, network, or clock access.
- WASM capabilities are Cedar-gated.
- WASM uses fuel, memory, and import allowlists per sandbox class.

Network and edge:

- Anycast GeoDNS.
- Cloudflare Workers, R2, and KV as first edge where applicable.
- Pingora-class self-operated edge later.
- HTTP/3 default.
- HTTP/2 fallback.
- HTTP/1.1 fallback.
- HTTP/1.0 forbidden.
- TLS 1.3 strict profile.
- HSTS, certificate transparency, OCSP stapling, and full chain validation.
- ECH where supported.
- PQ hybrid `X25519MLKEM768` where negotiated.
- Hybrid signatures for new oyatie-rooted chains where supported.
- Cilium ambient mesh.
- Envoy ingress.
- SPIFFE/SPIRE service identity.
- Egress denied by default unless Cedar and network policy allow it.

Infrastructure and supply chain:

- Kubernetes everywhere server-side except explicit edge exceptions.
- kubeadm plus containerd for self-managed on-prem contexts where applicable.
- OpenTofu-only infrastructure modules.
- Per-service IaC wrappers are thin; shared modules own real primitives.
- OCI images.
- Multi-arch amd64 and arm64.
- Distroless or scratch production images by default.
- Full base images require exception registry evidence.
- SBOM and provenance attestations are required.
- cosign verifies deployable artifacts.
- SLSA posture is required.
- cargo-deny and cargo-vet are part of dependency governance.

CI/CD:

- GitHub Actions for hosted PR checks.
- Jenkins LTS for self-hostable CI in air-gap, on-prem, colo, and
  Oyatie-as-provider contexts.
- Jenkins configuration uses JCasC and Jenkinsfile parity with hosted CI.
- ArgoCD is canonical GitOps CD.
- ArgoCD sync verifies signatures, enforces tenant namespace isolation, and
  emits audit-chain rows.
- Manual `kubectl apply` and manual Helm CLI deploys are replaced except for
  audited break-glass.
- `deployment-control-plane` owns upgrades, canaries, rollbacks, and air-gapped
  bundle delivery.

### D-12. Deployment models and portability

Product deployment models:

1. shared-cloud
2. dedicated-cloud
3. hybrid-byo-cloud
4. on-prem-connected
5. on-prem-air-gapped

Infrastructure contexts:

- guest-on-AWS
- guest-on-OCI
- guest-on-other-cloud
- colocation
- on-prem
- Oyatie-as-cloud-provider

The same service contracts, Cedar policy, containers, workflow definitions, and
deployment-control-plane semantics travel across all models. Infrastructure
adapters vary by context.

Supported host targets:

- Talos
- Ubuntu LTS
- Debian
- Fedora Server
- Oracle Linux
- RHEL-compatible distributions
- CentOS Stream
- Rocky Linux
- AlmaLinux
- SUSE Linux Enterprise
- macOS Apple Silicon local Kubernetes

Portability requirements:

- one-command setup
- one-click setup where productized UX exists
- remote config-driven secure cluster join
- fail-closed bootstrap prerequisites
- externalized bootstrap secrets
- production hardening evidence
- cluster membership evidence
- restore evidence
- rollback evidence
- DR drill evidence
- multi-arch OCI images
- image size and vulnerability budgets
- no host distribution lock-in

### D-13. Intelligence and self-modification

Intelligence is the active AI substrate.

Intelligence includes:

- inference runtime
- model registry
- embeddings
- RAG
- vector store integration
- tool-call router
- agent orchestration
- eval runner
- safety filter
- tracing
- BYOK key broker
- provider adapter registry
- tier/classifier
- consent projection
- rate limit and quota
- data-class routing
- guardrails
- red-team/eval workflows
- training and fine-tuning workflow surfaces
- attribution
- consumer brand surface

Delivery shape:

- Caller path is library-first by default.
- Network hops are opt-in.
- Network hops require Cedar authorization and an explicit reason.
- BYOK credentials are tenant-owned in regulated and enterprise contexts.
- Substrate-owned provider credentials are not used for regulated/BYOK tenant
  calls.
- Supported modalities are text, image, audio, video, code, and multimodal.
- Autonomy levels are assist, co-pilot, autonomous-bounded, and
  autonomous-self-modifying.

Self-modification:

- Oyatie can modify itself only through Cedar-gated self-modification
  principals.
- The `oyatie.foundry.*` namespace is an operational principal namespace, not a
  Foundry service.
- Self-modifying actions emit audit-chain evidence.
- Self-modifying actions require policy, evidence, and rollback.

### D-14. Oya VCS and agentic delivery

Oya VCS is the agent-facing coordination, changeset, evidence, merge, and
promotion surface.

Required lifecycle:

1. claim
2. work
3. verify
4. done
5. promote

Claim command shape:

```bash
oya vcs claim --agent <id> --intent "<slice>" <file-or-symbol-identifier>
```

Verify command shape:

```bash
oya vcs verify --agent <id> --evidence <evidence-reference>
```

Done command shape:

```bash
oya vcs done --agent <id> --evidence <evidence-reference>
```

Promote command shape:

```bash
oya vcs promote --agent <id> --bundle <bundle-id> --environment <environment>
```

Rules:

- `oya git <subcommand>` is the git drop-in surface.
- Raw provider primitives are not the agent-facing policy surface where Oya VCS
  applies.
- Changesets become ChangeBundles.
- ChangeBundles become promotion/release-train artifacts.
- Controller-owned rebase and merge queue prevent agent concurrency drift.
- Every changeset carries multispectrum evidence v2.4.0.
- Evidence must include architecture, tests, security, privacy, policy,
  operability, documentation, and residual-risk facets appropriate to the
  change.

Verifier contract:

```bash
oya verify --ci-required
cargo fmt --all --check
cargo check --workspace --all-targets --keep-going
cargo clippy --workspace --all-targets --keep-going -- -D warnings
cargo nextest run --workspace --no-fail-fast
oya gate run-all --ci-required
```

Skip flags are a closed development-only allowlist. The forward contract does
not allow skipping cargo check.

### D-15. Compliance, privacy, audit, DR, and evidence

Compliance packs are first-class signed bundles.

Each pack contains:

- manifest
- Cedar fragments
- IaC overlays
- audit schema additions
- data-class registrations
- retention policies
- regulator evidence emitters
- sovereign overlays
- workflow templates
- tenant activation checklist
- operational runbooks
- SLO floors
- DR floors
- residency controls
- privacy controls

Initial compliance families:

- SOC2
- ISO 27001
- ISO 27017
- ISO 27018
- ISO 22301
- GDPR
- CCPA
- KR PIPA
- KR CSAP
- KR FSS
- HIPAA
- PCI
- FedRAMP
- PSD2
- NDMO KSA
- GAIA-X
- EU AI Act
- UK GDPR
- JP APPI

DR:

- Every active service declares numeric RTO and RPO targets.
- Every active service declares multi-region posture.
- Every active service declares backup substrate.
- Every active service declares failover runbook.
- Every active service records last drill evidence.
- Compliance packs declare DR floors.
- Effective tenant DR target is no weaker than both service declaration and
  every applicable pack floor.

Audit:

- Every privileged action emits audit-chain evidence.
- Every policy decision emits audit-chain evidence.
- Every placement, migration, deploy, rollback, promotion, and self-modifying
  action emits audit-chain evidence.
- Audit rows carry tenant, cell, principal, resource, action, HLC timestamp,
  request ID, trace ID, data class, policy decision, evidence reference, cost,
  carbon, watt-hours, provider, and region.

FinOps and sustainability:

- Every audit row carries cost in minor USD units.
- Every audit row carries CO2 grams.
- Every audit row carries watt-hours.
- Every audit row carries provider and region.
- Every service declares sustainability emission model.
- Finops-portal rolls up by tenant, product, capability, provider, cell, and
  compliance pack.
- electricityMaps is the canonical carbon-intensity source with documented
  fallback to provider or grid average.
- Carbon-aware scheduling is allowed only when SLO, compliance, and DR budgets
  permit deferral.

Security:

- least privilege
- envelope encryption
- BYOK
- data residency
- data perimeter
- data lineage
- SLSA posture
- cosign signing and verification
- SBOM/provenance
- cargo-vet
- cargo-deny
- confidential-compute support where appropriate
- no silent fallback
- explicit brown-out/degradation signals

OSS stewardship:

- Every direct OSS dependency declares stewardship class.
- Classes are Maintainer, Contributor, and Consumer.
- Maintainer means Oyatie owns commit and release.
- Contributor means Oyatie actively contributes upstream.
- Consumer means Oyatie pins, monitors, and updates without upstream ownership.
- Contributor P0 CVE response target is <= 7 days.
- Contributor P1 CVE response target is <= 30 days.
- Consumer public-CVE pin update target is <= 14 days.
- Do not call stewardship class a tier. Tier is reserved for cellular tier and
  pod runtime tier.

### D-16. Per-service manifest contract

Every active service declares these fields when applicable:

- service name
- service taxonomy class
- owner team
- bounded context
- clean architecture layer map
- crate/process map
- public API versions
- SDK generation coverage
- capacity model
- baseline CPU per tenant
- baseline RAM per tenant
- baseline storage per tenant
- baseline connections per tenant
- scaling dimension
- cell placement class
- sharding automation
- pod runtime tier
- supported deployment models
- supported infrastructure contexts
- compliance pack support
- data classes
- residency constraints
- audit event classes
- SLOs
- error budget policy
- runbooks
- threat model
- DPIA when required
- DR block
- sustainability emission model
- OSS stewardship references
- import/export paths
- migration paths
- vendor-exit paths
- policy fragments
- OpenAPI contracts
- AsyncAPI contracts
- proto contracts
- ontology object types
- workflow events

### D-17. Hyperscaler invariants

Implementation must prove these with evidence:

- cell isolation
- shuffle sharding
- static stability
- idempotency
- outbox
- sagas
- no 2PC
- no distributed lock correctness dependency
- circuit breakers
- bulkheads
- backpressure
- load shedding
- no silent fallback
- multi-region failover
- capacity reservation
- SLO and error budgets
- burn-rate alerts
- four golden signals
- USE method
- distributed tracing
- structured logs
- incident runbooks
- postmortems
- least privilege
- envelope encryption
- data perimeter
- data residency
- data lineage
- supply-chain provenance
- progressive delivery
- rollback
- provider-degraded shedding
- toil reduction
- FinOps tagging
- sustainability metrics

### D-18. Implementation output contract

A greenfield implementation plan produced from this handoff must output:

- service catalog with taxonomy class per service
- service boundary rationale per service
- complete first-deliverable scope map
- phase-by-phase build plan
- serialized and parallel work boundaries
- contract inventory
- manifest schema
- service manifest for every active service
- clean architecture crate/process layout
- policy model
- tenant model
- cell model
- API versioning model
- SDK generation model
- data substrate model
- intelligence substrate model
- workflow and ontology integration model
- deployment model matrix
- infrastructure context matrix
- compliance pack model
- DR model
- observability model
- audit-chain model
- FinOps and sustainability model
- Oya VCS workflow
- CI/CD workflow
- local verification commands
- production promotion gates
- rollback strategy
- non-goals and retired-boundary refusal list

## Implementation Agent Prompt

Use this prompt as-is when starting an empty repository or greenfield
implementation branch:

```text
You are implementing Oyatie from scratch from a self-contained handoff.

Build the first production deliverable first: Tenant RBAC view plus SMB
Generic, full-depth production scope, with core, messenger, mail, community,
infrastructure, ops-dashboard-control-center, intelligence, workflow, ontology,
canonical base, and Korea localization pack. Do not build later sector verticals
until the first deliverable has production evidence.

Use flat single-concern microservices. Do not create suite, bundle, axis, or
vertical-grouping microservices. Use clean architecture with kernel, domain,
app/usecase, api, worker, adapter, and runtime layers. Contracts come before
handlers. Business logic does not live in handlers. Each service independently
deploys and scales.

Use Rust by default for services, backend logic, tools, and coordination. Use
Leptos SSR-first with selective WASM hydration for new web surfaces. Use
TypeScript for SDKs and narrow frontend cases only. Use Swift/SwiftUI, Kotlin/
Jetpack Compose, WinUI 3 C#/.NET, and GTK/libadwaita only at native client
boundaries.

Use Postgres+Citus, Kafka, Valkey, OpenSearch, Milvus, SeaweedFS/S3-compatible
object storage, Apache Iceberg, OpenBao, Zitadel/OIDC, Cedar v4.2, Cilium,
Envoy, SPIFFE/SPIRE, Wasmtime, Kubernetes, OpenTofu, OCI images, cosign,
Jenkins LTS, ArgoCD, GitHub Actions, deployment-control-plane, and Oya VCS.

Resolve stale service boundaries before coding: Foundry is absorbed into
intelligence; cell is a topology pattern plus cell-rebalancer and
cell-lifecycle, not a central service; network is absorbed into community;
shorts is absorbed into social; Redis is retired in favor of Valkey; Iceberg is
the OLAP write path; feature activation is Cedar-backed policy.

Do not introduce universal mediators. Policy-engine, intelligence, and Ontology
reads are library-first with network opt-in. Direct service calls are allowed
only under mTLS identity, Cedar authorization, audit emission, distributed
tracing, typed contracts, and ontology projection when service-owned data
crosses boundaries.

Implement public APIs with OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3. Public
API versions are YYYY-MM-DD and are carried by Oyatie-Version, /v/<YYYY-MM-DD>/,
and oyatie_version. SDK packages use semver and pin public date versions under
the hood. Generate SDKs for TypeScript, Python, Go, Java, Kotlin, Swift, Rust,
.NET C#, C, and C++.

Use HLC for default ordering, UUIDv7 for IDs, sagas for coordination,
idempotency keys for state-changing public operations, outbox for event
publication, and at-least-once plus dedupe for exactly-once semantics. Do not
use distributed locks or 2PC for business correctness.

Every service must declare taxonomy class, capacity model, sharding automation,
pod runtime tier, API versioning, DR, sustainability emission model, compliance
pack support, data classes, residency constraints, audit events, SLOs, runbooks,
threat model, import/export paths, migration paths, and vendor-exit paths.

Deploy to Kubernetes with OpenTofu and GitOps. Support shared-cloud,
dedicated-cloud, hybrid-byo-cloud, on-prem-connected, and on-prem-air-gapped
models across guest cloud, colocation, on-prem, and Oyatie-as-provider contexts.
Require one-command or one-click setup, secure cluster join, externalized
bootstrap secrets, hardening evidence, restore evidence, rollback evidence,
multi-arch OCI images, and distroless or scratch production images by default.

Use Oya VCS for agent work: claim, work, verify, done, promote. Capture
multispectrum evidence for every changeset. Before completion, run the full
local verification mirror: oya verify --ci-required; cargo fmt --all --check;
cargo check --workspace --all-targets --keep-going; cargo clippy --workspace
--all-targets --keep-going -- -D warnings; cargo nextest run --workspace
--no-fail-fast; oya gate run-all --ci-required.

Never declare implementation-ready, planning-complete, production-grade, or
hyperscaler-grade without corresponding gate output and evidence artifacts.
```

## Consequences

### Positive

- A greenfield implementer can work from one complete ADR.
- Retired service boundaries are explicitly refused.
- The technical stack is explicit.
- The delivery sequence is explicit.
- Product scope and non-goals are explicit.
- Verification and promotion posture are explicit.

### Negative

- This ADR is long because it intentionally replaces pointer-following with
  inline substance.
- If upstream doctrine changes, this ADR must be amended directly; a separate
  source changing elsewhere is not enough to update this handoff.
- Some details that would normally be registry entries are duplicated here so
  the handoff remains self-contained.

### Operational

Any implementation plan that claims to follow this handoff must either implement
these rules or explicitly propose an amendment to this ADR. Silent divergence is
not allowed.

## Alternatives Considered

### Alternative 1: Pointer-style handoff

Rejected. A pointer-style handoff forces the implementer to reconstruct the
current architecture from multiple existing sources and increases the chance of
reviving stale service boundaries.

### Alternative 2: Prompt-only handoff

Rejected. A prompt is useful for execution, but it does not preserve the
decision context, consequences, alternatives, and durable self-contained
contract expected from an architecture decision record.

### Alternative 3: Minimal ADR with source citations

Rejected. Source citations are valuable in ordinary ADRs, but this handoff is
intended to be complete on its own. Citations would turn it back into a pointer
document.
