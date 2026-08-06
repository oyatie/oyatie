---
doc_class: InvestorMaterial
doc_type: company_overview_deck
status: canonical_draft
date: 2026-05-20
owner: founder-office
target_audience: seed_investors, strategic_enterprise_buyers, regulated-market_partners
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Oyatie Company Overview Deck

## Slide 01 - Investor Thesis

- Oyatie is a unified enterprise operating substrate for regulated, multinational companies replacing fragmented SaaS suites.
- The company position is not "another CRM" or "another workflow app"; it is a policy-governed business system layer.
- The first commercial wedge is Tenant RBAC view plus Tenant RBAC view, matching the FD-001 delivery scope in `specs/masterplan.json`.
- The platform thesis is that CRM, ERP, HR, ITSM, collaboration, analytics, and workflow labels become capability tiers.
- The architecture thesis is that shared primitives beat duplicated product silos.
- The economic thesis is that enterprises are already funding consolidation because software spend keeps rising.
- Gartner forecasts worldwide IT spending of $6.316 trillion in 2026, with software at $1.444 trillion.
- Forrester forecasts global technology spend of $5.6 trillion in 2026.
- Gartner forecasts enterprise business applications reaching $254 billion in 2025 and $428 billion in 2029.
- Gartner reports the CRM software market reached $128 billion in 2024.
- Zylo reports average annual SaaS spend of $55.7 million and average portfolio size of 305 applications in its 2026 index.
- Oyatie's opportunity is the budget trapped in fragmented systems, not only new budget.
- Buyer pain is strongest where software fragmentation collides with compliance evidence.
- Target customers are multinational operators, regulated enterprises, and SaaS-heavy midmarket firms.
- The first commercial promise is lower governance drag per workflow.
- The second commercial promise is a single audit chain across business operations.
- The third commercial promise is capability-tier activation without service sprawl.
- The fourth commercial promise is policy-gated AI and automation with cost attribution.
- The fifth commercial promise is deployment portability across Kubernetes cells.
- The moat is not a UI feature; it is a governed substrate.
- The near-term ask model is $18.0 million seed capital.
- The 24-month use is productization, security evidence, go-to-market pilots, and compliance-pack readiness.
- The named stop condition for this deck is investor diligence readiness, not a claim of revenue traction.
- The core diligence question is whether this architecture can compress multiple enterprise categories into one governed platform.

## Slide 02 - The Problem

- Enterprise software budgets have become structurally fragmented.
- Large organizations buy CRM from Salesforce, ERP from SAP or Oracle, HCM from Workday, ITSM from ServiceNow, and collaboration from Microsoft or Google.
- Each suite ships its own object model, permission model, automation model, audit trail, and integration layer.
- The result is duplicated governance rather than compounding governance.
- Compliance teams must reconstruct evidence across many systems.
- Procurement teams must negotiate product bundles that duplicate adjacent functions.
- CIO teams must fund integration projects that do not create durable product advantage.
- RevOps teams live between CRM, CPQ, billing, support, and data warehouse systems.
- Finance teams reconcile ERP, subscription management, planning, treasury, and tax systems.
- HR teams reconcile HCM, identity, learning, payroll connectors, and workforce planning systems.
- Security teams must enforce zero trust across dozens or hundreds of vendors.
- Data teams build lineage after the fact because SaaS vendors own separate data models.
- AI adoption intensifies the problem because AI features arrive inside incumbent tools with new pricing.
- Zylo reports 78 percent of IT leaders saw unexpected charges tied to AI features or consumption pricing.
- The buyer does not need another isolated application.
- The buyer needs a governance layer that makes application boundaries less expensive.
- The buyer needs evidence generation as a default runtime behavior.
- The buyer needs per-tenant policy and quota controls that follow every workflow.
- The buyer needs portable deployment choices for regulated and sovereign workloads.
- The buyer needs a path away from vendor lock-in without losing enterprise feature depth.
- The current best-of-breed stack optimizes departmental adoption.
- The current best-of-breed stack under-optimizes audit, identity, data, and AI governance.
- Oyatie names that gap and builds directly at the substrate layer.
- The problem is budget leakage, governance drift, and duplicated product primitives.

## Slide 03 - Why Now

- The timing is favorable because the enterprise software market is large and actively repricing.
- Gartner's April 2026 forecast puts worldwide software spending at $1.444 trillion in 2026.
- Gartner's November 2024 cloud forecast put 2025 SaaS spend at $299.071 billion.
- Gartner's July 2025 enterprise business applications abstract points to a $254 billion 2025 market.
- Forrester expects more than 70 percent of 2025 to 2030 tech-spend growth to come from enterprise and government software and computer equipment investment.
- AI is changing buyer expectations from static systems of record to systems that reason and act.
- AI is also changing buyer fear because autonomous actions require policy, audit, and rollback controls.
- Hyperscalers are pushing platform consolidation through AWS Marketplace, Azure Marketplace, and Google Cloud Marketplace.
- Incumbent SaaS vendors are pushing suite consolidation through AI bundles and platform clouds.
- Customers are pushing back against bundle inflation and shelfware.
- Regulated buyers are pushing for evidence generation, data residency, and policy proof.
- Governments are increasing data-sovereignty and AI-risk scrutiny.
- Multinational companies need jurisdiction-aware cells, not single-region SaaS assumptions.
- The software budget center of gravity is moving from seat count to usage, AI consumption, and workflow execution.
- That shift favors platforms that can attribute cost per capability and per tenant.
- Oyatie's Foundry runtime, capability registry, policy engine, and audit chain are aimed at that shift.
- The platform can price by tenant, capability tier, usage, regulated pack, and deployment model.
- The current repo already expresses capability-tier doctrine in ADR-0316.
- The current repo already expresses Cedar policy coverage in ADR-0243.
- The current repo already expresses audit-chain doctrine in ADR-0003.
- The current repo has 78 microservice directories and 70 catalog-bearing microservice surfaces.
- The current repo has 61 capability-tier matrices and 575 Cedar policy files.
- The timing window is before enterprises standardize on incumbent AI governance add-ons.
- The now-or-never claim is to become the neutral operating substrate before AI makes fragmentation permanent.

## Slide 04 - Solution Summary

- Oyatie unifies enterprise workflows around shared primitives instead of isolated product modules.
- The product layer presents familiar business surfaces: CRM, tasks, mail, workflow, ontology, community, analytics, governance, and ops control.
- The substrate layer owns tenant identity, policy evaluation, audit chain, capability registry, workflow execution, evidence, quotas, and cells.
- The capability-tier layer turns product packaging into grantable bundles.
- Cedar policy fragments gate authorization, routing, retention, cost, residency, and autonomy.
- The audit chain records consequential state changes and evidence references.
- The ontology layer provides shared object types and relationship semantics.
- Workflow Engine provides state-machine and DAG orchestration.
- Foundry provides agentic implementation and capability runtime governance.
- Ops Dashboard and Control Center provide operational visibility and incident workflows.
- Regional and compliance packs adapt behavior to jurisdictions.
- Per-tenant quotas limit noisy-neighbor risk.
- Per-cell deployment limits blast radius.
- Oya VCS governs agentic development and promotion workflow.
- The same primitives serve internal platform work and customer-facing workloads.
- This makes "Oyatie as a tenant" a doctrine, not a metaphor.
- The product is sold as a unified ecosystem with governed activation.
- The economic wedge is replacing or containing spend across multiple B2B SaaS categories.
- The technical wedge is making governance unavoidable in runtime, not optional in policy documents.
- The buyer sees fewer vendors, clearer evidence, and lower integration drag.
- The investor sees a category-crossing platform with multiple expansion vectors.

## Slide 05 - First Product Wedge

- FD-001 names the first deliverable as Tenant RBAC view plus Tenant RBAC view full-depth production delivery.
- This deck treats FD-001 as the first fundable product wedge.
- Tenant RBAC view is the multinational regulated-operations wedge.
- Tenant RBAC view is the lower-friction adoption wedge.
- Both share the same substrates and capability-tier model.
- The first surfaces are core, messenger, mail, community, infra, ops-dashboard-control-center, foundry, workflow, ontology, canonical-base, and Korea localization pack.
- CRM expansion is supported by `microservices/crm`.
- Tasks expansion is supported by `microservices/tasks`.
- Payments expansion is supported by `microservices/payments`.
- Cloud billing expansion is supported by `microservices/cloud-billing` and `microservices/cloud-billing-tax`.
- Compliance expansion is supported by `microservices/compliance` and governance surfaces.
- Identity expansion is supported by `microservices/identity`.
- Audit expansion is supported by `microservices/audit-chain`.
- Policy expansion is supported by Cedar fragments and policy-engine doctrine.
- Workflow expansion is supported by `microservices/workflow-engine` and `microservices/workflow-studio`.
- Ontology expansion is supported by `microservices/ontology`.
- The wedge is not a thin minimum viable product.
- The wedge is a reference enterprise substrate slice.
- The named caution is that planning closure and production-grade claims remain gate-bound.
- Investor language must avoid saying "production complete" until gates are green.
- Investor language can say "architecture and artifact depth are designed for production-grade delivery."
- The commercial target is paid design-partner pilots beginning 2026-10-01.
- The conversion target is first contracted ARR by 2027-01-31.
- The wedge expands into vertical packs after shared substrate evidence is complete.

## Slide 06 - Product Experience

- The customer enters through a unified workspace rather than a single siloed module.
- Admins activate capability tiers per tenant, role, region, and compliance pack.
- Operators see role-based projections, not raw microservice boundaries.
- Finance users see quote, invoice, tax, settlement, treasury, and planning workflows.
- Sales users see accounts, contacts, opportunities, forecasting, CPQ, and contracts.
- Support users see cases, knowledge, community, incident, and contact-center workflows.
- HR users see identity-linked employee journeys, learning, workforce tasks, and policy approvals.
- SRE users see ops-dashboard-control-center, incident management, release evidence, and golden signals.
- Compliance users see audit-chain evidence, data lineage, retention, DSR, and pack status.
- Developers see API contracts, capability registry, SDKs, and Oya VCS workflow.
- AI agents see grantable capabilities, autonomy ceilings, cost ceilings, and Cedar decisions.
- Every consequential action can be tied to tenant, principal, policy, capability, data class, and audit event.
- The UX should feel like one product even when many microservices contribute.
- The buyer value is less about one screen and more about fewer reconciliation loops.
- The implementation value is that surfaces can evolve without forking product silos.
- The trust value is evidence from runtime operations.
- The compliance value is policy as executable control.
- The finance value is cost attribution per capability and tenant.
- The procurement value is fewer shelfware categories.
- The security value is default-deny policy posture.
- The SRE value is cell-aware blast-radius control.
- The data value is ontology-backed lineage.
- The AI value is governed action rather than unmanaged copilots.
- The platform value is composability without governance collapse.

## Slide 07 - Market Size Snapshot

- TAM model: B2B SaaS replacement and consolidation budget.
- Gartner 2026 software spending forecast: $1.444 trillion.
- Gartner 2025 SaaS public cloud forecast: $299.071 billion.
- Gartner 2025 enterprise business applications market: $254 billion.
- Gartner 2029 enterprise business applications forecast: $428 billion.
- Gartner 2024 CRM market: $128 billion.
- Forrester 2026 global technology spend forecast: $5.6 trillion.
- Zylo 2026 average annual SaaS spend per organization: $55.7 million.
- The TAM is not assumed to be all software spend.
- The modeled TAM in this deck is $900 billion across replaceable B2B SaaS and adjacent workflow software budgets.
- The modeled SAM is $110 billion among multinational and regulated buyers that value governance and compliance evidence.
- The modeled SOM is $60 million ARR reachable by year 3.
- Year 3 SOM assumes 30 enterprise tenants at $1.2 million ARR and 300 SMB tenants at $80,000 ARR.
- The enterprise tenant price is plausible against multi-suite replacement value.
- The SMB tenant price is plausible when sold as a governed business operating layer rather than a single point tool.
- The initial geographic focus is United States, South Korea, EU, Japan, Singapore, and Australia.
- The regulated vertical focus is financial services, healthcare, public sector contractors, manufacturing, logistics, and professional services.
- The horizontal focus is SaaS-heavy organizations with high audit and integration burden.
- The expansion budget includes CRM, workflow, compliance, ITSM, collaboration, analytics, and integration.
- The market-sizing document gives the detailed TAM/SAM/SOM breakdown.
- The deck claim is that the market is large enough for venture-scale return.
- The harder diligence question is execution, not market existence.
- The harder customer question is trust migration, not willingness to spend.
- The strongest wedge is where compliance evidence and software cost pain intersect.

## Slide 08 - Buyer Personas

- Persona 1: CIO of a 5,000 to 50,000 employee multinational trying to control SaaS sprawl.
- Persona 2: CFO or FinOps leader seeing AI and SaaS renewals create unpredictable spend.
- Persona 3: Chief Compliance Officer needing faster evidence packs across regions.
- Persona 4: General Counsel needing jurisdiction-specific retention and audit posture.
- Persona 5: VP RevOps trying to connect CRM, CPQ, billing, and support data.
- Persona 6: COO trying to standardize workflows across subsidiaries.
- Persona 7: CISO trying to enforce identity, policy, and data access consistently.
- Persona 8: Head of Platform Engineering trying to reduce vendor and integration complexity.
- Persona 9: Public sector contractor needing sovereign or controlled deployment models.
- Persona 10: Healthcare operator needing audit, DSR, consent, and regional pack controls.
- Persona 11: Manufacturing operator needing quality, supply chain, maintenance, and workflow integration.
- Persona 12: Professional services firm needing client matter controls and evidence trails.
- Buyer trigger 1: renewal consolidation program.
- Buyer trigger 2: failed ERP or CRM transformation.
- Buyer trigger 3: AI governance mandate.
- Buyer trigger 4: regulatory audit or breach remediation.
- Buyer trigger 5: cost reduction mandate.
- Buyer trigger 6: new geography or sovereign workload.
- Buyer trigger 7: post-merger application rationalization.
- Buyer trigger 8: cloud marketplace procurement commitment.
- Initial economic buyer is CIO plus CFO.
- Initial technical buyer is platform engineering plus security.
- Initial champion is compliance, RevOps, or FinOps depending on wedge.
- Initial pilot owner is one business function with cross-functional evidence pain.
- The sale is enterprise consultative and evidence-driven.

## Slide 09 - Traction to Date

- Current traction is artifact and architecture traction, not audited commercial traction.
- The repository contains 78 microservice directories under `microservices/`.
- The repository contains 540 catalog YAML records under `registry/catalog/`.
- The repository contains 70 microservice-local `catalog/` directories.
- The repository contains 61 capability-tier matrices.
- The repository contains 575 Cedar policy files under microservice scopes.
- The repository contains ADR-0316 for capability-tier over product fragmentation.
- The repository contains ADR-0243 for Cedar as universal gate.
- The repository contains ADR-0003 for audit-chain and evidence emission.
- The repository contains ADR-0251 for compliance pack and cell certification levels.
- The repository contains `specs/hyperscaler-architecture-invariants.json`.
- The repository contains `specs/capability-tier-schema.json`.
- The repository contains `specs/pack-overlay-schema.json`.
- The repository contains `specs/ontology-projection-schema.json`.
- The repository contains `microservices/intelligence/capability-tiers/tier-matrix.md`.
- The repository contains per-tenant resource quota standards.
- The repository contains governance around Oya VCS claim, verify, done, and promote.
- This is an unusually deep pre-product artifact base.
- The investor diligence caveat is that artifact depth must convert into running customer value.
- The next proof point is executable pilot workflow, not more prose.
- The next proof point is security and compliance evidence generated by runtime.
- The next proof point is one paid design partner.
- The next proof point is one migration path from Salesforce, SAP, Workday, ServiceNow, or Microsoft estate.
- The next proof point is one external audit-grade evidence pack.
- The deck treats current traction as build readiness evidence.
- The deck does not claim production adoption.

## Slide 10 - Business Model

- Primary model: annual subscription per tenant plus capability-tier activation.
- Secondary model: usage-based metering for AI, workflow execution, evidence generation, and integration volume.
- Tertiary model: regulated deployment premium for dedicated cells, sovereign packs, and high-assurance support.
- Services model: migration and implementation services through partners, capped to avoid becoming consulting-led.
- Marketplace model: revenue share on third-party capability packs and connector packs.
- Usage floor for AI and workflow execution: included allowance plus overage.
- Evidence-pack add-on: $60,000 to $250,000 annually depending on compliance packs.
- Dedicated cell add-on: $300,000 to $1,200,000 annually depending on region and SLO.
- Migration package: $75,000 to $500,000 one-time depending on source systems.
- Design-partner discount: up to 40 percent for first 6 enterprise customers.
- Channel margin: 15 percent for implementation partners.
- Marketplace take rate: 15 percent on third-party paid capability packs.
- Gross margin target at scale: 78 percent subscription gross margin.
- Early gross margin target: 55 percent to 65 percent while dedicated engineering supports pilots.
- Net revenue retention target: 130 percent by year 3.
- Logo retention target: 90 percent plus for enterprise tenants.
- CAC payback target: under 18 months by year 3.
- LTV/CAC target: 4.0x plus by year 3.
- Sales cycle assumption: 6 to 9 months enterprise, 60 to 120 days SMB.
- The unit-economics document gives the detailed model.

## Slide 11 - Go-To-Market

- Phase 1 runs from 2026-06-01 to 2026-09-30 and focuses on investor diligence plus design-partner recruitment.
- Phase 2 runs from 2026-10-01 to 2027-03-31 and focuses on three paid pilots.
- Phase 3 runs from 2027-04-01 to 2027-12-31 and focuses on repeatable enterprise onboarding.
- Phase 4 runs from 2028-01-01 to 2028-12-31 and focuses on regulated vertical expansion.
- ICP 1 is SaaS-heavy enterprise with more than 100 business applications.
- ICP 2 is regulated enterprise with audit evidence requests across multiple systems.
- ICP 3 is multinational midmarket company with region and compliance complexity.
- ICP 4 is AI-forward organization needing policy-controlled agentic workflows.
- Initial wedge offer: governed workflow plus evidence pack.
- Migration wedge 1: Salesforce Sales Cloud and Service Cloud replacement or containment.
- Migration wedge 2: ServiceNow ITSM and workflow containment.
- Migration wedge 3: Workday-adjacent workflow and evidence control.
- Migration wedge 4: SAP/Oracle adjacent ERP workflow and audit-chain control.
- Migration wedge 5: Microsoft 365 and Google Workspace governance overlays.
- Buyer motion starts with a concrete evidence pain, not a platform monologue.
- Sales assets include ROI model, audit evidence demo, policy decision trace, and migration checklist.
- Channel partners include boutique compliance firms, cloud consultancies, and systems integrators.
- Cloud marketplace listing follows after security baseline and legal package.
- Pricing anchors against the cost of three to five displaced or contained enterprise systems.
- Diligence proof includes live workflow, audit-chain event, Cedar policy decision, and cost attribution.
- First three logos target $300,000 to $900,000 ARR each.
- Year 2 target is $6.0 million ARR.
- Year 3 target is $60.0 million ARR in the aggressive case and $24.0 million ARR in the base case.
- The go-to-market is narrow before broad.

## Slide 12 - Competitive Landscape

- Hyperscaler competitors: AWS, Microsoft Azure, Google Cloud, Oracle Cloud, IBM Cloud.
- Suite competitors: Salesforce, SAP, Oracle, Workday, ServiceNow, Microsoft Dynamics 365.
- Vertical SaaS competitors: Veeva, nCino, Guidewire, Procore, Toast, Blackbaud, Epic-adjacent ecosystem vendors.
- Workflow competitors: Atlassian, Asana, Monday.com, Smartsheet, ClickUp.
- Integration competitors: MuleSoft, Boomi, Workato, Zapier, Tray.io.
- Data and ontology competitors: Palantir Foundry, Databricks, Snowflake, Microsoft Fabric.
- Governance competitors: ServiceNow GRC, Archer, OneTrust, Drata, Vanta, AuditBoard.
- Identity competitors: Okta, Microsoft Entra, CyberArk, Ping Identity.
- AI-agent competitors: OpenAI enterprise surfaces, Anthropic enterprise surfaces, Microsoft Copilot Studio, Salesforce Agentforce.
- Hyperscalers win on infrastructure breadth and marketplace procurement.
- Hyperscalers usually do not own end-to-end business application semantics.
- Salesforce wins on CRM ecosystem and GTM muscle.
- SAP wins on ERP depth and entrenched enterprise processes.
- Workday wins on HR and finance workflow credibility.
- ServiceNow wins on ITSM and enterprise workflow adoption.
- Palantir wins on ontology and high-touch enterprise transformation.
- Vertical SaaS wins on narrow workflow specificity.
- Oyatie's positioning is unified ecosystem plus policy-governed capability activation.
- Oyatie avoids being pure best-of-breed.
- Oyatie avoids being a monolithic suite clone.
- Oyatie's moat claim depends on shared primitives compounding across categories.
- Oyatie's weakness is execution burden and buyer trust before proof.
- The competitive-landscape document gives the full analysis.
- The defensibility question is whether primitives become more valuable as surfaces increase.

## Slide 13 - Moat Summary

- Moat factor 1: substrate primitives shared across every product surface.
- Moat factor 2: Cedar policy engine as universal gate.
- Moat factor 3: single audit chain and evidence emission.
- Moat factor 4: capability-tier doctrine instead of product fragmentation.
- Moat factor 5: 78 microservice directories with 70 catalog-bearing service surfaces.
- Moat factor 6: 575 Cedar policy files and 61 tier matrices.
- Moat factor 7: ADR-driven discipline and explicit claim gates.
- Moat factor 8: per-tenant quotas and cell-level isolation.
- Moat factor 9: ontology-backed projections and lineage.
- Moat factor 10: Oya VCS as governed autonomous development workflow.
- Moat factor 11: compliance packs and cell certification levels.
- Moat factor 12: Kubernetes-first deployment portability.
- Moat factor 13: internal-use doctrine where Oyatie itself is a tenant.
- Moat factor 14: regulated expansion surface across Korea, EU, US, Japan, Singapore, and Australia.
- Moat factor 15: agentic AI governance before enterprises standardize on ad hoc copilots.
- Competitor replication risk is real.
- Salesforce can add more Data Cloud and Agentforce controls.
- SAP can deepen Business Technology Platform.
- Microsoft can bundle Entra, Purview, Copilot, Fabric, and Dynamics.
- ServiceNow can expand Workflow Data Fabric and AI agents.
- Palantir can expand commercial application packaging.
- Oyatie's defense is architectural coherence plus focused execution.
- The moat is cumulative and weak until running customers prove it.
- The moat-and-defensibility document expands these factors.

## Slide 14 - Technology Architecture

- The architecture is flat microservices, not platform forks.
- Substrates include tenancy, identity, audit-chain, eventing, secrets, policy, ontology, workflow, observability, and cells.
- Product behavior is expressed through capability tiers and projections.
- Authorization and policy evaluation use Cedar fragments.
- Kubernetes is the default server workload runtime target.
- Cloud Hypervisor appears in tier matrices for controlled execution environments.
- Oya VCS governs claim, verify, done, and promote workflow.
- The platform supports dev, staging, and prod tenant environment tiers.
- Per-tenant resource quotas cover request rate, concurrent requests, memory, storage, and connections.
- Audit-chain doctrine uses append-only hash-chained events.
- Compliance packs define regulation-specific bundles such as HIPAA, PCI DSS, FedRAMP, EU GDPR, KR-PIPA, KR-FSS, DORA, and EU AI Act.
- Cell certification levels determine which tenants and packs a cell can host.
- Ontology projection schemas prevent product-specific object copies.
- Pack overlay schemas bind data class, jurisdiction, evidence, and retention rules.
- Capability-tier schemas bind permit sets, workflow templates, UX manifests, compliance overlays, and evidence.
- Hyperscaler invariants include cell isolation, shuffle-sharding, static stability, idempotency, and transactional outbox.
- Deployment posture includes OCI artifacts, SBOM, provenance, GitOps, OpenTofu, and multi-architecture images.
- The architecture is not cheap to build.
- The architecture is designed to reduce future category-specific duplication.
- The architecture can support self-hosted, managed, and sovereign deployment models.
- The architecture aligns with investor diligence around platform depth.
- The architecture must still prove runtime performance and customer migrations.
- The deepest technical risk is integration complexity.
- The deepest product risk is packaging a broad substrate into a crisp first buyer outcome.

## Slide 15 - Financial Model Snapshot

- Seed raise ask: $18.0 million.
- Post-money target range: $72.0 million to $90.0 million, subject to lead terms.
- Runway target: 24 months.
- Year 1 ARR target by 2027-05-31: $1.8 million.
- Year 2 ARR target by 2028-05-31: $6.0 million base case.
- Year 3 ARR target by 2029-05-31: $24.0 million base case.
- Year 3 upside ARR target by 2029-05-31: $60.0 million.
- Year 1 gross margin target: 55 percent.
- Year 2 gross margin target: 68 percent.
- Year 3 gross margin target: 78 percent.
- Engineering headcount target after seed: 28 full-time equivalents.
- Security and compliance headcount target after seed: 5 full-time equivalents.
- GTM headcount target after seed: 8 full-time equivalents.
- Customer engineering target after seed: 6 full-time equivalents.
- Cloud and test infrastructure budget: $2.4 million over 24 months.
- Compliance and legal budget: $1.6 million over 24 months.
- Founder and leadership reserve: $1.8 million over 24 months.
- Seed milestone A: demo-ready governed workflow by 2026-09-30.
- Seed milestone B: first paid design partner by 2026-12-15.
- Seed milestone C: security baseline by 2027-03-31.
- Seed milestone D: $1.8 million ARR by 2027-05-31.
- Series A readiness target: $3.0 million ARR run-rate plus 3 enterprise references by 2027-08-31.
- The model assumes no revenue is counted until signed contracts exist.
- The ask-and-use-of-funds document gives tranche-level detail.

## Slide 16 - Customer ROI

- ROI lever 1: displace duplicate SaaS seats.
- ROI lever 2: reduce integration maintenance.
- ROI lever 3: reduce audit evidence collection time.
- ROI lever 4: reduce failed workflow automation.
- ROI lever 5: reduce cloud and AI cost leakage through cost attribution.
- ROI lever 6: reduce policy exceptions through executable Cedar gates.
- ROI lever 7: reduce regional expansion friction through compliance packs.
- ROI lever 8: reduce vendor lock-in through portable deployment.
- Example enterprise baseline: $55.7 million annual SaaS spend from Zylo average.
- Example replacement target: 8 percent of spend addressable in first contract year.
- Example addressable budget: $4.456 million.
- Example direct software displacement target: $1.2 million.
- Example integration maintenance reduction: $400,000.
- Example audit evidence savings: $250,000.
- Example AI cost-control savings: $300,000.
- Example first-year modeled gross benefit: $2.15 million.
- Example first-year net benefit after Oyatie: $1.43 million.
- Example ROI: 1.99x on subscription cost before migration services.
- Regulated customers can justify higher ACV through audit and evidence savings.
- SMB customers justify lower ACV through consolidation and workflow automation.
- ROI proof must be customer-specific.
- Sales should require a baseline application and evidence-cost inventory.
- The unit-economics document defines ROI calculator assumptions.

## Slide 17 - Team and Operating Model

- Current named operating owners are founder-office, council-architecture, axis-foundry, council-security, council-privacy, ops-compliance, and ops-sre-reliability.
- Investor-facing team narrative must be updated with named humans before external distribution.
- The current repository proves role taxonomy and engineering governance, not final company staffing.
- Seed hiring pod 1: substrate engineering.
- Seed hiring pod 2: product engineering.
- Seed hiring pod 3: security and compliance.
- Seed hiring pod 4: customer engineering.
- Seed hiring pod 5: go-to-market.
- Seed hiring pod 6: finance and operations.
- First VP-level hire target: Head of Engineering by 2026-08-31.
- Second VP-level hire target: Head of Security and Compliance by 2026-10-31.
- First GTM leader target: enterprise founder-led sales lead by 2026-11-30.
- First customer engineering lead target: implementation architect by 2026-09-30.
- Advisory need 1: former CIO from regulated enterprise.
- Advisory need 2: enterprise compliance leader with SOC 2, ISO 27001, HIPAA, and GDPR evidence experience.
- Advisory need 3: ex-Salesforce or ServiceNow enterprise seller.
- Advisory need 4: cloud infrastructure leader with Kubernetes and sovereign deployment experience.
- Advisory need 5: legal counsel with data residency and AI governance depth.
- Engineering culture is ADR-driven and evidence-bound.
- Product culture should be buyer-pain-first, not architecture-first.
- GTM culture should sell a narrow wedge and land the broader platform later.
- Board construction should prioritize enterprise software operating experience.
- Hiring risk is high because the platform spans many domains.
- Mitigation is narrow milestone gates and capability ownership.
- The team story is credible only when paired with execution milestones.

## Slide 18 - Use Cases

- Use case 1: CRM replacement for regulated revenue teams.
- Use case 2: CRM containment where Salesforce remains system of record but Oyatie owns workflow and evidence.
- Use case 3: ITSM and incident workflow with audit-chain evidence.
- Use case 4: cross-region compliance evidence pack generation.
- Use case 5: policy-governed AI agent action for back-office operations.
- Use case 6: post-merger application rationalization.
- Use case 7: sovereign tenant cell for regulated subsidiary.
- Use case 8: quote-to-cash workflow across CRM, contract, billing, tax, and treasury.
- Use case 9: employee onboarding workflow across identity, tasks, mail, learning, and compliance.
- Use case 10: procurement approval with budget, policy, vendor, and audit-chain controls.
- Use case 11: data subject request cascade across microservices.
- Use case 12: FedRAMP or KR-FSS pack-gated workflow activation.
- Use case 13: workflow studio for enterprise operators.
- Use case 14: ontology-backed business graph for analytics and operations.
- Use case 15: marketplace deal-set settlement.
- Use case 16: AI cost budget enforcement at capability invocation.
- Use case 17: SaaS renewal rationalization by capability overlap.
- Use case 18: cross-product audit trail for regulators.
- Use case 19: tenant quota enforcement for noisy-neighbor controls.
- Use case 20: role-based projection for unified workspace UX.
- Initial demo should not try to show all use cases.
- Initial demo should show one workflow, one policy denial, one audit proof, one cost attribution, and one tier upgrade.
- The demo target date is 2026-09-30.
- The paid pilot target date is 2026-12-15.

## Slide 19 - Differentiated Claims

- Claim 1: Oyatie treats enterprise-market categories as capability tiers over shared primitives.
- Claim 2: Oyatie makes policy decisions explicit through Cedar.
- Claim 3: Oyatie makes evidence a runtime output rather than an after-the-fact project.
- Claim 4: Oyatie supports deployment portability for regulated and sovereign needs.
- Claim 5: Oyatie is designed around flat microservice ownership, not platform forks.
- Claim 6: Oyatie connects AI autonomy to capability grants, cost ceilings, and audit evidence.
- Claim 7: Oyatie can sell consolidation value without requiring every incumbent to be replaced on day one.
- Claim 8: Oyatie's internal engineering system is governed by the same primitives it sells.
- Claim 9: Oyatie's compliance-pack concept maps regulation to executable behavior.
- Claim 10: Oyatie's ontology projection approach avoids copying product data models per app.
- Claim 11: Oyatie's per-cell model gives a natural path to regulated deployments.
- Claim 12: Oyatie's Oya VCS path is a product and engineering differentiator for agentic software development.
- Claim 13: Oyatie's 78 microservice directory base suggests breadth, but not yet commercial completeness.
- Claim 14: Oyatie's 575 Cedar files suggest policy depth, but runtime enforcement must be proven.
- Claim 15: Oyatie's 61 tier matrices suggest packaging depth, but buyer packaging must be simplified.
- These claims are investor thesis claims, not customer evidence claims.
- Every claim needs a proof artifact in diligence.
- Proof artifact 1: live policy decision trace.
- Proof artifact 2: audit-chain row and verification path.
- Proof artifact 3: capability-tier grant and downgrade refusal.
- Proof artifact 4: per-tenant quota exhaustion behavior.
- Proof artifact 5: workflow execution with rollback or compensation.
- Proof artifact 6: cost attribution for AI or integration call.
- Proof artifact 7: migration map from named incumbent.
- Proof artifact 8: security and compliance roadmap.

## Slide 20 - Risks

- Risk 1: platform scope is too broad for seed-stage execution.
- Risk 2: incumbents bundle similar governance features faster.
- Risk 3: customers resist migrating systems of record.
- Risk 4: compliance claims outpace verified evidence.
- Risk 5: AI-cost volatility harms gross margins.
- Risk 6: dedicated-cell deployments reduce margin early.
- Risk 7: policy-engine complexity creates product friction.
- Risk 8: enterprise sales cycles exceed runway.
- Risk 9: insufficient human team depth for the domain span.
- Risk 10: broad architecture confuses first customer value.
- Risk 11: microservice breadth creates operational burden.
- Risk 12: data migration from Salesforce, SAP, Workday, or ServiceNow is harder than planned.
- Risk 13: security reviews block pilots.
- Risk 14: cloud marketplace listing takes longer than expected.
- Risk 15: documentation artifacts are mistaken for production readiness.
- Mitigation 1: narrow first wedge to governed workflow plus evidence.
- Mitigation 2: price first pilots around containment, not rip-and-replace.
- Mitigation 3: maintain claim gates and avoid false maturity claims.
- Mitigation 4: hire security and compliance early.
- Mitigation 5: keep dedicated-cell pricing premium explicit.
- Mitigation 6: use design partners with clear evidence pain.
- Mitigation 7: require ROI baseline before contract.
- Mitigation 8: sequence product depth before vertical breadth.
- Mitigation 9: keep board and advisors enterprise-heavy.

## Slide 21 - Milestones

- 2026-06-15: investor data room v1 complete.
- 2026-06-30: governed workflow demo scope frozen.
- 2026-07-31: live Cedar decision trace demo complete.
- 2026-08-31: audit-chain proof demo complete.
- 2026-09-30: integrated FD-001 wedge demo complete.
- 2026-10-15: first design-partner LOI signed.
- 2026-11-30: second design-partner LOI signed.
- 2026-12-15: first paid pilot contract signed.
- 2027-01-31: first contracted ARR recognized.
- 2027-03-31: SOC 2 readiness assessment complete.
- 2027-05-31: $1.8 million ARR target.
- 2027-08-31: Series A readiness checkpoint.
- 2027-12-31: $6.0 million ARR base-case target.
- 2028-03-31: compliance-pack GA for two named packs.
- 2028-06-30: cloud marketplace listing complete.
- 2028-12-31: 15 enterprise customers target.
- 2029-05-31: $24.0 million ARR base case.
- 2029-05-31: $60.0 million ARR upside case.
- Milestone evidence must include product proof, customer proof, and financial proof.
- Product proof means running software and test evidence.
- Customer proof means signed contract, renewal, or reference.
- Financial proof means ARR, gross margin, CAC, or usage metrics.
- Board review cadence should be monthly through seed.
- Milestone slip triggers scope cut before headcount expansion.

## Slide 22 - Capital Ask

- Seed ask: $18.0 million.
- Minimum viable seed close: $12.0 million.
- Target lead check: $8.0 million to $12.0 million.
- Strategic co-investor allocation: $3.0 million.
- Operator angel and advisor allocation: $1.0 million.
- Employee option pool refresh target: 12 percent post-financing.
- Runway target: 24 months.
- Engineering allocation: $7.2 million.
- Product and design allocation: $1.8 million.
- Security, compliance, and legal allocation: $2.3 million.
- GTM allocation: $2.6 million.
- Customer engineering allocation: $1.5 million.
- Cloud, CI, and test infrastructure allocation: $2.1 million.
- Contingency allocation: $0.5 million.
- Tranche 1 release target: close plus 30 days.
- Tranche 2 release target: integrated demo by 2026-09-30.
- Tranche 3 release target: first paid pilot by 2026-12-15.
- Tranche 4 release target: security baseline by 2027-03-31.
- Use-of-funds discipline is milestone-gated.
- The ask funds proof of category viability.
- The ask does not fund indiscriminate vertical expansion.
- The ask should be paired with tight investor reporting.
- The ask-and-use-of-funds document gives the named tranche breakdown.
- The stop condition is Series A readiness by 2027-08-31.

## Slide 23 - Diligence Package

- Diligence artifact 1: repository source map.
- Diligence artifact 2: root-hub pointer map.
- Diligence artifact 3: masterplan claim-boundary explanation.
- Diligence artifact 4: architecture diagram.
- Diligence artifact 5: live workflow demo.
- Diligence artifact 6: Cedar policy decision trace.
- Diligence artifact 7: audit-chain event proof.
- Diligence artifact 8: capability-tier grant proof.
- Diligence artifact 9: market-sizing model.
- Diligence artifact 10: pricing and unit-economics model.
- Diligence artifact 11: competitive matrix.
- Diligence artifact 12: use-of-funds plan.
- Diligence artifact 13: risk register.
- Diligence artifact 14: security roadmap.
- Diligence artifact 15: compliance roadmap.
- Diligence artifact 16: customer discovery notes.
- Diligence artifact 17: design-partner LOI templates.
- Diligence artifact 18: migration plan from Salesforce and ServiceNow.
- Diligence artifact 19: cloud deployment plan.
- Diligence artifact 20: hiring plan.
- Diligence artifact 21: board reporting template.
- Diligence artifact 22: ARR model.
- Diligence artifact 23: gross margin model.
- Diligence artifact 24: product milestone acceptance criteria.
- Diligence artifact 25: legal disclaimer and claims policy.

## Slide 24 - Source Register

- Gartner source: Worldwide IT spending forecast, April 22 2026, $6.316 trillion overall IT spending, $1.444 trillion software.
- Gartner source URL: https://www.gartner.com/en/newsroom/press-releases/2026-04-22-gartner-forecasts-worldwide-it-spending-to-grow-13-point-5-percent-in-2026-totaling-6-point-31-trillion-dollars
- Gartner source: Worldwide public cloud spending, November 19 2024, $723.421 billion 2025 total public cloud.
- Gartner source: Worldwide public cloud SaaS line, November 19 2024, $299.071 billion 2025 SaaS.
- Gartner source URL: https://www.gartner.com/en/newsroom/press-releases/2024-11-19-gartner-forecasts-worldwide-public-cloud-end-user-spending-to-total-723-billion-dollars-in-2025
- Gartner source: Enterprise business applications market opportunity map, published July 18 2025, $254 billion 2025 and $428 billion 2029.
- Gartner source URL: https://www.gartner.com/en/documents/6744434
- Gartner source: Customer experience and relationship management market share, published June 11 2025, CRM grew to $128 billion in 2024.
- Gartner source URL: https://www.gartner.com/en/documents/6582102
- Forrester source: Global technology spend forecast, February 2 2026, $5.6 trillion 2026 tech spend.
- Forrester source URL: https://www.forrester.com/press-newsroom/forrester-global-tech-forecast-2025-to-2030/
- Zylo source: 2026 SaaS Management Index, $75B plus spend dataset, $55.7M average annual SaaS spend, 305 average portfolio size.
- Zylo source URL: https://zylo.com/2026-saas-management-index
- Internal source: `specs/masterplan.json`, FD-001 scope and claim boundary.
- Internal source: `specs/hyperscaler-architecture-invariants.json`, architecture invariants.
- Internal source: `docs/decisions/ADR-0709-general-live-apex.md`, capability-tier doctrine.
- Internal source: `docs/decisions/ADR-0700-ci-admission-live-apex.md`, Cedar universal gate doctrine.
- Internal source: `docs/decisions/ADR-0709-general-live-apex.md`, audit-chain doctrine.
- Internal source: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`, compliance-pack doctrine.
- Internal source: `docs/standards/per-tenant-resource-quotas-canonical.md`, quota axes.
- Internal source: `microservices/intelligence/capability-tiers/tier-matrix.md`, tier model.
- Internal source: local file count, 78 microservice directories.
- Internal source: local file count, 540 catalog YAML files.
- Internal source: local file count, 575 Cedar files.
- Internal source: local file count, 61 capability-tier matrices.

## Slide 25 - Closing Narrative

- Oyatie is a wager that enterprise software is ready for substrate-level consolidation.
- The market is large enough: trillions in IT spend, hundreds of billions in SaaS and business applications.
- The pain is concrete: too many apps, unclear AI spend, duplicated audit work, and policy drift.
- The wedge is specific: Tenant RBAC view plus Tenant RBAC view with governed workflow and evidence.
- The product path is staged: demo, design partners, paid pilots, security baseline, ARR, Series A.
- The defensibility path is cumulative: shared substrates, Cedar, audit chain, capability tiers, compliance packs, and cell deployment.
- The competitive path is honest: incumbents are strong, but they are constrained by existing product silos and bundle incentives.
- The business model is concrete: tenant subscription, capability tiers, usage, regulated deployment premium, and marketplace take rate.
- The ask is concrete: $18.0 million for 24 months.
- The use of funds is concrete: engineering, security, compliance, GTM, customer engineering, infrastructure, and contingency.
- The investor risk is concrete: execution burden, migration friction, incumbents, claims discipline, and enterprise sales cycles.
- The proof plan is concrete: live workflow, policy trace, audit proof, capability-tier grant, customer contract, and ARR.
- The company should not claim production maturity before gates prove it.
- The company can claim architecture depth and a differentiated product thesis.
- The company can claim a large, named, research-backed market.
- The company can claim a precise first wedge.
- The company can claim a milestone-gated funding plan.
- The company can claim a reason investors should look now.
- If Oyatie works, it becomes the control layer for regulated enterprise work.
- If Oyatie works, software categories become activations over shared governed primitives.
- If Oyatie works, AI actions become auditable business operations instead of unmanaged assistants.
- If Oyatie works, enterprise customers buy fewer disconnected tools and gain stronger evidence.
- The near-term objective is not to boil the ocean.
- The near-term objective is to prove one governed enterprise workflow that expands.
- The investor ask is to fund that proof with enough runway to reach credible Series A metrics.
