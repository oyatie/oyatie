---
id: ADR-0248
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-security
  - council-privacy
  - ops-sre-reliability
  - ops-dr-capacity
  - ops-compliance
  - ops-security
  - axis-cell
  - axis-cloud-iac
  - axis-cloud-k8s
  - axis-network
  - axis-observability
  - axis-tenancy
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
supersedes: []
amends:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
superseded_by: [ADR-700]
amended_by: [ADR-0333]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0121-on-prem-k8s-stack.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-sustainability-tag.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0253-network-topology-edge-service-mesh.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/cell-topology.json
  - /specs/microservices/cell.json
  - /specs/microservices/cloud-iac.json
  - /specs/cell-tier-definitions.json
  - /specs/cell-certification-levels.json
  - /specs/shuffle-sharding-parameters.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_canonical_base_localization
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_autonomous_implementation_artifacts
  - feedback_bominal_inheritance_precedence
  - feedback_automate_everything
  - feedback_flat_product_catalog
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 7-of-14
purpose: >
  Adopt the AWS-canonical cell-based architecture as the platform's
  canonical topology. Establish a four-tier model — Tier 0 (external
  dependencies), Tier 1 (bootstrap cell), Tier 2 (control plane
  cells), Tier 3 (data plane cells) — plus dedicated peer-tier
  service cells (marketplace, dev-tools, audit-aggregator,
  analytics) plus Tier 4 reserved for post-certification financial-
  grade + fulfillment-grade workloads. Mandate shuffle sharding for
  tenant→cell assignment, static stability for cell isolation
  tolerance, constant-work patterns for control plane sizing,
  Cloud Hypervisor + Kata Containers for sandboxing, and Cloudflare-
  to-Pingora at edge per ADR-0253. Cells are the universal blast-
  radius primitive; every workload runs in a cell; every cell is
  K8s-everything except the edge POP layer.
enforcement_status: advisory-until-cell-substrate-lands
enforced_by:
  - oya gate validate cell-tier-coherence
  - oya gate validate shuffle-sharding-parameters
  - oya gate validate cell-isolation-tolerance
  - oya gate validate cross-cell-traffic-permits
  - oya gate validate cell-deployment-pattern
  - oya gate validate static-stability-coverage
  - oya gate validate constant-work-control-plane
---

# ADR-0248: Amazon-shape Cellular Architecture

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR.
This is keystone #7 of 14. Partial acceptance is rejected because the
doctrines are mutually-reinforcing and produced together to avoid the
drift pattern that produced the ADR-0220 → ADR-0239 amendment cycle
within twelve days.

Enforcement is `advisory-until-cell-substrate-lands`: the doctrine is
accepted in text on 2026-05-20; the CI lanes that enforce it move to
BLOCKER status only after:

1. ADR-0333 successor owners exist with the canonical cell registry in
   cloud-iac OpenTofu state, the tenant-cell bindings in tenancy, and the
   pure `crates/oya-shuffle-sharding` assignment algorithm per §D-7 +
   §D-12.
2. `microservices/cloud-iac/` ships per-tier Helm charts at
   `microservices/cloud-iac/iac/helm/cell-tier-1/`,
   `cell-tier-2/`, `cell-tier-3/`, and one Helm chart per service-cell
   subtype.
3. The bootstrap cell self-retirement procedure (§D-2) has been
   exercised end-to-end on a clean install with audit-chain evidence
   replay.
4. The Tier 3 data plane cell template has shipped to at least two
   sovereign packs (KR + EU) per ADR-0240, each with shuffle-
   sharding parameters from §D-7 applied to onboarded test tenants.
5. The cellular-topology observability dashboard
   (`microservices/observability/dashboards/cellular-topology.md`)
   exists and surfaces per-cell + per-tier health.

Until those five items land, validators emit findings without
failing CI. Post-substrate, the lanes promote to BLOCKER per
ADR-0139 agentic-SLO-gated-promotion.

## Date

2026-05-20.

## Context

### James Hamilton's 2007 Cellular Architecture Thesis

James Hamilton's 2007 LISA paper *"On Designing and Deploying
Internet-Scale Services"* (USENIX LISA '07, pp. 233-244) is the
foundational text for hyperscale operational architecture. The paper
named eighteen "lessons learned" from Microsoft Live and Amazon
operations; the lesson that became known as **"design for failure"**
crystallised into the cell-based architecture pattern that AWS
internalised over the subsequent decade.

Hamilton's core claim: at scale, the only viable shape is one in
which the operational unit is **small enough that any single failure
of that unit is bounded in blast radius, but large enough that
operations can amortise the fixed-cost overhead of running the
unit**. He named this unit a *cell* (sometimes a *pod* in the 2007
text; AWS canonical terminology after 2013 settled on *cell*). A
cell is a complete, self-contained slice of the service stack —
compute, storage, control plane, observability, identity, network —
with no shared fate with other cells beyond a small set of explicit
narrow APIs.

The Hamilton 2007 principles directly relevant to this ADR:

- **Principle 5 (Recovery-oriented).** Every dependency must be
  assumed to fail; the service must recover without operator
  intervention. Cells enforce this because a cell's dependencies are
  almost entirely intra-cell.
- **Principle 7 (Avoid single points of failure).** No fate-shared
  resource between cells.
- **Principle 9 (Partition the service).** Explicit partitioning into
  cells; partition keys chosen so noisy-neighbor is bounded.
- **Principle 10 (Understand the network design).** Cell boundaries
  align with network failure domains.
- **Principle 18 (Operate as a small number of clusters).** Each
  cluster (cell) is self-contained; the count of cells is bounded so
  operations remain tractable.

The 2007 paper was prescient: the AWS architectural shape that
emerged from 2008-2018 (S3, DynamoDB, Lambda, all internal AWS
services) reflects Hamilton's principles essentially verbatim. AWS
service teams build cells; AWS managers compose cells; AWS customers
consume cells.

### Colm MacCárthaigh's 2014 Shuffle Sharding Article

Colm MacCárthaigh's AWS Architecture Blog post *"Shuffle Sharding:
Massive and Magical Fault Isolation"* (AWS Architecture Blog,
2014-08-19) introduced shuffle sharding as the mathematical
extension of cell-based architecture. The article is the canonical
public reference; the technique was already in use inside AWS
Route 53 (since ~2010) and was extended to many other AWS services
between 2014 and 2018.

The core insight: if a service has `C` cells, and each tenant is
randomly assigned to `S` of those cells (a *shard width* of `S`), the
probability that any two tenants overlap in **all** of their `S`
cells is combinatorially small:

```
P(full overlap between tenants A and B)
    = C(S, S) / C(C, S)
    = 1 / C(C, S)
```

With `C = 100` cells and `S = 8` shard width, the probability of
any two tenants sharing all 8 cells is:

```
1 / C(100, 8) = 1 / 186,087,894,300 ≈ 5.4 × 10^-12
```

That is, fewer than one in a hundred billion. The probability that
a single cell's failure affects any particular tenant is `S/C`; with
8/100, that is 8% per tenant per cell. But the probability that
a single cell's failure affects ALL of a particular tenant's cells
(i.e., the tenant goes fully offline) is `0` because the tenant has
`S - 1 = 7` other cells still operating in shuffle sharding.

MacCárthaigh's full claim: shuffle sharding with appropriate
parameters reduces the *expected fraction of impacted tenants* on a
single cell failure to a tiny fraction — typically less than 1% of
all tenants when `C = 100, S = 8`. Without shuffle sharding (i.e.,
`S = 1`, each tenant in exactly one cell), a single cell failure
affects `1/C = 1%` of tenants but those tenants are fully offline;
with shuffle sharding, `(S/C) × C / S = 100%` of tenants experience
*some* degradation, but the degradation is bounded to `1/S` of the
tenant's capacity. For most workloads, capacity-bounded degradation
is dramatically preferable to all-or-nothing outage.

For the platform, with planned 100 Tier 3 cells in steady state at
maturity and `S = 8` shard width per tenant, MacCárthaigh's math
applies directly: any single cell failure affects roughly `8%` of
tenants in a bounded-capacity way, but no tenant goes fully offline,
and no two tenants experience identical degradation patterns
(meaning failures don't correlate across the customer base).

### AWS re:Invent ARC408 (2018) + ARC405 (2024)

The AWS re:Invent 2018 session ARC408 *"Designing for Failure with
Cellular Architecture"* (Brad Calder, AWS Senior Principal Engineer,
re:Invent 2018, Las Vegas, November 28, 2018) is the canonical AWS
public talk on cellular architecture in production. The session
described how AWS S3, AWS Lambda, AWS Step Functions, and AWS DynamoDB
all use cellular architecture internally; how shuffle sharding is
applied to multi-tenant control planes; how cells are sized to
balance operational overhead against blast radius; and how cell
deployments are rolled out in waves to bound the blast radius of bad
deployments.

The 2024 follow-on session ARC405 *"Cell Architecture in Practice"*
(Tom Killalea + Colm MacCárthaigh, re:Invent 2024, December 3, 2024)
updated the 2018 framing with six years of additional production
data. Key 2024 updates:

- Cell counts at AWS have grown beyond the 2018 description; many
  services run 200-1000+ cells in production.
- The cell-as-deployment-blast-radius pattern is now used not only
  for runtime failure isolation but also for **deployment blast
  radius** — bad code only ships to one cell at a time; AWS Wave
  Deployment ensures that a regression caught in cell 1 never
  reaches cells 2-N.
- Shuffle sharding parameters have evolved: `S = 4` to `S = 16`
  depending on the workload's per-tenant traffic distribution; AWS
  publishes per-service shard-width recommendations.
- The *constant work* pattern (Brooker 2020, see below) is now
  considered as foundational as shuffle sharding for control plane
  sizing.

The 2024 session also introduced the *cell certification level*
concept (which feeds directly into ADR-0251 in this keystone bundle):
some cells are certified for financial-grade workloads (PCI-DSS),
others for healthcare (HIPAA), others for sovereign data (CSAP,
GAIA-X). Tenants route to cells whose certifications match the
tenant's compliance pack set.

The 2024 ARC404 session *"Static Stability at Hyperscale"* (Becky
Weiss + Mike Furr, re:Invent 2024, December 4, 2024) extended the
canonical static-stability article (see below) with operational
guidance: cells should tolerate 24-hour control-plane isolation
without dropping data-plane traffic; data-plane operations should
continue with last-known-good control-plane state if the control
plane is unreachable.

### Stripe Cells 2024

Stripe's 2024 engineering blog post *"Building cellular architecture
at Stripe"* (Stripe Engineering Blog, March 14, 2024;
stripe.com/blog/cellular-architecture) documents Stripe's adoption
of cellular architecture for their global payments substrate. The
key Stripe-specific learnings, complementary to AWS:

- **Payment-grade cells require synchronous regional replication.**
  Unlike most AWS services where async replication is acceptable,
  Stripe's payment cells use synchronous cross-AZ replication within
  a region; cross-region is async with bounded staleness.
- **Tenant→cell binding is per-Stripe-account, not per-API-call.**
  An account's home cell is sticky for its lifetime unless a planned
  migration moves it.
- **Cells are sized at 1,000-10,000 tenants in production.** This is
  a wider range than the AWS 2024 description; Stripe found that
  for payment workloads, larger cells better amortise the fixed
  cost of payment-network connectivity (Visa/Mastercard/Amex direct
  links per cell are expensive).
- **Cell failures observed in production are concentrated in three
  categories**: deployment defects (most common, mitigated by wave
  deployment), control-plane saturation (mitigated by constant-work
  pattern), and underlying-substrate failure (rare; mitigated by
  multi-AZ + cross-region active-passive).

Stripe's experience validates the AWS doctrine and provides
quantitative parameters for payments-grade cells specifically; this
ADR reserves Tier 4 for that workload class.

### Werner Vogels 2019 + Marc Brooker 2020 + Becky Weiss / Mike Furr "Static Stability"

**Werner Vogels Re:Invent 2019 Keynote.** The Amazon CTO's keynote at
AWS re:Invent 2019 (December 3, 2019) articulated the *cell* concept
to a mass-developer audience for the first time, in the context of
"how Amazon.com runs on AWS." The keynote framed cells as the
mechanism by which Amazon retail can operate as a tenant of AWS
without privileged paths — Amazon retail is sharded across multiple
AWS cells the same way third-party customers are sharded. This is
the structural basis for ADR-0242 (oyatie-is-a-tenant doctrine);
ADR-0248 extends it by making cells the universal isolation primitive
that ADR-0242 assumes exists.

**Marc Brooker "Constant Work" 2020.** Marc Brooker's AWS Builders'
Library article *"Reliability, constant work, and a good cup of
coffee"* (AWS Builders' Library, 2020; aws.amazon.com/builders-
library/) is the canonical reference for the **constant work
pattern**: a control plane should perform work proportional to the
*configuration* it manages, not to the *changes* it observes. A
control plane that pushes updates to all clients on every change
scales poorly; a control plane that publishes a versioned snapshot
which all clients periodically poll scales constantly regardless of
fleet size or change rate. Brooker's article gives the Route 53
health-check propagation as the worked example: rather than push
health-state deltas to every dataplane node, Route 53 publishes a
periodic snapshot of all health states which every node downloads;
control-plane work is constant per snapshot, regardless of how many
health-states changed or how many nodes are watching.

The constant-work pattern is foundational for the platform's Tier 2
control plane cells: per ADR-0246 + ADR-0243 the policy engine
publishes Cedar fragment bundles on a periodic cadence; data-plane
cells (Tier 3) pull bundles, eliminating the alternative of having
the policy engine push per-change deltas to each data-plane.

**Becky Weiss + Mike Furr "Static Stability."** The AWS Builders'
Library article *"Static stability using Availability Zones"* (Becky
Weiss + Mike Furr, AWS Builders' Library, 2020;
aws.amazon.com/builders-library/static-stability-using-
availability-zones/) defines the **static stability** pattern: a
system is statically stable if it continues to function correctly
when its control plane is unavailable, by using cached state or
pre-computed plans. The article gives the canonical example of
multi-AZ EC2: if the control plane (the EC2 control plane) becomes
unavailable, existing EC2 instances continue running; new instances
cannot be launched, but existing capacity continues serving traffic.

The platform's data-plane cells (Tier 3) must tolerate 24-hour
isolation from the control plane (Tier 2) with no data-plane
degradation. Cached Cedar fragments, cached tenant→cell bindings,
cached SPIFFE certificates (rotated more frequently than 24 hours
but tolerable for 24-hour outage), cached observability
configuration — all enable static stability.

### How the prior single-instance-per-µservice topology fails at scale

Pre-ADR-0248, the portfolio's deployment topology was implicitly:

1. Each µservice has one Deployment per cell.
2. Cells are per-tenant per-region per ADR-0009.
3. Tenant→cell binding is 1:1 (each tenant lives in exactly one
   cell; not shuffle-sharded).
4. The control plane is implicitly co-located with data plane;
   there is no explicit Tier 2.
5. Cross-cell traffic patterns are not classified; ad-hoc API
   calls cross cell boundaries when convenient.

This worked at the portfolio's then-current scale (sub-1000 cells,
sub-100,000 tenants) but fails at the planned 2027+ scale (5000-
20,000 cells across all packs; 1M+ tenants):

- **Single-cell-per-tenant binding** means a cell failure takes the
  tenant fully offline. With 1M tenants and a 99.99% per-cell
  availability target, that's ~100 tenant-equivalent-hours of full
  outage per month per cell; across 1000 cells, that's 100,000
  tenant-equivalent-hours per month of complete outage exposure.
- **No explicit control plane vs data plane tier** means control
  plane saturation propagates to data plane. A control-plane
  incident (e.g., Cedar fragment publication storm) directly impacts
  data-plane request serving.
- **Ad-hoc cross-cell traffic** means correlated failures: when cell
  A goes down and cell B's data-plane was relying on cell A's
  policy-engine read-path, cell B also degrades.
- **No Tier 1 bootstrap separation** means catastrophic loss
  recovery requires manual operator intervention rather than the
  deterministic bootstrap-cell → control-plane → data-plane sequence
  per ADR-0242 §D-5.
- **No shuffle sharding** means cell failures don't compose
  benignly; a single cell's saturation is a single tenant's full
  outage.

### Why cellular is the only sustainable shape per quality bar

Per `feedback_quality_performance_scalability_bar`, the platform's
quality bar is hyperscaler-grade: Stripe, Palantir, Linear quality;
AWS, Google, Microsoft, Apple, Cloudflare, Salesforce scalability.
Every named reference operates a cellular architecture in 2024-2025.
None operate a single-instance-per-µservice topology at the planned
scale. The choice is not whether to adopt cellular; the choice is
which cellular shape.

The AWS shape (four-tier model + service cells + shuffle sharding +
static stability + constant work) is the most documented, the most
externally observable, and the most cited at hyperscale conferences.
It is the right choice as the canonical model for the platform.

### Why now (2026-05-20)

Three forcing functions:

- **The 14-keystone bundle requires a topology.** ADR-0240
  (sovereign cloud per pack), ADR-0241 (DR + BC tiers), ADR-0242
  (oyatie-is-a-tenant), ADR-0243 (Cedar as universal gate), and
  ADR-0251 (compliance pack + cell certification levels) all assume
  a uniform cell topology underneath. Without ADR-0248, those
  assumptions are floating; with it, they have a substrate to bind
  to.
- **The autonomous-masterplan goal**
  (feedback_autonomous_implementation_artifacts) requires
  unambiguous deployment topology so that agents scaffolding new
  µservices know which tier they belong to, which Helm chart
  template applies, and which cell-routing patterns to follow.
- **The build-ahead-of-certification doctrine (ADR-0250)** requires
  Tier 4 reservation today so that future financial-grade and
  fulfillment-grade workloads have an architectural slot waiting,
  rather than being grafted into existing cells when certifications
  arrive (which would force a topology refactor under deadline
  pressure).

## Decision

The platform adopts AWS cell-based architecture as the canonical
topology. Sixteen decisions follow.

REGCLOUD-001 planning-artifact registration: the non-mutating
planning/spec artifact `plan/compliance-selective-cell-placement-architecture.md`,
its ownership seed `plan/OWNERS`, and its multispectrum evidence packet
`evidence/multispectrum/regcloud-001-compliance-placement-20260701-1782912506.json`
are registered as review/governance surfaces for compliance-selective
cell placement. This registration is not product/cloud implementation
authority and does not promote this ADR beyond Proposed/planning-impact
status.

### D-1. Tier 0 — external dependencies (migrable later)

Tier 0 comprises platform dependencies that are external to the
platform's own substrate. They are external because we do not control
their failure modes, but they are necessary for bootstrap:

- **Physical hardware.** Compute servers, network switches, storage
  arrays from cloud providers (per ADR-0240) or from on-prem
  hardware partners (Dell PowerEdge, Supermicro, OpenCompute Project
  hardware for sovereign packs). The hardware lifecycle is owned by
  cloud-provider SLAs or on-prem operations; we plan for hardware
  failure as a probability distribution.
- **DNS apex resolution.** The platform's apex DNS records (e.g.,
  `oyatie.example`, `api.oyatie.example`, per-tenant subdomains)
  resolve via anycast DNS provided initially by Cloudflare DNS, AWS
  Route 53, or NS1 (per ADR-0253 §D-1). Per ADR-0253, Year 3+ this
  layer migrates to a self-hosted PowerDNS-based authoritative DNS
  deployment running inside a dedicated dev-tools-cell.
- **Git host.** Source code lives in GitHub Enterprise initially;
  Year 3+ migrates to a GitHub (interim) deployment inside the
  `oyatie.foundry.*` sub-scope's dev-tools-cell.
- **Container registry.** Container images live initially in
  Cloudflare Registry / AWS ECR / GHCR; Year 3+ migrates to a self-
  hosted Harbor deployment.
- **Public certificate authority.** TLS certificates are issued by
  Let's Encrypt for public-facing endpoints initially; private PKI
  for intra-cell + intra-cluster certificates uses cert-manager +
  SPIFFE/SPIRE per ADR-0148. Public CA does not migrate (Let's
  Encrypt remains canonical for public CT-logged certificates).
- **Certificate transparency logs.** Google CT logs + Cloudflare CT
  logs + Let's Encrypt CT logs; not migrable (CT is by design a
  cross-organization public ledger).
- **Time sources.** NTP roots (NIST, KRISS, NICT, PTB, NMI per
  jurisdiction); not migrable.

Tier 0 is the platform's external attack surface and its external
dependency surface. The platform must continue to operate (in static-
stability mode per D-8) during any single Tier 0 outage; the platform
must be reconstructible from clean Tier 0 state in case of
catastrophic loss.

The list above identifies which Tier 0 items have a planned
migration path (DNS, git, registry, with Year 3+ targets per
ADR-0253 §D-17 + §D-18) and which do not (CAs, CT logs, time
sources, hardware).

### D-2. Tier 1 — bootstrap cell

Tier 1 is a single, minimal Kubernetes cluster that exists to
bootstrap the platform from zero. It contains only the substrates
needed for the bootstrap sequence per ADR-0242 §D-5:

- `cloud-secrets` (OpenBao + Shamir-shared root key unseal)
- `identity` (Zitadel; initial admin)
- `tenancy` (creates the `oyatie` tenant row)
- `policy-engine` (loads bootstrap Cedar fragments)
- `audit-chain` (provisions `oyatie` stream)
- `cell` (registers Tier 1 as bootstrap-class; tracks own
  retirement)
- `workflow-engine` (orchestrates the bootstrap)
- `foundry` substrate-meta (per ADR-0245 §D-3.A; performs the
  first build of Tier 2)

Properties of the bootstrap cell:

- **Single K8s cluster.** Provisioned via `kubeadm init` per the
  on-prem K8s stack (ADR-0121). For cloud deployments, an
  equivalent provisioning path (EKS bootstrap cluster, GKE bootstrap
  cluster) but with the same minimal substrate set.
- **Cilium CNI in CNI-and-L4-only mode** (per ADR-0148 layer
  ownership). No Istio Ambient layer at bootstrap; bootstrap traffic
  is loopback within the cluster.
- **No tenant data.** The bootstrap cell hosts only `oyatie` tenant
  (per ADR-0242). No customer tenant ever lands in the bootstrap
  cell.
- **No production traffic.** The bootstrap cell has no public
  ingress; only the bootstrap operator (one human + the bootstrap
  workflow agent) has access.
- **Limited HSM.** A single HSM partition for the bootstrap
  workflow's signing key; no per-tenant HSM partitions.
- **Self-retiring.** After the bootstrap sequence completes (Tier 2
  control plane cells are up + `oyatie` tenant audit stream is
  initialised + Tier 3 data plane cells exist for the first
  customer tenant), the bootstrap cell **self-retires**. The
  retirement procedure:

  1. Audit-chain emits `BootstrapCellRetirementInitiated` event,
     signed by the bootstrap key (which itself was signed by the
     org root key).
  2. The bootstrap cell migrates the `oyatie` tenant's home_cell
     binding to a Tier 2 control plane cell.
  3. The bootstrap cell drains all running workloads; each
     substrate µservice acknowledges via audit-chain.
  4. The bootstrap cell's namespaces are torn down; the K8s
     cluster's etcd snapshot is sealed to OpenBao under a key
     held by `oyatie.security`.
  5. The bootstrap cell's K8s cluster is deprovisioned; the
     hardware (or cloud resources) released.
  6. Audit-chain emits `BootstrapCellRetirementComplete`.
  7. A `BootstrapCellRetirementReceipt` artifact lands in
     `evidence/bootstrap/retirement-<timestamp>.json` with the
     Merkle proof of the retirement audit-chain segment.

  Recovery from catastrophic loss requires a new bootstrap cell to
  be provisioned and the prior etcd snapshot replayed; the
  procedure is documented in
  `docs/runbooks/oyatie-bootstrap-recovery.md` per ADR-0242.

### D-3. Tier 2 — control plane cells

Tier 2 control plane cells host the platform's authoritative state
and the substrates that publish policy / topology / identity / audit
configuration to Tier 3 data plane cells. They are control plane in
the AWS sense: they hold authoritative state and produce
configuration snapshots that data plane cells consume.

**Initial deployment: 2-3 Tier 2 cells per region.** This is the
fault-isolation floor. With 2 Tier 2 cells, a single Tier 2 failure
still leaves a functioning control plane; with 3, an active-
active-active configuration tolerates one failure and one
maintenance window simultaneously. Per ADR-0241, Tier 2 µservices
declare `dr_tier: T1` (< 5 min RTO, 0 RPO) for authoritative-state
substrates and `dr_tier: T2` (< 1h RTO, < 1 min RPO) for
configuration-publication substrates.

**Substrates hosted on Tier 2 cells:**

- `tenancy` — authoritative tenant table
- `identity` — authoritative OIDC + SPIRE
- `policy-engine` — authoritative Cedar fragment registry + bundle
  publisher
- `audit-chain` — authoritative Merkle-sealed audit log (cross-cell
  rollup; per-tenant per-sub-scope sub-streams)
- `cell` — authoritative cell registry + tenant→cell binding +
  shuffle-sharding service
- `governance` — authoritative fitness gate registry
- `compliance` — authoritative compliance pack registry +
  certification level registry (per ADR-0251)
- `observability` (control-plane half) — authoritative metric +
  log + trace configuration; per-tenant dashboard registry
- `cloud-iac` — authoritative IaC module registry
- `consent-graph` (control-plane half) — authoritative consent
  state + DSAR cascade authority
- `intelligence` (control-plane half: model registry, fine-tune
  registry; not inference path) — authoritative model catalog

**Per-cell properties:**

- **One K8s cluster per cell.** Tier 2 cells are full multi-AZ K8s
  clusters with high availability storage (Postgres + Citus +
  Elasticsearch + Valkey clusters all replicated within the cell).
- **Cilium ambient + Istio Ambient at L7** per ADR-0148. SPIFFE
  workload identity per ADR-0253 §D-7.
- **Active-active across Tier 2 cells within a region.** Tenancy
  writes use multi-leader Citus + per-row tenant-shard ownership;
  Cedar fragment publishes use last-writer-wins with cosign
  signatures resolving conflicts.
- **Cross-region: active-passive** with bounded staleness (<1 sec
  for tenant table reads; < 5s for policy-engine fragment reads).
- **Constant work for fragment publication.** Per Brooker 2020,
  Tier 2 publishes versioned snapshot bundles on a 30-second
  cadence; Tier 3 cells pull. No per-change deltas pushed.
- **Static stability target: 24 hours.** Tier 3 cells must
  function with their last-pulled snapshot for up to 24 hours of
  Tier 2 unavailability.

**Sizing:** Tier 2 cells are sized to handle the entire region's
control plane workload. Per Tier 2 cell capacity envelope is
~10,000-50,000 tenant records, ~50,000-200,000 Cedar fragment
evaluations per second cross-region (cached evaluations served from
Tier 3; the rate is the *fragment update* rate), and ~1-5 PB
authoritative audit-chain ledger. The 2-3 cell count per region
holds steady at this sizing through 1M tenants in the region; beyond
1M tenants per region, additional Tier 2 cells are provisioned
horizontally.

### D-4. Tier 3 — data plane cells

Tier 3 data plane cells host **tenant workloads**. Every customer
tenant's `home_cell` is a Tier 3 cell; every tenant's `dr_cell` (per
ADR-0241) is a different Tier 3 cell in a different AZ or region.

**Initial deployment: 5-10 Tier 3 cells per region.** This number
grows with tenant count + per-cell-capacity utilisation. At maturity
(2027+ targets), 50-200 Tier 3 cells per region; globally, 200-1000+
Tier 3 cells.

**Substrates hosted on Tier 3 cells (the complete substrate stack):**

Every Tier 3 cell hosts the complete data-plane substrate set
required to serve tenant workloads end-to-end without crossing into
Tier 2:

- `network` (data-plane half: Cilium agent + ztunnel + waypoint
  Envoy per-cell)
- `cloud-k8s` (data-plane half: per-cell K8s cluster + Karpenter
  + HPA + VPA)
- `cloud-secrets` (data-plane half: OpenBao replica + per-tenant
  KMS keys + per-cell HSM partition)
- `audit-chain` (data-plane half: local audit-chain shard + Merkle-
  sealed local ledger; rolls up to Tier 2 per ADR-0028)
- `observability` (data-plane half: Prometheus + Mimir + Loki +
  Tempo per-cell; rolls up to Tier 2 control plane via OTel
  Collector)
- `policy-engine` (data-plane half: cached Cedar fragment bundle +
  in-cell evaluator pods + Valkey hot cache per ADR-0243 §D-6)
- `identity` (data-plane half: SPIRE agent + cached OIDC discovery
  documents + per-cell session cache)
- `tenancy` (data-plane half: cached tenant rows + per-tenant sub-
  scope cache)
- `consent-graph` (data-plane half: cached consent state)
- `compliance` (data-plane half: cached compliance pack fragments)
- `intelligence` (data-plane half: inference workers; per-tenant
  model caches; routes long-running inferences via workflow-engine)
- `workflow-engine` (data-plane half: per-cell scheduler + execution
  workers)
- `ontology` (data-plane half: per-cell Citus shard for tenant
  data; per-cell object-type cache)
- Every product µservice (mail, drive, calendar, meet, etc.) per
  ADR-0245 §D-3.B

**Tenant→cell binding:** Each tenant has a `home_cell` (primary
Tier 3 cell) and `dr_cell` (DR-pair Tier 3 cell, different AZ or
region per ADR-0241). The shuffle-sharding service (§D-7) assigns
each tenant to `S = 8` cells; the home_cell + dr_cell are 2 of those
8; the other 6 are read replicas (for cross-region reads + DR pre-
warming).

**Per-cell properties:**

- **One K8s cluster per cell** per ADR-0148. Cilium ambient + Istio
  Ambient at L7. SPIFFE/SPIRE for workload identity.
- **24-hour static-stability tolerance.** Per Weiss/Furr 2020, every
  Tier 3 cell must continue serving its bound tenants' traffic for
  up to 24 hours of Tier 2 unavailability. Cached state required:
  tenant rows, Cedar fragments, OIDC discovery, SPIFFE root
  certificates (rotated hourly; valid for 25 hours), compliance pack
  fragments, observability collector config.
- **Self-contained inference + retrieval.** Tenant LLM tool-calls
  resolve inside the cell using per-cell model caches + per-tenant
  vector indices; cross-cell inference only for model-fine-tune
  refresh (control-plane responsibility on Tier 2).
- **Per-cell HSM partition.** Per ADR-0009 inheritance; per-tenant
  KMS root keys live in the cell's HSM.

**Cell capacity sizing per §D-10:** 100-300 tenants per Tier 3 cell
in initial deployment; auto-spawn new Tier 3 cells when an existing
cell reaches 70% capacity utilisation (per the shuffle-sharding
admission gate).

**Tier 3 deployment pattern variants** (declared in µservice manifest
per ADR-0244 §D-5 `cellular_deployment_pattern`):

- `standard` — one Deployment per cell; the default for products.
- `dedicated` — one Deployment per tenant per cell; used for
  enterprise tenants that have negotiated dedicated capacity.
- `shared` — one Deployment across multiple cells; rare; only
  substrate-tier µservices may declare this and only with
  multispectrum-review approval.
- `edge` — deployed at edge cells; not in mainline Tier 3 cells.

### D-5. Service cells — peer-tier dedicated functions

Beyond substrates (Tier 2) and tenant workloads (Tier 3), the
platform operates *service cells*: peer-tier cells with dedicated
functions, neither substrate nor product, that host the platform's
operational backbone. Per ADR-0245 §D-5, the canonical service cell
subtypes are:

- `marketplace-cell-<N>` (hosts `microservices/marketplace/` —
  marketplace ingestion + indexing + ranking + search pipelines).
  2-5 cells globally at initial deployment; expand with marketplace
  catalog size.
- `dev-tools-cell-<N>` (hosts `microservices/developer-sdk/`
  backend pipelines + `microservices/plugin-app-store/` backend
  pipelines + self-hosted git GitHub + self-hosted container
  Harbor per Year 3+ migration). 1-2 cells per major region.
- `audit-aggregator-cell-<N>` (hosts per-jurisdiction audit
  aggregator — rolls up audit-chain streams from all Tier 3 cells
  in the jurisdiction; exposes regulator query interface). 1 cell
  per regulatory jurisdiction (KR, EU, US, JP, KSA, UAE).
- `analytics-cell-<N>` (hosts `microservices/analytics/` backend
  pipelines + per-tenant aggregation + per-cohort aggregation).
  2-4 cells globally; one per major region.
- `ops-console-cell-<N>` (hosts `ops-dashboard-control-center`
  backend — incident graph, on-call rotation state, runbook
  execution state). 1-2 cells globally; oyatie-tenant-only access.

**Per-cell properties:**

- **One K8s cluster per cell.** Same Cilium + Istio Ambient + SPIRE
  baseline as Tier 2 and Tier 3 cells.
- **Peer-tier traffic.** Service cells consume substrate from Tier
  2 cells; they do not host tenant data. Their consumers are other
  peer service cells, Tier 2 control plane, and humans (operators
  or regulators).
- **Tier 2 + Tier 3 dual access.** Service cells access Tier 2
  control-plane state (read) and may emit to Tier 2 audit (write).
  They may access Tier 3 cells only via well-defined audit
  aggregator + analytics aggregator interfaces; they do not call
  Tier 3 cells' product surfaces directly.
- **Per ADR-0245 §D-4.D**, service cells may depend on substrates +
  other service cells; never on products.

### D-6. Per-cell vs cross-cell bright line

The cell architecture's core operational property is that **the hot
path is always intra-cell**. Cross-cell traffic is permitted only
for narrow coordination concerns and is bounded in latency, payload
size, and frequency.

| Concern | Per-cell (intra-cell) | Cross-cell (inter-cell) |
|---|---|---|
| **Tenant request serving** | YES — entire request resolves in tenant's home_cell | NO — cross-cell hot-path serving forbidden |
| **Cedar policy evaluation** | YES — every request hits cell-local policy-engine evaluator | NO — never call Tier 2 from hot path |
| **Audit-chain emit** | YES — local Merkle-sealed shard | Async rollup to Tier 2 (out-of-band) |
| **Database write** | YES — cell-local Citus shard write | Async cross-region replication (per ADR-0049) |
| **Database read (tenant data)** | YES — cell-local read | NO — cross-cell tenant data reads forbidden |
| **Database read (oyatie tenant control plane data)** | NO — read from Tier 2 cached snapshot | YES — Tier 3 pulls Tier 2 snapshots on 30s cadence |
| **Cache lookup (Valkey)** | YES — cell-local cache | NO |
| **Message bus emit** | YES — cell-local Kafka topic | Async rollup to Tier 2 control-plane bus |
| **Workflow execution** | YES — cell-local workflow-engine scheduler + workers | Async fan-out via workflow-engine's cross-cell durable orchestration only when workflow spans tenants |
| **Observability metric emit** | YES — cell-local Prometheus / Mimir | Async rollup to Tier 2 (OTel collector) |
| **OIDC token validation** | YES — cell-local cache of OIDC discovery + JWKS | NO — never call Tier 2 identity from hot path |
| **SPIFFE certificate** | YES — cell-local SPIRE agent | Hourly rotation pulls from Tier 2 SPIFFE federation root |
| **Inference (LLM, embedding)** | YES — cell-local inference workers + cell-local model cache | NO — cross-cell inference forbidden |
| **Vector index search** | YES — cell-local Milvus | NO — per-tenant vector data is per-cell |
| **Plugin Wasmtime execution** | YES — cell-local Wasmtime sandboxes | NO |
| **Tenant→cell binding lookup** | YES — cell-local cache (with 24-hour static-stability tolerance) | Hourly pull from Tier 2 cell registry |
| **Cross-tenant collaboration (workflow sharing)** | NO — cross-tenant sharing is async via cross-cell durable workflow | YES — bounded |
| **Marketplace catalog read** | YES — cell-local cache (5-minute TTL) | Hourly pull from marketplace-cell |
| **DSAR cascade enumeration** | YES — cell-local cascade per tenant | Cross-cell coordination via consent-graph control plane (Tier 2) |
| **Backup + restore** | YES — cell-local backup | Cross-region replication of backups via cloud-iac substrate |

**Hot-path rule.** A request from a tenant principal must complete
end-to-end inside the tenant's home_cell unless the request
explicitly targets cross-cell coordination (cross-tenant share,
cross-cell workflow). The hot-path rule is enforced by the
`oya-check-cross-cell-traffic-permits` CI lane against every gRPC
call site, every HTTP fetch site, every database connection
declaration in every µservice's source. Violations are BLOCKER
post-substrate.

**Async coordination exemption.** Out-of-band rollups (audit-chain
to Tier 2, observability to Tier 2, tenant→cell snapshot pull from
Tier 2) are explicit cross-cell coordination paths. They are
permitted but must be:

- Asynchronous (no request-thread blocking on cross-cell call).
- Idempotent (retry-safe).
- Bounded in rate (rate-limited at source).
- Logged in audit-chain (cross-cell coordination is a recorded
  decision).
- Cedar-gated (every cross-cell call evaluates a Cedar permit per
  ADR-0243).

### D-7. Shuffle sharding per MacCárthaigh 2014

Tenant→cell assignment uses shuffle sharding with the following
parameters:

- **Cell pool: all Tier 3 cells in the tenant's pack(s).** A
  tenant whose jurisdiction is KR with sovereign-cloud pack `kr`
  draws its shuffle shard from the pool of all Tier 3 cells in the
  `kr` pack. A tenant whose jurisdiction is EU with pack `eu` draws
  from EU pack cells. A tenant operating in both KR and EU (rare,
  but possible for partner agencies) draws separate shards from
  each pack and operates two parallel home_cell bindings.
- **Shard width: `S = 8` per tenant.** Each tenant is assigned to 8
  cells from the pool. The 8 cells host the tenant's data with
  varying authority levels:
  - 1 of the 8 is the `home_cell` (primary serving cell; writes go
    here first).
  - 1 of the 8 is the `dr_cell` (DR-pair; cross-AZ or cross-region
    per ADR-0241).
  - 6 of the 8 are `read_replica_cells` (async replicated; serve
    read-only traffic for cross-region reads + DR pre-warming).
- **Assignment function: consistent hash with shuffle.** The
  cell-substrate's assignment function:

  ```python
  def assign(tenant_id: str, pool: list[str], shard_width: int) -> list[str]:
      # Deterministic shuffle based on tenant_id seed; produces
      # a stable permutation of the pool.
      rng = SeededShuffle(seed=tenant_id)
      shuffled = rng.permute(pool)
      return shuffled[:shard_width]
  ```

  This is stable: given the same `tenant_id` and the same `pool`,
  it always returns the same shard. Adding cells to the pool
  triggers a planned migration (§D-12) for some fraction of tenants
  whose assignments shift.
- **Math.** For a 100-cell pool with `S = 8` shard width:
  - Probability of two tenants sharing all 8 cells:
    `1 / C(100, 8) = 1 / 186,087,894,300 ≈ 5.4 × 10^-12`.
  - Expected fraction of tenants whose home_cell is in any given
    cell: `1/100 = 1%`. With shuffle sharding, exactly 8% of
    tenants have *some* presence in any given cell.
  - On a single cell failure, expected impact: 8% of tenants
    experience degradation in 1 of their 8 cells (i.e., capacity
    reduced by `1/8 = 12.5%` for that subset of tenants). No tenant
    goes fully offline.
  - On two simultaneous cell failures: expected `8/100 × 7/99 ≈
    0.566%` of tenants have BOTH failed cells in their shard.
    Those tenants lose `2/8 = 25%` capacity.
  - On three simultaneous cell failures: `~0.035%` of tenants have
    all three failed cells in their shard. Computed as
    `C(8, 3) / C(100, 3) = 56 / 161,700 ≈ 0.0346%`. Loss bounded at
    `3/8 = 37.5%` for that subset.
- **Per-workload shard width override.** Specific workloads may
  override the default `S = 8`:
  - `S = 16` for high-tier enterprise tenants (Tier 4 enterprise);
    deeper fault isolation, higher cost.
  - `S = 4` for sandbox/preview tenants; tighter cost, less
    isolation.
  - `S = 32` for payments-grade tenants once Tier 4 lands.
- **Onboarding flow.** When a new tenant is registered (per
  ADR-0244 §D-7 lifecycle PROVISIONING state), the cell substrate
  computes the tenant's shuffle shard, writes `home_cell` +
  `dr_cell` + `read_replica_cells` to the tenant row, and pushes
  the binding to all 8 cells.

The shuffle-sharding algorithm lives in
`crates/oya-shuffle-sharding`. Its parameters are
declared in `/specs/shuffle-sharding-parameters.json` and version-
controlled. Changes to shard width or pool definition trigger a
shuffle-sharding migration ChangeSet per ADR-0110.

### D-8. Static stability per Weiss/Furr

Every cell must tolerate 24 hours of isolation from its dependencies
without dropping data-plane traffic:

- **Tier 3 cell isolated from Tier 2.** Tier 3 cells operate on
  cached state:
  - Cached tenant rows (snapshot pulled hourly from Tier 2).
  - Cached Cedar fragment bundles (snapshot pulled every 30s).
  - Cached OIDC discovery + JWKS (snapshot pulled hourly).
  - Cached SPIFFE root + trust bundle (rotated hourly; valid 25h).
  - Cached compliance pack fragments (snapshot pulled hourly).
  - Cached marketplace catalog (snapshot pulled hourly).
  - Cached observability collector config (snapshot pulled hourly).

  During Tier 2 isolation, Tier 3 cells:
  - Continue serving existing tenants' read + write traffic
    (writes accumulate in local Citus shard; cross-region
    replication queues await reconnection).
  - Cannot onboard new tenants (Tier 2 tenancy is authoritative).
  - Cannot evaluate new Cedar fragments (operates on last-pulled).
  - Cannot validate brand-new OIDC clients (operates on cached
    JWKS; existing tokens validate).
  - Emits SEV-3 alert at 1 hour of Tier 2 isolation; SEV-2 at 6
    hours; SEV-1 at 12 hours; SEV-0 at 24 hours.
  - Per ADR-0176 brown-out signal, transitions to `degraded` at
    1 hour; `outage` at 24 hours (Tier 2 likely catastrophically
    lost; manual intervention required).
- **Tier 2 cell isolated from Tier 0 dependencies.** Tier 2 cells
  operate on cached external state:
  - Cached DNS resolutions (TTL-bound; per RFC 1035 cache).
  - Cached external CA certificates (until expiry).
  - Cached time (drift-bounded; per ADR-0028 inheritance).
- **Tier 3 cell isolated from sibling Tier 3 cells.** Hot-path is
  intra-cell per §D-6; sibling cell isolation has bounded impact:
  - Cross-tenant collaboration features unavailable (workflow
    sharing across tenants).
  - DR-pair sync queued (DR failover available only with the
    queue's bounded staleness).
- **Bootstrap cell isolated from everything (steady state).** The
  bootstrap cell is normally retired (§D-2). Re-provisioning the
  bootstrap cell is a manual operator procedure with a runbook.

**Static-stability CI lane.** `oya-check-static-stability-coverage`
verifies that every µservice deployed in Tier 3 declares its
required cached-state set + its TTL + its fallback behavior on
cached-state miss. Coverage gap → BLOCKER post-substrate.

### D-9. Constant work pattern per Brooker 2020

Control plane work must be proportional to *configuration*, not to
*change rate*. The platform implements this for every Tier 2 → Tier
3 distribution path:

- **Cedar fragment bundle.** Tier 2 policy-engine publishes a
  versioned snapshot of all Cedar fragments every 30 seconds
  (configurable). Tier 3 cells pull the snapshot, hot-reload the
  evaluator, and continue. This is the **Path B constant-work pull**
  path; worst-case propagation is ≤35s p99 (30s pull cadence + up to
  5s recompile) [P5..P95: 10s–35s] (evidence: modeling note
  docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md).
  The separate **Path A push** (Kafka pub-sub, EMERGENCY-priority
  activations only) achieves 5s p99 per ADR-0243 §D-10; it is NOT a
  per-change push path — it is reserved for emergency permits only,
  preserving the constant-work invariant for standard updates.
- **Tenant row snapshot.** Tier 2 tenancy publishes a per-cell
  snapshot of `(tenant_id, sub_scope, home_cell, dr_cell, …)` rows
  on hourly cadence. Tier 3 cells pull only the tenant rows whose
  `home_cell` or `read_replica` includes the cell.
- **OIDC discovery + JWKS snapshot.** Hourly publication; cached
  in cell.
- **Compliance pack fragment snapshot.** Hourly publication.
- **Marketplace catalog snapshot.** Hourly publication.
- **SPIFFE trust bundle.** Per-cell SPIRE agent pulls hourly.

The principle: the size of each snapshot is `O(N)` where `N` is
the number of items, not `O(M)` where `M` is the change rate.
Tier 2's publication work is bounded by `N`, regardless of how many
changes occurred between snapshots. Tier 3's pull work is bounded
by `N`, regardless of how many cells exist.

**Anti-pattern explicitly rejected.** Push-based per-change deltas
(Tier 2 pushes every Cedar fragment change to every Tier 3
evaluator over a pub-sub bus). This was Brooker's specific cautionary
example: the work scales as `O(change_rate × fleet_size)`, which
grows unboundedly. The platform does not use this pattern except
where unavoidable (e.g., audit-chain emit is per-event; but the
audit-chain is a write path, not a control-plane distribution
path, and the audit-chain target scales horizontally per shard).

**Constant-work CI lane.** `oya-check-constant-work-control-plane`
verifies that no Tier 2 → Tier 3 distribution path uses per-change
push semantics. Static analysis identifies pub-sub topics
originating in Tier 2 with high cardinality consumers in Tier 3 and
flags them for review.

### D-10. Cell sizing — 100-300 tenants per Tier 3 cell; auto-spawn at 70%

**Initial sizing target: 100-300 tenants per Tier 3 cell.** The
range reflects per-tenant traffic variance:

- High-traffic tenants (B2B enterprise; ~1k QPS per tenant): 100
  tenants per cell.
- Standard tenants (~10-100 QPS per tenant): 300 tenants per
  cell.
- Low-traffic tenants (sandbox, preview, free tier; <1 QPS): up
  to 1000 per cell (constrained by storage and Cedar evaluation
  count, not request rate).

The exact sizing is determined by the cell capacity envelope:

- Compute: 16-64 nodes per cell; each node 32-128 vCPU; total cell
  compute ~512-8192 vCPU.
- Storage: per-cell Citus shard 10-100 TB; per-tenant storage
  bounded by tenant tier resource_budget.
- Network: per-cell ingress 10-100 Gbps; per-cell egress 5-50 Gbps.
- Memory: 1-8 TB total cell memory.
- HSM: 1 HSM partition per cell (per ADR-0009 inheritance);
  partition supports 10,000-100,000 KEKs per the underlying HSM
  vendor (Thales Luna, AWS CloudHSM, Naver Cloud HSM, etc.).

**Auto-spawn at 70% utilisation.** The cell substrate's capacity
monitor watches `(cpu_utilization, memory_utilization,
storage_utilization, network_utilization, hsm_partition_utilization,
tenant_count, qps)` per cell. When any of these exceeds 70% over a
sustained window (30 min p99), the substrate triggers:

1. Provision a new Tier 3 cell via cloud-iac substrate.
2. The new cell joins the pool.
3. The shuffle-sharding service may migrate some existing tenants
   to the new cell (planned migration per §D-12) to balance load.
4. New tenant onboarding directs to a shard that includes the new
   cell.

The auto-spawn workflow is owned by `ops-dr-capacity` and
`axis-cell`; the runbook is at
`docs/runbooks/cell-tier-3-auto-spawn.md`.

**Cell decommissioning.** Cells whose utilisation falls below 20%
sustained for 7 days are candidates for decommissioning; tenants are
migrated out via planned migration, and the cell's K8s cluster is
deprovisioned. Decommissioning is council-approved (rare; usually
cells expand monotonically with tenant base growth).

### D-11. Cross-region routing — GeoDNS + failover

Per ADR-0253 §D-13, the platform uses GeoDNS for cross-region
client routing:

1. Client DNS query (`tenant-acme.example.oyatie`) hits anycast
   DNS (Cloudflare DNS initially; PowerDNS-self-hosted Year 3+).
2. GeoDNS returns the IP of the Cloudflare POP (or self-hosted
   Pingora POP Year 3+) closest to the client.
3. The edge POP terminates TLS, decodes the tenant ID from the
   Host header or JWT claim, looks up the tenant's `home_cell`
   in the edge-cached tenant→cell binding (5-minute TTL pulled
   from Tier 2 cell substrate), and forwards the request via
   the cell's region's mesh ingress.
4. If the `home_cell` health-check fails (cell unhealthy per the
   cell substrate's health signal), the edge forwards to the
   tenant's `dr_cell` instead.
5. If `dr_cell` is also unhealthy, the edge returns 503 with a
   brown-out signal per ADR-0176 + per ADR-0253 §D-5 retry-after.

**Per-tenant home_cell stability.** A tenant's `home_cell` is sticky
for the tenant's lifetime; migration is explicit per §D-12. This
prevents the "thrashing" failure mode where shuffle sharding
re-balancing constantly migrates tenants.

**Cross-region failover.** When a region-wide failure occurs (all
Tier 3 cells in region X unavailable), the edge POPs failover all
tenants whose `home_cell` is in region X to their `dr_cell` (in
another region). Per ADR-0241 T1/T2 replication shapes, the failover
is < 5 min RTO with bounded data loss per the tier.

### D-12. Tenant→cell assignment + planned migration workflow

**Onboarding assignment** (per §D-7 shuffle sharding):

1. Tenant registration enters PROVISIONING state per ADR-0244
   §D-7.
2. The cell substrate computes shuffle shard:
   `shuffle(tenant_id, pack_cells(jurisdiction), shard_width=8)`.
3. The first cell in the shuffle becomes `home_cell`; the second
   becomes `dr_cell` (with cross-AZ or cross-region constraint
   per ADR-0241); the remaining 6 are `read_replica_cells`.
4. The cell substrate writes the binding to the tenant row.
5. Each of the 8 cells receives a tenant-binding push (via Tier 2
   → Tier 3 snapshot or immediate Cedar fragment publication if
   urgent).
6. Tenant transitions to ACTIVE state when all 8 cells acknowledge
   the binding.

**Planned migration workflow.** A tenant's binding may need to
change due to:

- Cell decommissioning (cells removed from pool).
- Capacity rebalancing (cell at 90% util; migrate cold tenants out).
- Compliance pack activation (tenant activates HIPAA; must migrate
  to HIPAA-certified cell per ADR-0251).
- Jurisdiction change (tenant moves from US to EU; migrate to EU
  pack cells).
- Tenant tier upgrade (enterprise tenant requested dedicated cells
  per `cellular_deployment_pattern: dedicated`).

The planned migration workflow:

1. ChangeSet authored per ADR-0110: source binding, target binding,
   rationale, evidence URL.
2. Multispectrum review (F1 correctness, F5 security, F6
   performance, A4 architecture-adherence).
3. Pre-migration sync: tenant data replicated from source cells to
   target cells; replication catches up to within bounded staleness.
4. Cutover: tenant's `home_cell` binding updated; edge POPs pull
   the new binding within 5 minutes (per Tier 2 cell-registry
   snapshot cadence).
5. Source cells' tenant data retained for 7 days (rollback window);
   then garbage-collected with audit-chain emit
   `TenantMigrationGarbageCollect`.
6. Audit-chain emits `TenantCellMigrationComplete` with full event
   trail.

**Migration SLO.** Tier 1 (T1 per ADR-0241) tenant migration
completes within 1 hour RPO + 5 min RTO. Tier 2 within 4 hours
RPO + 1 h RTO.

### D-13. K8s-everything — every workload in Pods

Every workload runs in Kubernetes Pods. No bare-metal services. No
unmanaged VMs. No serverless-without-K8s. This is enforced at the
admission layer.

**Stack:**

- **Kubernetes 1.30+ LTS.** Per ADR-0121 on-prem K8s stack baseline.
- **Cilium ambient service mesh.** Per ADR-0148 layered with Istio
  Ambient. Cilium owns CNI / L3 / L4 / kernel-level observability.
  Istio Ambient owns SPIFFE mTLS + L7 policy via waypoint.
- **Cluster API (CAPI).** Per ADR-0121 inheritance, cell K8s
  clusters are managed via Cluster API providers (CAPI-AWS,
  CAPI-GCP, CAPI-Azure, CAPI-Naver, CAPI-OVH, etc.). Tier 2 cell
  substrate orchestrates CAPI to provision Tier 3 cells.
- **Karpenter.** Per ADR-0028 inheritance, Karpenter handles node
  auto-scaling within each cell. Karpenter consumes per-tenant
  resource budget from tenancy substrate and provisions nodes
  accordingly.
- **SPIFFE workload identity (SPIRE).** Per ADR-0148 + ADR-0253
  §D-7. Every Pod has a SPIFFE SVID (X.509 cert) rotated hourly.
  Workload identity is the basis for cross-cell mTLS, audit
  attribution, and Cedar principal claims.
- **Per ADR-0148:** Cilium in CNI-and-L4-only mode + Istio Ambient
  ztunnel at L4 mTLS + Istio Ambient waypoint Envoy at L7
  (opt-in per µservice). The waypoint hosts the `ext_authz` filter
  that calls the cell-local Cedar PDP per ADR-0243.
- **Kyverno** for admission control per ADR-0183. Kyverno enforces
  K8s-resource-level invariants: every namespace has a tenant
  label; every Pod has a SPIFFE-issued ServiceAccount; no privileged
  containers without a tier-specific exception.

**K8s-everything exceptions.** Two narrow exceptions, both
documented:

- **Edge POPs.** Per D-15, edge POPs (Cloudflare Workers initially;
  self-hosted Pingora Year 3+) are not K8s. Edge runs at finer
  POP density (~300 POPs) than K8s manages.
- **Hardware appliances.** HSM partitions, network switches, BGP
  routers are not K8s. They are managed via vendor-specific
  controllers + Cluster API extensions where available.

### D-14. Sandboxing in cells — Cloud Hypervisor + Kata Containers

Per ADR-0211 in-house Rust-primary tech-stack preference, the
canonical sandboxing pair for Tier 3 cells is:

- **Cloud Hypervisor (Rust-based, KVM-backed, Apache 2.0 license).**
  The canonical VMM for tenant context isolation, plugin sandboxing,
  and confidential computing. Cloud Hypervisor is an in-house-aligned
  choice: it's written in Rust by Intel + Microsoft + Arm + ByteDance
  + AWS + Cloudflare contributors; it's smaller and simpler than
  Firecracker (designed for a wider workload range); it's KVM-
  backed so it inherits Linux KVM's security model.
- **Kata Containers 3.x runtime.** Wraps Cloud Hypervisor as a
  Kubernetes container runtime via the Kubernetes CRI. Kata
  Containers gives every Pod a lightweight VM (~250ms cold-start;
  ~150MB memory overhead per VM vs. ~10MB per container). The
  Kata + Cloud Hypervisor pair is the canonical isolation primitive.

**Where Cloud Hypervisor + Kata is required:**

- **Plugin Wasmtime sandboxes.** Each tenant plugin runs in a
  Cloud Hypervisor VM via Kata; the plugin's Wasmtime runtime
  executes inside the VM. Two-layer isolation: VM boundary +
  Wasmtime capability sandbox.
- **Tenant context isolation.** High-tier tenants (enterprise tier
  + sovereign-pack tenants per ADR-0240) may declare per-tenant
  Pod isolation via `cellular_deployment_pattern: dedicated`. Each
  dedicated Pod runs in its own VM.
- **Confidential computing workloads.** Tenants that require AMD
  SEV-SNP (Secure Encrypted Virtualization — Secure Nested Paging)
  or Intel TDX (Trust Domain Extensions) attestation get Cloud
  Hypervisor with SEV-SNP / TDX configuration. The cell's
  underlying hardware must support the requested confidential
  computing technology; cells are tagged per supported tech (per
  ADR-0251 cell certification levels).
- **EU AI Act high-risk inference.** Per ADR-0144, AI inferences
  classified as high-risk run in Cloud Hypervisor VMs with
  attestation (TDX or SEV-SNP).

**Anti-pattern explicitly rejected: gVisor.** gVisor is Google's
user-space kernel sandbox for containers; it works by intercepting
syscalls and serving them from a user-space "Sentry" process. It is
written in Go (incompatible with the ADR-0211 Rust-primary
preference); it adds non-trivial syscall latency on hot paths; its
security model is harder to reason about than KVM (Sentry bugs
become container-escape vectors); it lacks confidential computing
support. The platform does not adopt gVisor.

**Anti-pattern explicitly rejected: containerd-only (no VM).**
containerd + runc is the K8s default container runtime; it provides
namespace + cgroup isolation but not VM-level isolation. For most
trusted in-cell workloads (substrate µservices, product µservices
serving tenants under Cedar policy), containerd-only is acceptable.
For plugin sandboxing + tenant context isolation + confidential
computing, containerd-only is insufficient.

The cell-level admission policy (Kyverno per ADR-0183) tags Pods
that must use Kata-cloud-hypervisor as their runtime: any Pod with
label `requires-vm-isolation: true` lands on Kata; others land on
containerd.

### D-15. Edge layer exception — Cloudflare → Pingora (per ADR-0253 §D-2 + §D-17)

The edge layer is the **only** part of the platform that is not
K8s-everything. Per ADR-0253:

**Year 0-3 (current to 2029): Cloudflare Workers at ~300 POPs.**

- Cloudflare Workers run on V8 isolates (~5ms cold-start; sub-
  millisecond invocation overhead).
- Cloudflare's edge network spans ~300 POPs globally (~330+ as of
  2025).
- Edge functions: TLS termination, GeoDNS routing, edge-cached
  static content, request decoding, WAF, DDoS protection, JWT
  validation against cached JWKS, edge-cached tenant→cell binding
  lookup, request forwarding to cell mesh ingress.
- HTTP/3 (QUIC, RFC 9114) is the default protocol per ADR-0253
  §D-5. Cloudflare supports HTTP/3 natively.
- TLS 1.3 with post-quantum hybrid KEX (ML-KEM-768 + X25519) per
  ADR-0253 §D-4, available in Cloudflare as of mid-2024.

**Year 3+ (post-2029): self-hosted Pingora at the same ~300 POPs.**

Per ADR-0253 §D-17 migration path, the platform self-hosts Pingora
at the platform's own POPs once the operational maturity supports
it. Pingora is Cloudflare's Rust-based HTTP/3-native proxy (open-
sourced 2022; production-proven at Cloudflare's edge). Self-hosted
Pingora:

- Removes Cloudflare-as-vendor dependency (per ADR-0211 in-house
  preference + per ADR-0240 sovereign-cloud-overlay for sovereign
  packs where Cloudflare may not be sanctioned).
- Retains the same HTTP/3, TLS 1.3, PQ hybrid KEX, and WAF
  capabilities.
- Operates at the platform's own POPs (initially the major
  regions; expand to ~100 POPs by Year 5, ~300 by Year 7).

**Why edge is NOT K8s.**

- Per ADR-0253 §"Why edge is NOT in K8s": edge POPs operate at
  finer geographic density than K8s clusters manage; an edge POP
  per major city is impractical as a K8s cluster (K8s per-cluster
  overhead is too high; ~300 POPs would be ~300 clusters).
- Cloudflare's V8 isolate model + Pingora's per-process model
  both achieve sub-millisecond startup, which K8s + container
  startup cannot match.
- Edge POPs serve massive request rates (~tens of thousands of
  QPS per POP) on minimal compute; K8s overhead would inflate
  cost dramatically.

**HTTP/3 / QUIC as default protocol** (per ADR-0253 §D-5). All
client-edge connections use HTTP/3 (UDP-based QUIC) where the
client supports it. HTTP/2 + HTTP/1.1 are fallbacks. TLS 1.3 is
mandatory; TLS 1.2 is rejected (per ADR-0253). Post-quantum
hybrid KEX (ML-KEM-768 + X25519) deploys Year 2 (2027) per
ADR-0253 §D-4 deployment schedule.

### D-16. Tier 4 (RESERVED) — financial-grade + fulfillment-grade

Tier 4 is reserved for future post-certification workload classes:

- **Financial-grade cells** (post-payments-cert; per ADR-0245
  §D-3.D `reserved-financial-grade` µservices). Host:
  - `microservices/payments/` (reserved per ADR-0245 §D-3.D)
  - `microservices/tax-engine/` (reserved per ADR-0245 §D-3.D)
  - `microservices/identity-verification/` (reserved per ADR-0245)
  Certification gates:
  - PCI DSS v4.0 Service Provider Level 1 (>6M transactions/year)
  - KR-FSS (Financial Services Commission) Electronic Payment
    Service Provider designation
  - EU PSD2 SCA (Strong Customer Authentication) compliance
  - AML/KYC pipeline per FinCEN + KR-FSC + EU EBA
  Properties:
  - Per-cell HSM partition FIPS 140-3 Level 3 certified
  - Synchronous cross-AZ replication for payment state (Stripe
    Cells 2024 pattern)
  - Confidential computing default (Cloud Hypervisor + AMD SEV-SNP
    or Intel TDX)
  - Shard width `S = 32` per tenant (deeper fault isolation)
  - Quarterly external compliance audit
  - Card scheme direct links (Visa, Mastercard, Amex) per cell;
    one cell per major payment network region
- **Fulfillment-grade cells** (post-marketplace-physical-cert).
  Host:
  - Physical-marketplace fulfillment µservices (when authored;
    not yet in portfolio; planned post-2028)
  Certification gates:
  - Carrier API certifications (FedEx, UPS, DHL, Korean Post,
    Japan Post, etc.)
  - Per-jurisdiction fulfillment-substrate registrations
  Properties:
  - Per-region presence (close to fulfillment centers)
  - Direct integration with carrier WMS APIs
  - Tier 1 DR per ADR-0241 (transactional commitments to carriers)
- **IL5+ workloads** (post-FedRAMP-High / DoD IL5 certification).
  Host:
  - US-Government tenants per ADR-0240 sovereign-cloud-overlay
  - Defense-grade workloads requiring DISA SRG IL5+
  Properties:
  - Cloud Hypervisor + AMD SEV-SNP confidential computing default
  - FIPS 140-3 Level 4 HSM where available
  - Air-gapped from non-IL5 cells (zero cross-cell traffic to
    non-IL5)
  - Per-pack regulatory evidence cadence (quarterly per
    ADR-0240)

**Tier 4 status: RESERVED.** No Tier 4 cells exist on 2026-05-20.
The architectural slot is reserved per ADR-0250 build-ahead-of-
certification doctrine. The reserved µservices in ADR-0245 §D-3.D
have `tier: reserved`; promotion ADRs (one per reserved µservice +
one per Tier 4 cell deployment) will move them out of reserved when
certifications are obtained.

**Helm chart skeletons for Tier 4 cells** are authored at:

- `microservices/cloud-iac/iac/helm/cell-tier-4-financial-grade/`
  (planned; build-ahead-of-cert)
- `microservices/cloud-iac/iac/helm/cell-tier-4-fulfillment-grade/`
  (planned)
- `microservices/cloud-iac/iac/helm/cell-tier-4-il5/` (planned)

The skeletons are NOT deployed; they exist as forward-declared
artifacts per ADR-0212 buildability doctrine.

## Alternatives considered

### Alt-1. Single-region monolith (status quo at portfolio inception)

Operate the entire platform in a single region (e.g., US-East-1)
with all µservices co-deployed. Tenants share a single K8s cluster
+ a single Citus cluster + a single Cedar evaluator pool.

**Pros:**

- Simplest topology to operate.
- Lowest fixed cost (no cross-region replication; no cross-cell
  traffic; no shuffle sharding implementation).
- Familiar from early-stage SaaS architectures.

**Cons:**

- **No sovereign-cloud overlay possible.** ADR-0240 + ADR-0010
  require per-pack provider isolation, impossible in single-region
  monolith.
- **No DR portfolio possible.** ADR-0241 T1/T2/T3/T4 tiers all
  require multi-region or at minimum multi-AZ; single-region
  monolith caps DR at single-AZ → single-region.
- **No blast-radius isolation.** A single noisy tenant degrades
  every tenant (ADR-0009 LEDG-010 anti-pattern; explicitly the
  problem ADR-0009 + this ADR exist to solve).
- **No `oyatie`-tenant separation.** ADR-0242 oyatie-is-a-tenant
  doctrine requires that `oyatie` tenant operates as one tenant
  among many under uniform isolation; impossible in monolith.
- **No certification-level segregation.** ADR-0251 compliance pack
  cell-certification levels require some cells to be PCI-certified,
  some HIPAA, some sovereign; impossible in monolith.
- **Quality bar fails.** Per
  `feedback_quality_performance_scalability_bar`, hyperscaler-
  grade scaling is required; monolith caps at maybe 100k tenants
  before saturation; goal is 1M+.

**Rejected** because every named precedent (AWS, Stripe, Google,
Microsoft, Apple, Salesforce, Cloudflare) operates cellular and the
portfolio's DR + sovereign-cloud + compliance-pack ADRs all require
the cellular topology.

### Alt-2. Multi-region but without cellular (region-level isolation only)

Adopt multi-region deployment per ADR-0010 regional-pack, but inside
each region operate a single non-cellular topology (one K8s cluster
per region, all tenants in that cluster).

**Pros:**

- Smaller than monolith; gives DR + sovereign-cloud capability.
- Lower complexity than full cellular.
- Familiar from multi-region SaaS architectures.

**Cons:**

- **Within-region blast radius is the whole region.** A single
  noisy tenant degrades every tenant in the region. ADR-0009's
  LEDG-010 anti-pattern reasserts at regional level.
- **No shuffle sharding fault isolation.** Single cell-per-region
  means tenant has no shard width; single cell failure is full
  outage for the region.
- **Cell certification level becomes "the whole region."** ADR-
  0251 compliance pack certifications can't apply to sub-region
  cells; either the whole region is PCI or none of it is.
- **Constant work pattern impossible.** Without cellular Tier 2 →
  Tier 3 distribution, control-plane scaling is regional-fleet-
  proportional rather than constant.
- **Static stability impossible.** Without Tier 2 / Tier 3
  separation, control-plane outage is also data-plane outage.

**Rejected** because the within-region blast radius is unbounded and
the platform's quality bar requires bounded blast radius.

### Alt-3. Cellular at Tier 3 only (no Tier 1 / Tier 2 explicit)

Adopt cellular architecture but only at the data plane (Tier 3
cells); leave control plane implicit (no explicit Tier 2 separation).

**Pros:**

- Simpler than full four-tier (no bootstrap cell self-retirement
  procedure; no Tier 2 fault-isolation).
- Tier 3 cellular addresses the blast-radius problem.
- Familiar from early cellular adoption (some companies start here
  and add Tier 2 later).

**Cons:**

- **Control-plane saturation cascades to data plane.** Without
  explicit Tier 2, a Cedar fragment publication storm directly
  impacts request serving in Tier 3.
- **Constant work pattern undermined.** Without Tier 2 publishing
  versioned snapshots, Tier 3 either polls each other (high
  cardinality) or relies on per-change deltas (Brooker 2020
  anti-pattern).
- **Static stability undermined.** Without Tier 2 / Tier 3
  separation, Tier 3 cells cannot operate during control-plane
  outage; the control plane is in the data plane.
- **Bootstrap is ad-hoc.** No explicit Tier 1 means catastrophic
  loss recovery is manual; per ADR-0242 §D-5 bootstrap sequence
  cannot apply.
- **AWS 2018 ARC408 + 2024 ARC405 explicitly recommend Tier 2 +
  Tier 3 separation.** This is the AWS-canonical pattern, not a
  later addition.

**Rejected** because explicit Tier 2 / Tier 3 separation is the
canonical pattern at every named hyperscaler and per the AWS public
guidance.

### Alt-4. Service-cell-of-everything (every µservice in its own peer cell)

Adopt cellular but make every µservice a service cell (peer-tier);
no Tier 3 (no shared-substrate cell). Every µservice has its own
cluster + its own scaling.

**Pros:**

- Maximum fault isolation between µservices.
- Each µservice scales independently.
- Familiar from "one service per cluster" school of thought.

**Cons:**

- **Coordination cost explosion.** Cross-µservice calls become
  cross-cell; latency + reliability degraded.
- **Per-cell fixed overhead × N µservices.** Each cell has
  baseline cost (K8s cluster, Cilium, Istio Ambient, observability,
  HSM, audit-chain shard); 54 µservices × 5-10 cells per pack ×
  6 packs = 1620-3240 K8s clusters. Operational overhead
  intractable.
- **Tenant isolation lost.** Without Tier 3 (tenant-bound cells),
  tenant→cell binding becomes per-µservice; no unified isolation.
- **Stripe Cells 2024 explicit anti-pattern.** Stripe specifically
  rejects per-service cells in favour of per-tenant-grouping cells.
- **Workflow-engine pattern fails.** Per ADR-0145 inheritance,
  workflow-engine orchestrates cross-µservice durable workflows
  within a cell; cross-cell durable orchestration is more
  expensive and less reliable.

**Rejected** because the operational overhead is intractable and
the precedent at scale is per-tenant-grouping cells.

### Alt-5. Four-tier model + service cells (CHOSEN)

Adopt the AWS-canonical four-tier model (Tier 0 external + Tier 1
bootstrap + Tier 2 control plane + Tier 3 data plane) plus peer-
tier service cells (marketplace, dev-tools, audit-aggregator,
analytics, ops-console) plus Tier 4 RESERVED for post-certification
financial-grade + fulfillment-grade + IL5+. Shuffle sharding with
`S = 8` default. Static stability 24-hour tolerance. Constant work
control-plane distribution. Cloud Hypervisor + Kata Containers for
sandboxing. Cloudflare → Pingora at edge per ADR-0253. HTTP/3
default. K8s-everything except edge.

**Pros:**

- **Matches every named hyperscaler precedent.** AWS (2018
  ARC408 + 2024 ARC405); Stripe Cells 2024; Google Borg cellular;
  Microsoft Azure scale units; Salesforce Trust + Pod model;
  Cloudflare edge POPs + control plane separation; Apple iCloud
  cellular shape (per Apple WWDC 2023 sessions on iCloud
  reliability).
- **Closes ADR-0009 LEDG-010** (single-cluster posture) at the
  architectural level + extends ADR-0009 with formal four-tier +
  shuffle sharding + static stability + constant work.
- **Supports ADR-0240 sovereign-cloud-overlay.** Cells live within
  pack-bound providers; cell-substrate respects pack overlays.
- **Supports ADR-0241 DR portfolio.** Per-µservice DR tier
  (T1/T2/T3/T4) declares per-cell replication shape; cells
  implement.
- **Supports ADR-0242 oyatie-is-a-tenant.** `oyatie` tenant
  shuffle-shards across Tier 3 cells the same as customer tenants.
- **Supports ADR-0243 Cedar as universal gate.** Cell-local Cedar
  evaluators provide sub-millisecond hot-path evaluation.
- **Supports ADR-0251 compliance pack cell-certification levels.**
  Cells declare certification level; tenants bind to certified-
  matching cells.
- **Supports ADR-0247 self-modification.** Tier 1 self-retirement
  + Tier 2 versioned snapshots enable the platform to modify
  itself under deterministic policy gates.

**Cons:**

- **Bounded one-time implementation cost.** Substantial: cell
  substrate, shuffle-sharding service, planned-migration workflow,
  per-tier Helm charts, observability dashboards, runbooks. Tracked
  in implementation surface §below.
- **Operational overhead non-trivial.** Multi-tier model requires
  multi-tier observability + multi-tier on-call rotations +
  multi-tier capacity planning. Mitigation: well-documented
  runbooks (per ADR-0241 + ADR-0028 inheritance); per-cell
  cellular-topology dashboard.
- **Cross-cell traffic discipline required.** Hot-path-intra-cell
  rule must be enforced by CI lane; violations during refactor.
  Mitigation: `oya-check-cross-cell-traffic-permits` lane (D-6).
- **Shuffle-sharding math requires education.** Engineers
  unfamiliar with combinatorial reasoning may misconfigure shard
  width. Mitigation: `/specs/shuffle-sharding-parameters.json` is
  centrally version-controlled; shard width override requires
  multispectrum-review approval.

**Accepted** as the foundational keystone topology for the platform.
The cons are bounded one-time and one-time-per-µservice; the pros
include canonical hyperscaler alignment + support for every other
keystone in this bundle.

## Consequences

### Positive

1. **Blast-radius bounded by shuffle-sharding math.** No tenant
   goes fully offline from any single-cell failure; ≤1% of tenants
   experience >25% capacity degradation from any 3-cell-failure
   correlated event (per §D-7 math).
2. **Hot-path SLO achievable.** Cell-local Cedar + Cell-local
   tenant cache + Cell-local Citus shard + Cell-local inference
   workers enable < 200ms p99 end-to-end on tenant request hot
   paths.
3. **Sovereign-cloud-overlay clean.** Cells live within pack-bound
   providers (per ADR-0240). Pack-KR cells live on Naver Cloud +
   KT Cloud; Pack-EU cells live on OVH + AWS-EU; cross-provider
   traffic crosses the WireGuard tunnel only with Cedar permits.
4. **DR portfolio achievable.** Per-µservice T1/T2 cells use
   active-active multi-AZ + active-passive cross-region replication
   shapes per ADR-0241 §D-4.
5. **Compliance pack cell-certification levels enabled.** Cells
   declare certification level (PCI, HIPAA, SOC2, ISO22301, FedRAMP-
   High, CSAP, GAIA-X, etc.); tenants bind to certified-matching
   cells per ADR-0251.
6. **Self-modification supported.** Per ADR-0247, Foundry workflows
   modify the platform via Tier 2 control-plane operations + Tier
   3 wave deployments + audit-chain evidence emission.
7. **`oyatie`-tenant uniform with customer tenants.** Per ADR-0242,
   `oyatie` shuffle-shards across Tier 3 cells; no carve-out paths.
8. **Hyperscaler-shape achieved.** Matches AWS, Stripe, Google,
   Microsoft, Salesforce, Cloudflare, Apple cellular patterns.
9. **Static stability validated.** 24-hour Tier 2 isolation drill
   becomes a quarterly exercise per ADR-0241 + the SLO bar.
10. **Wave deployment for blast-radius-bounded rollout.** Per AWS
    2024 ARC405, deployments wave through cells (1 cell at a time
    initially → 5% of cells → 25% → 50% → 100%) bounding the blast
    radius of bad code.

### Negative

1. **One-time implementation cost.** Substantial: cell substrate,
   shuffle-sharding service, planned-migration workflow, per-tier
   Helm charts, observability dashboards, runbooks. ~6-9 months of
   engineering effort spread across ops-sre-reliability,
   axis-cell, axis-cloud-iac, axis-network.
2. **Operational overhead.** Multi-tier model requires multi-tier
   observability + multi-tier on-call + multi-tier capacity. Each
   tier has its own SLO bar (per ADR-0245 §D-8 substrate-tier
   floor 99.99%; service-cell-tier floor 99.95%).
3. **HSM partition count grows.** Per-cell HSM partition × N cells
   = many HSM partitions. Cost: HSM partition is typically $1-10k
   USD per partition per month (depending on provider). At 100
   cells × $5k/month = $500k/month. Mitigation: HSM partition
   amortises per-cell-tenant-count; per-tenant HSM cost is
   bounded.
4. **Cross-provider tunnel cost.** Inter-cell traffic across
   providers (e.g., AWS cell → Naver Cloud cell) traverses
   WireGuard tunnel; egress costs apply. Per ADR-0240
   prohibited_egress denies sovereign-data crossing; remaining
   cross-provider traffic is bounded.
5. **Migration complexity.** Tenant migration (D-12) requires
   coordination of source + target cells, replication catch-up,
   cutover, and rollback window. Mitigation: planned migration
   workflow + multispectrum review + audit-chain emission.

### Operational

1. **New CI lanes** (advisory until bootstrap; BLOCKER post-
   substrate per `enforcement_status`):
   - `oya-check-cell-tier-coherence` — every µservice manifest
     declares `cellular_deployment_pattern` per ADR-0244 §D-5.
   - `oya-check-shuffle-sharding-parameters` — shard width per
     workload matches /specs/.
   - `oya-check-cell-isolation-tolerance` — every Tier 3 µservice
     declares 24-hour static-stability cached-state set.
   - `oya-check-cross-cell-traffic-permits` — no cross-cell hot-
     path gRPC / HTTP call site.
   - `oya-check-cell-deployment-pattern` — manifest pattern matches
     actual K8s topology.
   - `oya-check-static-stability-coverage` — cached-state TTLs +
     fallback behaviors declared.
   - `oya-check-constant-work-control-plane` — no per-change push
     deltas from Tier 2 → Tier 3.
2. **Cell pattern successor surfaces:**
   - `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`
     hosts cell provisioning, lifecycle, registry, and planned capacity.
   - `microservices/tenancy/ARCHITECTURE.md#cell-assignment` hosts
     tenant→cell binding and migration state.
   - `microservices/observability/ARCHITECTURE.md#cell-health` hosts
     per-cell health, SLO burn, and blast-radius isolation alerts.
   - `microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing`
     hosts cell-aware tenant routing.
   - `microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit`
     hosts per-cell audit scoping.
   - `crates/oya-shuffle-sharding` hosts the pure deterministic
     shuffle-sharding algorithm.
   - `microservices/cloud-iac/iac/helm/cell-tier-1/` (NEW; Helm
     chart for bootstrap cell).
   - `microservices/cloud-iac/iac/helm/cell-tier-2/` (NEW).
   - `microservices/cloud-iac/iac/helm/cell-tier-3/` (NEW).
   - `microservices/cloud-iac/iac/helm/service-cell-marketplace/`
     (NEW).
   - `microservices/cloud-iac/iac/helm/service-cell-dev-tools/`
     (NEW).
   - `microservices/cloud-iac/iac/helm/service-cell-audit-aggregator/`
     (NEW).
   - `microservices/cloud-iac/iac/helm/service-cell-analytics/`
     (NEW).
   - `microservices/cloud-iac/iac/helm/service-cell-ops-console/`
     (NEW).
   - `microservices/cloud-iac/iac/helm/cell-tier-4-financial-grade/`
     (NEW; reserved; build-ahead-of-cert).
   - `microservices/cloud-iac/iac/helm/cell-tier-4-fulfillment-
     grade/` (NEW; reserved).
   - `microservices/cloud-iac/iac/helm/cell-tier-4-il5/` (NEW;
     reserved).
3. **Observability:**
   - `microservices/observability/dashboards/cellular-topology.md`
     (NEW). Surfaces per-cell + per-tier health, tenant
     distribution per cell, shuffle-sharding entropy, cell capacity
     utilisation, Tier 2 → Tier 3 snapshot pull lag, static-
     stability TTL coverage, cross-cell traffic permit counts.
   - Per-cell SLO dashboard (extends ADR-0241 DR portfolio
     dashboard with cell-tier dimension).
   - Per-cell sustainability dashboard (extends ADR-0174
     sustainability tag with per-cell PUE).
4. **Registry surfaces:**
   - `registry/cells/cell-registry.tsv` (NEW). Canonical list of
     all cells: cell_id, tier, region, provider, AZ, certification
     levels, capacity envelope, status.
   - `registry/cells/tenant-bindings.tsv` (NEW). Sharded by
     pack; tenant_id → shuffle shard.
   - `registry/cells/cell-decommissioning.tsv` (NEW). Cells
     pending decommission + migration plan.
5. **Runbooks:**
   - `docs/runbooks/cell-tier-1-bootstrap.md` (NEW; per ADR-0242
     §D-5 inheritance).
   - `docs/runbooks/cell-tier-1-self-retirement.md` (NEW).
   - `docs/runbooks/cell-tier-2-provisioning.md` (NEW).
   - `docs/runbooks/cell-tier-3-auto-spawn.md` (NEW).
   - `docs/runbooks/cell-tier-3-decommissioning.md` (NEW).
   - `docs/runbooks/tenant-cell-migration.md` (NEW; per §D-12).
   - `docs/runbooks/cell-failover-cross-region.md` (NEW; per
     ADR-0241 inheritance).
   - `docs/runbooks/cell-static-stability-drill.md` (NEW;
     quarterly Tier 2 isolation drill).
   - `docs/runbooks/shuffle-sharding-parameter-change.md` (NEW).
6. **HSM ceremony:**
   - Per-cell HSM partition provisioning ceremony, owned by
     `oyatie.security.hsm-ops`. Quarterly partition health audit.
7. **Capacity planning:**
   - Per-cell capacity envelope tracking, owned by ops-dr-capacity.
   - Cell-pool admission gate refuses tenant onboarding when pool
     utilisation > 90% (forces auto-spawn before saturation).

### Sustainability

Per ADR-0174 sustainability tag, every cell tracks PUE (Power
Usage Effectiveness) of its underlying datacenter:

- AWS us-east-1: PUE 1.21
- Naver Cloud Chuncheon: PUE 1.18
- OVH France: PUE 1.09
- Google Cloud europe-west4 (Eemshaven): PUE 1.10
- Azure West Europe (Amsterdam): PUE 1.20

Per-cell PUE is the basis for the FinOps + carbon reporting. The
cellular-topology dashboard surfaces per-cell PUE; the FinOps
portal (per ADR-0245 §D-3.B `finops-portal`) shows per-tenant
carbon attribution based on tenant's cell PUE + cell tenancy
fraction.

**Per-cell PUE tracking is mandatory.** Every cell registered in
`registry/cells/cell-registry.tsv` declares its PUE; quarterly
audit confirms the value with the underlying datacenter's
sustainability report.

Cellular architecture's net sustainability impact:

- **Negative (raw compute):** Per-cell baseline overhead × N cells
  > monolith baseline. Estimated 5-15% incremental compute cost
  for substrate redundancy across cells.
- **Positive (utilisation):** Per-cell auto-scaling enables
  fine-grained capacity match; reduces over-provisioning relative
  to monolith. Estimated 10-25% utilisation improvement.
- **Positive (carbon-aware placement):** Future workload placement
  can bias toward lower-PUE cells within a pack's allowed
  providers. Substrate exists; policy comes Year 2+.

Net effect: roughly neutral to mildly positive sustainability;
substantial visibility improvement (per-tenant carbon attribution
becomes possible).

### Compliance

Per ADR-0251 compliance pack cell-certification levels, cells
declare certification levels:

- **SOC 2 Type II** — most cells. Annual audit per cell pool.
- **ISO 22301** — Tier 1 + Tier 2 cells (business continuity).
- **HIPAA** — designated Tier 3 cells in US + KR healthcare packs.
  Per-cell BAA with tenants requesting HIPAA.
- **PCI DSS v4.0 Service Provider Level 1** — Tier 4 financial-
  grade cells (when authored).
- **CSAP (KR)** — pack-kr Tier 3 cells. Per-cell K-ISMS-P
  certification.
- **GAIA-X (EU)** — pack-eu Tier 3 cells. Per-cell GAIA-X label.
- **FedRAMP Moderate / High** — Tier 3 cells in US-Government
  pack. Tier 4 IL5+ planned for DoD workloads.
- **NDMO / SDAIA (KSA)** — pack-ksa Tier 3 cells.
- **METI Cloud Security Mark (JP)** — pack-jp Tier 3 cells.

**Per-cell certification audit cadence** is quarterly to annual
depending on the certification framework. ops-compliance maintains
the per-cell certification calendar at
`registry/compliance/cell-certification-calendar.yaml`.

**Cell certification level signals tenant binding eligibility.**
Per ADR-0251 + ADR-0244 §D-3 tenancy table, tenants declare
required compliance packs; shuffle-sharding restricts the cell pool
to cells whose certification levels match the required packs.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/cell-topology.json` | NEW — canonical four-tier topology schema |
| `/specs/cell-tier-definitions.json` | NEW — per-tier enum + responsibilities |
| `/specs/cell-certification-levels.json` | NEW — per ADR-0251 |
| `/specs/shuffle-sharding-parameters.json` | NEW — pool definition + shard widths |
| `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md` | UPDATE — cell is pattern, not service |
| `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | UPDATE — cell registry, lifecycle, provisioning, planned capacity |
| `microservices/tenancy/ARCHITECTURE.md#cell-assignment` | UPDATE — tenant→cell binding storage and migration state |
| `microservices/observability/ARCHITECTURE.md#cell-health` | UPDATE — per-cell health, SLO burn, blast-radius isolation alerts |
| `microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing` | UPDATE — cell-aware routing |
| `microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit` | UPDATE — per-cell audit scope |
| `crates/oya-shuffle-sharding` | NEW — pure deterministic shuffle-sharding algorithm |
| `microservices/cloud-iac/iac/helm/cell-tier-1/` | NEW — bootstrap cell Helm chart |
| `microservices/cloud-iac/iac/helm/cell-tier-2/` | NEW — control plane cell Helm chart |
| `microservices/cloud-iac/iac/helm/cell-tier-3/` | NEW — data plane cell Helm chart |
| `microservices/cloud-iac/iac/helm/service-cell-marketplace/` | NEW |
| `microservices/cloud-iac/iac/helm/service-cell-dev-tools/` | NEW |
| `microservices/cloud-iac/iac/helm/service-cell-audit-aggregator/` | NEW |
| `microservices/cloud-iac/iac/helm/service-cell-analytics/` | NEW |
| `microservices/cloud-iac/iac/helm/service-cell-ops-console/` | NEW |
| `microservices/cloud-iac/iac/helm/cell-tier-4-financial-grade/` | NEW — reserved per ADR-0250 |
| `microservices/cloud-iac/iac/helm/cell-tier-4-fulfillment-grade/` | NEW — reserved |
| `microservices/cloud-iac/iac/helm/cell-tier-4-il5/` | NEW — reserved |
| `microservices/cloud-iac/iac/opentofu/<provider>/cell-tier-3-network/` | NEW per provider — per-cell network module |
| `microservices/cloud-iac/iac/opentofu/<provider>/cell-tier-3-compute/` | NEW per provider |
| `microservices/cloud-iac/iac/opentofu/<provider>/cell-tier-3-storage/` | NEW per provider |
| `microservices/cloud-iac/iac/opentofu/<provider>/cell-tier-3-hsm/` | NEW per provider |
| `microservices/observability/dashboards/cellular-topology.md` | NEW — per-cell health + topology dashboard |
| `microservices/observability/dashboards/shuffle-sharding-entropy.md` | NEW |
| `microservices/observability/dashboards/cell-capacity-utilization.md` | NEW |
| `microservices/observability/dashboards/static-stability-tier2-isolation.md` | NEW |
| `microservices/observability/dashboards/cross-cell-traffic-permits.md` | NEW |
| `registry/cells/cell-registry.tsv` | NEW |
| `registry/cells/tenant-bindings.tsv` | NEW |
| `registry/cells/cell-decommissioning.tsv` | NEW |
| `registry/cells/cell-certification-calendar.yaml` | NEW |
| `registry/cells/per-cell-pue.tsv` | NEW |
| `docs/standards/cellular-architecture.md` | NEW — full standards doc with worked examples |
| `docs/standards/shuffle-sharding-parameters.md` | NEW — operator guide to shard width selection |
| `docs/runbooks/cell-tier-1-bootstrap.md` | NEW |
| `docs/runbooks/cell-tier-1-self-retirement.md` | NEW |
| `docs/runbooks/cell-tier-2-provisioning.md` | NEW |
| `docs/runbooks/cell-tier-3-auto-spawn.md` | NEW |
| `docs/runbooks/cell-tier-3-decommissioning.md` | NEW |
| `docs/runbooks/tenant-cell-migration.md` | NEW |
| `docs/runbooks/cell-failover-cross-region.md` | NEW |
| `docs/runbooks/cell-static-stability-drill.md` | NEW |
| `docs/runbooks/shuffle-sharding-parameter-change.md` | NEW |
| `docs/runbooks/cell-hsm-partition-ceremony.md` | NEW |
| `tools/oya-check-cell-tier-coherence/` | NEW |
| `tools/oya-check-shuffle-sharding-parameters/` | NEW |
| `tools/oya-check-cell-isolation-tolerance/` | NEW |
| `tools/oya-check-cross-cell-traffic-permits/` | NEW |
| `tools/oya-check-cell-deployment-pattern/` | NEW |
| `tools/oya-check-static-stability-coverage/` | NEW |
| `tools/oya-check-constant-work-control-plane/` | NEW |
| Migration sweep: every µservice manifest gains `cellular_deployment_pattern` per ADR-0244 §D-5 | SWEEP |

## Verification

- [ ] ADR-0333 successor owners exist; `crates/oya-shuffle-sharding`
  builds and passes its integration test set.
- [ ] `oya gate validate cell-tier-coherence` returns exit 0 on a
  clean build with all µservices declaring
  `cellular_deployment_pattern`.
- [ ] `oya gate validate shuffle-sharding-parameters` returns exit
  0; `/specs/shuffle-sharding-parameters.json` declares `S = 8`
  default with documented overrides for sandbox (`S = 4`),
  enterprise (`S = 16`), and payments (`S = 32`).
- [ ] `oya gate validate cell-isolation-tolerance` returns exit 0;
  every Tier 3 µservice declares its cached-state set + TTL +
  fallback behavior.
- [ ] `oya gate validate cross-cell-traffic-permits` returns exit
  0 on representative product µservice (e.g.,
  `microservices/mail/`); no hot-path cross-cell calls.
- [ ] `oya gate validate static-stability-coverage` returns exit 0;
  every Tier 3 cached-state has a populated TTL ≤ 25 hours.
- [ ] `oya gate validate constant-work-control-plane` returns exit
  0; no Tier 2 → Tier 3 push-based per-change paths detected.
- [ ] Bootstrap cell provisioning drill: clean install from
  `kubeadm init` → Tier 1 cell → Tier 2 cell handoff → bootstrap
  cell self-retirement; audit-chain emits the full evidence trail
  per §D-2.
- [ ] Shuffle-sharding math test: 100-cell pool + `S = 8` test
  yields per-tenant 8-cell assignment with combinatorial-distinct
  shards for ≥ 99% of test tenant pairs.
- [ ] Tier 2 → Tier 3 snapshot pull benchmark: hot-reload < 5s
  p99; snapshot pull < 30s p99 for 10,000-tenant snapshot.
- [ ] 24-hour static-stability drill: Tier 2 isolated for 24 hours
  while Tier 3 continues serving 95% of pre-isolation request
  volume.
- [ ] Cell auto-spawn drill: capacity-loaded cell triggers auto-
  spawn within 30 minutes; new cell joins pool; tenant onboarding
  rebalances per §D-7 shuffle re-assignment.
- [ ] Cell decommissioning drill: tenant migration from
  decommissioning cell completes within tier-specific SLO; source
  cell K8s cluster deprovisioned with audit-chain receipt.
- [ ] Per-cell HSM partition provisioning ceremony drilled.
- [ ] `cellular-topology.md` dashboard surfaces per-cell health +
  shuffle-sharding entropy + capacity utilisation + Tier 2
  snapshot lag.
- [ ] At least 2 sovereign packs (KR + EU) have a working Tier 3
  cell template deployed per ADR-0240 + ADR-0245 + this ADR.

## References

### AWS Builder's Library + Hyperscaler primary sources

- **James Hamilton, "On Designing and Deploying Internet-Scale
  Services" (USENIX LISA '07, 2007, pp. 233-244).** Foundational
  text on cell-based architecture; eighteen principles for
  hyperscale operations.
- **Colm MacCárthaigh, "Shuffle Sharding: Massive and Magical
  Fault Isolation" (AWS Architecture Blog, 2014-08-19).** Canonical
  public reference on shuffle sharding mathematics + Route 53
  production example.
- **AWS re:Invent 2018 ARC408 — "Designing for Failure with
  Cellular Architecture"** (Brad Calder, November 28, 2018).
  Production cellular architecture at AWS: S3, DynamoDB, Lambda,
  Step Functions.
- **AWS re:Invent 2024 ARC405 — "Cell Architecture in Practice"**
  (Tom Killalea + Colm MacCárthaigh, December 3, 2024). Updated
  framing with 2018-2024 production data; per-service shard-width
  recommendations; wave deployment doctrine.
- **AWS re:Invent 2024 ARC404 — "Static Stability at
  Hyperscale"** (Becky Weiss + Mike Furr, December 4, 2024).
  Operational guidance on Tier 2 isolation tolerance.
- **AWS Builders' Library — "Static stability using Availability
  Zones"** (Becky Weiss + Mike Furr, 2020;
  aws.amazon.com/builders-library/static-stability-using-
  availability-zones/). The canonical static-stability article.
- **AWS Builders' Library — "Reliability, constant work, and a
  good cup of coffee"** (Marc Brooker, 2020;
  aws.amazon.com/builders-library/reliability-and-constant-
  work/). The canonical constant-work-pattern article.
- **AWS Builders' Library — "Building dashboards for operational
  visibility"** (2023). Per-cell observability patterns.
- **Werner Vogels Re:Invent 2019 Keynote** (December 3, 2019).
  Amazon.com runs on AWS as a tenant; cell-based isolation across
  internal + external customers.

### Stripe + other hyperscaler sources

- **Stripe Engineering Blog — "Building cellular architecture at
  Stripe" (March 14, 2024;
  stripe.com/blog/cellular-architecture).** Stripe's adoption of
  cellular architecture for global payments; per-account cell
  binding; synchronous regional replication for payments-grade
  workloads.
- **Cloudflare Engineering Blog — "Building Pingora" (2022) +
  "Open-sourcing Pingora" (2024).** Pingora as Rust-based HTTP
  proxy; canonical reference for self-hosted edge.
- **Cloudflare Engineering Blog — "Workers Runtime API"
  (2023-2024).** V8 isolate model + ~300 POPs as edge density.
- **Google CRE Book — chapter 8 ("Reducing toil")** and
  **Google SRE Workbook — chapters 2 + 4-5** (Beyer et al.,
  O'Reilly 2016 + 2018). SLO bar; tier-aligned DR; per-cell
  operational tooling.
- **Apple WWDC 2023 — "Maintaining the highest reliability for
  iCloud" session 10240.** Apple iCloud cellular architecture
  patterns (Apple does not publish full topology but the session
  describes per-shard isolation + per-region failover).
- **Microsoft Azure — "Scale units" documentation 2024.** Azure's
  internal cellular pattern (called "scale units"). Per-region
  + per-AZ scale-unit composition.
- **Salesforce Trust — "Pods and Instances" 2024.** Salesforce's
  internal cellular pattern (called "pods" + "instances").
  Per-pod tenant binding; pod auto-spawn.

### Cilium + Service mesh + Kubernetes

- **Cilium 1.16 LTS documentation** (cilium.io, 2024). eBPF CNI +
  L4 mesh + Hubble observability + ClusterMesh.
- **Istio Ambient 1.24 LTS documentation** (istio.io, 2024).
  Sidecarless dataplane + ztunnel + waypoint.
- **Solo.io reference architecture — "Cilium L3/L4 + Istio
  Ambient L7 layered" (2024).** The canonical layered-mesh
  reference.
- **SPIFFE + SPIRE** (spiffe.io, CNCF Graduated 2022). Workload
  identity.
- **Kubernetes Cluster API documentation** (cluster-api.sigs.k8s.io,
  v1.6+, 2024). Per-cell K8s cluster provisioning.
- **Karpenter documentation** (karpenter.sh, 2024). Per-cell
  node auto-scaling.

### Cloud Hypervisor + Kata Containers + confidential computing

- **Cloud Hypervisor specification + source** (cloud-hypervisor.org,
  2024). Rust-based VMM; KVM-backed; Apache 2.0 license; AMD
  SEV-SNP + Intel TDX support.
- **Kata Containers documentation** (katacontainers.io, 3.x,
  2024). Kubernetes CRI runtime wrapping Cloud Hypervisor.
- **AMD SEV-SNP specification** (AMD Developer Central, 2024).
  Secure Encrypted Virtualization — Secure Nested Paging.
- **Intel TDX specification** (intel.com/tdx, 2024). Trust Domain
  Extensions for confidential VMs.
- **Confidential Computing Consortium — "Common Terminology"
  (2024).** CCC's TDX + SEV-SNP + confidential containers terms.

### Pingora + HTTP/3 + Edge

- **Pingora source + documentation** (github.com/cloudflare/pingora,
  open-sourced 2024). Rust-based HTTP/3-native proxy; production-
  proven at Cloudflare's edge.
- **HTTP/3 specification (RFC 9114, IETF, 2022).** QUIC-based
  HTTP/3.
- **QUIC specification (RFC 9000, IETF, 2021).** UDP-based
  transport.
- **TLS 1.3 specification (RFC 8446, IETF, 2018).** Mandatory
  for HTTPS.

### Post-quantum cryptography

- **NIST FIPS 203 (ML-KEM, formerly Kyber)** — Module-Lattice-
  Based Key-Encapsulation Mechanism Standard, published 2024-08.
- **NIST FIPS 204 (ML-DSA, formerly Dilithium)** — Module-
  Lattice-Based Digital Signature Standard, published 2024-08.
- **TLS Working Group draft on hybrid PQ KEX** (IETF
  draft-ietf-tls-hybrid-design, 2024).

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** AMENDED
  by this ADR (extends with formal four-tier + shuffle sharding +
  static stability + constant work).
- **ADR-0010 — Regional pack architecture.** Cells live within
  pack bounds.
- **ADR-0028 — Cloud microservice architecture.** Per-µservice
  contracts inherited.
- **ADR-0049 — Cross-region replication + residency.** Cell-level
  replication shapes.
- **ADR-0099 — Data class registry.** Per-data-class enforcement
  at cell boundary.
- **ADR-0105 — Thirteen-layer canonical enum.** Cell substrate is
  kernel-layer.
- **ADR-0121 — On-prem K8s stack.** K8s baseline per cell.
- **ADR-0128 — Hyperscaler architecture invariants.** This ADR is
  one such invariant.
- **ADR-0131 — Per-microservice flat layout.** Cell substrate
  follows layout.
- **ADR-0132 — No-grouping forward policy.** Cell substrate is single-
  concern.
- **ADR-0144 — EU AI Act graduated-risk tier model.** High-risk AI
  inference uses confidential computing per §D-14.
- **ADR-0145 — Inter-microservice communication reform.** Direct
  gRPC inside cells; cross-cell via durable workflow.
- **ADR-0148 — Service mesh Cilium ambient layered.** Per-cell
  mesh substrate.
- **ADR-0150 — Cedar policy engine.** Cell-local Cedar evaluator.
- **ADR-0174 — Sustainability tag.** Per-cell PUE.
- **ADR-0176 — Brown-out + degradation signal.** Cell isolation
  signal.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Both
  per-cell.
- **ADR-0211 — In-house Rust-primary tech stack.** Cell substrate +
  Cloud Hypervisor Rust-aligned.
- **ADR-0212 — Buildability doctrine.** Reserved Tier 4 cells +
  service cells ship skeletons.
- **ADR-0240 — Sovereign cloud per regional pack.** Cells live
  within pack-bound providers.
- **ADR-0241 — DR + business-continuity portfolio policy.** Per-
  µservice DR tier declares per-cell replication shape.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine (keystone #1).**
  `oyatie` tenant shuffle-shards uniformly with customer tenants.
- **ADR-0243 — Cedar as universal gate (keystone #2).** Cell-
  local Cedar; cross-cell call permits.
- **ADR-0244 — Tenant as universal scoping primitive (keystone
  #3).** Tenant `home_cell` + `dr_cell` + `cellular_deployment_pattern`.
- **ADR-0245 — Substrate vs Product layering (keystone #4).**
  Cell substrate is `substrate-infra` subtype.
- **ADR-0246 — Policy-engine substrate promotion (keystone #5).**
  Policy engine on Tier 2.
- **ADR-0247 — Self-hosting / self-modification doctrine
  (keystone #6).** Self-modification via Tier 1 bootstrap + Tier 2
  control-plane operations + Tier 3 wave deployments.
- **ADR-0249 — Multi-category marketplace doctrine.** Marketplace
  service-cell tier.
- **ADR-0250 — Build-ahead-of-certification doctrine.** Tier 4
  reserved per this doctrine.
- **ADR-0251 — Compliance pack + cell certification levels.**
  Cells declare certification levels.
- **ADR-0253 — Network topology, edge, service mesh.** Edge +
  HTTP/3 + Cilium ambient + per-cell ingress.

### Auto-memory feedback

- `feedback_quality_performance_scalability_bar` — reinforced;
  cellular is the hyperscaler-grade shape.
- `feedback_clean_architecture_requirements` — reinforced;
  inward-only flow at cell boundary.
- `feedback_no_silent_regression` — reinforced; per-cell
  versioning + wave deployment.
- `feedback_canonical_base_localization` — reinforced; per-pack
  cell pools enable per-pack overlay.
- `feedback_oyatie_is_a_tenant_doctrine` — reinforced; oyatie
  shuffle-shards uniformly.
- `feedback_autonomous_implementation_artifacts` — reinforced;
  cellular substrate enables autonomous masterplan.
- `feedback_bominal_inheritance_precedence` — applies; oyatie
  cellular overrides any narrower Bominal cellular framing.
- `feedback_automate_everything` — reinforced; auto-spawn +
  planned-migration workflows automate cell lifecycle.
- `feedback_flat_product_catalog` — preserved; cells are deployment
  topology, not product catalog.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone bundle, every
decision in this ADR is attributed to a named hyperscaler pattern +
source citation + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Tier 0 external dependencies) | "External Dependency Inventory" | AWS Well-Architected Reliability pillar; Google CRE Book ch. 8 | "Undocumented External Coupling" — surprise outage when external dependency drops |
| D-2 (Tier 1 bootstrap cell) | "Bootstrap-and-Retire" | rustc stage0 bootstrap; Kubernetes kubeadm; Certificate Transparency log bootstrap | "Eternal Bootstrap" — bootstrap cell never retires; becomes architectural sediment |
| D-3 (Tier 2 control plane cells) | "Control Plane / Data Plane Separation" | AWS Route 53 control plane; GCP Spanner zone-master separation; Stripe Cells 2024 control plane | "Co-Located Control + Data" — control plane saturation propagates to data plane |
| D-4 (Tier 3 data plane cells) | "Per-Tenant-Group Cell" | AWS S3, Lambda cellular; Stripe Cells 2024 per-account cell; Salesforce Pods | "One Cell Per Service" — per-service cells produce per-cell-fixed-cost × N anti-pattern |
| D-5 (service cells) | "Peer-Tier Dedicated-Function Cell" | AWS Marketplace, AWS IAM Access Analyzer; Stripe Connect; Salesforce AppExchange | "Service-Cell-Sprawl" — every µservice gets a service cell |
| D-6 (per-cell vs cross-cell bright line) | "Hot-Path-Intra-Cell" | AWS S3 partition boundary; Google Spanner replica locality; Stripe per-account locality | "Cross-Cell Hot Path" — cross-cell synchronous call introduces fault correlation |
| D-7 (shuffle sharding `S=8`) | "Shuffle Sharding" | MacCárthaigh 2014 AWS Architecture Blog; Route 53 production; AWS Lambda concurrency model | "Single-Cell-Per-Tenant" — cell failure = tenant fully offline |
| D-8 (static stability 24h tolerance) | "Static Stability" | Weiss/Furr 2020 AWS Builder's Library; AWS 2024 ARC404 | "Fail-Fast-On-Control-Plane-Outage" — data plane shuts down when control plane goes |
| D-9 (constant work) | "Constant Work" | Brooker 2020 AWS Builder's Library; Route 53 health propagation | "Push-Per-Change Delta" — control plane scales with change rate × fleet size |
| D-10 (cell sizing + auto-spawn at 70%) | "Capacity-Aware Auto-Spawn" | AWS Lambda concurrency scaling; Kubernetes HPA + Karpenter; Stripe Cells 2024 sizing | "Manual Cell Provisioning" — operations bottleneck at scale |
| D-11 (cross-region routing GeoDNS) | "GeoDNS + Edge Failover" | Cloudflare GeoDNS + edge POPs; AWS Route 53 latency-based routing; Akamai EdgeDNS | "Centralised DNS Hot-Spot" — single DNS point of failure |
| D-12 (planned migration workflow) | "Audit-Trail-Backed Tenant Migration" | AWS Outposts migration; Stripe Cells 2024 account migration | "Live Tenant Migration Without Audit" — migration drops data; no rollback |
| D-13 (K8s-everything except edge) | "Workload-In-Pod Default" | Google Kubernetes Engine; AWS EKS; Microsoft Azure AKS | "Snowflake Workload" — bespoke deployment per-µservice |
| D-14 (Cloud Hypervisor + Kata Containers) | "VM-Per-Workload Isolation" | AWS Firecracker; Kata Containers at Bytedance + Tencent + Microsoft; Confidential Computing Consortium | "Container-Only Isolation For Untrusted" — gVisor user-space sandbox; container escape |
| D-14 (NOT gVisor) | "KVM-Backed Isolation" | AWS Firecracker; Kata + Cloud Hypervisor; Linux KVM hardware-backed | "User-Space Syscall Interception" — gVisor Sentry bugs become escape vectors |
| D-15 (Cloudflare → Pingora; HTTP/3 default) | "Distributed Edge POP" | Cloudflare edge ~300 POPs; AWS CloudFront; Fastly POPs | "Centralised Ingress" — single ingress point; geographic latency |
| D-16 (Tier 4 reserved) | "Build-Ahead-of-Certification" | AWS GovCloud (built before FedRAMP-High cert); AWS HealthLake (built before HIPAA cert) | "Graft-On-After-Cert" — topology refactor under regulator deadline |

---

## Appendix B: Worked example — shuffle sharding 1000 tenants across 20 cells with `S=8`

To illustrate the shuffle-sharding math and operational properties,
consider a worked example. We model a pack with `C = 20` Tier 3
cells and `T = 1000` tenants, each with shard width `S = 8`.

### B.1. Probability math

**Number of possible shards.**

```
C(20, 8) = 125,970
```

There are 125,970 distinct 8-element subsets of a 20-cell pool. With
1000 tenants, the expected number of tenants per distinct shard is
`1000 / 125,970 ≈ 0.0079`, meaning ~0.79% of shards are populated;
99.2% of possible shards are empty. The probability of any two
specific tenants being assigned identical shards is:

```
P(tenant A and tenant B have same shard)
    = 1 / C(20, 8)
    = 1 / 125,970
    = 7.94 × 10^-6
```

That is, fewer than 1 in 125,000 tenant pairs share the same shard.

**Expected per-cell tenant presence.**

```
E[tenants present in any given cell]
    = T × S / C
    = 1000 × 8 / 20
    = 400 tenants per cell
```

Each cell hosts ~400 of the 1000 tenants in some capacity (home_cell,
dr_cell, or read_replica). The home_cell binding is exactly 1 of the
8, so the expected count of `home_cell == cell_X` is `1000 / 20 =
50 tenants` per cell.

**Cell failure impact analysis.**

*Single-cell failure (cell X goes down):*

- Tenants for whom cell X is `home_cell` (~50 tenants, 5%): they
  failover to `dr_cell`. Per ADR-0241 T1 = < 5min RTO, T2 = < 1h
  RTO. The tenant experiences brief degradation during failover but
  remains fully online from `dr_cell`.
- Tenants for whom cell X is `dr_cell` but not `home_cell` (~50
  tenants, 5%): they continue serving from `home_cell`; their DR
  posture is temporarily reduced to a `read_replica_cell` being
  promoted to dr until cell X recovers.
- Tenants for whom cell X is a `read_replica_cell` (~300 tenants,
  30%): they may lose some cross-region read capacity (those
  reads route to another `read_replica_cell`); end-user effect is
  minimal.
- Tenants for whom cell X is none of their 8 cells (~600 tenants,
  60%): zero impact.

Net: **5% of tenants experience brief RTO-bounded degradation; 0%
go fully offline; 60% see zero impact.**

*Two-cell failure (cells X and Y simultaneously down):*

```
P(tenant has BOTH cells X and Y in their 8-cell shard)
    = C(18, 6) / C(20, 8)
    = 18,564 / 125,970
    = 14.7%
```

But the probability that BOTH cells are the tenant's `home_cell` +
`dr_cell` (the operational-critical pair) is much smaller:

```
P(X = home_cell AND Y = dr_cell, or vice versa)
    = 2 × (1/20) × (1/19)
    = 2 / 380
    = 0.53%
```

So 0.53% of tenants (~5 tenants out of 1000) experience the rare
case where both their `home_cell` and `dr_cell` are down
simultaneously. Those tenants failover to a `read_replica_cell`
that is promoted to active; the read_replica's catch-up window
may be longer than the standard DR replication shape (per ADR-0241
this is the BC-tabletop scenario, not the standard quarterly drill).

*Three-cell failure (cells X, Y, Z simultaneously down):*

```
P(tenant has ALL three of X, Y, Z in shard)
    = C(17, 5) / C(20, 8)
    = 6,188 / 125,970
    = 4.91%
```

So ~49 tenants of 1000 (4.91%) have all three failed cells in their
shard. Their effective capacity is `5/8 = 62.5%` of normal; their
shard is degraded but not exhausted. No tenant goes fully offline
unless all 8 cells fail (probability `C(12, 0) / C(20, 8) ≈ 7.9 ×
10^-6`, i.e., fewer than 1 in 125,000 tenants).

### B.2. Wave deployment example

A deployment of a new µservice version rolls out wave by wave:

- **Wave 1: 1 cell out of 20.** A bad deploy in this cell affects
  the ~400 tenants whose 8-cell shard includes the cell, but each
  of those tenants has 7 healthy cells still serving them. End-
  user effect bounded to ~12.5% capacity reduction for ~40% of
  tenants for the duration of wave 1.
- **Wave 2: 4 cells (20% of pool).** Bake time at wave 1 (e.g.,
  30 min) lets the metric-gated rollback per ADR-0040 catch any
  regression before wave 2. Wave 2 affects ~80% of tenants in 1
  of 8 shards each.
- **Wave 3: 10 cells (50%).** 100% of tenants affected in 1-4 of
  8 cells.
- **Wave 4: 20 cells (100%).** Full rollout.

**Bad deploy blast radius if caught at wave 1:** ~40% of tenants
experience ~12.5% capacity degradation for the duration of wave 1
bake time (~30 min). Per Stripe Cells 2024 + AWS 2024 ARC405, this
is the canonical blast-radius bound for a bad deploy at hyperscale.

### B.3. Cell capacity utilisation

At 1000 tenants × 8 cells / 20 cells = 400 tenants per cell on
average. If a single cell receives an asymmetric tenant assignment
(say 60 home_cell-bindings + 60 dr_cell-bindings + ~600 read-
replica), and each home_cell tenant generates ~10 QPS, the cell
serves:

- Hot writes: ~600 QPS (home_cell tenants).
- Hot reads: ~600 QPS (home_cell tenants) + ~600 QPS (dr_cell
  catch-up writes replayed) + ~6000 QPS (cross-region read
  amplification from read-replica tenants).

Per D-10 sizing, this is well within a Tier 3 cell's capacity
envelope. Auto-spawn would trigger only when this cell or its
peers reach 70% utilisation, indicating that the pack is
approaching capacity saturation.

### B.4. Migration scenario

Consider tenant `tenant-acme-corp` with initial shard
`{cell-1, cell-3, cell-5, cell-7, cell-9, cell-11, cell-13, cell-15}`,
home_cell `cell-1`, dr_cell `cell-3`. Suppose the operations team
needs to decommission `cell-1`:

1. ChangeSet authored: source `cell-1` (home_cell), target
   `cell-3` (current dr_cell promoted to home_cell, with new
   dr_cell `cell-5` to maintain DR pair).
2. Pre-migration sync: tenant-acme-corp's home data in `cell-1`
   is replicated to `cell-3` (the new home_cell) and `cell-5` (the
   new dr_cell). Replication catches up; lag drops to < 100 ms.
3. Cutover: tenant row updated: `home_cell: cell-3, dr_cell:
   cell-5, read_replicas: {cell-7, cell-9, cell-11, cell-13,
   cell-15, cell-17}` (added cell-17 to keep S=8).
4. Edge POPs pull new binding within 5 min; subsequent requests
   route to `cell-3`.
5. `cell-1` retains tenant-acme-corp's data for 7 days (rollback
   window); then GC.
6. Audit-chain emits `TenantCellMigrationComplete` with source +
   target + replication evidence.

The migration is auditable + reversible (within rollback window) +
bounded in latency impact (sub-100ms replication lag).

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oya-check-cell-tier-coherence` | N/A (check-family) | `check`.`cell-tier-coherence` | CI fitness-check per ADR-0105 Amendment 2; verifies every µservice manifest declares a valid `bootstrap_tier` consistent with its cellular deployment pattern. |
| `oya-check-shuffle-sharding-parameters` | N/A (check-family) | `check`.`shuffle-sharding-parameters` | CI fitness-check; verifies shuffle-sharding shard-width parameters fall within bounds prescribed by §D-3 (N-choose-K ≤ 0.035% 3-cell-failure probability per A7 math errata fix). |
| `oya-check-cell-isolation-tolerance` | N/A (check-family) | `check`.`cell-isolation-tolerance` | CI fitness-check; verifies every Tier 3 µservice cross-cell call is gated by a Cedar cross-cell traffic permit fragment. |
| `oya-check-cross-cell-traffic-permits` | N/A (check-family) | `check`.`cross-cell-traffic-permits` | CI fitness-check; verifies no cross-cell hot-path call is made without a Cedar permit per §D-6 cell-isolation semantics. |
| `oya-check-cell-deployment-pattern` | N/A (check-family) | `check`.`cell-deployment-pattern` | CI fitness-check; verifies manifest `cellular_deployment_pattern` value matches the actual Helm chart deployment shape. |
| `oya-check-static-stability-coverage` | N/A (check-family) | `check`.`static-stability-coverage` | CI fitness-check; verifies cached-state TTLs + static-stability fallback paths are present per §D static-stability doctrine. |
| `oya-check-constant-work-control-plane` | N/A (check-family) | `check`.`constant-work-control-plane` | CI fitness-check; verifies no per-change push pattern in control-plane paths; control plane must use constant-work polling per AWS cellular architecture doctrine. |

---

*End of ADR-0248.*
