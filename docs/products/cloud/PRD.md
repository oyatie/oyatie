---
doc_class: ProductRequirements
product: cloud
status: Draft
date: 2026-05-20
owner: council-product + axis-cloud
related_oyatie_adrs:
  - ADR-0003
  - ADR-0007
  - ADR-0009
  - ADR-0040
  - ADR-0043
  - ADR-0044
  - ADR-0045
  - ADR-0050
  - ADR-0199
  - ADR-0220
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0316
related_microservices:
  - cloud-region
  - cloud-cell
  - cloud-compute
  - cloud-storage
  - cloud-network
  - cloud-iam
  - cloud-kms
  - cloud-billing
  - cloud-finops
  - cloud-observability
tenant_class: ["demo_trial", "paid"]
live_readiness_claim: target_non_claim_until_changeset_gate_evidence
doc_status: published
---

# Oyatie — Product PRD: Cloud Provider (AWS-class)

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Readiness claim boundary:** target/non-claim until fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence are attached to a promotion packet.
> **Owning team:** [`teams/axis-cloud/CHARTER.md`](../../teams/axis-cloud/CHARTER.md)
> **Owning axis:** cloud
> **Catalog reference:** `registry/catalog/oya-cloud-*.yaml`
> **Last updated:** 2026-05-09 by Architecture Council

---

## 1. North star (required)

The Cloud axis is **the substrate that runs everything Oyatie ships**, exposed externally as a sovereignty-grade IaaS / PaaS surface (compute / storage / network / IAM / regions / billing / observability) that competes with AWS / GCP / Azure / Naver Cloud / NHN / KT / Kakao Cloud — but is differentiated by being agent-operated (Foundry runs the control plane), audit-chained (every mutation emits to ADR-0003 chain), and built canonical-architecture + regional-pack from day one (KR-Seoul / JP-Tokyo / US-Virginia / EU-Frankfurt all come up in parallel under one cell-routing model). The cloud axis exists *because* Oyatie's competitive moat requires sovereignty-grade infra under one tenancy model — and *also* because shipping cloud externally as a paid product validates the substrate at multi-tenant scale Oyatie cannot reach internally fast enough.

A standalone "Oyatie Cloud" sale (per-resource-hour IaaS) is a real commercial product. The primary architectural job, however, is **non-leakage with the SaaS, Search, Ads, Vertical, and Foundry axes**: one IAM hierarchy, one billing trail, one audit chain, one residency contract, one Cell taxonomy. Without this, "AI manages your cloud" is empty marketing.

## 2. Target users (required)

| Persona | What they get | What they pay for |
|---|---|---|
| **Cloud customer (startup)** | KR-resident or JP-resident or EU-resident IaaS without US hyperscaler exposure; managed Postgres / k8s / object store; per-second metering; serverless functions | Per-resource-hour, per-API-call, per-GB egress |
| **Cloud customer (enterprise)** | Direct interconnect, dedicated cells, encryption-BYOK / HYOK, Cedar-based IAM with SAML federation, signed audit log export, cross-region replication under explicit policy | Committed-use discount + per-resource overage; FinOps console |
| **Sovereign / regulated buyer** (KR public sector, KR healthcare, JP government, EU financial) | Per-pack regulatory evidence (CSAP / K-ISMS-P / KCMVP HSM, ISMAP, FedRAMP path, GAIA-X, DORA), data residency under cell-isolation evidence, regulator-equivalent attestation surface | Sovereign-tier pricing; per-region attestation surfaces |
| **Internal Oyatie axes** (SaaS / Search / Ads / Vertical / Foundry) | Cell-routed compute, storage, network; canonical billing event emission; per-cell IAM; observability dataplane; Foundry-callable mutators | (Internal — metered to axis cost center) |
| **Foundry agent** | Cloud control-plane API surfaced as capabilities; autonomy-ceiling-bound mutators (provision instance, publish IAM role, register region, rebalance capacity) | (Internal — agent-run cost metered to tenant) |
| **Marketplace ISV** | Listing slot for cloud-native ISV apps; revenue share; per-tenant cloud resource provisioning hook | Marketplace listing fees + revenue share |
| **Cloud platform engineer (Oyatie internal)** | IaC profile (OpenTofu per ADR-0050), Argo Rollouts (ADR-0050), Linkerd → Istio Ambient (ADR-0044), VictoriaMetrics → Mimir (ADR-0045, ADR-0042), Harbor (ADR-0044), OpenBao (ADR-0043) | (Internal) |

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| **W-Foundation** | `Resource`, `Region`, `AZ`, `Cell`, `IAM`, `BillingAccount` kernels (`oya-cloud-*-kernel`); cell-isolation primitive; per-cell metadata service (IMDSv2-only per greenfield A11); region/AZ/cell taxonomy; OpenTofu IaC profile (ADR-0050) | None public — kernels and IaC plumbing |
| **W-Substrate** | Foundry binding for cloud control-plane mutators; capability registry projection; Foundry-callable IAM publish, region register, capacity rebalance under autonomy ceiling; audit-chain emit on every mutation | Internal `Cloud Console` v0 (read-only resource browser); IaC pipelines |
| **W-Cloud-Preview** | VM service (KVM / Firecracker tenant compute), Kubernetes-as-a-service (managed), serverless / functions (limited language set), bare-metal lease, GPU fleet (per ADR-0044 hybrid), edge compute (limited PoP); Object store (S3-class), Block store (EBS-class), File store (EFS-class), Archive (Glacier-class), managed Postgres / Citus / pgvector / Redis / Kafka / ClickHouse; VPC + subnets, load balancers (L4 + L7), DNS (authoritative + recursive), CDN, direct interconnect, DDoS protection, service mesh integration; IAM + Account (Cedar policies, SAML/OIDC, STS, identity federation, MFA, audit); per-region per-AZ per-cell taxonomy; encryption-BYOK/HYOK KMS; per-resource-hour metering + per-region tax-invoice format; per-cell observability dataplane (audit log + SLO dashboards) — **all running canonical-architecture + first regional packs (KR-Seoul, JP-Tokyo, US-Virginia, EU-Frankfurt) in parallel** | `Cloud API v1` (control-plane REST + gRPC), `Cloud Console v1` (Leptos web), `Resource browser`, `IAM editor`, `Billing dashboard`, `Foundry capability surface` (cloud.compute.provision, cloud.iam.publish, cloud.region.register, etc.), KR CSAP path documented, audit-log export |
| **W-Cloud-Stable** | Public cloud-provider GA: marketplace, ISV onboarding, multi-AZ failover automation, FinOps surfaces, KR CSAP + K-ISMS-P + KCMVP HSM in production; reserved instance / committed-use; spot / preemptible; cross-region replication under explicit residency policy; managed-service catalog expansion (Cassandra gated ADR-0045, Iceberg gated ADR-0045, Milvus gated ADR-0047, Temporal gated ADR-0035) | Public Cloud API v1 frozen; SLA committed (99.99% data plane); Marketplace; ISV portal |
| **W-Public-GA** | SLA 99.99% control plane on critical mutators, 99.999% data plane on object storage durability (eleven-nines model); regulator-equivalent attestations (CSAP / ISMAP / FedRAMP / GAIA-X / DORA / MeitY / LGPD / NDMO / TDRA / IRAP per regional pack); enterprise procurement (committed use + private offer + custom ToS via legal pack) | All surfaces SLA-backed; regulator portal; private-offer surface |
| **W-Region-Fan-Out** | Add regions in parallel: secondary KR (Busan), JP-Osaka, US-West, EU-Paris, EU-Stockholm, IN-Mumbai, BR-São Paulo, KSA-Riyadh, UAE-Dubai, ANZ-Sydney, SG-Singapore (per regional pack roster); cross-region replication contract per residency class | Per-region surface; per-region regulator dashboard |

### 3.2 Out-of-scope (anti-scope)

- Hardware / chip / data-center construction. (Cloud axis runs on **leased racks + colo**; bare-metal is a *product offering* not an *internal capability*. Per [PRD.md §1.3](../../PRD.md#non-goals).)
- US-hyperscaler-style consumer / retail surfaces (Amazon-style retail, Google-Workspace-style SaaS) — those belong to other axes (SaaS) or are out of scope entirely.
- Cryptocurrency / blockchain mining as a managed service. (Compute is general-purpose; specific blockchain workloads are tenant responsibility.)
- Per-tenant bespoke compliance attestation outside the regional-pack model. Every regulatory binding rides through `oya-platform-regulatory-kernel`; a tenant cannot ask for a bespoke regulator beyond the pack roster.
- IPv4-only deployments after W-Public-GA. IPv6 is required from day 1; IPv4 is preserved for legacy.
- Any cloud-customer onboarding without explicit `region`, `residency`, `regulatory_packs` declaration up front.
- Forking the canonical eventing backbone for cloud-internal events. Cloud uses the same Outbox + Kafka per ADR-0046.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

The Cloud axis owns the **`cloud` bounded context** per [DESIGN.md §1](../../DESIGN.md). Crate prefix:

- `crates/oya-cloud-{compute,storage,network,iam,billing,observability,region,resource}-*`

Per ADR-0015 §1: `oya-<context>-<role>[-<capability>]`.

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — KVM/Firecracker, Kubernetes API, S3 backend, Postgres, KMS, BGP, OpenTofu
api       — inbound HTTP/gRPC servers (cloud control-plane API)
worker    — inbound queue/Kafka consumers (capacity rebalance, billing aggregation)
runtime   — composition root (binary)
```

| Crate | Role | One-line role |
|---|---|---|
| `oya-cloud-resource-kernel` | kernel | Generic Resource aggregate (id, type, region, owner, state, metering tag) |
| `oya-cloud-region-kernel` | kernel | Region / AZ / Cell taxonomy; residency class binding |
| `oya-cloud-region-domain` | domain | Region register / decommission, AZ failover, cell-rebalance |
| `oya-cloud-region-api` | api | Region / AZ listing REST API |
| `oya-cloud-cell-application` | application | (SUPERSEDED: stub orphan deleted per ADR-0106 §Consequences + audit #6; canonical `-app` scaffold pending M02-P18) Tenant cell-binding REST surface |
| `oya-cloud-iam-kernel` | kernel | IAM principal, role, policy (Cedar-based), STS session, federation |
| `oya-cloud-iam-domain` | domain | Identity federation, role assumption, key issuance |
| `oya-cloud-iam-adapter` | adapter | OIDC, SAML, OAuth, regional-pack IdP impls (Login.gov, eIDAS, Aadhaar, etc.) |
| `oya-cloud-iam-api` | api | IAM REST API v1, STS endpoint |
| `oya-cloud-kms-api` | api | KMS encrypt/decrypt authorization receipt REST API |
| `oya-cloud-compute-kernel` | kernel | Instance, ImageRef, Flavor, KeyPair, Snapshot, AutoScalingGroup |
| `oya-cloud-compute-domain` | domain | Provision / start / stop / snapshot / live-migrate / live-recover |
| `oya-cloud-compute-adapter-kvm` | adapter | KVM hypervisor + libvirt binding |
| `oya-cloud-compute-adapter-firecracker` | adapter | Firecracker microVM (function workloads) |
| `oya-cloud-compute-adapter-k8s` | adapter | Kubernetes-as-a-service control loop |
| `oya-cloud-compute-api` | api | Compute REST + gRPC API |
| `oya-cloud-compute-vm-api` | api | VM create REST API |
| `oya-cloud-compute-k8s-api` | api | Kubernetes cluster create REST API |
| `oya-cloud-compute-functions-api` | api | Function invocation REST API |
| `oya-cloud-storage-kernel` | kernel | Bucket, Object, Volume, Filesystem, Snapshot, ArchiveTier |
| `oya-cloud-storage-object-api` | api | Object metadata PUT/GET REST API |
| `oya-cloud-storage-adapter-s3` | adapter | S3-compatible object backend (Ceph / SeaweedFS / MinIO frontend) |
| `oya-cloud-storage-adapter-block` | adapter | iSCSI / NBD / Ceph RBD block backend |
| `oya-cloud-storage-adapter-file` | adapter | NFSv4 / SMB file backend (CephFS) |
| `oya-cloud-storage-block-api` | api | Block volume create REST API |
| `oya-cloud-storage-api` | api | Storage REST + S3-compatibility API |
| `oya-cloud-network-kernel` | kernel | VPC, Subnet, RouteTable, NIC, SecurityGroup, LoadBalancer, DnsZone |
| `oya-cloud-network-adapter` | adapter | OVN / OVS / BGP / FRR / CoreDNS |
| `oya-cloud-network-adapter-selfhosted` | adapter | Self-hosted/colo VPC + DNS request-contract adapter for OVN / OVS / BGP / FRR-backed tenant network segments and CoreDNS/authoritative-zone control |
| `oya-cloud-network-vpc-api` | api | VPC create REST API |
| `oya-cloud-network-lb-api` | api | Load balancer create REST API |
| `oya-cloud-network-dns-api` | api | DNS zone create REST API |
| `oya-cloud-network-api` | api | Network REST API |
| `oya-cloud-billing-kernel` | kernel | BillingAccount, MeterEvent, Invoice, Discount, BudgetAlert |
| `oya-cloud-billing-domain` | domain | Aggregation, rate-card application, tax-invoice issuance per regional pack |
| `oya-cloud-billing-adapter` | adapter | Postgres + ClickHouse billing aggregation; per-pack tax formatter |
| `oya-cloud-billing-api` | api | Billing REST API + invoice surfaces |
| `oya-cloud-billing-app` | app | Cloud billing event ingest CloudEvents/Protobuf surface + outbox publication |
| `oya-cloud-billing-tax-application` | application | (SUPERSEDED: stub orphan deleted per ADR-0106 §Consequences + audit #6; canonical `-app` scaffold pending M03 cloud-billing) Regional tax invoice generation REST surface |
| `oya-cloud-observability-kernel` | kernel | Metric, LogStream, Trace, Alert, Dashboard |
| `oya-cloud-observability-adapter` | adapter | VictoriaMetrics → Mimir (ADR-0045, ADR-0042); Loki; Tempo; OTel collector |
| `oya-cloud-observability-api` | api | Observability REST API |
| `oya-cloud-supply-chain-app` | app | Cosign / Trivy / SBOM attestation per ADR-0039 |
| `oya-cloud-marketplace-kernel` | kernel | ISV listing for cloud-native apps |
| `oya-cloud-marketplace-adapter` | adapter | Tied to `oya-saas-marketplace-kernel` for cross-axis listing |
| `oya-cloud-finops-api` | api | Per-tenant cost analytics report API, budget anomaly surfacing, FinOps recommendations |
| `oya-cloud-resource-runtime` | runtime | Composition root |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Cloud API v1` (Compute / Storage / Network / IAM / Region / Billing) | `contracts/cloud-api-v1.openapi.yaml` | control | p99 ≤ 500 ms control mutation; 99.95% (preview) → 99.99% (GA) availability |
| `Cloud Storage Object API` | `contracts/openapi/cloud/cloud-storage-object-v1.yaml` | data | p99 metadata GET ≤ 100 ms; per-object KMS shred binding |
| `S3-compatible Object API` | `contracts/cloud-storage-s3.openapi.yaml` | data | p99 GET ≤ 100 ms; 99.99% (GA); 11-nines durability |
| `Block Storage API` | `contracts/openapi/cloud/cloud-storage-block-v1.yaml` | control | p99 ≤ 500 ms block volume create; per-IOPS class SLO is enforced at attachment/runtime |
| `File Storage API` (NFS/SMB) | `contracts/cloud-storage-file.openapi.yaml` | data | per-throughput class SLO |
| `Compute VM API` | `contracts/openapi/cloud/cloud-compute-vm-v1.yaml` | control | p99 ≤ 500 ms VM create boundary; runtime provision p95 ≤ 60 s |
| `Compute Kubernetes API` | `contracts/openapi/cloud/cloud-compute-k8s-v1.yaml` | control | p99 ≤ 500 ms cluster create boundary; managed control plane provision p95 ≤ 10 min |
| `Compute Functions API` | `contracts/openapi/cloud/cloud-compute-functions-v1.yaml` | data | p99 ≤ 250 ms invocation receipt boundary; cold-start budget ≤ 1 s |
| `Compute API` (instance, k8s, function) | `contracts/cloud-compute.openapi.yaml` | control + data | provision p95 ≤ 60 s; 99.99% (GA) |
| `IAM + STS API` | `contracts/openapi/cloud/cloud-iam-v1.yaml` | control | p99 ≤ 100 ms STS issuance; 99.99% |
| `KMS API` | `contracts/openapi/cloud/cloud-kms-v1.yaml` | data | p99 ≤ 100 ms encrypt/decrypt authorization receipt; KCMVP/FIPS-bound |
| `Region API` | `contracts/openapi/cloud/cloud-region-v1.yaml` | control | p99 ≤ 100 ms region/AZ catalog reads; immutable public projection |
| `Cell Binding API` | `contracts/openapi/cloud/cloud-cell-bind-v1.yaml` | control | p99 ≤ 500 ms tenant cell-routing assignment; immutable per-tenant binding |
| `VPC / Network API` | `contracts/openapi/cloud/cloud-network-vpc-v1.yaml`; `contracts/openapi/cloud/cloud-network-lb-v1.yaml`; `contracts/openapi/cloud/cloud-network-dns-v1.yaml` | control | p99 ≤ 500 ms VPC/LB/DNS create boundary; per-mutation SLO for later network surfaces |
| `Billing Event Ingest` | `contracts/asyncapi/cloud/cloud-billing-events-v1.yaml`; `contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto` | data | CloudEvents 1.0 + Protobuf payload accepted idempotently and published through canonical outbox |
| `Billing API + Invoice surface` | `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` | control + analytics | end-of-cycle invoice within 48 h; p99 ≤ 500 ms invoice generation boundary |
| `FinOps Report API` | `contracts/openapi/cloud/cloud-finops-report-v1.yaml` | analytics | p99 ≤ 500 ms tenant cost report generation; budget and margin anomaly projection |
| `Observability API` (logs / metrics / traces / dashboards / audit read) | `contracts/openapi/cloud/cloud-observability-audit-v1.yaml` | analytics | per-stream class SLO; p99 ≤ 500 ms audit-read projection |
| `Marketplace API` (cloud-app listing) | `contracts/cloud-marketplace-v1.openapi.yaml` | control | 99.9% |
| `Foundry capability surface` (cloud.* mutators) | `registry/capability-templates/cloud.*.yaml` | control + audit | p99 ≤ 500 ms; every call audit-emits |
| `Cloud Console (web)` | `apps/oya-cloud-console/` (Leptos, ADR-0033) | control | p95 ≤ 1 000 ms; 99.9% |
| `Direct interconnect` | physical port + BGP | network | per-link SLA; 99.99% |
| `Audit log export` | per-tenant signed S3 stream | audit | 100% emission completeness |

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|
| Resource lifecycle | `Resource`, `ResourceKind`, `ResourceRepo` in `oya-cloud-resource-kernel` | All Oyatie axes that consume cloud (SaaS, Search, Ads, Foundry, Vertical) |
| Region / AZ / Cell | `RegionCode`, `AzCode`, `CellId`, `Cell::route_for(tenant)` in `oya-cloud-region-kernel` | All axes (residency-aware) |
| IAM / STS | `IamRole`, `Sts::assume(...)` in `oya-cloud-iam-kernel` | All axes |
| Billing event | `CloudBillingEvent`, `CloudBillingLedger::ingest(...)`, and `oya-cloud-billing-app` event boundary | SaaS (per-tenant cost), FinOps, Marketplace, Tax |
| Observability dataplane | `MetricStream`, `LogStream`, `TraceStream` | All axes |
| Foundry mutator surface | `cloud.compute.provision` / `cloud.iam.publish` / `cloud.region.register` / `cloud.capacity.rebalance` capabilities | Foundry |
| Object store | `Bucket`, `ObjectRepo` in `oya-cloud-storage-kernel` | Search (corpus), Ads (impression archive), SaaS (tenant-asset), Vertical (clinical-image archive) |

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| Tenant kernel | SaaS | `oya-platform-tenant-kernel` | Cross-axis (mandatory all-axis review) |
| Identity / Cedar policy | SaaS | `oya-platform-identity-kernel` | Two-ADR lockstep with `oya-cloud-iam-kernel` |
| Audit-chain event | SaaS / Audit subsystem | `oya-platform-audit-chain-kernel` | Audit + downstream-consumer review |
| Eventing backbone | SaaS | `oya-platform-eventing-kernel` | Cross-axis on topic shape |
| Capability invocation | Foundry | `contracts/foundry-capability.openapi.yaml` | Cross-axis (foundry + cloud) |
| Autonomy ceiling | Foundry | `oya-intelligence-policy-kernel` | Governance + security |
| Regulatory pack | SaaS / Vertical | `oya-platform-regulatory-kernel` | Vertical + regulatory review |
| Metering kernel | SaaS | `oya-platform-metering-kernel` | Billing + tax review |

(Mirror in [DESIGN.md §10](../../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-cloud-*-kernel`)

```rust
// oya-cloud-resource-kernel
pub struct Resource {
    pub id: ResourceId,                        // ulid; arn-style "oya:cloud:<region>:<tenant>:<kind>:<name>"
    pub tenant_id: TenantId,                   // every record carries tenant
    pub region: RegionCode,                    // KR-Seoul1, JP-Tokyo1, US-Virginia1, EU-Frankfurt1, ...
    pub az: Option<AzCode>,                    // KR-Seoul1-a/b/c
    pub cell_id: CellId,                       // physical isolation unit
    pub kind: ResourceKind,                    // Instance | Bucket | Volume | Vpc | LbV4 | LbV7 | Function | ...
    pub data_class: DataClass,                 // metadata is PUBLIC; data within is per-record
    pub owner_principal: PrincipalId,
    pub state: ResourceState,                  // pending | running | stopped | terminated | error
    pub tags: BTreeMap<TagKey, TagValue>,      // tenant-defined; cost allocation
    pub iam_policy_attachments: Vec<IamPolicyId>,
    pub residency: ResidencyClass,             // strict_kr | eea | us | jp | global
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC (metadata; the data within data-plane resources lives elsewhere with per-record class)

pub enum ResourceKind {
    ComputeInstance(InstanceFlavor),
    KubernetesCluster(K8sFlavor),
    Function(FunctionRuntime),
    BareMetal(BareMetalFlavor),
    GpuFleet(GpuFlavor),
    Bucket(BucketTier),
    Volume(VolumeTier),
    Filesystem(FilesystemTier),
    ArchiveVault,
    Vpc,
    Subnet,
    LoadBalancer(LbProtocol),
    DnsZone,
    CdnDistribution,
    DirectInterconnect,
    DdosProtection,
    Database(DatabaseEngine),
    QueueOrStream(QueueEngine),
    SearchIndex,
    KmsKey,
    SecretBundle,
    Image(ImageKind),
}
```

```rust
// oya-cloud-region-kernel
pub struct Region {
    pub code: RegionCode,                      // KR-Seoul1, JP-Tokyo1, ...
    pub display_name: String,                  // "Korea (Seoul)"
    pub regulatory_packs: Vec<RegulatoryPackId>, // PIPA+CSAP for KR-Seoul1; APPI+ISMAP for JP-Tokyo1
    pub data_class: DataClass,                 // PUBLIC
    pub azs: Vec<Az>,                          // typically 3
    pub state: RegionState,                    // planned | preview | ga | retiring
    pub provider_facing: bool,                 // cloud-customer visible
    pub residency_strictness: ResidencyClass,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control

pub struct Az {
    pub code: AzCode,                          // KR-Seoul1-a
    pub region: RegionCode,
    pub data_class: DataClass,                 // PUBLIC
    pub physical_ref: PhysicalSiteRef,         // colo / lease site
    pub power_zones: Vec<PowerZoneId>,
    pub cells: Vec<CellId>,
    pub state: AzState,
}

pub struct Cell {
    pub id: CellId,                            // ulid
    pub region: RegionCode,
    pub az: AzCode,
    pub data_class: DataClass,                 // PUBLIC (the cell metadata; data-plane is per-tenant)
    pub tenant_density: TenantDensityClass,    // shared | dedicated | sovereign | air-gapped
    pub allowed_residency: BTreeSet<ResidencyClass>,
    pub capacity: CellCapacity,                // {compute_vcpu, mem_gb, ssd_tb, gpu_count}
    pub utilization: CellUtilization,
    pub allocations: Vec<CellAllocation>,      // per-tenant slice
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC for metadata; tenant data within cell is per-record
```

```rust
// oya-cloud-iam-kernel
pub struct IamPrincipal {
    pub id: IamPrincipalId,
    pub tenant_id: TenantId,
    pub kind: IamPrincipalKind,                 // user | service-account | role | federated | external |
    pub display_name: String,                   // data_class: PII_QUASI for user kind; PUBLIC for service
    pub external_subject: Option<SubjectUri>,   // SAML/OIDC sub
    pub region_pack: RegionalPackId,            // for IdP routing
    pub mfa_state: MfaState,
    pub last_authenticated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: MIXED (display_name varies); aggregate is bound by tenant_id

pub struct IamRole {
    pub id: IamRoleId,
    pub tenant_id: TenantId,
    pub name: RoleName,
    pub cedar_policy_id: CedarPolicyId,         // links to Cedar policy bundle
    pub assumable_by: Vec<IamPrincipalId>,
    pub max_session_duration_sec: u32,
    pub data_class: DataClass,                  // PUBLIC (audit metadata)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control

pub struct StsSession {
    pub id: StsSessionId,
    pub tenant_id: TenantId,
    pub assumed_role: IamRoleId,
    pub assumed_by: IamPrincipalId,
    pub external_id: Option<String>,            // for cross-tenant assume-role
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<ScopeRef>,
    pub data_class: DataClass,                  // PUBLIC (token metadata; bearer token never persisted in clear)
    pub schema_version: u32,
}
// plane: control
```

```rust
// oya-cloud-compute-kernel
pub struct Instance {
    pub resource_id: ResourceId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub az: AzCode,
    pub cell_id: CellId,
    pub flavor: InstanceFlavor,                 // {vcpu, mem_gb, gpu, local_ssd_gb}
    pub image: ImageRef,                        // OCI artifact in Harbor (ADR-0044) or qcow2
    pub key_pair: Option<KeyPairId>,
    pub vpc: VpcId,
    pub subnet: SubnetId,
    pub security_groups: Vec<SecurityGroupId>,
    pub iam_role: Option<IamRoleId>,            // instance-attached role
    pub user_data_uri: Option<UserDataUri>,     // cloud-init payload
    pub state: InstanceState,                   // pending | running | stopping | stopped | terminated
    pub data_class: DataClass,                  // PUBLIC (control); guest data is per-tenant
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control + data (the metadata is control; the running guest is data plane)
```

```rust
// oya-cloud-storage-kernel
pub struct Bucket {
    pub resource_id: ResourceId,
    pub tenant_id: TenantId,
    pub name: BucketName,                       // tenant-globally-unique
    pub region: RegionCode,
    pub residency: ResidencyClass,
    pub tier: BucketTier,                       // standard | infrequent | archive | glacier
    pub replication: ReplicationPolicy,         // none | regional | cross-region (per residency)
    pub encryption: EncryptionMode,             // sse | sse-kms | byok | hyok
    pub kms_key: Option<KmsKeyId>,
    pub object_lock: Option<ObjectLockPolicy>,  // legal-hold + WORM
    pub allowed_data_classes: BTreeSet<DataClass>, // tenant-declared; CI fitness checks
    pub state: BucketState,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control (bucket metadata); data is data plane
// data_class: declared per bucket; CI fitness checks against tenant consent

pub struct StoredObject {
    pub bucket_id: ResourceId,
    pub tenant_id: TenantId,
    pub key: ObjectKey,
    pub size_bytes: u64,
    pub etag: ETag,
    pub data_class: DataClass,                  // per-object override; defaults to bucket
    pub stored_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
    // body lives in object backend, not in metadata
}
// plane: data
```

```rust
// oya-cloud-network-kernel
pub struct Vpc {
    pub resource_id: ResourceId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub cidr_v4: Ipv4Cidr,
    pub cidr_v6: Ipv6Cidr,
    pub flow_logs_enabled: bool,                // audit emission requires
    pub state: VpcState,
    pub data_class: DataClass,                  // PUBLIC (metadata)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control

pub struct LoadBalancer {
    pub resource_id: ResourceId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub kind: LbKind,                           // l4-tcp | l4-udp | l7-http | l7-grpc
    pub listeners: Vec<Listener>,
    pub target_groups: Vec<TargetGroup>,
    pub mtls: Option<MtlsConfig>,
    pub waf_policy: Option<WafPolicyId>,
    pub state: LbState,
    pub data_class: DataClass,                  // PUBLIC
    pub schema_version: u32,
}
// plane: control + data
```

```rust
// oya-cloud-billing-kernel
pub struct BillingAccount {
    pub id: BillingAccountId,
    pub tenant_id: TenantId,
    pub region: RegionCode,                     // primary tax region
    pub regional_pack: RegionalPackId,          // governs tax-invoice format
    pub payment_method: PaymentMethodRef,       // pseudonymous; PCI scope minimized
    pub credit_balance: Money,
    pub committed_use: Vec<CommitmentTerm>,
    pub budget_alerts: Vec<BudgetAlert>,
    pub state: BillingAccountState,             // active | suspended | delinquent
    pub data_class: DataClass,                  // FINANCIAL_GENERAL (data-class 5)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control + analytics
// data_class: FINANCIAL_GENERAL — never flows to ads; analytics aggregation only

pub struct Invoice {
    pub id: InvoiceId,
    pub billing_account_id: BillingAccountId,
    pub tenant_id: TenantId,
    pub period: BillingPeriod,
    pub line_items: Vec<InvoiceLineItem>,
    pub subtotal: Money,
    pub tax: Money,                             // per regional-pack TaxInvoiceFormatter
    pub total: Money,
    pub tax_invoice_format: TaxInvoiceFormat,    // KR 전자세금계산서 / JP 適格請求書 / EU per-country / IN GST / BR NF-e
    pub state: InvoiceState,                    // draft | issued | paid | overdue | void
    pub issued_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: analytics + control
// data_class: FINANCIAL_GENERAL
```

### 5.2 Aggregate boundaries

- **Resource aggregate**: `Resource` is the consistency boundary; per-kind details (`Instance`, `Bucket`, etc.) cluster under the Resource record.
- **Region aggregate**: `Region` + `Az[]` + `Cell[]` change as one unit (rare changes; council-gated).
- **IAM aggregate**: `IamPrincipal` + assumable `IamRole[]` cluster around the principal; Cedar policies are evaluated separately per request.
- **Billing aggregate**: `BillingAccount` + open `Invoice` cluster; closed invoices are immutable.
- **VPC aggregate**: `Vpc` + `Subnet[]` + `RouteTable` change as one unit per region.
- **Load balancer aggregate**: `LoadBalancer` + listeners + target groups.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Resource | Postgres (per-region cluster) | `(tenant_id, region)` | per-region per-tenant | streaming repl 3-AZ; cross-region read-only mirror gated by residency | indefinite (until terminate + retention horizon) |
| Region / AZ / Cell | Postgres + etcd-class consistent KV (control plane) | global single source | central with per-region cache | strong consistency; multi-region read replica | indefinite |
| IAM Principal / Role / Policy | Postgres + Cedar policy store | `tenant_id` | per-tenant | 3-AZ | indefinite |
| STS session | Redis (short-lived) | `session_id` | sharded | 3-replica | up to `max_session_duration` |
| Instance | Postgres (region cluster) | `(tenant_id, region)` | per-region per-tenant | 3-AZ | until terminate + 90 d for forensics |
| Bucket / Object metadata | Postgres + ClickHouse for object listing | `(tenant_id, bucket_id)` | per-bucket | 3-AZ + cross-region per replication policy | per-object lock |
| Object body | Object backend (Ceph / SeaweedFS / MinIO frontend) | `(bucket, key)` hash | erasure-coded, replicated | 3+1 EC default; cross-region per policy | per-tier lifecycle |
| Volume snapshot | Block backend + Postgres metadata | `(tenant_id, volume_id)` | per-volume | 3-replica | per-policy |
| VPC / Subnet / LB | Postgres (region cluster) | `(tenant_id, region)` | per-region per-tenant | 3-AZ | indefinite |
| BillingAccount / Invoice | Postgres + ClickHouse archive | `tenant_id` | per-tenant | 3-AZ + cold to Iceberg per ADR-0045 | 7y (tax) |
| MeterEvent (cloud) | ClickHouse | `(tenant_id, region)` + time | per-tenant per-day per-region | 3-AZ + cold | 7y |
| Audit-chain block (cloud-emitted) | Postgres + S3-class anchor | tenant + time | per-tenant per-day | 3-AZ + cross-region | indefinite |

### 5.4 Event schemas (events emitted)

All events go through the canonical eventing backbone per ADR-0050/0174 + outbox pattern. Implemented event surfaces use CloudEvents 1.0 envelopes, Protobuf payloads, and AsyncAPI schemas per `docs/standards/api-design.md`; legacy planned rows remain migration targets until implemented.

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `cloud.billing.event.ingest.v1` | `oya.cloud.billing` | `contracts/asyncapi/cloud/cloud-billing-events-v1.yaml`; `contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto` | SaaS billing, FinOps, Marketplace, Tax, platform metering | 7y | `idempotency_key` |
| `cloud.resource_created.v1` | `oya.cloud.resource` | `contracts/events/cloud.resource_created.v1.avsc` | Billing (start meter), Audit, FinOps, SaaS metering | 90 d | `(tenant_id, resource_id)` |
| `cloud.resource_terminated.v1` | `oya.cloud.resource` | `contracts/events/cloud.resource_terminated.v1.avsc` | Billing (stop meter), Audit, FinOps | 90 d | `(tenant_id, resource_id)` |
| `cloud.iam_role_assumed.v1` | `oya.cloud.iam` | `contracts/events/cloud.iam_role_assumed.v1.avsc` | Audit (per-assume record), Foundry (capability bind) | 90 d | `sts_session_id` |
| `cloud.iam_policy_changed.v1` | `oya.cloud.iam` | `contracts/events/cloud.iam_policy_changed.v1.avsc` | Audit, Cedar evaluator cache invalidate, Foundry policy projection | indefinite | `(tenant_id, policy_id, version)` |
| `cloud.region_registered.v1` | `oya.cloud.region` | `contracts/events/cloud.region_registered.v1.avsc` | All axes (residency-aware), regulatory pack binding, marketplace | indefinite | `region_code` |
| `cloud.cell_rebalanced.v1` | `oya.cloud.region` | `contracts/events/cloud.cell_rebalanced.v1.avsc` | Audit, FinOps, observability | 90 d | `(cell_id, rebalance_seq)` |
| `cloud.invoice_issued.v1` | `oya.cloud.billing` | `contracts/events/cloud.invoice_issued.v1.avsc` | SaaS billing-account update, Tax (regional pack), Tenant trust portal | 7y | `invoice_id` |
| `cloud.budget_alert.v1` | `oya.cloud.billing` | `contracts/events/cloud.budget_alert.v1.avsc` | Tenant FinOps surface, (notification) | 30 d | `(billing_account_id, alert_seq)` |
| `cloud.bucket_replication_lag.v1` | `oya.cloud.storage` | `contracts/events/cloud.bucket_replication_lag.v1.avsc` | Observability, Foundry remediation capability | 14 d | `(bucket_id, ts)` |
| `cloud.object_lifecycle_transitioned.v1` | `oya.cloud.storage` | `contracts/events/cloud.object_lifecycle_transitioned.v1.avsc` | Audit, FinOps, Search re-index hint | 90 d | `(bucket_id, key, transition_seq)` |
| `cloud.network_flow_anomaly.v1` | `oya.cloud.network` | `contracts/events/cloud.network_flow_anomaly.v1.avsc` | Security review, Foundry remediation, Audit | 90 d | `(vpc_id, anomaly_id)` |
| `cloud.kms_key_used.v1` | `oya.cloud.iam` | `contracts/events/cloud.kms_key_used.v1.avsc` | Audit (per-decrypt record per ADR-0003 properties) | indefinite | `(key_id, use_seq)` |

### 5.5 Index / search-index touchpoints

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|
| `Resource.tags` (when public-attribute) | `oya-search-cloud-resource-public` | `PUBLIC` only | Yes |
| `Marketplace listing` (cloud-app) | `oya-search-marketplace-public` | `PUBLIC` | Yes |
| `Region.display_name + capabilities` | `oya-search-cloud-region-public` | `PUBLIC` | n/a |

(Cloud is primarily a control + data plane; the search-index fan-out is light. Most cloud data is per-tenant private and never indexed cross-tenant.)

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../../DESIGN.md) + ADR-0003, every regulated capability must emit.

| Operation | Emits topic | Required fields |
|---|---|---|
| Resource created | `oya.audit.cloud_resource_created` | `tenant_id`, `region`, `cell_id`, `kind`, `actor`, `iam_role`, `timestamp`, `prev_hash` |
| Resource terminated | `oya.audit.cloud_resource_terminated` | `tenant_id`, `resource_id`, `actor`, `reason`, `timestamp`, `prev_hash` |
| IAM role assumed | `oya.audit.cloud_iam_assume` | `tenant_id`, `role_id`, `assumed_by`, `external_id`, `scopes`, `timestamp`, `prev_hash` |
| IAM policy changed | `oya.audit.cloud_iam_policy` | `tenant_id`, `policy_id`, `before_hash`, `after_hash`, `actor`, `timestamp`, `prev_hash` |
| Region registered | `oya.audit.cloud_region_register` | `region_code`, `regulatory_packs`, `actor`, `attestation_refs`, `timestamp`, `prev_hash` |
| KMS key used (decrypt) | `oya.audit.cloud_kms_use` | `tenant_id`, `key_id`, `purpose`, `actor`, `data_class_referenced`, `timestamp`, `prev_hash` |
| Cross-region replication | `oya.audit.cloud_replication` | `tenant_id`, `bucket_id`, `src_region`, `dst_region`, `data_classes_present`, `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Network flow anomaly | `oya.audit.cloud_flow_anomaly` | `tenant_id`, `vpc_id`, `flow_pattern`, `severity`, `disposition`, `timestamp`, `prev_hash` |
| Invoice issued | `oya.audit.cloud_invoice` | `tenant_id`, `billing_account_id`, `invoice_id`, `total`, `tax_invoice_format`, `regional_pack`, `timestamp`, `prev_hash` |
| Direct interconnect provisioned | `oya.audit.cloud_interconnect` | `tenant_id`, `interconnect_id`, `bandwidth`, `peer_asn`, `peering_location`, `actor`, `timestamp`, `prev_hash` |

### 5.7 Schema migration policy

- **Versioning**: `schema_version: u32` per kernel entity; monotonic per region.
- **Reversibility**: every migration ships up + down DDL; per-region rollout via Argo Rollouts (ADR-0050) with automated metric-gated rollback.
- **Dry-run gate**: Foundry fitness function `oya-governance-migration` runs against synthetic 10k-resource per-region tenant before merge.
- **Region-pack-conditional migrations**: regional packs declare migrations independently; canonical core never depends on a pack-specific column.

## 6. Optimization practices (required) — *slice-level*

| Practice | Implementation choice |
|---|---|
| Cell routing | `Tenant.region` + tenant density class chooses cell; Envoy header `x-oya-cell` routes resource API to cell-local Postgres + control-plane services |
| Sharding strategy | Per-region Postgres clusters; Citus (ADR-0045) per-tenant within cell; ClickHouse per-region per-day for billing + observability; object backend erasure-coded across AZs |
| Caching tier | In-memory (moka) for hot Region + Cell + Cedar policy; Redis for IAM session + STS short-circuit; CDN for static control-plane assets and Cloud Console |
| Bulk endpoint contract | `BatchCreateResources`, `BulkAttachIam`, `BulkObjectDelete`, `BulkSnapshotPolicy`; max batch 1 000 resources or 100 000 objects |
| Pagination | Cursor-based (`(updated_at, resource_id)` opaque token); list APIs default 100, max 10 000 |
| Idempotency | `Idempotency-Key` header on every mutating REST + gRPC call; outbox dedupes 24 h; cloud-init runs are deduped on `instance_id` |
| Batch dispatch | Capacity-rebalance batches every 5 s; billing aggregation batches every 60 s; observability ingest batches every 1 s or 256 events |
| Backpressure | Capacity-bound rejection at cell with `429`+`Retry-After`; observability ingest sheds to dead-letter at 95% lag; LB control loop slows under metric pressure |
| Hot-path benchmarks | STS issuance (`p99 ≤ 100 ms`), Cedar evaluation (`p99 ≤ 5 ms`), object GET (`p99 ≤ 100 ms`), instance `provision-to-running` (`p95 ≤ 60 s`) — wired to `oya-governance-bench` |
| Agent-driven optimization loops | Foundry capability `cloud.capacity.rebalance` (autonomy ≤ T2): proposes cell rebalance from utilization metrics; `cloud.cost.recommend` (≤ T1): identifies idle resources, recommends down-sizing; `cloud.iam.audit-narrow` (≤ T2): proposes least-privilege role narrowing from access logs; human approves before execution at T2 |
| FinOps unit-economics | Per-tenant cost = sum(`MeterEvent.units` × per-region rate-card); per-resource breakdown in `Cloud Console FinOps`; target gross-margin per region ≥ 50% at GA |
| Build-cache and CI affected-graph | `oya-cloud-*` is the largest per-region change subgraph; ADR-0015 flat boundaries keep change-radius bounded; per-region IaC profile (OpenTofu, ADR-0050) is run-once per affected region |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Region → regulatory binding | `RegulatoryPack` in `oya-platform-regulatory-kernel` | yes | KR (PIPA/CSAP/K-ISMS-P/KCMVP/KISA); JP (APPI/ISMAP); US (HIPAA/CCPA/SOX/FedRAMP); EU (GDPR/DORA/GAIA-X); IN (DPDP/MeitY); BR (LGPD); KSA (PDPL/NDMO/SDAIA); UAE (TDRA/ADGM); ANZ (Privacy Act/IRAP); SG (PDPA-SG/MAS) |
| Identity provider (cloud customer SSO) | `IdentityProvider` in `oya-platform-identity-kernel` | yes | KR (본인확인서비스, Kakao, Naver), JP (マイナンバー), US (Login.gov), EU (eIDAS), IN (Aadhaar), BR (gov.br), KSA (Absher), UAE (UAE-PASS), ANZ (myGovID) |
| Tax-invoice formatter | `TaxInvoiceFormatter` in `oya-platform-billing-tax-kernel` | yes | KR 전자세금계산서, JP 適格請求書, EU per-country e-invoicing, IN GST, BR NF-e, KSA FATOORA |
| Address validator (interconnect site, cloud customer billing) | `AddressValidator` in `oya-platform-address-kernel` | yes | every pack |
| Payment rail (cloud-customer-side) | `PaymentRail` in `oya-saas-billing-rail-kernel` (shared) | yes | KR (Toss/계좌이체), JP (口座振替), US (ACH/Wire), EU (SEPA), IN (UPI), BR (Pix), KSA (SADAD/Mada) |
| Per-region HSM/KMS | `HsmAdapter` in `oya-cloud-iam-kernel` | yes | KR (KCMVP-certified HSM), JP (CRYPTREC), US (FIPS-140-3), EU (Common Criteria EAL-4+), KSA (NCA-NCS) |
| Per-region attestation surface | `AttestationProvider` in `oya-cloud-supply-chain-app` | yes | KR (CSAP / K-ISMS-P), JP (ISMAP), US (FedRAMP / SOC-2 / ISO-27001), EU (C5 / GAIA-X), KSA (ECC), UAE (TRA / ADGM) |
| Per-region observability data residency | `ObservabilityResidency` in `oya-cloud-observability-kernel` | yes | every pack (logs/metrics/traces stay in-region by default) |
| Direct interconnect peering | `InterconnectPartner` in `oya-cloud-network-kernel` | yes | KR (KIX/KINX), JP (JPIX/BBIX), US (Equinix/Megaport), EU (DE-CIX/AMS-IX), SG (SGIX) |

## 8. In-house vs external dependency posture (required)

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `axum` / `tokio` / `serde` / `rustls` / `tonic` / `sqlx` | kernel-grade | MIT/Apache-2 | no | adopt |
| `KVM` / libvirt | secondary | GPL-2 *(host kernel boundary; not linked into product)* | own hypervisor — rejected | adopt at host boundary; never linked into Rust binary; ADR governs |
| `Firecracker` (AWS) | secondary | Apache-2 | own microVM — rejected | adopt for function workloads |
| `Kubernetes` (control plane) | secondary | Apache-2 | own orchestrator — rejected | adopt as managed service; ADR-0044 service mesh = Istio Ambient |
| `Ceph` / `SeaweedFS` / `MinIO` (object backend) | secondary | LGPL-3 (Ceph; boundary only) / Apache-2 (Seaweed) / AGPL-3 (MinIO; rejected) | own object backend — rejected | **adopt Ceph at process boundary** (LGPL crosses no Rust link) and SeaweedFS for non-EC workloads; **MinIO REJECTED** (AGPL-3 incompatible with product code) |
| `OVN` / `OVS` (network virtualization) | secondary | Apache-2 | own SDN — rejected | adopt |
| `BGP` / FRR | secondary | GPL-2 *(daemon boundary)* | own routing — rejected | adopt at daemon boundary; never linked into product |
| `OpenTofu` (IaC) | secondary | MPL-2 | own IaC — rejected | adopt (ADR-0050; supersedes Terraform after BUSL change) |
| `Cosign` / `Rekor` / `Trivy` | secondary | Apache-2 | own supply chain — rejected | adopt (ADR-0039) |
| `OpenBao` (secrets) | secondary | MPL-2 | own secret store — rejected | adopt (ADR-0043) |
| `Cedar` (IAM policy) | secondary | Apache-2 | OPA / own — Cedar wins on auditability | adopt with ADR |
| `VictoriaMetrics` / `Loki` / `Tempo` / `Mimir` (observability) | secondary | Apache-2 (VM/Loki/Tempo) / AGPL-3 (Mimir; boundary only) | Prometheus + own — rejected | adopt VM day-1 (ADR-0045); Mimir gated and only at process boundary (ADR-0042) |
| `OpenTelemetry` SDKs | kernel-grade | Apache-2 | no | adopt |
| `Argo Rollouts` (progressive delivery) | secondary | Apache-2 | own canary — rejected | adopt (ADR-0050) |
| `Harbor` (container registry) | secondary | Apache-2 | own registry — rejected | adopt (ADR-0044) |
| `Istio Ambient` (service mesh) | secondary | Apache-2 | Linkerd (ADR-0044 superseded) | adopt (ADR-0044) |
| `Apache Kafka` | secondary | Apache-2 | own event bus — rejected; outbox is day-1 | adopt gated (ADR-0046) |
| `ClickHouse` | secondary | Apache-2 | own OLAP — rejected | adopt (ADR-0045) |

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review. GPL daemons (KVM/FRR) and AGPL extensions (Mimir) are allowed only at process boundary; the boundary is governed by ADR-0039 and planned advisory lane `oya-governance-license`.

## 9. Success metrics (required)

| Metric | W-Cloud-Preview target | W-Cloud-Stable target | W-Public-GA target | W-Region-Fan-Out target |
|---|---|---|---|---|
| Regions in-production | 4 (KR-Seoul1, JP-Tokyo1, US-Virginia1, EU-Frankfurt1) | 6+ | 9+ | 14+ |
| Cells per region | ≥ 3 | ≥ 6 | ≥ 12 | ≥ 12 + sovereign cells |
| Public Cloud API availability | n/a (preview, internal-only ≥ 99.9%) | 99.95% | 99.99% | 99.99% per region |
| Object store durability | 11-nines (target) | 11-nines | 11-nines | 11-nines |
| Provision-to-running p95 (instance) | ≤ 90 s | ≤ 60 s | ≤ 60 s | ≤ 60 s |
| STS issuance p99 | ≤ 200 ms | ≤ 100 ms | ≤ 100 ms | ≤ 100 ms |
| Audit-chain emission completeness | ≥ 99% | 100% | 100% | 100% |
| Foundry agent-operated mutators | ≥ 5 capabilities live | ≥ 20 | ≥ 50 | per-region capability surface complete |
| Cross-region-replication lag p95 (when policy allows) | ≤ 60 s | ≤ 30 s | ≤ 15 s | per-pair SLO |
| Time to onboard a new region | n/a (parallel onboard during preview) | ≤ 6 weeks | ≤ 4 weeks | ≤ 2 weeks |
| Per-region gross margin | n/a | ≥ 30% | ≥ 50% | ≥ 50% |
| Regulator-equivalent attestations | KR CSAP audit underway | KR CSAP + K-ISMS-P + JP ISMAP + US SOC-2 + EU C5 | per-pack regulator audited | per-pack |
| Cross-axis contract violations on `main` | 0 | 0 | 0 | 0 |

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Cell-isolation evidence insufficient at audit time | Catastrophic | Per-cell tenant-density class enforced at provision; per-cell network ACL blocks cross-cell traffic by default; Cedar policy denies cross-cell IAM by default; cell-level chaos drills monthly | Cloud + Security |
| Object backend (Ceph/Seaweed) durability claim does not survive AZ loss | Catastrophic | EC 3+1 across AZs by default; per-bucket scrub schedule; quarterly forced AZ-isolation drill | Cloud-storage team |
| Cross-region replication leaks under residency-strict policy | Catastrophic | Per-bucket `allowed_data_classes` × residency policy enforced at replication ingress; audit-chain on every cross-region ship | Cloud + Privacy |
| KVM / hypervisor escape | Catastrophic | Firecracker for high-blast-radius workloads; per-cell host hardening; CVE-driven restart-policy; supply-chain attestation per host image (ADR-0039) | Cloud + Security |
| AGPL backend (MinIO) accidentally adopted | High | License-policy gate (`oya-governance-license`) hard-fails any MinIO link or vendoring | Cloud + Foundry |
| Foundry mutator misuse (e.g. `cloud.iam.publish` over-broad) | High | Autonomy-ceiling-bound; T3 required for IAM mutation by default; per-mutator scoped capability schema; audit-chain on every invocation | Cloud + Foundry + Governance |
| KR CSAP attestation slipping past target | High | Parallel KR-pack workstream; contract with KR auditor signed pre-W-Cloud-Preview; controls evidenced via Foundry agents (HIPAA/KISA pattern per DESIGN §3) | Cloud-KR-pack team |
| Multi-AZ failover not actually exercised | High | Argo Rollouts (ADR-0050) progressive delivery + monthly forced AZ-failover drill; metric-gated rollback validated quarterly | Cloud + SRE |
| FinOps unit economics red at GA | High | Per-region per-tier rate-card with margin gate; per-tenant cost surfacing forces tenant-side optimization; cloud-team budget alerts on internal cost-of-revenue | Cloud + FinOps |
| Direct interconnect site lock-in | Medium | Per-pack `InterconnectPartner` impls; multi-IXP at each major region (KR: KIX+KINX; JP: JPIX+BBIX) | Cloud-network team |
| Tax-invoice format drift (regulator updates) | Medium | Versioned `TaxInvoiceFormatter` per pack; tax-pack changelog reviewed quarterly | Cloud-billing + regional-pack maintainers |
| Service mesh (Istio Ambient) maturity | Medium | Linkerd available as fallback (ADR-0044 preserved as drop-in); per-cell mesh upgrade gated on metric stability | Cloud + SRE |

## 11f. User experience (required for user-facing surfaces)

| Field | Content |
|---|---|
| `ux_personas_ref` | Cloud operators, tenant admins, SRE-on-call, compliance reviewers from §2. |
| `accessibility_coverage` | WCAG 2.2 AA; topology, deployment, cost, and incident surfaces require keyboard/table mirrors. |
| `responsive_breakpoints` | tablet / desktop / wide-desktop; mobile is read-only incident status only. |
| `internationalization_scope` | locale-aware-dynamic; ko-KR and en-US launch gates for operator copy and error remediation. |
| `design_system_components_used` | `CloudCellTopologyMap`, `OpsDeploymentStatusPanel`, `AuditEvidenceTimeline`, `PolicyDisclosureBanner`. |
| `journey_critical_paths` | find tenant cell route < 60s; inspect canary/rollback state < 90s; export compliance evidence < 5m. |
| `error_state_coverage` | drift, stale telemetry, failed OpenTofu plan, blocked secret, canary pause, rollback running. |
| `offline_behavior` | no mutating offline ops; cached read-only topology/evidence clearly marked stale. |
| `keyboard_navigation_coverage_pct` | 100 for topology table mirror, deployment actions, rollback controls, evidence export. |
| `loading_state_coverage` | skeleton topology/table rows and determinate plan/apply progress; spinner-only states forbidden. |

## 11g. Frontend components (required for products with rendered UI)

| Component | Source | Variants | Tested-at-breakpoint |
|---|---|---|---|
| `CloudCellTopologyMap` | `$ref:specs/design-system/cloud-cell-topology-map.json` | region-overview / tenant-cell-route / canary-rollout / incident-mode | tablet / desktop / wide-desktop |
| `OpsDeploymentStatusPanel` | `$ref:specs/design-system/ops-deployment-status-panel.json` | plan-preview / apply-running / canary / rollback / drift-detected | tablet / desktop / wide-desktop |
| `AuditEvidenceTimeline` | `$ref:specs/design-system/audit-evidence-timeline.json` | compliance-control / release-evidence / changeset-provenance | tablet / desktop |
| `PolicyDisclosureBanner` | `$ref:specs/design-system/policy-disclosure-banner.json` | audit-access / expired-policy / requires-second-approver | tablet / desktop |

## 11. Open questions

1. **Cloud axis pricing model at public-GA**: per-resource-hour AWS-style, or per-tenant-bundle Connect-style? (Same as PRD §8.) Default proposed: per-resource-hour with committed-use discounts.
2. **encryption-BYOK / HYOK at preview**: tenant-key escrow with KCMVP HSM as default for KR-pack; deferred for non-KR packs until W-Cloud-Stable. Confirm at council.
3. **Bare-metal lease**: managed (Oyatie operates the bare-metal as a service) or unmanaged (tenant gets root)? Default proposed: managed-by-default with unmanaged opt-in.
4. **Air-gapped sovereign cell** (per ADR-0050 air-gap-first profile): which packs include it from preview (KR public sector? JP government? UAE ADGM?) vs. defer to GA. Council pending.
5. **Marketplace ISV onboarding**: same gate as `oya-saas-marketplace-kernel` (Plugin trust tiers ADR-0036), or separate cloud-app trust ladder?

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | Cloud axis runs on leased racks + colo; bare-metal is product, not internal capability | PRD §1.3 non-goal |
| 2026-05-09 | Canonical-architecture + regional-pack from day one | DESIGN §12; KR is one pack among many, not the default |
| 2026-05-09 | OpenTofu chosen over Terraform | ADR-0050; license drift on Terraform forced switch |
| 2026-05-09 | Istio Ambient pulled forward; Linkerd held as fallback | ADR-0044 supersedes ADR-0044 |
| 2026-05-09 | MinIO REJECTED | AGPL-3 incompatible with product code; Ceph + SeaweedFS chosen |

## 13. Sources scanned

- [`docs/PRD.md`](../../PRD.md)
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §5, §10, §12
- [`docs/PRIVACY-PROGRAM.md`](../../PRIVACY-PROGRAM.md) §2.2.1, §2.2.2, §3.4
- [`docs/GLOSSARY.md`](../../GLOSSARY.md) §1-7
- `/Users/jasonlee/oyatie/docs/raw/greenfield-cloud.md` (299 leaves: A.1 VM, A.2 K8s, A.3 Functions, A.4 Bare-metal, A.5 GPU, A.6 Edge; B.1 Object, B.2 Block, B.3 File, B.4 Archive, B.5 Database, B.6 Backup; C.1 VPC, C.2 LB, C.3 DNS, C.4 CDN, C.5 Interconnect, C.6 DDoS, C.7 Mesh; D IAM; E Regions; F Billing+Marketplace; G Observability; H FinOps; I Clean-arch; J KR-launch)
- ADR-0021 (OCI A1 launch), ADR-0022 (GitOps), ADR-0044 (Cloud-native infra), ADR-0044 (Data tier matrix), ADR-0013 (Envoy gateway), ADR-0045 (ClickHouse), ADR-0044 (Harbor), ADR-0043 (OpenBao), ADR-0045 (VictoriaMetrics), ADR-0046 (Kafka eventing), ADR-0045 (Cassandra gated), ADR-0045 (Citus), ADR-0047 (Vector store gated), ADR-0042 (Mimir gated), ADR-0035 (Temporal gated), ADR-0045 (Iceberg gated), ADR-0044 (OCI Always Free inventory), ADR-0044 (Istio Ambient), ADR-0050 (Argo Rollouts), ADR-0039 (Supply chain Trivy/Cosign/SBOM), ADR-0012 (Enterprise cloud readiness), ADR-0042 (GitOps baseline), ADR-0015 (Flat crates), ADR-0003 (Trust framework), ADR-0021 (Product control plane), ADR-0050 (Data + AI governance), ADR-0040 (Portfolio + capital allocation), ADR-0017 (Roadmap wave integration), ADR-0050 (Multi-cloud + on-prem IaC + air-gap)
- Hyperscaler benchmark references: AWS Well-Architected Framework, AWS Builders Library fallback guidance, AWS shuffle-sharding / cell isolation patterns, AWS S3 Replication Time Control, AWS IAM data-perimeter resource policies, Azure Well-Architected Framework, Google SRE 4 golden signals, Google SRE error-budget burn-rate alerting.
- Detailed audit artifact: [`evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json`](../../../evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json)

---

## Competitive benchmark

The detailed Cloud benchmark is in §14 and in `evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json`. It compares AWS, Azure, Google SRE/GCP, and adjacent Oyatie core-product patterns against cell isolation, idempotency, backpressure, observability, hot-key resilience, stuck-loop circuit breakers, insecure-by-default avoidance, and vendor lock-in avoidance.

## 14. Hyperscaler pattern audit (2026-05-17)

> Auditor: agent/audit-cloud-hyperscaler — cross-referenced against AWS Well-Architected Framework (6 pillars), Google SRE 4 golden signals, Azure Well-Architected Framework (5 pillars), and AWS Builders Library cell-isolation / fallback-avoidance patterns.

Score honesty note: this section is a design benchmark. `Strong` requires a named enforcement mechanism such as a CI lane, test, or concrete kernel/domain contract; prose-only architecture is capped at `Adequate` until an enforcement lane lands.

### 14.1 Patterns confirmed present (adopt with evidence)

| Pattern | Where it lives in this PRD | Hyperscaler reference |
|---|---|---|
| **Cell isolation / blast radius** | §4.2 `Cell` aggregate; `tenant_density: TenantDensityClass`; per-cell Postgres cluster; cell-local Envoy header `x-oya-cell`; per-cell chaos drills monthly (§10) | AWS shuffle-sharding + cell-based failure isolation; Azure mission-critical zone segmentation |
| **Multi-region active / active** | §3.1 W-Cloud-Preview: KR-Seoul1 + JP-Tokyo1 + US-Virginia1 + EU-Frankfurt1 in parallel; §5.3 streaming replication 3-AZ + cross-region read-only mirror | AWS multi-AZ + multi-region; Azure reliability pillar — zone + region redundancy |
| **Capacity reservation** | §6 `cloud.capacity.rebalance` Foundry capability (≤ T2 autonomy); per-cell utilization struct; batch rebalance every 5 s | AWS Reserved Instances / capacity reservations pattern; GCP committed-use contracts |
| **IAM least-privilege** | §4.2 Cedar policies; §5.1 `IamRole.max_session_duration_sec`; `cloud.iam.audit-narrow` capability proposes narrowing from access logs; per-mutator scoped capability schema; default-deny cross-cell IAM | AWS IAM least-privilege pillar; Azure zero-trust IAM |
| **FinOps: cost reporting + right-sizing** | §4.2 `oya-cloud-finops-api`; `cloud.cost.recommend` (≤ T1) identifies idle resources; per-tenant cost breakdown in Cloud Console; per-region gross-margin gate ≥ 50% at GA | AWS Cost Explorer + Trusted Advisor; Azure Cost Management + Advisor |
| **4 golden signals observability** | §4.2 `oya-cloud-observability-kernel` + `MetricStream`, `LogStream`, `TraceStream`; §4.3 SLO targets on every API surface (latency p99 + availability %); §6 hot-path benchmarks for STS / Cedar / object GET | Google SRE 4 golden signals: latency + traffic + errors + saturation |
| **Data perimeter / residency** | §5.1 `ResidencyClass` on every `Resource` + `Bucket`; per-bucket `allowed_data_classes`; per-cell `allowed_residency`; cross-region replication blocked unless residency policy explicitly permits | AWS data-perimeter controls; Azure data residency commitments |
| **KMS / encryption-BYOK / HYOK** | §3.1 W-Cloud-Preview encryption-BYOK/HYOK KMS; §4.2 `oya-cloud-kms-api`; `EncryptionMode: sse | sse-kms | byok | hyok`; per-pack HSM (KCMVP / FIPS-140-3 / Common Criteria EAL-4+); `cloud.kms_key_used.v1` audit event (indefinite retention) | AWS KMS + CloudHSM; Azure Key Vault + Managed HSM; GCP Cloud KMS + Cloud HSM |
| **Supply-chain attestation** | §4.2 `oya-cloud-supply-chain-app` (Cosign + Trivy + SBOM per ADR-0039); `oya-governance-license` hard-gate; per-host image supply-chain attestation (§10) | AWS SLSA / Sigstore; Google Binary Authorization; Azure Defender for DevOps |
| **Idempotency on every mutation** | §6 `Idempotency-Key` header on every mutating REST + gRPC; outbox deduplication 24 h; cloud-init deduped on `instance_id`; `idempotency_key` on every billing event | AWS SDK retry-with-idempotency; Temporal idempotent activities |
| **Backpressure / rate limiting** | §6 capacity-bound 429 + `Retry-After` at cell; observability ingest sheds at 95% lag; LB control loop slows under metric pressure | AWS throttling + token-bucket; Azure APIM throttling; Google SRE saturation signal |
| **Audit chain on every mutation** | §5.6 full audit-chain emission contract (10 regulated operations); `prev_hash` chained; per-tenant signed S3 stream; indefinite KMS-use retention | AWS CloudTrail immutable logs; Azure Monitor audit logs; GCP Cloud Audit Logs |
| **Vendor lock-in avoidance** | §8 every external dep behind adapter trait (`KVM`, `Firecracker`, `Ceph/SeaweedFS`, `OVN`, `FRR`, `OpenTofu`) — never imported directly into product crates; `oya-governance-license` enforces boundaries | AWS portability via IaC; Azure WAF vendor-neutrality |
| **Fallback-as-failover (not silent fallback)** | §6 Argo Rollouts metric-gated rollback; Istio Ambient with Linkerd as exercised fallback (ADR-0044) — both paths continuously exercised | AWS Builders Library: avoid fallback; convert to failover via continuous exercise |

### 14.2 Anti-patterns explicitly avoided

| Anti-pattern | How the PRD avoids it | Hyperscaler reference |
|---|---|---|
| **Shared global mutable state across tenants** | Per-tenant Postgres RLS + per-cell Postgres cluster; STS sessions in Redis sharded by `session_id`; `allowed_data_classes` enforced at replication ingress | AWS shuffle-sharding avoids noisy-neighbor; Azure mission-critical bulkhead |
| **Silent fallback to slower path** | §10 risk: Ceph/Seaweed EC 3+1 by default; forced AZ-isolation drill quarterly; Argo Rollouts blocks auto-merge on metric regression | AWS Builders Library: fallback causes amplified outages (2001 retail example) |
| **Unbounded fanout on single tenant** | §6 bulk endpoint max 1 000 resources / 100 000 objects; cursor pagination max 10 000; batch dispatch windows (5 s / 60 s / 1 s) | AWS SQS backpressure; GCP Pub/Sub flow control |
| **Insecure by default** | IMDSv2-only per greenfield A11 (§3.1); TLS everywhere via `rustls`; IPv6 from day 1; OpenBao for secrets; per-cell host hardening (§10) | AWS security pillar: secure by default; Azure zero-trust |
| **No retry budget** | `Retry-After` in 429 responses; observability dead-letter at 95% lag; `max_session_duration_sec` on STS | AWS retry with exponential backoff; Google SRE error budget |
| **Vendor lock-in via direct dependency** | MinIO REJECTED (AGPL-3); every hypervisor/network/storage dep at process boundary only; `cargo-deny` enforces denylist | AWS portability; Azure multi-cloud guidance |
| **Opaque cost allocation** | Per-resource-hour metering via `MeterEvent`; per-tenant breakdown in FinOps console; per-region gross-margin gate | AWS Cost Allocation Tags; Azure cost management per resource group |
| **Manual operational runbooks** | Foundry `cloud.capacity.rebalance` + `cloud.cost.recommend` + `cloud.iam.audit-narrow` automate repetitive ops; Argo Rollouts automates canary/rollback | AWS operational excellence: automate runbooks; Azure OE: automation over manual |

### 14.3 Gaps identified — recommended additions

The following patterns from hyperscaler frameworks are partially or entirely absent from the current PRD and should be added in the next revision:

| Gap | Severity | Recommended addition | Reference |
|---|---|---|---|
| **Explicit error-budget policy** | High | Add an error-budget burn-rate alert contract: per-SLO burn-rate thresholds (fast burn: 5× in 1 h triggers page; slow burn: 1× in 6 h triggers ticket); link to `oya-cloud-observability-kernel` | Google SRE error-budget burn-rate alerting |
| **Hot-key / hot-partition resilience** | High | Add per-tenant request-rate quotas at the cell-routing layer (not just bulk endpoint caps); add shuffle-sharding on `(tenant_id, resource_kind)` for IAM STS hot paths | AWS shuffle-sharding; DynamoDB adaptive capacity |
| **Structured chaos engineering programme** | Medium | §10 mentions "monthly AZ-isolation drill" and "quarterly forced AZ-failover" but does not specify the blast-radius assertion framework. Add: per-cell chaos manifests (Gremlin / Chaos Mesh) with automated green/red verdicts before GA | AWS GameDay / FIS; Google DiRT |
| **Capacity pre-warming SOP** | Medium | Add documented pre-warming runbook for new cell bring-up: `cloud.capacity.rebalance` should warm N% head-room before a region goes public-preview; tie to `cloud.region.register` mutator sequence | AWS capacity reservations pre-warm pattern |
| **Explicit saturation SLO for each resource dimension** | Medium | §6 lists hot-path latency benchmarks but not saturation thresholds (e.g., CPU, memory, network, IOPS per cell). Add `CellCapacity.saturation_slo_pct` field and reject provision when any dimension > threshold | Google SRE saturation signal |
| **Cross-region replication lag SLO tightening** | Medium | §9 sets p95 ≤ 60 s (preview) → 15 s (GA). AWS S3 Cross-Region Replication delivers < 1 min 99.99% of objects within 15 min. Expose a `replication_class: standard | accelerated` flag on `Bucket` matching S3 RTC semantics | AWS S3 Replication Time Control (RTC) |
| **Data perimeter — resource-based policy enforcement** | Medium | The PRD describes Cedar policies on principals but does not specify resource-based policies (analogous to AWS S3 bucket policies). Add `ResourcePolicy` to `Bucket` and `KmsKey` aggregates so resources can independently enforce who can access them regardless of principal-side roles | AWS data perimeter: resource-based policies; AWS IAM resource policies |
| **Control-plane / data-plane separation SLOs** | Low | SLO tables in §4.3 mix control and data planes but treat them as a single availability number. AWS and GCP independently SLA the control plane (create/delete) vs data plane (read/write existing resources). Split the availability target for `Cloud API v1` into control-plane availability (create/delete: 99.99%) vs data-plane availability (object GET/PUT: 99.999%) | AWS service SLA separation; GCP API availability tiers |
| **Graceful degradation contract on control-plane outage** | Low | Specify that all data-plane operations (object GET, instance network, running VMs) continue during a control-plane outage. Currently implied but not contractually stated in §4.3 | AWS: data plane continues when control plane is degraded |

### 14.4 Industry-standard adoption summary

| Dimension | Score | Evidence |
|---|---|---|
| Cell isolation | **Adequate** | Cell aggregate with density class, per-cell Postgres, per-cell IAM, monthly chaos drills; dedicated enforcement test still missing |
| Capacity reservation | **Adequate** | `cloud.capacity.rebalance` capability; lacks pre-warming SOP |
| Multi-region failover | **Adequate** | 4 regions in parallel at preview; Argo Rollouts progressive delivery; AZ-failover drills; end-to-end failover drill evidence still missing |
| IAM least-privilege | **Adequate** | Cedar policies; `audit-narrow` capability; default-deny cross-cell; resource-policy enforcement still missing |
| FinOps | **Adequate** | Per-resource-hour metering; `cloud.cost.recommend`; gross-margin gate; no dedicated FinOps CI lane yet |
| Observability (4 golden signals) | **Adequate** | Latency + availability SLOs present; explicit saturation SLO per cell dimension missing |
| Data perimeter | **Adequate** | Residency classes + `allowed_data_classes`; resource-based policies not yet specified |
| KMS / secrets | **Adequate** | encryption-BYOK/HYOK; per-pack HSM; OpenBao; `cloud.kms_key_used.v1` audit; HYOK integration evidence still missing |
| Supply-chain attestation | **Strong** | Cosign + Trivy + SBOM; license gate; per-host image attestation |
| Vendor lock-in avoidance | **Strong** | All external deps at adapter/process boundary; `cargo-deny` denylist |
| Error-budget policy | **Gap** | No burn-rate alert contract defined |
| Hot-key resilience | **Gap** | Bulk caps exist; shuffle-sharding on hot IAM/STS paths not specified |

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `cloud` | `axis-cloud` | scope, contract, capability | monthly | PRD.md, DESIGN.md, PRIVACY-PROGRAM.md, GLOSSARY.md |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` — add `cloud` row
- `machine-readable/catalog.json` — pointer at this PRD path
- `machine-readable/contracts.json` — every cross-axis contract row in §4.5
- `machine-readable/risks.json` — risks from §10
- `machine-readable/glossary.json` — Region, AZ, Cell, ResourceKind canonical terms

## Validation checks

`oya-governance-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10

---

## Hero Surface Substance Bar Addendum - Cloud

This addendum deepens the Cloud product PRD to the hero-surface bar. It preserves the existing AWS-class axis spec above and adds the missing product-documentation layer for personas, JTBDs, stories, surface maps, data, Cedar, workflow, intelligence, pack overlays, SLOs, ADR-0263 telemetry, migration, tier deltas, competitive positioning, roadmap, cross-product handoffs, and recovery behavior.

## Vision

Cloud exists so oyatie can run its own platform and sell a sovereign, audit-chained, agent-operated cloud to tenants that cannot accept generic hyperscaler control boundaries. The product is for regulated enterprises, sovereign buyers, startups, internal oyatie product teams, marketplace ISVs, and Foundry agents that need compute, storage, network, IAM, KMS, observability, and billing from one tenant model. The timing matters because Foundry, workplace, ERP, marketplace, and regional packs cannot honestly claim hyperscaler maturity unless the substrate they run on has region, cell, data perimeter, billing, and evidence behavior documented at the product level.

## Personas

- Primary: CISO Yuki Park, KR enterprise security owner; MASTER-ROSTER row 32.
- Primary: CFO Helena Brandt, cloud cost and committed-use buyer; MASTER-ROSTER row 26.
- Primary: CTO Diego Vargas, platform buyer and migration sponsor; MASTER-ROSTER row 28.
- Primary: Marcus Chen, multinational executive buyer; MASTER-ROSTER row 2.
- Primary: Diana Reyes, external auditor and regulator-facing reviewer; MASTER-ROSTER row 7.
- Secondary: Internal Oyatie engineer operating product workloads.
- Secondary: Foundry agent invoking cloud mutator capabilities.
- Secondary: Marketplace ISV publishing cloud-native listings.
- Secondary: Cloud platform SRE handling cell incidents.
- Secondary: Regional compliance officer validating CSAP, ISMAP, DORA, FedRAMP, and LGPD overlays.

## Jobs-to-be-Done

### JTBD-CLOUD-01 - Provision tenant compute with evidence
- Situation: Diego provisions a regulated workload in KR-Seoul.
- Acceptance: VM, image, subnet, KMS key, IAM role, and audit event are linked.
- Acceptance: provisioning fails closed when region, cell, or pack policy is incompatible.

### JTBD-CLOUD-02 - Store regulated objects under a data perimeter
- Situation: Yuki stores encrypted exports for a strict-KR tenant.
- Acceptance: bucket policy, KMS key policy, resource policy, residency class, and audit export are visible.
- Acceptance: cross-region replication is denied unless residency policy permits.

### JTBD-CLOUD-03 - Create a VPC without hidden shared tenancy
- Situation: Marcus's enterprise creates VPCs for subsidiaries.
- Acceptance: subnets, route tables, firewall rules, NAT, and load balancers are scoped to tenant, region, and cell.
- Acceptance: default route to non-approved region is rejected.

### JTBD-CLOUD-04 - Prove cost and margin per tenant
- Situation: Helena reviews cloud spend and internal product unit economics.
- Acceptance: MeterEvent, rate card, budget alert, invoice, tax treatment, and margin attribution are linked.
- Acceptance: cost anomaly opens FinOps investigation workflow.

### JTBD-CLOUD-05 - Operate a cell failure
- Situation: an AZ power zone degrades.
- Acceptance: cell health, workload placement, failover eligibility, SLO burn, and customer notification are visible.
- Acceptance: recovery emits cell-failover audit events and runbook evidence.

### JTBD-CLOUD-06 - Publish a managed service
- Situation: an internal team publishes managed Postgres or Kafka.
- Acceptance: service catalog entry declares SLO, backup, KMS, tenancy, scaling limits, and rollback.
- Acceptance: service cannot publish without runbook and telemetry contract.

### JTBD-CLOUD-07 - Let Foundry operate cloud safely
- Situation: Foundry agent rebalances capacity or narrows IAM.
- Acceptance: capability, autonomy tier, Cedar decision, blast radius, and rollback path are explicit.
- Acceptance: human approval is required for destructive T0/T1 actions.

### JTBD-CLOUD-08 - Migrate from a hyperscaler
- Situation: a tenant leaves AWS, Azure, GCP, Naver Cloud, or Oracle Cloud.
- Acceptance: inventory import maps identity, network, storage, compute, billing, and audit evidence.
- Acceptance: unsupported resource types are listed with migration blockers.

### JTBD-CLOUD-09 - Certify a sovereign region
- Situation: compliance prepares KR CSAP or EU DORA evidence.
- Acceptance: region pack shows controls, KMS mode, cell tier, logging, retention, auditor export, and exceptions.
- Acceptance: unresolved evidence gaps block region GA.

### JTBD-CLOUD-10 - Run marketplace cloud app install
- Situation: ISV app provisions tenant resources.
- Acceptance: install declares required resources, IAM scope, network egress, billing meters, and teardown plan.
- Acceptance: plugin cannot create resources outside declared scope.

## User Stories

### Story CLOUD-HS-001 - Region Catalog
As a cloud buyer, I want region status by jurisdiction so that workload placement is legal and available.
Pass: each region shows state, packs, cell tiers, SLO, and evidence status.
Pass: hidden or planned regions are not selectable for production workloads.

### Story CLOUD-HS-002 - Tenant Cell Binding
As a tenant admin, I want my workloads bound to a cell class so that isolation promises are enforceable.
Pass: tenant cell binding shows region, cell tier, density class, and change history.
Pass: binding changes require policy approval and audit event.

### Story CLOUD-HS-003 - VM Provision
As Diego, I want VM create to include image, flavor, subnet, key, role, and KMS choice so that compute is complete.
Pass: create command returns instance id and evidence id.
Pass: missing KMS key blocks regulated workloads.

### Story CLOUD-HS-004 - Kubernetes Cluster
As platform engineer, I want managed Kubernetes with tenant-scoped control plane so that apps can run without unmanaged clusters.
Pass: cluster includes version, node pools, network policy, backup, and upgrade channel.
Pass: unsupported version cannot be created.

### Story CLOUD-HS-005 - Function Deploy
As developer, I want a function deployment with cold-start budget so that event workloads are predictable.
Pass: function declares runtime, timeout, memory, IAM role, and event trigger.
Pass: invocation emits metric and audit receipt.

### Story CLOUD-HS-006 - Object Bucket Policy
As CISO Yuki, I want resource-based bucket policy so that access is not only principal-side.
Pass: bucket policy can deny even when principal role permits.
Pass: bucket policy change invalidates policy cache.

### Story CLOUD-HS-007 - KMS Key Lifecycle
As compliance officer, I want key create, rotate, disable, and shred with evidence so that data lifecycle is provable.
Pass: key action emits policy decision and audit event.

### Story CLOUD-HS-008 - VPC Build
As network engineer, I want VPC, subnets, routes, security groups, and load balancers in one workflow so that topology is consistent.
Pass: every subnet has region, AZ, CIDR, route table, and policy.
Pass: overlapping CIDR is rejected.

### Story CLOUD-HS-009 - DNS Zone
As tenant admin, I want DNS zone creation with DNSSEC and audit so that public endpoints are governed.
Pass: zone includes owner, nameserver set, DNSSEC state, and change evidence.
Pass: destructive record delete requires confirmation and audit.

### Story CLOUD-HS-010 - Load Balancer
As app owner, I want L4 and L7 load balancers with health checks so that traffic moves safely.
Pass: health check, TLS policy, backend set, and WAF mode are visible.
Pass: unhealthy all-backend state opens incident.

### Story CLOUD-HS-011 - Direct Interconnect
As enterprise buyer, I want private link provisioning so that regulated traffic avoids public internet.
Pass: interconnect shows port, BGP session, route filters, and SLA.
Pass: route leak is denied and alerted.

### Story CLOUD-HS-012 - Managed Postgres
As application owner, I want managed Postgres with backups, encryption, and maintenance window so that data services are standard.
Pass: DB create declares version, size, HA mode, backup policy, and KMS key.
Pass: no DB can run without backup policy.

### Story CLOUD-HS-013 - Managed Redis
As app owner, I want managed Redis with eviction policy and persistence mode so that cache behavior is predictable.
Pass: memory class, eviction, AOF/RDB, and network policy are visible.
Pass: unsafe public exposure is denied.

### Story CLOUD-HS-014 - Managed Kafka
As platform team, I want event streaming with topic governance so that workloads can publish reliably.
Pass: topic has retention, partition, schema, ACL, and quota.
Pass: schema-breaking publish is blocked.

### Story CLOUD-HS-015 - Observability Workspace
As SRE, I want metrics, logs, traces, and audit in one tenant workspace so that incidents are diagnosable.
Pass: every resource links to dashboards and alert routes.
Pass: missing telemetry blocks GA service catalog publish.

### Story CLOUD-HS-016 - Cost Explorer
As Helena, I want spend by tenant, product, region, tag, and resource so that margin and chargeback are clear.
Pass: every line item links to MeterEvent and rate card.
Pass: untagged spend appears in exception queue.

### Story CLOUD-HS-017 - Budget Alert
As finance owner, I want spend threshold alerts before overrun so that action happens early.
Pass: alert has threshold, forecast, owner, and workflow action.
Pass: acknowledged alert emits evidence.

### Story CLOUD-HS-018 - Invoice Export
As buyer, I want tax-compliant invoice export per jurisdiction so that finance can pay.
Pass: invoice includes tax pack, region, currency, usage, and signing evidence.
Pass: pack-specific missing fields block final invoice.

### Story CLOUD-HS-019 - IAM Role Narrowing
As CISO, I want access analyzer recommendations so that least privilege improves.
Pass: recommendation cites access logs and policy diff.
Pass: applying change requires owner approval.

### Story CLOUD-HS-020 - STS Session
As developer, I want short-lived sessions so that long-lived keys are not used.
Pass: STS issue p99 <= 100 ms and session has scope, expiry, and actor.
Pass: session cannot exceed role max duration.

### Story CLOUD-HS-021 - Secret Reference
As engineer, I want secret references instead of raw secrets so that logs never contain credentials.
Pass: API returns secret_ref only.
Pass: serializer redacts secret-like payloads.

### Story CLOUD-HS-022 - Image Signing
As platform owner, I want signed images only so that supply chain risk is bounded.
Pass: image launch checks signature, SBOM, vulnerability status, and provenance.
Pass: unsigned image launch is denied.

### Story CLOUD-HS-023 - Snapshot Restore
As app owner, I want restore preview before replacing data so that rollback is safe.
Pass: restore has target, source snapshot, dry-run result, and RPO estimate.
Pass: destructive restore requires approval.

### Story CLOUD-HS-024 - Cross-Region Replication
As regulated tenant, I want replication that respects residency so that durability does not violate law.
Pass: policy explains allowed regions.
Pass: illegal target region is denied.

### Story CLOUD-HS-025 - Cell Health Page
As SRE, I want a cell health page so that incidents show blast radius.
Pass: page lists affected resources, tenants, SLO burn, and failover eligibility.
Pass: customer notification is generated from affected tenant list.

### Story CLOUD-HS-026 - Capacity Rebalance
As Foundry agent, I want capacity rebalance under autonomy limits so that hot cells cool down safely.
Pass: plan lists moves, risk, rollback, and approvals.
Pass: destructive moves require human approval.

### Story CLOUD-HS-027 - Service Catalog Publish
As internal service owner, I want to publish managed services to the cloud catalog so that tenants can consume supported offerings.
Pass: publish requires SLO, telemetry, runbook, backup, KMS, and billing meter.
Pass: missing field blocks publish.

### Story CLOUD-HS-028 - Marketplace App Install
As tenant admin, I want marketplace app install to provision resources safely.
Pass: install plan declares compute, storage, IAM, network, billing, and teardown.
Pass: app cannot exceed declared scope.

### Story CLOUD-HS-029 - Regulator Evidence Export
As Diana, I want region and tenant evidence export so that certification can be reviewed.
Pass: export includes controls, events, KMS mode, cell tier, SLO, and exceptions.
Pass: export has signed hash.

### Story CLOUD-HS-030 - Hyperscaler Import
As migration owner, I want AWS or Azure inventory imported so that migration scope is known.
Pass: import maps resource types and lists unsupported resources.
Pass: imported credentials are secret references only.

## Surface Map

### Surface CLOUD-SURF-01 - Cloud Console Home
```
+ Region + Cell + Spend + Incidents + Compliance +
| KR-Seoul1 GA | cell-a hot | $42k MTD | 1 P2 | CSAP green |
+------------------------------------------------+
```

### Surface CLOUD-SURF-02 - Resource Browser
```
+ Resource + Type + Region + Cell + Policy + Cost +
| vm-818 | instance | KR-Seoul1 | c-a | role:web | $19.44 |
+------------------------------------------------+
```

### Surface CLOUD-SURF-03 - Region Control Room
```
+ Region + AZ + Cell + SLO burn + Evidence + Action +
| KR-Seoul1 | a | cell-07 | 0.7% | sealed | rebalance |
+------------------------------------------------+
```

### Surface CLOUD-SURF-04 - IAM Policy Editor
```
+ Principal + Action + Resource + Tier guard + Decision preview +
+------------------------------------------------+
```

### Surface CLOUD-SURF-05 - Network Builder
```
+ VPC + Subnets + Routes + SG + LB + DNS +
| vpc-prod | 3 subnets | 2 routes | 6 SG | lb-web | zone ok |
+------------------------------------------------+
```

### Surface CLOUD-SURF-06 - Storage Bucket Detail
```
+ Bucket + KMS + Residency + Policy + Replication + Audit +
| b-payroll | key-kr | strict_kr | locked | disabled | 100% |
+------------------------------------------------+
```

### Surface CLOUD-SURF-07 - Cost Explorer
```
+ Tenant + Product + Region + Resource + Tag + Cost + Margin +
| t-42 | workplace | KR | k8s-node | cost:center:hr | 812 | 31% |
+------------------------------------------------+
```

### Surface CLOUD-SURF-08 - Marketplace Install Plan
```
+ App + Compute + Storage + IAM + Network + Meters + Teardown +
| analytics-pro | 2 fn | 1 bucket | read-only | egress deny | 3 | yes |
+------------------------------------------------+
```

### Surface CLOUD-SURF-09 - Evidence Export
```
+ Control + Event count + KMS mode + Cell tier + Exceptions + Hash +
| CSAP-LOG-01 | 19022 | KCMVP | dedicated | 0 | sha256:... |
+------------------------------------------------+
```

### Surface CLOUD-SURF-10 - Migration Import
```
+ Source + Resources + Mapped + Unsupported + Risk + Next +
| AWS acct 123 | 820 | 744 | 76 | medium | network plan |
+------------------------------------------------+
```

## Data Model

### Entity CLOUD-ENT-01 - CloudAccount
- Fields: account_id, tenant_id, billing_account_id, root_policy_id, region_allowlist, status.
- Relationship: owns Resource, Budget, Invoice, IamPrincipal.
- Invariant: account has exactly one tenant root.

### Entity CLOUD-ENT-02 - Region
- Fields: region_code, display_name, status, pack_set, data_residency_class, launch_phase.
- Relationship: contains AZ and Cell.
- Invariant: GA requires evidence export green.

### Entity CLOUD-ENT-03 - AvailabilityZone
- Fields: az_code, region_code, physical_site_ref, power_zone_set, status.
- Relationship: contains Cell.
- Invariant: zone cannot host production if status is degraded.

### Entity CLOUD-ENT-04 - Cell
- Fields: cell_id, region_code, az_code, density_class, tenant_limit, health, isolation_tier.
- Relationship: hosts ResourcePlacement.
- Invariant: tenant density cannot exceed class.

### Entity CLOUD-ENT-05 - Resource
- Fields: resource_id, tenant_id, account_id, region_code, cell_id, kind, state, tags, cost_center.
- Relationship: parent for compute, storage, network, database, and managed service resources.
- Invariant: every resource has region and tenant.

### Entity CLOUD-ENT-06 - ComputeInstance
- Fields: instance_id, image_id, flavor_id, subnet_id, role_id, kms_key_id, boot_state.
- Relationship: attaches Volume and NetworkInterface.
- Invariant: regulated instance requires signed image and KMS key.

### Entity CLOUD-ENT-07 - KubernetesCluster
- Fields: cluster_id, version, node_pool_set, control_plane_cell, upgrade_channel, backup_policy.
- Relationship: owns NodePool and ClusterAddOn.
- Invariant: unsupported Kubernetes version cannot be created.

### Entity CLOUD-ENT-08 - FunctionService
- Fields: function_id, runtime, memory_mb, timeout_ms, trigger_ref, role_id.
- Relationship: consumes EventSource and emits InvocationMetric.
- Invariant: timeout cannot exceed tier policy.

### Entity CLOUD-ENT-09 - Bucket
- Fields: bucket_id, name, region_code, kms_key_id, resource_policy_id, retention_policy, replication_policy.
- Relationship: owns ObjectVersion.
- Invariant: object write requires bucket and KMS policy allow.

### Entity CLOUD-ENT-10 - ObjectVersion
- Fields: object_id, bucket_id, version_id, checksum, size_bytes, data_class, retention_until.
- Relationship: encrypted by KmsKey.
- Invariant: retention lock blocks delete.

### Entity CLOUD-ENT-11 - BlockVolume
- Fields: volume_id, size_gib, iops_class, attached_instance_id, snapshot_policy, encryption_state.
- Relationship: attached to ComputeInstance.
- Invariant: attached volume cannot be deleted.

### Entity CLOUD-ENT-12 - Vpc
- Fields: vpc_id, cidr, region_code, route_table_set, security_group_set, dns_mode.
- Relationship: owns Subnet.
- Invariant: CIDR cannot overlap within account.

### Entity CLOUD-ENT-13 - LoadBalancer
- Fields: lb_id, protocol, listener_set, backend_set, health_check, tls_policy.
- Relationship: fronts ComputeInstance or KubernetesService.
- Invariant: public LB requires WAF and TLS policy.

### Entity CLOUD-ENT-14 - DnsZone
- Fields: zone_id, domain, dnssec_state, owner_account, record_set_hash, last_change_id.
- Relationship: has DnsRecord.
- Invariant: record delete emits signed change event.

### Entity CLOUD-ENT-15 - IamRole
- Fields: role_id, account_id, trust_policy, permission_policy_set, max_session_duration, boundary_policy.
- Relationship: assumed by Principal via StsSession.
- Invariant: max_session_duration cannot exceed tier guard.

### Entity CLOUD-ENT-16 - ResourcePolicy
- Fields: policy_id, resource_id, policy_document, version, status, last_eval_cache_bust.
- Relationship: attached to Bucket, KmsKey, Queue, or Topic.
- Invariant: resource deny overrides principal allow.

### Entity CLOUD-ENT-17 - KmsKey
- Fields: key_id, region_code, key_class, material_origin, rotation_state, shred_state.
- Relationship: encrypts Bucket, Volume, Secret, and Database.
- Invariant: disabled key blocks decrypt.

### Entity CLOUD-ENT-18 - MeterEvent
- Fields: meter_event_id, resource_id, usage_qty, unit, rate_card_id, tags, emitted_at.
- Relationship: aggregates into InvoiceLine.
- Invariant: event idempotency key prevents duplicate billing.

### Entity CLOUD-ENT-19 - BudgetAlert
- Fields: alert_id, account_id, threshold, forecast_amount, owner, state.
- Relationship: opens FinOpsWorkflow.
- Invariant: alert cannot close without owner acknowledgement.

### Entity CLOUD-ENT-20 - EvidenceExport
- Fields: export_id, scope, event_ids, control_ids, pack_set, hash, requester.
- Relationship: read by auditor.
- Invariant: export cannot be ready with missing required control.

## Cedar Policy Model

- Principal cloud::AccountRoot can delegate but cannot bypass resource policy.
- Principal cloud::PlatformEngineer can mutate region and cell only with internal tenant scope.
- Principal cloud::TenantAdmin can create resources within account quota and region allowlist.
- Principal cloud::Developer can create resources only through assigned project boundary.
- Principal cloud::BillingViewer can read cost and invoice but not resource data.
- Principal cloud::Auditor can read EvidenceExport and audit logs but not secrets.
- Principal foundry::Agent can invoke cloud mutators only through capability registry.
- Action cloud::CreateInstance requires image signed, subnet allowed, role allowed, quota available.
- Action cloud::PutObject requires bucket policy allow, KMS policy allow, retention policy check.
- Action cloud::RotateKey requires key owner and pack-specific rotation window.
- Action cloud::ReplicateObject requires source and target residency compatible.
- Action cloud::CreateVpc requires non-overlap CIDR and tenant account ownership.
- Action cloud::PublishManagedService requires SLO, telemetry, runbook, billing meter, and rollback.
- Action cloud::ExportEvidence requires auditor scope and redaction pack.
- Resource cloud::Bucket carries tenant_id, account_id, region, data_class, retention_policy.
- Resource cloud::KmsKey carries key_class, material_origin, pack_set, and shred_state.
- Resource cloud::Cell carries isolation_tier, density_class, and health.
- Resource cloud::Invoice carries account_id, tax_pack, currency, and period.

## Workflow Engine Integration

- Node CLOUD-WF-001 ResolveAccount loads tenant, account, quota, region allowlist, and pack set.
- Node CLOUD-WF-002 PolicyPreview evaluates Cedar before resource plan.
- Node CLOUD-WF-003 PlanPlacement chooses region, AZ, and cell.
- Node CLOUD-WF-004 ReserveQuota locks compute, storage, network, and budget quota.
- Node CLOUD-WF-005 ProvisionCompute creates instance, cluster, function, or bare-metal lease.
- Node CLOUD-WF-006 ProvisionStorage creates bucket, volume, snapshot, or archive vault.
- Node CLOUD-WF-007 ProvisionNetwork creates VPC, subnet, route, SG, LB, DNS, or interconnect.
- Node CLOUD-WF-008 BindIamRole attaches principal and resource policy.
- Node CLOUD-WF-009 BindKmsKey attaches key and encryption context.
- Node CLOUD-WF-010 EmitMeterStart emits billing meter start.
- Node CLOUD-WF-011 PublishTelemetryBinding creates metrics, logs, traces, and alert routes.
- Node CLOUD-WF-012 SealAuditEvidence emits ADR-0263 event and evidence id.
- Node CLOUD-WF-013 NotifyTenant sends status through console and webhooks.
- Node CLOUD-WF-014 RollbackResource releases quota and deletes partial resources.
- Node CLOUD-WF-015 RebalanceCell computes safe moves and approval path.
- Node CLOUD-WF-016 FailoverAZ shifts eligible workloads and notifies tenants.
- Node CLOUD-WF-017 BuildEvidenceExport packages controls and event ids.
- Node CLOUD-WF-018 ImportHyperscalerInventory maps source resources.
- Branch CLOUD-BR-001 denies illegal residency target.
- Branch CLOUD-BR-002 holds destructive action for human approval.
- Branch CLOUD-BR-003 degrades to read-only control plane during incident.
- Branch CLOUD-BR-004 cancels marketplace install when declared scope is exceeded.

## AI / Intelligence Integration

- ADR-0220 layer: classify capacity risk, cost anomalies, policy-risk diffs, and migration blockers.
- ADR-0255 layer 1: tenant-private retrieval cites resources, audit events, runbooks, and policy decisions.
- ADR-0255 layer 2: aggregate cloud operations learns cost and incident patterns without tenant data.
- Capability cloud.capacity.rebalance proposes moves with blast-radius citation.
- Capability cloud.iam.audit-narrow proposes least-privilege diffs from access logs.
- Capability cloud.cost.explain-anomaly explains rate, usage, tag, and workload drivers.
- Capability cloud.migration.map-resource suggests AWS/Azure/GCP to oyatie mappings.
- Capability cloud.incident.summarize-status drafts customer-safe incident status.
- Capability cloud.slo.burn-triage ranks SLO burn sources.
- Capability cloud.policy.explain-denial explains Cedar denial with safe details.
- Prohibited: Intelligence cannot bypass Cedar, retrieve secrets, rotate keys, delete resources, or approve destructive actions.

## Pack Overlays

- KR-CSAP pack activates KCMVP/KR-HSM option, strict-KR residency, CSAP evidence, and Korean invoice fields.
- EU-DORA pack activates financial resilience reports, exit evidence, and EU-only replication.
- JP-ISMAP pack activates APPI/ISMAP controls, JP region rules, and Japanese evidence export.
- US-FedRAMP pack activates IL5/6 posture, FedRAMP High controls, and US government retention.
- BR-LGPD pack activates LGPD DSR, tax invoice, and Brazil region controls.
- KSA-NDMO pack activates sovereign custody, local egress denial, and Arabic evidence export.
- UAE-TDRA pack activates UAE data controls and telecom-grade evidence requirements.
- ANZ-IRAP pack activates IRAP evidence, AU region, and privacy retention profile.
- Healthcare pack activates HIPAA redaction and key separation.
- Public-sector pack activates procurement transparency, regulator export, and audit retention.

## SLO Targets

- Region catalog read p99 <= 100 ms.
- STS session issue p99 <= 100 ms.
- Cedar policy evaluation p99 <= 5 ms.
- VM create API receipt p99 <= 500 ms.
- VM provision-to-running p95 <= 60 s.
- Kubernetes cluster create receipt p99 <= 500 ms.
- Kubernetes control plane ready p95 <= 10 min.
- Function invocation receipt p99 <= 250 ms.
- Object metadata GET p99 <= 100 ms.
- Object write durability target >= eleven nines model at GA.
- Block volume create receipt p99 <= 500 ms.
- VPC create receipt p99 <= 500 ms.
- DNS record publish p95 <= 60 s.
- Load balancer config publish p95 <= 120 s.
- KMS encrypt/decrypt authorization receipt p99 <= 100 ms.
- Meter event ingestion p99 <= 500 ms.
- Invoice generation p95 <= 48 h after period close.
- Budget anomaly detection p95 <= 15 min after meter aggregation.
- Evidence export p95 <= 10 min for one tenant-period.

## Telemetry

- EVT-CLOUD-ACCOUNT-CREATED emits account_id, tenant_id, region_allowlist, and owner.
- EVT-CLOUD-REGION-STATE-CHANGED emits region_code, old_state, new_state, and approver.
- EVT-CLOUD-CELL-BINDING-CREATED emits tenant_id, cell_id, density_class, and isolation_tier.
- EVT-CLOUD-RESOURCE-PLAN-CREATED emits plan_id, resource_kinds, quota_lock, and policy_decision_id.
- EVT-CLOUD-INSTANCE-CREATED emits instance_id, image_id, subnet_id, role_id, and kms_key_id.
- EVT-CLOUD-CLUSTER-CREATED emits cluster_id, version, node_pools, and backup_policy.
- EVT-CLOUD-FUNCTION-DEPLOYED emits function_id, runtime, trigger_ref, and timeout_ms.
- EVT-CLOUD-BUCKET-CREATED emits bucket_id, region, kms_key_id, and resource_policy_id.
- EVT-CLOUD-OBJECT-WRITTEN emits bucket_id, object_id, version_id, checksum, and data_class.
- EVT-CLOUD-VOLUME-CREATED emits volume_id, size_gib, iops_class, and kms_key_id.
- EVT-CLOUD-VPC-CREATED emits vpc_id, cidr, region, and route_table_hash.
- EVT-CLOUD-LB-PUBLISHED emits lb_id, listener_count, tls_policy, and backend_count.
- EVT-CLOUD-DNS-CHANGE-PUBLISHED emits zone_id, change_id, record_hash, and dnssec_state.
- EVT-CLOUD-IAM-POLICY-CHANGED emits policy_id, actor, diff_hash, and cache_bust_id.
- EVT-CLOUD-RESOURCE-POLICY-CHANGED emits resource_id, policy_id, version, and effect_summary.
- EVT-CLOUD-KMS-KEY-ROTATED emits key_id, rotation_version, material_origin, and approver.
- EVT-CLOUD-STS-SESSION-ISSUED emits session_id, role_id, principal_id, and expiry.
- EVT-CLOUD-METER-EVENT-INGESTED emits meter_event_id, resource_id, unit, and usage_qty.
- EVT-CLOUD-BUDGET-ALERT-OPENED emits alert_id, threshold, forecast_amount, and owner.
- EVT-CLOUD-INVOICE-GENERATED emits invoice_id, period, tax_pack, and amount_total.
- EVT-CLOUD-COST-ANOMALY-DETECTED emits anomaly_id, account_id, driver, and estimated_impact.
- EVT-CLOUD-CELL-HEALTH-DEGRADED emits cell_id, severity, affected_resources, and slo_burn.
- EVT-CLOUD-AZ-FAILOVER-STARTED emits failover_id, az_code, workload_count, and approver.
- EVT-CLOUD-AZ-FAILOVER-COMPLETED emits failover_id, duration_ms, failed_moves, and customer_notified.
- EVT-CLOUD-MARKETPLACE-INSTALL-PLANNED emits install_id, app_id, resource_count, and scope_hash.
- EVT-CLOUD-EVIDENCE-EXPORT-GENERATED emits export_id, scope, event_count, hash, and requester.
- EVT-CLOUD-HYPERSCALER-IMPORT-COMPLETED emits source_vendor, account_ref, mapped_count, and unsupported_count.

## Migration Playbook Index

- AWS import: IAM, VPC, EC2, EKS, Lambda, S3, EBS, RDS, Route53, ELB, CloudWatch, CUR.
- Azure import: Entra, VNet, VM, AKS, Functions, Blob, Disk, SQL, DNS, LB, Monitor, Cost.
- GCP import: IAM, VPC, Compute Engine, GKE, Cloud Functions, GCS, Persistent Disk, Cloud SQL, DNS, LB, Logging.
- Oracle Cloud import: compartments, IAM, VCN, compute, OKE, object storage, block volume, database, DNS, load balancer.
- Naver Cloud import: VPC, server, Kubernetes, object storage, NAS, Cloud DB, Load Balancer, Cloud Log Analytics.
- NHN Cloud import: compute, network, storage, database, Kubernetes, security groups, billing.
- KT Cloud import: VM, network, storage, firewall, load balancer, monitoring.
- Kakao Cloud import: compute, Kubernetes, object storage, VPC, IAM, logging.
- VMware import: vCenter inventory, clusters, VMs, datastores, networks, tags, snapshots.
- OpenStack import: projects, flavors, images, instances, Cinder, Swift, Neutron, Keystone.

## Capability Tier Deltas


## Competitive Positioning

- AWS: oyatie wins on unified tenant graph, Cedar-by-default, and regional pack evidence.
- Azure: oyatie wins on evidence-native sovereign controls and smaller policy surface.
- GCP: oyatie wins on audit-chain productization and Foundry-operated control plane.
- Oracle Cloud: oyatie wins on workload portability and tenant-scoped marketplace integration.
- Naver Cloud: oyatie wins on cross-region product platform plus KR sovereignty.
- NHN Cloud: oyatie wins on agent-operated operations and audit export.
- KT Cloud: oyatie wins on developer-facing managed service breadth and automation.
- Kakao Cloud: oyatie wins on enterprise control, evidence, and cross-product handoffs.
- Cloudflare: oyatie wins on full compute/storage/database substrate, not only edge.
- DigitalOcean: oyatie wins on regulated enterprise and sovereign-cell depth.

## Roadmap

- Wave C1: region catalog, account, IAM, KMS, VPC, VM, object storage, metering.
- Wave C2: managed Kubernetes, function service, block volume, DNS, LB, evidence export.
- Wave C3: managed Postgres, Redis, Kafka, marketplace install plan, cost explorer.
- Wave C4: direct interconnect, cross-region replication, FinOps anomaly, regulator portal.
- Phase M04: internal oyatie workload migration.
- Phase M05: design partner regulated cloud tenants.
- Phase M06: public cloud GA per region pack.

## Cross-Product Dependencies

- Foundry invokes cloud.compute.provision, cloud.iam.audit-narrow, cloud.capacity.rebalance, and cloud.incident.summarize-status.
- Workplace depends on compute, storage, KMS, observability, and budget meters.
- ERP depends on billing, FinOps, storage, workflow compute, and interconnect.
- Marketplace depends on app install resource plans and billing meters.
- Intelligence depends on GPU fleet, vector storage, object storage, and private network.
- Policy-engine owns Cedar compilation and evaluation.
- Audit-chain owns event sealing and evidence export.
- Tenancy owns tenant, account, pack, and membership scoping.
- Identity owns principals, groups, passkeys, and federation.
- Observability owns shared metrics, logs, traces, dashboards, and alert routes.
- Compliance owns region pack and regulator export requirements.
- Billing and FinOps own rate cards, meter aggregation, budgets, and invoices.

## Failure Modes + Recovery

- Failure: region catalog stale. Recovery: force projection refresh and block new placement until status reconciles.
- Failure: cell hot spot. Recovery: capacity rebalance plan, quota hold, tenant notification, and migration audit.
- Failure: VM provision partial. Recovery: rollback quota, delete partial network and disks, preserve event chain.
- Failure: object write succeeds but meter event fails. Recovery: outbox replay meter event and mark billing lag.
- Failure: KMS key disabled accidentally. Recovery: deny decrypt, open key incident, and require authorized re-enable.
- Failure: resource policy cache stale. Recovery: cache-bust event and deny uncertain authorization.
- Failure: VPC route leak. Recovery: withdraw route, deny propagation, notify affected tenants, and seal event.
- Failure: LB all backends unhealthy. Recovery: fail closed or route to static maintenance per service policy.
- Failure: managed DB backup missing. Recovery: mark service red and block promotion until backup completes.
- Failure: cost meter duplicate. Recovery: idempotency key collapses duplicate and emits duplicate-meter event.
- Failure: invoice tax field missing. Recovery: block invoice finalization and open tax-pack remediation.
- Failure: evidence export missing control. Recovery: mark export incomplete and reseal from audit-chain.
- Failure: marketplace app over-scopes. Recovery: deny install, suspend plan, and require listing update.
- Failure: Foundry agent proposes destructive unsafe action. Recovery: Cedar denies and emits autonomy denial.
- Failure: hyperscaler import unsupported type. Recovery: list blocker and manual migration playbook owner.
- Failure: cross-region replication violates pack. Recovery: deny target and propose legal alternatives.
- Failure: observability gap on managed service. Recovery: block catalog publish.
- Failure: direct interconnect BGP instability. Recovery: route dampening, customer notice, and failover path.
- Failure: secret reference leaked in output. Recovery: serializer redacts, incident opens, key rotation assessed.
- Failure: SLO burn alert flaps. Recovery: apply burn-rate window, dedupe, and hand to incident workflow.

## Cloud Capability Acceptance Ledger

### CLOUD-CAP-001 - Region catalog publish
- Owner: cloud-region.
- Pass: immutable region code, pack set, and status are visible.
- Evidence: EVT-CLOUD-REGION-STATE-CHANGED.

### CLOUD-CAP-002 - Region GA gate
- Owner: cloud-region.
- Pass: GA requires SLO, evidence export, pack controls, and runbooks.
- Evidence: region_gate_result.

### CLOUD-CAP-003 - AZ registration
- Owner: cloud-region.
- Pass: AZ declares power zones, physical site, and status.
- Evidence: az_registered event.

### CLOUD-CAP-004 - Cell create
- Owner: cloud-cell.
- Pass: cell declares isolation tier, density, and tenant limit.
- Evidence: EVT-CLOUD-CELL-BINDING-CREATED.

### CLOUD-CAP-005 - Tenant cell bind
- Owner: cloud-cell.
- Pass: tenant binding is policy-approved and audit-sealed.
- Evidence: cell_binding_id.

### CLOUD-CAP-006 - Cell health degrade
- Owner: cloud-cell.
- Pass: degradation lists resources, tenants, and SLO burn.
- Evidence: EVT-CLOUD-CELL-HEALTH-DEGRADED.

### CLOUD-CAP-007 - Cell rebalance plan
- Owner: cloud-cell.
- Pass: plan lists moves, risk, approvals, and rollback.
- Evidence: rebalance_plan_id.

### CLOUD-CAP-008 - Account create
- Owner: cloud-iam.
- Pass: account is tenant-rooted and has billing account.
- Evidence: EVT-CLOUD-ACCOUNT-CREATED.

### CLOUD-CAP-009 - Quota reserve
- Owner: cloud-resource.
- Pass: resource plan locks quota before provisioning.
- Evidence: quota_lock_id.

### CLOUD-CAP-010 - Quota release
- Owner: cloud-resource.
- Pass: failed provision releases quota idempotently.
- Evidence: quota_release_id.

### CLOUD-CAP-011 - Image register
- Owner: cloud-compute.
- Pass: image has signature, SBOM, provenance, and vulnerability state.
- Evidence: image_attestation_id.

### CLOUD-CAP-012 - Image deny unsigned
- Owner: cloud-compute.
- Pass: unsigned image launch is denied.
- Evidence: policy_decision_id.

### CLOUD-CAP-013 - Flavor publish
- Owner: cloud-compute.
- Pass: flavor declares CPU, RAM, storage, and pack eligibility.
- Evidence: flavor_record_id.

### CLOUD-CAP-014 - VM create receipt
- Owner: cloud-compute.
- Pass: API receipt p99 target and evidence id are returned.
- Evidence: EVT-CLOUD-INSTANCE-CREATED.

### CLOUD-CAP-015 - VM boot state
- Owner: cloud-compute.
- Pass: boot transitions pending, running, stopped, terminated, error.
- Evidence: instance_state_event.

### CLOUD-CAP-016 - VM stop
- Owner: cloud-compute.
- Pass: stop preserves volume and emits meter state change.
- Evidence: vm_stop_event.

### CLOUD-CAP-017 - VM snapshot
- Owner: cloud-compute.
- Pass: snapshot links volume, checksum, KMS key, and retention.
- Evidence: snapshot_id.

### CLOUD-CAP-018 - VM live migrate
- Owner: cloud-compute.
- Pass: migration plan respects cell and host constraints.
- Evidence: live_migration_plan_id.

### CLOUD-CAP-019 - Bare-metal lease
- Owner: cloud-compute.
- Pass: lease has hardware attestation and teardown proof.
- Evidence: bare_metal_lease_id.

### CLOUD-CAP-020 - GPU fleet allocate
- Owner: cloud-compute.
- Pass: allocation shows GPU class, quota, and cost meter.
- Evidence: gpu_allocation_id.

### CLOUD-CAP-021 - Auto scaling group
- Owner: cloud-compute.
- Pass: group declares min, max, health check, and policy.
- Evidence: asg_policy_id.

### CLOUD-CAP-022 - Placement group
- Owner: cloud-compute.
- Pass: anti-affinity and latency placement rules are explicit.
- Evidence: placement_group_id.

### CLOUD-CAP-023 - Kubernetes cluster create
- Owner: cloud-compute.
- Pass: version, node pools, backup, and upgrade channel exist.
- Evidence: EVT-CLOUD-CLUSTER-CREATED.

### CLOUD-CAP-024 - Kubernetes node pool scale
- Owner: cloud-compute.
- Pass: scale respects quota and cell capacity.
- Evidence: node_pool_scale_id.

### CLOUD-CAP-025 - Kubernetes upgrade
- Owner: cloud-compute.
- Pass: unsupported version is denied and rollback is defined.
- Evidence: cluster_upgrade_plan_id.

### CLOUD-CAP-026 - Kubernetes backup
- Owner: cloud-compute.
- Pass: backup covers etcd, manifests, and persistent volume refs.
- Evidence: cluster_backup_id.

### CLOUD-CAP-027 - Function deploy
- Owner: cloud-compute.
- Pass: runtime, timeout, memory, trigger, and role are declared.
- Evidence: EVT-CLOUD-FUNCTION-DEPLOYED.

### CLOUD-CAP-028 - Function invoke receipt
- Owner: cloud-compute.
- Pass: invocation emits latency, status, and audit receipt.
- Evidence: function_invocation_id.

### CLOUD-CAP-029 - Function cold-start budget
- Owner: cloud-compute.
- Pass: cold-start budget is declared per runtime.
- Evidence: cold_start_metric.

### CLOUD-CAP-030 - Function event trigger
- Owner: cloud-compute.
- Pass: trigger binds to allowed event source only.
- Evidence: trigger_binding_id.

### CLOUD-CAP-031 - Bucket create
- Owner: cloud-storage.
- Pass: bucket declares region, KMS, policy, and retention.
- Evidence: EVT-CLOUD-BUCKET-CREATED.

### CLOUD-CAP-032 - Bucket resource policy
- Owner: cloud-storage.
- Pass: resource deny overrides principal allow.
- Evidence: EVT-CLOUD-RESOURCE-POLICY-CHANGED.

### CLOUD-CAP-033 - Object put
- Owner: cloud-storage.
- Pass: checksum, version, KMS context, and data class are stored.
- Evidence: EVT-CLOUD-OBJECT-WRITTEN.

### CLOUD-CAP-034 - Object get
- Owner: cloud-storage.
- Pass: authorization checks principal and resource policy.
- Evidence: object_read_event.

### CLOUD-CAP-035 - Object retention lock
- Owner: cloud-storage.
- Pass: locked object cannot be deleted before retention_until.
- Evidence: retention_denial_event.

### CLOUD-CAP-036 - Object replication
- Owner: cloud-storage.
- Pass: replication target is residency-compatible.
- Evidence: replication_policy_id.

### CLOUD-CAP-037 - Object lifecycle
- Owner: cloud-storage.
- Pass: transition and expire rules are policy-visible.
- Evidence: lifecycle_rule_id.

### CLOUD-CAP-038 - Archive vault
- Owner: cloud-storage.
- Pass: archive has restore SLA, retention, and cost class.
- Evidence: archive_vault_id.

### CLOUD-CAP-039 - Volume create
- Owner: cloud-storage.
- Pass: volume declares size, IOPS class, KMS, and snapshot policy.
- Evidence: EVT-CLOUD-VOLUME-CREATED.

### CLOUD-CAP-040 - Volume attach
- Owner: cloud-storage.
- Pass: attach requires same compatible region and account.
- Evidence: volume_attach_event.

### CLOUD-CAP-041 - Volume detach
- Owner: cloud-storage.
- Pass: detach preserves data and meter state.
- Evidence: volume_detach_event.

### CLOUD-CAP-042 - Volume snapshot
- Owner: cloud-storage.
- Pass: snapshot includes checksum and KMS key.
- Evidence: volume_snapshot_id.

### CLOUD-CAP-043 - Volume restore
- Owner: cloud-storage.
- Pass: restore preview exists before destructive replace.
- Evidence: restore_plan_id.

### CLOUD-CAP-044 - File share create
- Owner: cloud-storage.
- Pass: share declares protocol, ACL, throughput, and retention.
- Evidence: file_share_id.

### CLOUD-CAP-045 - File share mount
- Owner: cloud-storage.
- Pass: mount is restricted by network and IAM.
- Evidence: mount_authorization_id.

### CLOUD-CAP-046 - Backup policy
- Owner: cloud-storage.
- Pass: backup interval, retention, and restore test are declared.
- Evidence: backup_policy_id.

### CLOUD-CAP-047 - Backup restore test
- Owner: cloud-storage.
- Pass: restore test evidence exists for managed data services.
- Evidence: restore_test_id.

### CLOUD-CAP-048 - VPC create
- Owner: cloud-network.
- Pass: CIDR is non-overlap and region-scoped.
- Evidence: EVT-CLOUD-VPC-CREATED.

### CLOUD-CAP-049 - Subnet create
- Owner: cloud-network.
- Pass: subnet declares AZ, CIDR, route table, and tier.
- Evidence: subnet_id.

### CLOUD-CAP-050 - Route table update
- Owner: cloud-network.
- Pass: route update prevents illegal egress and overlap.
- Evidence: route_change_event.

### CLOUD-CAP-051 - Security group create
- Owner: cloud-network.
- Pass: default deny is enforced.
- Evidence: security_group_id.

### CLOUD-CAP-052 - Security group rule
- Owner: cloud-network.
- Pass: rule includes protocol, source, destination, purpose, expiry.
- Evidence: sg_rule_id.

### CLOUD-CAP-053 - Network ACL
- Owner: cloud-network.
- Pass: ACL blocks cross-cell traffic by default.
- Evidence: network_acl_id.

### CLOUD-CAP-054 - NAT gateway
- Owner: cloud-network.
- Pass: egress IP and logging are declared.
- Evidence: nat_gateway_id.

### CLOUD-CAP-055 - Internet gateway
- Owner: cloud-network.
- Pass: public egress requires explicit route and policy.
- Evidence: internet_gateway_id.

### CLOUD-CAP-056 - Private endpoint
- Owner: cloud-network.
- Pass: endpoint avoids public internet path.
- Evidence: private_endpoint_id.

### CLOUD-CAP-057 - Direct interconnect
- Owner: cloud-network.
- Pass: port, BGP, route filters, and SLA are declared.
- Evidence: interconnect_id.

### CLOUD-CAP-058 - BGP route filter
- Owner: cloud-network.
- Pass: route leak is denied and alerted.
- Evidence: route_filter_event.

### CLOUD-CAP-059 - Load balancer create
- Owner: cloud-network.
- Pass: listeners, backend, health check, and TLS policy exist.
- Evidence: EVT-CLOUD-LB-PUBLISHED.

### CLOUD-CAP-060 - Load balancer health
- Owner: cloud-network.
- Pass: all-backend unhealthy opens incident.
- Evidence: lb_health_incident_id.

### CLOUD-CAP-061 - WAF policy
- Owner: cloud-network.
- Pass: public L7 load balancer has WAF policy.
- Evidence: waf_policy_id.

### CLOUD-CAP-062 - DNS zone create
- Owner: cloud-network.
- Pass: zone has DNSSEC option and owner account.
- Evidence: dns_zone_id.

### CLOUD-CAP-063 - DNS record publish
- Owner: cloud-network.
- Pass: record change has signed change id.
- Evidence: EVT-CLOUD-DNS-CHANGE-PUBLISHED.

### CLOUD-CAP-064 - CDN distribution
- Owner: cloud-network.
- Pass: origin, cache rule, TLS, and invalidation path exist.
- Evidence: cdn_distribution_id.

### CLOUD-CAP-065 - DDoS protection
- Owner: cloud-network.
- Pass: protected endpoint has threshold and mitigation profile.
- Evidence: ddos_profile_id.

### CLOUD-CAP-066 - IAM principal create
- Owner: cloud-iam.
- Pass: principal maps to tenant identity and role.
- Evidence: iam_principal_id.

### CLOUD-CAP-067 - IAM role create
- Owner: cloud-iam.
- Pass: trust policy, boundary policy, and max duration exist.
- Evidence: iam_role_id.

### CLOUD-CAP-068 - IAM policy attach
- Owner: cloud-iam.
- Pass: policy diff and owner approval are recorded.
- Evidence: EVT-CLOUD-IAM-POLICY-CHANGED.

### CLOUD-CAP-069 - IAM policy simulate
- Owner: cloud-iam.
- Pass: decision preview is available before save.
- Evidence: policy_simulation_id.

### CLOUD-CAP-070 - IAM analyzer
- Owner: cloud-iam.
- Pass: least-privilege suggestions cite access logs.
- Evidence: access_analyzer_finding_id.

### CLOUD-CAP-071 - STS assume role
- Owner: cloud-iam.
- Pass: session has scope, expiry, and actor.
- Evidence: EVT-CLOUD-STS-SESSION-ISSUED.

### CLOUD-CAP-072 - Federation OIDC
- Owner: cloud-iam.
- Pass: OIDC provider has issuer, audience, thumbprint, and mapping.
- Evidence: federation_provider_id.

### CLOUD-CAP-073 - Federation SAML
- Owner: cloud-iam.
- Pass: SAML metadata and attribute map are versioned.
- Evidence: saml_provider_id.

### CLOUD-CAP-074 - MFA enforcement
- Owner: cloud-iam.
- Pass: high-risk action requires MFA or passkey.
- Evidence: mfa_requirement_event.

### CLOUD-CAP-075 - KMS key create
- Owner: cloud-kms.
- Pass: key declares class, origin, region, and pack.
- Evidence: kms_key_id.

### CLOUD-CAP-076 - KMS encrypt
- Owner: cloud-kms.
- Pass: encrypt validates key, policy, and context.
- Evidence: kms_encrypt_receipt.

### CLOUD-CAP-077 - KMS decrypt
- Owner: cloud-kms.
- Pass: decrypt denies disabled or out-of-context key.
- Evidence: kms_decrypt_receipt.

### CLOUD-CAP-078 - KMS rotate
- Owner: cloud-kms.
- Pass: rotation emits version and approver evidence.
- Evidence: EVT-CLOUD-KMS-KEY-ROTATED.

### CLOUD-CAP-079 - KMS disable
- Owner: cloud-kms.
- Pass: disable shows affected resources and approval.
- Evidence: key_disable_event.

### CLOUD-CAP-080 - KMS shred
- Owner: cloud-kms.
- Evidence: key_shred_event.

### CLOUD-CAP-081 - Secret reference create
- Owner: cloud-kms.
- Pass: raw secret is never returned.
- Evidence: secret_ref_id.

### CLOUD-CAP-082 - Secret rotate
- Owner: cloud-kms.
- Pass: rotation updates version and notifies consumers.
- Evidence: secret_rotation_id.

### CLOUD-CAP-083 - Secret access
- Owner: cloud-kms.
- Pass: access logs principal, purpose, and resource.
- Evidence: secret_access_event.

### CLOUD-CAP-084 - Managed Postgres create
- Owner: cloud-data.
- Pass: version, HA, backup, KMS, and maintenance window exist.
- Evidence: postgres_instance_id.

### CLOUD-CAP-085 - Managed Postgres backup
- Owner: cloud-data.
- Pass: backup completes within RPO policy.
- Evidence: postgres_backup_id.

### CLOUD-CAP-086 - Managed Postgres restore
- Owner: cloud-data.
- Pass: restore preview and target are explicit.
- Evidence: postgres_restore_plan_id.

### CLOUD-CAP-087 - Managed Redis create
- Owner: cloud-data.
- Pass: memory, eviction, persistence, and network policy exist.
- Evidence: redis_instance_id.

### CLOUD-CAP-088 - Managed Kafka cluster
- Owner: cloud-data.
- Pass: partitions, retention, schema, ACL, and quota exist.
- Evidence: kafka_cluster_id.

### CLOUD-CAP-089 - Kafka topic publish
- Owner: cloud-data.
- Pass: topic declares schema and retention.
- Evidence: kafka_topic_id.

### CLOUD-CAP-090 - Managed ClickHouse
- Owner: cloud-data.
- Pass: cluster declares shards, replicas, backup, and KMS.
- Evidence: clickhouse_cluster_id.

### CLOUD-CAP-091 - Meter event ingest
- Owner: cloud-billing.
- Pass: usage event has idempotency key and rate card.
- Evidence: EVT-CLOUD-METER-EVENT-INGESTED.

### CLOUD-CAP-092 - Meter aggregation
- Owner: cloud-billing.
- Pass: aggregation groups by account, resource, region, tag, unit.
- Evidence: meter_aggregation_id.

### CLOUD-CAP-093 - Rate card publish
- Owner: cloud-billing.
- Pass: rate card is versioned and effective-dated.
- Evidence: rate_card_id.

### CLOUD-CAP-094 - Invoice draft
- Owner: cloud-billing.
- Pass: draft includes usage, taxes, credits, and adjustments.
- Evidence: invoice_draft_id.

### CLOUD-CAP-095 - Invoice final
- Owner: cloud-billing.
- Pass: final invoice includes tax-pack fields and signed hash.
- Evidence: EVT-CLOUD-INVOICE-GENERATED.

### CLOUD-CAP-096 - Budget create
- Owner: cloud-finops.
- Pass: budget includes owner, period, threshold, and alert route.
- Evidence: budget_id.

### CLOUD-CAP-097 - Budget alert
- Owner: cloud-finops.
- Pass: alert opens before threshold breach when forecast predicts overrun.
- Evidence: EVT-CLOUD-BUDGET-ALERT-OPENED.

### CLOUD-CAP-098 - Cost anomaly
- Owner: cloud-finops.
- Pass: anomaly cites driver and estimated impact.
- Evidence: EVT-CLOUD-COST-ANOMALY-DETECTED.

### CLOUD-CAP-099 - Cost allocation tag
- Owner: cloud-finops.
- Pass: untagged spend is visible in exception queue.
- Evidence: tag_exception_id.

### CLOUD-CAP-100 - Margin report
- Owner: cloud-finops.
- Pass: report shows cost, revenue, gross margin by tenant and product.
- Evidence: margin_report_id.

### CLOUD-CAP-101 - Observability workspace
- Owner: cloud-observability.
- Pass: tenant has metrics, logs, traces, alerts, and audit links.
- Evidence: observability_workspace_id.

### CLOUD-CAP-102 - Metric stream
- Owner: cloud-observability.
- Pass: metric has dimensions and cardinality budget.
- Evidence: metric_stream_id.

### CLOUD-CAP-103 - Log stream
- Owner: cloud-observability.
- Pass: log stream has retention and data class.
- Evidence: log_stream_id.

### CLOUD-CAP-104 - Trace stream
- Owner: cloud-observability.
- Pass: trace has service, span, tenant, and retention fields.
- Evidence: trace_stream_id.

### CLOUD-CAP-105 - Alert route
- Owner: cloud-observability.
- Pass: alert has owner, severity, route, and escalation.
- Evidence: alert_route_id.

### CLOUD-CAP-106 - SLO burn alert
- Owner: cloud-observability.
- Pass: burn alert uses multi-window threshold.
- Evidence: slo_burn_event.

### CLOUD-CAP-107 - Dashboard publish
- Owner: cloud-observability.
- Pass: dashboard links resources and SLOs.
- Evidence: dashboard_id.

### CLOUD-CAP-108 - Audit log export
- Owner: cloud-observability.
- Pass: export includes event ids and signed hash.
- Evidence: EVT-CLOUD-EVIDENCE-EXPORT-GENERATED.

### CLOUD-CAP-109 - Evidence control map
- Owner: cloud-observability.
- Pass: controls map to events and pack requirements.
- Evidence: control_map_id.

### CLOUD-CAP-110 - Incident status
- Owner: cloud-observability.
- Pass: incident status cites affected cells and customer impact.
- Evidence: incident_status_id.

### CLOUD-CAP-111 - Marketplace install plan
- Owner: cloud-marketplace.
- Pass: plan declares resources, IAM, egress, meters, and teardown.
- Evidence: EVT-CLOUD-MARKETPLACE-INSTALL-PLANNED.

### CLOUD-CAP-112 - Marketplace scope denial
- Owner: cloud-marketplace.
- Pass: over-scoped install is denied.
- Evidence: marketplace_scope_denial_id.

### CLOUD-CAP-113 - Marketplace teardown
- Owner: cloud-marketplace.
- Pass: uninstall deletes or preserves resources per plan.
- Evidence: teardown_plan_id.

### CLOUD-CAP-114 - Service catalog publish
- Owner: cloud-resource.
- Pass: service has SLO, telemetry, runbook, backup, KMS, billing.
- Evidence: service_catalog_record_id.

### CLOUD-CAP-115 - Service catalog block
- Owner: cloud-resource.
- Pass: missing telemetry or runbook blocks publish.
- Evidence: service_catalog_block_event.

### CLOUD-CAP-116 - Hyperscaler import AWS
- Owner: cloud-migration.
- Pass: AWS resources map or list blockers.
- Evidence: EVT-CLOUD-HYPERSCALER-IMPORT-COMPLETED.

### CLOUD-CAP-117 - Hyperscaler import Azure
- Owner: cloud-migration.
- Pass: Azure resources map or list blockers.
- Evidence: azure_import_result.

### CLOUD-CAP-118 - Hyperscaler import GCP
- Owner: cloud-migration.
- Pass: GCP resources map or list blockers.
- Evidence: gcp_import_result.

### CLOUD-CAP-119 - VMware import
- Owner: cloud-migration.
- Pass: VM, network, datastore, and tag inventory maps.
- Evidence: vmware_import_result.

### CLOUD-CAP-120 - OpenStack import
- Owner: cloud-migration.
- Pass: project, instance, Cinder, Swift, Neutron, Keystone inventory maps.
- Evidence: openstack_import_result.

### CLOUD-CAP-121 - Migration unsupported report
- Owner: cloud-migration.
- Pass: unsupported resources have owner, reason, and target playbook.
- Evidence: unsupported_resource_report_id.

### CLOUD-CAP-122 - Migration cutover
- Owner: cloud-migration.
- Pass: cutover has freeze, delta, rollback, and evidence plan.
- Evidence: migration_cutover_id.

### CLOUD-CAP-123 - Migration rollback
- Owner: cloud-migration.
- Pass: rollback returns traffic and state to source checkpoint.
- Evidence: migration_rollback_id.

### CLOUD-CAP-124 - Compliance evidence export
- Owner: cloud-compliance.
- Pass: export maps control ids to events and artifacts.
- Evidence: compliance_export_id.

### CLOUD-CAP-125 - CSAP control
- Owner: cloud-compliance.
- Pass: KR controls show KMS, region, logging, and retention proof.
- Evidence: csap_control_result.

### CLOUD-CAP-126 - DORA control
- Owner: cloud-compliance.
- Pass: EU financial resilience export includes exit and continuity evidence.
- Evidence: dora_control_result.

### CLOUD-CAP-127 - FedRAMP control
- Owner: cloud-compliance.
- Pass: US government profile includes required retention and boundary evidence.
- Evidence: fedramp_control_result.

### CLOUD-CAP-128 - ISMAP control
- Owner: cloud-compliance.
- Pass: JP profile includes APPI and local audit language support.
- Evidence: ismap_control_result.

### CLOUD-CAP-129 - LGPD control
- Owner: cloud-compliance.
- Pass: BR profile includes DSR hooks and tax evidence.
- Evidence: lgpd_control_result.

### CLOUD-CAP-130 - Pack conflict resolve
- Owner: cloud-compliance.
- Pass: stricter control wins and legal review opens if needed.
- Evidence: pack_conflict_result.

### CLOUD-CAP-131 - Data perimeter deny
- Owner: cloud-policy.
- Pass: resource policy deny overrides principal allow.
- Evidence: data_perimeter_denial_id.

### CLOUD-CAP-132 - Cross-region deny
- Owner: cloud-policy.
- Pass: incompatible residency target is denied.
- Evidence: cross_region_denial_id.

### CLOUD-CAP-133 - Public endpoint deny
- Owner: cloud-policy.
- Pass: public endpoint requires explicit approved policy.
- Evidence: public_endpoint_denial_id.

### CLOUD-CAP-134 - Destructive approval hold
- Owner: cloud-policy.
- Pass: delete, shred, and failover require tier-specific approvals.
- Evidence: approval_hold_id.

### CLOUD-CAP-135 - Autonomy tier check
- Owner: cloud-policy.
- Pass: Foundry agent action is bounded by autonomy tier.
- Evidence: autonomy_decision_id.

### CLOUD-CAP-136 - Resource tag enforce
- Owner: cloud-resource.
- Pass: required tags are present before resource activation.
- Evidence: tag_enforcement_event.

### CLOUD-CAP-137 - Resource delete
- Owner: cloud-resource.
- Pass: delete checks dependencies, retention, and approval tier.
- Evidence: resource_delete_event.

### CLOUD-CAP-138 - Dependency graph
- Owner: cloud-resource.
- Pass: resource graph lists upstream and downstream dependencies.
- Evidence: dependency_graph_id.

### CLOUD-CAP-139 - Drift detect
- Owner: cloud-resource.
- Pass: resource state drift is detected and owner assigned.
- Evidence: drift_detection_id.

### CLOUD-CAP-140 - Drift remediate
- Owner: cloud-resource.
- Pass: remediation plan is previewed before apply.
- Evidence: drift_remediation_plan_id.

### CLOUD-CAP-141 - Backup gap alert
- Owner: cloud-storage.
- Pass: managed data resource without backup is red.
- Evidence: backup_gap_alert_id.

### CLOUD-CAP-142 - Restore RTO report
- Owner: cloud-storage.
- Pass: restore drill reports RTO and RPO.
- Evidence: restore_rto_report_id.

### CLOUD-CAP-143 - Endpoint egress log
- Owner: cloud-network.
- Pass: egress logs include principal, route, destination, and policy.
- Evidence: egress_log_event.

### CLOUD-CAP-144 - Private DNS
- Owner: cloud-network.
- Pass: private zone is scoped to VPC and account.
- Evidence: private_dns_zone_id.

### CLOUD-CAP-145 - TLS policy
- Owner: cloud-network.
- Pass: TLS policy enforces minimum version and ciphers.
- Evidence: tls_policy_id.

### CLOUD-CAP-146 - Certificate issue
- Owner: cloud-network.
- Pass: certificate issue has domain validation and expiry route.
- Evidence: certificate_id.

### CLOUD-CAP-147 - Certificate rotate
- Owner: cloud-network.
- Pass: rotation completes before expiry and emits evidence.
- Evidence: certificate_rotation_id.

### CLOUD-CAP-148 - API gateway route
- Owner: cloud-network.
- Pass: route declares auth, rate limit, and backend.
- Evidence: api_route_id.

### CLOUD-CAP-149 - Rate limit
- Owner: cloud-network.
- Pass: limit declares key, threshold, burst, and action.
- Evidence: rate_limit_policy_id.

### CLOUD-CAP-150 - Queue service
- Owner: cloud-data.
- Pass: queue declares retention, DLQ, encryption, and quota.
- Evidence: queue_id.

### CLOUD-CAP-151 - Topic service
- Owner: cloud-data.
- Pass: topic declares schema, retention, subscribers, and ACL.
- Evidence: topic_id.

### CLOUD-CAP-152 - Search service
- Owner: cloud-data.
- Pass: index declares schema, data class, shard, and backup.
- Evidence: search_service_id.

### CLOUD-CAP-153 - Vector service
- Owner: cloud-data.
- Pass: vector index declares tenant, embedding class, region, and delete path.
- Evidence: vector_index_id.

### CLOUD-CAP-154 - Database maintenance
- Owner: cloud-data.
- Pass: maintenance window and rollback are declared.
- Evidence: maintenance_plan_id.

### CLOUD-CAP-155 - Database upgrade
- Owner: cloud-data.
- Pass: upgrade dry run and backup checkpoint exist.
- Evidence: db_upgrade_plan_id.

### CLOUD-CAP-156 - Database failover
- Owner: cloud-data.
- Pass: failover emits RTO, RPO, and affected apps.
- Evidence: db_failover_event.

### CLOUD-CAP-157 - Data export
- Owner: cloud-data.
- Pass: export respects data class and pack redaction.
- Evidence: data_export_id.

### CLOUD-CAP-158 - Data deletion
- Owner: cloud-data.
- Pass: delete respects retention lock and legal hold.
- Evidence: data_delete_event.

### CLOUD-CAP-159 - Legal hold
- Owner: cloud-data.
- Pass: hold blocks deletion and lifecycle expiration.
- Evidence: legal_hold_id.

### CLOUD-CAP-160 - DSR cascade
- Owner: cloud-data.
- Pass: data-subject delete or export cascades to eligible resources.
- Evidence: dsr_cascade_id.

### CLOUD-CAP-161 - Tenant offboarding
- Owner: cloud-resource.
- Pass: offboarding plan lists export, delete, hold, invoice, and revoke tasks.
- Evidence: tenant_offboarding_plan_id.

### CLOUD-CAP-162 - Account suspend
- Owner: cloud-resource.
- Pass: suspend blocks new mutations while preserving data.
- Evidence: account_suspend_event.

### CLOUD-CAP-163 - Account reactivate
- Owner: cloud-resource.
- Pass: reactivation requires payment and policy status green.
- Evidence: account_reactivate_event.

### CLOUD-CAP-164 - SLA claim
- Owner: cloud-billing.
- Pass: SLA credit links incident, tenant, and service.
- Evidence: sla_credit_id.

### CLOUD-CAP-165 - Committed use discount
- Owner: cloud-billing.
- Pass: commitment has term, resource scope, discount, and exit rule.
- Evidence: commitment_id.

### CLOUD-CAP-166 - Reserved instance
- Owner: cloud-billing.
- Pass: reservation applies to eligible instance usage.
- Evidence: reservation_id.

### CLOUD-CAP-167 - Spot instance
- Owner: cloud-compute.
- Pass: interruption notice and workload eligibility are explicit.
- Evidence: spot_instance_id.

### CLOUD-CAP-168 - Preemptible eviction
- Owner: cloud-compute.
- Pass: eviction emits notice and billing adjustment.
- Evidence: preempt_event_id.

### CLOUD-CAP-169 - Edge compute
- Owner: cloud-compute.
- Pass: edge location has pack and data perimeter constraints.
- Evidence: edge_deployment_id.

### CLOUD-CAP-170 - CDN invalidation
- Owner: cloud-network.
- Pass: invalidation has scope, caller, and completion status.
- Evidence: cdn_invalidation_id.

### CLOUD-CAP-171 - Object malware scan
- Owner: cloud-storage.
- Pass: object scan result gates public exposure.
- Evidence: malware_scan_result_id.

### CLOUD-CAP-172 - SBOM store
- Owner: cloud-supply.
- Pass: release artifact SBOM is stored and linked to image.
- Evidence: sbom_artifact_id.

### CLOUD-CAP-173 - Vulnerability gate
- Owner: cloud-supply.
- Pass: critical vulnerabilities block publish unless waiver exists.
- Evidence: vulnerability_gate_result.

### CLOUD-CAP-174 - Cosign verify
- Owner: cloud-supply.
- Pass: image signature verifies before launch.
- Evidence: cosign_verification_id.

### CLOUD-CAP-175 - Provenance verify
- Owner: cloud-supply.
- Pass: provenance links build, source, and digest.
- Evidence: provenance_id.

### CLOUD-CAP-176 - Policy bundle publish
- Owner: cloud-policy.
- Pass: Cedar bundle version and tests are recorded.
- Evidence: policy_bundle_id.

### CLOUD-CAP-177 - Policy cache bust
- Owner: cloud-policy.
- Pass: cache bust follows policy or resource-policy change.
- Evidence: cache_bust_id.

### CLOUD-CAP-178 - Policy test
- Owner: cloud-policy.
- Pass: allow and deny fixtures pass before publish.
- Evidence: policy_test_result_id.

### CLOUD-CAP-179 - Quarantine resource
- Owner: cloud-security.
- Pass: quarantine isolates network and stops write mutations.
- Evidence: quarantine_event_id.

### CLOUD-CAP-180 - Incident containment
- Owner: cloud-security.
- Pass: containment plan lists scope, action, and rollback.
- Evidence: containment_plan_id.

### CLOUD-CAP-181 - Key compromise response
- Owner: cloud-security.
- Pass: response rotates or disables affected key and notifies owners.
- Evidence: key_compromise_run_id.

### CLOUD-CAP-182 - Credential leak response
- Owner: cloud-security.
- Pass: token revoked, logs searched, and blast radius reported.
- Evidence: credential_leak_run_id.

### CLOUD-CAP-183 - Tenant isolation test
- Owner: cloud-security.
- Pass: cross-tenant access tests fail closed.
- Evidence: isolation_test_result_id.

### CLOUD-CAP-184 - Cross-cell chaos drill
- Owner: cloud-security.
- Pass: drill proves cross-cell deny and alert.
- Evidence: chaos_drill_id.

### CLOUD-CAP-185 - DR pair promote
- Owner: cloud-region.
- Pass: promote preserves RPO/RTO and audit chain.
- Evidence: dr_promote_event_id.

### CLOUD-CAP-186 - DR pair demote
- Owner: cloud-region.
- Pass: demote has consistency check and rollback.
- Evidence: dr_demote_event_id.

### CLOUD-CAP-187 - Region evacuation
- Owner: cloud-region.
- Pass: evacuation plan lists workloads, data, DNS, and customer notice.
- Evidence: region_evacuation_plan_id.

### CLOUD-CAP-188 - Region retirement
- Owner: cloud-region.
- Pass: retirement has migration, retention, and contract notice.
- Evidence: region_retirement_id.

### CLOUD-CAP-189 - Resource search
- Owner: cloud-resource.
- Pass: search filters by type, tag, region, state, and owner.
- Evidence: resource_search_query_id.

### CLOUD-CAP-190 - Resource inventory export
- Owner: cloud-resource.
- Pass: export includes resource graph and policy attachments.
- Evidence: inventory_export_id.

### CLOUD-CAP-191 - Customer notification
- Owner: cloud-support.
- Pass: notification audience derives from affected resources.
- Evidence: customer_notification_id.

### CLOUD-CAP-192 - Support case
- Owner: cloud-support.
- Pass: case links tenant, resource, incident, and SLA.
- Evidence: support_case_id.

### CLOUD-CAP-193 - Maintenance notice
- Owner: cloud-support.
- Pass: notice includes window, impact, rollback, and owner.
- Evidence: maintenance_notice_id.

### CLOUD-CAP-194 - Runbook link
- Owner: cloud-support.
- Pass: every service has current runbook link.
- Evidence: runbook_link_check_id.

### CLOUD-CAP-195 - Customer status page
- Owner: cloud-support.
- Pass: page reflects incidents by region and service.
- Evidence: status_page_event_id.

### CLOUD-CAP-196 - Console session
- Owner: cloud-console.
- Pass: session is tied to identity, tenant, and MFA state.
- Evidence: console_session_id.

### CLOUD-CAP-197 - Console action preview
- Owner: cloud-console.
- Pass: preview shows policy result and blast radius.
- Evidence: action_preview_id.

### CLOUD-CAP-198 - Console action execute
- Owner: cloud-console.
- Pass: execution uses idempotency key and audit seal.
- Evidence: console_action_event_id.

### CLOUD-CAP-199 - API idempotency
- Owner: cloud-api.
- Pass: mutating APIs require idempotency key.
- Evidence: idempotency_record_id.

### CLOUD-CAP-200 - API pagination
- Owner: cloud-api.
- Pass: list APIs use cursor and bounded page size.
- Evidence: pagination_contract_id.

### CLOUD-CAP-201 - API deprecation
- Owner: cloud-api.
- Pass: deprecation has SemVer, sunset, and migration path.
- Evidence: api_deprecation_id.

### CLOUD-CAP-202 - API OpenAPI publish
- Owner: cloud-api.
- Pass: OpenAPI contract validates and links to tests.
- Evidence: openapi_validation_id.

### CLOUD-CAP-203 - SDK generate
- Owner: cloud-api.
- Pass: SDK generation uses published OpenAPI only.
- Evidence: sdk_generation_id.

### CLOUD-CAP-204 - CLI command
- Owner: cloud-api.
- Pass: CLI command maps to stable API and evidence output.
- Evidence: cli_command_test_id.

### CLOUD-CAP-205 - Terraform provider
- Owner: cloud-iac.
- Pass: provider maps desired state to cloud APIs idempotently.
- Evidence: terraform_provider_test_id.

### CLOUD-CAP-206 - OpenTofu module
- Owner: cloud-iac.
- Pass: module includes examples, rollback, and drift checks.
- Evidence: opentofu_module_id.

### CLOUD-CAP-207 - GitOps reconcile
- Owner: cloud-iac.
- Pass: GitOps sync reports drift and policy denials.
- Evidence: gitops_reconcile_id.

### CLOUD-CAP-208 - K8s admission
- Owner: cloud-iac.
- Pass: admission denies resources that violate pack or policy.
- Evidence: admission_denial_id.

### CLOUD-CAP-209 - Network policy
- Owner: cloud-iac.
- Pass: workload network policy is default deny.
- Evidence: network_policy_id.

### CLOUD-CAP-210 - Secret binding
- Owner: cloud-iac.
- Pass: deployment receives secret refs, not raw secrets.
- Evidence: secret_binding_id.

### CLOUD-CAP-211 - Sovereign evidence pack
- Owner: cloud-compliance.
- Pass: sovereign pack exports controls and exceptions.
- Evidence: sovereign_pack_export_id.

### CLOUD-CAP-212 - Regulator portal access
- Owner: cloud-compliance.
- Pass: regulator has read-only scoped portal.
- Evidence: regulator_portal_session_id.

### CLOUD-CAP-213 - Auditor redaction
- Owner: cloud-compliance.
- Pass: evidence export redacts data by pack.
- Evidence: auditor_redaction_event_id.

### CLOUD-CAP-214 - Evidence hash verify
- Owner: cloud-compliance.
- Pass: export hash verifies against audit-chain.
- Evidence: evidence_hash_verification_id.

### CLOUD-CAP-215 - Control exception
- Owner: cloud-compliance.
- Pass: exception has owner, expiry, compensating control, and approval.
- Evidence: control_exception_id.

### CLOUD-CAP-216 - Control expiry alert
- Owner: cloud-compliance.
- Pass: expiring exception alerts before expiry.
- Evidence: control_expiry_alert_id.

### CLOUD-CAP-217 - Customer BYOK
- Owner: cloud-kms.
- Pass: BYOK imports wrapped key material and custody evidence.
- Evidence: byok_import_id.

### CLOUD-CAP-218 - Customer HYOK
- Owner: cloud-kms.
- Pass: HYOK mode declares external key availability behavior.
- Evidence: hyok_binding_id.

### CLOUD-CAP-219 - HSM cluster
- Owner: cloud-kms.
- Pass: HSM cluster has region, certification, quorum, and backup.
- Evidence: hsm_cluster_id.

### CLOUD-CAP-220 - Key custody report
- Owner: cloud-kms.
- Pass: report shows key origin, custody, rotation, and access.
- Evidence: key_custody_report_id.

## AI substrate + Cellular automation

This product consumes the Wave 15-ZF doctrine for AI substrate, cellular automation, and self-hostable delivery:

- ADR-0346 full-mirror semantics are migration input only: Cloud Provider acceptance must be evidenced by current cloud-ci/oya-ci Rust gate packets and promotion artifacts. The retired `./bin/oya verify --ci-required` path is historical/provenance-only and must not be invoked, recreated, or treated as merge/exit authority.
- ADR-0347 binds Cloud governance and CI-lane authoring to the `oya-governance-*` lane vocabulary after the `oya-governance-*` bulk rename. Enforced-by cross-reference: `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`.
- ADR-0348 binds Region, AZ, Cell, tenant placement, capacity rebalance, and shard-count automation to cellular topology that MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING as control-plane-driven automation modes. Enforced-by cross-reference: `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`.
- ADR-0349 is amended by ADR-0513/platform-readiness: Jenkins is bridge evidence only until cutover, ArgoCD/Rollouts remain authorized bridge/reference CD adapters where separately governed, and canonical readiness/promotion evidence comes from cloud-ci/oya-ci gate packets plus deployment/audit artifacts rather than Jenkins as destination CI authority.

## References

- docs/standards/documentation-rigor.md
- docs/personas/MASTER-ROSTER-2026-05-21.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0050-automation-first-pipeline.md
- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0708-platform-foundations-live-apex.md
- docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
- docs/adr-archive/ADR-0263-observability-emission-contract.md
- docs/adr-archive/ADR-0316-capability-tier-over-product-fragmentation.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- specs/products/cloud.json
- contracts/openapi/cloud/cloud-region-v1.yaml
- contracts/openapi/cloud/cloud-compute-vm-v1.yaml
- contracts/openapi/cloud/cloud-storage-object-v1.yaml
- contracts/openapi/cloud/cloud-iam-v1.yaml

## 2a. Acceptance criteria traceability (required)

This section is a planning-maturity contract only. It does **not** claim runtime, product-ready, or hyperscaler-ready status; promotion still requires fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| CLOUD-PRD-AC-001 | The Cloud PRD is used as a planning contract and region, cell, resource, IAM/KMS, audit, billing, and observability contracts are referenced by a promotion packet | The planned-maturity gate scans product PRDs | Cloud region/cell/resource acceptance is linked to test and evidence paths instead of generic prose | CLOUD-PRD-GATE-001 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |
| CLOUD-PRD-AC-002 | cloud-provider preview, stable, or GA readiness is evaluated | Readiness evidence is evaluated | fresh CI, SLO, security, SBOM, rollback/DR, cost, audit, billing, and product-pain evidence is required outside this PRD | CLOUD-PRD-GATE-002 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |

## 9b. Verification commands (required) — one runnable check per metric

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| Cloud region/cell/resource/audit/billing planning maturity | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | At least one Cloud row names region, cell, resource, audit, billing, and SLO/security obligations | `oya-ci-required` |
| Cloud product-ready and hyperscaler-ready non-claim boundary | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | A Cloud promotion packet cannot treat this PRD as hyperscaler-ready evidence without fresh CI/SLO/security/SBOM/DR proof | `oya-ci-required` |
