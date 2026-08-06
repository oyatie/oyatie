---
doc_class: InvestorMaterial
doc_type: moat_and_defensibility
status: canonical_draft
date: 2026-05-20
owner: founder-office
target_audience: seed_investors, strategic_investors, enterprise_architecture_diligence
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Moat and Defensibility

## 01 - Core Moat Thesis

- Oyatie's moat is a compounding substrate, not a single application feature.
- The product is designed so every additional workflow strengthens shared primitives.
- The first primitive is tenancy.
- The second primitive is identity.
- The third primitive is Cedar policy.
- The fourth primitive is audit-chain evidence.
- The fifth primitive is ontology projection.
- The sixth primitive is workflow execution.
- The seventh primitive is capability-tier activation.
- The eighth primitive is compliance-pack overlays.
- The ninth primitive is per-tenant quota enforcement.
- The tenth primitive is cell-based deployment.
- The eleventh primitive is agentic development governance through Oya VCS and Foundry.
- The moat grows when many product categories share the same primitives.
- The moat weakens if product teams fork primitives per category.
- The moat weakens if buyer packaging becomes too abstract.
- The moat weakens if runtime proof lags architecture.
- The moat is strongest in regulated and multinational environments.
- The moat is strongest where evidence, policy, and data lineage are painful.
- The moat is weaker in pure SMB point-tool replacement.
- The moat is weaker where an incumbent already owns all workflows.
- The investor question is whether shared primitives can become durable switching costs.
- The buyer question is whether shared primitives reduce daily operational cost.
- The technical diligence question is whether primitive enforcement is real.
- The commercial diligence question is whether buyers will pay for governance before full replacement.

## 02 - Moat Factor: Substrate Primitives

- Substrate primitives are shared services that every product surface depends on.
- Substrate primitives prevent duplicated product silos.
- Substrate primitives include tenancy, identity, audit-chain, policy, ontology, workflow, eventing, observability, secrets, quotas, and cells.
- ADR-0001 names shared substrates as the basis of cohesion.
- ADR-0245 separates substrate and product layering.
- ADR-0316 turns adjacent product labels into capability tiers.
- Gartner forecasts software spending of $1.443621 trillion in 2026.
- Gartner forecasts enterprise business applications of $254 billion in 2025.
- The spend pool is large because enterprises buy many overlapping applications.
- A substrate moat targets overlap directly.
- Salesforce has CRM depth but historically organizes around Salesforce clouds.
- SAP has ERP depth but carries suite and migration complexity.
- Workday has HR and finance depth but does not unify every operational surface.
- ServiceNow has workflow strength but begins from IT and service management.
- Microsoft has bundle breadth but depends on Microsoft ecosystem gravity.
- Palantir has ontology depth but often requires high-touch implementation.
- Oyatie's claim is not that incumbents cannot build primitives.
- Oyatie's claim is that incumbents have incentives to preserve product bundle boundaries.
- Substrate primitives create architectural leverage.
- Substrate primitives also create execution burden.
- The defensible path is narrow proof plus broad reuse.
- The first proof milestone is integrated governed workflow by 2026-09-30.
- The second proof milestone is first paid pilot by 2026-12-15.
- The third proof milestone is $1.8 million ARR by 2027-05-31.
- Substrate primitives are a moat only after customer workflows depend on them.

## 03 - Moat Factor: Cedar Policy Engine

- Cedar policy is a central defensibility factor.
- ADR-0243 frames Cedar as the universal policy gate.
- Cedar evaluates authorization.
- Cedar evaluates routing.
- Cedar evaluates retention.
- Cedar evaluates eligibility.
- Cedar evaluates compliance pack behavior.
- Cedar evaluates tenant scope.
- Cedar evaluates AI autonomy boundaries.
- Cedar evaluates cost and budget predicates when composed with cost kernels.
- The repository contains 575 Cedar policy files under microservice scopes.
- Policy coverage is visible in microservice-local policy directories.
- Policy files are not enough without runtime enforcement.
- The moat is executable policy plus evidence.
- Salesforce has Shield, Data Cloud governance, and platform permissions.
- Microsoft has Entra, Purview, Defender, and Conditional Access.
- ServiceNow has policy and workflow controls.
- SAP has GRC and authorization objects.
- Workday has role-based security and business process controls.
- Palantir has security markings and granular access controls.
- Oyatie differentiates by making Cedar a shared app-level policy substrate across categories.
- The best customer proof is a policy denial that prevents a costly action.
- The best investor proof is a signed policy fragment, evaluation trace, and audit-chain event.
- The first Cedar proof should be demo-ready by 2026-07-31.
- The first commercial Cedar proof should appear in the 2026-12-15 paid pilot.
- The moat grows when every capability tier binds to permit sets.
- The moat grows when customers write or approve policy packs.
- The moat grows when auditors can inspect policy decisions.
- The moat weakens if policy authoring is too hard for operators.
- The mitigation is admin UX and pack templates.

## 04 - Moat Factor: Audit Chain

- Audit-chain evidence is the proof layer.
- ADR-0003 adopts an append-only hash-chained audit-event log.
- The audit chain is positioned as a tamper-evident record-keeping surface.
- The audit chain binds regulated events across product surfaces.
- The audit chain supports customer trust.
- The audit chain supports DSR and proof-of-erasure.
- The audit chain supports compliance-pack evidence.
- The audit chain supports AI action accountability.
- The audit chain supports development workflow evidence through Oya VCS.
- The audit chain becomes a switching cost when customer evidence workflows rely on it.
- Competitor evidence tools include AuditBoard, Drata, Vanta, Archer, OneTrust, and ServiceNow GRC.
- These competitors often manage evidence as compliance workflow.
- Oyatie's differentiator is evidence emitted by operational substrate.
- A compliance tool asks teams to upload proof.
- Oyatie should generate proof from runtime behavior.
- The dollar value is audit labor reduction plus reduced regulator response time.
- Example modeled annual evidence savings for an enterprise tenant: $250,000.
- Example modeled software displacement plus evidence benefit: $2.15 million gross annual benefit.
- The evidence value is higher in financial services, healthcare, public sector contractors, and enterprise SaaS.
- The first audit-chain proof demo target is 2026-08-31.
- The first paid audit-chain proof target is 2026-12-15.
- The first external audit readiness target is 2027-03-31.
- The audit chain must prove integrity, redaction, retrieval, and tenant isolation.
- The audit chain moat weakens if customers cannot understand or export evidence.
- The mitigation is trust portal, auditor view, and plain evidence packs.

## 05 - Moat Factor: Capability Tiers

- Capability tiers are Oyatie's packaging and product-fragmentation defense.
- ADR-0316 says enterprise categories become capability tiers over shared primitives.
- A capability tier is a tenant-visible activation unit.
- A capability tier is not a separate microservice.
- A capability tier can bind Cedar permit sets.
- A capability tier can bind ontology projections.
- A capability tier can bind workflow template libraries.
- A capability tier can bind UX shell manifests.
- A capability tier can bind compliance overlays.
- A capability tier can bind observability metadata.
- A capability tier can bind cost metadata.
- Competitors package by product edition.
- Salesforce packages Sales Cloud, Service Cloud, Marketing Cloud, Revenue Cloud, and Data Cloud.
- SAP packages module and suite functionality.
- Workday packages HCM, Financial Management, Adaptive Planning, and related products.
- ServiceNow packages ITSM, CSM, HRSD, SecOps, and industry workflows.
- Oyatie packages capabilities over one substrate.
- The moat grows if upgrades activate more shared primitives without migration.
- The moat grows if compliance packs push customers into higher tiers.
- The moat weakens if tiers become confusing.
- The mitigation is buyer-specific packaging and clear limits.
- The first packaging proof should be a tier grant and tier denial by 2026-09-30.

## 06 - Moat Factor: 70+ Microservice Depth

- The local repository has 78 microservice directories.
- The local repository has 70 microservice-local catalog directories.
- The local repository has 540 catalog YAML records under `registry/catalog`.
- The local repository has 61 capability-tier matrices.
- The local repository has 575 Cedar policy files.
- These counts indicate broad design and artifact depth.
- These counts do not prove customer adoption.
- These counts do not prove production readiness.
- These counts do not prove revenue.
- The defensibility value is surface coverage and design investment.
- The execution risk is operational complexity.
- The correct investor phrasing is "deep artifact base."
- The incorrect investor phrasing is "fully shipped platform."
- Microservice breadth supports cross-category expansion.
- Microservice breadth also increases test and deployment burden.
- The moat improves when microservices share substrates.
- The moat degrades when microservices duplicate primitives.
- Oya VCS and admission gates are intended to manage breadth.
- Hyperscaler invariants are intended to manage operational rigor.
- Capability-tier schema is intended to manage product packaging.
- Pack-overlay schema is intended to manage compliance variation.
- Ontology projection schema is intended to manage data reuse.
- The first proof is not all 78 services running.
- The first proof is a small number of services demonstrating substrate leverage.
- The first demo should involve policy, workflow, audit, ontology, and one buyer-facing workflow.
- The 70+ service moat becomes real as customers activate more surfaces.

## 07 - Moat Factor: ADR-Driven Discipline

- ADR-driven discipline is a governance moat.
- ADRs record decisions, alternatives, and claim boundaries.
- Oyatie has ADRs for audit chain, capability tiers, policy engine, compliance packs, deployment model, and cell architecture.
- ADR discipline reduces accidental architecture drift.
- ADR discipline supports enterprise diligence.
- ADR discipline supports regulator-facing explanation.
- ADR discipline supports new engineer onboarding.
- ADR discipline supports agentic implementation.
- ADR discipline is not a product moat by itself.
- ADR discipline becomes a moat when it prevents silent regressions.
- ADR discipline becomes a moat when it accelerates audits.
- ADR discipline becomes a moat when it makes platform claims inspectable.
- Competitors also have internal architecture discipline.
- Oyatie's distinction is exposing enough doctrine to create investor and customer trust early.
- The danger is over-documentation without runtime proof.
- The mitigation is evidence gates and executable specs.
- Markdown retirement policy shows the repo preference for machine-readable control surfaces.
- Investor docs are a user-directed exception for fundraising materials.
- Claims policy should separate accepted doctrine from implemented runtime behavior.
- The first diligence package should include a claim-boundary matrix by 2026-06-15.
- The first customer-facing evidence policy should be ready by 2026-09-30.
- The first board review should track false-green risks monthly.
- ADR-driven discipline reduces repeated re-litigation.
- ADR-driven discipline should not slow customer proof.
- The moat is disciplined execution, not documentation volume.

## 08 - Moat Factor: Compliance Packs

- Compliance packs are a commercial and defensibility layer.
- ADR-0251 defines compliance packs as versioned signed bundles.
- Example packs include HIPAA.
- Example packs include PCI DSS.
- Example packs include FedRAMP.
- Example packs include EU GDPR.
- Example packs include KR-PIPA.
- Example packs include KR-FSS.
- Example packs include DORA.
- Example packs include EU AI Act.
- Example packs include FERPA.
- Example packs include FDA 21 CFR Part 11.
- Compliance packs bind policy, data class, jurisdiction, evidence, and retention.
- Compliance packs are better than custom services per regulation.
- Compliance packs create expansion revenue.
- Evidence-pack add-on range is $60,000 to $250,000 annually.
- Dedicated-cell add-on range is $300,000 to $1,200,000 annually.
- Compliance pack value is strongest in regulated enterprises.
- Competitors include OneTrust, Drata, Vanta, AuditBoard, Archer, ServiceNow GRC, and Hyperproof.
- Oyatie's differentiator is operational enforcement plus evidence emission.
- The moat grows if packs are reusable across customers.
- The moat weakens if packs become consulting projects.
- First two pack GA target is 2028-03-31.
- SOC 2 readiness assessment target is 2027-03-31.
- Compliance-pack roadmap should be evidence-first.
- Investors should treat pack depth as upside until runtime enforcement exists.

## 09 - Moat Factor: Ontology and Data Lineage

- Ontology is the semantic data moat.
- Oyatie's ontology projection schema defines tenant-visible views over shared object types.
- Ontology avoids each product creating its own version of customer, invoice, task, asset, employee, or policy.
- Palantir Foundry is the clearest competitor in ontology-led enterprise software.
- Microsoft Fabric competes through data integration and semantic models.
- Salesforce Data Cloud competes through customer data unification.
- SAP Datasphere competes through business data fabric.
- Snowflake and Databricks compete through data platforms.
- Oyatie's distinction is coupling ontology to workflow, policy, audit, and capability tiers.
- Ontology creates switching cost when customer operations depend on shared object relationships.
- Ontology supports cross-product analytics.
- Ontology supports audit lineage.
- Ontology supports role-based projections.
- Ontology supports compliance-pack data rules.
- Ontology supports AI grounding and retrieval.
- The risk is ontology complexity overwhelming early users.
- The mitigation is narrow projections per capability tier.
- The first demo should show one object graph with an auditable workflow.
- The first customer proof should show one cross-system data lineage path.
- The first financial services use case could link account, approval, policy, and evidence.
- The first enterprise SaaS use case could link customer, incident, control, evidence, and source change.
- The first professional services use case could link client, matter, document, approval, and invoice.
- The ontology moat compounds with more customer-specific data.
- The ontology moat must avoid data lock-in accusations.
- Export and portability must be product principles.

## 10 - Moat Factor: Agentic Governance

- Agentic governance is a future-facing moat.
- Enterprise AI agents need policy, audit, cost, and rollback controls.
- Foundry is the internal agentic pipeline and capability runtime surface.
- Oya VCS controls claim, verify, done, and promote.
- Capability records can declare autonomy requirements.
- Cedar can enforce autonomy ceilings.
- Audit-chain can record agent actions.
- FinOps can attribute AI and workflow usage.
- Workflow engine can model compensating actions.
- This combination is stronger than a generic chat assistant.
- Competitors include OpenAI enterprise-market offerings.
- Competitors include Anthropic enterprise-market offerings.
- Competitors include Microsoft Copilot Studio.
- Competitors include Salesforce Agentforce.
- Competitors include ServiceNow AI agents.
- Competitors include Atlassian Rovo.
- Competitors include GitHub Copilot Enterprise.
- Oyatie should not compete as a foundation model.
- Oyatie should compete as the governed action layer.
- Gartner's 2026 software forecast notes GenAI effects on software spend.
- Zylo reports AI-native app spend up 108 percent overall and 393 percent in large enterprises.
- The buyer pain is unmanaged AI spend and uncontrolled actions.
- The product promise is action under policy.
- The monetization promise is usage-based metering with cost ceilings.
- The first agentic proof should show a capability invocation with policy, cost, and evidence.
- The risk is AI hype correction.
- The mitigation is selling governance and workflow value independent of model hype.

## 11 - Moat Factor: Deployment Portability

- Deployment portability matters for regulated and multinational buyers.
- Oyatie doctrine is Kubernetes-first for server workloads.
- Deployment surfaces include OCI images, SBOM, provenance, OpenTofu, GitOps, and Kubernetes manifests.
- Cell deployment can support managed, dedicated, sovereign, or self-hosted models.
- Hyperscalers compete through native platform services.
- AWS, Microsoft Azure, Google Cloud, Oracle Cloud, and IBM Cloud all benefit from workload lock-in.
- Oyatie's portability claim can appeal to buyers avoiding single-cloud dependence.
- Portability is not free.
- Portability increases test matrix burden.
- Portability can reduce gross margin if dedicated cells are underpriced.
- Dedicated cell premium is modeled at $300,000 to $1,200,000 annually.
- Portability moat grows if deployment evidence becomes part of audits.
- Portability moat grows if regulatory pressure favors sovereign deployments.
- Portability moat weakens if customers prefer hyperscaler-native convenience.
- Mitigation is cloud marketplace integration plus portable architecture.
- Cloud marketplace listing target is 2028-06-30.
- First security baseline target is 2027-03-31.
- First regional compliance-pack GA target is 2028-03-31.
- The deployment story should not be first-slide marketing.
- The deployment story is a trust and expansion lever.
- The investor question is margin impact.
- The customer question is compliance and control.
- The technical question is repeatable cluster conformance.

## 12 - Moat Factor: Cost Attribution and Unit Economics

- Cost attribution is a moat because AI and workflow spend are becoming volatile.
- Zylo reports unexpected AI and consumption charges for 78 percent of IT leaders.
- Oyatie can meter per tenant.
- Oyatie can meter per capability.
- Oyatie can meter per workflow.
- Oyatie can meter per AI invocation.
- Oyatie can meter per evidence event.
- Oyatie can enforce budget gates.
- Oyatie can attribute internal and external provider costs.
- Foundry provider adapters can emit cost events.
- FinOps surfaces can aggregate cost by capability tier.
- Cedar can deny actions when budget policy fails.
- Audit-chain can record cost-relevant actions.
- Competitors include Apptio, CloudHealth, AWS Cost Explorer, Azure Cost Management, and ServiceNow ITAM.
- Oyatie's distinction is cost controls attached to business workflow.
- Pricing can include base subscription plus overage.
- Gross margin depends on AI cost controls working.
- The unit-economics target is 78 percent subscription gross margin at scale.
- Early gross margin target is 55 to 65 percent.
- CAC payback target is under 18 months by year 3.
- LTV/CAC target is 4.0x plus by year 3.
- The moat grows when customers trust Oyatie's cost ledger.
- The moat weakens if AI costs are unpredictable.
- Mitigation is explicit included allowances and hard budget ceilings.
- First cost-attribution demo should be ready by 2026-09-30.

## 13 - Moat Factor: Customer Switching Costs

- Switching cost begins with workflow dependency.
- Switching cost grows with audit evidence history.
- Switching cost grows with policy fragments.
- Switching cost grows with ontology mappings.
- Switching cost grows with compliance-pack configuration.
- Switching cost grows with user role projections.
- Switching cost grows with integrations and connectors.
- Switching cost grows with AI action history.
- Switching cost grows with cost allocation and budget policies.
- Switching cost grows with tenant hierarchy configuration.
- Switching cost grows with dedicated cell deployment.
- Switching cost should not depend on data hostage-taking.
- Data export must remain credible.
- API portability must remain credible.
- Contract portability must remain credible.
- The moat should be value lock-in, not abusive lock-in.
- Salesforce has high switching cost through data, automation, app ecosystem, and trained users.
- SAP has high switching cost through core process depth.
- Workday has high switching cost through HR and finance business process configuration.
- ServiceNow has high switching cost through workflow and CMDB.
- Microsoft has high switching cost through identity, productivity, and bundle economics.
- Oyatie must create switching cost through cross-category primitives.
- The first switching-cost proof is customer expansion from one workflow to two.
- Expansion milestone target is second paid workflow in first enterprise pilot by 2027-03-31.
- Retention milestone target is first renewal or expansion by 2027-12-31.

## 14 - Moat Factor: Data Network Effects

- Oyatie can develop data network effects inside a tenant.
- Cross-tenant data network effects must be handled carefully because privacy and regulation constrain sharing.
- Within-tenant data effects come from ontology richness.
- Within-tenant data effects come from workflow history.
- Within-tenant data effects come from policy decisions.
- Within-tenant data effects come from audit evidence.
- Within-tenant data effects come from usage and cost telemetry.
- Cross-tenant learning can be metadata-based and privacy-preserving.
- Cross-tenant learning can improve templates.
- Cross-tenant learning can improve compliance pack defaults.
- Cross-tenant learning can improve migration playbooks.
- Cross-tenant learning can improve benchmark ranges.
- Cross-tenant learning must not leak tenant data.
- Competitors have stronger data positions today.
- Microsoft has broad productivity telemetry.
- Salesforce has broad CRM workflow telemetry.
- SAP has deep ERP process data.
- ServiceNow has IT and workflow telemetry.
- Workday has HR and finance process data.
- Palantir has customer-specific ontology deployments.
- Oyatie's data moat is future potential, not current evidence.
- The correct claim is "designed for data compounding."
- The incorrect claim is "data network effect already exists."
- First proof is template improvement from three pilots by 2027-06-30.
- First benchmark pack target is 2028-03-31.
- Privacy-by-design is required for defensibility.

## 15 - Competitive Response: Hyperscalers

- AWS can compete with native cloud services and marketplace distribution.
- AWS can bundle Bedrock, IAM, CloudTrail, Control Tower, AppFabric, and Marketplace.
- Microsoft can compete with Entra, Purview, Defender, Fabric, Copilot, Dynamics, Power Platform, and Azure.
- Google Cloud can compete with Vertex AI, BigQuery, Workspace, Security Command Center, and Apigee.
- Oracle can compete with OCI, Fusion Apps, NetSuite, and database lock-in.
- IBM can compete with Red Hat OpenShift, watsonx, Maximo, and regulated-industry services.
- Hyperscalers have distribution.
- Hyperscalers have procurement vehicles.
- Hyperscalers have cloud credits and marketplace leverage.
- Hyperscalers have security certifications.
- Hyperscalers do not typically own all business semantics.
- Hyperscalers often prefer platform services over full cross-suite business workflows.
- Oyatie can partner through marketplaces.
- Oyatie can deploy on hyperscalers.
- Oyatie can differentiate on business-level policy and evidence.
- Hyperscaler response risk is medium.
- Hyperscaler partnership upside is high.
- Cloud marketplace listing target is 2028-06-30.
- Defense strategy: stay portable, integrate deeply, avoid becoming infrastructure reseller.
- Defense strategy: make business workflow evidence the product.
- Defense strategy: publish deployment evidence.
- Defense strategy: price cell and regulated deployment correctly.
- Defense strategy: avoid single-cloud primitives where possible.
- Defense strategy: use Kubernetes and OpenTofu portability as buyer trust signals.
- Defense strategy: target buyers with multi-cloud or sovereignty concerns.

## 16 - Competitive Response: Tenant RBACs

- Salesforce can respond through Agentforce, Data Cloud, MuleSoft, Slack, Tableau, and industry clouds.
- SAP can respond through Business Technology Platform, S/4HANA, SuccessFactors, Ariba, Concur, and Joule.
- Oracle can respond through Fusion Cloud Applications, NetSuite, OCI, and database ecosystem.
- Workday can respond through HCM, Financial Management, Adaptive Planning, Extend, and AI agents.
- ServiceNow can respond through workflow expansion, ITSM strength, CMDB, AI agents, and GRC.
- Microsoft can respond through Dynamics 365, Power Platform, M365, Entra, Purview, Fabric, and Copilot.
- These suites have buyer trust.
- These suites have existing data.
- These suites have implementation partner ecosystems.
- These suites have procurement relationships.
- These suites can underprice add-ons to defend accounts.
- These suites can acquire adjacent startups.
- Oyatie's attack is not to replace all suites at once.
- Oyatie's attack is to contain fragmentation and own governed workflows.
- Oyatie's attack is to show audit evidence across suite boundaries.
- Oyatie's attack is to sell capability activation over shared substrate.
- Suite response risk is high.
- Defense strategy: target customers with multi-suite pain.
- Defense strategy: avoid pure CRM feature-for-feature battle first.
- Defense strategy: win evidence and governance workflows.
- Defense strategy: integrate with incumbents before replacing them.
- Defense strategy: create customer-owned policy and ontology assets.
- Defense strategy: show measurable cost and audit savings.
- Defense strategy: keep migration paths credible.
- Defense strategy: build partner channel around migration and evidence.

## 17 - Competitive Response: Vertical SaaS

- Veeva competes in life sciences.
- nCino competes in banking.
- Guidewire competes in insurance.
- Procore competes in construction.
- Toast competes in restaurants.
- Blackbaud competes in nonprofit and education.
- Epic ecosystem competes in healthcare.
- Yardi competes in real estate.
- Amdocs competes in telecom.
- Blue Yonder competes in supply chain.
- Vertical SaaS wins on workflow specificity.
- Vertical SaaS wins on domain vocabulary.
- Vertical SaaS wins on integrations and compliance templates.
- Vertical SaaS can be difficult to displace.
- Oyatie should not start by displacing deep vertical systems.
- Oyatie should start by governing cross-system workflows.
- Oyatie can become a unifying layer across vertical systems.
- Oyatie can later replace specific vertical workflows through capability tiers.
- Vertical response risk is medium.
- Defense strategy: respect vertical depth.
- Defense strategy: use pack overlays and ontology projections.
- Defense strategy: partner where vertical depth is better bought than built.
- Defense strategy: focus on cross-vertical primitives.
- Defense strategy: use data lineage and evidence as wedge.
- Defense strategy: avoid custom one-off projects that break reuse.
- Defense strategy: choose verticals only after horizontal proof.

## 18 - Competitive Response: Best-of-Breed Tools

- Atlassian competes in developer and work management.
- Asana competes in work management.
- Monday.com competes in work operating system positioning.
- Smartsheet competes in collaborative work management.
- ClickUp competes in all-in-one productivity.
- Airtable competes in flexible business apps.
- Notion competes in docs and workspace.
- Coda competes in docs-as-apps.
- Zapier competes in automation.
- Workato competes in enterprise automation.
- Boomi competes in integration platform.
- MuleSoft competes in integration and API management.
- Best-of-breed tools win on speed and adoption.
- Best-of-breed tools often lose on governance at scale.
- Oyatie should not mimic productivity virality first.
- Oyatie should win where governance is a buying criterion.
- Best-of-breed response risk is medium.
- Defense strategy: position as governed operating substrate.
- Defense strategy: integrate with best-of-breed tools where customers need them.
- Defense strategy: sell audit and policy value.
- Defense strategy: avoid overcomplicated UX for simple users.
- Defense strategy: keep workflow authoring ergonomic.
- Defense strategy: let customers consolidate gradually.
- Defense strategy: measure replacement and containment savings.

## 19 - Defensibility Milestones

- 2026-06-15: claim-boundary diligence package complete.
- 2026-06-30: first moat demo script frozen.
- 2026-07-31: Cedar policy decision trace demo complete.
- 2026-08-31: audit-chain proof demo complete.
- 2026-09-30: integrated governed workflow demo complete.
- 2026-10-15: first enterprise SaaS design-partner LOI target.
- 2026-11-30: first financial-services design-partner LOI target.
- 2026-12-15: first paid pilot target.
- 2027-01-31: first ARR recognition target.
- 2027-03-31: SOC 2 readiness assessment target.
- 2027-03-31: second paid workflow in first pilot target.
- 2027-05-31: $1.8 million ARR target.
- 2027-06-30: first reusable migration playbook target.
- 2027-08-31: Series A readiness target.
- 2027-12-31: first renewal or expansion target.
- 2028-03-31: first two compliance-pack GA target.
- 2028-06-30: cloud marketplace listing target.
- 2028-12-31: 15 enterprise customers target.
- 2029-05-31: $24.0 million base ARR target.
- 2029-05-31: $60.0 million upside ARR target.
- Defensibility should be measured by runtime proof.
- Defensibility should be measured by repeatability.
- Defensibility should be measured by expansion.
- Defensibility should be measured by customer evidence dependency.
- Defensibility should not be measured by document count alone.

## 20 - Moat Scorecard

| Moat factor | Current evidence | Strength today | Strength after 3 pilots | Key risk |
| --- | --- | --- | --- | --- |
| Substrate primitives | ADRs and specs | Medium | High | Runtime proof |
| Cedar policy | 575 Cedar files plus ADR-0243 | Medium | High | Policy UX |
| Audit chain | ADR-0003 plus audit-chain surfaces | Medium | High | Evidence export |
| Capability tiers | ADR-0316 plus 61 tier matrices | Medium | High | Packaging clarity |
| Microservice depth | 78 directories, 70 local catalogs | Medium | Medium | Operational complexity |
| Compliance packs | ADR-0251 and schemas | Low-medium | Medium | Certification burden |
| Ontology | schemas and microservice | Medium | High | Data migration |
| Agentic governance | Foundry and Oya VCS doctrine | Medium | High | AI hype and cost |
| Deployment portability | specs and tier matrices | Medium | Medium | Margin impact |
| Customer switching cost | Not yet proven | Low | Medium | Pilot depth |
| Data compounding | Not yet proven | Low | Medium | Privacy constraints |
| Partner ecosystem | Not yet built | Low | Medium | Distribution timing |
| Brand trust | Not yet built | Low | Medium | Security review |
| Cost attribution | planned and partially surfaced | Low-medium | Medium | AI unit cost |
| Marketplace leverage | planned | Low | Medium | Listing timeline |

## 21 - Moat Economics

- Moat economics begin with willingness to pay for consolidation.
- Zylo's $55.7 million average annual SaaS spend provides a benchmark for large SaaS portfolios.
- A 2 percent budget capture on $55.7 million equals $1.114 million ACV.
- A 3 percent budget capture on $55.7 million equals $1.671 million ACV.
- A 5 percent budget capture on $55.7 million equals $2.785 million ACV.
- This supports enterprise pricing if ROI is proven.
- Example direct software displacement: $1.2 million.
- Example integration maintenance savings: $400,000.
- Example audit evidence savings: $250,000.
- Example AI cost-control savings: $300,000.
- Example gross annual benefit: $2.15 million.
- Example net benefit before migration services: $1.43 million.
- The economic moat is expansion through more capability tiers.
- The economic moat is not charging more without value proof.
- The expansion path is single workflow to multi-workflow.
- The expansion path is one compliance pack to many packs.
- The expansion path is managed deployment to dedicated cell.
- The expansion path is human workflow to governed AI action.
- Gross margin target at scale is 78 percent.
- Gross margin risk comes from AI usage and dedicated deployments.
- Cost gates are therefore part of the moat.

## 22 - Defensibility Evidence Required for Investors

- Evidence 1: live governed workflow demo.
- Evidence 2: policy decision trace with Cedar fragment reference.
- Evidence 3: audit-chain event with verification path.
- Evidence 4: capability-tier grant and denial.
- Evidence 5: ontology projection used by a workflow.
- Evidence 6: per-tenant quota enforcement.
- Evidence 7: cost attribution on an AI or integration action.
- Evidence 8: integration with at least one incumbent system.
- Evidence 9: security architecture review.
- Evidence 10: compliance readiness plan.
- Evidence 11: claims policy separating implemented and planned capabilities.
- Evidence 12: customer discovery notes.
- Evidence 13: design-partner LOI.
- Evidence 14: first paid pilot contract.
- Evidence 15: gross margin model.
- Evidence 16: migration services plan.
- Evidence 17: deployment topology model.
- Evidence 18: competitive win/loss assumptions.
- Evidence 19: roadmap and milestone acceptance criteria.
- Evidence 20: hiring plan with named senior roles.
- The first five evidence items should be ready before seed close if possible.
- The first paid pilot is the strongest defensibility evidence.
- The first renewal is the strongest switching-cost evidence.
- The first expansion is the strongest platform evidence.
- The first third-party audit review is the strongest regulated-market evidence.

## 23 - Claims That Are Safe Today

- Safe claim: Oyatie has a differentiated architecture thesis.
- Safe claim: Oyatie targets large, research-backed software budgets.
- Safe claim: Oyatie has deep artifact coverage across microservices, policies, tiers, and ADRs.
- Safe claim: Oyatie is designed around shared primitives.
- Safe claim: Oyatie intends to use Cedar as universal policy gate.
- Safe claim: Oyatie intends to use audit-chain evidence as trust layer.
- Safe claim: Oyatie intends to use capability tiers to avoid product fragmentation.
- Safe claim: Oyatie has a specific first deliverable scope in Tenant RBAC view plus Tenant RBAC view.
- Safe claim: Oyatie has a milestone-gated funding plan.
- Safe claim: Oyatie can sell containment before replacement.
- Safe claim: Oyatie has a credible reason to focus on regulated and multinational buyers.
- Safe claim: Oyatie's moat depends on converting architecture into runtime proof.
- Unsafe claim: Oyatie is production complete.
- Unsafe claim: Oyatie is already hyperscaler mature.
- Unsafe claim: Oyatie has commercial traction unless contracts exist.
- Unsafe claim: Oyatie can replace Salesforce, SAP, Workday, and ServiceNow immediately.
- Unsafe claim: Oyatie has completed compliance certifications.
- Unsafe claim: Oyatie has a data network effect today.
- Unsafe claim: Oyatie has proven customer switching costs today.
- Unsafe claim: Oyatie has proven cloud marketplace distribution today.
- Unsafe claim: Oyatie has proven 78 production microservices today.
- Claim discipline is part of investor trust.
- Claim discipline is part of customer trust.
- Claim discipline should be enforced in all fundraising materials.
- This document uses "designed", "modeled", and "target" intentionally.

## 24 - Source Register

- Gartner, April 22 2026 worldwide IT forecast: $6.31655 trillion total IT spending and $1.443621 trillion software spending in 2026.
- Gartner URL: https://www.gartner.com/en/newsroom/press-releases/2026-04-22-gartner-forecasts-worldwide-it-spending-to-grow-13-point-5-percent-in-2026-totaling-6-point-31-trillion-dollars
- Gartner, November 19 2024 public cloud forecast: $723.421 billion 2025 total public cloud and $299.071 billion 2025 SaaS.
- Gartner URL: https://www.gartner.com/en/newsroom/press-releases/2024-11-19-gartner-forecasts-worldwide-public-cloud-end-user-spending-to-total-723-billion-dollars-in-2025
- Gartner, July 18 2025 enterprise business applications market map: $254 billion in 2025 and $428 billion in 2029.
- Gartner URL: https://www.gartner.com/en/documents/6744434
- Gartner, June 11 2025 CRM market share abstract: CRM software grew to $128 billion in 2024.
- Gartner URL: https://www.gartner.com/en/documents/6582102
- Forrester, February 2 2026 global tech forecast: $5.6 trillion global technology spend in 2026.
- Forrester URL: https://www.forrester.com/press-newsroom/forrester-global-tech-forecast-2025-to-2030/
- Zylo, 2026 SaaS Management Index: $75B plus SaaS and cloud spend, $55.7M average annual SaaS spend, 305 average portfolio size.
- Zylo URL: https://zylo.com/2026-saas-management-index
- Internal source: `docs/decisions/ADR-0705-product-protocol-live-apex.md`.
- Internal source: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal source: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Internal source: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- Internal source: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
- Internal source: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal source: `specs/capability-tier-schema.json`.
- Internal source: `specs/pack-overlay-schema.json`.
- Internal source: `specs/ontology-projection-schema.json`.
- Internal source: `specs/hyperscaler-architecture-invariants.json`.
- Internal source: local count of 78 microservice directories.
- Internal source: local count of 70 microservice-local catalog directories.
- Internal source: local count of 61 capability-tier matrices.
- Internal source: local count of 575 Cedar policy files.

## 25 - Defensibility Conclusion

- Oyatie's strongest moat is the compounding relationship between substrate primitives.
- Cedar policy alone is not the moat.
- Audit-chain evidence alone is not the moat.
- Ontology alone is not the moat.
- Capability tiers alone are not the moat.
- Microservice breadth alone is not the moat.
- ADR discipline alone is not the moat.
- The moat is the combination.
- The moat becomes durable when customers run workflows that depend on the combination.
- The moat becomes measurable when customers expand from one workflow to several.
- The moat becomes financially meaningful when ARR expands through tiers, packs, and usage.
- The moat becomes strategically meaningful when replacing Oyatie requires re-creating policy, evidence, ontology, workflow, and deployment posture together.
- Competitors can copy pieces.
- Competitors will likely copy pieces if the category proves valuable.
- The defense is speed to proof, focus, and customer-specific evidence depth.
- The first proof is not a whitepaper.
- The first proof is a live governed workflow.
- The second proof is a paying design partner.
- The third proof is an expansion from evidence workflow to broader operating workflow.
- The fourth proof is a renewal or reference.
- The fifth proof is repeatable onboarding.
- The $18.0 million seed ask funds the path from architecture to proof.
- The $24.0 million year-3 ARR base case funds proof of business model.
- The $60.0 million year-3 ARR upside case funds proof of category potential.
- The final moat claim should remain evidence-bound until those proof points exist.
