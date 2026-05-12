# Oyatie — Product PRD: SaaS Multi-Tenant Platform

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Owning team:** [`teams/axis-saas/CHARTER.md`](../../teams/axis-saas/CHARTER.md)
> **Owning axis:** saas
> **Catalog reference:** `registry/catalog/oya-platform-*.yaml`, `registry/catalog/oya-saas-*.yaml`
> **Last updated:** 2026-05-09 by Architecture Council

---

## 1. North star (required)

The SaaS multi-tenant platform is **the substrate that lets every other axis ship a tenant-aware product**. It owns the canonical `Tenant`, `Workspace`, `Identity`, `RBAC`, `ObjectGraph`, `Workflow`, `Plugin`, and `Metering` kernels that every other axis (Foundry, Cloud, Search, Ads, Vertical) reads from but never re-implements. A standalone "Oyatie SaaS" sale is a real commercial product (workflow studio + object graph + plugin marketplace on Oyatie Cloud), but the *primary* job of this axis is **non-leakage**: the same `TenantId` that authorizes a workflow run also routes a cloud cell, gates a search index, refuses ad targeting, and binds a regulatory pack. Without this axis, the cohesion thesis collapses and Oyatie becomes seven disconnected products.

This is the *only* axis whose contract surface every other axis depends on by definition. It is the SaaS platform that exists *because* there is a Foundry, a Cloud, a Search, an Ads, and a Vertical industry cloud — and the SaaS platform exists to make those addressable as one product to one tenant under one consent surface.

## 2. Target users (required)

| Persona | What they get | What they pay for |
|---|---|---|
| **Tenant operator** (HR director, plant manager, clinic admin, regional finance lead) | A vertical-tuned workflow surface (Workflow Studio), Object Graph, capability marketplace, audit-chain trust portal, per-vertical regulatory pack pre-installed | Per-seat or per-volume SaaS subscription, plus per-capability metering for Foundry-driven runs |
| **Tenant builder / IT engineer** | Workflow Studio (typed JSON workflows per ADR-0035), Object Graph schema authoring (ADR-0006..0112), capability authoring against the published trait surface, plugin runtime (Wasmtime + WASI Preview 2 per ADR-0023), public REST API at frozen stability tier (ADR-0040) | Builder seats, plugin runtime usage, marketplace publishing fees |
| **End user / employee of tenant** | Day-to-day forms, tasks, notifications, search-driven retrieval inside their tenant context, mobile + web parity (ADR-0037) | (Indirect — paid by their tenant) |
| **ISV / Connect partner** | Plugin substrate (manifest per ADR-0036, signing per ADR-0039, sandbox per ADR-0023), marketplace listing, webhook delivery surface, public REST API, OG-AG agent gateway (ADR-0021) | Marketplace revenue share, plugin runtime metering |
| **Tenant Privacy Officer** | Consent surface, DSR cascade, audit-chain trust portal, regulatory-pack evidence dashboards (per [PRIVACY-PROGRAM.md §3.4](../../PRIVACY-PROGRAM.md)) | (Bundled with operator subscription) |
| **Internal Foundry agents** | Capability invocation against tenant Object Graph, RBAC-aware reads, autonomy-ceiling-bound writes, audit-chain emission for every step | (Internal — agent run cost is metered to tenant) |

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| **W-Foundation** | Tenant kernel (`oya-platform-tenant-kernel`), Identity kernel (Cedar + SSO + OIDC), RBAC kernel, Workspace kernel, Object Graph kernel (`oya-platform-object-graph-kernel`) with property-tier engine (ADR-0006..0112), Audit-chain kernel (ADR-0003), Eventing-backbone outbox (ADR-0046), Metering kernel (ADR-0007), Webhook signing kernel (ADR-0040), DSR cascade kernel | None public — kernels and contract definitions only |
| **W-Substrate** | Foundry-tenant binding, capability registry projection from `registry/catalog/`, claim-ceiling validator wired to tenant `autonomy_tier`, plane-gated CI lanes consuming tenant kernel, branch-protection-as-code (#1295), audit-chain emission contracts | Internal `Tenant Admin Console` v0 (CRUD on tenants, workspaces, identities) |
| **W-SaaS-Preview** | Workflow engine (state-machine + DAG hybrid per ADR-0035), Workflow definition versioning (ADR-0035), Workflow canonical JSON spec (ADR-0035), Object Graph property-tier runtime (Vector ADR-0006, Geo ADR-0006, TimeSeries ADR-0006, CipherText ADR-0043, Struct ADR-0006), tenant-side schema-evolution surface (ADR-0011), plugin substrate (manifest ADR-0036, trust tiers ADR-0036, signing ADR-0039, WASM sandbox ADR-0023), public REST API stability tier v1 (ADR-0040), webhook delivery + signing, plugin marketplace catalog (ADR-0034), Tenant Activation + Data Import (ADR-0002) | `Workflow Studio` web UI, `Object Graph` schema editor, `Plugin Marketplace`, `Public REST API v1`, `Webhook delivery`, `OG Agent Gateway` (ADR-0021) for Foundry, Tenant Admin Console v1 |
| **W-SaaS-Stable** | Public REST stability commitments under ADR-0040, marketplace ISV revenue share, advanced workflow (parallel branches, sub-workflows, retries with backoff), workflow jurisdiction overlay (ADR-0035), tenant organization administration + governance console (ADR-0002), enterprise audit retention surfaces, Connect dual-context comms (ADR-0044) integrated, mobile-tablet parity (ADR-0037) | All preview surfaces frozen at v1; new surfaces under v1.1 with deprecation discipline |
| **W-Public-GA** | SLA-bearing 99.95% control plane, 99.99% data plane on tenant-critical reads (Object Graph), plugin signing on Cosign keyless + Rekor (ADR-0039, ADR-0039), supply-chain attestation per release, Operational Intelligence layer (ADR-0006) | All surfaces SLA-backed; `enterprise-cloud-readiness` claim gate (ADR-0012) cleared |
| **W-Region-Fan-Out** | Per-regional-pack tenant onboarding flows, residency-aware workspace creation, locale-overlaid workflow definitions, locale-aware webhook delivery, per-pack identity-provider adapter wiring | Same surfaces, plus regional-pack-aware Tenant Admin Console |

### 3.2 Out-of-scope (anti-scope)

- Hosting tenant data outside Oyatie Cloud cells. (Tenant data resides on Oyatie Cloud or — in air-gapped regulated cases — on a customer-leased dedicated cell per ADR-0050; never on third-party hyperscaler primary tenancy.)
- A standalone "SaaS-only" sale that omits Foundry, Cloud, or audit-chain. (The cohesion thesis forbids this; partial sales must still ride the unified tenancy + audit chain.)
- Per-vertical clinical/manufacturing/financial business logic in the SaaS axis. That belongs to the vertical axis (`oya-vertical-*`); the SaaS axis only ships the *substrate* that the vertical axis writes against.
- Direct payment processing in the SaaS axis. Payment-rail adapters live in `oya-saas-billing-rail-kernel` with regional-pack impls; the SaaS axis does not own the merchant relationship for tenant-side payment flows.
- Building an alternative to the canonical eventing backbone. Outbox + Kafka per ADR-0046 is the only path.
- Forking the Object Graph property model into per-axis variants. ADR-0006..0112 are the only typed-property contract.
- Opening tenant data to ad targeting. The SaaS axis emits `BEHAVIORAL_TENANT_PRODUCT` (data-class 7 per [PRIVACY-PROGRAM.md §2.2.1](../../PRIVACY-PROGRAM.md)) to per-tenant analytics only; cross-tenant retargeting requires explicit consent uplift owned by the ads axis.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

The SaaS axis owns the **`platform` and `saas` bounded contexts** per [DESIGN.md §1](../../DESIGN.md). Crate prefixes:

- `crates/oya-platform-*` — cross-axis kernels (tenant, identity, RBAC, object graph, audit chain, eventing, metering, secrets, observability, web, crypto, DSR, regulatory, address)
- `crates/oya-saas-*` — SaaS-axis-specific (workflow, plugin, marketplace, billing-rail, webhook)

Per ADR-0015 §1, all crates follow `oya-<context>-<role>[-<capability>]` where `<role>` ∈ {`kernel`, `domain`, `app`, `api`, `worker`, `adapter`, `runtime`}.

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — DB, HTTP client, KMS, eventing impls
api       — inbound HTTP/gRPC servers
worker    — inbound queue/Kafka consumers
runtime   — composition root (binary)
```

| Crate | Role | One-line role |
|---|---|---|
| `oya-platform-tenant-kernel` | kernel | The single canonical `Tenant` aggregate root used by every axis |
| `oya-platform-tenant-domain` | domain | Tenant lifecycle use cases (provision, suspend, archive, residency-change) |
| `oya-platform-tenant-app` | app | Saga orchestrators (cross-axis tenant provisioning) |
| `oya-platform-tenant-adapter` | adapter | Postgres + Citus shards (ADR-0045), KMS (ADR-0043), Kafka outbox (ADR-0046) |
| `oya-platform-tenant-api` | api | HTTP control-plane API (axum), including authenticated `tenant.create`; gRPC inbound for Foundry |
| `oya-platform-tenant-worker` | worker | Outbox poller + Kafka consumer for cross-axis tenant events |
| `oya-platform-tenant-runtime` | runtime | Composition root binary |
| `oya-platform-identity-kernel` | kernel | Identity, principal, RBAC, Cedar policy fragments |
| `oya-platform-identity-domain` | domain | Authentication / authorization use cases; SSO orchestration |
| `oya-platform-identity-api` | api | HTTP control-plane API for authenticated `identity.user.upsert` with per-tenant primary identifier uniqueness and regional IdP binding |
| `oya-platform-identity-app` | app | Authenticated STS token issue boundary over the identity kernel |
| `oya-platform-policy-cedar-kernel` | kernel | Versioned Cedar policy set with semver publication, tenant/global scope, supersession metadata, and authorization evaluation |
| `oya-platform-policy-cedar-api` | api | Authenticated `cedar.policy.publish` control-plane API with idempotency and OpenAPI/runtime/schema parity |
| `oya-platform-identity-adapter` | adapter | OIDC, SAML, SCIM, Cedar evaluator, regional-pack IdP impls |
| `oya-platform-object-graph-kernel` | kernel | Typed-entity layer, property tiers (Vector/Geo/TimeSeries/CipherText/Struct) |
| `oya-platform-object-graph-api` | api | Authenticated `object-graph.entity.upsert` API with tenant row-isolation, property tier/data-class validation, idempotency, and mutation-event evidence |
| `oya-platform-object-graph-domain` | domain | OG read/write/query use cases, schema-evolution proposals |
| `oya-platform-object-graph-adapter` | adapter | Postgres+pgvector+PostGIS+TimescaleDB-style hypertables (or append-only) |
| `oya-platform-audit-chain-kernel` | kernel | Audit-chain event types, Merkle-sealed Ed25519 evidence (ADR-0028, ADR-0003) |
| `oya-platform-audit-chain-app` | app | Authenticated CloudEvents/Protobuf audit event emit boundary plus hash-chain assembly and outbox publication |
| `oya-platform-eventing-kernel` | kernel | Outbox event shape, Kafka topic naming convention (ADR-0046) |
| `oya-platform-eventing-app` | app | Authenticated CloudEvents/Protobuf outbox publish boundary for all axes |
| `oya-platform-eventing-adapter` | adapter | Kafka producer/consumer + Schema Registry binding |
| `oya-platform-metering-kernel` | kernel | Metering event shape, per-tenant per-capability cost model (ADR-0007) |
| `oya-platform-metering-app` | app | Authenticated CloudEvents/Protobuf metering ingest boundary plus outbox publication |
| `oya-platform-dsr-kernel` | kernel | DSR (export/delete/restrict) cascade primitives |
| `oya-platform-dsr-app` | app | DSR cascade saga across all data-touching axes, backed by `contracts/openapi/platform/platform-dsr-v1.yaml` |
| `oya-platform-webhook-kernel` | kernel | Webhook signing, replay-protection, delivery semantics |
| `oya-platform-regional-pack-kernel` | kernel | RegionalPack value object + per-pack control mapping and residency class validation seam |
| `oya-platform-regulatory-pack-api` | api | Authenticated `regulatory-pack.bind` control-plane API with regional pack validation, immutable tenant residency binding, multi-pack records, and idempotency |
| `oya-platform-secrets-app` | app | OpenBao binding (ADR-0043) |
| `oya-platform-crypto-kernel` | kernel | KMS envelope encryption primitives, CipherText type wiring (ADR-0043) |
| `oya-saas-workflow-kernel` | kernel | Workflow definition (state-machine + DAG hybrid per ADR-0035), step contract |
| `oya-saas-workflow-domain` | domain | Workflow execution use cases, retry/idempotency, jurisdiction overlay |
| `oya-saas-workflow-app` | app | Workflow saga orchestration, plugin sandbox dispatch |
| `oya-saas-workflow-adapter` | adapter | Durable execution port (PG-backed default; Temporal gated per ADR-0035) |
| `oya-saas-workflow-api` | api | Workflow Studio API, internal capability invocation surface |
| `oya-saas-workflow-sdk-kernel` | kernel | Tenant-builder SDK type contract |
| `oya-saas-plugin-kernel` | kernel | Plugin manifest types (ADR-0036), trust tiers (ADR-0036) |
| `oya-saas-plugin-domain` | domain | Plugin install / verify / sandbox lifecycle |
| `oya-saas-plugin-adapter` | adapter | Wasmtime sandbox (ADR-0023), Cosign+Rekor signing (ADR-0039) |
| `oya-saas-marketplace-kernel` | kernel | Listing, ISV, revenue-share, marketplace contract (ADR-0034) |
| `oya-saas-billing-rail-kernel` | kernel | PaymentRail trait + tenant-side billing cycles |
| `oya-saas-webhook-app` | app | Webhook delivery saga, retries, dead-letter |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Tenant Admin Console` | `apps/oyatie-tenant-admin/` (Leptos, ADR-0033) | control | p99 ≤ 500 ms control-plane mutation; 99.95% availability at GA |
| `Workflow Studio Web UI` | `apps/oyatie-workflow-studio/` | control | p95 ≤ 800 ms author-time read; 99.9% availability |
| `Workflow Engine API` | `contracts/workflow-engine.openapi.yaml` | data (executes tenant data) | p99 step latency ≤ 200 ms; 99.95% availability |
| `Object Graph Read API` | `contracts/object-graph-read.openapi.yaml` | data | p99 ≤ 50 ms point read; p99 ≤ 200 ms typed query |
| `Object Graph Agent Gateway (OG-AG)` | `contracts/og-agent-gateway.openapi.yaml` (ADR-0021) | data + audit | p99 ≤ 100 ms tool-call; every call audit-emits |
| `Public REST API v1` | `contracts/public-rest-v1.openapi.yaml` (ADR-0040) | control + data | 99.95% availability; deprecation horizon ≥ 12 months |
| `Webhook Delivery` | `contracts/webhook-delivery.json` | data (egress) | p95 first-attempt ≤ 2 s; 99% delivery-within-24-h |
| `Plugin Marketplace API` | `contracts/marketplace.openapi.yaml` | control | 99.9% availability |
| `Tenant Activation + Data Import` (ADR-0002) | `contracts/tenant-activation.openapi.yaml` | control | per-import SLO declared by tenant |
| `DSR Trust Portal` | `apps/oyatie-trust-portal/` + `contracts/openapi/platform/platform-dsr-v1.yaml` | control + audit | DSR ack ≤ 30 days (PIPA / GDPR Art 12) |
| `Connect dual-context messaging` | `contracts/connect-messaging.openapi.yaml` (ADR-0044, ADR-0008) | data | per-message p99 ≤ 250 ms |

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|
| Tenant aggregate | `Tenant` struct + `TenantRepo` trait in `oya-platform-tenant-kernel`; `create_tenant_from_api` in `oya-platform-tenant-api` | All other axes (foundry, cloud, search, ads, vertical, foundry) |
| Identity / RBAC | `Principal`, `RoleBinding`, `CedarEvaluator`, `User` in `oya-platform-identity-kernel`; `PolicySet` in `oya-platform-policy-cedar-kernel`; `upsert_identity_user_from_api` in `oya-platform-identity-api`; `issue_identity_token_from_app` in `oya-platform-identity-app`; `publish_cedar_policy_from_api` in `oya-platform-policy-cedar-api` | All axes |
| Object Graph | `ObjectEntity`, `ObjectProperty`, and property tiers in `oya-platform-object-graph-kernel`; `upsert_object_graph_entity_from_api` in `oya-platform-object-graph-api` | Search (indexable), Ads (targetable per consent), Vertical (regulatory), Foundry (RAG ground) |
| Audit chain emit | `emit_audit_event_from_app` in `oya-platform-audit-chain-app` over `AuditChain::append_classifications` in `oya-platform-audit-chain-kernel` | Every axis that touches regulated data |
| Eventing outbox | `publish_eventing_outbox_from_app` in `oya-platform-eventing-app` over `Outbox::publish` in `oya-platform-eventing-kernel` | All axes |
| Metering | `ingest_metering_event_from_app` in `oya-platform-metering-app` over `Meter::record()` in `oya-platform-metering-kernel` | Cloud billing, marketplace, foundry capability cost |
| DSR cascade | `DsrCascadeAck` in `oya-platform-dsr-kernel`; `execute_dsr_cascade_from_api` in `oya-platform-dsr-app` | All data-touching axes (mandatory ack) |
| Webhook signer | `WebhookSigner::sign(payload)` in `oya-platform-webhook-kernel` | Cloud, Search, Ads (egress) |
| Regulatory pack | `RegionalPack` in `oya-platform-regional-pack-kernel`; `TenantResidencyRegistry` in `oya-platform-residency-kernel`; `bind_regulatory_pack_from_api` in `oya-platform-regulatory-pack-api` | Vertical, Cloud (region binding), Privacy program |
| Workflow engine | `WorkflowDef`, `Step`, `Run` in `oya-saas-workflow-kernel` | Vertical (writes workflows), Foundry (executes steps) |
| Plugin sandbox | `PluginContext` in `oya-saas-plugin-kernel` | Vertical (per-vertical plugins), Marketplace |
| Marketplace listing | `Listing` + `RevenueShare` in `oya-saas-marketplace-kernel` | Foundry catalog, ads (paid placement), all axes (capability publishing) |

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| Capability invocation | Foundry | `contracts/foundry-capability.openapi.yaml` | Cross-axis (foundry + saas) |
| Autonomy ceiling policy | Foundry + Governance (ADR-0050) | `oya-foundry-policy-kernel` | Governance + security |
| Cloud Region / AZ / Cell | Cloud | `oya-cloud-region-kernel` | Multi-axis (residency-impact) |
| Cloud IAM / SSO IdP | Cloud + SaaS | `oya-cloud-iam-kernel` ↔ `oya-platform-identity-kernel` | Two ADRs in lockstep |
| Cloud Billing event | Cloud | `oya-cloud-billing-kernel` ↔ `oya-platform-metering-kernel` | Billing + tax review |
| Search index lifecycle | Search | `oya-search-index-kernel` | Search + Object Graph review |
| Ad slot inventory (in-app) | Ads | `oya-ads-slot-kernel` | Ads + surface-owner |
| Foundry catalog record | Foundry | `registry/catalog/<crate>.yaml` | Catalog gate |
| Vertical regulatory pack | Vertical | `oya-vertical-<x>-kernel` | Vertical + regulatory review |

(Mirror in [DESIGN.md §10](../../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-platform-*-kernel`, `crates/oya-saas-*-kernel`)

```rust
// oya-platform-tenant-kernel
pub struct Tenant {
    pub id: TenantId,                                  // ulid
    pub display_name: TenantName,                      // data_class: PUBLIC
    pub region: RegionCode,                            // data_class: PUBLIC, plane: control
    pub residency: ResidencyClass,                     // strict_kr | eea | us | jp | global
    pub regulatory_packs: Vec<RegulatoryPackId>,       // PIPA, HIPAA, MFDS, FSC, GDPR, ...
    pub vertical: VerticalKind,                        // healthcare | fintech | industrial | ...
    pub data_use_consent: DataUseConsent,              // per-class consent ladder
    pub autonomy_tier: AutonomyTier,                   // T0..T5 per ADR-0022
    pub billing_account: BillingAccountId,             // FK to cloud billing
    pub plugin_trust_tier: PluginTrustTier,            // per ADR-0036
    pub state: TenantState,                            // provisioned | active | suspended | archived
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: see field annotations; aggregate-level data_class = MIXED-CONTROL

pub struct DataUseConsent {
    pub search_indexable_classes: BTreeSet<DataClass>,   // see PRIVACY-PROGRAM §2.2.1
    pub ad_targeting_classes: BTreeSet<DataClass>,       // never includes PHI / PII / PIPA-Art-23
    pub analytics_classes: BTreeSet<DataClass>,
    pub cross_tenant_aggregate_opt_in: bool,
    pub k_anonymity_floor: u16,                          // ≥ 10 per PRIVACY-PROGRAM §2.2.2
    pub consent_receipts: Vec<ConsentReceiptRef>,        // chain link to audit-chain
}
```

```rust
// oya-platform-identity-kernel
pub struct Principal {
    pub id: PrincipalId,                       // ulid
    pub tenant_id: TenantId,                   // every record carries tenant
    pub kind: PrincipalKind,                   // user | service | agent | external_partner
    pub subject_uri: SubjectUri,               // IdP-issued sub
    pub display_name: String,                  // data_class: PII_QUASI
    pub email: Option<EmailAddress>,           // data_class: PII_IDENTIFYING
    pub locale: LocaleTag,                     // BCP-47
    pub region_pack: RegionalPackId,           // for IdP routing
    pub mfa_state: MfaState,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: control
// data_class: MIXED — display_name=PII_QUASI, email=PII_IDENTIFYING; aggregate handled per Privacy Program

pub struct RoleBinding {
    pub id: RoleBindingId,
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub role: RoleId,
    pub scope: ScopeRef,                       // workspace | tenant | object | property
    pub cedar_policy_id: CedarPolicyId,        // links to Cedar evaluator
    pub granted_by: PrincipalId,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC (audit metadata)
```

```rust
// oya-platform-object-graph-kernel
pub struct Entity {
    pub id: EntityId,                           // ulid
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub schema_id: ObjectSchemaId,
    pub schema_version: u32,
    pub properties: BTreeMap<PropertyKey, PropertyValue>,  // tier-classified per ADR-0008
    pub data_class_overlay: BTreeMap<PropertyKey, DataClass>, // per-record override
    pub region: RegionCode,                     // for cell-routing
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: PrincipalId,
    pub edit_history_root: Option<MerkleHash>,  // OG audit chain
}
// plane: data
// data_class: MIXED — derived from property tier + per-record overlay

pub enum PropertyValue {
    Scalar(ScalarValue),
    Vector(VectorValue),                        // ADR-0006, dim ≤ 4096
    Geo(GeoValue),                              // ADR-0006, GeoJSON or PostGIS WKT
    TimeSeries(TimeSeriesRef),                  // ADR-0006, hypertable or append-only
    CipherText(CipherTextRef),                  // ADR-0043, KMS envelope
    Struct(StructValue),                        // ADR-0006, schemars-validated JSON
    Reference(EntityRef),                       // graph edge
}

pub struct ObjectSchema {
    pub id: ObjectSchemaId,
    pub tenant_id: TenantId,
    pub kind: ObjectKind,                       // person | document | task | order | ...
    pub property_defs: Vec<PropertyDef>,        // {key, type, tier, data_class, indexable}
    pub vertical_overlay: Option<VerticalKind>, // FHIR | manufacturing | EDI | ...
    pub revision: u32,                          // monotonic per schema-evolution proposal
    pub schema_version: u32,
}
// plane: control (schema is metadata; instance data lives in Entity)
```

```rust
// oya-saas-workflow-kernel
pub struct WorkflowDef {
    pub id: WorkflowDefId,                          // ulid
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub name: WorkflowName,
    pub canonical_spec: CanonicalWorkflowJson,      // ADR-0035
    pub jurisdiction_overlay: Vec<JurisdictionRef>, // ADR-0035
    pub versions: Vec<WorkflowVersion>,
    pub plane: PlaneTag,                            // control | data | analytics
    pub autonomy_tier_required: AutonomyTier,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control (definition); execution emits to data plane

pub struct Run {
    pub id: RunId,                                  // ulid
    pub tenant_id: TenantId,
    pub workflow_def_id: WorkflowDefId,
    pub version: u32,
    pub initiator: PrincipalId,
    pub state: RunState,                            // pending | running | completed | failed | cancelled
    pub steps: Vec<StepExecution>,
    pub data_classes_touched: BTreeSet<DataClass>,
    pub audit_chain_root: Option<MerkleHash>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: data
// data_class: MIXED — per-step references annotate the touched classes
```

```rust
// oya-saas-plugin-kernel
pub struct Plugin {
    pub id: PluginId,
    pub publisher_id: TenantId,                       // ISV tenant
    pub manifest: PluginManifest,                     // ADR-0036
    pub trust_tier: PluginTrustTier,                  // ADR-0036
    pub signature: CosignSignature,                   // ADR-0039 (Cosign+Rekor)
    pub sandbox_caps: BTreeSet<CapabilityId>,         // PluginContext capabilities
    pub wasm_artifact: WasmArtifactRef,               // OCI artifact ref, ADR-0044 Harbor
    pub semver: SemanticVersion,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC

pub struct Listing {                                  // marketplace
    pub id: ListingId,
    pub plugin_id: PluginId,
    pub publisher_id: TenantId,
    pub display_name: String,
    pub category: MarketplaceCategory,
    pub revenue_share_bps: u16,                       // basis points to publisher
    pub regional_packs_supported: Vec<RegionalPackId>,
    pub state: ListingState,                          // draft | review | published | delisted
    pub created_at: DateTime<Utc>,
}
```

```rust
// oya-platform-metering-kernel
pub struct MeterEvent {
    pub id: MeterEventId,
    pub tenant_id: TenantId,
    pub capability_id: CapabilityId,
    pub plane: PlaneTag,
    pub units: MeterUnits,                            // {request, byte_in, byte_out, ms, gpu_sec, llm_token}
    pub source_axis: AxisId,                          // saas | foundry | cloud | search | ads | vertical
    pub recorded_at: DateTime<Utc>,
    pub idempotency_key: Uuid,
    pub data_class: DataClass,                        // event metadata; PUBLIC by default
    pub schema_version: u32,
}
// plane: analytics
```

### 5.2 Aggregate boundaries

- **Tenant aggregate**: `Tenant` is the consistency boundary; `DataUseConsent`, `RegulatoryPackId[]`, `BillingAccountId` change as one unit.
- **Identity aggregate**: `Principal` + `RoleBinding[]` for the principal's scope; Cedar policies are evaluated on read.
- **Object aggregate**: `Entity` + its property values + edit-history root. ObjectSchema is a separate aggregate (slow-changing).
- **Workflow aggregate**: `WorkflowDef` (versioned) is one aggregate; `Run` is a separate aggregate referencing it.
- **Plugin aggregate**: `Plugin` + its `Listing` cluster only at marketplace boundaries; otherwise separate.
- **Audit-chain block**: append-only; aggregate per chain segment with Merkle root anchored externally.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Tenant | Postgres + Citus (ADR-0045) | `tenant_id` | per-tenant shard | streaming replication 3-AZ | indefinite (until DSR-archive) |
| Principal / RoleBinding | Postgres + Citus | `tenant_id` | per-tenant | 3-AZ | indefinite |
| Entity (ObjectGraph) | Postgres + pgvector + PostGIS | `(tenant_id, schema_id)` | per-tenant + per-schema sub-partition | 3-AZ; cross-region only if `residency` allows | per-class retention (PII purge ≤ 5y default) |
| ObjectSchema | Postgres | `tenant_id` | per-tenant | 3-AZ | indefinite |
| WorkflowDef | Postgres | `tenant_id` | per-tenant | 3-AZ | indefinite |
| Run | Postgres (recent) + ClickHouse archive (ADR-0045) | `tenant_id` + time | per-tenant + monthly time | 3-AZ + cold to Iceberg per ADR-0045 | 7y default; per-class purge cascade |
| Plugin / Listing | Postgres | `publisher_id` | per-publisher | 3-AZ | indefinite |
| MeterEvent | ClickHouse (ADR-0045) | `tenant_id` + time | per-tenant per-day | 3-AZ + cold to Iceberg | 7y |
| Audit-chain block | Postgres + S3-class object store (Merkle root anchor) | tenant + time | per-tenant per-day | 3-AZ + cross-region | indefinite (immutable) |

### 5.4 Event schemas (events emitted)

All events go through the canonical eventing backbone per ADR-0005/ADR-0046 with CloudEvents 1.0 envelopes, Protobuf payloads, AsyncAPI schemas, and outbox pattern.

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `eventing.outbox.publish.v1` | `oya.platform.eventing` | `contracts/asyncapi/platform/eventing-outbox-v1.yaml`; `contracts/proto/platform/eventing/v1/eventing-outbox-v1.proto` | All axes via platform eventing worker/adapter | 90 days | `(tenant_id, target_topic, idempotency_key)` |
| `tenant.provisioned.v1` | `oya.platform.tenant` | `contracts/events/tenant.provisioned.v1.avsc` | Cloud (account create), Foundry (capability bind), Search (tenant index init), Ads (consent bind) | 90 days | `tenant_id` |
| `tenant.consent_changed.v1` | `oya.platform.tenant` | `contracts/events/tenant.consent_changed.v1.avsc` | Search (re-evaluate index), Ads (re-evaluate audience), Audit (chain link) | 90 days | `(tenant_id, change_seq)` |
| `tenant.dsr_requested.v1` | `oya.platform.dsr` | `contracts/events/tenant.dsr_requested.v1.avsc` | Search (index purge), Ads (audience purge), Vertical (regulator notification), Cloud (cell-level purge), Audit (cascade chain) | 7y (compliance) | `dsr_request_id` |
| `objectgraph.entity_changed.v1` | `oya.platform.objectgraph` | `contracts/events/objectgraph.entity_changed.v1.avsc` | Search (re-index), Foundry (RAG re-embed), Analytics (property-tier change-data-capture) | 30 days | `(tenant_id, entity_id, version)` |
| `objectgraph.schema_evolved.v1` | `oya.platform.objectgraph` | `contracts/events/objectgraph.schema_evolved.v1.avsc` | Search (mapping update), Vertical (overlay verify), Foundry (capability re-bind) | indefinite | `(tenant_id, schema_id, revision)` |
| `workflow.run_completed.v1` | `oya.saas.workflow` | `contracts/events/workflow.run_completed.v1.avsc` | Metering (per-step cost), Analytics, Foundry (evidence chain), Audit | 90 days | `run_id` |
| `workflow.step_emitted.v1` | `oya.saas.workflow` | `contracts/events/workflow.step_emitted.v1.avsc` | Audit (per-step record), Foundry (capability invocation echo), Metering | 30 days | `(run_id, step_seq)` |
| `plugin.installed.v1` | `oya.saas.plugin` | `contracts/events/plugin.installed.v1.avsc` | Catalog (capability bind), Foundry (registry projection), Audit | indefinite | `(tenant_id, plugin_id, semver)` |
| `marketplace.listing_published.v1` | `oya.saas.marketplace` | `contracts/events/marketplace.listing_published.v1.avsc` | Catalog, Search (listing index), Ads (sponsored-listing eligibility) | indefinite | `listing_id` |
| `webhook.delivered.v1` | `oya.platform.webhook` | `contracts/events/webhook.delivered.v1.avsc` | Audit, Tenant Trust Portal | 30 days | `(delivery_id, attempt_seq)` |
| `metering.event.ingest.v1` | `oya.platform.metering` | `contracts/asyncapi/platform/metering-events-v1.yaml`; `contracts/proto/platform/metering/v1/metering-event-v1.proto` | Cloud billing, FinOps surface, Marketplace revenue share | 7y | `meter_event_id` |

### 5.5 Index / search-index touchpoints

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|
| `Entity.properties[k]` where `property_def.indexable = true` | `oya-search-tenant-private` | `BEHAVIORAL_TENANT_PRODUCT` (7), `DECLARED_PREFERENCE` (9), `PUBLIC` (1) | Yes |
| `Entity.properties[k]` where vector tier | `oya-search-vector-tenant-private` (pgvector → Milvus per ADR-0047) | as above | Yes |
| `Plugin` listing fields | `oya-search-marketplace-public` | `PUBLIC` only | Yes (delisting cascades) |
| `WorkflowDef.name` (when published as template) | `oya-search-workflow-templates-public` | `PUBLIC` only | Yes |
| Tenant-public profile (opt-in only) | `oya-search-cross-tenant-aggregate` | `CROSS_TENANT_AGGREGATE` consent tier | Yes (k-anonymity ≥ 10 enforced) |

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../../DESIGN.md) + ADR-0003, every regulated capability must emit.

| Operation | Emits topic | Required fields |
|---|---|---|
| Tenant provisioned | `oya.audit.tenant_provisioned` | `tenant_id`, `region`, `residency`, `regulatory_packs`, `actor`, `timestamp`, `prev_hash` |
| Consent changed | `oya.audit.consent_changed` | `tenant_id`, `before`, `after`, `actor`, `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Object Graph write (regulated property) | `oya.audit.og_write` | `tenant_id`, `entity_id`, `property_keys`, `data_classes`, `actor`, `timestamp`, `prev_hash` |
| Workflow run | `oya.audit.workflow_run` | `tenant_id`, `run_id`, `data_classes_touched`, `autonomy_tier`, `actor`, `started_at`, `completed_at`, `prev_hash` |
| Plugin install | `oya.audit.plugin_install` | `tenant_id`, `plugin_id`, `semver`, `signature_ref`, `trust_tier`, `actor`, `timestamp`, `prev_hash` |
| DSR cascade | `oya.audit.dsr_cascade` | `tenant_id`, `dsr_request_id`, `cascade_acks[]`, `proof_of_erasure_root`, `timestamp`, `prev_hash` |
| Webhook signed delivery | `oya.audit.webhook_delivery` | `tenant_id`, `delivery_id`, `recipient`, `signature`, `timestamp`, `prev_hash` |
| Cross-axis data flow | `oya.audit.crossaxis_flow` | `tenant_id`, `src_axis`, `dst_axis`, `data_classes`, `purpose`, `consent_receipt_ref`, `timestamp`, `prev_hash` |

### 5.7 Schema migration policy

- **Versioning**: `schema_version: u32` is monotonic per kernel entity. Reads must accept v ≤ current; writes must emit at current.
- **Reversibility**: every migration ships with up + down DDL; the down half is required to land in the same PR.
- **Dry-run gate**: Foundry fitness function `oya-foundry-fitness-migration` runs every migration against a synthetic 100k-row tenant before merge.
- **ObjectGraph schema-evolution** (ADR-0011): tenant-builder proposes a schema change; evolved schema bumps `revision`; live entities are dual-written across the boundary until promoted.

## 6. Optimization practices (required) — *slice-level*

| Practice | Implementation choice |
|---|---|
| Cell routing | `Tenant.region` chooses cloud cell; tenant queries route via Envoy header `x-oya-tenant` → cell-local Postgres shard |
| Sharding strategy | Citus per-tenant for OLTP (`tenant_id` shard key); ClickHouse per-tenant per-day for analytics; pgvector per-tenant per-schema for vector |
| Caching tier | In-memory (moka) for hot Tenant + Cedar policy + ObjectSchema; Redis for session + RBAC short-circuits; CDN for marketplace listing assets |
| Bulk endpoint contract | `BatchCreateEntities`, `BatchUpdateProperties`, `BulkExport` (cursor-paged, streamed); max batch 10 000 rows |
| Pagination | Cursor-based (`(updated_at, id)` opaque token); default page 100, max 10 000; filter contract via JSON-DSL with allow-list |
| Idempotency | `Idempotency-Key` header on every mutating REST + gRPC call; outbox dedupes 24 h |
| Batch dispatch | Workflow step submission batches every 10 ms or 256 events; webhook delivery batches per-recipient every 50 ms or 64 events |
| Backpressure | Workflow worker reads from Kafka with consumer-group rebalance; outbox shedder drops to dead-letter at 95% lag; 429 responses with `Retry-After` to public REST callers |
| Hot-path benchmarks | Object Graph point read (`p99 ≤ 50 ms`), Cedar evaluation (`p99 ≤ 5 ms`), Tenant resolve from header (`p99 ≤ 1 ms`) — all wired to `oya-foundry-fitness-bench` |
| Agent-driven optimization loops | Foundry capability `saas.workflow.tune` (autonomy ≤ T2): proposes step concurrency + retry policy from past `workflow.run_completed.v1` analytics; human approves before promotion |
| FinOps unit-economics | Per-tenant cost = sum(`MeterEvent.units` × cell-rate-card); per-call cost surfaced in Tenant Admin Console; FinOps target ratio (cost-of-revenue per tenant) ≤ 30% at GA |
| Build-cache and CI affected-graph | `oya-platform-*`, `oya-saas-*` are in the largest affected subgraph; ADR-0015 flat-crate boundaries keep change-radius bounded; `cargo build --workspace` runs only changed crates via Bazel-style affected analysis |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Identity provider adapter | `IdentityProvider` in `oya-platform-identity-kernel` | yes | KR (본인확인서비스, Kakao, Naver), JP (マイナンバー), US (Login.gov), EU (eIDAS), IN (Aadhaar), BR (gov.br), KSA (Absher), UAE (UAE-PASS), ANZ (myGovID) — all per ADR `regional-pack` initial roster |
| Regulator → control mapping | `RegionalPack` in `oya-platform-regional-pack-kernel`; `bind_regulatory_pack_from_api` in `oya-platform-regulatory-pack-api` | yes | KR (PIPA/KISA/MFDS/FSC/CSAP/K-ISMS-P/KCMVP), JP (APPI/ISMAP), US (HIPAA/CCPA/SOX/FedRAMP), EU (GDPR/DORA), IN (DPDP/RBI), BR (LGPD), KSA (PDPL/NDMO/SDAIA), UAE (TDRA/ADGM), ANZ (Privacy Act/IRAP) |
| Tax-invoice formatter | `TaxInvoiceFormatter` in `oya-platform-billing-tax-kernel` | yes | KR 전자세금계산서, JP 適格請求書, EU per-country e-invoicing, IN GST, BR NF-e, KSA FATOORA |
| Address validator | `AddressValidator` in `oya-platform-address-kernel` | yes | KR (도로명주소), JP (郵便番号), US (USPS), EU (per-country), IN (PIN), BR (CEP) |
| Payment rail (tenant-side) | `PaymentRail` in `oya-saas-billing-rail-kernel` | yes | KR (Toss/Kakao Pay/계좌이체), JP (口座振替), US (ACH/Wire), EU (SEPA), IN (UPI), BR (Pix), KSA (SADAD/Mada) |
| Locale + copy | `LocaleBundle` in `oya-platform-web-kernel` | yes | every pack |
| Holiday calendar (workflow scheduling, jurisdiction overlay per ADR-0035) | `HolidayCalendar` in `oya-saas-workflow-domain` | yes | every pack |
| Document templating (workflow output) | `DocumentTemplate` in `oya-saas-workflow-domain` | yes | every pack with localized templates |

## 8. In-house vs external dependency posture (required)

Per the in-house build preference (PRD §3.1 §6 constraint):

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `axum` (HTTP) | kernel-grade | MIT/Apache-2 | no | adopt — kernel-grade, no ADR needed |
| `tokio` (async) | kernel-grade | MIT | no | adopt |
| `serde` / `serde_json` | kernel-grade | MIT/Apache-2 | no | adopt |
| `rustls` (TLS) | kernel-grade | MIT/Apache-2/ISC | no | adopt |
| `tonic` (gRPC) | kernel-grade | MIT | no | adopt |
| `sqlx` / Postgres driver | kernel-grade | MIT/Apache-2 | no | adopt |
| `pgvector` | secondary | PostgreSQL License | considered own KNN store; pgvector won (ADR-0050 → ADR-0047 keeps pgvector as day-1) | adopt with ADR-0050/0177 |
| `Citus` (Postgres extension) | secondary | AGPL-3 *(distribution; not linked into Rust binary)* | TiDB / Vitess (gated per ADR-0045) | **adopt as managed extension only** — AGPL boundary respected because Citus runs in Postgres binary, not linked into Rust crate code; ADR-0045 governs |
| `Wasmtime` | secondary | Apache-2 | own WASM runtime — rejected (kernel-grade Wasmtime is canonical) | adopt (ADR-0023) |
| `Cosign` / `Rekor` | secondary | Apache-2 | own signing — rejected | adopt (ADR-0039, ADR-0039) |
| `Cedar` (AWS) | secondary | Apache-2 | OPA / own policy engine — Cedar wins on auditability | adopt with ADR — Cedar is the SaaS RBAC engine |
| `OpenBao` (secrets) | secondary | MPL-2 | own secret store — rejected | adopt (ADR-0043, supersedes Vault) |
| `Wasmtime + WASI Preview 2` | secondary | Apache-2 | already covered | adopt |
| `Apache Kafka` (eventing) | secondary | Apache-2 | own outbox — built as day-1 substitute; Kafka is end-state | adopt gated (ADR-0046) |
| `pgroonga` | secondary | LGPL-2.1 *(extension; not linked)* | full-text in Postgres — pgroonga supplies KR morphology | adopt as Postgres extension only (boundary respected; ADR-0047) |
| `mecab-ko / khaiii` | secondary | BSD / Apache-2 | own KR tokenizer — rejected | adopt for regional-pack-kr search |
| `OpenTelemetry` | kernel-grade | Apache-2 | no | adopt |

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review. AGPL extensions (Citus) are allowed only when running as a separate process/binary boundary, never linked into product code; the boundary is enforced by `oya-foundry-fitness-license` (ADR-0039 supply chain).

## 9. Success metrics (required)

| Metric | W-SaaS-Preview target | W-SaaS-Stable target | W-Public-GA target |
|---|---|---|---|
| Tenants provisioned end-to-end | ≥ 25 internal pilots | ≥ 250 paying tenants | ≥ 2 500 paying tenants |
| Cross-axis contract violations on `main` | 0 detected per quarter | 0 | 0 |
| Audit-chain emission completeness | ≥ 99% on regulated capabilities | 100% | 100% |
| Object Graph point-read p99 | ≤ 100 ms | ≤ 75 ms | ≤ 50 ms |
| Public REST API availability | n/a (preview) | 99.9% | 99.95% |
| Workflow Studio author-time p95 | ≤ 1 500 ms | ≤ 1 000 ms | ≤ 800 ms |
| Plugin install round-trip | ≤ 60 s | ≤ 30 s | ≤ 15 s |
| DSR cascade ack ≤ 30 days | ≥ 95% | ≥ 99% | 100% |
| Marketplace listings published | ≥ 50 | ≥ 1 000 | ≥ 10 000 |
| Foundation-bypass count (per quarter) | not increasing | decreasing | 0 |

Plus structural metrics: cross-axis-contract-violation count = 0; audit-chain emission completeness = 100%; foundation-bypass count not increasing.

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Tenant kernel evolves under load and other axes diverge | High | Cross-axis review label is mandatory on any `oya-platform-tenant-kernel` change; CI fitness `oya-foundry-fitness-contracts` blocks orphan consumer drift | SaaS-platform team + Architecture council |
| Object Graph property-tier engine slows hot-path reads | High | Hot-path benchmark gate; tier-classified columnar layout; pgvector + PostGIS as Postgres extensions co-located with primary | SaaS-platform team |
| Plugin sandbox escape via Wasmtime CVE | Catastrophic | WASI Preview 2 capability allow-list; per-plugin signature (Cosign+Rekor); plugin trust tier gates network/FS access; Trivy 4-layer scanning per ADR-0039 | SaaS-platform + Security |
| AGPL leak from Citus / pgroonga into product code | High | License-policy gate (`oya-foundry-fitness-license`) hard-fails non-extension linkage; CI affected-graph isolates extension boundary | Foundry + SaaS |
| Workflow engine scaling beyond PG-backed scheduler | Medium | Hexagonal port keeps Temporal (ADR-0035) as drop-in; switch when p99 step latency floor cannot be held | SaaS-platform |
| Cedar policy explosion (per-property RBAC) | Medium | Versioned policy publish gate in `oya-platform-policy-cedar-api` over `oya-platform-policy-cedar-kernel`; per-tenant policy size budget; lint at policy-author time | SaaS-platform + Security |
| ObjectGraph schema evolution (ADR-0011) introduces dual-write divergence | Medium | Promotion gate requires verified equivalence on synthetic + sampled real data; rollback by `revision` decrement | SaaS-platform |
| Tenant-data exfiltration via webhook | High | Webhook signing (HMAC + replay window 5 min); per-tenant egress budget; audit-chain on every delivery | SaaS-platform + Security |
| DSR cascade failures on offline-archive tier | High | Iceberg cold-tier (ADR-0045) cascade is async-proof-of-erasure; root anchor published to trust portal | SaaS-platform + Privacy |
| Marketplace plugin abuses sandbox cap | Medium | Per-plugin `sandbox_caps` allow-list; runtime cap-enforcement instrumentation; recall path via marketplace state machine | SaaS-platform + Security |

## 11. Open questions

1. **Plugin marketplace revenue-share defaults**: ISV gets 70% / 80% / 90%? Council to set; currently parameterized as per-listing field.
2. **Object Graph cross-tenant collaboration model**: shared workspace, federated entity, or copy-on-share? Default proposed: copy-on-share (cleanest privacy boundary; loses real-time collaboration).
3. **Public REST API stability tier ladder**: how many concurrent stability tiers does ADR-0040 commit to (`v1` only vs `v1 + v2 in beta`)? Council decision pending.
4. **Workflow Studio low-code vs typed JSON authoring**: do tenant-IT users write canonical JSON directly, or only via Studio? Default proposed: Studio for authoring, JSON for export+round-trip.
5. **Connect dual-context messaging (ADR-0044) integration boundary**: full mux of personal+professional in the same client, or split clients with shared identity? Council has draft, not yet ratified.

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | SaaS axis is the substrate; cohesion thesis | One canonical Tenant + Identity + ObjectGraph; every other axis consumes |
| 2026-05-09 | Citus + pgvector + pgroonga as managed extensions only | Respects AGPL/LGPL boundary; ADR-0045, ADR-0047, ADR-0047 govern |
| 2026-05-09 | Cedar as RBAC engine | Auditability + AWS open-source maturity; ADR pending |
| 2026-05-09 | Outbox-first; Kafka gated | Day-1 PG-backed outbox per ADR-0046; Kafka end-state |

## 13. Sources scanned

- [`docs/PRD.md`](../../PRD.md)
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §5, §10, §12
- [`docs/PRIVACY-PROGRAM.md`](../../PRIVACY-PROGRAM.md)
- [`docs/GLOSSARY.md`](../../GLOSSARY.md) §1-7, §8
- ADR-0018 (Tenancy + RLS), ADR-0028 (Audit-chain Merkle Ed25519), ADR-0006..0112 (Object Graph property tiers), ADR-0021 (OG-AG), ADR-0011 (Schema evolution), ADR-0002 (Tenant Activation + Data Import), ADR-0006 (Cross-product cookie + redirect), ADR-0017 (Domain naming canon), ADR-0022 (Persona tier), ADR-0008 (Data ownership pillars), ADR-0008 (Tier-classified OG properties), ADR-0034 (Marketplace operating model), ADR-0007 (Tenant-configurable optimization + ML), ADR-0035 (Workflow engine model), ADR-0035 (Workflow definition versioning + jurisdiction overlay), ADR-0014 (Rust-first sovereignty), ADR-0034 (Form schema standard), ADR-0036 (Plugin manifest), ADR-0036 (Plugin trust tiers), ADR-0039 (Plugin signing — Cosign+Rekor), ADR-0023 (WASM sandbox — Wasmtime + WASI Preview 2), ADR-0035 (Workflow canonical spec format), ADR-0013 (Envoy gateway), ADR-0044 (Harbor), ADR-0043 (OpenBao), ADR-0046 (Kafka eventing), ADR-0045 (Citus), ADR-0047 (Vector store), ADR-0047 (Search backend), ADR-0050 (Argo Rollouts), ADR-0039 (Supply chain Trivy/Cosign/SBOM), ADR-0002 (Tenant org admin console), ADR-0015 (Repo structure), ADR-0044 (Connect), ADR-0033 (Leptos client), ADR-0008 (Connect retention), ADR-0015 (Flat crates), ADR-0003 (Trust framework), ADR-0021 (Product control plane), ADR-0001 (Ecosystem integration), ADR-0050 (Data + AI governance), ADR-0040 (Evolution + Simplification plane), ADR-0017 (Roadmap wave integration framework)

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `saas-platform` | `axis-saas` | scope, contract, capability | monthly | PRD.md, DESIGN.md, PRIVACY-PROGRAM.md, GLOSSARY.md |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` — add `saas-platform` row
- `machine-readable/catalog.json` — pointer at this PRD path
- `machine-readable/contracts.json` — every cross-axis contract row in §4.5
- `machine-readable/risks.json` — risks from §10
- `machine-readable/glossary.json` — Tenant, ObjectGraph, Workflow, Plugin canonical terms

## Validation checks

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
