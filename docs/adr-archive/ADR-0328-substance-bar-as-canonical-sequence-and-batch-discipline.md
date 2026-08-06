---
id: ADR-0328
title: Substance Bar as Canonical Sequence and Batch Discipline
status: Superseded
superseded_by: [ADR-700]
amended_by: [ADR-0329, ADR-0619]
date: 2026-05-20
owner_team: council-architecture
related_adrs: [ADR-0244, ADR-0263, ADR-0316, ADR-0321, ADR-0322, ADR-0323, ADR-0324, ADR-0327, ADR-0136-amendment, ADR-0247, ADR-0255-amendment, ADR-0138]
decision_owner: council-architecture
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 800
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: Proposed for immediate use in Wave 1 dispatch briefs
enforced_by:
  - oya-governance-substance-bar
  - oya-governance-no-template-stamping
  - oya-governance-brief-anchor-header
  - oya-governance-wave-batch-ceiling
  - oya-governance-microservice-coherence-audit
source_anchors:
  - /Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md
  - /Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  - /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md#1.1
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0327-wave-3-completion-criteria-and-promotion-gates.md
companion_docs:
  - /Users/jasonlee/oyatie/specs/master-plan-sequencing.json
  - /Users/jasonlee/oyatie/specs/markdown-retirement-policy.json
  - /Users/jasonlee/oyatie/docs/AGENTS.md
  - /Users/jasonlee/oyatie/docs/ADR-INDEX.md
purpose: >
  Codify the realignment sequence, batch discipline, anchor discipline,
  verification SLA, Foundry absorption model, and Codex-only dispatch convention
  that prevent Oyatie documentation and microservice ownership work from drifting
  away from the unified ecosystem thesis while still satisfying the substance bar.
---

# ADR-0328: Substance Bar as Canonical Sequence and Batch Discipline

## Status

Proposed on 2026-05-20.

This ADR is immediately usable as the Wave 1 control surface for briefs,
microservice ownership audits, ADR-0321 cleanup, and remediation batching.

Promotion to Accepted is governed by ADR-0327, and therefore requires the
substance-bar verdict, multispectrum review evidence, cross-reference density
evidence, and wave ledger evidence before downstream docs can cite it as final.

Until promotion, agents MAY cite this ADR as the proposed canonical sequence when
they are explicitly working inside the realignment wave described by the source
spec.

No implementation code is introduced by this decision.

No new microservice is introduced by this decision.

No markdown-retirement exception is introduced by this decision.

The sole durable product of this ADR is a normative ordering and verification
contract for subsequent authoring and audit waves.

## Context

### A.1 Named pressure: drift now creates the wrong product

The realignment spec states the core pressure plainly: Oyatie has a large and
substantive corpus, but continued authoring without a sequence backbone compounds
drift.

That drift is not cosmetic.

If product PRDs, microservice IPs, journey docs, vendor dossiers, and ADRs are
written from different implicit product models, downstream teams build different
products under one brand.

The unified ecosystem thesis rejects that outcome.

The platform is one identity, one tenancy model, one policy engine, one workflow
engine, one ontology, one audit chain, one marketplace settlement model, and one
UX shell vocabulary.

Product names such as CRM, HR, ERP, ITSM, mail, calendar, marketplace, workflow
studio, analytics, and governance are role and capability projections over the
shared substrate.

They are not permission to fork identity, policy, workflow, ontology, audit,
settlement, training, compliance, or extension semantics.

The build sequence therefore has to be canonical, not advisory.

Agents cannot choose whether to start with a vendor dossier, a microservice IP,
a remediation patch, or a runbook based on whatever file is nearby.

They must know which phase the artifact belongs to, which predecessor phase must
be substance-bar complete, and which anchors prove the artifact is aligned.

### A.2 Named pressure: line count alone already failed

ADR-0322 exists because prior authoring waves showed that structure and length
do not guarantee substance.

Template stamping, lambda-wrapped pseudo-content, and table-of-contents-only
artifacts passed some shape checks while failing the intern-buildability bar.

This ADR does not weaken ADR-0322.

It applies ADR-0322 to orchestration itself.

An audit wave that produces 316 files is not valuable if each file cites the
wrong anchors, uses the wrong sequence, reviews only line counts, or skips the
per-microservice ownership question.

An ADR-0321 cleanup is not complete if it removes duplicates but leaves the Big
8 priority order implicit.

A Foundry retirement is not complete if it deletes a directory but loses the
canonical `oyatie.foundry.*` principal namespace.

Sequence, anchor discipline, and batch discipline are therefore substance-bar
mechanics, not project-management decoration.

### A.3 Named pressure: ownership was fragmented

The realignment spec identifies a second failure: parallel agents touched the
same corpus areas without a single owner of coherence.

One microservice could be edited by separate agents for PRD, IP slices, runbook,
journey coverage, feature parity, capability tiers, and ADRs.

That creates locally plausible artifacts and globally inconsistent ownership.

The remedy is not to stop parallelism.

The remedy is to align parallelism with ownership.

One audit agent owns a microservice end-to-end for the audit wave.

That agent does not fix everything during the audit wave.

It reads the microservice path, checks cross-references, checks substance,
checks canonical direction, checks industry parity, and produces the four audit
deliverables.

Remediation then happens in later sub-waves from the aggregated findings.

### A.4 Named pressure: ADR-0321 scope was misunderstood

The realignment spec corrects a prior framing error.

The issue with ADR-0321 was not that cloud-infra, PaaS, developer tools, or
long-tail B2B SaaS were out of scope.

They are in scope.

The issue was priority, duplication, ordering, and substance.

The Big 8 enterprise displacement families must lead because they define the
largest enterprise product-pressure envelope.

Cloud-infra primitives, PaaS, developer tools, specialty products, and niche B2B
SaaS remain in scope but do not pre-empt HR, ERP, CRM, ITSM, and the remaining
Big 8 sequence.

### A.5 Canonical anchors this ADR binds

Anchor 1 is the realignment spec:
`/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md`.

That spec supplies the objective, the five-phase canonical sequence, the audit
wave structure, the top-3 counterpart parity bar, the agent-class anchor concept,
and the Codex-only dispatch constraint.

Anchor 2 is the unified ecosystem thesis:
`/Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md`.

That thesis supplies the product doctrine: one substrate, product labels as
navigation promises, role projections instead of adoption islands, and capability
tiers instead of fragmented product boundaries.

Anchor 3 is the documentation-rigor standard:
`/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md` Section 1.1.

That standard supplies the hyperscaler-grade rigor sub-test: named precedent,
failure-mode tree, capacity math, observability hooks, rollback path,
multi-region awareness, sovereign-cell awareness, and versioning plus
deprecation.

Anchor 4 is ADR-0322:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md`.

That ADR supplies the doctrine that substance is a blocker-class requirement,
not a reviewer preference.

Anchor 5 is ADR-0327:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0327-wave-3-completion-criteria-and-promotion-gates.md`.

That ADR supplies the promotion-gate model and the rule that proposed doctrine
does not become authoritative until gates are satisfied.

### A.6 Constraints inherited by this ADR

This ADR must remain compatible with ADR-0244 tenant scoping.

Every audit finding, brief, verification result, and remediation backlog item
must carry enough tenant and corpus context to be evaluated without guessing.

This ADR must remain compatible with ADR-0263 audit emission.

Every landing check described here is expected to emit or reference an audit
evidence class once the governing lanes exist.

This ADR must remain compatible with ADR-0316 capability tiers.

Vendor or product labels do not force new microservices when a capability-tier
overlay can hold the product surface on the existing substrate.

This ADR must remain compatible with ADR-0321.

The B2B SaaS industry-leader universe stays broad, while the Big 8 ordering
becomes mandatory for sequencing.

This ADR must remain compatible with ADR-0323 and ADR-0324.

Wave sequencing and anti-script authoring doctrine remain active; no batch rule
in this ADR permits scripted content generation or template substitution.

This ADR must remain compatible with ADR-0247 and ADR-0255-amendment.

Foundry capability survives as a substrate capability even as
`microservices/foundry/` is retired as a standalone service path.

This ADR must remain compatible with ADR-0138.

Retirement of the foundry path follows the six-path deprecation pattern and does
not silently delete canonical provenance.

## Decision

### B.1 Decision statement

Oyatie realignment work MUST follow the five-phase canonical build sequence in
Section D-1.

Phase 4 work MUST follow the Big 8 sub-sequence in Section D-2.

Every agent dispatch in the realignment wave MUST include an agent-class-specific
five-anchor set as defined in Section D-3.

Every microservice ownership audit MUST use the five-dimension protocol in
Section D-4.

Every microservice parity check MUST use top-3 counterpart union coverage as
defined in Section D-5.

Every microservice audit wave landing MUST include the four named deliverables
in Section D-6.

Audit batches MUST be grouped by canonical phase and capped at eight Codex
agents per batch as defined in Section D-7 and Section D-14.

Wave 14 MUST aggregate audit findings into a realignment remediation backlog as
defined in Section D-8.

Wave 15 and later remediation MUST be split by severity and phase as defined in
Section D-9.

Every landing MUST satisfy the verification SLA in Section D-10 before it can be
called done.

Every brief MUST use the five-citation header and decision-tree convention in
Section D-11.

Foundry MUST be absorbed according to Section D-12.

ADR-0321 scope MUST be interpreted according to Section D-13.

Dispatch MUST be Codex-only under the convention in Section D-14 while the active
directive remains in force.

### B.2 What this decision does not do

This ADR does not author any of the 316 microservice audit deliverables.

This ADR does not remediate any findings that future audits identify.

This ADR does not rewrite ADR-0321.

This ADR does not create a new master-plan projection.

This ADR does not change the microservice roster.

This ADR does not approve a Claude, Opus, Gemini, or non-Codex subagent dispatch
for the realignment batches.

This ADR does not allow agents to skip Oya VCS claim, verify, done, or promote
state transitions.

This ADR does not permit scripted or template-generated substantive content.

This ADR does not retire the `oyatie.foundry.*` principal namespace.

This ADR does not make `retired external agent harness` a canonical primitive.

### B.3 Decision drivers

Driver 1: canonical sequence must precede further depth.

Driver 2: broad scope must be ordered, not narrowed incorrectly.

Driver 3: microservice ownership must be audited before remediation.

Driver 4: substance must be verified by reading artifacts and checking anchors,
not by trusting self-report or line counts.

Driver 5: Big 8 enterprise displacement order must be visible in every Phase 4
brief.

Driver 6: Foundry capability must survive without preserving a standalone
foundry runtime path.

Driver 7: batch parallelism must improve throughput without recreating
same-file conflict and partial-completion failures.

Driver 8: every dispatch must be self-contained enough that a fresh Codex agent
can act without tribal memory.

## Consequences

### C.1 Positive consequences

Realignment agents have a single ordering doctrine.

Brief authors no longer invent their own anchor set.

The audit wave can parallelize without losing microservice-level ownership.

ADR-0321 cleanup can preserve the broad in-scope universe while enforcing Big 8
priority.

Foundry retirement can proceed without accidental capability loss.

Verification moves from self-report to artifact sampling plus anchor checks.

Promotion evidence can be organized according to ADR-0327 instead of scattered
across ad hoc notes.

### C.2 Negative consequences

The sequence slows opportunistic authoring.

An agent that finds an obvious Phase 4 long-tail gap must not pre-empt Phase 0,
Phase 1, Phase 2, Phase 3, or Big 8 work unless explicitly dispatched.

The audit wave produces more front-loaded reading.

Each microservice owner must inspect the local path and cross-references before
writing findings, even when a shallow artifact-count check would be faster.

Briefs become stricter.

A malformed five-citation header is now a dispatch defect, not a formatting
preference.

Some existing files will be found internally inconsistent.

This ADR deliberately surfaces those contradictions instead of smoothing them
over during audit.

### C.3 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | One canonical phase sequence and one brief convention prevent agents from encoding different local orders. | Every realignment brief names phase, batch, agent class, five anchors, in-scope set, out-of-scope set, and halt condition. |
| Observability | Every landing produces readable evidence: line count, sampled artifacts, anchor checks, and Oya VCS state transition evidence. | `oya vcs verify`, `oya vcs done`, and `oya vcs promote` carry evidence strings naming line counts or artifact counts. |
| Scalability | Twelve batches of eight Codex agents are allowed without losing ownership because each agent owns a microservice or bounded authoring surface. | No two agents write the same microservice audit deliverable or ADR section unless a leader assigns a handoff. |
| Performance | The orchestration target is throughput through safe parallel batches, not single-agent heroics. | Batch plans keep independent microservices parallel and keep dependent remediation sequential. |
| Optimization | Audit work is separated from remediation so discovery does not thrash the same files being corrected. | Audit waves write findings first; remediation sub-waves consume the backlog later. |
| Code quality | Documentation quality is treated as a governed artifact with shape, anchors, substance, and verification gates. | Substance-bar lanes, no-template-stamping lanes, promotion gates, and anchor-header checks can independently fail a landing. |

### C.4 Hyperscaler-grade rigor application

Named precedent: the phase sequence follows hyperscaler platform layering rather
than product-platform sprawl.

Cloud substrate is established before platform substrate, as AWS, Google Cloud,
Azure, and Cloudflare establish identity, networking, key management, storage,
compute, billing, and control-plane primitives before higher product surfaces.

Platform substrate is established before capability substrates, as Palantir
Foundry and ServiceNow Now Platform expose ontology, workflow, policy, and audit
primitives before solution templates.

Communication and collaboration land before enterprise-app displacement because
Microsoft 365 and Google Workspace show that daily work surfaces train the user
on identity, sharing, search, calendar, document, and collaboration primitives.

Distribution and B2B SaaS land after the substrate because Salesforce,
ServiceNow, Workday, SAP, Oracle, Adobe, Atlassian, and HubSpot surfaces can be
represented as role, workflow, ontology, policy, and capability-tier projections.

Failure-mode tree: dispatch without anchors, dispatch in the wrong phase,
parallel writes to the same surface, artifact completion by line count only,
and foundry deletion without capability absorption are all named failure modes
with blocking behavior in Sections D-10 through D-14.

Capacity math: the batch ceiling is eight Codex agents per batch because the
realignment spec calls for twelve batches of eight across roughly seventy-nine
microservices; that gives enough parallel throughput while keeping leader
verification finite.

Observability hooks: each landing must emit Oya VCS evidence and, once lanes
exist, governance audit events for anchor check, sampled artifact check, and
promotion-gate check.

Rollback path: a bad audit wave is rolled back by refusing `done` or `promote`,
leaving the claim open, and dispatching a bounded fix against the affected
deliverable rather than rewriting the whole batch.

Multi-region awareness: this ADR is documentation-only, but the phase sequence
requires cloud-cell, cloud-network, cloud-kms, cloud-secrets, identity, tenancy,
audit-chain, and observability to be coherent before product surfaces claim
multi-region maturity.

Sovereign-cell awareness: phase ordering keeps compliance, tenancy, audit-chain,
governance, cloud-kms, cloud-secrets, and consent-graph ahead of product-layer
claims for KR-CSAP, EU-sovereign, FedRAMP High, IL5/6, CN-PIPL, and similar
packs.

Versioning and deprecation: Foundry retirement follows ADR-0138 and does not
rename or remove canonical principals without a deprecation path.

## D. Detailed Mechanics

### D-1: 5-Phase Canonical Build Sequence

D-1.1. The five-phase sequence is normative for realignment dispatch.

D-1.2. A later phase cannot claim corpus completeness while an earlier phase has
unresolved P0 contradictions or substance-bar failures.

D-1.3. A later phase may be audited in parallel only when the dispatch explicitly
states that the work is read-only or findings-only and cannot promote ahead of
the predecessor phase gate.

D-1.4. The sequence is a documentation and product-ordering doctrine, not a
runtime deployment order.

D-1.5. Phase 0 is Shared Infrastructure.

D-1.6. Phase 0 covers the cloud-* family and adjacent shared-infrastructure
services.

D-1.7. Phase 0 service 01: `cloud-iam`.

D-1.8. Phase 0 service 02: `cloud-kms`.

D-1.9. Phase 0 service 03: `cloud-secrets`.

D-1.10. Phase 0 service 04: `cloud-iac`.

D-1.11. Phase 0 service 05: `cloud-network`.

D-1.12. Phase 0 service 06: `cloud-network-dns`.

D-1.13. Phase 0 service 07: `cloud-data`.

D-1.14. Phase 0 service 08: `cloud-storage`.

D-1.15. Phase 0 service 09: `cloud-compute-functions`.

D-1.16. Phase 0 service 10: `cloud-compute-k8s`.

D-1.17. Phase 0 service 11: `cloud-compute-vm`.

D-1.18. Phase 0 service 12: `cloud-billing`.

D-1.19. Phase 0 service 13: `cloud-billing-tax`.

D-1.20. Phase 0 service 14: `cloud-capacity`.

D-1.21. Phase 0 service 15: `cloud-cell`.

D-1.22. Phase 0 service 16: `cloud-dcops`.

D-1.23. Phase 0 service 17: `cloud-finops`.

D-1.24. Phase 0 service 18: `cloud-marketplace`.

D-1.25. Phase 0 service 19: `cloud-fsh`.

D-1.26. The Phase 0 audit question is: can every higher-level service trust the
shared cloud substrate for identity binding, key custody, secret custody,
networking, DNS, storage, compute, billing, tax, capacity, cells, data-center
operations, FinOps, marketplace substrate, and field-service/hardware support.

D-1.27. If any Phase 0 service has a contradictory tenant, region, key, secret,
network, billing, or capacity story, downstream product maturity claims are
blocked.

D-1.28. Phase 1 is Foundations / Platform Substrate.

D-1.29. Phase 1 intentionally excludes `foundry`.

D-1.30. Phase 1 service 01: `identity`.

D-1.31. Phase 1 service 02: `tenancy`.

D-1.32. Phase 1 service 03: `audit-chain`.

D-1.33. Phase 1 service 04: `governance`.

D-1.34. Phase 1 service 05: `compliance`.

D-1.35. Phase 1 service 06: `observability`.

D-1.36. Phase 1 service 07: `payments`.

D-1.37. Phase 1 service 08: `finops-portal`.

D-1.38. Phase 1 service 09: `api-gateway`.

D-1.39. Phase 1 service 10: `application`.

D-1.40. Phase 1 service 11: `developer-sdk`.

D-1.41. Phase 1 service 12: `network`.

D-1.42. Phase 1 service 13: `cell`.

D-1.43. The Phase 1 audit question is: can every higher-level service rely on
one identity, one tenant boundary, one audit chain, one governance plane, one
compliance model, one observability substrate, one payment model, one FinOps
view, one gateway, one application shell, one SDK, one network abstraction, and
one cell abstraction.

D-1.44. Phase 2 is Core Capability Substrate.

D-1.45. Phase 2 jointly absorbs foundry capability.

D-1.46. Phase 2 service 01: `intelligence`.

D-1.47. Phase 2 service 02: `ontology`.

D-1.48. Phase 2 service 03: `workflow-engine`.

D-1.49. Phase 2 service 04: `workflow-studio`.

D-1.50. Phase 2 service 05: `consent-graph`.

D-1.51. Phase 2 service 06: `detection`.

D-1.52. The Phase 2 audit question is: can agentic, workflow, ontology,
consent, abuse-detection, AI, policy-adjacent, and Foundry-like product behavior
be composed from these six services without a standalone Foundry runtime.

D-1.53. Phase 3 is Communication and Collaboration.

D-1.54. Phase 3 service 01: `messenger`.

D-1.55. Phase 3 service 02: `mail`.

D-1.56. Phase 3 service 03: `drive`.

D-1.57. Phase 3 service 04: `calendar`.

D-1.58. Phase 3 service 05: `meet`.

D-1.59. Phase 3 service 06: `recordings`.

D-1.60. Phase 3 service 07: `notes`.

D-1.61. Phase 3 service 08: `docs`.

D-1.62. Phase 3 service 09: `sheets`.

D-1.63. Phase 3 service 10: `slides`.

D-1.64. Phase 3 service 11: `forms`.

D-1.65. Phase 3 service 12: `connector`.

D-1.66. Phase 3 service 13: `comms-email`.

D-1.67. Phase 3 service 14: `community`.

D-1.68. Phase 3 service 15: `shorts`.

D-1.69. Phase 3 service 16: `analytics`.

D-1.70. Phase 3 service 17: `tasks`.

D-1.71. Phase 3 service 18: `translate`.

D-1.72. Phase 3 service 19: `search`.

D-1.73. The Phase 3 audit question is: can everyday collaboration, messaging,
content, work coordination, communication search, and translation train the user
on the same substrate vocabulary before enterprise displacement surfaces land.

D-1.74. Phase 4 is Distribution plus B2B Enterprise SaaS.

D-1.75. Phase 4 starts with distribution substrate.

D-1.76. Phase 4 distribution service 01: `marketplace`.

D-1.77. Phase 4 distribution service 02: `plugin-app-store`.

D-1.78. Phase 4 distribution service 03: `workplace-integration`.

D-1.79. Phase 4 distribution service 04: `feature-flags`.

D-1.80. Phase 4 then covers the B2B-leader and ERP displacement services.

D-1.81. Phase 4 B2B/ERP service: `crm`.

D-1.82. Phase 4 B2B/ERP service: `marketing-automation`.

D-1.83. Phase 4 B2B/ERP service: `contact-center`.

D-1.84. Phase 4 B2B/ERP service: `performance-management`.

D-1.85. Phase 4 B2B/ERP service: `learning-management`.

D-1.86. Phase 4 B2B/ERP service: `itsm`.

D-1.87. Phase 4 B2B/ERP service: `incident-management`.

D-1.88. Phase 4 B2B/ERP service: `contract-lifecycle-management`.

D-1.89. Phase 4 B2B/ERP service: `data-warehouse`.

D-1.90. Phase 4 B2B/ERP service: `design-collaboration`.

D-1.91. Phase 4 B2B/ERP service: `whiteboard`.

D-1.92. Phase 4 B2B/ERP service: `data-pipeline`.

D-1.93. Phase 4 B2B/ERP service: `healthcare-integration`.

D-1.94. Phase 4 ERP service: `production-planning`.

D-1.95. Phase 4 ERP service: `quality-management`.

D-1.96. Phase 4 ERP service: `plant-maintenance`.

D-1.97. Phase 4 ERP service: `warehouse`.

D-1.98. Phase 4 ERP service: `real-estate`.

D-1.99. Phase 4 ERP service: `treasury`.

D-1.100. Phase 4 ERP service: `supply-chain-planning`.

D-1.101. Phase 4 ERP service: `global-trade`.

D-1.102. Phase 4 ERP service: `financial-planning`.

D-1.103. The Phase 4 audit question is: can enterprise SaaS vendor families be
covered as capability-tier projections and operationally justified services,
without recreating vendor suite boundaries.

D-1.104. A service name in this section is a phase-placement rule.

D-1.105. It is not a statement that every current directory is already at
substance bar.

D-1.106. It is not permission to create missing directories without a separate
claim, ADR, and microservice doc-set plan.

D-1.107. If the live repository uses an alias such as `cloud-k8s` for a canonical
name such as `cloud-compute-k8s`, the audit finding records the alias and the
remediation backlog resolves it.

D-1.108. Alias cleanup must not happen inside an audit-only wave unless the
brief explicitly authorizes remediation.

### D-2: Big 8 Sub-Sequence within Phase 4

D-2.1. Phase 4A is the Big 8 enterprise SaaS displacement layer.

D-2.2. Phase 4B is the long-tail B2B SaaS, cloud-infra, PaaS, and developer-tool
dossier layer.

D-2.3. Phase 4A.1 is HR/Payroll, the Workday family.

D-2.4. HR/Payroll ships first.

D-2.5. The Workday family includes workforce, payroll, benefits, performance
management, learning management, compensation, recruiting, time tracking, and
employee identity flows.

D-2.6. The reason HR/Payroll ships first is that employee identity, manager
hierarchy, job role, pay group, benefits eligibility, shift, worker type, and
cost center become inputs to nearly every other enterprise journey.

D-2.7. A CRM or ERP workflow that cannot bind the user to workforce identity
will duplicate HR data or invent local manager semantics.

D-2.8. Phase 4A.2 is ERP, the SAP family.

D-2.9. ERP ships second.

D-2.10. The SAP family includes production planning, quality management, plant
maintenance, warehouse, real estate, treasury, supply-chain planning, global
trade, financial planning, procurement-adjacent flows, and financial close.

D-2.11. The reason ERP ships second is that ERP establishes the deepest
enterprise operational moat: material movement, financial truth, supply chain,
plant state, treasury exposure, and regulated records.

D-2.12. Phase 4A.3 is CRM, the Salesforce family.

D-2.13. CRM ships third.

D-2.14. The Salesforce family includes sales force automation, marketing
automation, customer support, contact-center composition, partner/community
surfaces, CPQ, quote-to-cash, and account intelligence.

D-2.15. CRM ships after HR and ERP because sales, marketing, service, and revenue
flows depend on workforce, account, product, inventory, contract, financial, and
fulfillment data.

D-2.16. Phase 4A.4 is ServiceNow.

D-2.17. ServiceNow covers ITSM, incident, service catalog, change, asset, CMDB,
employee service, and workflow automation patterns.

D-2.18. Phase 4A.5 is HubSpot.

D-2.19. HubSpot covers SMB and mid-market CRM, marketing automation, content,
support, lifecycle automation, and operations hub patterns.

D-2.20. Phase 4A.6 is Microsoft.

D-2.21. Microsoft covers Dynamics, Power Platform, Graph, Teams-adjacent
collaboration, SharePoint-adjacent content, and identity-heavy enterprise app
patterns.

D-2.22. Phase 4A.7 is Oracle.

D-2.23. Oracle covers database-adjacent enterprise applications, NetSuite,
Fusion, HCM, supply chain, EPM, and OCI-linked enterprise stack patterns.

D-2.24. Phase 4A.8 is Adobe.

D-2.25. Adobe covers experience cloud, analytics, campaign, commerce, document,
creative collaboration, and content-supply-chain patterns.

D-2.26. Phase 4A.9 is Atlassian.

D-2.27. Atlassian covers Jira, Confluence, Bitbucket, service management,
Compass, Loom-adjacent async collaboration, and software-delivery planning
patterns.

D-2.28. The realignment shorthand "4A.4-4A.8" means the remaining default order
after the first three priority families.

D-2.29. This ADR spells Atlassian explicitly so no agent drops it from the Big 8
family list.

D-2.30. Quarterly tactical priority MAY refine the order after ServiceNow, but
only through a follow-up ADR or explicit council directive.

D-2.31. Tactical refinement MUST NOT move HR/Payroll below ERP or CRM.

D-2.32. Tactical refinement MUST NOT move ERP below CRM.

D-2.33. Tactical refinement MUST NOT promote Phase 4B above any unresolved Big 8
family.

D-2.34. Phase 4B includes long-tail B2B SaaS.

D-2.35. Phase 4B includes specialty B2B SaaS.

D-2.36. Phase 4B includes niche B2B SaaS.

D-2.37. Phase 4B includes cloud-infra primitives.

D-2.38. Phase 4B includes PaaS.

D-2.39. Phase 4B includes developer tools.

D-2.40. Phase 4B is lowest priority, not out of scope.

D-2.41. An ADR-0321 dossier for a long-tail vendor is valid only if the brief
states why it is not pre-empting Big 8 work.

D-2.42. A Phase 4B dossier that lacks this sequencing note fails the brief
format convention in D-11.

### D-3: Agent-Class-Specific 5-Anchor Sets

D-3.1. A five-anchor set is a dispatch contract.

D-3.2. It is not a bibliography.

D-3.3. The agent must read or inspect the five anchors and use them to decide
scope, alignment, and verification.

D-3.4. The brief must encode the anchor set by agent class so the agent does not
infer the wrong sources from a generic template.

D-3.5. Agent class 1: microservice-ownership-audit agent.

D-3.6. Anchor 1: unified ecosystem thesis.

D-3.7. Anchor 2: the microservice's own PRD.

D-3.8. Anchor 3: the microservice's existing artifact coherence summary, or the
closest live local artifact inventory if no summary exists.

D-3.9. Anchor 4: the microservice's feature-parity-matrix top-3 counterparts.

D-3.10. Anchor 5: documentation-rigor Section 1.1.

D-3.11. Agent class 2: ADR-0321-dossier authoring agent.

D-3.12. Anchor 1: ADR-0321 Section A scope definition.

D-3.13. Anchor 2: Wave-3-G unified ecosystem thesis.

D-3.14. Anchor 3: ADR-0316 capability-tier doctrine.

D-3.15. Anchor 4: the microservice's surface-coverage artifact for that vendor.

D-3.16. Anchor 5: feature-parity-matrix for that vendor.

D-3.17. Agent class 3: IP-slice authoring agent.

D-3.18. Anchor 1: the microservice's own PRD.

D-3.19. Anchor 2: ADR-0263 audit emission contract.

D-3.20. Anchor 3: ADR-0244 tenant scoping.

D-3.21. Anchor 4: the microservice's journey-coverage artifact.

D-3.22. Anchor 5: substance bar from documentation-rigor and ADR-0322.

D-3.23. Agent class 4: per-microservice ADR authoring agent.

D-3.24. Anchor 1: the microservice's PRD.

D-3.25. Anchor 2: the Wave-3-G doctrine cluster.

D-3.26. Anchor 3: ADR-0105 layer enum.

D-3.27. Anchor 4: the microservice's IP set.

D-3.28. Anchor 5: substance bar from documentation-rigor and ADR-0322.

D-3.29. Agent class 5: journey-author.

D-3.30. Anchor 1: the persona or role dossier for the journey actor.

D-3.31. Anchor 2: the owning microservice PRD.

D-3.32. Anchor 3: the cross-microservice handoff matrix for the journey path.

D-3.33. Anchor 4: ADR-0244 tenant and personal/work boundary rules.

D-3.34. Anchor 5: documentation-rigor Section 1.1 plus the relevant UX-shell
doctrine from the unified ecosystem thesis.

D-3.35. Agent class 6: runbook-author.

D-3.36. Anchor 1: the owning microservice ARCHITECTURE.md or operational design
doc.

D-3.37. Anchor 2: the microservice SLO document or service-level target source.

D-3.38. Anchor 3: ADR-0263 audit emission contract.

D-3.39. Anchor 4: the relevant failure-mode or incident-response doc.

D-3.40. Anchor 5: documentation-rigor runbook and hyperscaler-grade requirements.

D-3.41. Agent class 7: pack-overlay-author.

D-3.42. Anchor 1: the base microservice PRD or pack integration doc.

D-3.43. Anchor 2: the compliance pack manifest or regional pack authority.

D-3.44. Anchor 3: ADR-0244 tenant scoping and sovereign-child boundary.

D-3.45. Anchor 4: ADR-0263 audit emission contract and retention class.

D-3.46. Anchor 5: documentation-rigor Section 1.1 with sovereign-cell awareness.

D-3.47. Agent class 8: cross-handoff-matrix-author.

D-3.48. Anchor 1: the source microservice PRD.

D-3.49. Anchor 2: the target microservice PRD.

D-3.50. Anchor 3: ADR-0244 tenancy boundary.

D-3.51. Anchor 4: ADR-0263 audit emission contract.

D-3.52. Anchor 5: the unified ecosystem thesis sections on one workflow, one
ontology, and one UX shell.

D-3.53. Agent class 9: benchmark-author.

D-3.54. Anchor 1: the microservice performance benchmark doc if it exists.

D-3.55. Anchor 2: feature-parity matrix for the top-3 counterparts.

D-3.56. Anchor 3: capability-tier registry for Bronze, Silver, Gold, and
Platinum expectations.

D-3.57. Anchor 4: observability and SLO documents for the microservice.

D-3.58. Anchor 5: documentation-rigor capacity math and performance rules.

D-3.59. Agent class 10: remediation-author.

D-3.60. Anchor 1: the Wave 14 finding row.

D-3.61. Anchor 2: the artifact being remediated.

D-3.62. Anchor 3: the canonical phase and Big 8 ordering rule that governs it.

D-3.63. Anchor 4: the source ADR or standard that the finding says is violated.

D-3.64. Anchor 5: the verification evidence from the failed landing.

D-3.65. If an agent class is missing from this section, the orchestrator must
choose the closest existing class and record the mapping in the brief.

D-3.66. The orchestrator must not send a generic "read relevant docs" instruction
in place of a five-anchor set.

### D-4: Per-Microservice Ownership-Coherence Audit 5-Dimension Protocol

D-4.1. The audit wave is an ownership-coherence audit.

D-4.2. It is not a remediation wave.

D-4.3. Each audit agent owns one microservice for the audit scope.

D-4.4. The agent reads the microservice path before writing findings.

D-4.5. Dimension 1 is internal coherence.

D-4.6. Internal coherence asks whether PRD, ARCHITECTURE, README, compliance,
contracts, IPs, runbooks, SLOs, policies, capability tiers, onboarding, test
plans, benchmarks, and handoff docs agree with each other.

D-4.7. Internal coherence failures include contradictory tenant models,
duplicated policy authorities, incompatible event names, inconsistent service
ownership, mismatched tier definitions, divergent data models, and stale
microservice names.

D-4.8. Dimension 2 is outbound cross-references.

D-4.9. Outbound cross-references ask whether the microservice cites the right
root ADRs, related microservices, personas, journeys, packs, contracts, and
standards.

D-4.10. Outbound failures include broken links, citations to retired docs,
missing ADR-0244 tenancy references, missing ADR-0263 audit references, missing
ADR-0316 capability-tier references, and references to Foundry as a standalone
runtime after absorption.

D-4.11. Dimension 3 is substance bar.

D-4.12. Substance asks whether the artifact could let a programming-capable
intern build or operate the described surface from cold.

D-4.13. Substance failures include generic prose, placeholder mechanics,
template-stamped lists, missing failure modes, missing capacity math, missing
observability hooks, missing rollback, and missing versioning or deprecation.

D-4.14. Dimension 4 is canonical-direction alignment.

D-4.15. Canonical direction asks whether the microservice is a projection of the
unified ecosystem thesis instead of a copied vendor suite boundary.

D-4.16. Alignment failures include product-island architecture, separate
identity, separate workflow engines, separate policy engines, separate audit
logs, separate training models, and ungoverned extension systems.

D-4.17. Dimension 5 is industry-counterpart parity.

D-4.18. Industry parity asks whether the microservice covers the union of major
features across its top-3 counterparts, subject to intentional out-of-scope
marking.

D-4.19. Parity failures include missing core counterpart features, benchmark
numbers without named counterpart sources, capability tiers that cannot map to
vendor plans, and unsupported migration paths.

D-4.20. The audit verdict vocabulary is PASS, PASS-WITH-FINDINGS, REVISE, or
BLOCK.

D-4.21. PASS means all five dimensions clear with only minor editorial findings.

D-4.22. PASS-WITH-FINDINGS means the microservice can proceed in sequence but
has non-blocking remediation rows.

D-4.23. REVISE means the microservice cannot promote past the current phase
gate until findings are remediated.

D-4.24. BLOCK means the microservice contains a hard contradiction that can
mislead downstream implementation.

D-4.25. A BLOCK finding must name the exact file and fix shape.

D-4.26. A REVISE finding must name the missing substance, anchor, or parity item.

D-4.27. A PASS-WITH-FINDINGS item must still enter the Wave 14 backlog.

D-4.28. The audit agent does not silently fix findings unless the brief says the
wave is remediation-capable.

D-4.29. The audit agent may correct obvious typos only if the brief explicitly
authorizes incidental cleanup.

D-4.30. This ADR's default is findings-only.

### D-5: Top-3 Counterparts UNION-Coverage Parity Bar

D-5.1. Each microservice must identify its top-3 industry counterparts.

D-5.2. The counterpart sources are ADR-0321, the capability-tier registry, and
the benchmark catalog.

D-5.3. If those sources disagree, the audit records the disagreement instead of
choosing silently.

D-5.4. The parity bar is union coverage.

D-5.5. Union coverage means that if any of the top-3 counterparts has a major
feature, Oyatie must either cover it or mark it intentionally out of scope.

D-5.6. Union coverage is stricter than average coverage.

D-5.7. A feature cannot be ignored because only one counterpart has it.

D-5.8. A feature cannot be ignored because it is difficult.

D-5.9. A feature cannot be ignored because it belongs to a vendor suite boundary
that Oyatie refuses to copy.

D-5.10. The correct question is which Oyatie microservice, capability tier,
workflow, ontology projection, policy permit, marketplace integration, or pack
overlay owns the feature.

D-5.11. Niche features can be marked `out-of-scope intentional`.

D-5.12. An intentional out-of-scope marker must name the reason.

D-5.13. Valid reasons include conflict with unified ecosystem thesis, conflict
with tenant boundary, conflict with compliance pack, deprecation in the industry,
insufficient enterprise relevance, or better expression through an extension.

D-5.14. Invalid reasons include "not yet documented", "hard to build", "vendor
specific", "future work", or "not in current template".

D-5.15. The feature-parity matrix uses the states `covered`, `partial`,
`missing`, and `out-of-scope intentional`.

D-5.16. `covered` requires a path to the owning artifact.

D-5.17. `partial` requires a missing-gap note.

D-5.18. `missing` requires a proposed remediation target.

D-5.19. `out-of-scope intentional` requires a doctrine reason and approving ADR
or standard.

D-5.20. The top-3 set must be named in the header of the parity matrix.

D-5.21. The benchmark doc must use the same top-3 set unless it records a reason
for deviation.

D-5.22. The capability-tier delta doc must use the same top-3 set unless it
records a reason for deviation.

D-5.23. A microservice cannot pass the parity dimension when the three audit docs
use different counterpart sets without explanation.

D-5.24. The union-coverage bar applies per capability tier where counterpart
products have tiered offerings.

D-5.25. Bronze can intentionally exclude a feature only when Silver, Gold,
Platinum, or an extension path covers it and the tier doctrine allows that split.

### D-6: 4-Doc Deliverable per Microservice in Audit Wave

D-6.1. Each audit wave agent lands exactly four required deliverables unless the
brief states that an existing file is being revised in place.

D-6.2. Deliverable 1 is `coherence-audit-2026-05-20.md`.

D-6.3. The coherence audit contains the five-dimension verdicts from D-4.

D-6.4. The coherence audit names the microservice owner, phase, batch, top-3
counterparts, source anchors, sampled files, and final verdict.

D-6.5. The coherence audit does not hide missing local documents.

D-6.6. If a PRD, ARCHITECTURE, manifest, contract, runbook, or IP set is missing,
that absence is a finding.

D-6.7. Deliverable 2 is `feature-parity-matrix-2026-05-20.md`.

D-6.8. The feature parity matrix lists major features from the top-3 counterparts
and the Oyatie owning surface for each feature.

D-6.9. The matrix marks each row covered, partial, missing, or out-of-scope
intentional.

D-6.10. Deliverable 3 is `performance-benchmark-numbers-2026-05-20.md`.

D-6.11. The benchmark doc names latency p50, p95, p99, throughput, cost, scale
ceiling, and stress-scenario evidence where available.

D-6.12. If live benchmark numbers do not exist, the doc must distinguish
measured values from target budgets and counterpart-public claims.

D-6.13. A target budget must not be presented as measured evidence.

D-6.14. Deliverable 4 is
`capability-tier-deltas-vs-counterparts-2026-05-20.md`.

D-6.15. The tier delta doc maps Oyatie Bronze, Silver, Gold, and Platinum against
counterpart tiers.

D-6.16. The tier delta doc names which missing features belong to which tier.

D-6.17. The tier delta doc names which features are extension-only.

D-6.18. The tier delta doc names which features are out-of-scope intentional.

D-6.19. These four files are audit evidence, not marketing collateral.

D-6.20. Each file must cite the agent-class five anchors in its frontmatter or
opening section.

D-6.21. Each file must include the microservice phase and batch.

D-6.22. Each file must include a `Verification Notes` section explaining what
was read.

D-6.23. Each file must include a `Findings` section even when empty.

D-6.24. Each file must include a `Backlog Rows` section or a statement that no
backlog rows are produced.

D-6.25. A missing deliverable blocks the audit agent's `done` transition.

### D-7: Audit Wave Batch Grouping by Canonical Phase

D-7.1. Audit wave batching follows canonical phase order.

D-7.2. The batch ceiling is eight Codex agents.

D-7.3. Twelve batches of eight Codex agents are the nominal throughput model.

D-7.4. The ceiling protects against leader verification overload and same-surface
coordination collisions.

D-7.5. Wave 1 is this backbone-authoring wave.

D-7.6. Wave 1 includes ADR-0328 and any directly associated control surface that
the active plan assigns.

D-7.7. Wave 2 covers Phase 0.

D-7.8. Wave 2 uses three batches.

D-7.9. Wave 2 Batch A covers the first eight Phase 0 cloud services.

D-7.10. Wave 2 Batch B covers the next eight Phase 0 cloud services.

D-7.11. Wave 2 Batch C covers the remaining Phase 0 cloud services plus any
Phase 0 alias-resolution findings.

D-7.12. Wave 3 covers Phase 1.

D-7.13. Wave 3 uses two batches.

D-7.14. Wave 3 Batch A covers identity, tenancy, audit-chain, governance,
compliance, observability, payments, and finops-portal.

D-7.15. Wave 3 Batch B covers api-gateway, application, developer-sdk, network,
cell, and any Phase 1 alias or missing-doc findings.

D-7.16. Wave 4 covers Phase 2.

D-7.17. Wave 4 uses one batch plus the Foundry absorption special case.

D-7.18. Wave 4 covers intelligence, ontology, workflow-engine, workflow-studio,
consent-graph, and detection.

D-7.19. Wave 4 also records the foundry absorption findings required by D-12.

D-7.20. Wave 5 covers the first Phase 3 batch.

D-7.21. Wave 6 covers the second Phase 3 batch.

D-7.22. Wave 7 covers the third Phase 3 batch.

D-7.23. Phase 3 batches should keep tightly coupled collaboration services near
each other for review.

D-7.24. Messenger, mail, drive, calendar, meet, recordings, notes, and docs
belong together unless the leader assigns a better split.

D-7.25. Sheets, slides, forms, connect, comms-email, community, shorts,
analytics, tasks, translate, and search are split across the remaining Phase 3
batches according to available agent count.

D-7.26. Wave 8 begins Phase 4.

D-7.27. Wave 8 should prioritize distribution substrate and HR/Payroll.

D-7.28. Wave 9 should prioritize ERP.

D-7.29. Wave 10 should prioritize CRM.

D-7.30. Wave 11 should prioritize ServiceNow and HubSpot.

D-7.31. Wave 12 should prioritize Microsoft, Oracle, Adobe, and Atlassian.

D-7.32. Wave 13 should cover remaining Phase 4B long-tail, cloud-infra, PaaS,
developer-tool, specialty, and niche dossiers when Big 8 work is not blocked.

D-7.33. Phase 4 may require three to six batches depending on live roster and
audit depth.

D-7.34. The leader records the actual batch allocation when dispatching.

D-7.35. No batch may exceed eight Codex agents.

D-7.36. If an agent halts cleanly with a checkpoint, its replacement does not
increase the concurrent ceiling.

D-7.37. If an agent is stuck without checkpoint, the leader must mark the slice
blocked and avoid silent duplicate writes.

D-7.38. A batch is complete only when every assigned agent has landed its
deliverables, failed with a named blocker, or transferred a checkpoint.

D-7.39. A wave is complete only when all batches have evidence and the leader has
sampled the outputs under D-10.

### D-8: Wave 14 Audit Findings Aggregation

D-8.1. Wave 14 is orchestrator-authored.

D-8.2. It consumes the findings from Waves 2 through 13.

D-8.3. It produces a realignment remediation backlog.

D-8.4. The backlog is not a prose summary.

D-8.5. The backlog is a structured queue of findings that can be assigned to
remediation agents.

D-8.6. Every finding row names the microservice.

D-8.7. Every finding row names severity.

D-8.8. Severity values are P0, P1, P2, and P3.

D-8.9. P0 means hard contradiction or unsafe downstream instruction.

D-8.10. P1 means substance or canonical-direction failure that blocks phase
promotion.

D-8.11. P2 means parity, benchmark, or cross-reference gap that must be fixed
before later readiness claims.

D-8.12. P3 means cosmetic, ordering, naming, or link cleanup that does not block
phase progress but must be tracked.

D-8.13. Every finding row names category.

D-8.14. Categories include internal-coherence, outbound-cross-reference,
substance-bar, canonical-direction, parity, benchmark, capability-tier,
foundry-absorption, ADR-0321-cleanup, and brief-format.

D-8.15. Every finding row names file.

D-8.16. File may be a missing expected path when the finding is absence.

D-8.17. Every finding row names fix.

D-8.18. Fix must be concrete enough for a remediation agent to act without
re-triaging the entire corpus.

D-8.19. The backlog is prioritized per Big 8.

D-8.20. HR/Payroll findings outrank ERP findings when severity is equal.

D-8.21. ERP findings outrank CRM findings when severity is equal.

D-8.22. CRM findings outrank ServiceNow, HubSpot, Microsoft, Oracle, Adobe, and
Atlassian findings when severity is equal.

D-8.23. Big 8 findings outrank Phase 4B long-tail findings when severity is
equal.

D-8.24. Earlier canonical phases outrank later phases when a contradiction would
propagate downstream.

D-8.25. The backlog must preserve evidence links back to the four audit docs.

D-8.26. The backlog must distinguish audit observation from remediation decision.

### D-9: Wave 15+ Remediation Sub-Wave Structure

D-9.1. Wave 15 and later are remediation waves.

D-9.2. Remediation consumes the Wave 14 backlog.

D-9.3. Remediation does not re-open the audit methodology unless a finding proves
the audit protocol itself is wrong.

D-9.4. Sub-wave 15A handles P0 hard contradictions.

D-9.5. A P0 hard contradiction includes incompatible tenant boundary, contradictory
principal namespace, contradictory service owner, contradictory phase placement,
or instructions that would produce the wrong product.

D-9.6. Sub-wave 15A must run before substance gap waves.

D-9.7. Sub-waves 15B through 15F handle substance gaps by phase.

D-9.8. 15B handles Phase 0 substance gaps.

D-9.9. 15C handles Phase 1 substance gaps.

D-9.10. 15D handles Phase 2 substance gaps.

D-9.11. 15E handles Phase 3 substance gaps.

D-9.12. 15F handles Phase 4 substance gaps.

D-9.13. Sub-wave 15G handles ADR-0321 cleanup.

D-9.14. ADR-0321 cleanup includes duplicate dossier removal, monotonic section
ordering, Big 8 density upgrade, scope note correction, and counterpart-source
normalization.

D-9.15. Sub-wave 15H handles cross-reference and cosmetic cleanup.

D-9.16. Cross-reference cleanup includes broken links, stale doc names, retired
paths, missing inbound citations, and mismatch between frontmatter and body.

D-9.17. Cosmetic cleanup is allowed only after blocking substance and coherence
work is clear.

D-9.18. Sub-wave 15I handles Foundry retirement plus retired external agent harness-drop cleanup.

D-9.19. 15I retires `microservices/foundry/` through ADR-0138.

D-9.20. 15I preserves `oyatie.foundry.*` Cedar principal namespace.

D-9.21. 15I moves capability references to intelligence, workflow-engine,
workflow-studio, ontology, governance, and tenancy.

D-9.22. 15I drops `retired external agent harness` as a canonical primitive.

D-9.23. A remediation sub-wave must not combine P0 contradiction fixes with
cosmetic cleanup unless the same line cannot be edited safely twice.

D-9.24. A remediation sub-wave must verify the specific finding rows it claims.

D-9.25. A remediation sub-wave must not close unrelated backlog rows because they
are nearby.

D-9.26. A remediation sub-wave must preserve audit evidence so future reviewers
can see why the edit was made.

### D-10: Verification SLA per Landing

D-10.1. Every landing must be verified before it is called done.

D-10.2. The minimum verification is not a line count.

D-10.3. The minimum verification is not an agent completion notification.

D-10.4. The minimum verification is not the existence of a file path.

D-10.5. The verifier must read three random artifacts from the agent's output
when the agent produced more than three artifacts.

D-10.6. If the agent produced one artifact, the verifier reads that artifact.

D-10.7. If the agent produced two artifacts, the verifier reads both artifacts.

D-10.8. If the agent produced three artifacts, the verifier reads all three.

D-10.9. "Read" means inspect enough content to evaluate scope, anchors, and
substance, not skim the first heading.

D-10.10. The verifier cross-checks the five agent-class-specific canonical
anchors.

D-10.11. Anchor check failure blocks `done`.

D-10.12. Missing anchor citation blocks `done`.

D-10.13. Wrong phase blocks `done`.

D-10.14. Missing Big 8 priority note blocks Phase 4 `done`.

D-10.15. Missing top-3 counterpart set blocks microservice audit `done`.

D-10.16. Missing four deliverables blocks microservice audit `done`.

D-10.17. Missing Foundry absorption note blocks Phase 2 `done` when foundry is
in scope.

D-10.18. Missing Codex-only dispatch evidence blocks batch completion under the
active directive.

D-10.19. Verification evidence must name the sampled artifacts.

D-10.20. Verification evidence must name the anchor set.

D-10.21. Verification evidence must name pass or fail for each anchor.

D-10.22. Verification evidence must name the line count where a line floor is
part of the claim.

D-10.23. Verification evidence must name any known gap.

D-10.24. A known gap is acceptable only if it is explicitly below the landing's
blocking threshold.

D-10.25. A verifier who cannot complete the check marks the landing blocked,
not done.

D-10.26. The Oya VCS `verify` transition carries a concise evidence string.

D-10.27. The Oya VCS `done` transition carries the same or stricter evidence.

D-10.28. The Oya VCS `promote` transition carries the bundle name, environment,
and evidence.

D-10.29. The stop condition is fresh evidence, no blocking anchor failures, no
missing deliverables, and no unresolved P0 contradiction in the landing scope.

### D-11: Brief Format Convention

D-11.1. Every realignment brief leads with a five-citation header.

D-11.2. The header must be visible before task instructions.

D-11.3. The header must name each anchor path or ADR.

D-11.4. The header must name why that anchor applies.

D-11.5. The header must name the agent class.

D-11.6. The header must name the canonical phase.

D-11.7. The header must name the batch.

D-11.8. The header must name in-scope artifacts.

D-11.9. The header must name out-of-scope artifacts.

D-11.10. The header must name the stop condition.

D-11.11. A valid header shape is:

```text
5-CITATION HEADER
Agent class: <class from ADR-0328 D-3>
Canonical phase: <phase and service family>
Batch: <wave/batch id, max eight Codex agents>
Anchor 1: <path or ADR> - <why this constrains the work>
Anchor 2: <path or ADR> - <why this constrains the work>
Anchor 3: <path or ADR> - <why this constrains the work>
Anchor 4: <path or ADR> - <why this constrains the work>
Anchor 5: <path or ADR> - <why this constrains the work>
In scope: <specific files/directories/artifacts>
Out of scope: <specific exclusions>
Stop condition: <verified deliverables plus Oya VCS transition>
```

D-11.12. Every brief encodes the agent-class anchor set.

D-11.13. Every brief encodes a decision tree for in-scope and out-of-scope.

D-11.14. The decision tree must be specific to artifact class.

D-11.15. A microservice audit decision tree asks whether the file is inside the
owned microservice path, whether it is required to judge one of the five
dimensions, and whether editing is prohibited by audit-only scope.

D-11.16. An ADR-0321 dossier decision tree asks whether the vendor is B2B SaaS,
cloud-infra, PaaS, or developer-tool in scope, whether it belongs to Big 8 or
Phase 4B, and whether the dossier duplicates an existing section.

D-11.17. An IP-slice decision tree asks whether the slice maps to the owning
microservice PRD, emits ADR-0263 audit events, respects ADR-0244 tenant scoping,
and fits a single implementation PR.

D-11.18. A runbook decision tree asks whether the procedure has a trigger,
pre-checks, steps, verification, rollback, post-incident section, observability
hooks, and references.

D-11.19. A pack-overlay decision tree asks whether the overlay is jurisdiction,
tenant, data-class, policy, retention, and audit aware.

D-11.20. A cross-handoff decision tree asks whether both microservices agree on
contract, event, tenant boundary, error mode, and owner.

D-11.21. Briefs must say `Codex only` while the active directive remains in
force.

D-11.22. Briefs must say `no scripted authoring` when the output is substantive
documentation.

D-11.23. Briefs must say `HALT-CLEANLY with checkpoint` when an agent cannot
finish.

D-11.24. Briefs must not ask an agent to "fill the template" for substantive
content.

D-11.25. Briefs must not use "read relevant docs" as a substitute for anchors.

D-11.26. Briefs must not hide lower-priority Phase 4B work inside a Big 8 batch.

### D-12: Foundry Absorption

D-12.1. `microservices/foundry/` retires as a standalone microservice path.

D-12.2. Retirement follows ADR-0138 six-path-deprecation.

D-12.3. Retirement is not deletion without successor mapping.

D-12.4. Foundry capability is absorbed by `intelligence`.

D-12.5. `intelligence` owns library-first LLM binding, model/tool mediation,
agent assistance, and AI substrate capabilities under the governing ADR-0255
amendment.

D-12.6. Foundry capability is absorbed by `workflow-engine`.

D-12.7. `workflow-engine` owns durable workflow execution, agentic pipeline
runtime, replay, retries, idempotency, and execution state.

D-12.8. Foundry capability is absorbed by `workflow-studio`.

D-12.9. `workflow-studio` owns visual authoring, no-code workflow composition,
agentic node generation, and human ergonomic editing of workflow surfaces.

D-12.10. Foundry capability is absorbed by `ontology`.

D-12.11. `ontology` owns entity graph projection, agent state projection,
cross-microservice object semantics, and Foundry-like ontology actions.

D-12.12. Foundry capability is absorbed by `governance`.

D-12.13. `governance` owns policy authority, review gates, ADR promotion,
substance-bar lanes, and agent authority checks.

D-12.14. Foundry capability is absorbed by `tenancy`.

D-12.15. `tenancy` owns tenant membership, sovereign-child boundaries, principal
scope, and the namespace binding required by foundry principals.

D-12.16. The `oyatie.foundry.*` Cedar principal namespace remains canonical.

D-12.17. The principal namespace is provisioned and governed by tenancy and
governance.

D-12.18. The principal namespace does not imply a standalone foundry runtime.

D-12.19. Existing references to Foundry as a product, service, or capability
must be classified during audit.

D-12.20. Valid references to foundry capability are rewritten or confirmed as
capability references.

D-12.21. Invalid references to foundry as a standalone runtime become Wave 15I
remediation findings.

D-12.22. `retired external agent harness` framing is dropped from canonical primitives.

D-12.23. retired external agent harness may remain as historical prose only if the artifact clearly says
it is not a canonical primitive.

D-12.24. No new brief may cite retired external agent harness as a build target, primitive, service, or
capability owner.

D-12.25. A Phase 2 audit that ignores Foundry absorption fails D-10.

D-12.26. A Phase 2 remediation that removes foundry references without successor
mapping fails ADR-0138 compatibility.

### D-13: ADR-0321 In-Scope Universe

D-13.1. ADR-0321 is broad.

D-13.2. All B2B SaaS is in scope.

D-13.3. Hero B2B SaaS is in scope.

D-13.4. Long-tail B2B SaaS is in scope.

D-13.5. Specialty B2B SaaS is in scope.

D-13.6. Niche B2B SaaS is in scope.

D-13.7. Cloud-infra primitives are in scope.

D-13.8. PaaS is in scope.

D-13.9. Developer tools are in scope.

D-13.10. B2C consumer products are out of scope for ADR-0321.

D-13.11. IaaS hyperscalers in the narrow compute-rental sense are out of scope
for ADR-0321.

D-13.12. The out-of-scope IaaS boundary does not remove cloud-infra primitives
from Phase 4B.

D-13.13. The out-of-scope B2C boundary does not remove enterprise collaboration,
community, commerce, content, analytics, or workflow products when they are B2B
SaaS surfaces.

D-13.14. The correct ADR-0321 error class is not "wrong universe" when an agent
authors a cloud-infra, PaaS, developer-tool, or long-tail B2B dossier.

D-13.15. The correct error class is priority, duplicate, ordering, density,
counterpart selection, capability-tier mapping, or surface ownership.

D-13.16. Dossiers must be ordered so Big 8 priority remains visible.

D-13.17. Dossiers must avoid duplicate vendor sections.

D-13.18. Dossiers must use monotonic section numbering.

D-13.19. Dossiers must meet substance density comparable to the W3-W10 high
density examples in ADR-0321.

D-13.20. Dossiers must map vendor features to capability tiers or operationally
justified services.

D-13.21. Dossiers must not create vendor grouping microservices.

D-13.22. Dossiers must not treat product label familiarity as architecture.

D-13.23. Dossiers must cite the unified ecosystem thesis when explaining how
Oyatie avoids product-island behavior.

D-13.24. Dossiers must cite ADR-0316 when using capability tiers.

D-13.25. Dossiers must cite relevant microservice coverage artifacts when the
vendor surface maps into existing services.

### D-14: Codex-Only Dispatch Convention

D-14.1. The active directive is Codex-only dispatch.

D-14.2. No Claude Opus subagents are used for these batches while the directive
remains active.

D-14.3. No non-Codex authoring agent is substituted silently.

D-14.4. The concurrent ceiling is eight Codex agents per batch.

D-14.5. The ceiling applies to active agents, not just launched commands.

D-14.6. A halted agent with a completed checkpoint is no longer active after the
leader records the checkpoint and closes or replaces the lane.

D-14.7. A hung agent without checkpoint still consumes operational attention and
must be resolved before launching a replacement on the same slice.

D-14.8. Every agent gets an isolated ownership surface.

D-14.9. For a microservice audit, the ownership surface is one microservice path
and its required cross-reference reads.

D-14.10. For an ADR-0321 dossier cleanup, the ownership surface is a bounded
section range or vendor family, not the whole ADR unless assigned.

D-14.11. For a remediation row, the ownership surface is the finding row plus the
minimum files needed for the fix.

D-14.12. HALT-CLEANLY is mandatory.

D-14.13. HALT-CLEANLY means the agent stops before corrupting a partial artifact.

D-14.14. HALT-CLEANLY means the agent leaves a checkpoint that names completed
reads, completed edits, unverified claims, and next command.

D-14.15. HALT-CLEANLY means the leader can resume or reassign without guessing.

D-14.16. An agent that cannot satisfy anchors, cannot access a required file,
detects a same-file conflict, or reaches an ambiguity that changes scope must
halt cleanly.

D-14.17. An agent must not pad content to hit a line floor before halting.

D-14.18. An agent must not fabricate counterpart coverage when source artifacts
are missing.

D-14.19. An agent must not mark done when verification could not run.

D-14.20. An agent must not use scripts to generate substantive documentation.

D-14.21. Mechanical commands for line counts, searches, and verification are
allowed.

D-14.22. Scripted prose generation, loop-generated vendor sections, and
template-substituted artifact bodies are prohibited by ADR-0322 and ADR-0324.

D-14.23. A batch leader may use automation to check existence, counts, links, and
VCS state.

D-14.24. A batch leader may not use automation to replace bespoke authoring.

D-14.25. The stop condition for each batch is verified deliverables, no missing
anchors, no unresolved same-file conflicts, and Oya VCS evidence recorded.

### D-15: Multi-Context Deployment Matrix

D-15.1. The deployment matrix is mandatory for every Wave 2 through Wave 13
audit brief that touches a µservice with runtime, data, IAM, network, billing,
observability, or IaC behavior.

D-15.2. The matrix has six deployment contexts, not three generic buckets.

D-15.3. Context 1 id: `oyatie-public-cloud`.

D-15.4. Context 1 name: Oyatie public cloud.

D-15.5. Context 1 scope: Oyatie operates the tenant on Oyatie-owned cloud cells
and sells the service as the default managed SaaS/PaaS/IaaS bundle.

D-15.6. Context 1 hyperscaler/host: Oyatie-operated cells, which may sit on
Oyatie-owned hardware, OCI, AWS, or later facilities, but the tenant sees Oyatie
as the provider.

D-15.7. Context 1 IaC target: OpenTofu modules under
`microservices/<name>/iac/oyatie-public-cloud/`.

D-15.8. Context 1 network seam: `cloud-network`, `cloud-network-dns`,
`cloud-cell`, `api-gateway`, and Cilium policy form the tenant ingress, egress,
cell, DNS, and service-mesh boundary.

D-15.9. Context 1 IAM seam: `cloud-iam`, `identity`, `tenancy`, and Cedar policy
issue and verify principals without leaking any AWS or OCI account identity into
the tenant contract.

D-15.10. Context 1 observability seam: `observability` owns tenant-visible
traces, metrics, logs, SLO burn, evidence links, and support-export views.

D-15.11. Context 1 billing seam: `cloud-billing`, `cloud-billing-tax`,
`payments`, and `finops-portal` emit usage, cost, tax, invoice, credit, and
chargeback events.

D-15.12. Context 1 required µservice surface: every public GA µservice that is
part of FD-001 or a later paid managed offering must ship in this context.

D-15.13. Context 1 optional µservice surface: internal-only facilities such as
lab-only hardware support may be marked N/A only with an explicit service
manifest reason and audit finding link.

D-15.14. Context 1 CI lane: `ci-context-oyatie-public-cloud` must validate
OpenTofu plan, policy pack, smoke contract, telemetry emission, and cost event
shape before a public-cloud readiness claim.

D-15.15. Context 1 tenant onboarding: contract acceptance creates a tenant
record, selects the public-cloud cell, renders the OpenTofu variables, runs
`tofu init`, runs `tofu plan`, and runs `tofu apply` through `cloud-iac`.

D-15.16. Context 1 zero-handroll rule: no console click, SSH setup, or manual DNS
mutation is allowed in the onboarding path.

D-15.17. Context 2 id: `guest-on-aws`.

D-15.18. Context 2 name: Oyatie guest on AWS.

D-15.19. Context 2 scope: the same Oyatie stack runs inside a customer-owned or
Oyatie-managed AWS account while remaining architecturally provider-agnostic.

D-15.20. Context 2 hyperscaler/host: AWS primitives such as EC2, EBS, S3, VPC,
IAM roles, KMS, ELB, Route 53, CloudWatch, and Organizations are backing
resources, not product surfaces.

D-15.21. Context 2 IaC target: OpenTofu modules under
`microservices/<name>/iac/guest-on-aws/`.

D-15.22. Context 2 network seam: `cloud-network` maps the portable Oyatie
network model onto VPC, subnet, route table, security group, load balancer, and
private-link resources.

D-15.23. Context 2 IAM seam: `cloud-iam` brokers AWS role assumption only as an
adapter; tenant identity remains Oyatie identity plus Cedar authorization.

D-15.24. Context 2 observability seam: `observability` exports AWS-side metrics
where useful but canonicalizes them into Oyatie trace, metric, log, and audit
schemas.

D-15.25. Context 2 billing seam: `cloud-billing` and `cloud-finops` import AWS
Cost and Usage Report data, then normalize it into Oyatie usage events and tenant
chargeback rows.

D-15.26. Context 2 required µservice surface: shared infrastructure, platform,
security, audit, observability, collaboration, and enterprise workloads that a
customer can BYO-cloud must ship here unless service-local data gravity makes it
N/A.

D-15.27. Context 2 optional µservice surface: marketplace seller payout,
public-listing discovery, or hardware fleet support can be N/A if the service
manifest explains why AWS-guest tenancy is not a valid delivery shape.

D-15.28. Context 2 CI lane: `ci-context-guest-on-aws` must run static OpenTofu
validation, provider lock verification, policy evaluation, AWS plan generation,
and AWS cost-event fixture tests.

D-15.29. Context 2 tenant onboarding: customer supplies or delegates an AWS
account, `cloud-iac` assumes the bootstrap role, and the only provisioning path
is `tofu init → tofu plan → tofu apply`.

D-15.30. Context 2 forbidden pattern: a µservice must not call AWS APIs directly
from business logic when a `cloud-*` service owns that primitive.

D-15.31. Context 3 id: `guest-on-oci`.

D-15.32. Context 3 name: Oyatie guest on OCI.

D-15.33. Context 3 scope: the same Oyatie stack runs inside an Oracle Cloud
Infrastructure tenancy, with an Always Free sub-profile for demo, sandbox,
trial, and dev tenants.

D-15.34. Context 3 hyperscaler/host: OCI Compute, Ampere A1, Block Volume,
Object Storage, VCN, Vault, Load Balancer, Autonomous Database, Streaming,
Functions, API Gateway, and WAF are backing resources.

D-15.35. Context 3 IaC target: OpenTofu modules under
`microservices/<name>/iac/oci-guest/`, with Always Free modules under
`microservices/<name>/iac/oci-guest/always-free/`.

D-15.36. Context 3 network seam: `cloud-network` maps portable networks onto
VCN, subnet, gateway, load balancer, security list, NSG, and private endpoint
resources.

D-15.37. Context 3 IAM seam: `cloud-iam` maps Oyatie tenant roles to OCI dynamic
groups and policies without making OCI IAM the application authority.

D-15.38. Context 3 observability seam: `observability` imports OCI Monitoring,
Logging, Audit, and Health Check signals into Oyatie evidence and SLO schemas.

D-15.39. Context 3 billing seam: `cloud-billing` records OCI paid usage and
Always Free zero-cost usage with identical per-tenant attribution events.

D-15.40. Context 3 required µservice surface: all BYO-cloud capable platform and
product µservices must ship here unless their manifest marks OCI as N/A with a
specific missing primitive or capacity reason.

D-15.41. Context 3 optional µservice surface: GPU-heavy, region-restricted, or
hardware-adjacent services can gate Always Free while still supporting paid OCI
tiers.

D-15.42. Context 3 CI lane: `ci-context-guest-on-oci` must validate OCI provider
locks, Always Free budget fixtures, Oracle Linux arm64 builds, and Object
Storage state-backend wiring.

D-15.43. Context 3 tenant onboarding: tenant class selects standard OCI or
Always Free, `cloud-iac` renders OCI variables, and provisioning remains
`tofu init → tofu plan → tofu apply`.

D-15.44. Context 3 forbidden pattern: a demo tenant must not spill into AWS or
Oyatie-public paid capacity to hide an OCI Always Free capacity breach.

D-15.45. Context 4 id: `on-prem`.

D-15.46. Context 4 name: customer on-premises.

D-15.47. Context 4 scope: the same Oyatie stack runs in a customer-controlled
data center where the customer owns facility, hardware, network underlay, and
local operating procedures.

D-15.48. Context 4 hyperscaler/host: customer-provided bare metal, private
virtualization, Kubernetes nodes, storage arrays, HSMs, firewalls, and facility
networking.

D-15.49. Context 4 IaC target: OpenTofu modules under
`microservices/<name>/iac/on-prem/`.

D-15.50. Context 4 network seam: `cloud-network` maps portable intent onto
Cilium, BGP, MetalLB or equivalent, customer DNS, firewall rules, and physical
rack or VLAN constraints.

D-15.51. Context 4 IAM seam: `identity`, `tenancy`, `cloud-iam`, and Cedar own
application identity while integrating with customer IdP, LDAP, SAML, OIDC,
hardware token, or HSM anchors.

D-15.52. Context 4 observability seam: `observability` supports local retention,
customer export, disconnected operation, and delayed upstream support bundle
upload.

D-15.53. Context 4 billing seam: `cloud-billing`, `cloud-finops`, and
`payments` support license, subscription, internal chargeback, hardware
allocation, and customer-owned cost baselines.

D-15.54. Context 4 required µservice surface: regulated enterprise, sovereign,
defense, manufacturing, healthcare, and disconnected-capable workflows must
declare on-prem support or an explicit non-support reason.

D-15.55. Context 4 optional µservice surface: public marketplace discovery,
Oyatie-operated support automation, or public email delivery may be N/A only
when a local substitute or disconnected limitation is documented.

D-15.56. Context 4 CI lane: `ci-context-on-prem` must validate OpenTofu modules,
Kubernetes conformance, air-gap artifact availability, local state backend,
Talos/RHEL/SLES/Ubuntu/Debian packaging, and no cloud-console assumptions.

D-15.57. Context 4 tenant onboarding: customer hardware inventory and bootstrap
variables feed OpenTofu, and `tofu apply` creates the local cell without an SSH
runbook.

D-15.58. Context 4 forbidden pattern: a µservice must not require a managed AWS,
OCI, GitHub, Terraform Cloud, or SaaS control plane to become operational.

D-15.59. Context 5 id: `colo`.

D-15.60. Context 5 name: colocation deployment.

D-15.61. Context 5 scope: the same Oyatie stack runs on rented or owned
hardware in a colocation provider facility with stronger facility abstraction
than on-prem and less provider abstraction than public cloud.

D-15.62. Context 5 hyperscaler/host: Equinix Metal, Cyxtera, customer-rented
racks, carrier-neutral cross-connects, remote hands, BGP peers, and dedicated
storage or HSM devices.

D-15.63. Context 5 IaC target: OpenTofu modules under
`microservices/<name>/iac/colo/`.

D-15.64. Context 5 network seam: `cloud-network` maps portable networks onto
BGP sessions, cross-connects, routed VLANs, Cilium, MetalLB, DNS delegation, and
facility firewall constraints.

D-15.65. Context 5 IAM seam: `cloud-iam` integrates with facility APIs only for
infrastructure custody; tenant identity stays in Oyatie identity and Cedar.

D-15.66. Context 5 observability seam: `observability` adds facility health,
remote-hands events, hardware telemetry, cross-connect status, and regional
latency evidence to the normal Oyatie schema.

D-15.67. Context 5 billing seam: `cloud-billing`, `cloud-finops`, and
`cloud-dcops` attribute rack, power, cross-connect, remote-hands, bandwidth,
and hardware costs to tenants or cells.

D-15.68. Context 5 required µservice surface: services that claim sovereign
cell, dedicated hardware, regulated low-latency, or customer-controlled facility
support must ship here.

D-15.69. Context 5 optional µservice surface: services that depend on public
cloud proprietary managed databases may not claim colo support until the data
plane has a portable implementation.

D-15.70. Context 5 CI lane: `ci-context-colo` must validate provider modules,
facility fixture plans, bare-metal host bootstrap, hardware telemetry stubs, and
state backend isolation.

D-15.71. Context 5 tenant onboarding: facility allocation, network cross-connect
variables, hardware profile, and cell placement feed `cloud-iac`, then the
tenant is created by `tofu apply`.

D-15.72. Context 5 forbidden pattern: remote-hands instructions cannot replace
a declarative provisioning resource when the operation is repeatable.

D-15.73. Context 6 id: `oyatie-as-cloud-provider`.

D-15.74. Context 6 name: Oyatie as cloud provider.

D-15.75. Context 6 scope: Oyatie sells compute, storage, network, IAM, KMS,
billing, marketplace, capacity, cells, and data-center operations as its own
IaaS/PaaS surface to external customers.

D-15.76. Context 6 hyperscaler/host: Oyatie-owned cloud cells backed by
Oyatie-operated hardware and by the `cloud-*` µservice family as the customer
visible control plane.

D-15.77. Context 6 IaC target: `cloud-*` µservices are the IaaS surface, and
OpenTofu modules live under `microservices/<name>/iac/oyatie-iaas/`.

D-15.78. Context 6 network seam: `cloud-network`, `cloud-network-dns`,
`cloud-cell`, Cilium, BGP, load balancing, and tenant VPC-equivalent resources
form the product API.

D-15.79. Context 6 IAM seam: `cloud-iam`, `cloud-kms`, `cloud-secrets`,
`identity`, `tenancy`, and Cedar are not adapters; they are the authoritative
cloud provider identity and security services.

D-15.80. Context 6 observability seam: `observability` exposes provider-grade
customer telemetry, SLOs, audit export, control-plane health, usage evidence,
and support correlation.

D-15.81. Context 6 billing seam: `cloud-billing`, `cloud-billing-tax`,
`cloud-finops`, `payments`, `cloud-capacity`, and marketplace settlement own
metering, rating, invoicing, credit, tax, quota, and forecast semantics.

D-15.82. Context 6 required µservice surface: every Phase 0 cloud µservice is
mandatory because this context cannot exist without them.

D-15.83. Context 6 required platform surface: identity, tenancy, audit-chain,
governance, compliance, observability, payments, api-gateway, network, and cell
must ship because they are provider control-plane prerequisites.

D-15.84. Context 6 optional product surface: higher SaaS products may consume
Oyatie IaaS but are not required to be exposed as IaaS primitives.

D-15.85. Context 6 CI lane: `ci-context-oyatie-iaas` must validate provider API
contracts, OpenTofu provider plugin fixtures, tenant isolation, quota, metering,
audit, and control-plane SLOs.

D-15.86. Context 6 tenant onboarding: customer account creation provisions a
provider tenant, namespaces IAM/KMS/storage/network/compute, and applies the
customer's initial OpenTofu stack through the Oyatie provider.

D-15.87. Context 6 forbidden pattern: any claim that `cloud-storage` is an S3
wrapper, `cloud-kms` is a KMS wrapper, or `cloud-compute-vm` is an EC2 wrapper
is a canonical-direction defect.

D-15.88. The cloud-* family is Oyatie's own IaaS surface because customers
interact with Oyatie contracts, quotas, audit events, policies, SLOs, support,
and billing, not AWS or OCI contracts.

D-15.89. AWS and OCI adapters may back cells, but they must not leak provider
object shapes into product language, tenant policy, billing semantics, or
service boundaries.

D-15.90. `cloud-iam` owns Oyatie cloud identities even when guest-on-AWS uses
role assumption or guest-on-OCI uses dynamic groups underneath.

D-15.91. `cloud-kms` owns key lifecycle, rotation, custody, evidence, and
policy; AWS KMS and OCI Vault are backing adapters in guest contexts.

D-15.92. `cloud-storage` owns bucket/object/container semantics, retention,
encryption, quota, evidence, and billing; S3 and OCI Object Storage are backing
adapters.

D-15.93. `cloud-compute-vm` owns VM product semantics, scheduling, tenancy,
quota, isolation, images, and billing; EC2 and OCI Compute are backing adapters.

D-15.94. `cloud-compute-k8s` owns cluster product semantics, upgrade policy,
node pools, tenant namespaces, admission, and evidence.

D-15.95. `cloud-network` owns portable VPC-equivalent semantics, not a direct
copy of any provider's route-table vocabulary.

D-15.96. `cloud-billing` owns the usage ledger even when importing provider
costs from AWS CUR or OCI usage reports.

D-15.97. `cloud-marketplace` owns Oyatie marketplace settlement and listing
semantics even when a deployment context has no public marketplace surface.

D-15.98. `cloud-capacity` owns provider-neutral capacity classes such as CPU,
memory, storage, egress, cell, region, zone, and accelerator budgets.

D-15.99. `cloud-dcops` and `cloud-fsh` own hardware/facility workflows for
on-prem, colo, and Oyatie-as-cloud-provider contexts.

D-15.100. The architectural rationale is ADR-0215: one platform engine supports
multiple deployment contexts without forked product semantics.

D-15.101. The tenant-control rationale is ADR-0218: deployment context is a
tenant-level control, not a hidden operator preference.

D-15.102. A µservice manifest must name supported deployment contexts as an
array of the six canonical ids.

D-15.103. A µservice may mark a context N/A only with `reason`,
`missing_primitives`, `customer_impact`, `remediation_owner`, and
`target_revisit_gate`.

D-15.104. A µservice that stores tenant data must explain data residency and
state backend behavior for every supported context.

D-15.105. A µservice that emits events must explain audit-chain and
observability routing for every supported context.

D-15.106. A µservice that performs billing-relevant work must emit cost and
usage events in all supported contexts, including zero-cost OCI Always Free.

D-15.107. A µservice that provisions infrastructure must use `cloud-iac` and
OpenTofu for every supported context.

D-15.108. A µservice that serves public traffic must define ingress, DNS,
certificate, WAF, rate-limit, and tenant isolation behavior per context.

D-15.109. A µservice that performs privileged operations must define IAM, Cedar,
break-glass, and audit evidence per context.

D-15.110. A µservice that depends on hardware must distinguish on-prem, colo,
and Oyatie-as-cloud-provider facility ownership.

D-15.111. A µservice that depends on managed database semantics must show a
portable backing path for on-prem and colo.

D-15.112. A µservice that depends on cloud queue, stream, or function semantics
must map them through Oyatie-owned abstractions.

D-15.113. A µservice that cannot run disconnected must mark on-prem or colo N/A
unless the context explicitly allows connected operation.

D-15.114. The default Phase 0 expectation is all six contexts supported.

D-15.115. The default Phase 1 expectation is all six contexts supported.

D-15.116. The default Phase 2 expectation is all six contexts supported because
workflow, ontology, intelligence, consent, and detection are substrate services.

D-15.117. The default Phase 3 expectation is public cloud, AWS guest, OCI guest,
and Oyatie-as-cloud-provider; on-prem and colo require service-local review for
email, meet, recordings, and public-content dependencies.

D-15.118. The default Phase 4 expectation is support for contexts where the
target enterprise buyer reasonably operates that product class.

D-15.119. `cloud-iam` example: supports all six contexts because every context
needs principal, service-account, tenant-role, and policy binding.

D-15.120. `messenger` example: supports `oyatie-public-cloud`, `guest-on-aws`,
`guest-on-oci`, and `oyatie-as-cloud-provider` by default; on-prem and colo may
be required for regulated customers when push-notification and retention seams
are documented.

D-15.121. `foundry-replacement` capability example: intelligence, workflow,
ontology, governance, and tenancy together support all six contexts because the
agentic workflow substrate is not a public-cloud-only feature.

D-15.122. `marketplace` example: public listing and seller payout may be N/A for
disconnected on-prem, but local extension catalog install must still have an
on-prem story if extensions are supported.

D-15.123. `mail` example: public-cloud delivery can use hosted reputation
controls, while on-prem and colo require explicit DKIM, SPF, DMARC, abuse, and
egress constraints.

D-15.124. `cloud-fsh` example: guest-on-AWS may be N/A unless field-service
hardware is owned by Oyatie, but on-prem, colo, and Oyatie-as-cloud-provider are
primary contexts.

D-15.125. Per-context CI must include at least one plan-only IaC lane.

D-15.126. Per-context CI must include a service contract lane that does not
assume a specific provider SDK is available in business logic.

D-15.127. Per-context CI must include an observability fixture proving events,
traces, metrics, and logs carry the deployment context id.

D-15.128. Per-context CI must include an IAM fixture proving a tenant principal
can access only its allowed context resources.

D-15.129. Per-context CI must include a billing fixture proving usage is
attributed to tenant, context, cell, service, and capability tier.

D-15.130. Per-context CI must include failure-mode fixtures for provider outage,
cell loss, state-backend lock contention, quota exhaustion, and policy denial.

D-15.131. Tenant onboarding begins with tenant intent, not infrastructure
operator intent.

D-15.132. Tenant onboarding records `tenant_id`, `deployment_context`,
`cell_or_region`, `capability_tier`, `data_residency`, `billing_account`, and
`support_model`.

D-15.133. Tenant onboarding passes those values to `cloud-iac` as OpenTofu
variables.

D-15.134. Tenant onboarding produces a signed plan artifact before apply.

D-15.135. Tenant onboarding emits audit events for requested, planned, approved,
applied, verified, and rollback-ready states.

D-15.136. Tenant onboarding cannot be documented as "operator provisions the
account" without the `tofu` invocation and evidence events.

D-15.137. Tenant onboarding cannot be different for demo, trial, paid,
self-hosted, or BYO-cloud except through declared variables and modules.

D-15.138. Forbidden pattern: provider SDK calls in domain logic for IAM,
storage, compute, networking, billing, or KMS.

D-15.139. Forbidden pattern: README-only manual setup for a context that claims
support.

D-15.140. Forbidden pattern: context support claimed in prose without an
`iac/<context>/` module or N/A manifest.

D-15.141. Forbidden pattern: cloud-* service described as an adapter to AWS,
OCI, Azure, GCP, or a colocation API.

D-15.142. Forbidden pattern: tenant billing that collapses all contexts into a
single unmanaged cost bucket.

D-15.143. Forbidden pattern: telemetry that omits deployment context, cell,
region, tenant, and service labels.

D-15.144. Forbidden pattern: on-prem or colo support that depends on Terraform
Cloud, AWS Organizations, OCI tenancy policy, or GitHub-hosted control planes.

D-15.145. Audit severity P0 applies when a P0-priority µservice such as
HR/Payroll, ERP, or CRM violates a required context and would mislead Wave 2
through Wave 13 agents.

D-15.146. Audit severity P1 applies when any other in-scope µservice claims a
context without IaC, CI, tenant onboarding, IAM, observability, and billing
detail.

D-15.147. Audit severity P2 applies when the code and module shape are coherent
but the documentation or manifest fails to expose the context matrix.

D-15.148. A microservice audit must cite
`feedback_multi_context_provider_agnostic_2026_05_20.md` when applying this
section.

D-15.149. A microservice audit must cite ADR-0215 when explaining why one engine
serves all contexts.

D-15.150. A microservice audit must cite ADR-0218 when explaining tenant-level
deployment-context choice.

D-15.151. The stop condition for this constraint is a complete context matrix,
context-specific OpenTofu path or N/A reason, CI lane mapping, and tenant
onboarding flow for the audited µservice.

D-15.152. A Wave 2 audit that ignores this section cannot be promoted because it
would allow provider lock-in to enter the remediation backlog as normal work.

### D-16: Zero-Handroll OpenTofu IaC Substrate

D-16.1. The IaC substrate for Oyatie is OpenTofu, not Terraform.

D-16.2. `tofu` is the canonical CLI spelling in briefs, runbooks, examples,
module READMEs, CI lanes, and tenant onboarding procedures.

D-16.3. The word Terraform may appear only to say Terraform is forbidden,
superseded, migrated, or not the binary used by Oyatie.

D-16.4. OpenTofu version policy: every µservice module set pins the OpenTofu
minor version in `versions.tf` and in the CI lane that validates it.

D-16.5. OpenTofu version policy: version bumps are platform changes and must
land through `cloud-iac` compatibility tests before µservices adopt them.

D-16.6. OpenTofu version policy: no brief may say "latest OpenTofu" as a build
or deployment instruction.

D-16.7. Provider-pinning policy: every provider source and version is pinned in
`versions.tf`.

D-16.8. Provider-pinning policy: provider lock files are generated through the
approved OpenTofu path and verified in CI.

D-16.9. Provider-pinning policy: provider checksums must be part of supply-chain
evidence when modules are released.

D-16.10. Provider-pinning policy: unpinned provider constraints such as `>=`
without an upper bound are forbidden in release modules.

D-16.11. Provider-pinning policy: context-specific providers are scoped to the
context module and do not become business-logic dependencies.

D-16.12. Per-µservice directory contract: a deployable µservice owns
`microservices/<name>/iac/`.

D-16.13. Per-context directory contract: each supported context owns
`microservices/<name>/iac/<context>/`.

D-16.14. Canonical context directory: `iac/oyatie-public-cloud/`.

D-16.15. Canonical context directory: `iac/guest-on-aws/`.

D-16.16. Canonical context directory: `iac/oci-guest/`.

D-16.17. Canonical context directory: `iac/on-prem/`.

D-16.18. Canonical context directory: `iac/colo/`.

D-16.19. Canonical context directory: `iac/oyatie-iaas/`.

D-16.20. OCI Always Free sub-profile directory:
`iac/oci-guest/always-free/`.

D-16.21. Required file: `main.tf` declares resources and module composition for
the context.

D-16.22. Required file: `variables.tf` declares typed input variables, defaults
only where safe, validation rules, and tenant/context/cell identifiers.

D-16.23. Required file: `outputs.tf` exposes only stable outputs consumed by
`cloud-iac`, deployment verification, or dependent modules.

D-16.24. Required file: `versions.tf` pins OpenTofu and every provider.

D-16.25. Required file: `README.md` explains purpose, inputs, outputs, state
backend, supported context, CI lane, signing status, and zero-handroll
invocation.

D-16.26. Required optional file: `providers.tf` is allowed when provider
configuration would make `main.tf` unclear.

D-16.27. Required optional file: `locals.tf` is allowed for deterministic
resource naming, tagging, and context mapping.

D-16.28. Required optional file: `tests/` may contain OpenTofu-native module
tests and fixtures when the module has non-trivial branching.

D-16.29. Required optional file: `policy/` may contain policy-as-code checks
only when owned by the IaC lane and not by application runtime logic.

D-16.30. Required variable: `tenant_id`.

D-16.31. Required variable: `deployment_context`.

D-16.32. Required variable: `cell_id` or explicit N/A reason for services that
are not cell-scoped.

D-16.33. Required variable: `region` or `facility_id` according to context.

D-16.34. Required variable: `capability_tier`.

D-16.35. Required variable: `billing_account_id` or local chargeback owner for
self-hosted contexts.

D-16.36. Required variable: `data_residency_policy`.

D-16.37. Required output: `service_endpoint` when the µservice exposes an
endpoint.

D-16.38. Required output: `observability_export` when the module creates
metrics, logs, dashboards, alerts, or collectors.

D-16.39. Required output: `billing_meter_ids` when the module creates
billable or chargeback-relevant resources.

D-16.40. Required output: `iam_bindings` when the module creates roles, service
accounts, dynamic groups, or policy bindings.

D-16.41. Required output: `state_backend_ref` so verification can prove state
is stored in the approved backend.

D-16.42. Required output: `module_attestation_ref` so `cloud-iac` can verify
module provenance.

D-16.43. Module signing is mandatory under ADR-0039.

D-16.44. Module signing uses sigstore and cosign.

D-16.45. Module signing applies to module packages, provider-plugin artifacts
owned by Oyatie, generated SBOMs, and release manifests.

D-16.46. Module signing must occur before a module can be consumed by tenant
onboarding.

D-16.47. Module signing evidence must include module digest, signer identity,
OIDC issuer or key reference, timestamp, and policy result.

D-16.48. Unsigned local modules can be used only in isolated development lanes
and cannot satisfy Wave 2 audit readiness.

D-16.49. State backend for `guest-on-aws`: S3 for state plus DynamoDB-compatible
locking.

D-16.50. State backend for `guest-on-oci`: OCI Object Storage for state plus an
Autonomous Database lock table or approved lock equivalent.

D-16.51. State backend for `on-prem`: MinIO or customer-approved object storage
plus a durable lock table.

D-16.52. State backend for `colo`: MinIO or facility-approved object storage
plus a durable lock table.

D-16.53. State backend for `oyatie-public-cloud`: internal Oyatie-managed
storage surfaced through approved cloud-storage semantics.

D-16.54. State backend for `oyatie-as-cloud-provider`: Oyatie `cloud-storage`
and its lock primitive through the Oyatie provider.

D-16.55. State backend policy: Terraform Cloud is forbidden.

D-16.56. State backend policy: local disk state is forbidden for shared,
tenant, CI, staging, or production modules.

D-16.57. State backend policy: hand-edited tfstate is forbidden in every
context.

D-16.58. State backend policy: state import must be captured as a controlled
OpenTofu operation with audit evidence.

D-16.59. State backend policy: secrets must not be stored in state unless the
provider resource makes it unavoidable and the module documents mitigation.

D-16.60. `cloud-iac` is the IaC orchestrator µservice.

D-16.61. `cloud-iac` owns module registry integration.

D-16.62. `cloud-iac` owns per-tenant variable rendering.

D-16.63. `cloud-iac` owns context selection.

D-16.64. `cloud-iac` owns state backend selection.

D-16.65. `cloud-iac` owns plan generation.

D-16.66. `cloud-iac` owns policy checks before apply.

D-16.67. `cloud-iac` owns approval workflow integration.

D-16.68. `cloud-iac` owns apply invocation.

D-16.69. `cloud-iac` owns drift detection.

D-16.70. `cloud-iac` owns rollback plan generation.

D-16.71. `cloud-iac` owns attestation verification.

D-16.72. `cloud-iac` owns module signing verification.

D-16.73. `cloud-iac` does not own application business logic.

D-16.74. `cloud-iac` does not allow each µservice to invent a separate
provisioning CLI.

D-16.75. Tenant onboarding command sequence is `tofu init → tofu plan → tofu
apply`.

D-16.76. `tofu init` must configure the approved state backend for the selected
context.

D-16.77. `tofu init` must use signed module sources or development-only sources
that cannot promote.

D-16.78. `tofu plan` must produce a persisted signed plan artifact.

D-16.79. `tofu plan` must include tenant, context, cell, service, and module
version labels.

D-16.80. `tofu plan` must run policy checks before approval.

D-16.81. `tofu apply` must consume an approved plan artifact when the target is
shared, staging, tenant, or production infrastructure.

D-16.82. `tofu apply` must emit audit events for start, resource delta, success,
failure, rollback-ready, and drift baseline.

D-16.83. `tofu apply` must update observability and billing surfaces when
resource changes affect them.

D-16.84. `tofu destroy` is not part of normal onboarding and must be gated as a
destructive deprovision operation.

D-16.85. Forbidden pattern: `null_resource`.

D-16.86. Forbidden pattern: `local-exec`.

D-16.87. Forbidden pattern: `remote-exec`.

D-16.88. Forbidden pattern: SSH provisioners.

D-16.89. Forbidden pattern: hand-edited tfstate.

D-16.90. Forbidden pattern: unsigned modules.

D-16.91. Forbidden pattern: `terraform` binary invocation.

D-16.92. Forbidden pattern: Terraform Cloud as a state backend.

D-16.93. Forbidden pattern: Pulumi as a primary infrastructure engine.

D-16.94. Forbidden pattern: CloudFormation as the primary infrastructure engine.

D-16.95. Forbidden pattern: ARM/Bicep templates as the primary infrastructure
engine.

D-16.96. Forbidden pattern: shell script bootstrapping that creates durable
infrastructure outside OpenTofu state.

D-16.97. Forbidden pattern: manual cloud-console setup instructions.

D-16.98. Forbidden pattern: README-only tenant onboarding.

D-16.99. Forbidden pattern: provider credentials embedded in module variables,
outputs, state, or docs.

D-16.100. Forbidden pattern: context-specific one-off module that bypasses
`cloud-iac`.

D-16.101. If a provider cannot express a needed operation declaratively, the
proper remedy is an OpenTofu provider plugin or upstream provider contribution.

D-16.102. If a module needs a post-provision validation, the validation must be
a separate verification step and not provisioning hidden inside `local-exec`.

D-16.103. If a customer has an existing resource, onboarding must model it with
import or data sources and not with untracked manual assumptions.

D-16.104. If a context cannot support a resource, the module must fail at plan
time with a typed validation error.

D-16.105. If a µservice needs per-OS provisioning deltas, those deltas belong in
context modules or nested modules, not ad hoc scripts.

D-16.106. `guest-on-aws` example: S3 bucket creation for storage-backed state
must be OpenTofu resource declarations, not an AWS CLI pre-step.

D-16.107. `guest-on-oci` example: OCI Object Storage namespace and bucket wiring
must be OpenTofu resource declarations or documented data sources.

D-16.108. `on-prem` example: MinIO bucket creation must be declarative through
the provider path or a provider plugin, not SSH into a host.

D-16.109. `colo` example: BGP peer or load-balancer wiring must be declared in
modules or approved provider plugins.

D-16.110. `oyatie-as-cloud-provider` example: the Oyatie OpenTofu provider
calls `cloud-*` APIs as a cloud product surface.

D-16.111. `cloud-iac` example: per-tenant plan approval emits
`iac.plan.approved` and `iac.apply.started` audit events before resources
change.

D-16.112. `cloud-billing` example: module outputs identify meters so billing
can attribute resource creation immediately.

D-16.113. `observability` example: module outputs identify dashboard, alert,
log sink, and trace collector resources for tenant support evidence.

D-16.114. Every module README must include a "Forbidden operations" section
listing the forbidden patterns in this section.

D-16.115. Every module README must include a "Tenant onboarding" section with
the exact `tofu init`, `tofu plan`, and `tofu apply` sequence.

D-16.116. Every module README must include a "State backend" section for the
specific context.

D-16.117. Every module README must include a "Signing and provenance" section.

D-16.118. Every module README must include a "CI lane" section naming the
blocking validation lane.

D-16.119. Every module README must include a "Drift and rollback" section.

D-16.120. Every module README must include a "N/A context" explanation only if
the µservice manifest marks that context unsupported.

D-16.121. The Wave 2 audit must verify `cloud-iac` before accepting downstream
IaC claims.

D-16.122. A downstream µservice can pass the IaC dimension only when its module
structure and `cloud-iac` integration are both present or explicitly N/A.

D-16.123. A downstream µservice must not define its own state-backend policy in
conflict with this section.

D-16.124. A downstream µservice must not depend on hidden operator credentials.

D-16.125. A downstream µservice must not require bootstrap order that
contradicts the canonical build sequence.

D-16.126. Audit severity P0 applies when a P0-priority µservice depends on
Terraform, manual provisioning, unsigned modules, forbidden provisioners, or
untracked state for a deployment it claims to support.

D-16.127. Audit severity P1 applies when any other in-scope µservice has the
same IaC violation.

D-16.128. Audit severity P2 applies when OpenTofu modules exist and are
coherent but docs, README, signing evidence, or CI lane names are missing.

D-16.129. Audit finding text must say "OpenTofu", not "Terraform-compatible",
unless the finding is naming a forbidden old reference.

D-16.130. Audit agents must cite
`feedback_zero_handroll_opentofu_only_2026_05_20.md` when applying this
section.

D-16.131. Audit agents must cite ADR-0039 when module signing or provenance is
involved.

D-16.132. Audit agents must cite ADR-0216 when explaining why Terraform Cloud
or closed provider lock-in is disallowed.

D-16.133. Audit agents must cite ADR-0218 when explaining tenant-specific
deployment variables.

D-16.134. The OpenTofu path is a product contract, not an operator preference.

D-16.135. The same declarative resource graph is how Oyatie proves repeatable
deployment across public cloud, guest cloud, on-prem, colo, and its own IaaS.

D-16.136. The same declarative resource graph is how Oyatie proves rollback and
drift detection.

D-16.137. The same declarative resource graph is how Oyatie produces cost,
capacity, policy, and audit evidence.

D-16.138. The same declarative resource graph is how Wave 2 audit agents avoid
accepting hand-roll as "implementation detail".

D-16.139. A missing `iac/` directory is a finding even when the µservice is
currently documentation-only.

D-16.140. A present `iac/` directory with only placeholders is a substance-bar
finding.

D-16.141. A context module with no state backend contract is a finding.

D-16.142. A context module with no provider lock policy is a finding.

D-16.143. A context module with no signing evidence is a finding.

D-16.144. A context module with manual prerequisite steps is a finding.

D-16.145. A context module with hidden provider-specific semantics in output
names is a canonical-direction finding.

D-16.146. A context module that creates billable resources without meter outputs
is a billing seam finding.

D-16.147. A context module that creates privileged resources without IAM/Cedar
mapping is an IAM seam finding.

D-16.148. A context module that creates network resources without network seam
outputs is a network seam finding.

D-16.149. The stop condition for this constraint is validated OpenTofu module
shape, pinned versions, signed module evidence, approved state backend, no
forbidden patterns, and a `cloud-iac` tenant onboarding path.

D-16.150. A Wave 2 audit that accepts hand-roll provisioning must be reopened
because it invalidates multi-context deployment evidence.

### D-17: OS Support Matrix

D-17.1. The OS support matrix is a deployment portability contract for every
µservice that ships a binary, container image, host agent, CLI, daemon,
controller, or native bundle.

D-17.2. Tier 1 OS support is blocking for release claims.

D-17.3. Tier 2 OS support is test-only and soft-gated.

D-17.4. Out-of-scope OSes cannot appear in support claims, marketing claims,
CI-blocking lanes, package targets, or brief acceptance criteria.

D-17.5. Tier 1 OS 1: Talos Linux.

D-17.6. Talos scope: Kubernetes-native immutable node OS with API-driven
operation and no shell-based setup assumption.

D-17.7. Talos package format: container image plus Talos extension where a host
extension is required.

D-17.8. Talos CI lane: block on container image boot, Kubernetes conformance,
extension validation when used, and no SSH dependency.

D-17.9. Talos forbidden pattern: runbook steps that require logging into the
node shell.

D-17.10. Tier 1 OS 2: RHEL 9.x+.

D-17.11. RHEL scope: enterprise Linux baseline for regulated and support-heavy
customers.

D-17.12. RHEL package format: RPM plus container image.

D-17.13. RHEL CI lane: block on `linux/amd64`, `linux/arm64`, SELinux policy
fixtures where relevant, RPM install/upgrade/remove, and service smoke tests.

D-17.14. RHEL forbidden pattern: disabling SELinux as the default install path.

D-17.15. Tier 1 OS 3: Oracle Linux 9.x+.

D-17.16. Oracle Linux scope: OCI enterprise default and UEK-backed customer
deployments.

D-17.17. Oracle Linux package format: RPM plus container image.

D-17.18. Oracle Linux CI lane: block on UEK compatibility, Ampere arm64 builds,
RPM lifecycle, and OCI guest fixtures.

D-17.19. Oracle Linux forbidden pattern: treating Oracle Linux as "same as RHEL"
without UEK and OCI default testing.

D-17.20. Tier 1 OS 4: SLES 15 SP6+.

D-17.21. SLES scope: SUSE enterprise customer environments and regulated
European deployments.

D-17.22. SLES package format: RPM plus container image.

D-17.23. SLES CI lane: block on AppArmor/systemd integration, RPM lifecycle,
container smoke, and SUSE-supported kernel assumptions.

D-17.24. SLES forbidden pattern: relying on RHEL-only package scripts.

D-17.25. Tier 1 OS 5: Ubuntu 24.04 LTS+.

D-17.26. Ubuntu scope: default Linux server and developer-friendly LTS target.

D-17.27. Ubuntu package format: DEB plus container image.

D-17.28. Ubuntu CI lane: block on `linux/amd64`, `linux/arm64`, AppArmor
fixtures where relevant, DEB lifecycle, and service smoke tests.

D-17.29. Ubuntu forbidden pattern: depending on non-LTS PPAs for release
functionality.

D-17.30. Tier 1 OS 6: Debian 13+.

D-17.31. Debian scope: stable community and enterprise-compatible server target.

D-17.32. Debian package format: DEB plus container image.

D-17.33. Debian CI lane: block on stable package lifecycle, libc assumptions,
container smoke, and systemd integration when a host service exists.

D-17.34. Debian forbidden pattern: Ubuntu-only package assumptions.

D-17.35. Tier 1 OS 7: Rocky Linux 9.x+.

D-17.36. Rocky scope: RHEL-compatible customer estates.

D-17.37. Rocky package format: RPM plus container image.

D-17.38. Rocky CI lane: block on RPM lifecycle, systemd units where relevant,
SELinux fixtures, and container smoke.

D-17.39. Rocky forbidden pattern: treating it as untested because RHEL passed.

D-17.40. Tier 1 OS 8: AlmaLinux 9.x+.

D-17.41. AlmaLinux scope: RHEL-compatible customer estates.

D-17.42. AlmaLinux package format: RPM plus container image.

D-17.43. AlmaLinux CI lane: block on RPM lifecycle, systemd units where
relevant, SELinux fixtures, and container smoke.

D-17.44. AlmaLinux forbidden pattern: treating it as untested because Rocky
passed.

D-17.45. Tier 1 OS 9: CentOS Stream 10+.

D-17.46. CentOS Stream scope: RHEL upstream compatibility and early drift
detection.

D-17.47. CentOS Stream package format: RPM plus container image.

D-17.48. CentOS Stream CI lane: block on RPM lifecycle, SELinux fixtures,
kernel/userspace drift detection, and container smoke.

D-17.49. CentOS Stream forbidden pattern: making it a best-effort lane when
support is claimed.

D-17.50. Tier 1 OS 10: Amazon Linux 2023+.

D-17.51. Amazon Linux scope: AWS guest default host target.

D-17.52. Amazon Linux package format: RPM plus container image.

D-17.53. Amazon Linux CI lane: block on AWS guest fixtures, RPM lifecycle,
Graviton arm64 build, and IAM adapter assumptions where relevant.

D-17.54. Amazon Linux forbidden pattern: depending on Ubuntu or RHEL host paths
inside AWS guest modules.

D-17.55. Tier 1 OS 11: Flatcar Container Linux.

D-17.56. Flatcar scope: immutable container host OS.

D-17.57. Flatcar package format: container image plus ignition or system
extension pattern where approved.

D-17.58. Flatcar CI lane: block on container boot, update strategy, Cilium/K8s
fit, and no package-manager dependency.

D-17.59. Flatcar forbidden pattern: RPM or DEB install as the primary delivery
shape.

D-17.60. Tier 1 OS 12: Photon 5.x+.

D-17.61. Photon scope: VMware-managed and minimal container-host estates.

D-17.62. Photon package format: container image plus RPM where host install is
required.

D-17.63. Photon CI lane: block on minimal userspace, container smoke, RPM
lifecycle when used, and kernel feature assumptions.

D-17.64. Photon forbidden pattern: assuming a full general-purpose distro
userland.

D-17.65. Tier 1 OS 13: macOS Apple Silicon M5+.

D-17.66. macOS M5+ scope: developer workstation, Apple Silicon server, and
native frontend development target only where service-local docs authorize it.

D-17.67. macOS package format: `.pkg`, Homebrew formula, signed/notarized
artifact where relevant, and dev-container or local Kubernetes evidence when
the µservice supports local execution.

D-17.68. macOS CI lane: block on `darwin/arm64-m5+` build, no Intel fallback,
code signing or notarization where required, and package smoke.

D-17.69. macOS forbidden pattern: claiming support for Intel macOS or pre-M5
Apple Silicon.

D-17.70. Tier 2 target 1: `linux/ppc64le`.

D-17.71. Tier 2 ppc64le scope: IBM Power customer signal and portability test
coverage.

D-17.72. Tier 2 ppc64le policy: soft-gate, test-only, no GA support claim
unless a later ADR promotes it.

D-17.73. Tier 2 target 2: `linux/s390x`.

D-17.74. Tier 2 s390x scope: IBM Z customer signal and portability test
coverage.

D-17.75. Tier 2 s390x policy: soft-gate, test-only, no GA support claim unless
a later ADR promotes it.

D-17.76. Explicitly out of scope: Intel macOS.

D-17.77. Explicitly out of scope: Apple Silicon M1.

D-17.78. Explicitly out of scope: Apple Silicon M2.

D-17.79. Explicitly out of scope: Apple Silicon M3.

D-17.80. Explicitly out of scope: Apple Silicon M4.

D-17.81. Explicitly out of scope: FreeBSD.

D-17.82. Explicitly out of scope: OpenBSD.

D-17.83. Explicitly out of scope: Windows Server.

D-17.84. Explicitly out of scope: Solaris.

D-17.85. Architecture matrix primary target: `linux/amd64`.

D-17.86. Architecture matrix primary target: `linux/arm64`.

D-17.87. Architecture matrix primary target: `darwin/arm64-m5+`.

D-17.88. Architecture matrix Tier 2 target: `linux/ppc64le-test-only`.

D-17.89. Architecture matrix Tier 2 target: `linux/s390x-test-only`.

D-17.90. `linux/amd64` applies to Linux Tier 1 OSes where the OS supports it.

D-17.91. `linux/arm64` applies to Linux Tier 1 OSes and is mandatory for OCI
Ampere and AWS Graviton support.

D-17.92. `darwin/arm64-m5+` applies only to macOS Apple Silicon M5 and newer.

D-17.93. No `darwin/amd64` release artifact is allowed.

D-17.94. No `windows/server` release artifact is allowed for backend or
µservice runtime support.

D-17.95. Windows desktop frontend may exist only under the language policy and
does not create Windows Server support.

D-17.96. Per-OS package rule: RPM covers RHEL, Oracle Linux, SLES, Rocky,
AlmaLinux, CentOS Stream, Amazon Linux, and Photon host-install cases.

D-17.97. Per-OS package rule: DEB covers Ubuntu and Debian host-install cases.

D-17.98. Per-OS package rule: container image covers every Linux Tier 1 OS as
the primary Kubernetes deployment unit.

D-17.99. Per-OS package rule: Talos extension covers Talos host integration
only when a container image alone is insufficient.

D-17.100. Per-OS package rule: Flatcar ignition or system extension covers
Flatcar host integration only when a container image alone is insufficient.

D-17.101. Per-OS package rule: macOS `.pkg` covers workstation and Apple
Silicon server install paths.

D-17.102. Per-OS package rule: Homebrew formula covers macOS developer
workstation install paths.

D-17.103. Per-OS package rule: signed OCI container images are mandatory for
Linux deployment.

D-17.104. Per-OS package rule: SBOM and provenance attestations are mandatory
for every package format.

D-17.105. Per-µservice manifest path: `microservices/<name>/supported-oses.json`.

D-17.106. Manifest top-level field: `version`.

D-17.107. Manifest top-level field: `service`.

D-17.108. Manifest top-level field: `tier_1`.

D-17.109. Manifest top-level field: `tier_2_test_only`.

D-17.110. Manifest top-level field: `out_of_scope_explicit`.

D-17.111. Manifest top-level field: `arch_matrix`.

D-17.112. Manifest top-level field: `package_formats`.

D-17.113. Manifest top-level field: `ci_lanes`.

D-17.114. Manifest top-level field: `exceptions`.

D-17.115. Manifest top-level field: `rationale_refs`.

D-17.116. Manifest `tier_1` row requires `os`.

D-17.117. Manifest `tier_1` row requires `version_floor`.

D-17.118. Manifest `tier_1` row requires `architectures`.

D-17.119. Manifest `tier_1` row requires `package_formats`.

D-17.120. Manifest `tier_1` row requires `ci_lane`.

D-17.121. Manifest `tier_1` row requires `deployment_contexts`.

D-17.122. Manifest `tier_1` row requires `notes` when support differs by
context.

D-17.123. Manifest `exceptions` row requires a per-µservice ADR reference.

D-17.124. Manifest `exceptions` row cannot approve an out-of-scope OS without a
root ADR amendment.

D-17.125. Tier 1 CI policy: all Tier 1 lanes are blocking.

D-17.126. Tier 1 CI policy: build, package, smoke, and portability tests must
all pass for release claims.

D-17.127. Tier 1 CI policy: a µservice may not substitute one RHEL-compatible
distro for another.

D-17.128. Tier 1 CI policy: macOS M5+ lane blocks only when the µservice claims
macOS support or provides local developer tooling, but unsupported macOS must be
explicit in the manifest.

D-17.129. Tier 2 CI policy: ppc64le and s390x failures produce soft-gate
findings unless a service has promoted them by ADR.

D-17.130. Cross-OS portability gate: backend binaries and µservice runtime code
must be Rust.

D-17.131. Cross-OS portability gate: no Python interpreter dependency is allowed
for build, validation, bootstrap, or runtime.

D-17.132. Cross-OS portability gate: release builds use locked Cargo
dependencies.

D-17.133. Cross-OS portability gate: statically linked binaries are preferred
where feasible and musl/glibc choices must be explicit.

D-17.134. Cross-OS portability gate: package scripts cannot hide logic that
should be a Rust CLI or OpenTofu resource.

D-17.135. Cross-OS portability gate: container images must support
`linux/amd64` and `linux/arm64`.

D-17.136. Cross-OS portability gate: host agents must document kernel, eBPF,
TPM, HSM, Secure Enclave, SELinux, AppArmor, and systemd assumptions.

D-17.137. Cross-OS portability gate: no service can claim "Linux" support
without naming the Tier 1 distribution lanes it passes.

D-17.138. Example `cloud-kms`: tests TPM, HSM, OCI Vault adapter, AWS KMS
adapter, Apple Secure Enclave, and SoftHSM paths without moving key authority
outside Oyatie policy.

D-17.139. Example `cloud-compute-k8s`: tests Talos, Flatcar, Ubuntu, RHEL,
Oracle Linux, and SLES node assumptions as separate lanes.

D-17.140. Example `developer-sdk`: may support macOS M5+ Homebrew and `.pkg`
while explicitly excluding Intel macOS.

D-17.141. Example `messenger`: container image lanes must pass on all Linux Tier
1 OSes before a hosted or BYO-cloud claim.

D-17.142. Example `mail`: package and container tests must cover DKIM/SPF/DMARC
dependencies without shell-only setup.

D-17.143. Example `cloud-iac`: OpenTofu validation must run on Linux and macOS
M5+ tooling targets without Python helper scripts.

D-17.144. Forbidden pattern: `supported on Linux` without distro rows.

D-17.145. Forbidden pattern: package scripts that download unpinned binaries.

D-17.146. Forbidden pattern: CI lane named "enterprise Linux" that tests only
one distro.

D-17.147. Forbidden pattern: macOS support that silently includes Intel or M1
through M4 hardware.

D-17.148. Forbidden pattern: FreeBSD, OpenBSD, Windows Server, or Solaris
support claims in service docs.

D-17.149. Forbidden pattern: Python virtualenv, Node, Bash, Ruby, or Go tooling
as a portability prerequisite.

D-17.150. Audit severity P0 applies when a P0-priority µservice violates Tier 1
OS, package, architecture, or no-Python portability requirements.

D-17.151. Audit severity P1 applies when any other in-scope µservice violates
the same OS support requirements.

D-17.152. Audit severity P2 applies when implementation appears portable but
`supported-oses.json`, package docs, or CI lane names are missing.

D-17.153. Audit agents must cite
`feedback_os_support_matrix_2026_05_20.md` when applying this section.

D-17.154. Audit agents must cite the master-plan `supported_oses` key once
landed.

D-17.155. The stop condition for this constraint is a present manifest, all Tier
1 rows declared, Tier 2 rows marked test-only, out-of-scope rows explicit,
package formats mapped, and CI lane policy stated.

D-17.156. A Wave 2 audit that permits vague OS support would let downstream
agents build non-portable service assumptions into every deployment context.

### D-18: Rust-Strict Language Policy

D-18.1. Backend code is Rust.

D-18.2. µservice runtime code is Rust.

D-18.3. CLI tooling is Rust.

D-18.4. Validation tooling is Rust.

D-18.5. Code generation tooling is Rust.

D-18.6. Scripting is Rust.

D-18.7. CI logic that contains durable project behavior is Rust.

D-18.8. Deployment automation behavior is OpenTofu HCL where it provisions
infrastructure and Rust where it is a tool.

D-18.9. No backend or µservice exception exists without a per-µservice ADR.

D-18.10. The per-µservice exception ADR path is
`microservices/<name>/decisions/ADR-MS-NNN-non-rust-justification.md`.

D-18.11. Exception ADRs must name the non-Rust code path.

D-18.12. Exception ADRs must name why Rust is insufficient for that exact path.

D-18.13. Exception ADRs must name the language, runtime, dependency manager,
supply-chain posture, and integration boundary.

D-18.14. Exception ADRs must name observability and error-handling behavior.

D-18.15. Exception ADRs must name cross-OS portability impact.

D-18.16. Exception ADRs must name sunset or migration criteria.

D-18.17. Exception ADRs must not waive this section for convenience, ecosystem
familiarity, or agent preference.

D-18.18. Authorized backend non-Rust extension: `.tf` for OpenTofu modules.

D-18.19. Authorized backend non-Rust extension: `.cedar` for Cedar policy.

D-18.20. Authorized backend non-Rust extension: `.yaml` for configuration,
contracts, manifests, OpenAPI, AsyncAPI, OpenSLO, or Kubernetes objects.

D-18.21. Authorized backend non-Rust extension: `.json` for configuration,
contracts, manifests, evidence, or machine-readable specs.

D-18.22. Authorized backend non-Rust extension: `.proto` for proto3 contracts.

D-18.23. Authorized backend non-Rust file: `openapi.yaml`.

D-18.24. Authorized backend non-Rust file: `asyncapi.yaml`.

D-18.25. Authorized backend non-Rust extension: `.openslo.yaml`.

D-18.26. Authorized backend non-Rust extension: `.sql` for sqlx migrations and
schema fixtures.

D-18.27. Authorized backend non-Rust extension: `.md` for documentation.

D-18.28. Authorized backend non-Rust content cannot contain application logic
hidden in config.

D-18.29. Authorized backend SQL cannot become business workflow logic.

D-18.30. Authorized backend YAML or JSON cannot become scripted control flow for
agents.

D-18.31. Authorized backend Markdown cannot contain copy-paste shell procedures
that become required provisioning logic.

D-18.32. Frontend native bundle language: Swift for iOS.

D-18.33. Swift iOS code lives under `frontend/ios/`.

D-18.34. Swift iOS code is frontend only.

D-18.35. Swift iOS code does not authorize Swift backend services.

D-18.36. Frontend native bundle language: Swift for macOS.

D-18.37. Swift macOS code lives under `frontend/macos/`.

D-18.38. Swift macOS code is frontend only.

D-18.39. Swift macOS code does not authorize Swift scripting or µservice
runtime code.

D-18.40. Frontend native bundle language: Kotlin for Android.

D-18.41. Kotlin Android code lives under `frontend/android/`.

D-18.42. Kotlin Android code is frontend only.

D-18.43. Kotlin Multiplatform is allowed only for shared frontend code with a
per-µservice ADR.

D-18.44. Kotlin does not authorize JVM backend services.

D-18.45. Frontend native bundle language: WinUI 3 C#/.NET.

D-18.46. WinUI 3 code lives under `frontend/windows/`.

D-18.47. WinUI 3 code targets `net8.0+`.

D-18.48. WinUI 3 code is frontend only.

D-18.49. The .NET dependency is scoped to the Windows desktop bundle only.

D-18.50. Backend Rust µservices never depend on .NET.

D-18.51. Windows desktop frontend support does not imply Windows Server backend
support.

D-18.51a. Frontend web bundle framework: Leptos.

D-18.51b. Leptos web code is authored in Rust.

D-18.51c. Leptos web code lives under `frontend/web/` for the shared web
surface, and under `microservices/<name>/frontend/web/` for service-local web
surfaces.

D-18.51d. Mandatory runtime shape for first-party Oyatie web surfaces is
server-side rendering plus SELECTIVE WebAssembly hydration scoped to
interactive component or island boundaries; static sections SSR with no
hydration payload, interactive sections hydrate only the components that
require client-side reactivity.

D-18.51e. The SSR side runs as a Rust process using `leptos_axum` or an
equivalent server integration crate and is treated as a backend µservice
surface for build, deploy, observability, and IAM purposes.

D-18.51f. The hydration bundle is compiled to WebAssembly via `cargo-leptos`
or `wasm-bindgen` and shipped as static assets behind a CDN-cacheable URL.

D-18.51g. The `wasm-bindgen` JavaScript trampoline is generated bootstrap
output, not authored application logic, and is allowlisted only when its
provenance is the generated bundle.

D-18.51h. Hand-authored JavaScript or TypeScript application logic anywhere
under `frontend/web/` or `microservices/<name>/frontend/web/` is forbidden.

D-18.51i. CSR-only web shapes (entire page rendered client-side via
JavaScript with no server-rendered HTML) are forbidden for first-party
Oyatie web surfaces; SSR is mandatory for the initial document.

D-18.51j. Pure SSR with no hydration is the DEFAULT for any section without
client-side reactivity, and is fully permitted at section, component, route,
or island granularity; hydration is opt-in per island, not page-wide.

D-18.51m. Hydration boundary granularity: hydration MUST be scoped to a
single component, route segment, or Leptos `<Island>` boundary; whole-page
hydration is forbidden when any portion of the page is static.

D-18.51n. Default posture: every component is SSR-only with zero WASM
payload UNTIL the author explicitly opts the component into hydration via
the island or `#[component(transparent)]` interactive marker.

D-18.51o. Selective hydration implementation: use the Leptos islands
feature flag (`leptos_islands` or successor) and `<Island>` boundaries; if
the islands feature is not available, scope hydration via per-route
hydration entry points with explicit static-vs-interactive routing.

D-18.51p. WASM payload budget: per-route initial WASM transfer SHOULD stay
under 100 KB compressed for a static-heavy route and under 250 KB for an
interactivity-heavy route; routes exceeding these budgets MUST cite
ADR-0212 buildability doctrine and the µservice's capability-tier matrix.

D-18.51q. Hydration cost telemetry: every Leptos web surface MUST emit
TTI (time-to-interactive) and per-island hydration cost metrics via the
`observability` µservice contract; budgets land in the µservice's
`slos/web-frontend.openslo.yaml`.

D-18.51r. Forbidden pattern: hydrating a static page solely to add a
single interactive widget; the correct shape is islanding the widget and
leaving the rest as pure SSR.

D-18.51k. Because Leptos is Rust, this entry is not a non-Rust exception; it
is the canonical web frontend stack within the Rust-strict policy and does
not require a per-µservice non-Rust ADR.

D-18.51l. The Leptos web stack must respect the OS support matrix on the
server side and ship statically-linked Rust binaries for SSR per ADR-0328
D-17; the client side targets the browser engines that the matrix Tier 1
operating systems ship, with no Intel macOS, pre-M5 Apple Silicon, FreeBSD,
or Windows Server browser environments treated as supported.

D-18.52. Forbidden language: Python.

D-18.53. Forbidden Python path: `*.py`.

D-18.54. Forbidden Python use: validation script.

D-18.55. Forbidden Python use: code generation script.

D-18.56. Forbidden Python use: migration script.

D-18.57. Forbidden Python use: deployment helper.

D-18.58. Forbidden Python use: one-off corpus authoring helper.

D-18.59. Forbidden language: JavaScript application logic.

D-18.60. Forbidden JavaScript path: `*.js` when it contains authored
application behavior.

D-18.61. Forbidden JavaScript use: backend route, worker, CLI, deployment,
validation, codegen, or µservice glue.

D-18.62. Forbidden language: TypeScript application logic.

D-18.63. Forbidden TypeScript path: `*.ts` or `*.tsx` when it contains authored
application behavior outside allowed frontend bundle exceptions.

D-18.64. Generated SDK clients may exist only with generation provenance,
contract source, and no authored backend logic.

D-18.65. Forbidden language: Ruby.

D-18.66. Forbidden language: Perl.

D-18.67. Forbidden language: PHP.

D-18.68. Forbidden language: Java.

D-18.69. Forbidden language: Scala.

D-18.70. Forbidden language: Groovy.

D-18.71. Forbidden language: Go.

D-18.72. Forbidden language: F#.

D-18.73. Forbidden language: C++ except a narrow FFI shim justified by ADR.

D-18.74. Forbidden language: C# outside `frontend/windows/`.

D-18.75. Forbidden language: shell as durable project logic beyond tiny command
glue.

D-18.76. Canonical backend build invocation:
`cargo build --workspace --release --all-features --locked`.

D-18.77. The canonical build invocation is a release claim input, not a local
developer preference.

D-18.78. `cargo check --workspace --locked` may be used for faster verification
but does not replace the canonical release build.

D-18.79. `cargo test --workspace --locked` may be used for verification but
does not authorize non-Rust tooling.

D-18.80. `cargo xtask` is allowed only when `xtask` is Rust code inside the
workspace.

D-18.81. Forbidden backend build invocation: `make`.

D-18.82. Forbidden backend build invocation: `npm run`.

D-18.83. Forbidden backend build invocation: `python setup.py`.

D-18.84. Forbidden backend build invocation: `gradle`.

D-18.85. Forbidden backend build invocation: `mvn`.

D-18.86. Forbidden backend build invocation: `go build`.

D-18.87. Forbidden backend build invocation: `bundle exec`.

D-18.88. Forbidden backend build invocation: `php artisan`.

D-18.89. Forbidden backend build invocation: ad hoc shell build pipelines that
replace Cargo.

D-18.90. Anti-pattern: Codex agent writes a generator script to output Rust
source.

D-18.91. Anti-pattern: Codex agent writes Python to expand template sections.

D-18.92. Anti-pattern: Codex agent writes JavaScript to create OpenAPI files.

D-18.93. Anti-pattern: Codex agent writes shell loops that generate substantive
Rust modules or Markdown policy bodies.

D-18.94. Anti-pattern: Codex agent copies a Rust file and global-replaces nouns
without reading the service boundary.

D-18.95. Required authoring behavior: agent authors Rust code via direct edit
operations under ADR-0324 anti-script doctrine.

D-18.96. Required authoring behavior: mechanical tools may inspect, count,
format, compile, and test.

D-18.97. Required authoring behavior: mechanical tools may not create
substantive bodies by loop or template.

D-18.98. Required authoring behavior: generated code is allowed only when
generated by an approved Rust build/codegen path from canonical contracts.

D-18.99. Required authoring behavior: generated SDKs must declare the source
contract and generator provenance.

D-18.100. Required authoring behavior: generated SDKs do not become the source
of truth.

D-18.101. Runtime boundary example: `cloud-billing` rating logic is Rust even if
pricing tables are JSON.

D-18.102. Runtime boundary example: `cloud-iac` orchestration logic is Rust even
though it invokes OpenTofu modules.

D-18.103. Runtime boundary example: `governance` policy decisions are Rust plus
Cedar, not Python policy scripts.

D-18.104. Runtime boundary example: `observability` collector coordination is
Rust, not Node-based glue.

D-18.105. Runtime boundary example: `developer-sdk` source contracts are
OpenAPI, AsyncAPI, proto3, and Rust, not authored TypeScript logic.

D-18.106. Runtime boundary example: SQL migrations define schema, while Rust
owns business mutations and invariants.

D-18.107. Runtime boundary example: Kubernetes YAML declares deployment shape,
while Rust owns controller behavior unless a standard upstream controller is in
use with ADR approval.

D-18.108. Frontend boundary example: `frontend/ios/` Swift can render a native
app that calls Rust-defined APIs.

D-18.109. Frontend boundary example: `frontend/android/` Kotlin can render a
native app that calls Rust-defined APIs.

D-18.110. Frontend boundary example: `frontend/windows/` WinUI 3 C# can render a
desktop app that calls Rust-defined APIs.

D-18.111. Frontend boundary violation: Kotlin shared business logic used by a
backend worker.

D-18.112. Frontend boundary violation: C# service daemon supporting a Windows
desktop bundle.

D-18.113. Frontend boundary violation: Swift script used to generate backend
config.

D-18.114. Frontend boundary violation: TypeScript app logic under a µservice
runtime path.

D-18.115. Repository scan target: `*.py`.

D-18.116. Repository scan target: `*.js`.

D-18.117. Repository scan target: `*.ts`.

D-18.118. Repository scan target: `*.tsx`.

D-18.119. Repository scan target: `*.rb`.

D-18.120. Repository scan target: `*.pl`.

D-18.121. Repository scan target: `*.php`.

D-18.122. Repository scan target: `*.java`.

D-18.123. Repository scan target: `*.scala`.

D-18.124. Repository scan target: `*.groovy`.

D-18.125. Repository scan target: `*.go`.

D-18.126. Repository scan target: `*.fs`.

D-18.127. Repository scan target: `*.fsx`.

D-18.128. Repository scan target: `*.cs` outside `frontend/windows/`.

D-18.129. Repository scan target: `*.kt` outside `frontend/android/` and
approved frontend shared paths.

D-18.130. Repository scan target: `*.swift` outside `frontend/ios/` and
`frontend/macos/`.

D-18.131. Repository scan target: `Makefile` when it contains backend build
logic rather than wrapper targets with approved Cargo/OpenTofu invocations.

D-18.132. Repository scan target: package manifests such as `package.json`,
`pyproject.toml`, `Gemfile`, `go.mod`, `pom.xml`, and `build.gradle` in backend
paths.

D-18.133. Forbidden pattern: "temporary Python" for migration, validation, or
data cleanup.

D-18.134. Forbidden pattern: "just a Node script" for OpenAPI or SDK generation.

D-18.135. Forbidden pattern: "Go for cloud tooling" inside Oyatie-owned
backend code.

D-18.136. Forbidden pattern: "Java for enterprise compatibility" inside
µservice runtime code.

D-18.137. Forbidden pattern: "Gradle for shared mobile/backend logic" crossing
the frontend boundary.

D-18.138. Forbidden pattern: checking in vendored non-Rust runtime code because
an upstream CLI is convenient.

D-18.139. Forbidden pattern: relying on language-specific package managers
other than Cargo for backend release.

D-18.140. Forbidden pattern: runtime behavior hidden inside YAML templating.

D-18.141. Forbidden pattern: Bash bootstrap that mutates production state.

D-18.142. Forbidden pattern: Markdown instructing operators to run forbidden
language scripts.

D-18.143. Audit severity P0 applies when a P0-priority µservice contains
forbidden backend language code, forbidden build invocation, or missing
exception ADR.

D-18.144. Audit severity P1 applies when any other in-scope µservice contains
the same violation.

D-18.145. Audit severity P2 applies when the implementation is Rust-strict but
the manifest, brief, README, or build docs fail to state the language boundary.

D-18.146. Audit agents must cite
`feedback_rust_strict_only_no_python_2026_05_20.md` when applying this section.

D-18.147. Audit agents must cite ADR-0324 when forbidding agent-authored
script generation.

D-18.148. Audit agents must cite ADR-0211 when explaining Rust-primary
portability and in-house stack preference.

D-18.149. The stop condition for this constraint is a Rust-only backend scan,
allowed non-Rust extensions classified, frontend language paths scoped, build
invocation canonicalized, and every exception tied to a per-µservice ADR.

D-18.150. A Wave 2 audit that permits Python, JavaScript application logic, or
other forbidden backend languages must be reopened because it breaks OS,
supply-chain, observability, and build reproducibility claims.

### D-19: OCI Always Free Maximization

D-19.1. OCI Always Free is a sub-profile of the `guest-on-oci` deployment
context.

D-19.2. OCI Always Free is not a parallel cloud strategy.

D-19.3. OCI Always Free is not permission to make the architecture OCI-specific.

D-19.4. OCI Always Free applies to tenant classes `demo`, `sandbox`, `trial`,
and `dev` unless a service manifest marks the sub-profile unsupported.

D-19.5. OCI Bronze tier means Always Free when the deployment context is
`guest-on-oci`.

D-19.6. OCI Silver, Gold, and Platinum mean paid OCI capacity, not cross-cloud
fallback.

D-19.7. Cross-cloud fallback is forbidden for the OCI Always Free sub-profile.

D-19.8. Provider-agnostic design remains mandatory everywhere outside this
sub-profile.

D-19.9. Compute resource: Ampere A1 ARM Always Free capacity totals 4 OCPU and
24 GB RAM.

D-19.10. Compute resource: the 4 OCPU and 24 GB RAM can be partitioned across
one or more A1 instances according to module policy.

D-19.11. Compute resource: two AMD E2.1.Micro instances may exist for legacy
or auxiliary use.

D-19.12. Compute policy: Ampere A1 arm64 is the default capacity target for
serious demo and sandbox workloads.

D-19.13. Compute policy: AMD micro capacity is not a substitute for arm64
readiness.

D-19.14. Compute policy: µservices must fit Bronze capacity or mark Bronze
unsupported with a concrete capacity reason.

D-19.15. Storage resource: 200 GB block volume budget across instances and boot
volumes.

D-19.16. Storage resource: 10 GB Object Storage standard tier.

D-19.17. Storage resource: 10 GB Archive Storage.

D-19.18. Storage resource: Storage Gateway capacity may be used only when it
fits the published Always Free constraints.

D-19.19. Storage policy: Object Storage hosts OpenTofu state where the context
requires OCI state storage.

D-19.20. Storage policy: block volume use must be budgeted per tenant and per
cell before apply.

D-19.21. Storage policy: object storage use must be metered even when cost is
zero.

D-19.22. Database resource: two Autonomous Databases are available in the
Always Free profile.

D-19.23. Database resource: each Autonomous Database has a 20 GB storage budget.

D-19.24. Database policy: one database may be OLTP-oriented and one OLAP-oriented
when the tenant profile needs both.

D-19.25. Database policy: database use must be declared by the µservice module
instead of silently consuming a shared database.

D-19.26. Database policy: a service that exceeds two Autonomous Databases or
20 GB per database cannot claim Bronze OCI Always Free support.

D-19.27. Networking resource: one VCN with subnets and gateways can back the
Always Free tenant profile.

D-19.28. Networking resource: one Always Free Load Balancer is available with
10 Mbps throughput.

D-19.29. Networking resource: outbound data transfer budget is 10 TB per month.

D-19.30. Networking policy: alert at 8 TB egress for demo, sandbox, trial, and
dev tenants.

D-19.31. Networking policy: load-balancer saturation is a Bronze capacity event,
not a reason to spill traffic into another cloud.

D-19.32. Networking policy: API Gateway and WAF resources are used where they
fit Always Free limits and service exposure requires them.

D-19.33. Security resource: OCI Vault Always Free limits include vault and key
capacity suitable for demo and sandbox key management.

D-19.34. Security resource: OCI IAM is free and can back guest context adapter
bindings.

D-19.35. Security resource: OCI Audit is free and provides provider-side audit
logs.

D-19.36. Security policy: `cloud-kms` remains the Oyatie key authority even
when OCI Vault stores backing keys.

D-19.37. Security policy: `cloud-iam` remains the Oyatie identity authority even
when OCI IAM backs dynamic groups or policies.

D-19.38. Security policy: audit events must be imported into Oyatie audit-chain
instead of leaving provider audit as the only evidence.

D-19.39. Observability resource: OCI Logging includes Always Free ingestion
capacity.

D-19.40. Observability resource: OCI Monitoring metrics can support provider
health evidence.

D-19.41. Observability resource: OCI Notifications free tier can support alert
fanout within limits.

D-19.42. Observability policy: Oyatie `observability` owns golden signals,
tenant views, SLO burn, and support evidence.

D-19.43. Observability policy: OCI Monitoring is an input, not the canonical
SLO authority.

D-19.44. Observability policy: Bronze tenants still emit the same traces,
metrics, logs, and audit events as paid tiers.

D-19.45. Streaming resource: OCI Streaming Always Free can provide limited
partition and throughput capacity.

D-19.46. Streaming policy: event-heavy µservices must declare when Bronze
capacity is insufficient.

D-19.47. Streaming policy: `cloud-billing` and audit-chain events still emit
even when no paid stream is used.

D-19.48. Functions resource: OCI Functions Always Free invocation and compute
budget can support narrow bootstrap or integration tasks.

D-19.49. Functions policy: Functions cannot become a hidden non-Rust backend
logic escape hatch.

D-19.50. Functions policy: any use must be declared in OpenTofu and tied to
Rust-owned behavior or approved provider integration.

D-19.51. API Gateway resource: OCI API Gateway free call budget can support
demo ingress and trial APIs.

D-19.52. API Gateway policy: `api-gateway` remains the Oyatie API product
surface.

D-19.53. API Gateway policy: OCI API Gateway is an ingress backing primitive,
not a replacement for Oyatie routing, auth, rate limit, or audit semantics.

D-19.54. WAF resource: OCI WAF free request budget can support demo exposure
where appropriate.

D-19.55. WAF policy: `cloud-network` and `api-gateway` own security posture and
surface WAF decisions as Oyatie policy.

D-19.56. Email Delivery resource: the free allotment is small and suitable only
for constrained demo or system notification cases.

D-19.57. Email Delivery policy: `mail` and `comms-email` must not claim full
email production readiness from the Always Free email allotment.

D-19.58. Bastion resource: OCI Bastion may exist but must not become a manual
setup dependency.

D-19.59. Health Checks resource: OCI Health Checks may feed availability
signals.

D-19.60. Resource Manager resource: OCI Resource Manager may be referenced only
if it does not replace Oyatie's OpenTofu and `cloud-iac` control surface.

D-19.61. Per-µservice module contract: every µservice that supports OCI Bronze
has `microservices/<name>/iac/oci-guest/always-free/`.

D-19.62. Always Free module required file: `main.tf`.

D-19.63. Always Free module required file: `variables.tf`.

D-19.64. Always Free module required file: `outputs.tf`.

D-19.65. Always Free module required file: `versions.tf`.

D-19.66. Always Free module required file: `README.md`.

D-19.67. Always Free module required input: `tenant_class`.

D-19.68. Always Free module required input: `always_free_budget_profile`.

D-19.69. Always Free module required input: `oci_region`.

D-19.70. Always Free module required input: `tenant_id`.

D-19.71. Always Free module required output: `compute_budget_used`.

D-19.72. Always Free module required output: `storage_budget_used`.

D-19.73. Always Free module required output: `database_budget_used`.

D-19.74. Always Free module required output: `egress_budget_alert_threshold`.

D-19.75. Always Free module required output: `billing_meter_ids`.

D-19.76. Always Free module required output: `state_backend_ref`.

D-19.77. `cloud-iac` selects the Always Free module when `deployment_context` is
`guest-on-oci` and tenant class is demo, sandbox, trial, or dev.

D-19.78. `cloud-iac` must fail plan when requested capacity exceeds Always Free
budgets and the tenant class still requires Bronze.

D-19.79. `cloud-iac` must produce a plan summary with OCPU, RAM, block, object,
database, egress, LB, stream, function, and API Gateway budget impact.

D-19.80. `cloud-billing` emits per-tenant cost-attribution events even when the
charge amount is zero.

D-19.81. Cost event field: `tenant_id`.

D-19.82. Cost event field: `deployment_context`.

D-19.83. Cost event field: `capability_tier`.

D-19.84. Cost event field: `oci_free_resource_class`.

D-19.85. Cost event field: `quantity`.

D-19.86. Cost event field: `unit_cost`.

D-19.87. Cost event field: `effective_cost`.

D-19.88. Cost event field: `budget_remaining`.

D-19.89. Cost event field: `paid_upgrade_trigger`.

D-19.90. Zero-cost events matter because demos, trials, and dev tenants still
consume capacity and forecast conversion risk.

D-19.91. Zero-cost events matter because support, quota, abuse, and SLO
decisions require usage evidence.

D-19.92. Zero-cost events matter because trial-to-paid conversion needs a
credible before/after cost model.

D-19.93. State backend for Always Free OCI uses OCI Object Storage within the
Always Free object-storage budget.

D-19.94. State locking uses Autonomous Database or an approved OCI-native lock
path within Always Free constraints.

D-19.95. State backend must not use AWS, Terraform Cloud, GitHub, or a local
operator laptop.

D-19.96. Always Free matters for real demo tenants.

D-19.97. Always Free matters for sandbox tenants.

D-19.98. Always Free matters for trial-to-paid funnel.

D-19.99. Always Free matters for low-cost regional presence.

D-19.100. Always Free matters for per-employee dev boxes.

D-19.101. Always Free matters for field testing arm64 and Oracle Linux paths.

D-19.102. Always Free matters because it offers perpetual capacity rather than
a 12-month trial clock.

D-19.103. There is no equivalent AWS Free Tier maximization doctrine.

D-19.104. AWS Free Tier is time-limited and must not drive a parallel
architecture branch.

D-19.105. There is no equivalent GCP free-tier maximization doctrine in this
ADR.

D-19.106. On-prem and colo have no cloud free-tier equivalent.

D-19.107. Provider-agnostic modules must remain provider-agnostic outside the
OCI Always Free sub-profile.

D-19.108. OCI-specific code must stay in the OCI context module, OCI provider
adapter, or service-local ADR-approved integration boundary.

D-19.109. Forbidden pattern: using Always Free as a reason to hardcode OCI in
business logic.

D-19.110. Forbidden pattern: spillover to AWS or Oyatie public paid capacity
for a Bronze OCI tenant.

D-19.111. Forbidden pattern: no cost events because the cost is zero.

D-19.112. Forbidden pattern: no capacity tests because the tenant is only demo.

D-19.113. Forbidden pattern: paid OCI resources silently added to Bronze.

D-19.114. Forbidden pattern: Always Free README without an OpenTofu module.

D-19.115. Forbidden pattern: Object Storage state backend exceeding the 10 GB
budget without a paid-tier transition.

D-19.116. Forbidden pattern: relying on AMD micro instances for workloads that
must prove Ampere arm64 readiness.

D-19.117. Audit severity P0 applies when a P0-priority µservice claims OCI
Bronze support but violates Always Free capacity, state, cost-event, or
cross-cloud rules.

D-19.118. Audit severity P1 applies when any other in-scope µservice violates
the same OCI Always Free rules.

D-19.119. Audit severity P2 applies when modules and behavior are coherent but
the service docs, tier matrix, budget outputs, or cost-event docs are missing.

D-19.120. Audit agents must cite
`feedback_oci_always_free_maximization_2026_05_20.md` when applying this
section.

D-19.121. Audit agents must cite ADR-0215 to state that Always Free is a
sub-profile inside multi-context deployment.

D-19.122. Audit agents must cite ADR-0218 to state that tenant class selects the
Always Free profile.

D-19.123. Audit agents must cite ADR-0316 when mapping OCI Bronze to Always Free
and paid OCI to Silver, Gold, and Platinum.

D-19.124. The stop condition for this constraint is an `iac/oci-guest/always-free/`
module, budget outputs, zero-cost billing events, OCI Object Storage state
backend, no cross-cloud spillover, and docs explaining Bronze limits.

D-19.125. A Wave 2 audit that treats OCI as just another paid cloud context
misses the demo, sandbox, trial, regional, and dev-box strategy this section
exists to preserve.

### D-20: How Audit Agents Apply These 5 Constraints

D-20.1. This section extends the ownership-coherence audit from five dimensions
to nine dimensions for Wave 2 through Wave 13.

D-20.2. Dimensions 1 through 5 remain exactly as defined in D-4.

D-20.3. Dimension 1 remains internal coherence.

D-20.4. Dimension 2 remains outbound cross-references.

D-20.5. Dimension 3 remains substance bar.

D-20.6. Dimension 4 remains canonical-direction alignment.

D-20.7. Dimension 5 remains industry-counterpart parity.

D-20.8. Dimension 6 is multi-context deployment.

D-20.9. Dimension 7 is OpenTofu IaC.

D-20.10. Dimension 8 is OS support.

D-20.11. Dimension 9 is Rust-strict language policy.

D-20.12. Dimension 6 asks whether the µservice declares all six deployment
contexts or documents why a context is N/A.

D-20.13. Dimension 6 requires context ids `oyatie-public-cloud`,
`guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and
`oyatie-as-cloud-provider`.

D-20.14. Dimension 6 requires `iac/oyatie-public-cloud/` or an N/A reason.

D-20.15. Dimension 6 requires `iac/guest-on-aws/` or an N/A reason.

D-20.16. Dimension 6 requires `iac/oci-guest/` or an N/A reason.

D-20.17. Dimension 6 requires `iac/on-prem/` or an N/A reason.

D-20.18. Dimension 6 requires `iac/colo/` or an N/A reason.

D-20.19. Dimension 6 requires `iac/oyatie-iaas/` or an N/A reason.

D-20.20. Dimension 6 requires the audit agent to classify cloud-* µservices as
Oyatie's own IaaS surface, not wrappers.

D-20.21. Dimension 6 requires network seam evidence.

D-20.22. Dimension 6 requires IAM seam evidence.

D-20.23. Dimension 6 requires observability seam evidence.

D-20.24. Dimension 6 requires billing seam evidence.

D-20.25. Dimension 6 requires tenant onboarding evidence using OpenTofu for
every supported context.

D-20.26. Dimension 6 finding example P0: HR/Payroll claims guest-on-OCI support
but has no OCI context module, no tenant selection path, and no context-labeled
audit events.

D-20.27. Dimension 6 finding example P1: `messenger` claims on-prem support but
documents only public-cloud push notification behavior.

D-20.28. Dimension 6 finding example P2: `cloud-iam` supports all contexts in
module shape but its README omits the billing and observability seams.

D-20.29. Dimension 7 asks whether every supported context is provisioned by
OpenTofu.

D-20.30. Dimension 7 forbids Terraform as engine or binary.

D-20.31. Dimension 7 forbids Pulumi as primary infrastructure engine.

D-20.32. Dimension 7 forbids CloudFormation as primary infrastructure engine.

D-20.33. Dimension 7 forbids ARM templates as primary infrastructure engine.

D-20.34. Dimension 7 requires pinned OpenTofu versions.

D-20.35. Dimension 7 requires pinned provider versions.

D-20.36. Dimension 7 requires `main.tf`.

D-20.37. Dimension 7 requires `variables.tf`.

D-20.38. Dimension 7 requires `outputs.tf`.

D-20.39. Dimension 7 requires `versions.tf`.

D-20.40. Dimension 7 requires `README.md`.

D-20.41. Dimension 7 requires sigstore and cosign module signing evidence.

D-20.42. Dimension 7 requires approved state backend per context.

D-20.43. Dimension 7 requires `cloud-iac` orchestration.

D-20.44. Dimension 7 forbids `null_resource`.

D-20.45. Dimension 7 forbids `local-exec`.

D-20.46. Dimension 7 forbids SSH provisioners.

D-20.47. Dimension 7 forbids hand-edited tfstate.

D-20.48. Dimension 7 forbids unsigned modules.

D-20.49. Dimension 7 requires tenant onboarding text to use `tofu init → tofu
plan → tofu apply`.

D-20.50. Dimension 7 finding example P0: ERP tenant provisioning uses a
Terraform Cloud workspace and manual state edits.

D-20.51. Dimension 7 finding example P1: `calendar` has OpenTofu for public
cloud but README instructs AWS console setup for guest-on-AWS.

D-20.52. Dimension 7 finding example P2: `cloud-network` modules are present
but signing evidence is not documented.

D-20.53. Dimension 8 asks whether the µservice declares `supported-oses.json`.

D-20.54. Dimension 8 requires all Tier 1 OSes or service-local N/A entries.

D-20.55. Dimension 8 requires Talos.

D-20.56. Dimension 8 requires RHEL 9.x+.

D-20.57. Dimension 8 requires Oracle Linux 9.x+.

D-20.58. Dimension 8 requires SLES 15 SP6+.

D-20.59. Dimension 8 requires Ubuntu 24.04 LTS+.

D-20.60. Dimension 8 requires Debian 13+.

D-20.61. Dimension 8 requires Rocky 9.x+.

D-20.62. Dimension 8 requires AlmaLinux 9.x+.

D-20.63. Dimension 8 requires CentOS Stream 10+.

D-20.64. Dimension 8 requires Amazon Linux 2023+.

D-20.65. Dimension 8 requires Flatcar.

D-20.66. Dimension 8 requires Photon 5.x+.

D-20.67. Dimension 8 requires macOS Apple Silicon M5+ only when macOS support
or local tooling is claimed.

D-20.68. Dimension 8 requires Tier 2 ppc64le and s390x to be marked test-only.

D-20.69. Dimension 8 requires out-of-scope explicit entries for Intel macOS,
pre-M5 Apple Silicon, FreeBSD, OpenBSD, Windows Server, and Solaris.

D-20.70. Dimension 8 requires architecture matrix `linux/amd64`, `linux/arm64`,
`darwin/arm64-m5+`, `linux/ppc64le-test-only`, and
`linux/s390x-test-only`.

D-20.71. Dimension 8 requires RPM, DEB, container image, Talos extension,
Flatcar extension, macOS `.pkg`, and Homebrew mapping where applicable.

D-20.72. Dimension 8 requires Tier 1 CI lanes to be blocking.

D-20.73. Dimension 8 requires Tier 2 CI lanes to be soft-gates.

D-20.74. Dimension 8 requires no Python interpreter dependency.

D-20.75. Dimension 8 requires Rust statically linked or explicitly portable
binary posture.

D-20.76. Dimension 8 finding example P0: CRM ships Python helper scripts needed
for RHEL and Ubuntu install.

D-20.77. Dimension 8 finding example P1: `mail` claims Oracle Linux support but
has no arm64/UEK lane.

D-20.78. Dimension 8 finding example P2: `developer-sdk` builds on macOS M5+
but omits the explicit Intel/pre-M5 exclusion.

D-20.79. Dimension 9 asks whether backend, µservice, scripting, validation,
codegen, and durable CI behavior are Rust-only.

D-20.80. Dimension 9 allows `.tf`.

D-20.81. Dimension 9 allows `.cedar`.

D-20.82. Dimension 9 allows `.yaml` and `.json` for configuration, contracts,
manifests, and specs.

D-20.83. Dimension 9 allows `.proto`.

D-20.84. Dimension 9 allows `openapi.yaml`.

D-20.85. Dimension 9 allows `asyncapi.yaml`.

D-20.86. Dimension 9 allows `.openslo.yaml`.

D-20.87. Dimension 9 allows `.sql` migrations.

D-20.88. Dimension 9 allows `.md` docs.

D-20.89. Dimension 9 allows Swift only under `frontend/ios/` and
`frontend/macos/`.

D-20.90. Dimension 9 allows Kotlin only under `frontend/android/` or approved
frontend shared code with ADR.

D-20.91. Dimension 9 allows WinUI 3 C#/.NET only under `frontend/windows/`.

D-20.91a. Dimension 9 allows Leptos under `frontend/web/` and
`microservices/<name>/frontend/web/`, authored in Rust, compiled to
WebAssembly, with mandatory server-side rendering plus SELECTIVE WebAssembly
hydration scoped to interactive component or island boundaries (static
sections SSR-only with zero WASM payload, hydration opt-in per island);
the generated `wasm-bindgen` JavaScript trampoline is allowlisted by
provenance only; whole-page hydration when any portion is static, or
CSR-only initial documents, are findings.

D-20.92. Dimension 9 forbids Python.

D-20.93. Dimension 9 forbids JavaScript application logic.

D-20.94. Dimension 9 forbids TypeScript application logic.

D-20.95. Dimension 9 forbids Ruby.

D-20.96. Dimension 9 forbids Perl.

D-20.97. Dimension 9 forbids PHP.

D-20.98. Dimension 9 forbids Java.

D-20.99. Dimension 9 forbids Scala.

D-20.100. Dimension 9 forbids Groovy.

D-20.101. Dimension 9 forbids Go.

D-20.102. Dimension 9 forbids F#.

D-20.103. Dimension 9 forbids C++ except ADR-approved FFI shim.

D-20.104. Dimension 9 requires canonical build invocation `cargo build
--workspace --release --all-features --locked`.

D-20.105. Dimension 9 forbids backend `make`, `npm run`, `python setup.py`, and
`gradle` release paths.

D-20.106. Dimension 9 requires each exception to cite a per-µservice ADR.

D-20.107. Dimension 9 requires audit agents to distinguish generated SDK
clients from authored application logic.

D-20.108. Dimension 9 finding example P0: HR/Payroll contains Python payroll
calculation scripts or Java backend services.

D-20.109. Dimension 9 finding example P1: `analytics` contains Go data movers
without exception ADR.

D-20.110. Dimension 9 finding example P2: `frontend/android/` Kotlin is
correctly scoped but README does not state backend Rust dependency boundaries.

D-20.111. Severity P0 means a constraint is violated and the violating service
ships in a P0-priority µservice.

D-20.112. P0-priority µservices include HR/Payroll family services.

D-20.113. P0-priority µservices include ERP family services.

D-20.114. P0-priority µservices include CRM family services.

D-20.115. P0 applies when the violation would let downstream implementation
build the wrong tenant, deployment, OS, IaC, or language posture.

D-20.116. Severity P1 means a constraint is violated in any other in-scope
µservice.

D-20.117. P1 applies even when the service is not in the Big 8 if the violation
would make its deployment, support, or implementation claim false.

D-20.118. Severity P2 means documentation gap with no current code or module
violation found.

D-20.119. P2 applies when the µservice behavior appears compliant but the
manifest, README, brief, ADR, CI lane name, or audit doc omits the constraint.

D-20.120. P2 is not a free pass; it enters the Wave 14 backlog.

D-20.121. P3 remains available for cosmetic issues unrelated to these five
constraints.

D-20.122. Decision tree step 1: identify the microservice and canonical phase.

D-20.123. Decision tree step 2: determine whether the µservice is P0-priority
HR/Payroll, ERP, or CRM.

D-20.124. Decision tree step 3: inspect the microservice path before writing
findings.

D-20.125. Decision tree step 4: inspect any service manifest, PRD,
ARCHITECTURE, README, iac directory, supported-OS manifest, build docs, and
frontend directory.

D-20.126. Decision tree step 5: evaluate Dimension 6.

D-20.127. Decision tree step 6: evaluate Dimension 7.

D-20.128. Decision tree step 7: evaluate Dimension 8.

D-20.129. Decision tree step 8: evaluate Dimension 9.

D-20.130. Decision tree step 9: if a violation exists in a P0-priority
µservice, classify P0.

D-20.131. Decision tree step 10: if a violation exists in a non-P0 in-scope
µservice, classify P1.

D-20.132. Decision tree step 11: if no violation is found but documentation is
missing, classify P2.

D-20.133. Decision tree step 12: if the µservice legitimately marks a context,
OS, or frontend path N/A with complete rationale, do not create a violation for
that N/A row.

D-20.134. Decision tree step 13: if N/A rationale is generic, classify P2 at
minimum and P1 when the absence hides a required support path.

D-20.135. Decision tree step 14: if a forbidden language file is generated SDK
output, require provenance before clearing it.

D-20.136. Decision tree step 15: if a forbidden IaC pattern appears only in
retired archived material, record provenance and do not inflate severity unless
current docs point agents to it.

D-20.137. Decision tree step 16: if a support claim appears in marketing prose
but not manifests, audit the claim as live because downstream agents may rely on
it.

D-20.138. Decision tree step 17: if code is absent but docs prescribe a
violating future path, classify according to the prescribed path because briefs
are build inputs.

D-20.139. Decision tree step 18: if an artifact cannot be inspected, HALT-CLEANLY
or record blocked evidence rather than assuming pass.

D-20.140. Cross-reference for Dimension 6: this ADR D-15 and
`feedback_multi_context_provider_agnostic_2026_05_20.md`.

D-20.141. Cross-reference for Dimension 7: this ADR D-16 and
`feedback_zero_handroll_opentofu_only_2026_05_20.md`.

D-20.142. Cross-reference for Dimension 8: this ADR D-17 and
`feedback_os_support_matrix_2026_05_20.md`.

D-20.143. Cross-reference for Dimension 9: this ADR D-18 and
`feedback_rust_strict_only_no_python_2026_05_20.md`.

D-20.144. Cross-reference for OCI Bronze sub-profile: this ADR D-19 and
`feedback_oci_always_free_maximization_2026_05_20.md`.

D-20.145. Cross-reference for tenant deployment context: ADR-0218.

D-20.146. Cross-reference for multi-context platform doctrine: ADR-0215.

D-20.147. Cross-reference for module signing: ADR-0039.

D-20.148. Cross-reference for anti-script authoring: ADR-0324.

D-20.149. Cross-reference for capability tier mapping: ADR-0316.

D-20.150. Every audit deliverable must include a `New Constraint Dimensions`
section naming pass, finding, or N/A for Dimensions 6 through 9.

D-20.151. Every feature-parity matrix must avoid treating AWS, OCI, or provider
adapters as product counterparts when the actual surface is Oyatie cloud-*.

D-20.152. Every performance benchmark doc must state OS, architecture,
deployment context, and tenant class.

D-20.153. Every capability-tier delta doc must state whether OCI Bronze maps to
Always Free for that µservice.

D-20.154. Every coherence audit must include grep or manifest evidence for
forbidden backend languages.

D-20.155. Every coherence audit must include OpenTofu module evidence or N/A
evidence per supported context.

D-20.156. Every coherence audit must include supported OS manifest evidence or
missing-manifest finding.

D-20.157. Every coherence audit must include tenant onboarding evidence for
supported contexts.

D-20.158. The audit stop condition is nine dimensions evaluated, severity
assigned with this decision tree, memory-file anchor cited, and any P0/P1/P2
rows prepared for Wave 14 aggregation.

D-20.159. An audit agent that evaluates only the original five dimensions must
be treated as incomplete after this amendment lands.

D-20.160. An orchestrator that dispatches Wave 2 without these five constraints
in the brief template risks producing non-enforcing reports and must relaunch
the affected audits.

## E. Implementation Footprint

E.1. This ADR creates one normative decision file:
`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`.

E.2. No code crate is created by this ADR.

E.3. No schema is created by this ADR.

E.4. Future implementation may add machine-readable projections of this doctrine
to `specs/master-plan-sequencing.json` or a successor control-surface spec.

E.5. Future implementation may add `oya-governance-brief-anchor-header` to check
five-citation headers.

E.6. Future implementation may add `oya-governance-wave-batch-ceiling` to enforce
the eight-Codex batch ceiling.

E.7. Future implementation may add `oya-governance-microservice-coherence-audit`
to check the four audit deliverables and five-dimension verdicts.

E.8. Future implementation may add `oya-governance-top3-union-parity` to check
feature-parity matrix shape.

E.9. Future implementation may add `oya-governance-foundry-absorption` to detect
standalone foundry runtime references after Wave 15I.

E.10. Future implementation must use the neutral strict-zero residue gate required by ADR-0619;
source-derived gate names are forbidden.

E.11. The ADR does not require those lanes to exist before Wave 1 can use the
human-readable doctrine.

E.12. Until lanes exist, leaders verify manually using D-10.

E.13. Once lanes exist, manual verification remains required for substance
sampling because line count and shape checks are not sufficient.

## F. Migration and Rollout

F.1. Step 1: land ADR-0328.

F.2. Step 2: use ADR-0328 in the next realignment dispatch brief.

F.3. Step 3: ensure every Wave 2 through Wave 13 brief includes a five-citation
header.

F.4. Step 4: ensure every Wave 2 through Wave 13 brief names phase, batch, agent
class, in-scope files, out-of-scope files, and stop condition.

F.5. Step 5: collect the four audit deliverables per microservice.

F.6. Step 6: apply the D-10 verification SLA to each landing.

F.7. Step 7: aggregate findings in Wave 14.

F.8. Step 8: remediate by Wave 15 sub-wave structure.

F.9. Step 9: promote ADR-0328 under ADR-0327 once evidence is complete.

F.10. Rollback path: if this ADR is found wrong before promotion, keep it
Proposed, author an amendment, and update briefs to cite the amendment.

F.11. Rollback path: if a batch launches with the wrong anchor set, stop the
batch, preserve checkpoints, and relaunch only the affected slices.

F.12. Rollback path: if an audit deliverable is found template-stamped, mark the
landing REVISE, keep the claim open or reopen a remediation claim, and reauthor
bespoke content from the five anchors.

F.13. Rollback path: if Foundry absorption guidance causes capability loss,
block Wave 15I and author a narrow amendment that preserves the capability
mapping before retiring paths.

## G. Verification Contract

G.1. The file exists at the requested ADR path.

G.2. The line count must be at least 800 lines.

G.3. The frontmatter must include id, title, status, date, related_adrs, and
decision_owner as requested.

G.4. The body must cite the five canonical anchors.

G.5. The body must define D-1 through D-14.

G.6. The body must list the 19 Phase 0 services by name.

G.7. The body must list the 13 Phase 1 services by name and explicitly exclude
foundry.

G.8. The body must list the 6 Phase 2 services by name and explain foundry
absorption.

G.9. The body must list the 19 Phase 3 services by name.

G.10. The body must state the Phase 4 distribution and B2B/ERP sequence.

G.11. The body must state the Big 8 sub-sequence with HR/Payroll first, ERP
second, CRM third, and the remaining default order.

G.12. The body must define at least four agent-class anchor sets.

G.13. The body defines ten agent-class anchor sets.

G.14. The body must define the five-dimension ownership-coherence audit protocol.

G.15. The body must define the top-3 union-coverage parity bar.

G.16. The body must define the four audit deliverables.

G.17. The body must define batch grouping by canonical phase.

G.18. The body must define Wave 14 aggregation.

G.19. The body must define Wave 15+ remediation sub-waves.

G.20. The body must define the verification SLA.

G.21. The body must define the brief format convention.

G.22. The body must define Foundry absorption and retired external agent harness-drop.

G.23. The body must define the ADR-0321 in-scope universe.

G.24. The body must define Codex-only dispatch.

G.25. Oya VCS verify, done, and promote evidence must include `adr_lines:X`
where X is the measured line count.

## H. References

H.1. `/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md` supplies the realignment objective, sequence, audit-wave grouping, top-3 parity bar, verification SLA, and Codex-only directive.

H.2. `/Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md` supplies the unified substrate doctrine and the rejection of product-island architecture.

H.3. `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md` Section 1.1 supplies the hyperscaler-grade rigor sub-test used by this ADR.

H.4. `/Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md` supplies blocker-class substance doctrine and anti-template-stamping expectations.

H.5. `/Users/jasonlee/oyatie/docs/decisions/ADR-0327-wave-3-completion-criteria-and-promotion-gates.md` supplies the promotion-gate model and Proposed-to-Accepted discipline.

H.6. ADR-0244 supplies tenant scoping.

H.7. ADR-0263 supplies audit emission discipline.

H.8. ADR-0316 supplies capability-tier doctrine.

H.9. ADR-0321 supplies B2B SaaS industry-leader coverage doctrine.

H.10. ADR-0323 supplies multi-wave sequencing doctrine.

H.11. ADR-0324 supplies anti-script and anti-template doctrine.

H.12. ADR-0247 and ADR-0255-amendment supply the Foundry capability absorption
context.

H.13. ADR-0138 supplies the six-path deprecation pattern.
