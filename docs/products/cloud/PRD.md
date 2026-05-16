---
doc_status: published
---

# Oyatie — Product PRD: Cloud Provider (AWS-class)

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
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
| **Cloud customer (enterprise)** | Direct interconnect, dedicated cells, BYOK / HYOK, Cedar-based IAM with SAML federation, signed audit log export, cross-region replication under explicit policy | Committed-use discount + per-resource overage; FinOps console |
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
| **W-Cloud-Preview** | VM service (KVM / Firecracker tenant compute), Kubernetes-as-a-service (managed), serverless / functions (limited language set), bare-metal lease, GPU fleet (per ADR-0044 hybrid), edge compute (limited PoP); Object store (S3-class), Block store (EBS-class), File store (EFS-class), Archive (Glacier-class), managed Postgres / Citus / pgvector / Redis / Kafka / ClickHouse; VPC + subnets, load balancers (L4 + L7), DNS (authoritative + recursive), CDN, direct interconnect, DDoS protection, service mesh integration; IAM + Account (Cedar policies, SAML/OIDC, STS, identity federation, MFA, audit); per-region per-AZ per-cell taxonomy; BYOK/HYOK KMS; per-resource-hour metering + per-region tax-invoice format; per-cell observability dataplane (audit log + SLO dashboards) — **all running canonical-architecture + first regional packs (KR-Seoul, JP-Tokyo, US-Virginia, EU-Frankfurt) in parallel** | `Cloud API v1` (control-plane REST + gRPC), `Cloud Console v1` (Leptos web), `Resource browser`, `IAM editor`, `Billing dashboard`, `Foundry capability surface` (cloud.compute.provision, cloud.iam.publish, cloud.region.register, etc.), KR CSAP path documented, audit-log export |
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
| `Foundry capability surface` (cloud.* mutators) | `product-control/capabilities/cloud.*.yaml` | control + audit | p99 ≤ 500 ms; every call audit-emits |
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
| Autonomy ceiling | Foundry | `oya-foundry-policy-kernel` | Governance + security |
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
| `cloud.budget_alert.v1` | `oya.cloud.billing` | `contracts/events/cloud.budget_alert.v1.avsc` | Tenant FinOps surface, Connect (notification) | 30 d | `(billing_account_id, alert_seq)` |
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
- **Dry-run gate**: Foundry fitness function `oya-foundry-fitness-migration` runs against synthetic 10k-resource per-region tenant before merge.
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
| Hot-path benchmarks | STS issuance (`p99 ≤ 100 ms`), Cedar evaluation (`p99 ≤ 5 ms`), object GET (`p99 ≤ 100 ms`), instance `provision-to-running` (`p95 ≤ 60 s`) — wired to `oya-foundry-fitness-bench` |
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

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review. GPL daemons (KVM/FRR) and AGPL extensions (Mimir) are allowed only at process boundary; the boundary is enforced by `oya-foundry-fitness-license` (ADR-0039).

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
| AGPL backend (MinIO) accidentally adopted | High | License-policy gate (`oya-foundry-fitness-license`) hard-fails any MinIO link or vendoring | Cloud + Foundry |
| Foundry mutator misuse (e.g. `cloud.iam.publish` over-broad) | High | Autonomy-ceiling-bound; T3 required for IAM mutation by default; per-mutator scoped capability schema; audit-chain on every invocation | Cloud + Foundry + Governance |
| KR CSAP attestation slipping past target | High | Parallel KR-pack workstream; contract with KR auditor signed pre-W-Cloud-Preview; controls evidenced via Foundry agents (HIPAA/KISA pattern per DESIGN §3) | Cloud-KR-pack team |
| Multi-AZ failover not actually exercised | High | Argo Rollouts (ADR-0050) progressive delivery + monthly forced AZ-failover drill; metric-gated rollback validated quarterly | Cloud + SRE |
| FinOps unit economics red at GA | High | Per-region per-tier rate-card with margin gate; per-tenant cost surfacing forces tenant-side optimization; cloud-team budget alerts on internal cost-of-revenue | Cloud + FinOps |
| Direct interconnect site lock-in | Medium | Per-pack `InterconnectPartner` impls; multi-IXP at each major region (KR: KIX+KINX; JP: JPIX+BBIX) | Cloud-network team |
| Tax-invoice format drift (regulator updates) | Medium | Versioned `TaxInvoiceFormatter` per pack; tax-pack changelog reviewed quarterly | Cloud-billing + regional-pack maintainers |
| Service mesh (Istio Ambient) maturity | Medium | Linkerd available as fallback (ADR-0044 preserved as drop-in); per-cell mesh upgrade gated on metric stability | Cloud + SRE |

## 11. Open questions

1. **Cloud axis pricing model at public-GA**: per-resource-hour AWS-style, or per-tenant-bundle Connect-style? (Same as PRD §8.) Default proposed: per-resource-hour with committed-use discounts.
2. **BYOK / HYOK at preview**: tenant-key escrow with KCMVP HSM as default for KR-pack; deferred for non-KR packs until W-Cloud-Stable. Confirm at council.
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

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
