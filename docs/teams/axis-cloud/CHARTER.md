---
doc_status: published
---

# Team: Axis — Cloud Provider

## Mission
This team owns Oyatie's cloud provider axis: compute, storage, network, IAM, billing, and observability — delivered as a public cloud service in a region-agnostic canonical architecture with per-locale regional packs. It exists because the cohesion thesis requires tenant data to reside in tenant-controlled cloud resources, with one billing trail and one IAM hierarchy spanning SaaS, search, ads, and agent runtime. It does **not** begin substantive work until the Data Use Boundary ADR is Accepted and Move #0 tenancy is established.

## Owned axes / surfaces / contracts
- **Axis(es):** Cloud provider (Axis 5)
- **Surfaces:**
  - `cloud-resource-kernel` — `ResourceType`, `ResourceId`, `ResourceSpec`, `ResourceState`
  - `cloud-region-kernel` — `RegionCode`, `AzId`, `CellId`, `ResidencyClass`
  - `cloud-iam-kernel` — `IamPolicy`, `StsToken`, `RoleArn`, `SsoProvider` (cloud-customer-facing)
  - `cloud-compute-*` — managed k8s, functions, VM, bare-metal lease, GPU
  - `cloud-storage-*` — object, block, KMS-shred, archive, backup
  - `cloud-network-*` — VPC, LB, DNS, CDN, interconnect, DDoS, mesh
  - `cloud-billing-kernel` — `BillingEvent`, `ResourceUsageMeter`, `TaxInvoiceFormat`
  - `cloud-observability-*` — audit log, SLO dashboards, distributed tracing
  - `cloud-iam-*` — Cedar + SSO + STS (cloud-customer surface; seam co-owned with `platform-tenancy-identity`)
  - Products owned: `products/cloud/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Cloud resource type` (owner) — cloud customers, tenant resource lifecycle, billing
  - `Region / AZ / Cell` (owner) — all axes read tenant residency from this
  - `IAM / SSO / SAML / OIDC IdP` (co-owner with `platform-tenancy-identity` for SaaS-facing IAM)
  - `Billing event` (co-owner with `axis-saas` metering and SaaS billing rail)
- **Catalog records:** `crates/cloud-*`
- **Runbooks:** `runbooks/region-failover.md`, `runbooks/cell-isolation-breach.md`, `runbooks/iam-key-rotation.md`, `runbooks/kcmvp-hsm-incident.md`
- **ADRs:** ADR-0044 (data residency — co-author), regional-pack cloud sections

## In-scope work
- Compute: managed Kubernetes (multi-AZ), serverless functions, VM fleet, bare-metal lease, GPU nodes
- Storage: object store (S3-compatible), block store, KMS-shred (tenant-key deletion proof), archive, database hosting (Postgres multi-AZ)
- Network: VPC, load balancer, DNS, CDN, DDoS mitigation, service mesh, interconnect
- IAM (cloud-customer-facing): Cedar policy engine instance, SSO federation, STS token issuance for cloud resources
- Billing: per-resource-hour metering, egress billing, per-API-call metering, regional tax-invoice format (via regional pack seam — `TaxInvoiceFormatter` trait)
- Observability: audit log surface, SLO dashboards, distributed tracing, per-tenant metrics
- Region/AZ/cell taxonomy: the `RegionCode` and `CellId` that every axis reads from `Tenant.region`
- Cell-isolation enforcement: tenant compute lives in a cell; cross-cell traffic is an explicit contract
- Multi-AZ failover automation (#214): quarterly non-prod drill, annual prod drill
- KR CSAP + K-ISMS-P + KCMVP HSM integration (W-Cloud-Stable gate)
- Marketplace: cloud resource marketplace for ISVs
- FinOps surfaces: per-tenant unit economics, cost-anomaly detection (co-owned with `ops-finops`)

## Out-of-scope (anti-scope)
- Hardware/chip/data-center construction (always — leased racks + colo only)
- SaaS application logic (→ `axis-saas`)
- Agent runtime (→ `axis-foundry` — Foundry runs *on* the cloud but doesn't provision cells)
- Search index shards (→ `axis-search` — search runs on cloud cells; cloud team provisions the cells)
- Per-tenant billing business logic (→ `axis-saas` metering for SaaS; cloud owns the cloud-resource billing event)
- Does NOT begin substantive work before: (a) Data Use Boundary ADR Accepted; (b) Move #0 tenancy established

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | `Tenant.region`, `Tenant.residency` for cell routing; IAM seam co-ownership | Per-release |
| `platform-privacy-dub` | Data Use Boundary ADR Accepted (hard gate) | ADR gate |
| `platform-audit-evidence` | Audit-chain emission for IAM mutations, resource provisioning | Per-release |
| `axis-foundry` | Capability invocation for control-plane agent operators; Foundry catalog gates | Per-release |
| `ops-dr-capacity` | DR drills, capacity planning input, region-failover automation | Quarterly |
| `ops-finops` | FinOps unit economics surfaces, cost-anomaly models | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-saas` | Compute cells for workflow execution, Postgres for OG storage | Wave gate |
| `axis-foundry` | Compute cells for daemon hosting, KMS for SecretProvider | Wave gate |
| `axis-search` | Compute cells for index shards, storage for index snapshots | Wave gate |
| `axis-ads-analytics` | Compute cells for auction, storage for attribution | Wave gate |
| All vertical teams | Tenant residency + regulatory-pack binding to cloud cells | Per vertical onboard |
| `platform-tenancy-identity` | `RegionCode` taxonomy for `Tenant.region` field | Monthly sync |

## Success metrics
- **Internal cloud control-plane uptime:** ≥ 99.9% on W-Cloud-Preview; ≥ 99.99% at W-Cloud-Stable (PRD §4.1)
- **Cell-isolation evidence collected:** 100% before any axis runs tenant workloads
- **Mean time to provision a new region:** ≤ 2 weeks post-W-Cloud-Stable IaC profile (PRD §4.2)
- **Multi-AZ failover drill success:** 100% quarterly in non-prod
- **KMS-shred proof-of-deletion:** 100% within 24 h of tenant off-board
- **Cross-axis contract violations (Region/AZ/Cell row):** 0 per quarter

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) — region/AZ/cell taxonomy changes
- Compliance: `ops-compliance` — CSAP/K-ISMS-P/KCMVP evidence pack
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 60-min sync — cell capacity, IAM incidents, billing reconciliation, DR drill schedule
- Cross-team review: monthly cross-axis contract audit for Region/Cell/IAM rows

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; IAM PRs require security-reviewer agent; region/cell PRs require multi-axis review label
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Cloud axis built before Move #0 tenancy | High | Hard gate: Data Use Boundary ADR + tenancy Move #0 must be complete |
| Cell-isolation breach — tenant workloads co-mingle | Catastrophic | Cell-isolation evidence required; `ops-security` review on isolation PRs |
| Region/cell taxonomy drift from tenant-kernel | High | Monthly sync with `platform-tenancy-identity`; fitness function cross-reference |
| KR CSAP audit failure at W-Cloud-Stable | High | `ops-compliance` owns evidence pack; cloud team provides raw evidence |

## Sources scanned
PRD.md §3.1 (W-Cloud-Preview, W-Cloud-Stable gates), §4.1 (uptime metric), §4.2 (region provisioning metric), DESIGN.md §1 (Axis 5), §10 (cloud resource, region/AZ/cell, IAM, billing event rows), products/cloud/PRD.md (draft).
