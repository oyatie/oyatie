---
plan_id: PLAN-HYPERSCALER-GRADE-CLOUD-GREENFIELD
title: Hyperscaler-Grade Cloud Platform Greenfield Plan
status: Draft
date: 2026-05-23
scope: greenfield-cloud-platform
research_posture: online-official-upstream-sources-only
repo_documentation_read: false
---

# Hyperscaler-Grade Cloud Platform Greenfield Plan

## 0. How To Read This Plan

This is a greenfield plan for creating a hyperscaler-grade cloud platform from
the ground up: treat the target as the first-generation operating model for a
company aspiring to build an AWS, Azure, or Google Cloud-class platform.

This plan is intentionally systematic across three lanes:

1. **Platform engineering:** the internal product, golden paths, control planes,
   self-service, developer experience, reliability, security, and operations
   model that let teams build and run cloud services at scale.
2. **Project and program management:** strategy, portfolio governance, product
   lifecycle, dependency management, launch gates, ownership, review cadence, and
   executive operating mechanisms.
3. **Development lifecycle:** specification, design, implementation, review,
   testing, secure supply chain, release, observability, incident response,
   deprecation, and continuous improvement.

No existing repository documentation is required to understand or execute this
plan.

### 0.1 Honest Claim Boundary

This plan is a roadmap to building a hyperscaler-class cloud platform; it is
not a claim that the organization already has AWS, Google Cloud, or Azure scale.
The honest claim is:

- the plan covers the same **classes of capabilities** required by hyperscale
  clouds: global account/IAM, regions, zones, cells, compute, networking,
  storage, managed runtime, data services, security, observability, billing,
  marketplace, support, compliance, and developer experience;
- the plan covers the same **operating disciplines** required at that scale:
  ownership, launch gates, SLOs, error budgets, incident response, secure SDLC,
  supply-chain controls, capacity planning, FinOps, and customer trust;
- the plan decomposes scale into **repeatable units**: service templates,
  service catalogs, regional control planes, cells, shards, fleet automation,
  and evidence-gated releases;
- the plan defines **measurable proof points** that must be achieved before
  preview, GA, region expansion, or portfolio expansion.

No team should market this as a hyperscaler until the relevant evidence gates
pass. Until then, it is a staged roadmap toward hyperscaler-class architecture,
quality, and operations.

---

## 1. Assumptions

1. The goal is not merely to deploy workloads on Kubernetes; the goal is to
   create a cloud provider with productized compute, storage, networking,
   identity, security, billing, observability, developer APIs, and operating
   model.
2. The first version may run on leased datacenter/colocation or public-cloud
   bootstrap capacity, but the architecture must be capable of transitioning to
   owned regions, availability zones, backbone networks, and hardware supply
   chain over time.
3. Customer workloads are hostile by default. Internal workloads are trusted only
   through identity, policy, workload isolation, and attestable deployment
   provenance.
4. Reliability must be designed through regional isolation, zone/cell isolation,
   static stability, graceful degradation, backpressure, retries with jitter,
   rate limiting, capacity buffers, and explicit SLOs.
5. The platform organization itself is a product organization. Internal service
   teams are customers of the platform engineering team; external cloud users
   are customers of the public cloud product.
6. The plan prefers boring, proven primitives over clever novelty. Novel
   technology requires an explicit risk review, blast-radius limit, migration
   plan, and rollback path.
7. "Hyperscaler scale" is achieved by repeatable regional, cell, service,
   operational, commercial, and governance systems. It is not achieved by one
   large cluster, one heroic team, or one monolithic control plane.
8. Every capability must be designed for public API stability, customer trust,
   operational ownership, fleet economics, and multi-year lifecycle management.

---

## 2. Best-Practice Research Summary

### Direct Recommendation

Build the cloud as a layered product system:

- Start with the **operating model**: ownership, SLOs, error budgets, design
  review, launch gates, incident response, and lifecycle discipline.
- Build the **foundation substrate**: regions, availability zones, cells,
  identity, policy, audit, metering, network, deployment, observability, and
  secure supply chain.
- Productize **service families** in this order: account/IAM/billing, compute,
  VPC networking, object/block storage, Kubernetes/containers, load balancing,
  databases, observability, security, marketplace, and higher-level services.
- Treat every service as an independent product with a public API, control
  plane, data plane, SLO, quota model, metering model, security model,
  operational runbook, and deprecation policy.
- Use cell-based scaling and fault containment. A failed cell must not take down
  a region; a failed region must not take down global control planes.
- Use platform engineering to provide paved roads: service templates, CI/CD,
  test harnesses, API standards, deployment lanes, observability, secure
  defaults, and self-service environments.

### Evidence Used

Official/upstream sources used for this plan:

- AWS Well-Architected Framework:
  https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html
  — establishes operational excellence, security, reliability, performance,
  cost optimization, and sustainability as recurring architectural review axes.
- AWS Implementing Microservices:
  https://docs.aws.amazon.com/whitepapers/latest/microservices-on-aws/microservices-on-aws.html
  — establishes API-driven, event-driven, and data-streaming microservice
  patterns, plus autonomous ownership and well-defined APIs.
- Google Cloud Architecture Framework:
  https://cloud.google.com/architecture/framework
  — establishes system design categories for reliability, operational
  excellence, security, privacy, compliance, cost, and performance.
- Azure Well-Architected Framework:
  https://learn.microsoft.com/en-us/azure/well-architected/
  — establishes workload assessment and design review categories for Azure-scale
  systems.
- Azure Architecture Center microservices domain analysis:
  https://learn.microsoft.com/en-us/azure/architecture/microservices/model/domain-analysis
  — supports designing service boundaries around business capabilities,
  bounded contexts, cohesion, and loose coupling rather than horizontal layers.
- Azure Architecture Center microservices style:
  https://learn.microsoft.com/en-us/azure/architecture/guide/architecture-styles/microservices
  — documents microservice antipatterns such as tight coupling through shared
  libraries and direct exposure without an API gateway.
- Google SRE book:
  https://sre.google/sre-book/service-level-objectives/
  — establishes SLOs and error budgets as reliability management primitives.
- DORA:
  https://dora.dev/guides/dora-metrics-four-keys/
  — establishes deployment frequency, lead time for changes, change failure
  rate, and failed deployment recovery time as delivery performance metrics.
- NIST Cybersecurity Framework 2.0:
  https://www.nist.gov/cyberframework
  — establishes governance, identify, protect, detect, respond, and recover
  as cybersecurity risk-management functions; the Govern function is especially
  relevant to accountability.
- Cloud Security Alliance Cloud Controls Matrix v4.1:
  https://cloudsecurityalliance.org/artifacts/cloud-controls-matrix-v4-1
  — establishes a cloud-focused control framework with 207 controls across
  17 security domains.
- FedRAMP Rev. 5:
  https://www.fedramp.gov/archive/2023-05-30-rev-5-baselines-have-been-approved
  — establishes a cloud authorization baseline aligned to NIST SP 800-53 Rev. 5
  for US federal cloud service providers.
- AWS Builders' Library on static stability:
  https://aws.amazon.com/builders-library/static-stability-using-availability-zones/
  — supports designing systems that keep serving traffic through dependency or
  zone failure.
- AWS Builders' Library on timeouts, retries, backoff, and jitter:
  https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/
  — supports bounded retries, jitter, and overload avoidance.
- AWS Builders' Library on load shedding:
  https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/
  — supports layered overload protection, early rejection, and visibility into
  shed traffic.
- AWS Builders' Library on shuffle sharding:
  https://aws.amazon.com/builders-library/workload-isolation-using-shuffle-sharding/
  — supports multi-tenant workload isolation and reduced correlated blast
  radius.
- Google SRE Workbook on canarying releases:
  https://sre.google/workbook/canarying-releases/
  — supports deployment to a subset of users, automated evaluation, and release
  integration before full rollout.
- Google SRE book on Production Readiness Reviews:
  https://sre.google/sre-book/evolving-sre-engagement-model/
  — supports service readiness review across architecture, dependencies,
  instrumentation, emergency response, capacity, change management, and
  performance.
- CISA Secure by Design:
  https://www.cisa.gov/resources-tools/resources/secure-by-design
  — supports secure defaults, transparency, accountability, and shifting
  cybersecurity risk reduction toward the manufacturer/provider.
- NIST Privacy Framework:
  https://www.nist.gov/privacy-framework
  — supports privacy-risk identification and management across data flows.
- FOCUS Specification:
  https://focus.finops.org/focus-specification/
  — supports standard technology billing and usage datasets; latest version
  observed during this review was FOCUS 1.3.
- Kubernetes architecture:
  https://kubernetes.io/docs/concepts/architecture/
  — establishes control plane and node separation for container orchestration.
- Kubernetes Deployments and Horizontal Pod Autoscaling:
  https://kubernetes.io/docs/concepts/workloads/controllers/deployment/ and
  https://kubernetes.io/docs/concepts/workloads/autoscaling/horizontal-pod-autoscale/
  — establish deployment rollout/rollback mechanics and horizontal scaling
  primitives for Kubernetes-hosted services.
- NIST Zero Trust Architecture:
  https://csrc.nist.gov/publications/detail/sp/800-207/final
  — establishes no implicit trust and continuous policy evaluation.
- NIST Secure Software Development Framework:
  https://csrc.nist.gov/publications/detail/sp/800-218/final
  — establishes secure software development practices.
- CISA/NSA Kubernetes Hardening Guidance:
  https://www.cisa.gov/resources-tools/resources/kubernetes-hardening-guidance
  — supports hardened cluster configuration, least privilege, logging, and
  network separation.
- Kubernetes Pod Security Standards:
  https://kubernetes.io/docs/concepts/security/pod-security-standards/
  — supports privileged, baseline, and restricted workload policy levels for
  managed Kubernetes and platform workloads.
- CNCF cloud native definition:
  https://github.com/cncf/toc/blob/main/DEFINITION.md
  — supports resilient, manageable, observable, loosely coupled systems.
- CNCF Platform Engineering Maturity Model:
  https://tag-app-delivery.cncf.io/whitepapers/platform-eng-maturity-model/
  — supports platform engineering maturity progression and platform-as-product
  thinking.
- OpenTelemetry docs:
  https://opentelemetry.io/docs/
  — establishes telemetry signals and instrumentation model.
- OpenTelemetry semantic conventions 1.41.0:
  https://opentelemetry.io/docs/specs/semconv/
  — establishes common telemetry attributes, span names, metric instruments, and
  resource conventions for consistent observability.
- SLSA specification v1.2:
  https://slsa.dev/spec/v1.2/
  — supports supply-chain provenance and build integrity.
- FinOps Foundation Framework:
  https://www.finops.org/framework/
  — establishes a cloud financial-management operating model across Inform,
  Optimize, and Operate phases.
- Green Software Foundation Software Carbon Intensity:
  https://greensoftware.foundation/standards/sci/
  — establishes ISO/IEC 21031:2024 software carbon-intensity measurement.
- OpenAPI Specification:
  https://spec.openapis.org/oas/v3.2.0.html
  — supports standard HTTP API description.
- OpenGitOps Principles:
  https://opengitops.dev/
  — supports declarative desired state, versioned changes, and reconciliation.
- Scrum Guide:
  https://scrumguides.org/scrum-guide.html
  — supports lightweight product backlog, sprint, review, retrospective, and
  increment concepts for teams that use Scrum.

### Version / Date Context

Research was performed on 2026-05-23. The plan uses official/upstream sources
for principles and operating practices, but implementation teams must re-check
versions, support windows, and current security guidance before selecting a
specific product version or making a production commitment.

### Boundaries / Non-goals

This plan does not choose a final vendor for hardware, colocation, optical
networking, billing tax compliance, or every managed service engine. It defines
the cloud platform architecture and execution system that can evaluate and
productize those choices.

---

## 3. Objective

Create a hyperscaler-grade cloud platform that can:

- Run internal and external customer workloads.
- Expose productized cloud services through stable APIs, consoles, SDKs, CLIs,
  and IaC providers.
- Scale through regions, availability zones, cells, and service-specific shards.
- Provide strong identity, authorization, isolation, audit, metering, billing,
  compliance evidence, and observability.
- Support secure multi-tenant compute, storage, networking, databases,
  containers, serverless, analytics, AI infrastructure, and managed operations.
- Operate with disciplined project management, service ownership, development
  lifecycle, launch gates, and reliability management.

Success means the platform can start with one internal region and grow into a
public cloud with repeatable regions, independently evolvable services, measured
SLOs, customer trust controls, and self-service developer velocity.

### 3.1 Hyperscaler-Scale Roadmap Thesis

AWS, Google Cloud, and Azure are not just collections of services; they are
operating systems for building, launching, and running services repeatedly at
massive scale. A credible roadmap must therefore define four compounding
systems:

1. **A cloud product system:** accounts, organizations, IAM, regions, zones,
   cells, service APIs, quotas, billing, support, status, compliance, and a
   developer experience that make the cloud consumable.
2. **A cloud infrastructure system:** datacenters or colocation, racks, hosts,
   network fabric, backbone, storage fleets, capacity buffers, hardware
   lifecycle, repair loops, supply chain, and physical security.
3. **A cloud engineering system:** service templates, API standards, clean
   architecture, CI/CD, security, observability, release automation, SLOs,
   incident response, and continuous optimization.
4. **A cloud business system:** product portfolio strategy, pricing, metering,
   contracts, marketplace, partner ecosystem, support, compliance attestations,
   regional expansion, and capital planning.

This plan is structured around those systems. Core IaaS is only the first proof
point; the roadmap becomes hyperscaler-grade only when the organization can
repeatably launch new service families and regions without reinventing the
foundation.

---

## 4. Product Scope

### 4.1 In Scope

| Domain | Product scope |
|---|---|
| Global foundation | Accounts, organizations, regions, availability zones, cells, quotas, tagging, resource model |
| Identity and access | User identity, service identity, IAM policies, roles, STS, federation, secrets, KMS |
| Control plane | API gateway, resource registry, workflow engine, scheduler, deployment orchestrator, audit, metering |
| Compute | VM, bare metal, images, placement, autoscaling, host maintenance, snapshots, GPU fleet |
| Containers | Managed Kubernetes, container registry, service mesh, ingress, policy, add-ons |
| Serverless | Functions, event triggers, build/deploy pipeline, sandboxing, scale-to-zero |
| Networking | VPC/VNet, subnets, routing, NAT, firewalls/security groups, load balancing, DNS, private link, backbone |
| Storage | Object storage, block volumes, file storage, archive, encryption, replication, lifecycle |
| Data services | Managed relational database, key-value/cache, event streaming, search, analytics, backups |
| Security | Zero trust, tenant isolation, vulnerability management, supply-chain security, SIEM/SOC integrations |
| Observability | Metrics, logs, traces, events, SLOs, alerting, dashboards, audit trails, customer telemetry export |
| Billing and commerce | Metering, pricing, invoices, budgets, credits, marketplace, partner offers, cost explorer |
| Developer experience | Console, CLI, SDKs, Terraform/OpenTofu provider, API docs, examples, service catalog |
| Platform engineering | Golden paths, paved roads, service templates, environment automation, policy-as-code, release automation |
| Project management | Portfolio governance, quarterly planning, launch gates, risk registers, dependencies, operating cadence |
| Lifecycle | Spec, design, threat model, implementation, tests, review, release, operations, incident, deprecation |

### 4.2 Out of Scope For First Public Release

- Consumer productivity platform.
- Ads business.
- General-purpose social network.
- Frontier AI model lab.
- Custom chip design.
- Attempting to match every AWS/Azure/GCP service before proving core cloud
  primitives.
- Global public launch before at least one region is operationally proven.

### 4.3 Hyperscaler Capability Maturity Ladder

The roadmap should be assessed by maturity, not by marketing breadth.

| Maturity level | Product reality | Required proof |
|---|---|---|
| L0: Prototype cloud | One-off demos of compute/network/storage | Manual operations accepted only in lab |
| L1: Internal platform | Internal workloads can run on account/IAM, network, compute, storage | Internal SLOs, audit, metering, and support loop exist |
| L2: Private preview cloud | Design partners can self-serve core resources under quotas | Support, billing preview, incident process, launch gates |
| L3: Public preview cloud | Broader customers can onboard with clear limits and preview SLOs | Self-service onboarding, status page, abuse prevention, docs |
| L4: GA regional cloud | Core services meet GA SLOs in at least one region | SLO history, DR evidence, billing accuracy, compliance evidence |
| L5: Multi-region cloud | Repeatable region buildout with consistent control planes | Region factory, capacity planning, regional compliance pack |
| L6: Multi-service hyperscaler | Many service families launch on common substrate | Service factory, marketplace, partner ecosystem, portfolio governance |
| L7: Global hyperscaler | Global backbone, multiple regions, enterprise trust, ecosystem gravity | Mature operations, large-scale capacity, resilient business model |

Every level must preserve the prior level's evidence. A team that launches many
services without SLOs, support, billing accuracy, and failure isolation has
expanded breadth without becoming hyperscaler-grade.

---

## 5. North-Star Architecture

### 5.1 Layered Model

```text
Customer interfaces
  Console | CLI | SDKs | IaC provider | API gateway | Partner marketplace

Global control layer
  Identity | Organizations | IAM | Billing | Metering | Audit | Resource registry
  Quotas | Policy | Workflow | Global catalog | Support | Compliance evidence

Regional control layer
  Region API cells | schedulers | placement | regional metadata | regional audit buffer
  regional metering buffer | deployment controllers | capacity controllers

Data-plane service families
  Compute | Containers | Networking | Storage | Databases | Eventing | Security | Observability

Infrastructure substrate
  Datacenters | racks | hosts | switches | routers | optics | power | cooling | hardware lifecycle

Operational substrate
  SLOs | telemetry | incident response | change management | backup/restore | DR | capacity | cost
```

### 5.2 Region, Zone, Cell, And Shard Model

| Unit | Purpose | Failure boundary |
|---|---|---|
| Global | Identity root, account registry, product catalog, global routing, billing aggregation | Must degrade gracefully; should not be required for most regional data-plane operations |
| Region | Geographic and regulatory boundary | Region outage must not cascade globally |
| Availability zone | Independent power/network/facility boundary within a region | Zone outage should be survivable by multi-zone services |
| Cell | Operational shard that contains a bounded slice of tenants/resources/control-plane components | Cell failure affects only assigned customers/resources |
| Service shard | Service-specific partition for scale and isolation | Shard failure affects only its partition |
| Host | Physical compute unit | Host failure handled by placement and repair automation |

Static-stability principle: a healthy zone or cell should continue serving
existing workloads during dependency failures where possible. Avoid designs that
require synchronous global dependencies for regional data-plane survival.

### 5.3 Control Plane / Data Plane Separation

| Plane | Examples | Rules |
|---|---|---|
| Global control plane | account creation, IAM roots, billing catalog, global resource registry | Strongly consistent where necessary; designed for degradation |
| Regional control plane | VM placement, volume attach, load balancer config, Kubernetes cluster lifecycle | Region-local operation during global impairment |
| Data plane | packet forwarding, object reads/writes, block I/O, VM execution, database requests | Must continue serving existing workloads during most control-plane outages |
| Management plane | deployment, monitoring, fleet repair, capacity, compliance evidence | Restricted identity; isolated from customer data path |

### 5.4 Hyperscaler Planes And Factories

A hyperscaler-grade roadmap needs repeatable factories, not bespoke launches.

| Plane/factory | Responsibility | Scaling proof |
|---|---|---|
| Region factory | Builds new regions with zones, cells, control planes, fleet, network, observability, support, and compliance pack | New region can be launched from a repeatable bill of materials and runbook |
| Cell factory | Creates bounded blast-radius cells for control-plane and data-plane services | New cell can be added, drained, upgraded, and retired without regional outage |
| Service factory | Generates new service skeletons with contracts, clean architecture, telemetry, SLOs, runbooks, launch gates, and billing hooks | New service can reach internal preview without custom platform work |
| Fleet factory | Provisions, images, inventories, repairs, drains, and retires hosts and network devices | Host fleet can absorb failure and maintenance through automation |
| API factory | Produces stable APIs, SDKs, CLI, IaC resources, docs, compatibility tests, and deprecation metadata | API consumers can build without bespoke integration support |
| Trust factory | Applies IAM, KMS, audit, policy, vulnerability, compliance, and evidence controls to every service | New service cannot bypass trust controls |
| Operations factory | Produces dashboards, alerts, incident templates, runbooks, support routing, and status-page integration | New service has operational readiness before launch |
| Commerce factory | Produces meter definitions, pricing hooks, budgets, invoices, credits, and cost allocation | New service can be billed and optimized |

Without these factories, the organization can build a cloud-like product but not
a hyperscaler-grade cloud. With them, every new region and service becomes a
repeatable program rather than an artisanal project.

### 5.5 Physical, Network, And Fleet Roadmap

The cloud platform must eventually own or deeply control the physical and
network substrate. The roadmap has staged maturity:

| Stage | Substrate posture | Goal |
|---|---|---|
| Bootstrap | Public cloud or leased capacity for control-plane development | Prove product and operating model quickly |
| Colo pilot | Leased racks, controlled network edge, owned host images | Prove fleet lifecycle, rack operations, and support model |
| Regional colo | Multiple zones in one region, private network fabric, regional control plane | Prove availability-zone and cell isolation |
| Owned region | Dedicated facilities or long-term strategic facilities, controlled power/cooling/security model | Improve economics, sovereignty, and operational control |
| Backbone expansion | Private inter-region backbone, edge PoPs, traffic engineering | Prove multi-region customer experience and replication |
| Hardware lifecycle | Forecasting, procurement, burn-in, secure provisioning, repair, RMA, retirement | Prove capacity economics and supply-chain resilience |

Minimum physical/fleet functions:

- hardware qualification and burn-in;
- secure boot and trusted provisioning;
- host identity and inventory;
- rack/host placement and capacity accounting;
- network device inventory and config reconciliation;
- break/fix and RMA workflow;
- secure wipe and retirement;
- spare-parts and supply-chain planning;
- power, cooling, PUE/WUE/CUE, and carbon reporting;
- physical access controls and audit evidence.

### 5.6 Global Network And Edge Roadmap

At hyperscaler scale, networking is a product and a substrate.

| Layer | Required capability |
|---|---|
| Region fabric | High-throughput, low-latency, redundant leaf/spine or equivalent network fabric |
| VPC overlay | Tenant-isolated address spaces, routing, firewall/security groups, NAT, private link |
| Load balancing | L4/L7 load balancing, health checks, target groups, failout, TLS integration |
| DNS | Hosted zones, health-aware routing, private DNS, resolver, DNSSEC path |
| Backbone | Inter-region private transport, traffic engineering, encryption, DDoS posture |
| Edge | CDN, DDoS absorption, API acceleration, regional ingress, customer peering |
| Observability | Packet drops, flow logs, latency, saturation, route convergence, customer-visible network health |

Network scale proof requires:

- route scale test;
- security group/rule scale test;
- load balancer target scale test;
- DNS query scale test;
- zone impairment routing test;
- DDoS tabletop and mitigation drill;
- flow-log and customer network-debug tooling.

### 5.7 Customer Trust, Compliance, And Shared Responsibility

Enterprise and regulated customers will not treat the platform as hyperscaler
grade unless trust is productized.

Required trust surfaces:

- public shared-responsibility model per service;
- IAM action/resource matrix per service;
- customer audit log export;
- customer key management and encryption posture;
- data residency and retention controls;
- vulnerability disclosure and patch SLAs;
- incident communication policy;
- compliance evidence portal;
- security whitepapers and architecture diagrams;
- penetration-test and audit report handling;
- customer support escalation and TAM/enterprise support path.

Trust maturity ladder:

| Level | Trust capability |
|---|---|
| T0 | Internal security controls only |
| T1 | Customer-visible IAM, audit logs, encryption, and status |
| T2 | Private-preview evidence packs and shared-responsibility docs |
| T3 | Third-party assessments and compliance roadmap |
| T4 | Formal certifications/authorizations for target markets |
| T5 | Continuous compliance evidence and customer self-service exports |

### 5.8 Service Portfolio Expansion Model

Core IaaS is necessary but insufficient. A hyperscaler roadmap must show how
new service families are added without destabilizing the foundation.

| Expansion stage | Service families | Promotion condition |
|---|---|---|
| Core trust | account, IAM, KMS, audit, metering, billing, support, status | Customer can safely own and pay for resources |
| Core IaaS | compute, VPC, block storage, object storage, load balancing, DNS | Customers can run basic applications |
| Runtime | managed Kubernetes, registry, serverless, service mesh | Customers can run modern platforms |
| Stateful managed services | relational DB, cache, event streaming, search, backup | Customers can run production applications |
| Data and analytics | lake, warehouse, streaming analytics, BI connectors | Customers can operate data platforms |
| AI infrastructure | GPU, accelerator scheduling, model serving, vector services | Customers can run AI workloads |
| Enterprise ecosystem | marketplace, private offers, partner network, support tiers | Customers and partners can build businesses on the platform |
| Global expansion | more regions, edge, backbone, compliance packs | Customers can run multi-region and regulated workloads |

Each new service family must reuse:

- account/IAM;
- KMS/secrets;
- audit and metering;
- quota/rate limiting;
- API/versioning standards;
- launch gates;
- observability and SLOs;
- billing dimensions;
- support/status integration;
- deprecation policy.

### 5.9 Hyperscaler Readiness Scorecard

This scorecard determines whether the roadmap is progressing toward
hyperscaler-grade reality.

| Category | Evidence |
|---|---|
| Product breadth | Core trust, IaaS, runtime, stateful services, data, AI, marketplace roadmap |
| Regional repeatability | Region factory, zone model, cell factory, regional compliance pack |
| Service repeatability | Service factory, clean architecture template, launch gates |
| Operational maturity | SLOs, error budgets, incidents, game days, support, status |
| Security maturity | zero trust, IAM, KMS, vulnerability program, compliance evidence |
| Reliability maturity | static stability, graceful degradation, DR, failure drills |
| Scalability maturity | partitioning, shard/cell addition, capacity forecasting |
| Commercial maturity | metering, billing, pricing, budgets, credits, marketplace |
| Developer maturity | console, CLI, SDK, IaC, docs, examples, support |
| Fleet maturity | hardware lifecycle, capacity buffers, network fabric, repair automation |

Preview and GA decisions should fail if this scorecard exposes a critical gap
without an explicit risk acceptance and customer-impact boundary.

### 5.10 Microservice And Clean Architecture Implementation Model

Microservices are an organizational, deployment, and scaling boundary. Clean
architecture is the internal design discipline inside each service. Use both:
microservices keep cloud products independently owned and horizontally scalable;
clean architecture keeps each service maintainable, testable, and insulated
from frameworks, vendors, and infrastructure churn.

#### 5.10.1 Service Boundary Rules

Create a microservice only when it has a durable business or infrastructure
capability, independent ownership, independent scaling profile, and clear data
ownership. Do not create a service for every table, every endpoint, or every
team preference.

| Boundary type | Examples | Rule |
|---|---|---|
| Global business foundation | account, organization, IAM, billing, metering, audit | Owns global customer/account truth; exposed through stable APIs and events |
| Regional control plane | regional resource registry, placement, quota, capacity, cell lifecycle | Operates region-locally during global impairment |
| Data-plane service | object data path, block I/O path, packet forwarding, VM host agent | Optimized for latency, throughput, and failure isolation; no synchronous global dependency |
| Product service family | compute, networking, storage, Kubernetes, database, observability | May contain multiple deployable microservices with one product owner |
| Platform service | service catalog, CI/CD, artifact signing, policy, telemetry, deployment orchestration | Serves internal engineers; must be productized and self-service |

Initial service map:

| Service | Primary responsibility | Scaling unit |
|---|---|---|
| account | Account, organization, region opt-in, lifecycle state | Global shard by organization/account |
| iam | Identity, roles, policies, STS, federation | Global + regional caches |
| audit | Append-only control-plane and privileged-action events | Region buffer, global aggregation |
| metering | Usage records and billing dimensions | Region buffer, service/account partitions |
| billing | Pricing, invoices, budgets, credits | Account shard |
| quota | Service quota and rate-limit decisions | Region/account shard |
| resource-registry | Canonical resource identity and lifecycle state | Region/cell shard |
| placement | Host/cell/zone placement and anti-affinity | Region/cell shard |
| compute-api | VM and image lifecycle control API | Region/cell shard |
| host-agent | Host-local VM lifecycle and health | One per host |
| vpc-api | VPC, subnet, route, security group, IPAM | Region/account shard |
| network-agent | Node/rack network programming | One per host/rack/network device class |
| load-balancer | Listener, target group, health, data-plane programming | Region/cell shard |
| dns | Hosted zones, records, resolver, health-based routing | Global + regional authoritative partitions |
| object-storage | Buckets, objects, lifecycle, replication | Region/cell/object partition |
| block-storage | Volumes, attach, detach, snapshots | Zone/cell partition |
| kms | Keys, grants, encryption operations | Region with hardware-backed protection |
| kubernetes | Managed clusters, node pools, add-ons, upgrades | Region/account/cluster shard |
| observability | Metrics, logs, traces, events, SLOs | Region/service/account partitions |
| support-status | Incidents, status page, customer communications | Global with regional failover |

#### 5.10.2 Clean Architecture Layers Inside Each Service

Each service uses inward dependencies. Business rules and service invariants do
not depend on HTTP, gRPC, Kafka, databases, Kubernetes, cloud SDKs, or framework
types.

```text
runtime/
  main process, dependency wiring, config loading, graceful shutdown

api/
  HTTP/gRPC handlers, auth context extraction, request/response mapping

worker/
  event consumers, scheduled jobs, replay workers, repair loops

application/
  commands, queries, workflows, sagas, idempotency, transactions

domain/
  entities, value objects, invariants, lifecycle state machines, policies

ports/
  traits/interfaces for repositories, event publishers, clocks, ID generation,
  policy checks, key operations, telemetry sinks, and external services

adapters/
  Postgres, object store, Kafka, KMS/HSM, Kubernetes API, network devices,
  hypervisor, filesystem, external provider clients

contracts/
  OpenAPI, protobuf, event schemas, public error model, compatibility tests
```

Dependency rule:

```text
runtime -> api/worker -> application -> domain
                    application -> ports
                    adapters -> ports
domain imports nothing from outer layers
```

Layer rules:

| Layer | Allowed | Forbidden |
|---|---|---|
| domain | pure types, invariants, state transitions, deterministic policy decisions | database clients, network clients, wall-clock access, random IDs, framework annotations |
| ports | interfaces, DTO-neutral abstractions, capability contracts | business logic, concrete clients |
| application | command handlers, query handlers, sagas, idempotency, transaction boundaries | direct SQL strings in business logic, HTTP response types |
| adapters | concrete repositories, event emitters, KMS clients, hypervisor clients, cloud device clients | owning domain invariants |
| api | request validation, auth context, mapping to application commands | business rules not expressed below |
| worker | event handling, replay, compensation, repair | hidden synchronous coupling |
| runtime | dependency injection, process lifecycle | product behavior |

#### 5.10.3 Standard Service Repository Shape

Every service generated from the golden path should look like this:

```text
services/<service-name>/
  README.md
  service.yaml
  contracts/
    openapi.yaml
    proto/
    events/
    errors.yaml
  src/
    domain/
    ports/
    application/
    adapters/
    api/
    worker/
    runtime/
  migrations/
  deploy/
    base/
    overlays/dev/
    overlays/stage/
    overlays/prod/
  tests/
    unit/
    contract/
    integration/
    chaos/
    load/
    security/
  runbooks/
  dashboards/
```

`service.yaml` is mandatory and includes:

- service owner
- product family
- lifecycle state
- API contracts
- data stores
- dependencies
- SLOs
- quota dimensions
- metering dimensions
- audit-event coverage
- dashboard links
- runbook links
- deployment cells
- rollback strategy

#### 5.10.4 Separation Of Concerns

| Concern | Owner | Implementation rule |
|---|---|---|
| Authentication | API gateway and service middleware | Extract actor context; never authorize solely at gateway |
| Authorization | IAM/policy service plus service-local enforcement | Every service validates action/resource/context before mutation |
| Idempotency | Application layer | Mutations require idempotency key and replay-safe result storage |
| Transactions | Application + repository adapter | Local transaction only; cross-service work uses saga/events |
| Cross-service consistency | Workflow/saga or event choreography | Avoid distributed transactions across service-owned databases |
| Data ownership | Owning service | No cross-service writes to another service database |
| Reads across services | API query, event projection, or materialized read model | No runtime database joins across services |
| Audit | Application layer emits through audit port | Audit cannot be an optional side effect after success |
| Metering | Application or data-plane boundary | Usage is emitted near the source of truth |
| Observability | Platform kit plus service instrumentation | Service owns useful domain metrics, not just generic process metrics |
| Deployment | Runtime/deploy layer | No service-specific manual production deployment |

#### 5.10.5 Communication Rules

Use the weakest coupling that satisfies the product requirement.

| Interaction | Preferred pattern | Notes |
|---|---|---|
| User or customer client to cloud | API gateway to service API | Gateway handles edge concerns; service still enforces authz |
| Service command to owning service | Synchronous API only when immediate answer is required | Timeouts, retries with jitter, circuit breaker, and idempotency required |
| Cross-service business workflow | Saga orchestration or event choreography | State is explicit and replayable |
| High-volume telemetry or usage | Async events/streams | Regional buffering required |
| Data-plane hot path | Local cache, local agent, or direct data-plane protocol | No synchronous global dependency |
| Read models | Event-driven projection | Eventually consistent and clearly documented |
| Internal admin workflow | Workflow engine | Human approval and audit for risky actions |

Do not allow:

- shared mutable database between services
- service A writing service B's tables
- synchronous call chains deeper than the approved latency budget
- global service dependencies in data-plane hot paths
- public API changes without contract versioning and compatibility tests

#### 5.10.6 Data Ownership And Consistency

Each microservice owns its write model. Shared data is copied through events or
queried through APIs.

Rules:

- One service owns each aggregate.
- One service owns each database schema.
- Cross-service workflows use sagas, not two-phase commit.
- Events are emitted through transactional outbox or equivalent atomic publish
  pattern.
- Events are versioned and backward compatible.
- Read models can be denormalized and eventually consistent.
- Customer-visible stale reads must publish staleness semantics.
- Data retention and deletion are service-owner responsibilities, verified by
  compliance tests.

For example:

```text
Create VM request
  compute-api validates request and IAM
  compute-api records idempotency key
  compute-api asks quota for regional/account allowance
  compute-api asks placement for candidate cell/host
  compute-api writes VM lifecycle state
  compute-api emits VmCreateRequested
  host-agent executes host-local VM creation
  host-agent emits VmStarted or VmStartFailed
  compute-api updates lifecycle state
  metering records VM-hours
  audit records control-plane mutation
```

#### 5.10.7 Horizontal Scalability Model

Horizontal scalability is designed at every layer.

| Layer | Scaling mechanism |
|---|---|
| Edge/API gateway | Global and regional load balancing; per-account rate limiting |
| Stateless API services | Multiple replicas per cell/zone; autoscale on RPS, latency, saturation, queue depth |
| Workers | Partitioned queues; autoscale on backlog age and processing latency |
| Control-plane state | Partition by region, cell, account, resource type, or shard |
| Data-plane agents | One or more per host/rack/device; local reconciliation loops |
| Databases | Sharding, partitioning, read replicas, leader/follower where necessary |
| Object storage | Partition by bucket/object hash plus region/cell placement |
| Block storage | Partition by zone/cell/volume; placement-aware replicas |
| Event streams | Partition by account, resource, region, or service-defined key |
| Observability | Partition by tenant/account, service, region, signal, and retention class |

Autoscaling policy:

- Use Kubernetes HPA for stateless API/worker scale where Kubernetes is the
  runtime.
- Prefer custom metrics over CPU-only scaling for cloud services: queue age,
  request latency, saturation, active connections, inflight operations, and error
  budget burn.
- Use vertical scaling only as a temporary pressure valve or for specialized
  data-plane components that cannot scale horizontally.
- Use predictive scaling only after reactive scaling and capacity buffers are
  proven.
- Preserve minimum warm capacity for control-plane and customer-facing services.

Partitioning rules:

- Every resource has account, region, zone/cell where applicable, and resource
  type in its identity or metadata.
- Every list API requires partition-aware filters and pagination.
- Every high-volume event stream has a declared partition key.
- Every service declares maximum shard size and split strategy.
- Every service supports adding a shard/cell without customer-visible downtime.

#### 5.10.8 Availability And Failure Isolation

Microservice architecture must not become distributed fragility. Use these
failure-isolation rules:

- Keep data-plane hot paths independent from global control-plane health.
- Keep regional control planes operational during global impairment.
- Use bulkheads for connection pools, worker pools, queues, memory, and CPU.
- Use circuit breakers and graceful degradation for optional dependencies.
- Use local caches with bounded staleness for authorization and metadata where
  safe.
- Prefer asynchronous repair over synchronous blocking during partial failure.
- Spread replicas across zones and cells using topology constraints.
- Use per-cell deployment and rollback before regional rollout.
- Run game days for dependency outage, cell outage, zone impairment, bad deploy,
  traffic spike, and metering backlog.

#### 5.10.9 Common Code Policy

Shared code is allowed only when it is stable, domain-neutral, and improves
consistency without coupling product behavior.

Allowed shared packages:

- typed IDs
- error model primitives
- pagination primitives
- telemetry wrappers
- auth context extraction
- idempotency primitives
- contract-test helpers
- retry/backoff utilities
- time and clock interfaces

Forbidden shared packages:

- cross-service business logic
- shared repositories that access multiple service databases
- "god" clients that hide dependencies
- shared domain models that force multiple services to release together
- framework abstractions that prevent services from choosing fit-for-purpose
  adapters

#### 5.10.10 Platform Enforcement

The golden service template and CI/CD gates enforce clean architecture and
microservice discipline:

- no imports from adapters into domain
- no framework types in domain
- no database clients outside adapters
- no service without owner/SLO/runbook
- no public API without contract tests
- no mutation without idempotency and audit
- no list endpoint without pagination
- no deployment without signed artifact and rollback
- no production service without dashboards and alerts
- no cross-service database access
- no global dependency in a declared regional data-plane hot path

---

## 6. Platform Engineering Operating Model

### 6.1 Platform Mission

The platform engineering organization exists to make the secure, reliable,
compliant path the fastest path for every cloud service team.

Platform engineering owns:

- Golden service templates.
- API and resource-model standards.
- CI/CD and release automation.
- Environment provisioning.
- Observability and SLO tooling.
- Secrets, identity, and policy defaults.
- Service catalog and ownership registry.
- Incident and launch readiness tooling.
- Developer portals and internal documentation.
- Compliance evidence automation.

Platform engineering does not own every product service. Product service teams
own their services end to end, including customer outcomes, SLOs, costs,
on-call, incidents, and deprecation.

### 6.2 Internal Developer Platform Capabilities

| Capability | Why it exists | Minimum viable internal feature |
|---|---|---|
| Service catalog | Know every service, owner, API, SLO, dependency, and runbook | Searchable registry with owners and lifecycle state |
| Golden paths | Reduce bespoke service creation | Templates for API service, worker, data-plane service, control-plane service |
| Environment automation | Reduce ticket-driven provisioning | Self-service dev/stage/prod cell creation with quotas |
| Policy-as-code | Make compliance fast and automatic | Pre-merge and pre-deploy policy checks |
| CI/CD | Make release repeatable | Build, test, scan, sign, deploy, canary, rollback |
| Observability kit | Make service health visible by default | Metrics, logs, traces, SLO dashboards, alerts |
| Security kit | Make secure defaults easy | IAM, secrets, mTLS, vulnerability scans, SBOM, provenance |
| Load and chaos kit | Make resilience testable | Load-test harness and fault-injection scenarios |
| Cost kit | Make unit economics visible | Per-service and per-tenant cost labels and reports |
| Launch gate automation | Make readiness objective | Checklist backed by evidence, not promises |

### 6.3 Platform Maturity Levels

| Level | Description | Exit criteria |
|---|---|---|
| L0 Manual | Teams provision and deploy manually | Inventory manual steps and critical risks |
| L1 Standardized | Templates and conventions exist | New service can start from a standard template |
| L2 Self-service | Teams provision standard environments without tickets | Self-service dev/stage/prod path works for simple services |
| L3 Guardrailed | Security, reliability, cost, and compliance gates are automated | Bad changes fail before deployment |
| L4 Measured | DORA, SLO, incident, cost, and quality metrics are visible | Teams can improve based on data |
| L5 Optimizing | Platform continuously reduces toil and improves outcomes | Toil, lead time, incidents, and cost per unit trend down |

---

## 7. Project And Program Management System

### 7.1 Organizational Model

| Group | Responsibility |
|---|---|
| Executive cloud council | Strategy, funding, risk acceptance, public launch decisions |
| Product council | Customer segmentation, packaging, roadmap, pricing, launches |
| Architecture council | Cross-service technical standards, risk reviews, design approvals |
| Security and trust council | Threat model, compliance, privacy, vulnerability response, audit readiness |
| Platform engineering | Internal developer platform, shared runtime, gates, paved roads |
| Infrastructure engineering | datacenters, fleet, network backbone, hardware, capacity |
| Service teams | Own individual cloud products end to end |
| SRE / production engineering | Reliability coaching, shared incident process, critical service operations |
| Program management office | Dependency tracking, milestone governance, risk register, executive reporting |
| Developer relations | SDKs, examples, docs, customer onboarding, feedback loops |

### 7.2 Operating Cadence

| Cadence | Meeting/artifact | Output |
|---|---|---|
| Annual | Strategy and capital plan | Regions, products, capacity, compliance targets, investment plan |
| Quarterly | Portfolio planning | Objectives, funding, committed launches, risk register |
| Monthly | Architecture and security review | Approved designs, accepted risks, remediation plans |
| Biweekly | Program increment review | Milestone status, dependency changes, blockers |
| Weekly | Service team execution | Sprint/flow plan, operational risks, readiness evidence |
| Daily | Team standup or async update | Blockers, handoffs, incident awareness |
| Per design | Design review | API, architecture, threat model, SLO, cost, launch plan |
| Per launch | Launch review | Evidence-based readiness approval |
| Per incident | Incident review | Timeline, contributing factors, action items, prevention |
| Per quarter | Retrospective | Metric review, process improvements, roadmap correction |

### 7.3 Required Artifacts

| Artifact | Owner | Required before |
|---|---|---|
| Product brief | Product manager | Roadmap commitment |
| PRD | Product manager + engineering lead | Design starts |
| Architecture design | Engineering lead | Implementation starts |
| API contract | Service team | Client development or public preview |
| Threat model | Security + service team | Implementation starts |
| SLO and error budget | Service team + SRE | Production deployment |
| Cost model | Service team + finance/FinOps | Public pricing |
| Launch plan | Service team + TPM | Customer preview |
| Runbook | Service team | Production deployment |
| Deprecation plan | Service team + product | Breaking change or retirement |
| Post-incident review | Incident commander | Incident closure |

### 7.4 Decision Gates

| Gate | Question answered | Evidence required |
|---|---|---|
| Concept gate | Should this exist? | Customer problem, alternatives, business value |
| Architecture gate | Can this be built safely? | Design, dependency map, threat model, SLO, cost model |
| Build gate | Is implementation ready? | API contract, task breakdown, test strategy |
| Preview gate | Can limited customers use it? | Functional tests, security review, docs, support plan |
| GA gate | Can all target customers rely on it? | SLO history, incident readiness, scale tests, billing, compliance |
| Expansion gate | Can it scale to more regions/cells? | Capacity proof, automation, regional checklist |
| Deprecation gate | Can it be changed or removed safely? | Customer impact, migration path, support window |

---

## 8. Development Lifecycle

### 8.1 Lifecycle Stages

```text
Idea
  -> Product brief
  -> PRD
  -> Architecture design
  -> Threat model
  -> API/resource contract
  -> Task breakdown
  -> Implementation slices
  -> Tests and verification
  -> Code review
  -> Security review
  -> Staged deployment
  -> Preview
  -> GA
  -> Operate and improve
  -> Deprecate or evolve
```

### 8.2 Engineering Rules

- Every public service has an API contract before implementation.
- Every service has an owner, SLO, runbook, dashboard, alert, and incident
  escalation path before production.
- Every service has tenant isolation tests, authz tests, quota tests, and
  failure-mode tests.
- Every deployment is progressive: dev, integration, staging, one cell, multiple
  cells, one region, multiple regions.
- Every rollback path is tested before launch.
- Every customer-impacting incident gets a blameless review and tracked action
  items.
- Every dependency has an owner, support window, upgrade process, and emergency
  patch process.
- Every artifact is signed or traceable to a trusted build.

### 8.3 Secure SDLC Controls

| Stage | Controls |
|---|---|
| Design | threat model, data classification, abuse cases, compliance mapping |
| Code | code review, static analysis, secret scanning, dependency review |
| Build | hermetic builds where feasible, SBOM, provenance, signed artifacts |
| Test | unit, integration, contract, fuzz, load, chaos, security tests |
| Deploy | policy checks, image signing verification, canary, rollback |
| Operate | detection, incident response, vulnerability response, patch SLAs |
| Retire | data deletion, customer migration, API sunset notices |

### 8.4 Quality Metrics

| Metric | Target |
|---|---|
| Deployment frequency | Improve over time without increasing incident rate |
| Lead time for changes | Shorten for safe standard changes |
| Change failure rate | Trend down; critical services set tighter budgets |
| Failed deployment recovery time | Trend down; critical services drill rollbacks |
| SLO attainment | Meets published targets within error budget |
| Toil percentage | Decreases as automation matures |
| Security remediation SLA | Met by severity class |
| Unit cost per service operation | Trends down with scale |
| Customer-impacting incident recurrence | Zero repeat incidents from same known cause |

---

## 9. Hyperscaler-Class Engineering Bar

The following seven dimensions are release blockers, not aspirations. A service
cannot pass preview or GA gates unless it has measurable evidence for each
dimension.

### 9.1 Performance

Hyperscaler-class performance means predictable tail latency under high
concurrency, noisy-neighbor pressure, partial failure, and regional scale.

| Requirement | Standard |
|---|---|
| Latency budgets | Every API defines p50, p90, p99, and p999 latency budgets before launch |
| Tail protection | Load shedding, queue limits, timeout budgets, retry budgets, and backpressure are mandatory |
| Benchmark tiers | Microbench, service load test, cell-scale test, region-scale simulation, and sustained soak test |
| Noisy-neighbor protection | Every shared substrate has per-account quota, fair scheduling, and abuse throttles |
| Capacity headroom | Critical services maintain explicit headroom and admission control |
| Hot path discipline | No synchronous global dependency on regional data-plane hot paths |
| Regression policy | Performance regressions block promotion unless explicitly risk-accepted |

Minimum release evidence:

- Baseline load test at 1x expected launch traffic.
- Stress test at 2x expected launch traffic.
- Failure-mode performance test with one dependency degraded.
- Tail-latency report showing p99 and p999 behavior.
- Capacity model with headroom, saturation points, and scale triggers.

### 9.2 Reliability

Hyperscaler-class reliability means the system is designed for component
failure, overload, operator error, dependency impairment, and regional
degradation.

| Requirement | Standard |
|---|---|
| SLOs | Every service has SLIs, SLOs, and error budgets |
| Error-budget policy | Feature velocity is constrained when error budget is exhausted |
| Blast-radius control | Region, zone, cell, shard, and account isolation are explicit |
| Static stability | Healthy zones/cells continue serving existing workloads during common dependency failures |
| Resilience testing | Chaos, game days, dependency failure, replay, and restore drills are required |
| Safe retries | Retries use bounded attempts, exponential backoff, jitter, and retry budgets |
| Idempotency | Customer-visible mutations are idempotent and replay-safe |
| Incident learning | Repeat incidents from the same known cause are treated as reliability failures |

Minimum release evidence:

- SLO dashboard with burn-rate alerts.
- Error-budget policy approved by product and engineering.
- Game-day report for cell failure, zone impairment, dependency outage, and
  rollback.
- Post-incident review template and action-item tracking.

### 9.3 Accountability

Hyperscaler-class accountability means every decision, change, operation,
resource, cost, risk, and customer impact has an owner and evidence trail.

| Requirement | Standard |
|---|---|
| Ownership | Every service, API, data store, dashboard, alert, runbook, and meter has an owner |
| Change traceability | Every production change is linked to requirement, review, build, artifact, deploy, and rollback |
| Auditability | Control-plane actions, privileged actions, policy decisions, and customer-impacting mutations are auditable |
| Risk ownership | Every accepted risk has an owner, expiry, mitigation, and review date |
| Cost ownership | Every service has unit-cost owner, cost allocation tags, and optimization backlog |
| Customer responsibility | Shared-responsibility boundaries are documented for every service |
| Governance | NIST CSF Govern-style roles, responsibilities, policies, and supply-chain oversight are explicit |

Minimum release evidence:

- Service catalog entry with owner and lifecycle state.
- RACI or directly responsible individual for launch, operations, security, and
  cost.
- Customer responsibility matrix.
- Audit-event coverage report.
- Risk register with no unowned critical risks.

### 9.4 Observability

Hyperscaler-class observability means operators and customers can understand
system behavior from outside the process, without ad hoc shell access or heroic
debugging.

| Requirement | Standard |
|---|---|
| Telemetry signals | Metrics, logs, traces, events, profiles, and audit records are available where appropriate |
| Semantic consistency | OpenTelemetry semantic conventions are used where applicable |
| Golden signals | Latency, traffic, errors, saturation, availability, and correctness are visible |
| RED/USE | Request-driven and resource-driven services expose the right metric families |
| Correlation | Request ID, trace ID, account, region, cell, zone, service, version, and deployment are correlated |
| Cardinality control | High-cardinality labels are governed and cost-limited |
| Customer visibility | Customer-impacting health is visible by account, region, service, and resource where safe |
| Debuggability | Dashboards link to logs, traces, recent deploys, runbooks, owners, and rollback actions |

Minimum release evidence:

- SLO dashboard.
- Service dashboard.
- Dependency dashboard.
- Customer-impact dashboard.
- Alert routing test.
- Trace/log/metric correlation test.

### 9.5 Scalability

Hyperscaler-class scalability means the platform grows by adding cells, zones,
regions, shards, hosts, and service partitions without redesigning core
architecture.

| Requirement | Standard |
|---|---|
| Horizontal scaling | Services scale horizontally by cell, shard, or partition |
| Control-plane scale | Control planes are partitioned; no single scheduler or database becomes universal bottleneck |
| Data-plane independence | Data planes continue handling existing workloads when control planes degrade |
| Quotas | Quotas prevent one account, region, or service from exhausting shared resources |
| Capacity planning | Demand forecasts, placement constraints, supply chain, and headroom are modeled |
| Elasticity | Autoscaling is driven by saturation and queue metrics, not just CPU |
| Backpressure | Overload is controlled through admission control and explicit rejection, not collapse |

Minimum release evidence:

- Cell-addition test.
- Shard-addition test.
- 2x launch-scale test.
- Capacity forecast for 6, 12, and 24 months.
- Quota and rate-limit abuse test.

### 9.6 Availability

Hyperscaler-class availability means users can rely on the platform during
planned maintenance, host failure, zone impairment, cell repair, dependency
outage, and partial regional disruption.

| Requirement | Standard |
|---|---|
| Multi-zone design | Customer-critical regional services are multi-zone where the product promise requires it |
| Maintenance safety | Host, cell, and zone maintenance use drain, migration, and customer communication workflows |
| Degraded mode | Services define read-only, write-limited, and emergency modes |
| RTO/RPO | Stateful services define and test recovery objectives |
| Failover | Failover is tested, not theoretical |
| Status communication | Customer-facing status and incident communication are launch requirements |
| Dependency classification | Services classify dependencies as hard, soft, cached, async, or optional |

Minimum release evidence:

- Availability architecture review.
- RTO/RPO drill.
- Maintenance drill.
- Dependency-failure drill.
- Status-page and customer-communication rehearsal.

### 9.7 Optimization

Hyperscaler-class optimization means the platform continuously improves
performance, cost, utilization, sustainability, reliability, developer
productivity, and customer value.

| Requirement | Standard |
|---|---|
| FinOps loop | Inform, Optimize, and Operate are embedded in service reviews |
| Unit economics | Every service tracks cost per API call, GB-month, VM-hour, request, or product-specific unit |
| Fleet utilization | Compute, storage, and network utilization are tracked with safe headroom |
| Continuous profiling | Hot services use profiling to guide optimization |
| Right-sizing | Idle, overprovisioned, and underutilized capacity is reviewed regularly |
| Carbon intensity | Carbon and energy metrics are tracked where data is available |
| Optimization backlog | Every service maintains a ranked optimization backlog |
| Guardrail | Optimization must not break SLO, security, compliance, or maintainability |

Minimum release evidence:

- Service unit-cost dashboard.
- Capacity utilization report.
- Optimization backlog.
- Carbon/energy measurement plan.
- Performance-per-cost review before GA.

### 9.8 Standards Mapping

| Dimension | Current official standards and frameworks to align with |
|---|---|
| Performance | AWS/GCP/Azure Well-Architected performance pillars; OpenTelemetry metrics; load, stress, soak, and overload testing |
| Reliability | Google SRE SLO/error-budget practices; AWS static stability, retry, load-shedding, and shuffle-sharding guidance; Well-Architected reliability pillars |
| Accountability | NIST CSF 2.0 Govern; CSA CCM v4.1; FedRAMP Rev. 5; CISA Secure by Design; service ownership registry |
| Observability | OpenTelemetry core docs and semantic conventions 1.41.0; SLO dashboards; RED/USE metrics; customer-impact correlation |
| Scalability | Cloud native architecture principles; cell-based, shuffle-sharded, and partitioned control-plane design |
| Availability | Multi-zone and failure-domain design; RTO/RPO; static stability; load shedding; incident communications |
| Optimization | FinOps Framework; FOCUS 1.3 cost-and-usage data; AWS sustainability pillar; Software Carbon Intensity ISO/IEC 21031:2024 |
| Privacy and data governance | NIST Privacy Framework; data inventory; retention/deletion; residency; customer export and deletion workflows |
| Runtime hardening | Kubernetes Pod Security Standards; CISA/NSA Kubernetes hardening guidance; sandboxing for untrusted workloads |
| Release safety | Google SRE canarying releases and Production Readiness Review; progressive delivery; one-cell and one-region gates |
| Secure lifecycle | NIST SSDF SP 800-218; SLSA v1.2; Kubernetes hardening guidance; Zero Trust Architecture |

### 9.9 Additional Hyperscaler Quality Requirements From Code Review

The quality review added the following requirements because they materially
improve hyperscaler-class correctness, security, reliability, performance, and
operability. These are release gates, not optional polish.

| Improvement | Required quality behavior | Evidence gate |
|---|---|---|
| Layered overload protection | Every public API and internal service hop defines admission control, request-cost classification, priority, fairness, bounded queues, retry budgets, and load shedding before saturation cascades. | Overload test demonstrates useful work continues while excess work is rejected with observable reason codes. |
| Shuffle-sharded tenant isolation | Multi-tenant control-plane and data-plane services assign tenants/resources to cells, shards, queues, and workers to reduce correlated failures and noisy-neighbor impact. | Isolation drill proves one tenant or shard cannot create regional or service-wide impairment. |
| Automated canary and PRR gate | Every service moves through design-time production readiness review, one-cell canary, automated canary evaluation, rollback, and post-rollout observation before wider rollout. | Canary gate blocks rollout on error, latency, saturation, cost, or customer-impact divergence. |
| Privacy and data governance | Every service declares personal-data categories, residency, retention, deletion, export, legal hold, logs, telemetry, and backup behavior. | Privacy control pack verifies inventory, deletion, export, and retention behavior in tests. |
| FOCUS-compatible cost data | Metering and billing produce cost-and-usage data with stable dimensions for account, service, region, resource, SKU, usage unit, tags, discounts, credits, and allocation. | Cost-data export validates against the selected FOCUS version and reconciles with invoice and unit-cost dashboards. |
| Secure-by-design accountability | Defaults are secure, high-risk features need explicit enablement, vulnerability response has public commitments, and customer-facing security evidence is maintained. | Secure-by-design scorecard is reviewed before private preview and GA. |
| Kubernetes runtime hardening | Managed Kubernetes and platform clusters enforce Pod Security Standards, namespace isolation, admission policy, least-privilege RBAC, image provenance, and network policy. | Runtime-hardening tests reject privileged or non-compliant workloads except through explicit, expiring exceptions. |
| Abuse, fraud, and DDoS readiness | Signup, quota, payment, support, network ingress, DNS, load balancing, and public APIs include abuse detection, throttling, suspension workflow, and customer-safe appeals. | Abuse/DDoS drill validates mitigation without harming unrelated tenants or customer communication. |

Minimum additional release evidence:

- Overload/load-shedding report with per-tier rejection and customer-impact
  metrics.
- Tenant-isolation/shuffle-sharding drill report.
- Automated canary evaluation and rollback report.
- Privacy inventory and deletion/export evidence.
- FOCUS-compatible cost-and-usage export validation.
- Kubernetes runtime-hardening policy report.
- Abuse, fraud, and DDoS readiness drill.

---

## 10. Technical Stack Baseline

This is a baseline for a greenfield implementation. Final selections require
additional dependency review, proof-of-concept validation, licensing review,
security review, and total-cost analysis.

| Area | Baseline choice |
|---|---|
| Primary service language | Rust or Go for control-plane and data-plane services; choose one primary per service family |
| API contracts | OpenAPI for HTTP, protobuf/gRPC for low-latency internal APIs, AsyncAPI or equivalent schemas for events |
| Identity | OIDC/SAML federation, workload identity, short-lived credentials, policy engine |
| Orchestration | Kubernetes for platform services and customer container service |
| Infrastructure as code | OpenTofu or Terraform-compatible workflow with policy checks |
| GitOps | Declarative desired state with reconciliation |
| Observability | OpenTelemetry-compatible traces, metrics, logs, events |
| Supply chain | SLSA-aligned provenance, SBOMs, signed artifacts |
| Data stores | PostgreSQL-compatible metadata store, strongly consistent key-value where required, object store, event log |
| Networking | BGP/EVPN/VXLAN or equivalent fabric, VPC overlays, programmable load balancing |
| Isolation | VM isolation for VMs, container isolation for trusted platform workloads, stronger sandbox/VM isolation for untrusted workloads |
| Security | Zero trust, least privilege, mTLS, policy-as-code, vulnerability management |

### 10.1 Greenfield Repository Structure

Use a **monorepo-first** layout for the initial cloud platform. A hyperscaler
cloud needs consistent API standards, release gates, generated clients,
deployment policy, evidence, and platform tooling. A monorepo makes those
standards enforceable while service ownership remains independent. If the
organization later splits repositories, the same path envelopes become repo
boundaries.

Repository principles:

- service code is owned by service teams;
- contracts are stable boundaries and can be consumed without importing service
  implementation;
- shared libraries are small, domain-neutral, and reviewed as platform
  primitives;
- generated clients live outside service source trees;
- deployments are declarative and environment overlays are separated from
  service business logic;
- tests and evidence are first-class, not afterthoughts;
- no service imports another service's `src/` tree.

```text
cloud/
  README.md
  Makefile
  go.work | Cargo.toml | workspace.yaml

  .github/ | ci/
    workflows/
    policy/
    actions/

  tools/
    codegen/
    contract-lint/
    service-generator/
    launch-check/
    local-dev/

  libs/
    foundation/
      ids/
      errors/
      pagination/
      idempotency/
      auth-context/
      telemetry/
      retry-backoff/
      clock/
      test-fixtures/

  contracts/
    common/
      resource-identity.yaml
      pagination.yaml
      errors.yaml
      idempotency.yaml
      audit-event.yaml
      metering-event.yaml
    openapi/
    proto/
    events/
    policy/
    compatibility/

  generated/
    clients/
      rust/
      go/
      typescript/
      python/
    iac/
      opentofu-provider/
    docs/

  services/
    account/
    iam/
    billing/
    metering/
    audit/
    compute/
    network/
    storage/
    kubernetes/
    observability/

  control-plane/
    api-gateway/
    resource-registry/
    workflow-engine/
    quota-service/
    placement-service/

  data-plane/
    host-agent/
    network-agent/
    storage-agent/

  platform/
    service-templates/
    ci-cd/
    policy/
    observability-kit/
    security-kit/
    release-orchestrator/
    service-catalog/
    quality-harness/
    dr-harness/
    launch-gates/

  infra/
    regions/
    cells/
    zones/
    racks/
    network/
    fleet/
    environments/

  tests/
    unit/
    integration/
    contract/
    load/
    chaos/
    security/
    compliance/
    disaster-recovery/

  evidence/
    builds/
    sbom/
    provenance/
    launches/
    incidents/
    game-days/
    restore-drills/

  plans/
  runbooks/
  dashboards/
  docs/
    public/
    internal/
    api/
    architecture/
    operations/
```

#### 10.1.1 Service Directory Shape

Each microservice has the same clean-architecture skeleton. The service may be
implemented in Rust, Go, or another approved language, but the layer names and
dependency direction stay the same.

```text
cloud/services/<service-name>/
  service.yaml
  ownership.yaml
  README.md

  contracts/
    openapi/
      <service>-v1.yaml
    proto/
      <service>.proto
    events/
      <service>-events.yaml
    policy/
      iam-actions.yaml
      resource-policy.yaml
    compatibility/
      breaking-change-tests.yaml

  src/
    domain/
      entities/
      value_objects/
      aggregates/
      state_machines/
      invariants/
    ports/
      repositories/
      event_publishers/
      policy/
      metering/
      audit/
      clocks/
      idempotency/
    application/
      commands/
      queries/
      sagas/
      workflows/
      transaction_boundaries/
    adapters/
      postgres/
      object_store/
      event_stream/
      kms/
      iam/
      telemetry/
      external/
    api/
      http/
      grpc/
      middleware/
      mappers/
    worker/
      consumers/
      schedulers/
      repair_loops/
      replay/
    runtime/
      main/
      config/
      dependency_wiring/
      shutdown/

  migrations/
    forward/
    rollback/

  deploy/
    base/
      deployment.yaml
      service.yaml
      serviceaccount.yaml
      networkpolicy.yaml
      pdb.yaml
      hpa.yaml
    overlays/
      dev/
      stage/
      prod/
    cell/
      cell-a/
      cell-b/
    region/
      dev-1/
      prod-1/

  tests/
    unit/
    contract/
    integration/
    fuzz/
    load/
    chaos/
    security/
    compliance/

  runbooks/
    deploy.md
    rollback.md
    incident.md
    restore.md

  dashboards/
    service.json
    slo.json
    dependency.json

  evidence/
    launch/
    performance/
    reliability/
    security/
    compliance/
```

#### 10.1.2 Data-Plane Service Shape

Data-plane components have the same clean architecture, but add host/device
reconciliation and local failure behavior.

```text
cloud/data-plane/<agent-name>/
  agent.yaml
  contracts/
    command-api/
    telemetry/
    host-events/
  src/
    domain/
    ports/
    application/
      reconciliation/
      repair/
      health/
    adapters/
      kernel/
      hypervisor/
      network-device/
      local-storage/
      metadata-cache/
    api/
      local-grpc/
      health/
    worker/
      reconcile-loop/
      command-consumer/
    runtime/
  deploy/
    daemonset/
    host-systemd/
  tests/
    unit/
    integration/
    host-simulation/
    chaos/
```

Data-plane rule: the local agent must be able to keep existing workloads safe
when regional or global control-plane components are impaired.

#### 10.1.3 Control-Plane Component Shape

Control-plane components that are not customer-facing product services still
follow service discipline.

```text
cloud/control-plane/<component-name>/
  component.yaml
  contracts/
  src/
    domain/
    ports/
    application/
    adapters/
    api/
    worker/
    runtime/
  tests/
  deploy/
  runbooks/
  dashboards/
```

Examples:

- `resource-registry` owns canonical regional resource lifecycle state.
- `placement-service` owns host/cell/zone placement decisions.
- `quota-service` owns account/region/service quota decisions.
- `workflow-engine` owns saga orchestration and replay for control-plane
  operations.

#### 10.1.4 Shared Library Policy

Allowed shared libraries live under `libs/foundation/` and are
domain-neutral:

- typed IDs;
- error envelopes;
- pagination;
- auth context;
- idempotency key primitives;
- telemetry wrappers;
- retry/backoff utilities;
- clocks and test clocks;
- contract-test fixtures.

Forbidden shared libraries:

- service-specific domain models;
- cross-service repositories;
- shared database access;
- business rules;
- generated mega-clients that hide dependencies;
- framework abstractions that force all services to deploy together.

Dependency rule:

```text
services/<a>/src/** may import:
  libs/foundation/**
  generated/clients/<language>/<service-b>/**
  contracts/common/**

services/<a>/src/** must not import:
  services/<b>/src/**
  services/<b>/migrations/**
  services/<b>/adapters/**
```

#### 10.1.5 Contract And Generated Client Layout

Contracts are source of truth for cross-service and customer-facing interfaces.
Generated clients are outputs, not hand-maintained integration code.

```text
contracts/
  common/
  openapi/<service>/<version>.yaml
  proto/<service>/<version>.proto
  events/<service>/<version>.yaml
  policy/<service>/iam-actions.yaml
  compatibility/<service>/

generated/
  clients/<language>/<service>/<version>/
  iac/opentofu-provider/<service>/
  docs/api/<service>/<version>/
```

Rules:

- public API changes start in `contracts/`;
- contract tests fail on breaking changes unless versioned;
- generated clients must not be edited manually;
- service implementations conform to contracts, not the reverse;
- SDK/IaC changes are parallelizable after contract freeze.

#### 10.1.6 Deployment And Environment Layout

Deployment configuration is separated from service source.

```text
infra/
  regions/<region>/
    region.yaml
    zones/
    cells/
    network/
    capacity/
  environments/
    dev/
    stage/
    prod/

services/<service>/deploy/
  base/
  overlays/dev/
  overlays/stage/
  overlays/prod/
  region/<region>/
  cell/<cell>/
```

Rules:

- `base/` contains service-owned defaults;
- environment overlays contain environment-specific resource sizing and
  deployment policy;
- region/cell overlays contain topology placement and rollout constraints;
- secrets are referenced, never stored;
- rollout, rollback, and progressive delivery settings are versioned.

#### 10.1.7 Testing And Evidence Layout

Tests live with the service when they verify service behavior. Cross-service
journeys live under top-level `tests/`.

```text
services/<service>/tests/
  unit/
  contract/
  integration/
  fuzz/
  load/
  chaos/
  security/
  compliance/

tests/
  journeys/
    account-vpc-vm/
    account-bucket-object/
    cluster-create-deploy-delete/
  regional/
    cell-failure/
    zone-impairment/
    control-plane-outage/
  platform/
    launch-gate/
    provenance/
    restore-drill/

evidence/
  launches/<service>/<date>/
  game-days/<scenario>/<date>/
  restore-drills/<service>/<date>/
  incidents/<incident-id>/
```

Rules:

- service tests cannot require another service's private database;
- integration tests use public APIs or generated clients;
- every launch gate emits evidence;
- evidence is append-only after promotion;
- failed evidence is kept and linked to follow-up work.

#### 10.1.8 Mandatory `service.yaml`

Every service owns a machine-readable service descriptor.

```yaml
service: compute
owner: team-compute
lifecycle: internal-preview
product_family: core-iaas
planes:
  - regional-control
dependencies:
  hard:
    - iam
    - quota
    - placement
  async:
    - audit
    - metering
data_stores:
  - name: compute-regional-metadata
    owner: compute
    partition_key: region_id/cell_id/account_id
apis:
  openapi:
    - contracts/openapi/compute/v1.yaml
slo:
  availability: 99.95
  latency_p99_ms: 200
audit:
  required_for:
    - CreateInstance
    - TerminateInstance
metering:
  dimensions:
    - instance_hours
    - volume_attachment_hours
deployment:
  rollout: cell-canary
  rollback: required
```

Validation rules:

- production lifecycle requires owner, SLO, runbook, dashboard, audit, metering,
  rollback, and support route;
- hard dependencies must be declared and tested;
- data stores must declare owner and partition key;
- every service declares which plane it runs in;
- services with customer-visible mutations declare audit coverage.

#### 10.1.9 Clean Architecture Import Enforcement

Enforce import direction in CI:

```text
domain -> no service/framework/database/network imports
ports -> domain only
application -> domain + ports + foundation
adapters -> ports + vendor clients
api -> application + contracts + auth context
worker -> application + event contracts
runtime -> all layers for wiring only
```

Fail the build if:

- `domain` imports `adapters`, `api`, `worker`, `runtime`, database, HTTP, or
  Kubernetes clients;
- `application` imports concrete vendor clients directly;
- `services/<a>` imports `services/<b>/src`;
- public APIs are implemented without contract tests;
- customer-visible mutations lack idempotency/audit/metering checks;
- list endpoints lack pagination;
- deployment manifests lack SLO labels, owner labels, and rollback settings.

### 10.2 Commands

The greenfield repo should standardize these commands from day one:

```bash
make fmt
make lint
make test
make test-contract
make test-integration
make test-load
make test-security
make test-overload
make test-isolation
make test-canary
make test-privacy
make test-cost-data
make test-k8s-policy
make build
make sbom
make provenance
make sign
make verify
make deploy ENV=dev REGION=dev-1 CELL=cell-a
make rollback ENV=dev REGION=dev-1 CELL=cell-a RELEASE=<release-id>
make launch-check SERVICE=<service-name> REGION=<region> CELL=<cell>
```

### 10.3 Code Style Example

The exact language may vary, but resource APIs should be typed, idempotent,
tenant/account scoped, and auditable.

```rust
pub struct CreateVolumeRequest {
    pub account_id: AccountId,
    pub region: RegionId,
    pub zone: ZoneId,
    pub idempotency_key: IdempotencyKey,
    pub size_gib: u64,
    pub encrypted: bool,
    pub tags: Tags,
}

pub trait VolumeControlPlane {
    fn create_volume(
        &self,
        request: CreateVolumeRequest,
        actor: AuthenticatedActor,
    ) -> Result<VolumeId, ControlPlaneError>;
}
```

Style rules:

- Prefer explicit resource identities over raw strings.
- Every mutation takes an idempotency key.
- Every mutation records actor, account, region, request ID, and audit event.
- Every public list endpoint is paginated.
- Every error is typed and maps to a stable public error code.

---

## 11. Service Architecture Pattern

Every cloud service must implement the same product contract:

| Requirement | Description |
|---|---|
| API | Stable public API with versioning, pagination, idempotency, rate limits, and error model |
| Resource model | IDs, ARNs/URIs, tags, lifecycle state, ownership, region, zone/cell placement |
| Control plane | CRUD, validation, workflow, policy, quota, placement, audit, metering |
| Data plane | High-performance operation path that survives most control-plane failures |
| Security | IAM actions, resource policies, encryption, tenant isolation, threat model |
| Reliability | SLO, redundancy, backups, restore, failover, chaos tests |
| Observability | RED/USE metrics, traces, logs, audit events, customer-visible health |
| Billing | Meter names, dimensions, aggregation, pricing hooks, budget alerts |
| Operations | Runbook, dashboards, alerts, paging, incident playbooks, on-call |
| Lifecycle | Preview, GA, deprecation, migration, compatibility guarantees |

---

## 12. Roadmap And Task Breakdown

### Phase 0: Company And Operating System

#### Task 0.1: Establish cloud product charter

**Description:** Define target customers, first region, first service families,
public promise, support model, and business constraints.

**Acceptance criteria:**
- Charter names first customer segment and first three cloud services.
- Charter defines first launch stage: internal, private preview, public preview,
  GA.
- Charter names explicit non-goals for the first release.

**Verification:**
- Executive cloud council approves charter.
- Product council confirms roadmap alignment.

**Dependencies:** None
**Estimated scope:** Small

#### Task 0.2: Define service ownership and launch governance

**Description:** Create the ownership model, launch gates, incident governance,
architecture review, and security review paths.

**Acceptance criteria:**
- Every service has a single accountable owner.
- Launch gates are defined for concept, architecture, build, preview, GA,
  expansion, and deprecation.
- Incident commander, communications lead, and action-item owner roles are
  defined.

**Verification:**
- Dry-run one mock service through the gates.

**Dependencies:** Task 0.1
**Estimated scope:** Medium

### Phase 1: Platform Engineering Foundation

#### Task 1.1: Create the golden service template

**Description:** Build the initial service template for a cloud control-plane
service with API contract, auth middleware, metrics, logging, tracing, health
checks, tests, and deployment metadata.

**Acceptance criteria:**
- A new service can be generated from the template.
- Generated service exposes health, readiness, metrics, and one example
  idempotent endpoint.
- Generated service includes default CI checks.

**Verification:**
- Generate a sample service.
- Run `make verify` on the generated service.

**Dependencies:** Task 0.2
**Estimated scope:** Medium

#### Task 1.2: Create service catalog and ownership registry

**Description:** Create the internal catalog that records service owner, APIs,
SLOs, dependencies, data classification, runbook, dashboard, and lifecycle
state.

**Acceptance criteria:**
- Catalog can register a service and query by owner, dependency, or lifecycle
  state.
- Catalog rejects production state without owner, SLO, and runbook.

**Verification:**
- Register the generated sample service.
- Attempt invalid registration and verify rejection.

**Dependencies:** Task 1.1
**Estimated scope:** Medium

#### Task 1.3: Create CI/CD and provenance lane

**Description:** Establish build, unit test, lint, contract test, vulnerability
scan, SBOM, provenance, signing, and deploy workflow.

**Acceptance criteria:**
- Every build produces an artifact, SBOM, and provenance record.
- Unsigned artifacts cannot deploy.
- Failed tests block deployment.

**Verification:**
- Run `make build`, `make sbom`, `make provenance`, `make sign`, `make verify`.
- Attempt deployment with unsigned artifact and verify denial.

**Dependencies:** Task 1.1
**Estimated scope:** Medium

### Checkpoint A: Platform Foundation

- [ ] Service template exists.
- [ ] Service catalog exists.
- [ ] CI/CD, SBOM, provenance, and signing exist.
- [ ] One sample service passes all checks.
- [ ] Architecture and security councils approve platform foundation.

### Phase 2: Region, Cell, And Infrastructure Bootstrap

#### Task 2.1: Define region/zone/cell metadata model

**Description:** Define the canonical model for regions, availability zones,
cells, racks, hosts, networks, capacity pools, and failure domains.

**Acceptance criteria:**
- Metadata model represents global, region, zone, cell, rack, host, and service
  shard.
- Metadata model can answer "where can this resource be placed?"
- Metadata model captures capacity and failure boundary.

**Verification:**
- Create sample region with three zones and three cells per zone.
- Run placement simulation for compute and storage.

**Dependencies:** Checkpoint A
**Estimated scope:** Medium

#### Task 2.2: Build bootstrap region control plane

**Description:** Build minimal regional control plane for metadata, health,
capacity, placement, audit buffer, and deployment reconciliation.

**Acceptance criteria:**
- Region control plane starts in a dev region.
- It can register cells and report health.
- It can continue serving read-only metadata if global control plane is down.

**Verification:**
- Kill simulated global dependency and verify regional metadata read path.

**Dependencies:** Task 2.1
**Estimated scope:** Medium

#### Task 2.3: Build cell lifecycle automation

**Description:** Automate creation, upgrade, cordon, drain, repair, and
retirement of cells.

**Acceptance criteria:**
- New cell can be created from declarative desired state.
- Cell can be marked unhealthy without affecting other cells.
- Cell can be drained and retired.

**Verification:**
- Create dev cell, deploy sample service, drain cell, verify relocation.

**Dependencies:** Task 2.2
**Estimated scope:** Medium

### Checkpoint B: Region And Cell Bootstrap

- [ ] Dev region exists.
- [ ] At least two cells exist.
- [ ] Cell health, drain, and repair workflows are tested.
- [ ] Regional control plane can degrade independently from global control plane.

### Phase 3: Global Account, IAM, Billing, Audit, And Metering

#### Task 3.1: Build account and organization service

**Description:** Implement account creation, organization hierarchy, account
state, tags, quotas, and region opt-in.

**Acceptance criteria:**
- Account can be created and assigned to organization.
- Account can enable a region.
- Account has default quotas.

**Verification:**
- Contract test for create/list/update account.
- Quota enforcement test for denied over-quota request.

**Dependencies:** Checkpoint B
**Estimated scope:** Medium

#### Task 3.2: Build IAM and STS baseline

**Description:** Implement users, roles, policies, temporary credentials,
federation hooks, and service identity.

**Acceptance criteria:**
- Actor can assume a role with scoped temporary credentials.
- Policy denies unauthorized resource creation.
- Service identity is available for internal calls.

**Verification:**
- Positive and negative authorization tests.
- Token expiry and rotation test.

**Dependencies:** Task 3.1
**Estimated scope:** Medium

#### Task 3.3: Build audit and metering pipeline

**Description:** Implement append-only audit events, metering events, regional
buffers, global aggregation, and customer export path.

**Acceptance criteria:**
- Every control-plane mutation emits audit and metering events.
- Regional buffer survives temporary global aggregation outage.
- Customer can query account-level audit events.

**Verification:**
- Simulate global aggregator outage; verify regional buffering and later replay.

**Dependencies:** Task 3.2
**Estimated scope:** Medium

#### Task 3.4: Build billing baseline for internal preview

**Description:** Implement meter catalog, price dimensions, invoice preview,
budget alerts, and cost allocation tags.

**Acceptance criteria:**
- Metered resources produce billable usage records.
- Invoice preview can aggregate by account, service, region, tag.
- Budget alert fires on threshold.

**Verification:**
- Generate usage for sample service and verify invoice preview.

**Dependencies:** Task 3.3
**Estimated scope:** Medium

### Checkpoint C: Trust And Commerce Foundation

- [ ] Accounts, IAM, audit, metering, and billing preview work together.
- [ ] Unauthorized access is denied.
- [ ] Metering events survive aggregator disruption.
- [ ] Every control-plane mutation is traceable.

### Phase 4: Compute Service

#### Task 4.1: Define compute resource contract

**Description:** Define API and resource lifecycle for VM instances, images,
instance types, placement, volumes, network interfaces, snapshots, and host
maintenance.

**Acceptance criteria:**
- API supports create, describe, start, stop, reboot, terminate.
- API includes idempotency keys, pagination, stable error codes, and tags.
- Resource lifecycle has explicit states and transitions.

**Verification:**
- Contract tests pass for create/describe/terminate.

**Dependencies:** Checkpoint C
**Estimated scope:** Medium

#### Task 4.2: Build host agent and placement service

**Description:** Build host inventory, capacity accounting, placement rules,
health reporting, and host agent communication.

**Acceptance criteria:**
- Placement service chooses host based on zone, capacity, constraints, and
  anti-affinity.
- Host agent reports health and capacity.
- Unhealthy host is removed from placement.

**Verification:**
- Simulate host failure and verify placement excludes host.

**Dependencies:** Task 4.1
**Estimated scope:** Medium

#### Task 4.3: Launch first VM in dev cell

**Description:** Implement minimum viable VM launch path with image selection,
network attach, boot, metadata, audit, and metering.

**Acceptance criteria:**
- User can create a VM and SSH or console into it in dev cell.
- VM has network identity and metadata.
- VM usage is metered.

**Verification:**
- End-to-end launch, connect, stop, start, terminate test.

**Dependencies:** Task 4.2
**Estimated scope:** Medium

### Phase 5: Networking Service

#### Task 5.1: Build VPC and subnet control plane

**Description:** Implement VPC, subnet, route table, security group, IPAM, and
network interface APIs.

**Acceptance criteria:**
- Account can create isolated VPC and subnet.
- VM can attach to network interface.
- Security group denies inbound traffic by default.

**Verification:**
- VM-to-VM connectivity test within VPC.
- Denied inbound test from outside VPC.

**Dependencies:** Task 4.3
**Estimated scope:** Medium

#### Task 5.2: Build load balancer and DNS preview

**Description:** Implement L4 load balancer, health checks, target groups,
regional DNS records, and basic certificate integration.

**Acceptance criteria:**
- User can expose service behind load balancer.
- Failed target is removed from rotation.
- DNS record points to load balancer.

**Verification:**
- Kill target and verify load balancer routes to healthy targets.

**Dependencies:** Task 5.1
**Estimated scope:** Medium

### Phase 6: Storage Service

#### Task 6.1: Build object storage contract and metadata plane

**Description:** Define bucket/object APIs, metadata, authz, versioning,
encryption, replication class, lifecycle, and audit.

**Acceptance criteria:**
- Account can create bucket, put object, get object, list objects, delete
  object.
- Bucket policy denies unauthorized access.
- Object operations emit audit and metering.

**Verification:**
- Contract tests for bucket/object lifecycle and denied access.

**Dependencies:** Checkpoint C
**Estimated scope:** Medium

#### Task 6.2: Build block storage volume lifecycle

**Description:** Implement volume create/attach/detach/snapshot/delete,
encryption, zone placement, and failure handling.

**Acceptance criteria:**
- VM can attach encrypted volume.
- Snapshot can restore into new volume.
- Detached volume persists across VM termination.

**Verification:**
- End-to-end attach/write/snapshot/restore test.

**Dependencies:** Task 4.3
**Estimated scope:** Medium

### Phase 7: Managed Kubernetes And Containers

#### Task 7.1: Build managed cluster contract

**Description:** Define cluster, node pool, version, add-on, upgrade, IAM,
networking, logging, and billing model.

**Acceptance criteria:**
- Account can create managed cluster in a VPC.
- Cluster has node pool, network policy, logging, and monitoring.
- Cluster lifecycle is represented in API states.

**Verification:**
- Contract tests for cluster create/describe/delete.

**Dependencies:** Tasks 4.3, 5.1, 6.2
**Estimated scope:** Medium

#### Task 7.2: Launch first managed cluster

**Description:** Implement cluster creation, node provisioning, API endpoint,
certificate management, add-ons, and upgrade path.

**Acceptance criteria:**
- User can create cluster and deploy a sample workload.
- Cluster emits metrics and logs.
- Cluster can upgrade one patch version with rollback plan.

**Verification:**
- End-to-end create/deploy/scale/upgrade/delete test.

**Dependencies:** Task 7.1
**Estimated scope:** Medium

### Phase 8: Observability, Security, Compliance, And Operations

#### Task 8.1: Build central telemetry and SLO system

**Description:** Implement telemetry ingestion, dashboards, alerts, SLOs, error
budgets, customer health views, and service owner views.

**Acceptance criteria:**
- Every production service has metrics, logs, traces, SLO, and alert route.
- SLO burn-rate alert can page the owner.
- Customer-visible health can be scoped by region/service.

**Verification:**
- Simulate SLO burn and verify alert route.

**Dependencies:** Checkpoint C
**Estimated scope:** Medium

#### Task 8.2: Build vulnerability and patch program

**Description:** Implement asset inventory, dependency inventory, vulnerability
scanning, severity SLAs, emergency patch workflow, and exception process.

**Acceptance criteria:**
- Vulnerability can be traced to owning service and artifact.
- Critical vulnerability has emergency patch path.
- Exception requires expiry and risk owner.

**Verification:**
- Run tabletop for critical vulnerability response.

**Dependencies:** Task 1.3
**Estimated scope:** Medium

#### Task 8.3: Build backup, restore, and disaster recovery program

**Description:** Implement backup policies, restore drills, DR pairings,
recovery objectives, and evidence storage.

**Acceptance criteria:**
- Critical metadata stores have backup and restore tests.
- Restore drill records RTO/RPO outcome.
- DR decision tree is available to incident commander.

**Verification:**
- Restore a regional metadata store into a test cell.

**Dependencies:** Tasks 2.3, 3.3
**Estimated scope:** Medium

### Phase 9: Internal Preview

#### Task 9.1: Dogfood internal workloads

**Description:** Move selected internal workloads onto the new cloud platform to
exercise real use cases before external customers.

**Acceptance criteria:**
- At least three internal workloads run on compute/network/storage foundation.
- Workloads have SLOs and cost allocation.
- Incidents and friction are tracked as product feedback.

**Verification:**
- Run one planned zone/cell impairment and verify workload behavior.

**Dependencies:** Phases 4, 5, 6, 8
**Estimated scope:** Medium

#### Task 9.2: Launch private customer preview

**Description:** Onboard a small number of design partners with strict
guardrails, support, quotas, and feedback loops.

**Acceptance criteria:**
- Preview tenants can create accounts, VPCs, VMs, volumes, buckets, and managed
  clusters.
- Support and incident process is staffed.
- Usage and cost are visible.

**Verification:**
- Customer onboarding rehearsal passes.
- Launch review approves private preview.

**Dependencies:** Task 9.1
**Estimated scope:** Medium

### Phase 10: Public Preview And GA

#### Task 10.1: Public preview readiness

**Description:** Expand docs, SDKs, console, quotas, billing, support, security
disclosures, status page, and customer onboarding.

**Acceptance criteria:**
- New customer can self-serve onboarding.
- Core services have published limits and preview SLOs.
- Status page and support process are live.

**Verification:**
- External beta user completes onboarding without internal admin actions.

**Dependencies:** Task 9.2
**Estimated scope:** Medium

#### Task 10.2: GA readiness

**Description:** Complete GA gates for reliability history, billing accuracy,
support readiness, compliance evidence, incident readiness, scale tests, and
customer migration commitments.

**Acceptance criteria:**
- Core services meet GA SLO history threshold.
- Billing accuracy is validated.
- Incident, backup, restore, support, and security response drills pass.

**Verification:**
- Formal GA launch review approves release.

**Dependencies:** Task 10.1
**Estimated scope:** Medium

---

## 13. Hyperscaler Service Portfolio Sequencing

| Wave | Service family | Why now |
|---|---|---|
| 1 | Account, IAM, audit, metering, billing | Trust and commerce foundation |
| 2 | Compute, VPC networking, block storage | Core IaaS |
| 3 | Object storage, load balancing, DNS | Usable application substrate |
| 4 | Managed Kubernetes, registry, container networking | Modern workload adoption |
| 5 | Managed relational database, cache, event streaming | Stateful application adoption |
| 6 | Observability, security center, key management, secrets | Enterprise trust |
| 7 | Serverless functions, workflow, eventing | Higher-level developer velocity |
| 8 | Data lake, warehouse, search, analytics | Data platform revenue |
| 9 | GPU, AI training/inference, model serving | AI infrastructure market |
| 10 | Marketplace, partner network, private offers | Ecosystem growth |

### 13.1 Multi-Year Strategic Roadmap

This roadmap is intentionally staged. Trying to launch every hyperscaler-class
service at once creates false breadth and operational fragility.

| Horizon | Target state | Product proof | Operating proof |
|---|---|---|---|
| H0: Lab | Service factory, contracts, sample service, dev region/cell | Sample service generated and deployed | CI, signing, catalog, launch gates work |
| H1: Internal cloud | Account/IAM, audit, metering, compute, VPC, object/block storage in one dev region | Internal teams run workloads | SLOs, incident process, rollback, restore drills |
| H2: Private preview | Design partners use core IaaS with quotas and support | Customer can run basic app | Support, status, billing preview, customer audit logs |
| H3: Public preview | Self-service onboarding and published limits | Broader customers onboard | Abuse controls, docs, SDK/IaC, preview SLOs |
| H4: Regional GA | One region meets GA gates for core IaaS and managed Kubernetes | Production workloads run | SLO history, billing accuracy, compliance evidence |
| H5: Multi-region | Repeatable region factory launches second and third regions | Customers can choose regions | Region build playbook, regional DR, capacity forecasting |
| H6: Portfolio expansion | Databases, event streaming, serverless, data, AI services | Customers build full platforms | Service factory launches new families predictably |
| H7: Ecosystem | Marketplace, private offers, partner network, enterprise support | Partners sell through platform | Commercial ops, trust portal, support tiers |
| H8: Global hyperscaler posture | Multiple regions, backbone, edge, compliance packs, mature portfolio | Global/regional enterprise customers | Fleet economics, backbone ops, continuous compliance |

### 13.2 Region Launch Roadmap

Every region launch is a product launch and infrastructure launch.

| Step | Output |
|---|---|
| Region business case | customer demand, regulatory need, capacity forecast, capital model |
| Region design | zones, cells, network, power/cooling, compliance, service scope |
| Region bootstrap | identity, audit, metering, observability, deployment, support |
| Core IaaS enablement | compute, VPC, block, object, load balancer, DNS |
| Runtime enablement | managed Kubernetes, registry, secrets, KMS |
| Reliability drills | zone impairment, cell failure, bad deploy, restore, network partition |
| Commercial readiness | pricing, tax, invoice, support, status, customer docs |
| Launch approval | evidence-gated decision by product, architecture, security, SRE, support, finance |

### 13.3 Service Family Launch Roadmap

Every new service family must pass the same sequence:

```text
Brief -> PRD -> API contract -> threat model -> cost model -> SLO model
  -> internal implementation -> dogfood -> private preview -> public preview
  -> GA -> multi-region expansion -> lifecycle/deprecation management
```

No service family may skip account/IAM, audit, metering, quota, observability,
support, billing, and launch-gate integration.

---

## 14. Testing Strategy

| Test type | Purpose | Minimum requirement |
|---|---|---|
| Unit | Verify local behavior | Required for domain logic and policy decisions |
| Contract | Prevent API drift | Required for every public API and SDK |
| Integration | Verify service dependencies | Required for control-plane workflows |
| End-to-end | Verify user journeys | Required for launch gates |
| Fuzz | Find parser/authz/input bugs | Required for public APIs and policy engines |
| Load | Verify throughput and latency | Required before preview and GA |
| Chaos | Verify failure behavior | Required for critical services |
| Security | Verify threat controls | Required before production |
| Compliance evidence | Verify controls are measurable | Required for regulated customers |
| Disaster recovery | Verify backup and restore | Required for critical state |

### 14.1 Hyperscaler-Scale Validation Matrix

| Validation class | Question answered | Required before |
|---|---|---|
| Cell scale | Can a cell handle its declared tenant/resource envelope? | Private preview |
| Region scale | Can a region handle expected launch traffic with headroom? | Public preview |
| Multi-zone impairment | Can service continue through one-zone impairment? | GA for multi-zone services |
| Control-plane outage | Does data plane continue serving existing workloads? | Public preview |
| Backlog recovery | Can event/worker systems recover without data loss after outage? | Public preview |
| Hot partition | Can shard split or mitigation handle one large customer/resource? | GA |
| Noisy neighbor | Can one account/resource class be throttled without harming others? | Private preview |
| Billing reconciliation | Does metered usage match invoice preview within tolerance? | Public preview |
| Customer audit export | Can customer reconstruct resource-changing activity? | Private preview |
| Region expansion rehearsal | Can a second region be created from the region factory? | Multi-region launch |

---

## 15. Boundaries

### Always Do

- Define owner, API, SLO, runbook, dashboard, quota, audit, metering, and cost
  model for every production service.
- Use idempotency keys on mutations.
- Use pagination on lists.
- Deny by default for network and IAM.
- Emit audit and metering events for customer-visible mutations.
- Test failure modes before launch.
- Deploy progressively by cell and region.
- Maintain rollback paths.
- Review security and reliability before public preview.

### Ask First

- Adding a new foundational dependency.
- Creating a new public API style.
- Bypassing launch gates.
- Accepting customer data without defined data classification and retention.
- Launching a service without SLO history.
- Running single-zone or single-cell for customer-critical workloads.
- Sharing control-plane dependencies across failure boundaries.

### Never Do

- Ship a public service without an owner.
- Ship a public mutation without idempotency.
- Ship a production service without audit, metrics, logs, alerts, and runbook.
- Store secrets in source, logs, telemetry, or tickets.
- Make global control plane a hard dependency for regional data-plane survival.
- Treat retries as a substitute for capacity, backpressure, and graceful
  degradation.
- Launch GA based on roadmap pressure rather than readiness evidence.

---

## 16. Launch Readiness Checklist

For every service:

- [ ] PRD approved.
- [ ] Architecture design approved.
- [ ] Threat model approved.
- [ ] API contract frozen for launch stage.
- [ ] IAM actions and resource policies defined.
- [ ] Quotas and rate limits implemented.
- [ ] Audit and metering implemented.
- [ ] SLOs, dashboards, alerts, and runbooks implemented.
- [ ] Backups and restore tests completed if stateful.
- [ ] Load test completed.
- [ ] Failure-mode test completed.
- [ ] Security tests completed.
- [ ] Billing accuracy validated.
- [ ] Customer support playbook ready.
- [ ] Status page integration ready.
- [ ] Deprecation and compatibility policy published.
- [ ] Launch review approved.
- [ ] Performance, reliability, accountability, observability, scalability,
      availability, and optimization evidence is attached.

---

## 17. Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Trying to match hyperscaler breadth too early | High | Start with core IaaS, prove operating model, then expand |
| Global control plane becomes single point of failure | Critical | Region-local operation, static stability, cell isolation |
| Customer isolation failure | Critical | Zero trust, strong tenancy model, authz tests, fuzzing, audit |
| Billing inaccuracies | High | Meter catalog tests, reconciliation, shadow billing before charging |
| Platform team becomes ticket queue | High | Self-service golden paths, catalog, automation, SLAs |
| Launch gates become theater | High | Evidence-backed gates with automated checks |
| Teams optimize for feature throughput over reliability | High | SLO/error budgets and launch readiness gates |
| Supply-chain compromise | Critical | SBOM, provenance, signing, restricted deploys |
| Network design cannot scale | Critical | Early fabric modeling, failure drills, expert review |
| Operational toil overwhelms teams | High | Toil budget, automation roadmap, operational reviews |
| Compliance added too late | High | Evidence automation from day one |
| Over-standardization blocks innovation | Medium | Paved road plus exception process with expiry |

---

## 18. Parallelization Strategy

Safe to parallelize after contracts are stable:

- Console, CLI, SDK, and IaC provider for a defined API.
- Service docs/examples and service implementation.
- Load-test harness and service implementation.
- Security tests and feature implementation after threat model is approved.
- Observability dashboards and service implementation after telemetry schema is
  defined.

Must be sequential:

- Account/IAM before customer resource creation.
- Metering before billing.
- Region/cell metadata before placement automation.
- Compute host agent before VM launch.
- VPC network interface before VM network attach.
- Storage volume attach before stateful VM workloads.
- SLO definition before production launch.

Needs explicit coordination:

- Shared resource model.
- IAM action naming.
- Public error model.
- Billing meter dimensions.
- Tagging and cost allocation.
- Service catalog schema.
- Global/regional control-plane boundaries.

---

## 19. Code Review And Quality Gate

Every implementation change should be reviewed across five axes:

| Axis | Required review question |
|---|---|
| Correctness | Does the change satisfy the spec and edge cases? |
| Readability | Can another engineer understand and maintain it? |
| Architecture | Does it preserve service boundaries and failure boundaries? |
| Security | Does it validate input, enforce authz, protect secrets, and treat external data as untrusted? |
| Performance | Does it avoid unbounded work, excessive fanout, and hot-path regressions? |

Blocking findings:

- Missing authz on customer-visible API.
- Missing idempotency on mutation.
- Missing pagination on list endpoint.
- Missing audit/metering on customer-visible mutation.
- Secrets in code, logs, telemetry, or test fixtures.
- Hidden cross-region or global dependency in a regional data path.
- Unbounded retries without backoff and jitter.
- Production deploy path without rollback.

---

## 20. Management Metrics

| Level | Metrics |
|---|---|
| Executive | revenue readiness, region readiness, critical risks, capital burn, customer commitments |
| Product | adoption, activation, retention, NPS/customer feedback, roadmap confidence |
| Program | milestone health, dependency risk, launch gate pass/fail, staffing gaps |
| Engineering | DORA metrics, defect escape rate, review latency, test reliability |
| Reliability | SLO attainment, incident count, MTTR, error-budget burn, repeat incidents |
| Security | vulnerability SLA, control coverage, audit findings, incident response time |
| Platform | template adoption, self-service success, toil reduction, developer satisfaction |
| FinOps | unit cost, idle capacity, forecast accuracy, gross margin by service |

### 20.1 Executive Hyperscaler Scorecard

Executives should review these as one scorecard, not as isolated engineering
metrics:

| Dimension | Leading indicators | Lagging indicators |
|---|---|---|
| Product | API coverage, docs/examples readiness, preview activation | retained workloads, service adoption, customer expansion |
| Reliability | SLO coverage, game-day pass rate, error-budget burn | incidents, SLA credits, churn from reliability |
| Security/trust | threat-model coverage, vuln SLA compliance, audit coverage | audit findings, customer security blockers |
| Scale | cell-addition time, capacity headroom, quota effectiveness | saturation events, expansion lead time |
| Delivery | DORA metrics, launch gate throughput, review latency | missed launches, escaped defects |
| Operations | toil %, alert quality, incident action closure | MTTR, repeat incidents, support escalations |
| Economics | unit cost, utilization, idle capacity, committed capacity risk | gross margin, capex efficiency, pricing misses |
| Ecosystem | SDK/IaC usage, marketplace readiness, partner onboarding | marketplace revenue, partner retention |

### 20.2 Minimum Honest-Claim Metrics

Before using "hyperscaler-grade" externally for any service or region, require:

- 100% service ownership coverage;
- 100% production service SLO coverage;
- 100% customer-visible mutation audit coverage;
- 100% customer-visible mutation metering coverage where billable;
- signed/provenance-attested artifacts for production;
- tested rollback for every production deployment path;
- tested restore for critical state;
- successful cell failure drill;
- successful dependency impairment drill;
- published customer support and status process;
- documented shared-responsibility model;
- no unowned critical security, reliability, or capacity risks.

---

## 21. Open Questions

1. What is the first target customer segment: startups, regulated enterprise,
   internal workloads, government, or AI infrastructure customers?
2. What is the first launch geography and regulatory posture?
3. Does the first region run on leased public cloud, colocation, or owned
   facilities?
4. Which primary implementation language should be standard for control-plane
   services?
5. What is the public API compatibility promise for preview and GA?
6. What SLOs are commercially committed versus internal-only for the first
   release?
7. Which services are built in-house immediately versus wrapped behind provider
   abstractions during bootstrap?
8. What compliance frameworks are mandatory for the first paid customers?
9. What customer support model exists before public preview?
10. What is the capital budget for hardware, network, and capacity buffers?

---

## 22. Final Readiness Definition

The hyperscaler program is ready for first public preview when:

- At least one region has multiple cells.
- Account, IAM, audit, metering, billing preview, compute, VPC, block storage,
  object storage, load balancing, DNS, and managed Kubernetes work end to end.
- Internal workloads have dogfooded the platform.
- Every public service has owner, API, SLO, quota, audit, metering, billing,
  dashboard, alert, runbook, and support playbook.
- Critical state has backup and restore evidence.
- Failure drills prove cell and zone containment.
- Security review and threat models are complete.
- Supply-chain signing and deployment verification are enforced.
- Preview customers can onboard without privileged internal manual steps.
- Product, architecture, security, SRE, support, and executive launch reviews
  approve the release using evidence, not aspiration.
- The seven hyperscaler-class evidence packs pass: performance, reliability,
  accountability, observability, scalability, availability, and optimization.
