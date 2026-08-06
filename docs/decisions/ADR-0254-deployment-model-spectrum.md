---
id: ADR-0254
status: Accepted
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-security
  - council-compliance
  - ops-sre-reliability
  - ops-dr-capacity
  - ops-compliance
  - ops-security
  - axis-cloud
  - axis-tenancy
  - axis-deployment
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0121-on-prem-k8s-stack.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-multi-provider-mesh.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-cost-tag.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0180-stateful-disaster-recovery.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/deployment-models.json
  - /specs/microservices/deployment-control-plane.json
  - /specs/microservices/cell.json
  - /specs/microservices/cloud-iac.json
  - /specs/artifact-bundle-format.json
  - /specs/byo-cloud-onboarding.json
  - /specs/air-gap-bundle-delivery.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_bominal_inheritance_precedence
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_flat_product_catalog
  - feedback_canonical_base_localization
  - feedback_automate_everything
  - feedback_no_silent_regression
  - feedback_clean_architecture_requirements
  - feedback_workflow_objectgraph_adapter_layer
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 13-of-14
purpose: >
  Establish the canonical five-model deployment spectrum for the oyatie
  platform — shared-cloud (multi-tenant SaaS), dedicated-cloud
  (single-tenant cell operated by oyatie), hybrid / BYO-cloud (cell in
  tenant's own cloud account), on-prem connected (cell on tenant's own
  hardware with periodic sync), and on-prem air-gapped (cell on tenant's
  own hardware with cross-domain bundle delivery). All five models ship
  the same Helm charts, Cedar policy bundles, container images, and
  workflow definitions; the substrate beneath the cell varies, the cell
  contents do not. A new µservice
  `microservices/deployment-control-plane/` (Palantir Apollo equivalent)
  orchestrates upgrades, canary, rollback, and air-gapped bundle
  delivery across all five models.
enforcement_status: advisory-until-deployment-control-plane-lands
enforced_by:
  - oya gate validate deployment-model-coherence
  - oya gate validate artifact-bundle-signature
  - oya gate validate air-gap-bundle-manifest
  - oya gate validate per-model-slo-declaration
  - oya gate validate cell-topology-per-model
---

> **Disposition light-edit (2026-08-06):** Deployment model spectrum remains

# ADR-0254: Deployment model spectrum

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR. This
keystone is #13 of 14. Partial acceptance is rejected because the
deployment spectrum is meaningless without the tenant doctrine
(ADR-0242), sovereign cloud (ADR-0240), DR policy (ADR-0241), Cedar
gate (ADR-0243), compliance pack uniformity (ADR-0251), encryption-BYOK (ADR-0252),
and observability multi-tenant rollup (ADR-0253) keystones that the
spectrum depends on.

Enforcement is `advisory-until-deployment-control-plane-lands`. The
deployment-control-plane µservice is a new µservice
(`microservices/deployment-control-plane/` per ADR-0131 per-microservice
flat layout); validators promote to BLOCKER once:

1. `microservices/deployment-control-plane/` exists with bootstrap
   capability to enumerate cells, sign artifact bundles, and orchestrate
   per-cell canary deploys.
2. `.oab` (oyatie artifact bundle) format is locked at v1.0 with cosign
   attestation + SLSA L3 provenance and a verifiable offline test
   harness.
3. Per-cloud OpenTofu modules (AWS, GCP, Azure, Naver, KT, OVH, STC,
   AWS GovCloud, Azure Government) all expose the BYOC onboarding
   surface declared in §D-11.
4. At least one reference deployment of each model has succeeded:
   shared-cloud (the canonical oyatie SaaS instance per ADR-0242);
   dedicated-cloud (an internal preview tenant per ADR-0242 §D-8);
   hybrid (a BYOC dogfood deployment in a separate oyatie-owned cloud
   account); on-prem connected (the bootstrap cell per ADR-0247);
   on-prem air-gapped (a tabletop validation against the bundle delivery
   format).

Until those four conditions land, validators emit findings without
failing CI. Post-validation, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### Why a deployment spectrum exists at all

The oyatie platform serves customers whose deployment requirements
span a continuum of operational ownership and connectivity:

- A B2C consumer (a personal user of Mail, Drive, Calendar, Messenger,
  Notes, Recordings, Sites, Social, Community, Shorts, Tasks per the
  flat product catalog memory `feedback_flat_product_catalog`) expects
  the platform to be operated by oyatie, instantly available, with no
  infrastructure choice exposed.
- A small B2B tenant (a startup, a small agency, a community
  organization) expects a tenant boundary on a shared substrate;
  operational ownership belongs entirely to oyatie.
- A medium-enterprise B2B tenant (a regulated SaaS company, a
  regional bank, a regional healthcare provider) may require a single-
  tenant cell (per ADR-0009 cell architecture + ADR-0248 Amazon-shape
  cellular architecture) where data isolation is provable and
  noisy-neighbor risk is zero, but where infrastructure ownership still
  belongs to oyatie.
- A large enterprise (a Fortune 500 company, a national bank, a major
  hospital network) may require BYO-cloud — they own the cloud
  account, the infrastructure bill, the regulator relationship; oyatie
  ships the cell into their cloud and operates it under shared
  responsibility (the Snowflake BYOC + Confluent BYOC + Astronomer BYOC
  shape).
- A government / defense / intelligence customer with classified
  workloads (US DoD IL5 / IL6, UK MoD OFFICIAL-SENSITIVE / SECRET,
  KR NIS classified, KSA NDMO classified, EU Restreint UE / EU SECRET)
  may require on-prem deployment — the platform runs on the customer's
  own hardware in the customer's own datacenter or sovereign facility.
- Within on-prem, two further sub-modes exist: **connected** (the cell
  can periodically sync with oyatie's control plane for updates,
  observability rollup, and license attestation — per Palantir Apollo
  connected deployments + Anduril Lattice tactical edge + AWS Outposts
  connected variant); and **air-gapped** (the cell is physically or
  logically isolated from oyatie's network — updates arrive via signed
  artifact bundles delivered via cross-domain solution (CDS) one-way
  diode or removable media; audit-chain export flows back via the same
  channel — per Palantir Apollo air-gapped + IL5/6 + intelligence
  community reference deployments).

The portfolio's `feedback_quality_performance_scalability_bar`
memory establishes the bar as "industry leaders — Stripe / Palantir /
Linear" plus "hyperscaler-grade." Every named reference at that bar
implements a *subset* of this spectrum:

- **Salesforce** implements shared-cloud (multi-tenant SaaS) and a
  dedicated-edition tier for enterprise (Government Cloud, Health
  Cloud), but does not offer BYOC or on-prem.
- **Snowflake** implements shared-cloud, dedicated-cloud (Virtual
  Private Snowflake), and BYOC (Snowflake on AWS/Azure/GCP with
  customer account) — but does not offer on-prem.
- **Confluent** implements shared-cloud (Confluent Cloud), dedicated-
  cloud (Confluent Cloud Dedicated), BYOC (Confluent BYOC), and on-prem
  (Confluent Platform self-managed) — but does not offer air-gapped
  with first-class bundle delivery.
- **Palantir** implements all five — Palantir Foundry on Apollo
  (shared / dedicated / on-prem / air-gapped); Apollo is their
  deployment control plane.
- **GitHub** implements shared-cloud (GitHub.com), dedicated-cloud
  (GitHub Enterprise Cloud), on-prem (GitHub Enterprise Server), and
  air-gapped (GHES with manual update bundles via signed releases) —
  matching four of five.
- **HashiCorp** implements shared-cloud (HCP), on-prem (Terraform
  Enterprise, Vault Enterprise) — but BYOC is via HCP-on-customer-cloud
  which is a thin variant of shared-cloud.

oyatie's strategic positioning — per
`feedback_ecosystem_as_a_service_architecture` (ADR-0213) and
`feedback_canonical_base_localization` — requires the full five-model
spectrum. A regulator-facing tenant in KR-government can require
on-prem air-gapped today; the same tenant may onboard a B2C consumer
brand surface tomorrow via shared-cloud; oyatie cannot afford a
deployment-model gap that prevents either.

### Why the five models are concrete and not a continuum

A spectrum is sometimes mistaken for a smooth continuum. It is not.
The five models are *discrete* — each has a distinct operational
posture, distinct substrate footprint, distinct connectivity model,
distinct support SLO, and distinct compliance envelope. The discrete
character matters because operational runbooks, pricing tiers (per
ADR-0250), support contracts, and regulator evidence packets (per
ADR-0241 + ADR-0251) all bind to the discrete model identity.

Naming the five models concretely also resolves the long-standing
ambiguity in industry language: "hybrid cloud" means different things
to different vendors (AWS Outposts hybrid; Azure Arc hybrid; Anthos
hybrid; HCP hybrid; etc.). In this ADR, hybrid is **exactly**
BYO-cloud — the tenant brings their own cloud account, oyatie deploys
the cell into it. On-prem is exactly customer-owned hardware in a
customer-owned facility. The two are not the same and are not
conflated.

### Why the same architecture runs across all five models

The prior-portfolio temptation has been to ship different builds for
different deployment models — a "consumer build" for shared-cloud, an
"enterprise build" for dedicated-cloud, a "BYOC build" with API
exposure tweaks, an "on-prem build" with different defaults, and an
"air-gapped build" stripped of telemetry hooks. Every named hyperscaler
reference has tried this at some point and walked it back. The
walked-back lesson is consistent: maintaining N parallel codebases for
N deployment models is the fastest way to ship CVEs in M of N branches
because the security patch landed only in the N-th. Palantir's Apollo
talks at Palantir Forward (2023, 2024) explicitly discuss this — Apollo
ships **one** Foundry build across all deployment modes; the differences
live in the manifest layer above, not in the binary layer.

oyatie inherits that lesson by design: **one Helm chart set, one Cedar
policy bundle set, one workflow definition set, one container image
set, one artifact bundle format**, applied across all five models. The
substrate beneath the cell varies (cloud provider per ADR-0240, on-prem
hardware per the on-prem cell topology, air-gap operational envelope
per the cross-domain solution); the cell contents do not.

This is also a `feedback_no_silent_regression` requirement: a deployment
model is a public contract; any divergence in cell contents per model
is a silent regression vector. The single-build invariant is BLOCKER
day 1.

### Connection to other keystones in this bundle

This keystone (#13 of 14) presupposes:

- **ADR-0242 (oyatie-is-a-tenant, keystone #1):** Every tenant in every
  deployment model is a first-class tenant under the same tenancy
  substrate. `oyatie` is the canonical org-tenant; customer tenants
  live as siblings. Deployment model is a tenant property, not a
  µservice property.
- **ADR-0240 (sovereign cloud per regional pack):** Each deployment
  model that touches a cloud substrate (shared-cloud, dedicated-cloud,
  hybrid) binds to a regional pack's `sovereign_cloud_overlay.yaml`.
  On-prem models are pack-agnostic at the substrate level but apply
  pack overlays at the policy + compliance level.
- **ADR-0241 (DR + business-continuity portfolio policy):** Each
  deployment model carries a per-model SLO and DR tier profile.
  Air-gapped models have a degraded DR profile (best-effort,
  customer-operated).
- **ADR-0243 (Cedar as universal gate):** Cedar policy bundles are
  identical across deployment models; the differences are in how
  bundles are delivered (online sync vs offline bundle) and revoked
  (online vs CRL distribution).
- **ADR-0244 (tenant as universal scoping primitive):** Deployment
  model is one of the tenant's properties; per-tenant deployment-model
  declaration drives cell placement and substrate selection.
- **ADR-0245 (substrate vs product layering):** The deployment-control-
  plane µservice is substrate; the cells it deploys carry both
  substrate and product layers.
- **ADR-0246 (policy-engine substrate promotion):** Cedar fragments
  flow into bundles via the deployment control plane.
- **ADR-0247 (self-hosting / self-modification doctrine):** oyatie's
  own deployment-control-plane µservice deploys itself to oyatie's
  cells; the bootstrap path is documented.
- **ADR-0248 (Amazon-shape cellular architecture):** Each deployment
  model maps to a specific cell topology; the cell is the unit of
  deployment.
- **ADR-0249 (per-tenant data residency spectrum):** Residency
  constraints layer atop deployment model selection.
- **ADR-0250 (per-deployment pricing model):** Pricing varies per
  deployment model; ADR-0250 carries the details.
- **ADR-0251 (compliance pack uniform application):** Compliance packs
  apply across all five models; on-prem + air-gapped unlock IL5/6 +
  classified.
- **ADR-0252 (key-custody-BYOK everywhere canonical):** Customer key management
  applies uniformly across models, but air-gapped models require
  offline key ceremonies.
- **ADR-0253 (observability multi-tenant rollup):** Telemetry shipping
  varies per model (online for shared / dedicated; shared
  responsibility for hybrid; degraded sync for on-prem connected;
  bundled export for air-gapped).
- **ADR-0255 (intelligence substrate rewrite):** Intelligence substrate
  must operate across all five models; air-gapped requires local model
  serving (no cloud LLM API egress).

### Forcing functions that make this ADR necessary now (2026-05-20)

Three forcing functions:

1. **Three customer prospects in the funnel today require three
   different deployment models.** A KR-government prospect (CSAP-
   regulated) requires on-prem connected at minimum, with a path to
   air-gapped for classified work. A US-defense prospect requires
   air-gapped IL5 from day one. A Korean enterprise prospect requires
   BYO-cloud on Naver Cloud (their existing CSAP contract). Without
   this keystone, sales cannot give consistent technical answers, and
   engineering cannot estimate without per-prospect bespoke design.
2. **ADR-0242 + ADR-0240 + ADR-0241 collectively imply a deployment
   spectrum that no ADR currently names.** ADR-0242 assumes tenants
   exist; ADR-0240 assumes substrate per pack exists; ADR-0241 assumes
   per-µservice DR tier exists. None names *deployment model* as a
   first-class concept. The implicit assumption (that all deployments
   are shared-cloud) is incompatible with ADR-0240's sovereign-pack
   enforcement (which contemplates non-AWS substrate) and with the
   `feedback_autonomous_implementation_artifacts` requirement (which
   needs the masterplan to run without per-deployment-model bespoke
   work).
3. **The deployment-control-plane µservice has no doctrine to bind to
   today.** Palantir's Apollo equivalent must exist in the µservice
   catalog. Per ADR-0131 (per-µservice flat layout) and ADR-0132 (no
   grouping µservices), creating a new µservice requires the ADR that
   names its purpose. This ADR names that purpose.

## Decision

### D-1. The five deployment models

The oyatie platform supports exactly five deployment models. The set
is closed; additions require a new ADR.

#### D-1.1. Shared-cloud (multi-tenant SaaS)

**Definition.** oyatie operates the cell; multiple tenants share the
cell via shuffle sharding (per ADR-0248); cell substrate is one of
oyatie's contracted cloud providers per the cell's regional pack
(per ADR-0240). Each cell hosts hundreds-to-thousands of tenants
depending on tier and noisy-neighbor profile.

**Operational ownership.** oyatie owns: substrate provisioning,
capacity planning, upgrade orchestration, incident response, SLO
enforcement, compliance evidence emission, audit-chain operation,
billing aggregation. Tenant owns: their data, their workflow
definitions, their Cedar policy fragments above the platform default,
their integration endpoints.

**Connectivity.** Always online to oyatie's control plane. Tenant
access via public internet (with VPN + private link options).

**Identity envelope.** OIDC service principals per `tenant-<id>.*`
sub-scope (per ADR-0242 dotted hierarchy applied to customer tenants);
human users authenticate via passkey/WebAuthn (per ADR-0188).

**Typical customers.** B2C consumers; small B2B (< 100 users);
medium B2B (100-1000 users) with shared-tenant tolerance; non-regulated
SaaS workloads.

**Cell topology.** Tier 3 standard cell (per ADR-0248 cell tier
taxonomy). Tier 3 = full µservice catalog + tenant isolation guarantees
+ shuffle sharding + cross-AZ replication + cross-region warm standby
per ADR-0241 T2.

**Compliance envelope.** SOC 2 Type II + ISO 27001 + GDPR + KR PIPA
universally. Additional packs per cell's regional pack (CSAP for
KR-resident cells; GAIA-X-relevant overlays for EU cells; SDAIA for
KSA cells per ADR-0240).

**SLO.** Standard SLO per ADR-0241 — 99.95% availability [P5..P95:
99.85%..99.99%], < 200ms p99 read latency [modeled: 91ms p99;
P5..P95: 91ms–180ms], < 500ms p99 write latency [modeled: 201ms
p99; P5..P95: 201ms–450ms], < 5 min RTO for T1 µservices, < 1h
RTO for T2 µservices (evidence: modeling note
docs/performance-budgets/deployment-model-slo-budgets.md).

**Default deployment model.** This is the *default* model for any
tenant whose contract does not specify otherwise.

#### D-1.2. Dedicated-cloud (single-tenant cell, operated by oyatie)

**Definition.** oyatie operates the cell; exactly one tenant lives on
the cell (no shuffle sharding within the cell because there is only one
shuffle); cell substrate is one of oyatie's contracted cloud providers
per the cell's regional pack.

**Operational ownership.** Identical to shared-cloud — oyatie owns
substrate, upgrades, IR, SLO, compliance, audit-chain, billing.

**Connectivity.** Always online to oyatie's control plane. Tenant
access via dedicated private link (AWS PrivateLink / GCP Private
Service / Azure Private Link / Naver Cloud Private Endpoint
/ etc.) by default; public internet as fallback when private link
unavailable in a region.

**Identity envelope.** OIDC service principals per `tenant-<id>.*`
sub-scope; the cell is bound exclusively to one tenant. Cell-scoped
Cedar fragments may grant the tenant additional admin authority that
shared-cloud tenants cannot have (e.g., per-tenant LLM model pinning,
per-tenant feature-flag override).

**Typical customers.** Large enterprise B2B (1000+ users) without
on-prem requirement but with isolation requirement; regulated SaaS
without sovereignty requirement beyond what pack overlay provides;
mid-tier financial services; regulated healthcare without on-prem
mandate; defense suppliers up to IL2.

**Cell topology.** Tier 3 single-tenant cell. Substrate footprint
identical to a shared-cloud Tier 3 cell; the only difference is the
tenant assignment table contains exactly one row.

**Compliance envelope.** Shared-cloud envelope + tenant-specific
overlays (e.g., HIPAA-strict, PCI DSS Level 1, FedRAMP Moderate where
substrate allows). FedRAMP-High possible only on AWS GovCloud / Azure
Government substrate.

**SLO.** Same as shared-cloud, plus per-tenant SLO commitments may be
negotiated (tighter latency, dedicated capacity).

#### D-1.3. Hybrid / BYO-cloud (cell in tenant's own cloud account)

**Definition.** Tenant provides the cloud account (AWS / GCP / Azure /
Naver Cloud / KT Cloud / OVH Cloud / STC Cloud / etc., subject to
ADR-0240 per-pack provider catalog); oyatie deploys the cell into the
tenant's cloud account via IAM-delegated access; tenant owns the
infrastructure bill from their cloud provider; oyatie owns the
operation of the cell, the upgrades, the SLO, and the platform
license.

**Operational ownership.** Shared responsibility. Tenant owns:
substrate provisioning (executes OpenTofu modules in their cloud
account), infrastructure cost (their cloud bill), regulator
relationship with the cloud provider (CSAP contract with Naver,
GDPR DPA with AWS EU, etc.), cell-substrate networking. oyatie owns:
cell upgrade orchestration, Cedar policy distribution, IR for
platform issues, SLO for platform SLOs (with explicit caveats around
substrate-availability-out-of-oyatie's-control), license attestation.

**Connectivity.** Online to oyatie's control plane via IAM-delegated
access. Tenant grants oyatie an IAM role with bounded permissions
(deployment, upgrade, telemetry collection, IR access) per
deployment-control-plane's published role policy.

**Identity envelope.** Two-leg identity: tenant's cloud-provider IAM
governs the substrate; oyatie's tenancy substrate governs the
platform principals.

**Typical customers.** Large enterprise B2B with mandatory cloud-
provider relationship (e.g., a Korean conglomerate with a Naver Cloud
contract); fintech with sovereignty requirements beyond shared-cloud
provider; healthcare with HIPAA-strict + Customer-Managed-Key
requirements; defense suppliers with FedRAMP-High requirement (BYO into
AWS GovCloud); EU enterprises with GAIA-X-certified provider
requirement (BYO into OVH).

**Industry pattern source.** Snowflake BYOC (Bring Your Own Cloud,
2022) + Confluent BYOC (2023) + Astronomer BYOC (2024) + Databricks
Customer-Managed VPC (2021) + MongoDB Atlas Customer-Managed-Keys
(2020).

**Cell topology.** Tier 3 cell in customer cloud. The cell is
functionally identical to a Tier 3 shared-cloud cell — same Helm
charts, same Cedar bundles, same workflows, same container images.
The only difference is the substrate is the tenant's cloud account
under oyatie's IAM-delegated operational access.

**Compliance envelope.** Shared between oyatie and tenant. oyatie
attests to platform compliance (Cedar policy correctness, code
provenance via SLSA L3, audit-chain integrity, vulnerability
management). Tenant attests to infrastructure compliance (substrate
patching beyond oyatie's reach, network compliance, cloud-provider
DPA). Combined compliance pack varies per tenant deal.

**SLO.** Shared SLO with substrate-availability carve-out. oyatie
commits to platform SLO conditional on substrate availability. Tenant's
infrastructure SLO is their cloud provider's SLO.

#### D-1.4. On-prem connected (cell on customer hardware, periodic sync)

**Definition.** Tenant runs the cell on their own datacenter hardware
(per the reference architectures in §D-15); cell periodically syncs
with oyatie's control plane for upgrades, observability rollup, and
license attestation. Disconnection tolerance: 24 hours per ADR-0248
cell tier 2-3 invariant. Cell continues to operate during disconnection;
upon reconnection, observability events buffer-and-flush, audit-chain
syncs forward, upgrades resume.

**Operational ownership.** Tenant owns: hardware (PowerEdge / ProLiant
/ ThinkSystem / etc.), datacenter operation, network operation, storage
operation (Ceph or SeaweedFS cluster), Kubernetes upgrade operation.
oyatie owns: platform upgrade authoring (cell upgrades shipped as
signed bundles), Cedar policy distribution (via bundle), SLO advisory,
IR partnership.

**Connectivity.** Periodic sync — typically once per hour for routine
telemetry; once per day for license attestation; on-demand for upgrade
bundles. Sync goes through a tenant-controlled outbound proxy if the
tenant's network policy requires.

**Identity envelope.** Tenant's existing identity provider (Active
Directory, Okta, Azure AD, etc.) federates into the cell's tenancy
substrate via SAML / OIDC. oyatie's control plane identity is
separate; the tenant's IT operator has a `tenant-<id>.platform-admin`
principal.

**Typical customers.** Defense suppliers IL2-IL4; regulated healthcare
(HIPAA-strict, certain VA Hospital deployments); regulated finance
(certain regional bank deployments); KR government CSAP-on-prem;
KSA NDMO-on-prem; large enterprise with mandatory air-gap-tolerant
configurations.

**Industry pattern source.** Palantir Apollo connected (2022 onwards)
+ Anduril Lattice tactical edge (connected variant, 2023+) + AWS
Outposts (connected variant) + Azure Arc-enabled servers + Google
Anthos on-prem.

**Cell topology.** Tier 3 cell on customer hardware. Hardware spec
per §D-15. Storage via Ceph (preferred for IO-heavy) or SeaweedFS
(preferred for object-storage-heavy). Networking via Cilium with
optional Sidero Talos Linux as the K8s host OS.

**Compliance envelope.** Tenant-specific. Tenant typically holds
the regulator relationship (CSAP, HIPAA, IL4); oyatie attests to
platform compliance via signed evidence packets shipped during sync.

**SLO.** Best-effort SLO with disconnection tolerance. oyatie commits
to upgrade bundle availability within publication SLO (typically 4h
from upstream release). oyatie does not commit to per-tenant cell SLO
because the substrate is tenant-operated; oyatie offers SLO advisory +
runbook guidance.

#### D-1.5. On-prem air-gapped (cell on customer hardware, no network sync)

**Definition.** Tenant runs the cell on their own hardware; cell is
physically or logically isolated from oyatie's network (no outbound
TCP/IP route to oyatie infrastructure). Upgrades flow inbound via
signed `.oab` artifact bundles delivered through a cross-domain
solution (CDS) one-way diode or via removable media (signed USB,
optical disc). Audit-chain reconciliation flows outbound via the same
channel — periodic bundled export with Merkle-sealed signatures, merged
into oyatie's central audit-chain on next connection.

**Operational ownership.** Tenant owns everything: hardware,
datacenter, network, K8s, cell operation, IR, SLO, and the human
operations (operating the CDS, scheduling bundle delivery, performing
key ceremonies). oyatie owns: bundle authoring (signed artifact bundles
published to oyatie's bundle distribution endpoint), bundle integrity
attestation (cosign attestation + SLSA L3 provenance per bundle),
emergency advisory (via tenant's designated CDS escalation path).

**Connectivity.** None. Air-gap operational envelope strictly. No
oyatie-originated outbound TCP/IP traffic from the cell.

**Identity envelope.** Tenant's identity provider (typically a CAC/PIV
or hardware-token-based system for defense; sometimes an isolated
Active Directory forest for intelligence). Federation happens only at
the cell perimeter; no oyatie identity primitives cross the air gap.

**Typical customers.** US DoD IL5 / IL6; intelligence community
(IC ITE deployments); KR NIS classified; KSA classified; EU Restreint
UE / EU SECRET; UK MoD SECRET; defense primes operating classified
satellite ground stations; SCIF (Sensitive Compartmented Information
Facility) deployments.

**Industry pattern source.** Palantir Apollo air-gapped (multiple
classified deployments documented in Palantir's investor materials
2020-2024); Anduril Lattice classified variants; defense IL5/6
reference architectures (DoD Cloud Computing SRG); NSA cross-domain
solution guidance.

**Cell topology.** Tier 3 cell + bundled minimal Tier 2 control plane.
The Tier 2 control plane (deployment-control-plane subset + minimal
audit-chain ingestion + minimal observability collection) ships
inside the air-gap so the tenant has local upgrade orchestration even
without oyatie connectivity. Hardware spec per §D-15.

**Compliance envelope.** Tenant holds the regulator relationship
universally (oyatie has no visibility past the air gap). oyatie
attests to bundle integrity (cosign + SLSA L3); tenant attests to all
operational compliance.

**SLO.** Best-effort with quarterly support cadence. oyatie commits
to bundle publication SLO (typically monthly cadence + emergency
security bundles within 72h of CVE publication for critical
severity). Tenant's cell SLO is entirely tenant-operated.

### D-2. Same architecture across all five models

Every deployment model deploys the **same** primitives:

- **Same Helm charts.** Every µservice ships exactly one Helm chart
  (per ADR-0131 per-µservice flat layout). The chart's values file
  has per-model variation (`values-shared-cloud.yaml`,
  `values-dedicated-cloud.yaml`, `values-hybrid.yaml`,
  `values-on-prem-connected.yaml`, `values-on-prem-air-gapped.yaml`)
  but the *chart* is one chart.
- **Same Cedar policy bundles.** Every deployment loads the same Cedar
  policy fragment set (per ADR-0243 universal Cedar gate). Per-tenant
  fragments overlay on top per the tenancy substrate (per ADR-0244).
- **Same container images.** Every deployment runs the same OCI-
  compliant container images, content-addressed by digest, cosign-
  attested with SLSA L3 provenance.
- **Same workflow definitions.** Every deployment loads the same
  workflow catalog (per Workflow Engine PRD). Tenant-specific workflow
  customization layers on top via the tenant's workflow workspace.
- **Same audit-chain schema.** Every deployment emits audit events
  under the same schema (per ADR-0028); the delivery channel varies
  (online to central audit-chain for shared / dedicated / hybrid;
  buffered sync for on-prem connected; bundled export for air-gapped).
- **Same observability primitives.** Same OpenTelemetry instrumentation
  emission; same metric names; same trace schema; same log structured-
  fields. Delivery varies as above.

The differences between deployment models live in **two layers** only:

1. **The cell substrate.** Cloud-iac per-provider OpenTofu modules
   (per ADR-0240) for shared-cloud / dedicated-cloud / hybrid; on-prem
   hardware provisioning runbooks for on-prem connected / on-prem
   air-gapped.
2. **The control-plane connectivity.** Online sync for shared /
   dedicated / hybrid; periodic sync for on-prem connected; bundled
   delivery for air-gapped.

No other layer varies. A cell that is dropped into a tenant's BYO-cloud
runs identical bits to a cell in oyatie's shared-cloud SaaS.

This invariant is `feedback_no_silent_regression`-enforced. The CI
lane `deployment-model-coherence` checks that:

- No µservice has per-model conditional binary code paths (the
  configuration knob lives in Helm values, not in source code
  `#cfg(deployment_model = "air_gapped")`).
- No Cedar fragment is per-model-conditional (the fragment binds to
  tenant + cell + principal, not deployment model).
- No workflow definition is per-model-conditional.

### D-3. Cell topology per model

| Model | Cell tier (per ADR-0248) | Tenant assignment | Cross-AZ | Cross-region | DR profile |
|---|---|---|---|---|---|
| Shared-cloud | Tier 3 (standard, multi-tenant) | Shuffle-sharded; hundreds-to-thousands per cell | Yes (multi-AZ active-active) | Yes (warm standby per ADR-0241 T2) | Per ADR-0241 — T1 µservices < 5min RTO, T2 < 1h RTO, T3 < 4h RTO, T4 < 24h RTO |
| Dedicated-cloud | Tier 3 (standard, single-tenant) | One tenant per cell | Yes (multi-AZ active-active) | Yes (warm standby) | Same as shared-cloud per-µservice tier |
| Hybrid / BYO-cloud | Tier 3 (standard, in customer cloud) | One tenant per cell (tenant's own cloud account) | Yes (multi-AZ active-active where customer cloud supports) | Optional (per customer deal) | Per ADR-0241 with substrate-availability carve-out |
| On-prem connected | Tier 3 (standard, on customer hardware) | One tenant per cell | Yes (multi-rack via L2/L3 redundancy) | Optional (customer-owned DR site) | Best-effort SLO with disconnection-tolerant DR profile |
| On-prem air-gapped | Tier 3 + bundled Tier 2 control plane | One tenant per cell + isolated control plane | Yes (multi-rack) | Customer-operated DR (often via second air-gapped facility) | Tenant-operated DR; oyatie advisory only |

Cell tier 3 universally because tier 3 = full µservice catalog. Tier 0
(bootstrap) and tier 1 (infrastructure provisioning) cells are
internal-only (per ADR-0248) and not exposed as deployment models.
Tier 2 (control plane) cells exist in shared-cloud / dedicated-cloud /
hybrid as oyatie-operated infrastructure; in on-prem connected, a
hybrid model where the cell's Tier 2 control plane points at oyatie's
central tier 2; in air-gapped, a fully-local bundled Tier 2.

### D-4. Deployment control plane substrate (NEW µservice)

The deployment control plane is a new µservice:
`microservices/deployment-control-plane/` (per ADR-0131 per-µservice
flat layout; ADR-0132 no grouping µservices). It is Palantir Apollo's
functional equivalent for oyatie.

Responsibilities:

1. **Cell registry.** Maintains the catalog of all cells across all
   five deployment models. Each cell row carries:
   `{cell_id, deployment_model, tenant_id, regional_pack,
   substrate_provider | hardware_id, cell_tier, current_version,
   target_version, last_sync_at, sync_status, slo_profile, dr_tier,
   compliance_pack_set, cedar_bundle_version, contact_owner_team}`.
2. **Artifact bundle authoring.** Produces signed `.oab` bundles (per
   §D-5) from upstream release tags. Bundles are content-addressed
   by Merkle root; cosign-attested; SLSA L3 provenance attached.
3. **Per-cell upgrade orchestration.** For shared-cloud / dedicated-
   cloud / hybrid / on-prem connected, the control plane pulls each
   cell to the target version via per-cell upgrade workflows. For
   on-prem air-gapped, the control plane prepares the bundle for
   delivery (signed + manifested) and publishes to the bundle
   distribution endpoint; tenant operations retrieve via CDS.
4. **Canary + rollback.** Per-cell canary deploys via Flagger (per
   ADR-0040 progressive delivery); SLO-gated promotion (per ADR-0241
   tier targets); auto-rollback on SLO breach. Tenants can pin a cell
   to a specific version (override the upgrade pull) or schedule
   upgrades within a customer-defined maintenance window.
5. **License attestation.** Each cell periodically attests to its
   license via signed beacon (online for shared / dedicated / hybrid /
   on-prem connected; bundled for air-gapped). Attestation includes
   cell tier, tenant count, capability flags, upgrade version.
6. **Compliance evidence emission.** Per-cell evidence packets
   (per ADR-0241 + ADR-0251) emit on cadence; deployment-control-plane
   aggregates per-tenant.
7. **Substrate provisioning orchestration.** For shared-cloud /
   dedicated-cloud, the control plane provisions substrate via
   cloud-iac OpenTofu modules (per ADR-0240). For hybrid, the control
   plane invokes tenant-delegated OpenTofu runs. For on-prem,
   provisioning is tenant-operated; the control plane provides
   reference modules + runbook guidance.

The deployment-control-plane µservice itself ships in all five
deployment models (per ADR-0247 self-modification doctrine — oyatie's
control plane is itself a tenant of the platform; per ADR-0242 it's
the `oyatie.platform-ops` sub-scope). In on-prem air-gapped, the
deployment-control-plane ships as a subset (the local upgrade
orchestrator + manifest validator); the full control plane lives in
oyatie's primary cells.

Internal layout (per ADR-0131 per-µservice flat layout; each
bounded-context × layer pairing is a separate crate under
`microservices/deployment-control-plane/src/crates/`):

```
microservices/deployment-control-plane/
├── manifest.json
├── planned_contracts/
│   ├── openapi/deployment-control-plane.yaml
│   └── asyncapi/deployment-events.yaml
├── slos/
│   ├── bundle-publication.openslo.yaml
│   └── upgrade-canary.openslo.yaml
├── iac/
│   └── cell-provisioning/
├── migrations/
├── tests/
│   └── integration/
├── fragments/
│   └── deployment-control-plane.cedar
└── src/
    └── crates/
        ├── oya-cloud-deployment-control-plane-cell-registry-domain/
        ├── oya-cloud-deployment-control-plane-cell-registry-app/
        ├── oya-cloud-deployment-control-plane-artifact-bundle-domain/
        ├── oya-cloud-deployment-control-plane-artifact-bundle-app/
        ├── oya-cloud-deployment-control-plane-upgrade-orchestrator-domain/
        ├── oya-cloud-deployment-control-plane-upgrade-orchestrator-app/
        ├── oya-cloud-deployment-control-plane-upgrade-orchestrator-worker/
        ├── oya-cloud-deployment-control-plane-canary-controller-domain/
        ├── oya-cloud-deployment-control-plane-canary-controller-app/
        ├── oya-cloud-deployment-control-plane-license-attestation-domain/
        ├── oya-cloud-deployment-control-plane-license-attestation-app/
        ├── oya-cloud-deployment-control-plane-compliance-evidence-domain/
        ├── oya-cloud-deployment-control-plane-compliance-evidence-app/
        ├── oya-cloud-deployment-control-plane-provisioning-domain/
        ├── oya-cloud-deployment-control-plane-provisioning-app/
        ├── oya-cloud-deployment-control-plane-air-gap-domain/
        ├── oya-cloud-deployment-control-plane-air-gap-app/
        ├── oya-cloud-deployment-control-plane-air-gap-worker/
        ├── oya-cloud-deployment-control-plane-adapter/
        ├── oya-cloud-deployment-control-plane-rest/
        ├── oya-cloud-deployment-control-plane-grpc/
        └── oya-cloud-deployment-control-plane-worker/
```

Per-BC × layer table (all crates conform to ADR-0105 13-value layer enum and
BNF v4.1; µservice slot = `cloud`, BC tokens = `deployment-control-plane-<bc>`):

| BC | Layers present | Crate names |
|----|---------------|-------------|
| `cell-registry` | `domain`, `app` | `oya-cloud-deployment-control-plane-cell-registry-domain`, `…-app` |
| `artifact-bundle` | `domain`, `app` | `oya-cloud-deployment-control-plane-artifact-bundle-domain`, `…-app` |
| `upgrade-orchestrator` | `domain`, `app`, `worker` | `oya-cloud-deployment-control-plane-upgrade-orchestrator-domain`, `…-app`, `…-worker` |
| `canary-controller` | `domain`, `app` | `oya-cloud-deployment-control-plane-canary-controller-domain`, `…-app` |
| `license-attestation` | `domain`, `app` | `oya-cloud-deployment-control-plane-license-attestation-domain`, `…-app` |
| `compliance-evidence` | `domain`, `app` | `oya-cloud-deployment-control-plane-compliance-evidence-domain`, `…-app` |
| `provisioning` | `domain`, `app` | `oya-cloud-deployment-control-plane-provisioning-domain`, `…-app` |
| `air-gap` | `domain`, `app`, `worker` | `oya-cloud-deployment-control-plane-air-gap-domain`, `…-app`, `…-worker` |
| cross-cutting | `adapter`, `rest`, `grpc`, `worker` | `oya-cloud-deployment-control-plane-adapter`, `…-rest`, `…-grpc`, `…-worker` |

**Authority**: ADR-0131 §"Per-µservice flat layout" mandates
`microservices/<ms>/src/crates/<crate>/`; a flat `src/lib.rs` root is
non-conformant. Each bounded context owns its layers independently; no
shared `lib.rs` aggregating multiple BCs.

### D-5. Signed artifact bundle format (`.oab`)

The `.oab` format ("oyatie artifact bundle") is the canonical
distribution unit. One bundle = one platform version targeting one
cell tier set. Bundle structure:

```
release-2026-05-20-r1.oab           (tarball, gzip-compressed)
├── manifest.json                   (signed; declares contents + signatures)
├── container-images/               (OCI layout, cosign-attested)
│   ├── deployment-control-plane@sha256:...
│   ├── tenancy@sha256:...
│   ├── identity@sha256:...
│   ├── policy-engine@sha256:...
│   ├── audit-chain@sha256:...
│   ├── workflow-engine@sha256:...
│   ├── ontology@sha256:...
│   ├── intelligence-substrate@sha256:...
│   ├── observability/...
│   ├── (every other µservice)
├── helm-charts/                    (one chart per µservice + umbrella)
│   ├── deployment-control-plane-1.0.0.tgz
│   ├── tenancy-1.0.0.tgz
│   ├── (...)
│   └── oyatie-umbrella-1.0.0.tgz
├── cedar-policy-bundles/
│   ├── universal-fragments.cedar.bundle
│   ├── reserved-namespace.cedar.bundle
│   └── (...)
├── workflow-definitions/
│   ├── universal-workflows.bundle
│   └── (...)
├── compliance-packs/
│   ├── pack-soc-2.bundle
│   ├── pack-iso-27001.bundle
│   ├── pack-gdpr.bundle
│   ├── pack-kr-pipa.bundle
│   ├── pack-csap.bundle
│   ├── pack-hipaa.bundle
│   ├── pack-fedramp-moderate.bundle
│   ├── pack-fedramp-high.bundle
│   ├── pack-il5.bundle
│   ├── pack-il6.bundle
│   ├── pack-eu-ai-act-tier1.bundle
│   ├── pack-eu-ai-act-tier2.bundle
│   └── (...)
├── provenance/
│   ├── slsa-l3-provenance.json
│   ├── cosign-attestations/
│   ├── sbom-cyclonedx.json
│   ├── sbom-spdx.json
│   └── vulnerability-scan-results.json
└── signatures/
    ├── manifest.sig                (Ed25519, oyatie release key)
    ├── manifest.cosign.bundle      (cosign + Sigstore Rekor receipt)
    └── root-key-attestation.txt
```

Manifest schema (`manifest.json`):

```json
{
  "format_version": "1.0",
  "bundle_id": "release-2026-05-20-r1",
  "platform_version": "2026.05.20-r1",
  "release_channel": "stable" | "lts" | "edge" | "security-emergency",
  "build_timestamp": "2026-05-20T08:00:00Z",
  "build_provenance": {
    "slsa_level": 3,
    "builder_id": "oyatie-foundry-builder-prod",
    "build_invocation_id": "...",
    "source_repo": "github.com/oyatie/oyatie",
    "source_commit": "<sha>",
    "build_workflow": "foundry-release-build-2026-05-20"
  },
  "contents": {
    "container_images": [
      {"name": "deployment-control-plane", "digest": "sha256:...",
       "size_bytes": ..., "cosign_attestation": "..."}
    ],
    "helm_charts": [...],
    "cedar_bundles": [...],
    "workflow_bundles": [...],
    "compliance_packs": [...]
  },
  "dependencies": {
    "minimum_kubernetes": "1.30",
    "minimum_cilium": "1.16",
    "minimum_postgresql": "16.0",
    "minimum_ceph": "19.0",
    "minimum_openbao": "2.0",
    "minimum_zitadel": "3.0"
  },
  "compatibility_matrix": {
    "previous_versions_upgradable_from": ["2026.04.20-r1", "2026.03.20-r1"],
    "schema_migrations": ["0042_add_deployment_model_column.sql", "..."]
  },
  "deployment_model_targets": [
    "shared-cloud", "dedicated-cloud", "hybrid",
    "on-prem-connected", "on-prem-air-gapped"
  ],
  "merkle_root": "sha256:...",
  "signing_key_id": "oyatie-release-2026-key-7",
  "signature_algorithm": "Ed25519",
  "signature": "..."
}
```

Bundle verification is **fully offline**. A tenant in an air-gapped
environment can verify a bundle's integrity using only the bundle
itself plus the pre-distributed oyatie release public key chain. No
network access required.

Bundle verification steps:

1. Verify `manifest.sig` against the bundle's declared signing key,
   chained to oyatie's release root key (pre-distributed).
2. Compute the Merkle root of all contents and compare with
   `merkle_root` in manifest.
3. For each container image, verify cosign attestation against the
   bundled Sigstore Rekor receipt.
4. Verify SLSA L3 provenance is internally consistent.
5. Cross-check SBOM (CycloneDX + SPDX) for vulnerability scan
   recency.
6. (Optional, air-gapped) Verify against tenant's pre-cached Sigstore
   Rekor mirror.

### D-6. Air-gap one-way diode delivery

For IL5/6 and intelligence-community air-gapped deployments, updates
flow via a cross-domain solution (CDS). The CDS is operated by the
tenant (or a tenant-contracted CDS provider) per the NSA Raise the
Bar (RTB) guidance and the National Cross Domain Strategy and
Management Office (NCDSMO) approved product list.

Delivery topology:

```
[oyatie publish endpoint]
      |
      | publish signed .oab bundle
      v
[oyatie bundle distribution CDN]
      |
      | tenant downloads via cleared courier or VPN
      v
[tenant LOW-side staging server]
      |
      | bundle scan + signature verification on LOW side
      v
[CDS one-way diode (LOW -> HIGH)]
      |
      | one-way file transfer (no return path)
      v
[tenant HIGH-side ingestion staging]
      |
      | re-verify signatures on HIGH side
      v
[tenant HIGH-side cell deployment]
      |
      | upgrade applied via local deployment-control-plane
      v
[cell running new version]
```

Reverse direction (audit-chain export) flows via a separate CDS or
via removable media:

```
[tenant HIGH-side cell]
      |
      | periodic audit-chain export (signed bundle)
      v
[tenant HIGH-side export staging]
      |
      | bundle signed by tenant-held export key (NOT oyatie key)
      | classification review + sanitization per tenant policy
      v
[CDS one-way diode (HIGH -> LOW)] OR [removable media transfer]
      |
      v
[tenant LOW-side reconciliation staging]
      |
      | bundle uploaded to oyatie's audit-chain reconciliation endpoint
      v
[oyatie central audit-chain]
      |
      | merge against expected Merkle root; tamper detection
```

Critical properties:

- **Reverse channel is tenant-policy-gated.** Audit-chain export
  content + cadence is subject to tenant classification review.
  oyatie cannot pull export; tenant pushes when policy allows.
- **No two-way protocol.** The CDS is strictly one-way per direction.
  No request-response over the diode.
- **Signed bundles only.** No raw config files, no scripts, no
  unsigned data crosses the air gap in either direction.
- **Removable media is a supported fallback** for tenants without
  CDS infrastructure. Signed bundles on cosign-attested USB or
  optical media; tenant chain-of-custody documented.

### D-7. Update + rollback per model

Update mechanics vary by connectivity:

| Model | Update trigger | Canary | Rollback trigger | Pin-version |
|---|---|---|---|---|
| Shared-cloud | Pull by deployment-control-plane on release cadence | Flagger per-cell canary; SLO-gated promotion per ADR-0040 + ADR-0241 | Auto on SLO breach (latency, error rate, saturation per ADR-0176 brown-out signal) | Tenant can pin (paid feature, requires support ticket) |
| Dedicated-cloud | Pull by deployment-control-plane on tenant-defined cadence | Per-cell canary (50/50 or 25/75 traffic split during canary) | Auto on SLO breach; tenant can also trigger rollback via support ticket | Tenant can pin (default available) |
| Hybrid / BYO-cloud | Pull by deployment-control-plane with tenant IT approval on per-upgrade basis | Per-cell canary in tenant's cloud | Auto on SLO breach; tenant IT can trigger rollback via tenant-facing dashboard | Tenant can pin |
| On-prem connected | Pull by tenant's local deployment-control-plane subset within tenant-defined maintenance window | Per-cell canary; tenant IT can pause | Tenant-triggered or SLO-triggered (within tenant's monitoring) | Tenant pins; oyatie publishes recommended-version channel |
| On-prem air-gapped | Manual bundle retrieval by tenant operator; manual scheduling | Per-cell canary; full manual control | Tenant-triggered only (no oyatie visibility) | Tenant pins exclusively |

Progressive delivery primitives are identical across models (per
ADR-0040). What varies is who triggers the promotion / rollback and
how SLO signals reach the decider.

Critical security: an emergency security bundle (CVE response,
zero-day patch) ships on an expedited cadence:

- Shared / dedicated: < 4h from upstream patch availability to all
  cells.
- Hybrid: < 8h (additional time for tenant IT review window).
- On-prem connected: < 24h (tenant maintenance window).
- On-prem air-gapped: < 72h to bundle availability; tenant operator
  schedules delivery and application per their own SLA.

### D-8. Per-model SLO + support

| Model | Availability SLO | Latency SLO (p99 read / write) | RTO/RPO (T1 µservice) | Incident response | Support cadence |
|---|---|---|---|---|---|
| Shared-cloud | 99.95% [P5..P95: 99.85%..99.99%] (modeled; docs/performance-budgets/deployment-model-slo-budgets.md §1.1) | 200ms / 500ms [modeled: 91ms / 201ms p99; P5..P95: 91ms–180ms / 201ms–450ms] (docs/performance-budgets/deployment-model-slo-budgets.md §1.2–1.3) | < 5min / 0 RPO | 24x7 oyatie-operated | Standard (web ticket + chat + 24x7 hotline for paid tiers) |
| Dedicated-cloud | 99.95% (negotiated to 99.99% available) [same model basis as shared-cloud; single-tenant cell removes shuffle-sharding variance] | 200ms / 500ms (tighter on negotiated tier) [same model basis as shared-cloud] | < 5min / 0 RPO | 24x7 oyatie-operated; tenant-named escalation contact | Standard + dedicated TAM |
| Hybrid / BYO-cloud | 99.9% [P5..P95: 99.45%..99.9%] (substrate-availability-conditional; conditional on BYO substrate ≥ 99.9% independent availability) (modeled; docs/performance-budgets/deployment-model-slo-budgets.md §3) | 250ms / 600ms [modeled: 151ms / 261ms p99; P5..P95: 151ms–240ms / 261ms–590ms] (substrate-conditional; docs/performance-budgets/deployment-model-slo-budgets.md §2) | < 5min / 0 RPO when substrate available | 24x7 oyatie-operated for platform; tenant IT for substrate | Standard + dedicated TAM + shared-responsibility runbook |
| On-prem connected | Best-effort; oyatie SLO advisory only | Advisory only | Best-effort | Tenant-operated; oyatie partnership during business hours | Quarterly engagement + emergency hotline |
| On-prem air-gapped | Best-effort; tenant-operated | Tenant-operated | Tenant-operated | Tenant-operated; oyatie advisory via cleared channels | Quarterly engagement; emergency advisory via designated channel |

Shared-responsibility model for hybrid: tenant maintains substrate
(K8s version, node patching, network ACLs, IAM role lifecycle); oyatie
maintains the platform stack (Helm releases, Cedar bundles, workflow
definitions, audit-chain operation). The contractual responsibility
matrix lives at `docs/standards/hybrid-shared-responsibility.md`.

### D-9. Per-model pricing reference

Pricing is established in ADR-0250 (per-deployment pricing model);
this ADR cites the shape for completeness:

| Model | Pricing axis | Example billing event |
|---|---|---|
| Shared-cloud | Per-usage (per workflow run, per LLM token, per storage GB, per audit-chain event) | Tenant uses 1M LLM tokens / 10 GB storage / 1000 workflow runs in a month → invoiced per ADR-0250 unit rates |
| Dedicated-cloud | Per-cell base fee + per-usage | Tenant pays $X/month for the cell + per-usage |
| Hybrid / BYO-cloud | Per-cell platform-license fee + tenant's own cloud bill (paid directly to cloud provider) | Tenant pays $X/month platform license to oyatie + tenant's AWS/GCP/Azure/Naver bill |
| On-prem connected | Per-cell platform license + per-µservice license + support tier | Tenant pays $X/year platform license + support tier |
| On-prem air-gapped | Annual contract; perpetual license possible; support tier | Tenant pays $X/year contracted; support per cleared channel |

ADR-0250 carries the full pricing details; deployment-control-plane
emits per-cell metering events that feed FinOps (per ADR-0174) and
billing.

### D-10. Compliance per model

Compliance packs (per ADR-0251) apply uniformly. The deployment model
determines which packs are *feasible*, not which are *required* (the
tenant's regulator relationship determines required):

| Compliance pack | Shared-cloud | Dedicated-cloud | Hybrid | On-prem connected | On-prem air-gapped |
|---|---|---|---|---|---|
| SOC 2 Type II | Standard | Standard | Standard | Standard (tenant attests substrate) | Standard (tenant attests) |
| ISO 27001 | Standard | Standard | Standard | Standard | Standard |
| ISO 22301 | Standard | Standard | Standard | Standard | Standard |
| GDPR | Standard | Standard | Standard | Standard | Standard |
| KR PIPA | Standard | Standard | Standard | Standard | Standard |
| CSAP | Pack-bound (KR cells) | Pack-bound (KR cells) | Pack-bound (KR substrate) | Available | Available |
| HIPAA (standard) | Available | Available | Available | Available | Available |
| HIPAA-strict (PHI + customer-managed-key) | Limited | Available | Available | Available | Available |
| PCI DSS Level 1 | Limited (specific cells) | Available | Available | Available | Available |
| FedRAMP Moderate | Limited (specific cells) | Available (GovCloud cells) | Available (BYO into GovCloud) | Available | Available |
| FedRAMP High | Not available | Available (GovCloud cells) | Available (BYO into GovCloud) | Available | Available |
| IL4 | Not available | Limited | Available (BYO into IL4-cert cloud) | Available | Available |
| IL5 | Not available | Not available | Not available | Limited | Available |
| IL6 | Not available | Not available | Not available | Not available | Available |
| EU AI Act Tier 1-3 (per ADR-0144) | Standard | Standard | Standard | Standard | Standard |
| EU AI Act Tier 4 (high-risk classified) | Not available | Not available | Limited | Available | Available |
| Classified workloads (SECRET / TOP SECRET / SCI) | Not available | Not available | Not available | Not available | Available |

This matrix is illustrative; the canonical, authoritative matrix lives
at `docs/standards/compliance-pack-matrix.md` and updates as
certifications evolve.

### D-11. BYO-cloud setup

BYO-cloud setup is a tenant-onboarding workflow with three legs:

**Leg 1: Tenant prerequisites.** Tenant arrives with:
- A cloud account in one of the supported providers (AWS, GCP, Azure,
  Naver, KT, OVH, STC, AWS GovCloud, Azure Government per ADR-0240).
- Authority to grant oyatie an IAM role with the bounded scope
  documented at `docs/standards/byoc-iam-role.md`.
- A regulator-relationship-of-record with the cloud provider (their
  DPA / BAA / GDPR contract / CSAP contract / etc.).
- A network connectivity plan (private link, public-internet-with-VPN,
  or both).
- A maintenance window definition.

**Leg 2: Substrate provisioning.** Tenant runs the oyatie-published
OpenTofu module catalog for their provider:

```
# Example for Naver Cloud
module "oyatie_byoc_cell" {
  source = "git::https://github.com/oyatie/cloud-iac.git//iac/opentofu/naver/byoc-cell?ref=v2026.05.20"

  tenant_id                   = "tenant-acme-corp"
  cell_id                     = "tenant-acme-corp-cell-1"
  region                      = "kr-seoul"
  pack_id                     = "kr"
  oyatie_iam_role_external_id = "<provided-by-oyatie>"
  cell_tier                   = "T3-standard"
  observability_shipping      = "anonymized-and-cedar-gated"
  byok_key_ring_arn           = "arn:naver:kms:kr-seoul:..."
  maintenance_window          = "Sun 02:00-06:00 KST"
}
```

The module:
- Provisions VPC + subnets + security groups per oyatie's reference
  network architecture.
- Provisions the Kubernetes cluster (NKS for Naver, EKS for AWS,
  GKE for GCP, AKS for Azure, etc.) with the minimum K8s version
  declared in the latest `.oab` bundle manifest.
- Provisions Postgres / Redis / object storage / KMS per the per-
  provider canonical interface (per ADR-0028 + ADR-0240).
- Provisions an IAM role for oyatie's deployment-control-plane with
  bounded permissions.
- Provisions VPC endpoints / private link endpoints for control plane
  connectivity.
- Emits a `cell-ready` signal back to oyatie (signed beacon) once
  provisioning completes.

**Leg 3: Cell deployment.** Once the `cell-ready` signal arrives,
oyatie's deployment-control-plane:
- Assumes the tenant-granted IAM role.
- Deploys the latest stable `.oab` bundle via Crossplane CRDs (preferred)
  or direct Helm (fallback for providers without Crossplane provider).
- Registers the cell in the cell registry.
- Bootstraps the tenant's Cedar policy fragments + workflow definitions.
- Onboards the tenant's identity provider via Zitadel federation.
- Runs smoke tests; emits `cell-operational` signal on success.

Total tenant time-to-first-workflow target: < 4h for a green-field
deploy in a supported provider; < 24h for a first-time deploy in a
new provider variant.

### D-12. Observability + cost in BYO-cloud

In BYO-cloud, the observability stack runs in the tenant's cloud
account (the cell's full µservice catalog includes
`microservices/observability/` per ADR-0131 per-µservice flat
layout). Telemetry is shipped to oyatie's central control plane
under three constraints (per ADR-0253 observability multi-tenant
rollup):

1. **Anonymization.** Tenant-data-derived fields are hashed at the
   source; only structural telemetry (metrics, traces, logs with
   sensitive fields redacted) ships outbound. The redaction policy
   is itself a Cedar fragment loaded into the cell.
2. **Cedar-gated egress.** Each outbound telemetry shipment passes
   through a Cedar gate; the tenant can deny categories of telemetry
   (e.g., refuse all trace shipping but allow metric shipping). The
   default policy allows structural telemetry but never tenant-data.
3. **Per-pack residency.** Telemetry destined for oyatie's central
   control plane respects per-pack residency rules (per ADR-0240); a
   KR-cell's telemetry stays in KR-resident oyatie infrastructure.

Cost: the tenant's cloud bill is their own (paid directly to their
cloud provider). oyatie bills only the platform license fee per
ADR-0250. The deployment-control-plane emits cost-related metering
events (workflow runs, LLM tokens, storage GB-months, etc.) for
platform billing; the tenant's cloud provider bills them separately
for the substrate.

### D-13. Migration between models

A tenant can migrate between deployment models. This is rare and
planned (not an automatic on-demand operation); a typical migration
follows the workflow:

1. **Discovery.** Tenant declares migration intent (e.g., "we want
   to move from shared-cloud to BYO-cloud"). Tenant's account
   manager initiates a migration ticket.
2. **Plan.** A migration plan is authored by oyatie's solutions team
   in partnership with tenant. Plan declares: source model + cell;
   target model + cell; data migration approach; downtime budget
   (typically 0-4h depending on tenant's tolerance); rollback plan;
   compliance review (does the target model preserve all required
   compliance posture?).
3. **Pre-flight.** Target cell is provisioned (per the target model's
   provisioning workflow). Cedar policies + workflow definitions are
   replicated to the target. Identity federation is configured at the
   target.
4. **Migration execution.** Workflow Engine durably executes the
   migration saga:
   a. Quiesce writes on source.
   b. Snapshot all µservice state on source.
   c. Transfer snapshots to target (over private link / VPN /
      bundled media depending on target model).
   d. Replay snapshots into target µservices.
   e. Validate per-µservice data integrity.
   f. Cut over identity federation + DNS / endpoint pointers.
   g. Resume writes on target.
   h. Verify smoke tests.
5. **Rollback (if needed).** Workflow saga rolls back: identity
   federation + DNS reverts to source; writes resume on source;
   target cell is decommissioned.
6. **Post-migration.** Source cell is retained for a tenant-defined
   rollback window (typically 30-90 days), then decommissioned.

The migration saga lives in
`microservices/deployment-control-plane/src/migration_saga.rs`. It
emits per-step events to the audit chain.

Constraints:

- **Air-gapped to anything else.** Migration from air-gapped requires
  a one-time CDS export of all data (under tenant classification
  review) + import on the target side. This is non-trivial; oyatie
  provides bundled-export tooling; tenant operates the CDS.
- **Compliance preservation.** A migration that would weaken
  compliance posture (e.g., FedRAMP-High → FedRAMP-Moderate) requires
  council-compliance approval before execution.

### D-14. Air-gap audit-chain reconciliation

Air-gapped cells emit audit-chain events to the local audit-chain
µservice; the local chain is Merkle-sealed per ADR-0028 standard.
Periodic reconciliation with oyatie's central audit-chain:

**Outbound (HIGH → LOW):**

1. Local audit-chain emits a periodic export bundle (configurable
   cadence, typical: weekly or monthly per tenant policy).
2. Export bundle contains: every audit event in the period; Merkle
   root over the period; signature by tenant-held export key (NOT
   oyatie key — tenant controls what leaves the air gap).
3. Tenant classification review: the bundle is reviewed against
   tenant's classification policy; sanitization (e.g., removing
   classified workflow names, replacing with hashes) per tenant
   policy.
4. Bundle crosses CDS / removable media.
5. Bundle arrives at oyatie's reconciliation endpoint.

**Reconciliation:**

1. oyatie verifies bundle signature against tenant's pre-registered
   export key.
2. oyatie merges events into central audit-chain under
   `tenant-<id>.air-gap-cell-<cell-id>` sub-stream.
3. Merkle root of merged sub-stream is compared with bundled Merkle
   root for tamper detection.
4. On Merkle mismatch, alert fires to ops-security + council-
   compliance + tenant's designated escalation contact.

**Forward-fill on tenant request:** A tenant may request all events
within a period (e.g., for regulator audit); oyatie produces a signed
report listing all events with Merkle proofs.

Critical: the air-gap audit-chain reconciliation is *eventually*
consistent. There can be weeks-to-months of lag depending on tenant
cadence. The cadence is contractually negotiated.

### D-15. On-prem hardware requirements

Minimum cell hardware spec for on-prem (connected or air-gapped):

| Component | Minimum | Recommended | Notes |
|---|---|---|---|
| Compute nodes | 6 nodes, 32 cores + 256GB RAM each | 12 nodes, 64 cores + 512GB RAM | Mix of control plane (3) + worker (3+) per Tier 3 cell |
| Storage cluster | 3 nodes Ceph OR SeaweedFS, 50TB usable | 6+ nodes, 200TB usable, NVMe SSD | Erasure-coded (per Ceph 4+2 or SeaweedFS LRC default) |
| GPU (Intelligence substrate) | 0 (Intelligence falls back to CPU mode) | 4-8x NVIDIA H100 or H200 per GPU node | Required for local LLM inference per ADR-0255 air-gap requirement |
| Network | Cilium-compatible NICs; 10GbE minimum | 25GbE+; redundant ToR switches | BGP-capable preferred for advanced Cilium features |
| Management | iDRAC / iLO / Lenovo XCC for OOB management | Same | Per vendor |

Reference architectures published by oyatie:

- **Dell PowerEdge reference architecture** —
  `docs/standards/on-prem-reference-arch-dell-poweredge.md`. PowerEdge
  R760 compute nodes + PowerStore storage OR Ceph on PowerEdge.
  Validated for IL5 deployments with FIPS 140-3 module.
- **HPE ProLiant reference architecture** —
  `docs/standards/on-prem-reference-arch-hpe-proliant.md`. ProLiant
  DL380 compute + HPE Alletra storage OR Ceph on ProLiant. Validated
  for HIPAA-strict + FedRAMP-High.
- **Lenovo ThinkSystem reference architecture** —
  `docs/standards/on-prem-reference-arch-lenovo-thinksystem.md`.
  ThinkSystem SR650 compute + ThinkSystem DM storage OR SeaweedFS
  on ThinkSystem. Validated for KR CSAP-on-prem + NIS.
- **AWS Outposts reference architecture** —
  `docs/standards/on-prem-reference-arch-aws-outposts.md`. AWS-managed
  hardware in tenant facility; cell deploys via deployment-control-
  plane's Outposts adapter; available for connected-only (Outposts
  cannot operate air-gapped).
- **Air-gap defense reference architecture** —
  `docs/standards/on-prem-reference-arch-airgap-il5-il6.md`. Hardened
  PowerEdge / ProLiant + FIPS 140-3 modules + classified-network-
  certified switches; CDS integration; SCIF-deployable form factor.

Storage choice:

- **Ceph** preferred for IO-intensive workloads (Postgres backing,
  audit-chain WAL, observability metric storage); operator via Rook
  on Kubernetes.
- **SeaweedFS** preferred for object-storage-heavy workloads (Drive
  files, recordings, large dataset storage in Ontology); operator via
  the SeaweedFS Kubernetes operator.

Both are supported; tenants may choose based on their existing
operational expertise.

Networking:

- **Cilium** is the canonical CNI per ADR-0121 on-prem K8s stack.
  Provides eBPF-based policy enforcement + observability + service
  mesh integration.
- **Talos Linux** is the recommended host OS (immutable, declarative
  K8s host) for tenants without an existing Linux-host policy.
  Ubuntu LTS or RHEL is supported alternative.

GPU support for Intelligence substrate:

- Per-cluster GPU pool sized to tenant's expected inference load.
- For air-gapped: local LLM serving (no cloud API egress); typically
  Llama 4 / Qwen 3 / Mistral / Phi-class models hosted via vLLM
  per ADR-0255.
- GPU drivers + CUDA distribution ships in the `.oab` bundle for
  air-gapped (separate `gpu-driver-bundle.oab` due to size).

### D-16. Tenant onboarding workflow varies by model

Onboarding workflow is parameterized by deployment model. Steps that
vary:

| Step | Shared-cloud | Dedicated-cloud | Hybrid | On-prem connected | On-prem air-gapped |
|---|---|---|---|---|---|
| KYB depth | Standard (corporate identity verification) | Enhanced (financial check + reference customers) | Enhanced + cloud-provider contract verification | Enhanced + hardware procurement verification | Maximum (cleared personnel + facility security check) |
| Contract type | Standard ToS | MSA + dedicated-cell addendum | MSA + BYOC addendum + IAM role MOU | MSA + on-prem license addendum + support tier | Annual contract; cleared-channel addendum; SOW with detailed deliverables |
| Compliance pack negotiation | Default (SOC 2 + ISO 27001 + GDPR + KR PIPA) | Default + tenant-specific overlay | Default + tenant + cloud-provider compliance overlay | Tenant + sovereign + sometimes IL2-4 | Tenant-classified; oyatie ships full classified-pack-eligible set |
| Infrastructure setup | None (oyatie operates) | None (oyatie operates) | Tenant runs OpenTofu in their cloud (typically 2-4h) | Tenant procures + racks hardware (typically 4-12 weeks); oyatie partnership during setup | Tenant procures + racks + facility setup (typically 6-26 weeks including facility clearance) |
| Identity federation | OIDC / SAML setup to tenant's IdP | Same + per-tenant attribute mapping review | Same + cloud-provider IAM federation | Same + tenant Active Directory federation | Same + cleared-personnel attribute mapping; offline federation seeding |
| Smoke tests | Automated (< 30 min) | Automated + tenant-witnessed | Tenant-witnessed | Tenant-operated; oyatie advisory | Tenant-operated; oyatie advisory via cleared channel |
| Time-to-first-workflow | < 30 min | < 4h | < 4h (after cloud account ready) | Days-to-weeks (after hardware racking) | Weeks (after facility ready) |

Onboarding workflow definitions live in
`microservices/workflow-engine/workflows/tenant-onboarding/` with
per-model variants. Each variant is a workflow YAML loaded at platform
boot; tenants don't author these (oyatie's solutions team uses them).

## Alternatives considered

### Alt-1. Cloud-only (shared + dedicated only; no on-prem; no BYOC; no air-gap)

Restrict oyatie to cloud-only deployments. Operate the entire platform
as SaaS; offer dedicated-cloud for enterprise; refuse on-prem and
BYOC opportunities.

**Pros:**

- Smallest engineering surface — one substrate type (oyatie-operated
  cloud), no per-deployment-model complexity.
- Fastest velocity — no on-prem hardware reference architectures, no
  CDS integration, no per-cloud OpenTofu modules beyond oyatie's
  contracted providers.
- Familiar pattern — matches Stripe (cloud-only), Linear (cloud-only),
  Notion (cloud-only).
- Lowest support cost — no per-customer hardware support, no
  classified-channel operations.

**Cons:**

- **Excludes the entire defense / intelligence / classified market.**
  IL5 / IL6 / SECRET / TOP SECRET workloads cannot deploy to oyatie.
  This is non-trivial: the addressable market shrinks by a large
  portion in the US + EU + UK + KR + KSA + JP markets where defense
  + intelligence are major customer segments at the enterprise tier.
- **Excludes large enterprises with BYO-cloud requirement.** Many
  Fortune 500 companies have mandatory existing cloud-provider
  relationships (their CIO has signed a 7-figure contract with AWS /
  Azure / GCP / Naver) and cannot sign up for additional vendor
  cloud bills. Snowflake's BYOC is partially a response to this; not
  offering BYOC means losing those tenants to competitors who do.
- **Excludes KR + KSA + sovereign-cloud-strict tenants whose regulator
  requires on-prem or in-country dedicated substrate beyond what
  oyatie operates.**
- **Contradicts `feedback_ecosystem_as_a_service_architecture` (ADR-0213).**
  The ecosystem positioning explicitly contemplates oyatie as a
  participant in tenant-operated infrastructure (BYOC at minimum).
- **Doesn't match Palantir's actual shape.** Palantir's Apollo
  operates all five models; restricting oyatie to two means
  permanently positioning oyatie below Palantir in the addressable
  market.

**Rejected** because the addressable-market reduction is large
relative to the engineering surface saved.

### Alt-2. On-prem-only (Palantir Foundry classic shape; no cloud at all)

Restrict oyatie to on-prem deployments only. No shared-cloud SaaS;
no dedicated-cloud; no BYOC; only on-prem connected + on-prem
air-gapped.

**Pros:**

- Maximum sovereignty story — every tenant runs on their own
  hardware; no per-pack regulator concerns about oyatie's substrate.
- Matches Palantir Foundry's *classic* (pre-2020) shape closely.
- Single deployment-model concern engineering-wise (with the air-gap
  + connected sub-variants).

**Cons:**

- **Excludes the entire B2C consumer market.** Personal users of Mail,
  Drive, Calendar, Messenger, Notes, Recordings (per the flat product
  catalog) cannot run a personal on-prem cell. The B2C market is the
  largest addressable segment for an ecosystem.
- **Excludes small + medium B2B.** A startup cannot rack hardware for
  a 5-user tenant; an on-prem-only model excludes them.
- **Excludes SaaS-native enterprises.** Many enterprises *prefer*
  SaaS now (because operating on-prem K8s is expensive); on-prem-only
  loses them.
- **Doesn't match modern Palantir.** Palantir Foundry moved to Apollo
  (multi-model) in 2020; the on-prem-only era is over.
- **Contradicts the autonomous-implementation goal** (`feedback_autonomous_implementation_artifacts`)
  because autonomous bootstrap of the platform implies oyatie operates
  the platform's bootstrap cells (a shared-cloud-equivalent operation).
- **Loses cost efficiency.** Shared-cloud's multi-tenant economies of
  scale (per shuffle sharding per ADR-0248) is the only model that
  makes the per-user cost low enough for B2C; on-prem-only forces
  high per-user cost for every tenant.

**Rejected** because B2C + small/medium B2B markets are excluded;
this is the inverse of Alt-1's failure mode but with the same
structural issue (excluding a large market segment).

### Alt-3. Cloud + on-prem only (no hybrid / no BYOC)

Adopt shared-cloud + dedicated-cloud + on-prem connected + on-prem
air-gapped, but skip BYOC.

**Pros:**

- Four models instead of five — slightly smaller engineering surface
  (no per-cloud-provider OpenTofu modules tenant-side).
- Cleaner contractual model (oyatie owns the substrate everywhere
  except on-prem; on-prem has clear tenant ownership).
- Matches GitHub's shape (GHEC + GHES, no BYOC variant).

**Cons:**

- **Excludes BYOC-only enterprises.** A Fortune 500 with a mandatory
  Naver Cloud or AWS contract cannot use oyatie unless they're
  willing to rack hardware (on-prem) or use oyatie's substrate
  (shared/dedicated). Many will simply not adopt; lost market.
- **Snowflake, Confluent, Astronomer, MongoDB all offer BYOC.**
  Competitive positioning suffers — oyatie cannot win deals against
  these providers at the BYOC criterion.
- **EU enterprises with GAIA-X / OVH requirements** are reachable
  only via oyatie operating in OVH; if oyatie doesn't have a current
  OVH contract, the deal is lost. BYOC lets the tenant bring their
  OVH contract instead.
- **Per-pack sovereign-cloud overlay (ADR-0240) is partially
  defeated.** ADR-0240 contemplates per-pack provider catalogs;
  BYOC is the most natural way to onboard tenants whose required
  provider isn't in oyatie's standard contract set.

**Rejected** because BYOC is the lowest-cost addition to the spectrum
(reusing the same cell architecture + reusing per-cloud OpenTofu
modules from cloud-iac per ADR-0240) and unlocks a major market
segment.

### Alt-4. Cloud + dedicated only (Salesforce shape; no on-prem; no BYOC; no air-gap)

Adopt shared-cloud + dedicated-cloud only. Matches Salesforce's shape
(SFDC + Government Cloud + Health Cloud).

**Pros:**

- Single substrate type (oyatie-operated cloud).
- Matches a successful incumbent's shape.
- Smallest engineering surface among non-Alt-1 options.

**Cons:**

- **Same cons as Alt-1 for defense / intelligence / classified.**
  No on-prem means losing classified workloads.
- **Same cons as Alt-3 for BYOC enterprises.**
- **Salesforce is increasingly losing classified workloads to Palantir
  + Microsoft (Azure Government) + AWS (GovCloud)** precisely because
  Salesforce lacks on-prem. Repeating their pattern is repeating their
  market loss.

**Rejected** for the same reasons as Alt-1 + Alt-3.

### Alt-5. Five-model spectrum (shared / dedicated / hybrid / on-prem connected / on-prem air-gapped) ← **CHOSEN**

The selected alternative, fully specified in §Decision.

**Pros:**

- **Maximum addressable market.** Every tenant segment from B2C
  consumer to IL6 classified is reachable.
- **Matches every named hyperscaler reference at the bar.** Palantir
  Apollo + Confluent + Snowflake (4 of 5) + GitHub (4 of 5).
- **Reuses cell architecture (ADR-0009 + ADR-0248).** The cell is the
  unit of deployment; the cell's contents don't vary per model;
  engineering investment scales sub-linearly with model count.
- **Reuses sovereign-cloud overlay (ADR-0240).** Per-cloud OpenTofu
  modules serve both oyatie-operated cells and BYOC cells.
- **Reuses DR / BC portfolio (ADR-0241).** Per-µservice DR tier
  declaration is identical; per-model SLO variation lives in support
  contracts.
- **Reuses compliance pack uniformity (ADR-0251).** Same packs apply
  across all five models.
- **Reuses Cedar universal gate (ADR-0243).** Same Cedar fragments
  across all five models.
- **Single binary invariant** (per `feedback_no_silent_regression`) —
  no per-model code branches; configuration knobs only.
- **Buildable by intern with the spec.** Every model is concrete;
  every configuration knob is documented; every onboarding workflow
  has a step-by-step runbook.

**Cons:**

- **Operational complexity bounded but non-zero.** Per-model runbooks
  (5 runbook sets); per-model support cadence (5 cadences); per-model
  pricing schedules (5 schedules per ADR-0250). Mitigation: the
  per-model variation is in *configuration*, not in code; runbooks
  are versioned with the platform.
- **CDS integration is non-trivial.** Air-gap mode requires CDS
  integration which is a specialized engineering area. Mitigation:
  the CDS is tenant-operated; oyatie's responsibility ends at
  publishing signed `.oab` bundles to a public-internet-accessible
  bundle distribution endpoint.
- **On-prem hardware support is a new operational area.** Mitigation:
  reference architectures per vendor; partnership with Dell + HPE +
  Lenovo (and Naver/KT/STC for sovereign on-prem) for tenant-side
  hardware support; oyatie's responsibility ends at the platform
  Helm release.
- **Per-provider OpenTofu module catalog grows.** Mitigation: each
  module exposes the canonical interface per ADR-0028 + ADR-0240;
  the module-per-provider cost is bounded.

**Accepted** as the foundational keystone. The cons are bounded and
addressable; the pros include addressing the full addressable market
+ matching every named hyperscaler reference.

## Consequences

### Positive

1. **Full addressable market.** Every tenant segment from B2C consumer
   to IL6 classified is reachable. No structural exclusion.
2. **Single-binary invariant preserved.** Same Helm charts + Cedar
   bundles + container images + workflow definitions across all
   five models. CVE patching lands in one place.
3. **Reuses prior keystones.** Cell architecture, sovereign cloud,
   DR portfolio, compliance pack uniformity, Cedar universal gate
   all serve the spectrum without per-model duplication.
4. **Deployment-control-plane µservice unlocks autonomous
   deployment.** Per `feedback_autonomous_implementation_artifacts`,
   the deployment-control-plane is the substrate that lets the
   masterplan run without per-deployment-model bespoke engineering.
5. **Sales unblocked.** Three current prospects (KR-government,
   US-defense, Korean enterprise on Naver) can be sold with
   consistent technical answers.
6. **Palantir Apollo parity.** The five-model spectrum is the same
   shape as Apollo. Competitive positioning matches the named industry
   benchmark.
7. **Per-tenant deployment model declared at tenancy boundary.** Per
   ADR-0242 + ADR-0244, deployment model is a tenant property; the
   tenancy substrate carries it; no special-case logic anywhere
   downstream.
8. **Bundle format is offline-verifiable.** Air-gapped tenants can
   verify integrity without network access; cosign + SLSA L3 + Merkle
   root all bundled.
9. **Migration between models is durable.** Workflow-saga-based
   migration with per-step audit emission; rollback supported.

### Negative

1. **Five sets of operational runbooks.** Mitigation: per-model
   variation is configuration-only; runbooks share 80%+ content.
2. **Per-cloud OpenTofu modules grow.** Mitigation: canonical
   interface (per ADR-0028 + ADR-0240) caps per-provider work; one
   module per provider, not one per service.
3. **CDS integration is specialized engineering.** Mitigation: the
   CDS is tenant-operated; oyatie publishes signed bundles to a
   bundle distribution endpoint; tenant operates the diode.
4. **On-prem hardware lifecycle.** Hardware doesn't update like
   software; refresh cycles are 3-5 years; oyatie's platform must
   continue to support older hardware until the tenant refreshes.
   Mitigation: minimum-hardware-spec declared per `.oab` release;
   deprecation policy per ADR-0251.
5. **Per-model SLO matrix.** Five SLO profiles to track. Mitigation:
   per-µservice DR tier (per ADR-0241) carries the canonical RTO/RPO;
   per-model overlay is in the support contract, not the platform.
6. **Bundle distribution endpoint is a new public-internet surface.**
   Must be highly available, DDoS-resistant, geographically
   distributed. Mitigation: deployment-control-plane's bundle CDN
   is itself a Tier 2 µservice with T1 DR profile.

### Operational

1. **New CI lanes (advisory until validators land; BLOCKER post-validation):**
   - `oya-check-deployment-model-coherence` — verifies cell registry
     declares deployment model for every cell; verifies no µservice
     has per-model conditional binary code.
   - `oya-check-artifact-bundle-signature` — verifies every published
     `.oab` bundle is cosign-attested + SLSA L3 + Merkle-root-valid.
   - `oya-check-air-gap-bundle-manifest` — verifies bundle manifests
     for air-gap delivery are offline-verifiable.
   - `oya-check-per-model-slo-declaration` — verifies per-model SLO
     profiles are declared in the cell registry.
   - `oya-check-cell-topology-per-model` — verifies each cell's
     topology matches its declared deployment model.
2. **New µservice `microservices/deployment-control-plane/`** per
   ADR-0131 per-µservice flat layout.
3. **Per-cloud OpenTofu module catalog** under
   `microservices/cloud-iac/iac/opentofu/<provider>/byoc-cell/`
   per ADR-0240's per-provider module home.
4. **On-prem reference architecture standards** per §D-15.
5. **CDS integration runbook** per §D-6.
6. **Per-model onboarding workflows** in
   `microservices/workflow-engine/workflows/tenant-onboarding/` per
   §D-16.
7. **Per-model SLO declaration** in support contracts; not in code.
8. **Per-cell metering events** flow to FinOps (per ADR-0174) +
   billing (per ADR-0250).

### Sustainability

- **Per-model PUE differs.** Shared-cloud + dedicated-cloud + hybrid
  inherit per-provider PUE per ADR-0240 sustainability table. On-prem
  PUE is tenant-dependent (typical enterprise DC PUE 1.5-2.0; modern
  hyperscale 1.1-1.2). Air-gapped facilities (SCIFs) often have
  worse PUE due to additional cooling for compartmentalization.
- **GPU power consumption.** Intelligence substrate GPU usage scales
  with deployment model: shared / dedicated batch-share GPUs across
  tenants (highest utilization); hybrid + on-prem dedicate GPUs per
  cell (lower utilization); air-gapped has no batching across
  tenants (lowest utilization).
- **Refresh cycles.** On-prem hardware refresh every 3-5 years.
  Tenant-operated; oyatie advisory.
- Per ADR-0174 FinOps + sustainability tag, per-cell carbon footprint
  is visible regardless of deployment model.

### Compliance

- **Compliance pack uniformity preserved.** Same packs apply across
  all models (per ADR-0251); deployment model determines which packs
  are *feasible* (per §D-10 matrix), not which packs apply.
- **On-prem + air-gap unlock classified.** IL5/6 + SECRET / TOP
  SECRET workloads are reachable.
- **FedRAMP-High reachable.** Dedicated-cloud on GovCloud, BYOC into
  GovCloud, on-prem connected, on-prem air-gapped — all support
  FedRAMP-High.
- **GDPR Article 17 / KR PIPA Article 36 DSAR cascade** is uniform
  across all models per ADR-0242 oyatie-is-a-tenant doctrine; the
  cascade traverses the cell registry to enumerate all tenant data
  locations.
- **Regulator evidence packet (per ADR-0241 §D-8)** emits per-model
  variant; the packet structure is identical, the substrate provider
  + cell location fields vary.

## Implementation surface

The following artifacts are required for this keystone to be considered
implemented:

| Artifact | Status |
|---|---|
| `/specs/deployment-models.json` | NEW — derived from §D-1 |
| `/specs/artifact-bundle-format.json` | NEW — `.oab` schema per §D-5 |
| `/specs/byo-cloud-onboarding.json` | NEW — per §D-11 |
| `/specs/air-gap-bundle-delivery.json` | NEW — per §D-6 |
| `microservices/deployment-control-plane/` (new µservice per ADR-0131 + ADR-0132) | NEW — full µservice scaffold |
| `microservices/deployment-control-plane/manifest.json` | NEW |
| `microservices/deployment-control-plane/src/cell_registry.rs` | NEW — §D-4.1 |
| `microservices/deployment-control-plane/src/artifact_bundle_authoring.rs` | NEW — §D-4.2 |
| `microservices/deployment-control-plane/src/upgrade_orchestrator.rs` | NEW — §D-4.3 |
| `microservices/deployment-control-plane/src/canary_controller.rs` | NEW — §D-4.4 |
| `microservices/deployment-control-plane/src/license_attestation.rs` | NEW — §D-4.5 |
| `microservices/deployment-control-plane/src/compliance_evidence_aggregator.rs` | NEW — §D-4.6 |
| `microservices/deployment-control-plane/src/substrate_provisioning_dispatcher.rs` | NEW — §D-4.7 |
| `microservices/deployment-control-plane/src/air_gap_bundle_distribution.rs` | NEW — §D-6 |
| `microservices/deployment-control-plane/src/byoc_iam_delegation.rs` | NEW — §D-11 |
| `microservices/deployment-control-plane/src/on_prem_sync_handler.rs` | NEW — §D-1.4 |
| `microservices/deployment-control-plane/src/migration_saga.rs` | NEW — §D-13 |
| `microservices/deployment-control-plane/slos/bundle-publication.openslo.yaml` | NEW |
| `microservices/deployment-control-plane/slos/upgrade-canary.openslo.yaml` | NEW |
| `microservices/cloud-iac/iac/opentofu/aws/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/gcp/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/azure/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/naver/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/kt/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/ovh/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/stc/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/aws-gov/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/opentofu/azure-gov/byoc-cell/` | NEW |
| `microservices/cloud-iac/iac/on-prem/dell-poweredge/cell-reference/` | NEW |
| `microservices/cloud-iac/iac/on-prem/hpe-proliant/cell-reference/` | NEW |
| `microservices/cloud-iac/iac/on-prem/lenovo-thinksystem/cell-reference/` | NEW |
| `microservices/cloud-iac/iac/on-prem/aws-outposts/cell-reference/` | NEW |
| `microservices/cloud-iac/iac/on-prem/airgap-defense/cell-reference/` | NEW |
| `microservices/workflow-engine/workflows/tenant-onboarding/shared-cloud.yaml` | NEW |
| `microservices/workflow-engine/workflows/tenant-onboarding/dedicated-cloud.yaml` | NEW |
| `microservices/workflow-engine/workflows/tenant-onboarding/hybrid-byoc.yaml` | NEW |
| `microservices/workflow-engine/workflows/tenant-onboarding/on-prem-connected.yaml` | NEW |
| `microservices/workflow-engine/workflows/tenant-onboarding/on-prem-airgapped.yaml` | NEW |
| `docs/standards/deployment-model-spectrum.md` | NEW — full standards doc with worked examples |
| `docs/standards/artifact-bundle-format.md` | NEW — `.oab` reference |
| `docs/standards/byoc-iam-role.md` | NEW — IAM role schema for BYOC |
| `docs/standards/hybrid-shared-responsibility.md` | NEW — shared-responsibility matrix |
| `docs/standards/on-prem-reference-arch-dell-poweredge.md` | NEW |
| `docs/standards/on-prem-reference-arch-hpe-proliant.md` | NEW |
| `docs/standards/on-prem-reference-arch-lenovo-thinksystem.md` | NEW |
| `docs/standards/on-prem-reference-arch-aws-outposts.md` | NEW |
| `docs/standards/on-prem-reference-arch-airgap-il5-il6.md` | NEW |
| `docs/standards/compliance-pack-matrix.md` | NEW — per §D-10 |
| `docs/standards/air-gap-bundle-delivery.md` | NEW — CDS integration runbook |
| `docs/runbooks/deploy-byoc-cell-aws.md` | NEW |
| `docs/runbooks/deploy-byoc-cell-naver.md` | NEW |
| `docs/runbooks/deploy-on-prem-cell.md` | NEW |
| `docs/runbooks/deploy-on-prem-airgap-cell.md` | NEW |
| `docs/runbooks/air-gap-update-bundle-delivery.md` | NEW |
| `docs/runbooks/air-gap-audit-chain-reconciliation.md` | NEW |
| `docs/runbooks/migrate-between-deployment-models.md` | NEW |
| `microservices/observability/dashboards/deployment-model-spectrum.md` | NEW |
| Addition of CI lanes: `oya-check-deployment-model-coherence`, `oya-check-artifact-bundle-signature`, `oya-check-air-gap-bundle-manifest`, `oya-check-per-model-slo-declaration`, `oya-check-cell-topology-per-model` | NEW |
| Addition to `AGGREGATED_VALIDATE_LANES` (advisory) | NEW |

## Verification

- [ ] `microservices/deployment-control-plane/` exists with the §D-4
  module set; tests for `cell_registry.rs` pass on an empty database.
- [ ] `.oab` bundle authoring produces a bundle whose manifest is
  cosign-attested, has valid SLSA L3 provenance, and verifies offline
  with only the bundle + pre-distributed root key.
- [ ] Per-cloud OpenTofu modules (AWS, GCP, Azure, Naver, KT, OVH,
  STC, AWS GovCloud, Azure Government) exist with the canonical BYOC
  cell interface; `terraform plan` succeeds against a sandbox account
  per provider.
- [ ] Reference deployment of each model has succeeded at least once:
  - Shared-cloud (the oyatie SaaS instance).
  - Dedicated-cloud (an internal `oyatie.preview.*` cell per ADR-0242).
  - Hybrid (a BYOC dogfood cell in a separate oyatie-owned cloud
    account).
  - On-prem connected (the bootstrap cell per ADR-0247).
  - On-prem air-gapped (a tabletop validation against the bundle
    delivery format).
- [ ] `oya gate validate deployment-model-coherence` exits 0.
- [ ] `oya gate validate artifact-bundle-signature` exits 0.
- [ ] `oya gate validate air-gap-bundle-manifest` exits 0.
- [ ] `oya gate validate per-model-slo-declaration` exits 0.
- [ ] `oya gate validate cell-topology-per-model` exits 0.
- [ ] Air-gap audit-chain reconciliation test: a synthetic air-gapped
  cell exports a signed audit bundle; oyatie central audit-chain
  ingests the bundle; Merkle root matches; tamper detection
  detects synthetic tampering.
- [ ] Migration saga test: a synthetic tenant migrates from shared-
  cloud to dedicated-cloud and back; data integrity preserved at
  every step; audit events emitted per step.
- [ ] Per-model SLO declarations exist in the cell registry for every
  registered cell.
- [ ] Per-model onboarding workflows execute end-to-end against a
  sandbox tenant for each of the five models.
- [ ] CVE-response bundle publication SLO test: a synthetic critical
  CVE is filed; bundle publication time-to-availability meets the §D-7
  per-model expedited cadence.

## References

### Industry sources

- **Palantir Apollo product documentation** (palantir.com/platforms/apollo, 2022-2024).
  Apollo's multi-model deployment + air-gap delivery + per-cell
  upgrade orchestration is the canonical reference for this ADR's
  shape. Palantir Forward keynotes (2023, 2024) discuss Apollo's
  one-build-across-all-deployments invariant.
- **Snowflake BYOC documentation** (docs.snowflake.com/en/user-guide/intro-byoc,
  2022-2024). BYOC IAM-delegation pattern + customer-cloud-account
  ownership model.
- **Confluent BYOC documentation** (docs.confluent.io, 2023-2024).
  BYOC for Apache Kafka; customer VPC peering + dedicated cluster
  model.
- **Astronomer BYOC documentation** (docs.astronomer.io, 2024).
  BYOC for Apache Airflow; customer-account control plane delegation.
- **Databricks Customer-Managed VPC** (docs.databricks.com, 2021-2024).
  BYOC predecessor pattern; customer-managed VPC + Databricks-managed
  control plane.
- **MongoDB Atlas Customer-Managed Keys** (docs.atlas.mongodb.com,
  2020-2024). key-custody-BYOK + BYOC adjacent pattern.
- **AWS Outposts product documentation** (aws.amazon.com/outposts, 2019-2024).
  Connected on-prem variant; hardware-managed-by-AWS in customer
  facility.
- **AWS Snowball Edge** (aws.amazon.com/snowball, 2018-2024).
  Air-gap-capable storage + compute device; bundle-delivery pattern.
- **Azure Arc** (learn.microsoft.com/azure/azure-arc, 2020-2024).
  On-prem + multi-cloud connected pattern; Azure-managed control
  plane in customer infrastructure.
- **Azure Stack Hub + Azure Stack Edge** (learn.microsoft.com/azure-stack, 2017-2024).
  Connected on-prem variant; Azure Resource Manager extension.
- **Google Anthos** (cloud.google.com/anthos, 2019-2024).
  Multi-cloud + on-prem hybrid pattern.
- **Anduril Lattice product documentation** (anduril.com/lattice, 2023-2024).
  Tactical edge + connected/air-gapped variants for defense.
- **GitHub Enterprise Server** (docs.github.com/enterprise-server, 2014-2024).
  On-prem variant; signed release bundle delivery; air-gap-capable.
- **HashiCorp Terraform Enterprise** (developer.hashicorp.com/terraform/enterprise, 2017-2024).
  On-prem variant with signed updates.
- **Salesforce Government Cloud** (salesforce.com/products/government-cloud, 2014-2024).
  Dedicated-cloud variant for FedRAMP-High.
- **NSA Raise the Bar (RTB) Guidance for Cross Domain Solutions** (NSA Cybersecurity Advisory, 2020-2024).
  Authoritative source for CDS one-way diode + air-gap delivery.
- **National Cross Domain Strategy and Management Office (NCDSMO)
  Approved CDS Products List**. The set of CDS products tenants can
  use for air-gap bundle delivery.
- **DoD Cloud Computing Security Requirements Guide (SRG) v1r4** (DISA, 2017, updates 2020-2024).
  IL2-IL6 impact level definitions + reference architectures.
- **DISA STIGs for Kubernetes + Linux + Cilium**. On-prem hardening
  reference for IL4+.
- **Dell PowerStore + PowerEdge reference architectures** (dell.com/en-us/dt/solutions, 2023-2024).
  Hardware reference for on-prem cells; PowerEdge R760 for compute;
  PowerStore + PowerVault for storage.
- **HPE ProLiant DL380 + Alletra reference architectures** (hpe.com/us/en/servers, 2023-2024).
  Hardware reference; ProLiant DL380 Gen11 compute; Alletra storage.
- **Lenovo ThinkSystem SR650 + DM series** (lenovo.com/us/en/data-center, 2023-2024).
  Hardware reference; ThinkSystem SR650 compute; DM series storage.
- **SeaweedFS production deployments** (github.com/seaweedfs/seaweedfs, 2014-2024).
  Object storage cluster reference for on-prem.
- **Ceph production deployments** (ceph.io, 2012-2024). Block + object
  storage cluster reference.
- **Cilium production deployments** (cilium.io, 2018-2024). CNI
  reference for on-prem K8s.
- **Talos Linux** (talos.dev, 2020-2024). Immutable K8s host OS.
- **Sigstore Cosign** (sigstore.dev, 2021-2024). OCI image attestation.
- **SLSA (Supply-chain Levels for Software Artifacts) Level 3** (slsa.dev, 2022-2024).
  Build provenance attestation reference.
- **The Update Framework (TUF)** (theupdateframework.io, 2017-2024).
  Signed update distribution reference.
- **Open Container Initiative (OCI) Image Format Specification**
  (github.com/opencontainers/image-spec). Container image distribution.
- **CycloneDX SBOM** (cyclonedx.org, 2022-2024). SBOM format for
  bundle provenance.
- **SPDX SBOM** (spdx.dev, 2017-2024). SBOM format alternative.
- **Crossplane** (crossplane.io, 2018-2024). Cloud provisioning via
  Kubernetes CRDs; used for BYOC cell deployment.
- **Helm** (helm.sh, 2016-2024). Chart format for K8s deployments.
- **Flagger** (flagger.app, 2019-2024). Progressive delivery + canary
  for K8s; SLO-gated promotion per ADR-0040.
- **OpenSLO** (openslo.com, 2021-2024). Per-µservice SLO declaration
  format per ADR-0130.
- **OpenTofu** (opentofu.org, 2023-2024). Open-source Terraform fork;
  per-provider modules per ADR-0240.
- **Snowflake "How Snowflake Engineers Build Snowflake on Snowflake"** (snowflake.com/engineering blog, 2022).
  Dogfooding pattern reinforcing single-build invariant.
- **GitHub Actions OIDC for Cloud Deployment** (docs.github.com,
  2022-2024). Federated IAM pattern for BYOC.
- **Vercel Preview Deployments** (vercel.com/docs, 2018-2024).
  Ephemeral tenant pattern for preview environments.

### Regulatory sources

- **GDPR Articles 17, 28, 32, 44-50.** Data subject rights + processor
  obligations + cross-border data transfer; apply across all
  deployment models.
- **KR PIPA Articles 17, 36.** Equivalents.
- **CSAP (Cloud Security Assurance Program) v3.1** (KISA, MSIT).
  Pack-bound KR-substrate requirement.
- **K-ISMS-P** (KISA). Korean ISMS extension to privacy.
- **SDAIA Cloud Computing Framework v1.0** (KSA). Pack-bound KSA-
  substrate.
- **NDMO Cloud Sovereignty Requirements** (KSA NDMO, 2023). Pack-bound.
- **GAIA-X Trust Framework** (gaia-x.eu, 2022-2024). EU sovereignty
  reference.
- **EU AI Act (Regulation 2024/1689)** Articles 6, 9, 16-26.
  High-risk AI system requirements; tier per ADR-0144.
- **DoD Cloud Computing SRG IL2-IL6** (DISA, 2017-2024).
- **FedRAMP Moderate + High Baseline** (FedRAMP PMO).
- **FedRAMP High Equivalent for IL4+** (DoD CIO Memo, 2022).
- **HIPAA Security Rule + Breach Notification Rule** (45 CFR Parts
  160 + 164).
- **PCI DSS v4.0** (PCI SSC, 2022).
- **SOC 2 Type II Trust Service Criteria** (AICPA, updates 2022-2024).
- **ISO 22301:2019 Business Continuity Management Systems.**
- **ISO 27001:2022 Information Security Management Systems.**
- **FRCP 37(e) Failure to Preserve ESI.**
- **FAR + DFARS** for US government contracting (deployment-model
  selection often dictated by contract clauses).
- **METI Cloud Security Mark** (Japan). Pack-bound JP-substrate
  reference.

### Internal portfolio ADRs

- **ADR-0009** Cell architecture per-tenant per-region.
- **ADR-0010** Regional pack architecture.
- **ADR-0028** Cloud microservice architecture (per-provider canonical
  interface).
- **ADR-0040** Progressive delivery — canary + blue-green + metric-
  gated rollback.
- **ADR-0044** Inter-cell mesh tunnel.
- **ADR-0049** Cross-region replication + residency.
- **ADR-0105** Thirteen-layer canonical enum.
- **ADR-0121** On-prem K8s stack.
- **ADR-0128** Hyperscaler architecture invariants.
- **ADR-0131** Per-microservice flat layout.
- **ADR-0132** No-grouping forward policy.
- **ADR-0144** EU AI Act graduated risk tier model.
- **ADR-0145** Inter-microservice communication reform.
- **ADR-0148** Multi-provider mesh.
- **ADR-0150** Cedar policy engine.
- **ADR-0174** FinOps cost tag.
- **ADR-0176** Brown-out + degradation signal API.
- **ADR-0180** Stateful disaster recovery.
- **ADR-0183** Policy engine separation (Cedar app authz + Kyverno
  admission).
- **ADR-0211** In-house Rust-primary tech stack preference.
- **ADR-0212** Buildability doctrine.
- **ADR-0213** Ecosystem-as-a-service architecture.
- **ADR-0215** Multi-context platform.
- **ADR-0218** Tenant-granular control surface.
- **ADR-0240** Sovereign cloud per regional pack.
- **ADR-0241** DR + business-continuity portfolio policy.
- **ADR-0242** `oyatie`-is-a-tenant doctrine (keystone #1).
- **ADR-0243** Cedar as universal gate (keystone #2 — companion).
- **ADR-0244** Tenant as universal scoping primitive (keystone #3 —
  companion).
- **ADR-0245** Substrate vs Product layering (keystone #4 — companion).
- **ADR-0246** Policy-engine substrate promotion (keystone #5 —
  companion).
- **ADR-0247** Self-hosting / self-modification doctrine (keystone #6
  — companion).
- **ADR-0248** Amazon-shape cellular architecture (keystone #7 —
  companion).
- **ADR-0249** Per-tenant data residency spectrum (keystone #8 —
  companion).
- **ADR-0250** Per-deployment pricing model (keystone #9 — companion).
- **ADR-0251** Compliance pack uniform application (keystone #10 —
  companion).
- **ADR-0252** key-custody-BYOK everywhere canonical (keystone #11 — companion).
- **ADR-0253** Observability multi-tenant rollup (keystone #12 —
  companion).
- **ADR-0255** Intelligence substrate rewrite (keystone #14 — companion).

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — every deployment model has
  a tenant; deployment model is a tenant property.
- `feedback_bominal_inheritance_precedence` — this ADR overrides
  Bominal's implicit cloud-only assumption.
- `feedback_quality_performance_scalability_bar` — matches Palantir
  Apollo five-model shape.
- `feedback_autonomous_implementation_artifacts` — deployment-control-
  plane unlocks autonomous deployment across all models.
- `feedback_flat_product_catalog` — every product in the catalog
  serves all five deployment models.
- `feedback_canonical_base_localization` — per-pack localization
  applies across all models.
- `feedback_no_silent_regression` — single-build invariant prevents
  per-model code drift.
- `feedback_clean_architecture_requirements` — deployment-control-
  plane is a substrate µservice per ADR-0245.
- `feedback_automate_everything` — per-cell upgrade orchestration is
  automated; tenant-side BYOC provisioning is automated via OpenTofu.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the foundational keystone bundle,
every architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1.1 (Shared-cloud multi-tenant SaaS) | "Shuffle-Sharded Multi-Tenant SaaS" | AWS Builders' Library — *Workload isolation using shuffle sharding* (2017-2024); Salesforce Trust Cloud architecture | "Noisy-Neighbor Tenant Sprawl" — uncapped tenant placement on cells |
| D-1.2 (Dedicated-cloud single-tenant) | "Dedicated Cell Pattern" | AWS Outposts (dedicated variant); Salesforce Government Cloud; Snowflake Virtual Private Snowflake | "Shared Substrate Sovereign-Risk" — sovereign tenants on multi-tenant substrate |
| D-1.3 (Hybrid / BYO-cloud) | "Bring-Your-Own-Cloud (BYOC)" | Snowflake BYOC (2022); Confluent BYOC (2023); Astronomer BYOC (2024); Databricks Customer-Managed VPC (2021) | "Mandatory Vendor Cloud Lock-In" — tenant forced onto vendor's cloud contract |
| D-1.4 (On-prem connected) | "Connected Edge / Hybrid On-Prem" | AWS Outposts connected; Azure Arc; Google Anthos; Palantir Apollo connected; Anduril Lattice tactical edge | "Disconnected Forever On-Prem" — on-prem with no upgrade path |
| D-1.5 (On-prem air-gapped) | "Air-Gapped Bundle Delivery" | Palantir Apollo air-gapped; GitHub Enterprise Server with TUF; Anduril Lattice classified; defense IL5/6 reference architectures | "Online-Only Update Required" — air-gap incompatibility |
| D-2 (Same architecture across models) | "Single-Build Multi-Deployment" | Palantir Apollo one-Foundry-build; Snowflake's "build Snowflake on Snowflake" blog 2022; Confluent Platform vs Cloud single-codebase | "N Parallel Codebases" — per-deployment code branches; CVE patching gaps |
| D-3 (Cell topology per model) | "Cell as Unit of Deployment" | Amazon's cellular architecture (Werner Vogels, re:Invent 2018); ADR-0248 inheritance | "Pre-Cellular Deployment Unit" — service-level deployment without isolation |
| D-4 (Deployment control plane) | "Palantir Apollo Pattern" | Palantir Forward 2023+2024 keynotes; Apollo product page | "Per-Customer Bespoke Deployment Tooling" — no canonical deployment substrate |
| D-5 (`.oab` signed artifact bundle) | "TUF + Cosign + SLSA L3 Distribution" | The Update Framework spec; Sigstore Cosign 2021+; SLSA L3 specification | "Unsigned Distribution" — supply-chain attack surface |
| D-6 (Air-gap one-way diode) | "Cross-Domain Solution (CDS) Bundle Delivery" | NSA RTB guidance; NCDSMO approved CDS products list; DoD SRG IL5/6 reference | "Bidirectional Channel Across Air-Gap" — covert exfil risk |
| D-7 (Update + rollback per model) | "Flagger Progressive Delivery + Per-Model Pull Cadence" | ADR-0040 inheritance; Flagger production deployments | "Big-Bang Cross-Cell Update" — fleet-wide outage on bad release |
| D-8 (Per-model SLO + support) | "Tiered Support Matrix" | AWS Support tiers; Salesforce Premier vs Standard; Palantir Mission Support | "Single Support Tier" — under-served enterprise + over-priced B2C |
| D-9 (Per-model pricing — see ADR-0250) | "Cost-Aligned Pricing" | AWS On-Demand vs Reserved vs Savings Plan; Snowflake credit consumption; Confluent per-model SKU | "Single Pricing Across Heterogeneous Deployments" — cost-misaligned tenant base |
| D-10 (Compliance per model) | "Compliance-Pack Uniform Application" | ADR-0251 inheritance | "Per-Model Compliance Carve-Out" — uneven compliance posture |
| D-11 (BYO-cloud setup) | "IAM-Delegated Customer-Account Provisioning" | Snowflake BYOC IAM pattern; Databricks Customer-Managed VPC IAM | "Customer-Provides-Root-Credentials" — over-privileged access |
| D-12 (Observability in BYO-cloud) | "Anonymized Cedar-Gated Telemetry" | ADR-0253 inheritance; Snowflake telemetry shipping pattern | "Raw Tenant Data Egress" — privacy violation in BYOC |
| D-13 (Migration between models) | "Workflow-Saga Durable Migration" | AWS Step Functions saga pattern; Confluent Cluster Linking migration | "Lossy Migration" — tenant data corruption across models |
| D-14 (Air-gap audit-chain reconciliation) | "Merkle-Sealed Bundled Audit Export" | Palantir Apollo air-gap audit pattern; Bitcoin block reconciliation analogue | "Lost Air-Gap Audit Continuity" — tamper detection gap |
| D-15 (On-prem hardware requirements) | "Vendor Reference Architecture" | Dell PowerEdge / HPE ProLiant / Lenovo ThinkSystem reference architectures; AWS Outposts hardware spec | "Hardware-Agnostic Spec" — under-specified tenant procurement |
| D-16 (Tenant onboarding per model) | "Per-Model Onboarding Workflow" | Salesforce Onboarding flows; AWS Partner-Led Onboarding; Palantir Mission Specialist onboarding | "Generic Onboarding" — model-specific risk untracked |

---

## Appendix B: Worked example — air-gapped defense customer onboarding + first update bundle delivery

To illustrate that the air-gapped model is genuinely operable (not
just claimed), here is a worked example.

### Scenario

A US defense customer ("Acme Defense Systems, Inc.") has won a
classified program with the US Air Force at the IL5 impact level.
The program requires:

- Air-gapped deployment of the oyatie platform on Acme's SCIF-located
  hardware.
- Compliance pack set: DoD SRG IL5, FedRAMP-High, FIPS 140-3, NIST SP
  800-53 Rev 5 Moderate + High baseline, ITAR + EAR for export control.
- Cleared personnel only (SECRET + SCI as needed).
- Annual contract with 5-year option.

### Onboarding workflow

**Week 1-4: KYB + contract.**

- KYB depth: maximum. Acme Defense Systems' corporate entity verified;
  beneficial ownership traced; export-control review (no Foreign
  Ownership, Control, or Influence — FOCI — issues); cleared personnel
  rosters exchanged.
- Contract: annual subscription with 5-year option; cleared-channel
  addendum naming designated escalation contacts on both sides;
  SOW with detailed deliverables (hardware spec, software bundle
  versioning, support cadence, incident-response runbook).
- Compliance pack negotiation: DoD SRG IL5 + FedRAMP-High + FIPS 140-3
  + NIST SP 800-53 + ITAR/EAR. oyatie ships
  `compliance-pack-il5.bundle` + `compliance-pack-fedramp-high.bundle`
  in the `.oab`. Acme attests to all operational compliance; oyatie
  attests to platform compliance (bundle integrity, Cedar correctness,
  audit-chain operability).

**Week 5-16: Hardware procurement.**

- Acme procures Dell PowerEdge R760 compute nodes (6 nodes minimum,
  recommendation 12) with FIPS 140-3 validated cryptographic modules
  (Dell iDRAC FIPS mode enabled).
- Storage: Ceph cluster on PowerEdge R760xs (3 nodes minimum, 6
  recommended) with 200TB usable per the recommended spec.
- GPU: 4x NVIDIA H100 SXM5 per GPU node (2 nodes minimum) for local
  Intelligence substrate (per ADR-0255).
- Network: Cisco Nexus 9300 ToR switches with TLS 1.3 + FIPS-validated
  cipher suites.
- Talos Linux selected as the K8s host OS.
- Acme's facilities team racks the hardware in the SCIF; runs initial
  network burn-in.

**Week 17-20: Facility + clearance setup.**

- SCIF is certified for SECRET//SCI per DCID 6/9.
- Acme's personnel rosters are cleared for the program.
- CDS deployment: Acme procures a NCDSMO-approved one-way diode
  product (specific product name omitted; OWL Cyber Defense or
  Fox-IT family).
- LOW-side bundle ingestion staging server is provisioned (an
  unclassified-but-controlled subnet).

**Week 21-22: First bundle delivery.**

- oyatie publishes `release-2026-09-15-r1.oab` to the bundle
  distribution endpoint (cosign-attested, SLSA L3, ~85GB).
- Acme's cleared personnel (with appropriate program access)
  download the bundle to the LOW-side staging server via VPN.
- LOW-side signature verification: `oab verify release-2026-09-15-r1.oab`
  validates cosign + SLSA + Merkle root.
- Vulnerability scan on LOW-side: bundle SBOM checked against latest
  CVE feed; no critical/high findings.
- Acme transfers the bundle via CDS one-way diode from LOW to HIGH.
- HIGH-side re-verification: bundle signatures re-verified using
  pre-staged oyatie release public key chain.
- HIGH-side deployment: Acme's HIGH-side operator runs
  `deployment-control-plane bootstrap --bundle release-2026-09-15-r1.oab
  --cell-id acme-airgap-cell-1 --tenant-id tenant-acme-defense
  --deployment-model on-prem-airgapped --regional-pack us-classified`.
- deployment-control-plane (subset, bundled in `.oab`):
  - Reads bundle manifest.
  - Verifies all container image digests.
  - Deploys umbrella Helm chart with `values-on-prem-airgapped.yaml`.
  - Bootstraps tenancy substrate; creates `tenant-acme-defense`
    tenant row.
  - Bootstraps identity substrate; configures CAC/PIV federation to
    Acme's Active Directory (offline-seeded).
  - Bootstraps policy-engine; loads Cedar fragments including
    classified-pack fragments.
  - Bootstraps audit-chain; provisions HIGH-side audit log + signing
    key in OpenBao.
  - Bootstraps observability; configures HIGH-side metric retention
    (no outbound shipping).
  - Bootstraps workflow-engine; loads canonical workflows + Acme-
    specific workflows.
  - Bootstraps Intelligence substrate; pre-loads Llama 4 model
    weights from bundle (~35GB for the model bundle portion); vLLM
    starts.
- Smoke tests run; cell-operational signal emitted to HIGH-side
  monitoring.

**Week 23: First workflow execution.**

- Acme's HIGH-side users authenticate via CAC/PIV.
- First workflow: a classified data ingest pipeline that loads
  intelligence data from a HIGH-side data lake, runs vLLM-served Llama
  4 inference against the data, produces classified analytical
  outputs, stores results in HIGH-side object storage with classified-
  marking metadata.
- Audit-chain emits events under `tenant-acme-defense.cell-1.workflow.*`
  sub-stream.

**Week 26 (1 month later): First audit-chain reconciliation.**

- Acme HIGH-side audit-chain exports weekly bundle covering 4 weeks of
  audit events.
- Bundle signed by Acme's HIGH-side export key (pre-registered with
  oyatie at contract signing).
- Acme's classification review team reviews the bundle: workflow
  names containing classified program identifiers are replaced with
  hashes; tenant-data references are pseudonymized; non-classified
  audit events pass through unchanged.
- Reviewed bundle transferred via CDS HIGH → LOW.
- LOW-side: bundle uploaded to oyatie's audit-chain reconciliation
  endpoint.
- oyatie ingests bundle: signature verified; Merkle root matches;
  events merged into `tenant-acme-defense.air-gap-cell-1` sub-stream
  in oyatie's central audit-chain.
- No tamper detected; reconciliation complete.

**Week 28 (CVE response):**

- A critical CVE is filed against one of the container images in the
  bundle (e.g., a vulnerability in Postgres or Redis).
- oyatie's security team triages; builds an emergency security
  bundle `release-2026-10-13-r1-security.oab`.
- Bundle published to bundle distribution endpoint within 48h of CVE
  publication (well within the §D-7 air-gapped 72h target).
- Acme's cleared personnel download the security bundle (priority
  channel notification via cleared escalation path).
- Bundle traverses CDS LOW → HIGH (within Acme's 7-day SLA for
  critical security updates per the SOW).
- HIGH-side: `deployment-control-plane upgrade --bundle
  release-2026-10-13-r1-security.oab --canary 25` triggers a 25%
  canary deploy.
- Flagger monitors SLO during canary (per ADR-0040 + ADR-0241);
  promotes to 100% after canary success.
- Audit event `SecurityUpgradeComplete` emitted; included in next
  reconciliation bundle.

### What this worked example demonstrates

1. **Operational feasibility.** Every step is tractable; no
   hand-wave at "and then air-gap happens."
2. **CDS integration is tenant-operated.** oyatie's responsibility
   ends at publishing the signed bundle; Acme operates the diode.
3. **Single-build invariant preserved.** Acme runs the same
   container images, Helm charts, Cedar fragments as a shared-cloud
   tenant — just with different Helm values + air-gap delivery.
4. **Compliance is uniform-by-construction.** DoD SRG IL5 +
   FedRAMP-High + FIPS 140-3 are pack overlays loaded at deployment;
   they don't require code changes.
5. **Audit-chain continuity preserved across air gap.** Tenant
   classification review respects classification policy; merkle-
   seal preserves tamper detection.
6. **CVE response within SLO.** Even an air-gapped tenant gets
   critical security updates within a contractually-defined SLA.
7. **Per-tenant Intelligence substrate operates locally.** No cloud
   LLM API egress; Llama 4 weights ship in the bundle; vLLM runs on
   tenant's GPU.

Under the prior portfolio state (without this keystone), Acme's
deployment would have required:

- Per-customer bespoke deployment engineering for the air-gap mode
  (since no `.oab` format existed).
- Per-customer bespoke compliance attestation packaging (since
  compliance packs didn't bundle).
- Per-customer bespoke CDS integration (since no documented
  bundle-delivery pattern existed).
- Per-customer bespoke CVE response pipeline (since no expedited
  air-gap bundle cadence existed).
- An estimated 16-26 weeks of engineering investment before Acme's
  first deployment.

The keystone reduces that to a documented runbook + reuse of
existing primitives, with realistic engineering investment bounded by
the §Implementation surface table.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-cloud-deployment-control-plane-domain` | `domain` | `oya` · `cloud` · `deployment-control-plane` · `domain` | Domain logic for deployment control plane BC; `cloud` µservice slot; per ADR-0131 crate lives at `microservices/cloud/src/crates/oya-cloud-deployment-control-plane-domain/` |
| `oya-cloud-deployment-control-plane-app` | `app` | `oya` · `cloud` · `deployment-control-plane` · `app` | Application orchestration for deployment control plane BC |
| `oya-cloud-deployment-control-plane-adapter` | `adapter` | `oya` · `cloud` · `deployment-control-plane` · `adapter` | Adapters for K8s API / Talos API / cloud-provider APIs; `adapter` layer per ADR-0105 |
| `oya-cloud-deployment-control-plane-rest` | `rest` | `oya` · `cloud` · `deployment-control-plane` · `rest` | REST entry-point for deployment control plane per ADR-0105 `rest` layer |
| `oya-cloud-deployment-control-plane-grpc` | `grpc` | `oya` · `cloud` · `deployment-control-plane` · `grpc` | gRPC entry-point for deployment control plane; per ADR-0105 `grpc` layer |
| `oya-cloud-deployment-control-plane-worker` | `worker` | `oya` · `cloud` · `deployment-control-plane` · `worker` | Background worker for async artifact-bundle processing; `worker` layer per ADR-0105 |
| `oya-check-deployment-model-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `deployment-model-coherence` | Fitness-check; verifies cell registry declares valid deployment-model per §D-1; `oya-check-*` flat namespace |
| `oya-check-artifact-bundle-signature` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `artifact-bundle-signature` | Fitness-check; verifies every `.oab` bundle carries cosign signature per §D-5; `oya-check-*` flat namespace |
| `oya-check-air-gap-bundle-manifest` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `air-gap-bundle-manifest` | Fitness-check; verifies air-gap bundle manifests carry cryptographic bill-of-materials per §D-6; `oya-check-*` flat namespace |
| `oya-check-per-model-slo-declaration` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `per-model-slo-declaration` | Fitness-check; verifies per-model SLO declared in OpenSLO YAML per §D-8; `oya-check-*` flat namespace |
| `oya-check-cell-topology-per-model` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `cell-topology-per-model` | Fitness-check; verifies each cell's topology matches declared deployment model per §D-3; `oya-check-*` flat namespace |

---

*End of ADR-0254.*
