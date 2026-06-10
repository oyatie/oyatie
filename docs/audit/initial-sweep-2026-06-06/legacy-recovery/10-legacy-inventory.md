# Legacy Product / Capability Inventory — "oyatie" docs portal

Recovered from the trashed legacy portal at `~/.Trash/oyatie/docs/data/` (JSON = source of truth).
Scope: every named product/service/module, every capability/substrate primitive, every
decision/PRD intent, and everything the legacy itself flagged as a source-gap.

Project self-description (THESIS-001): *"Oyatie is a full cloud-provider foundation and one
coherent ecosystem layer: infrastructure primitives, managed service modules, internal products,
public B2C experiences, and external B2B/customer workloads share the same tenant-aware substrate."*
The legacy was **architecture/docs only** — no runtime code shipped; every closure lane
(implementation / evidence / launch) is **open**.

---

## (a) PRODUCTS / SERVICES / MODULES the legacy names

### A1. Owned platform planes / engines (first-party "products" Oyatie builds & dogfoods)

| Name | One-line purpose | Legacy source file |
|---|---|---|
| Oyatie Policy Engine (PDP/PEP) | Proprietary Rust policy decision plane; Cedar authorization kernel + Zanzibar-inspired relationship graph; service-local enforcement points | policy-decision-plane.json |
| Oyatie Economic Substrate | One universal metering/quota/usage-evidence/FinOps-attribution control plane for cloud + module surfaces | billing-finops-substrate.json |
| Oyatie Trust Plane | Centralized identity federation, trust roots, KMS/key lifecycle, secret distribution, workload identity, break-glass | trust-plane-architecture.json |
| Oyatie Resource Control Plane | Universal resource control-plane API (canonical envelope, LRO, idempotency, reconciliation, finalizers, SDK gen) | resource-control-plane-architecture.json |
| Oyatie Traffic Plane | Tenant-aware API gateway ingress, service-to-service traffic, governed egress, context propagation, resilience | traffic-plane-architecture.json |
| Oyatie Network Fabric | Hyperscaler-compatible VPC, IPAM, subnets, route tables, security groups, private connectivity, NAT/egress, K8s networking | network-fabric-architecture.json |
| Oyatie Compute Substrate | VM instances, host pools, machine images, boot trust, placement, K8s clusters/node pools, repair/drain/upgrade | compute-substrate-architecture.json |
| Oyatie Storage Substrate | Block volumes, object buckets, file shares, snapshots, backups, replication, durability tiers, CSI attachment | storage-substrate-architecture.json |
| Oyatie Eventing/Messaging Substrate | Event buses, pub/sub topics, durable queues, append-only streams, webhooks, schema registry, replay, DLQ (CloudEvents/AsyncAPI) | eventing-messaging-architecture.json |
| Oyatie Workflow Orchestration Substrate | Durable workflow/operation/saga/human-approval/reconciliation engine (deterministic, replayable) | workflow-orchestration-architecture.json |
| Oyatie Metadata/Consistency Substrate | Metadata store, watch, lease, lock, leader election, fencing, resource-version, schema migration, cache consistency | metadata-consistency-architecture.json |
| Oyatie Reliability/Observability Plane | SLOs, error budgets, telemetry, alerting, incident response, blameless postmortems | reliability-observability-architecture.json |
| Oyatie State/Data Plane | State classes, data classification, backup/restore, encryption/KMS binding, retention, residency-aware replication | state-data-storage-architecture.json |
| Oyatie Capacity Admission Plane | Capacity truth, quota reservation, placement, scheduling, autoscaling, priority/preemption, overload protection | capacity-admission-architecture.json |
| Oyatie Artifact Supply Chain | Immutable artifact identity, SLSA-style provenance, Sigstore-compatible signing, SBOM, registry + runtime admission | artifact-supply-chain-architecture.json |
| Oyatie Control-Plane / Cell architecture | Global/regional/cell control planes + data/evidence/experience planes; cells = blast-radius isolation units | control-plane-cell-architecture.json |
| Org/Tenant/Resource Hierarchy | Organization→account→tenant→project→environment + service catalog, entitlement, quota, budget, residency | org-tenant-resource-hierarchy.json |
| **oya** (Rust monorepo toolchain) | First-class CLI: `verify`, `container-validate`, `tofu-validate`, `promotion-check`, container-build; local + CI entrypoint | root-hub.json, microservice-architecture.json, platform-foundation-prd.json |

### A2. Managed ecosystem module (the only concrete app-level module specced)

| Name | One-line purpose | Legacy source file |
|---|---|---|
| KR Enterprise/Corporate Workflow Editor & HR/Payroll Module (`enterprise-corporate-workflow`) | First production-quality managed ecosystem module: governed workflow editor/studio + approval inbox + KR HR/payroll workflow packs, composing substrate (NOT a separate platform). Market scope: KR only | enterprise-corporate-workflow-module.json |
| → Workflow packs inside it | 청구서/invoice-claim, 근태/attendance, 급여대장/payroll-ledger, 지급명세서/wage-statement, 지급내역/payment-history, 근로자 등록/worker-registration, 근로자 관리/worker-management, 연차/annual-leave | enterprise-corporate-workflow-module.json |

### A3. Service taxonomy — the 5 declared service categories (each spans many future modules)

| Category | Examples named | Legacy source file |
|---|---|---|
| Shared substrate services | tenancy, identity, policy, audit, metering, billing, service-catalog, observability | service-taxonomy.json |
| Cloud-provider primitives | regions, zones, cells, networking, compute-vm, managed-kubernetes, object/block storage, image-registry, dns, load-balancing | service-taxonomy.json |
| Managed ecosystem modules | workflow, search, analytics, media, commerce, payments, messaging, marketplace, developer-platform | service-taxonomy.json |
| Industry modules | healthcare, finance, logistics, retail, education, hospitality, public-sector | service-taxonomy.json |
| Product experiences | public B2C apps, internal ops tools, customer-built apps, partner extensions | service-taxonomy.json |

### A4. Reference microservice exemplars (named service roots, not yet built)

`services/identity/`, `services/policy-engine/`, `services/billing-finops/`, `services/compute-vm/`,
`services/managed-kubernetes/`, `services/workflow/` — microservice-architecture.json

### A5. First proof slice (SLICE-001)

| Name | One-line purpose | Legacy source file |
|---|---|---|
| First Proof Slice | Bind identity/tenancy + cloud resource catalog + KR enterprise workflow module into one end-to-end reference flow proving platform seams | first-proof-slice.json, service-taxonomy.json |

---

## (b) CAPABILITIES / SUBSTRATE PRIMITIVES

### B1. Capability map (Capability Translation Brief — 8 P0 capabilities)

| Capability | One-line purpose | Legacy source file |
|---|---|---|
| CAP-ID-TENANT — Identity & tenant context | principal, active context binding, org/account/tenant/project hierarchy, workload identity, federation/MFA | capability-translation-brief.json |
| CAP-CONTROL-LIFECYCLE — Resource control-plane lifecycle | resource envelope, CRUD, LRO, reconciliation, finalizers, drift detection | capability-translation-brief.json |
| CAP-AUDIT-EVIDENCE — Audit & evidence plane | immutable ledger, evidence export, forensic trail, retention | capability-translation-brief.json |
| CAP-POLICY-AUTHZ — Policy & authorization runtime | PDP/PEP contract, principal/context/resource/action tuple, signed bundles, graph consistency | capability-translation-brief.json |
| CAP-DEPLOYMENT-PATH — Deployment & promotion | oya verify, Jenkins, ArgoCD, dev/staging/prod, OpenTofu, Podman scratch/distroless | capability-translation-brief.json |
| CAP-COMMERCIAL-ENTITLEMENT — Commercial lifecycle & entitlements | SKU, entitlement enablement, usage event, quota/budget/billing scope | capability-translation-brief.json |
| CAP-API-EVENT-WORKFLOW — API/event/workflow contracts | OpenAPI, AsyncAPI, idempotency, domain events, workflow execution, retries/DLQs | capability-translation-brief.json |
| CAP-OPERATE-RELIABILITY — Operate/observe/recover | SLOs, telemetry, runbooks, incident path, error budgets, backup/DR | capability-translation-brief.json |

### B2. Resource catalog primitive types (SPEC-RESOURCE-CATALOG — reserved first-class resource types)

Common envelope (`api_version/kind/uid/spec/status/generation/resourceVersion/etag/finalizers/...`) plus
lifecycle states: requested→validating→provisioning→active→suspended→updating→deleting→deleted→failed. Types by category — source: cloud-resource-catalog.json:

- **substrate**: tenant_project, identity_principal, policy_binding, audit_stream, meter, quota, resource_operation, resource_type_registration, resource_event_subscription, api_schema_version
- **cloud-primitives**: region, zone, cell, vpc, subnet, vm_instance, kubernetes_cluster, object_bucket, ipam_pool, ipam_allocation, route_table, network_interface, security_group, network_acl, private_endpoint, nat_gateway, egress_gateway, compute_host, host_pool, machine_image, boot_profile, kubernetes_node_pool, compute_operation, block_volume, file_share, snapshot, backup_policy, backup_vault, storage_class, volume_attachment, replication_policy, lifecycle_policy
- **eventing**: event_bus, event_topic, event_subscription, message_queue, event_stream, webhook_destination, schema_registry, event_schema, dead_letter_queue, replay_cursor, outbox_record, inbox_record
- **workflow-orchestration**: workflow_definition, workflow_execution, activity_task, reconciliation_controller, operation_saga, compensation_step, approval_gate, workflow_schedule, worker_task_queue
- **metadata-consistency**: metadata_store, metadata_table, config_namespace, config_snapshot, lease, coordination_lock, election, watch_stream, resource_version_cursor, fencing_token, consistency_policy, schema_migration
- **hierarchy-governance**: organization, account, tenant, project, environment, service_catalog_item, service_enablement, entitlement, quota_policy, budget, billing_profile, residency_policy
- **ecosystem-modules**: module_installation

### B3. Cross-cutting hooks every microservice must wire (substrate-not-copy)

tenant_scope, active_context_binding, policy_enforcement_point, audit_event_emission,
quota_reservation, usage_event_when_metered, idempotency_key_for_mutations, trace_context,
structured_logging, health_and_readiness — microservice-architecture.json

### B4. Economic-substrate owned capabilities

meter registry, usage-event ingestion contract, quota reservation/consumption authority, allocation
& cost-dimension model, charge-target resolver, showback/chargeback fact tables, self-tenant
accounting, FOCUS-compatible export, OpenCost/K8s allocation bridge, audit/financial evidence linkage
— billing-finops-substrate.json

### B5. Identity / tenancy / context primitives

- Tenant kinds: external_organization, oyatie_reserved, public_consumer_context, partner_developer, enterprise_workload, personal_b2c, internal_platform — identity-tenancy-spec.json
- Principal context types: work, personal_b2c, internal_oyatie, partner_developer — principal-context-model.json
- Hierarchy terms: Organization, Account, Tenant, Project, Environment, Entitlement, Service enablement, Context binding, Session context, Principal context switch — glossary.json
- Coined concept: **"Ecosystem-as-a-Service"** (cloud primitives + managed modules + products + customer workloads sharing one substrate vs disconnected SaaS silos) — glossary.json

### B6. Trust-plane primitives

human-idp-federation (OIDC/SAML), principal-context-binding, workload-identity (SPIFFE-inspired
SVID-like), trust-root-registry, kms-key-management (9 key classes), secret-distribution,
certificate-token-lifecycle, break-glass-recovery, trust-evidence-plane — trust-plane-architecture.json

---

## (c) DECISIONS / PRD INTENT / REQUIREMENTS

### C1. Accepted ADRs (31) — the decision SSOT

| ADR | Title | Source |
|---|---|---|
| ADR-0001 | JSON is source of truth; HTML is rendering layer | adr-ledger.json |
| ADR-0002 | Oyatie is one coherent ecosystem layer | adr-ledger.json |
| ADR-0003 | Tenant-first platform model; Oyatie is also a tenant | adr-ledger.json |
| ADR-0004 | Cloud-provider primitives first-class from day one | adr-ledger.json |
| ADR-0005 | Reference audit promotion gate | adr-ledger.json |
| ADR-0006 | First proof slice = identity/tenancy + resource catalog + KR enterprise workflow module | adr-ledger.json |
| ADR-0007 | One canonical principal model with explicit context bindings | adr-ledger.json |
| ADR-0008 | Jenkins CI + ArgoCD GitOps + hygiene gates + automation-first verification early | adr-ledger.json |
| ADR-0009 | Adopt `oya` Rust-first monorepo toolchain with dev→staging→prod promotion | adr-ledger.json |
| ADR-0010 | Podman-first scratch/distroless container practice | adr-ledger.json |
| ADR-0011 | OpenTofu-first module-composed IaC | adr-ledger.json |
| ADR-0012 | Current stable + LTS stack baselines as version authority | adr-ledger.json |
| ADR-0013 | Oyatie Policy Engine: Cedar kernel + Zanzibar-inspired graph | adr-ledger.json |
| ADR-0014 | One billing/metering/quota/FinOps substrate for cloud + modules equally | adr-ledger.json |
| ADR-0015 | Clean architecture + flattened independently scalable microservices | adr-ledger.json |
| ADR-0016 | Separate control/data/evidence/experience planes + cells from day one | adr-ledger.json |
| ADR-0017 | Tenant-aware API gateway + service-to-service + egress traffic plane | adr-ledger.json |
| ADR-0018 | Reliability/observability/SLO/error-budget/incident/postmortem before prod | adr-ledger.json |
| ADR-0019 | State ownership/data classification/backup/encryption/lifecycle/residency before prod | adr-ledger.json |
| ADR-0020 | Capacity admission control plane (truth/quota/placement/autoscale/priority/overload) | adr-ledger.json |
| ADR-0021 | Artifact supply-chain provenance/signing/SBOM/registry/runtime admission | adr-ledger.json |
| ADR-0022 | Identity federation/trust roots/KMS/secret distribution/workload identity = one trust plane | adr-ledger.json |
| ADR-0023 | VPC/IPAM/security groups/private connectivity/K8s networking = first-class network fabric | adr-ledger.json |
| ADR-0024 | VM + Kubernetes clusters = first-class compute substrate lifecycle | adr-ledger.json |
| ADR-0025 | Block/object/file/snapshot/backup/K8s storage = first-class storage primitives | adr-ledger.json |
| ADR-0026 | Universal resource control-plane API lifecycle contract | adr-ledger.json |
| ADR-0027 | Eventing/messaging/streaming/webhook/schema/replay/delivery substrate | adr-ledger.json |
| ADR-0028 | Durable workflow/reconciliation/operation/saga/human-approval substrate | adr-ledger.json |
| ADR-0029 | Metadata/config/coordination/consistency/watch/lease/schema-migration substrate | adr-ledger.json |
| ADR-0030 | Org/account/tenant/project/environment/entitlement/service-catalog hierarchy | adr-ledger.json |
| ADR-0031 | Foundation ADR roadmap + decision portfolio governance | adr-ledger.json |

### C2. Planned candidate ADRs (29: ADR-0032 – ADR-0060) — declared future-decision portfolio

Source: foundation-adr-roadmap.json (all status=`planned`)

- ADR-0032 Global topology: regions/zones/cells/sovereignty/residency routing
- ADR-0033 Edge platform: DNS, certs, CDN, global LB, WAF/edge auth
- ADR-0034 Commercial lifecycle: SKUs, pricing, rating, invoicing, tax, GL, settlement
- ADR-0035 Product/service catalog & entitlement fulfillment
- ADR-0036 API covenant: OpenAPI 3.2 / AsyncAPI 3.1 / versioning / idempotency / errors / deprecation
- ADR-0037 Developer experience: console, portal, CLI, SDKs, API keys, OAuth apps, sandboxes
- ADR-0038 Policy runtime hardening: PDP SLOs, signed bundles, hot reload, Zanzibar graph storage, K8s admission
- ADR-0039 Collaboration model: groups, teams, delegation, cross-tenant sharing
- ADR-0040 Compliance evidence plane: SOC 2, ISO, industry packs, legal SLA evidence
- ADR-0041 Privacy & data governance: consent, purpose limitation, DSAR, minimization, legal hold
- ADR-0042 Trust & safety: abuse, fraud, spam, rate limits, risk engine
- ADR-0043 Customer operations: support, status page, incident comms, escalation, SLA lifecycle
- ADR-0044 Observability data platform: metrics/logs/traces/profiles, retention, cardinality controls
- ADR-0045 Fleet/datacenter/hardware lifecycle: asset inventory, host OS, repair, decommissioning
- ADR-0046 Workload runtime security: VM/K8s isolation, host agents, patching, multi-tenant hardening
- ADR-0047 Service mesh & workload networking runtime
- ADR-0048 Managed data services substrate: relational, key-value, cache, search, vector, analytics
- ADR-0049 Data lake, warehouse, event analytics, internal/customer analytics
- ADR-0050 Data lifecycle: retention, deletion, backup conflicts, archive, erasure verification
- ADR-0051 Release engineering: progressive delivery, feature flags, experiments, rollback
- ADR-0052 Migration, deprecation, compatibility, customer-notice policy
- ADR-0053 Security program & secure SDLC: threat modeling, vuln mgmt, pen testing, bug bounty
- ADR-0054 Business continuity & DR: chaos, GameDays, DR tiers, regional failover
- ADR-0055 Ecosystem module runtime & extension/plugin architecture
- ADR-0056 Marketplace & partner publishing lifecycle
- ADR-0057 Industry vertical architecture & compliance packs
- ADR-0058 B2C product substrate: social graph, content, feed, messaging, commerce, moderation
- ADR-0059 Search, recommendation, discovery, AI/ML governance
- ADR-0060 Sustainability, efficiency, carbon-aware capacity, waste reduction

(Roadmap also reserves a candidate-intake band ADR-0061 – ADR-0074 referenced by the capability brief
and readiness matrix; not individually titled in the roadmap — foundation-readiness-matrix.json.)

### C3. PRD-001 requirement intent (platform-foundation-prd.json)

Greenfield foundation enabling hyperscaler-grade cloud primitives + ecosystem services on one
tenant-aware platform *before implementation begins*. ~50 numbered requirements; notable families:
- Docs contract: REQ-DOC-001..003 (README entrypoint, HTML human docs, JSON SSOT, no private-ref leakage)
- Platform: REQ-PLAT-001 tenant = universal scope (Oyatie is a tenant, no internal bypass); REQ-PLAT-002 cloud primitives day-one; REQ-PLAT-003 Ecosystem-as-a-Service shares substrate
- Security/Fin/Ops: REQ-SEC-001, REQ-FIN-001, REQ-OPS-001, REQ-AUDIT-001
- First slice: REQ-SLICE-001, REQ-TEN-001, REQ-CAT-001, REQ-MOD-001 (KR enterprise workflow), REQ-IDCTX-001
- CI/CD & stack: REQ-CICD-001, REQ-HYGIENE-001, REQ-AUTOMATION-001, REQ-TOOLCHAIN-001 (oya), REQ-PROMOTION-001, REQ-CONTAINER-001, REQ-IAC-001, REQ-STACK-001
- Substrate reqs (one per architecture): REQ-POLICY-001, REQ-SVCARCH-001, REQ-CELL-001, REQ-TRAFFIC-001, REQ-RELIABILITY-001, REQ-STATE-001, REQ-CAPACITY-001, REQ-ARTIFACT-001, REQ-TRUST-001, REQ-NETWORK-001, REQ-COMPUTE-001, REQ-STORAGE-001, REQ-RESOURCE-API-001, REQ-EVENTING-001, REQ-WORKFLOW-001, REQ-METADATA-001, REQ-ORG-TENANT-001
- Lifecycle/process: REQ-FOUNDATION-ADR-ROADMAP-001, REQ-DELIVERY-MOBILIZATION-001, REQ-CAPABILITY-TRANSLATION-001, REQ-PRD-LIFECYCLE-001, REQ-TECHNICAL-SPEC-INDEX-001, REQ-CONTROL-EVIDENCE-MAPPING-001, REQ-IMPLEMENTATION-PLAN-001
- Closure: REQ-SOURCE-CLOSURE-001..012 (one per capability pack below)

Stack baseline (REQ-STACK-001): Rust 1.95.0 edition 2024, Node.js 24.16 LTS, OpenTofu 1.12,
Podman 5.8, Jenkins LTS, Argo CD 3.4.

Standards targeted: OpenAPI 3.2.0, AsyncAPI 3.1.0, CloudEvents-compatible event metadata.

### C4. Canonical 8-stage delivery artifact chain (capability-translation-brief.json / PRD-001)

ADR Portfolio → Capability Translation Brief → PRD → Technical SPEC → Control + Evidence Mapping →
Implementation Plan → Build/Verify/Certify → Launch/Operate/Improve. (First slice must exercise
identity, tenant model, control-plane lifecycle, audit/evidence, deployment path, CI/CD,
observability, security/policy gates.)

### C5. Open questions the legacy left unresolved

- OQ-002 Which first public B2C product to model as a tenant workload? (decision-group b2c-reference-product)
- OQ-003 Which first B2B industry module validates Ecosystem-as-a-Service? (industry-module-wedge)
- OQ-004 Which IdP + policy-engine tech implements the principal/context model? (identity-policy)
- POLICY-Q1/Q2/Q3 PDP p99/availability SLO; max bundle+graph staleness; analyzer/coverage gates before tenant overlays (policy-decision-plane.json)
- Trust residual risks: KMS/HSM/vault product selection deferred; SCIM provisioning unspecified; hardware-backed custody varies by sovereign region (trust-plane-architecture.json)

---

## (d) SOURCE-GAPS the legacy itself flagged

### D1. Source-gap matrix (source-gap-matrix.json) — 12 requirements, **0 fully closed at runtime**

Summary: total 12 · already-covered 0 · needs-capability-pack 10 · needs-stronger-detail 2 ·
needs-ADR 0 · needs-implementation-task 12 · rejected 0 · deferred 0 ·
fully_closed_runtime_requirements **0**. Every requirement is planning-covered only; implementation,
evidence, and launch lanes are **open**. Each maps SRC-REQ-00N → capability pack:

| Gap req | Capability pack | Disposition |
|---|---|---|
| SRC-REQ-001 | CAP-PACK-IDENTITY-TENANT-POLICY | needs capability pack |
| SRC-REQ-002 | CAP-PACK-RESOURCE-LIFECYCLE | needs capability pack |
| SRC-REQ-003 | CAP-PACK-AUDIT-EVIDENCE | needs capability pack |
| SRC-REQ-004 | CAP-PACK-CICD-SUPPLY-CHAIN | needs capability pack |
| SRC-REQ-005 | CAP-PACK-DEVELOPER-API | needs capability pack |
| SRC-REQ-006 | CAP-PACK-COMMERCIAL-FINOPS | needs capability pack |
| SRC-REQ-007 | CAP-PACK-OPERATIONS-READINESS | needs capability pack |
| SRC-REQ-008 | CAP-PACK-MARKETPLACE-PARTNER | needs capability pack |
| SRC-REQ-009 | CAP-PACK-SOCIAL-CONTENT | needs capability pack |
| SRC-REQ-010 | CAP-PACK-INDUSTRY-COMPLIANCE | needs capability pack |
| SRC-REQ-011 | CAP-PACK-FIRST-DELIVERY-GOVERNANCE | **needs stronger detail** |
| SRC-REQ-012 | CAP-PACK-CLEAN-ROOM-CLOSURE | **needs stronger detail** |

Typical `missing_before_closure` (per entry): implementation evidence package, runtime verifier
output, launch readiness decision, runtime service code, contract/integration tests, audit/evidence
export proof.

### D2. Source-requirement closure (source-requirement-closure.json) — 12 capability packs

status_counts: accepted 9 · adapted 3 · closed 0 · planning_closed_implementation_open 12 ·
fully_closed_runtime_requirements **0**. Closure model = 4 lanes (planning / implementation /
evidence / launch); only **planning** is closed. First implementation slice =
TASK-SOURCE-CLOSURE-001/002/003. Five upstream `SRC-FAM-001..005` source families were all
"adapted into Oyatie-native requirements" (clean-room, no copy/cite).

| Capability pack | Scope (one-line) | Source |
|---|---|---|
| CAP-PACK-IDENTITY-TENANT-POLICY | Identity, tenancy, active context, policy decision, tenant isolation, break-glass | source-requirement-closure.json |
| CAP-PACK-RESOURCE-LIFECYCLE | Resource envelope, lifecycle, LRO, idempotency, reconciliation, finalizers, versions, events, webhooks, SDK/OpenAPI | source-requirement-closure.json |
| CAP-PACK-AUDIT-EVIDENCE | Append-only audit/evidence events, export packages, retention, legal hold, redaction, verifier-friendly control evidence | source-requirement-closure.json |
| CAP-PACK-CICD-SUPPLY-CHAIN | Branch promotion, Jenkins/oya gates, provenance, SBOM, signing, registry/runtime admission, GitOps, rollback evidence | source-requirement-closure.json |
| CAP-PACK-DEVELOPER-API | API covenant, OpenAPI/AsyncAPI publication, SDK gen, compatibility policy, catalog, docs, deprecation | source-requirement-closure.json |
| CAP-PACK-COMMERCIAL-FINOPS | Entitlements, trials, subscriptions, metering, usage ledger, invoices, credits, allocation, showback/chargeback, FinOps exports | source-requirement-closure.json |
| CAP-PACK-OPERATIONS-READINESS | SLOs, error budgets, telemetry, dashboards, alerts, runbooks, incident response, postmortems, release readiness | source-requirement-closure.json |
| CAP-PACK-MARKETPLACE-PARTNER | Publisher onboarding, package review, entitlement settlement, revenue share, install lifecycle, suspension/withdrawal | source-requirement-closure.json |
| CAP-PACK-SOCIAL-CONTENT | Consumer identity, profiles, graph, feed/content moderation, notifications, creator controls, privacy, trust/safety | source-requirement-closure.json |
| CAP-PACK-INDUSTRY-COMPLIANCE | Regulated vertical control overlays, data classification, residency, retention, consent, evidence mappings, launch gates | source-requirement-closure.json |
| CAP-PACK-FIRST-DELIVERY-GOVERNANCE | Separation of planning/implementation/evidence/launch closure; non-MVP production-grade bars | source-requirement-closure.json |
| CAP-PACK-CLEAN-ROOM-CLOSURE | Opaque inventory, dispositions, public research evidence, idea refinement, leakage prevention, false-closure validators | source-requirement-closure.json |

---

## Cross-cutting recovery notes

- **Clean-room discipline everywhere**: every file carries `drafting_policy: clean-room-from-reference`
  and `source_material_policy: reference-only-no-copy-no-move-no-citation`. The legacy adapted (never
  copied) outside source families and refused to cite private reference paths.
- **Authority model**: README.md = human entrypoint; JSON = source of truth; HTML = rendering layer
  (consumes JSON via `data-doc-json`). 56 JSON files total under docs/data/ (this inventory covers the
  product/capability-relevant ones; remaining files are policies/standards/traceability/validators).
- **Benchmarked-against (named external pressure)**: Cedar, Zanzibar, OPA/Rego, FOCUS, OpenCost,
  SPIFFE/SVID, SLSA, Sigstore, NIST 800-57/207/218, OIDC, SAML 2.0, CloudEvents, OpenAPI 3.2,
  AsyncAPI 3.1; module benchmarked vs SAP Build Process Automation + ServiceNow App Engine + OWASP SAMM.
