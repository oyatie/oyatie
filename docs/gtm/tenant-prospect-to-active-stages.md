---
doc_class: GoToMarketPlaybook
title: Tenant Prospect-to-Active Stages
status: Draft
date: 2026-05-20
owner: gtm-sales-se + gtm-customer-success + gtm-marketing
related_oyatie_adrs:
  - ADR-0002
  - ADR-0007
  - ADR-0008
  - ADR-0009
  - ADR-0010
  - ADR-0175
  - ADR-0222
  - ADR-0242
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0263
  - ADR-0311
  - ADR-0313
  - ADR-0316
related_personas:
  - SDR Kofi Asante
  - Sales AE Maya Lindqvist
  - Sales Manager Anthony Costa
  - Customer Success Manager Sofia Rezende
  - Customer Champion Akemi Sato
  - CEO Aoki Tanaka
  - CFO Helena Brandt
  - CTO Diego Vargas
  - CHRO Linda Foster
  - CISO Yuki Park
  - CCO Naveen Iyer
  - Procurement Manager Wei Liu
  - External Auditor Hyo-Jin Lee
  - External Regulator Inspector Sergei Petrov
related_journeys:
  - j54-quote-to-contract-to-payment-saas
  - j117-api-customer-tenant-incident-response
  - j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
---

# Tenant Prospect-to-Active Stages

## Purpose

This playbook defines the tenant funnel from first known account signal to expansion.

It is the field-facing sequence for a new organization becoming an Oyatie tenant.

It binds marketing, sales, solutions engineering, legal, finance, security, customer success, and platform operations to one stage model.

The model uses ten named stages even though field teams may compress them for SMB deals.

The canonical stage path is Prospect -> MQL -> SQL -> Demo -> POC -> Negotiate -> Contract -> Onboard -> Active -> Expansion.

The stage model is intentionally tenant-first, not product-first.

Oyatie sells one tenancy, one identity, one audit chain, and capability-tier activation over shared substrate.

A deal is not considered real until the prospect can be mapped to a tenant model, a buyer committee, and a compliance posture.

A demo is not considered complete until it proves the relevant tenant boundary, Cedar permit, and audit evidence story.

A POC is not considered complete until the customer has run a bounded workload with named success metrics.

A contract is not considered complete until the tenant provisioning inputs are present and payment or procurement authority is clear.

An onboarded tenant is not considered active until the onboard saga reaches `Active`, users can work in their own tenant, and support has a named owner.

Expansion is a separate stage because it changes capability tier, evidence obligations, cell posture, and success ownership.

The field team should use this document before creating opportunity stages in CRM.

The solutions engineering team should use this document to decide what must be shown, proved, or refused at each gate.

Customer success should use this document to decide when ownership moves from sales to delivery to steady-state success.

Platform operations should use this document to avoid provisioning tenants that have not cleared legal, policy, residency, or security requirements.

The document is not a pricing sheet.

The document is not a product brochure.

The document is not an implementation runbook for the tenancy microservice.

The document is the commercial control surface for deciding whether a buyer can progress.

Every stage has a named exit-gate.

Every exit-gate has evidence, owner, tools, Cedar permits, and failure modes.

Every exit-gate is reversible until Contract, except regulated data imports that must follow the migration playbook.

## Personas Involved (named — from MASTER-ROSTER)

- SDR Kofi Asante owns first qualified outreach and campaign response triage.

- Sales AE Maya Lindqvist owns commercial discovery, mutual close plan, and executive next steps.

- Sales Manager Anthony Costa owns stage hygiene, forecast discipline, and exception approval for non-standard motions.

- Marketing Specialist Riya Sharma owns campaign-source attribution and prospect nurture content.

- Marketing Manager Olu Adeyemi owns segment messaging, event follow-up, and regional campaign quality.

- Customer Success Manager Sofia Rezende owns success-plan continuity from SQL through Day 90.

- CS-IC Lin Chen owns tactical customer-success follow-up, meeting notes, and onboarding artifact hygiene.

- Customer Champion Akemi Sato owns reference-customer storytelling and champion enablement.

- CEO Aoki Tanaka represents the executive buyer for a mid-large enterprise tenant.

- CFO Helena Brandt represents budget authority, service-credit concerns, and contract liability review.

- CTO Diego Vargas represents technical architecture, integration risk, and platform strategy.

- CHRO Linda Foster represents HR, workforce rollout, and work-tenant change management.

- CISO Yuki Park represents security, tenant isolation, audit evidence, and BYOK expectations.

- CCO Naveen Iyer represents compliance, legal posture, and regulatory pack approval.

- Procurement Manager Wei Liu represents vendor onboarding, procurement steps, and purchase-order readiness.

- Finance Director Mei-Ling Wu represents invoicing, payment timing, and cost-center mapping.

- Office Manager Priya Ramanathan represents workplace rollout logistics for mid-market and SMB tenants.

- Business Analyst Aditya Verma represents requirements capture and operational mapping.

- Data Analyst Felipe Andrade represents reporting validation and metric acceptance.

- External Auditor Hyo-Jin Lee represents Big-4 evidence expectations during regulated enterprise deals.

- External Regulator Inspector Sergei Petrov represents government or regulator-readiness review.

- Board director Patrick O'Reilly represents board-level risk review for high-value enterprise contracts.

## Stages / Steps (named, sequenced)

The funnel uses the following sequence.

Stage 01 is Prospect.

Stage 02 is MQL.

Stage 03 is SQL.

Stage 04 is Demo.

Stage 05 is POC.

Stage 06 is Negotiate.

Stage 07 is Contract.

Stage 08 is Onboard.

Stage 09 is Active.

Stage 10 is Expansion.

No stage may skip tenant-scope analysis.

No stage may skip the data residency question.

No stage may skip buyer-committee identification for mid-market, enterprise, public-sector, or regulated buyers.

No stage may skip Cedar permit mapping when the customer asks for automation, migration, cross-tenant access, privileged support, or regulated data.

SMB self-serve may collapse Prospect, MQL, SQL, and Demo into one web motion, but the exit evidence remains required.

Enterprise field sales may expand Demo and POC into multiple workshops, but the named exit-gates remain the same.

Public-sector deals may insert procurement registration steps inside Negotiate, but may not bypass Contract or Onboard gates.

Regulated customers may require External Auditor Hyo-Jin Lee style evidence before POC begins.

Partner-led deals must still name the end-customer tenant, partner tenant, and reseller or agency posture.

The funnel owner for the current stage is responsible for keeping CRM, workflow-engine, audit-chain evidence, and mutual action plan synchronized.

### Stage 01 -- Prospect

Stage name: Prospect.

Stage owner: SDR Kofi Asante for inbound and outbound first touch.

Supporting owner: Marketing Specialist Riya Sharma for source attribution.

Buyer-facing objective: determine whether the account has a plausible Oyatie problem.

Internal objective: map the organization to an initial tenant hypothesis.

Entry signal: named organization appears through campaign response, event scan, referral, partner introduction, community signal, or direct outbound target list.

Entry signal: the organization has a business domain, legal entity, or public-sector agency identifier.

Entry signal: the organization can be associated with at least one likely buyer persona.

Entry signal: the organization has a pain tied to integration tax, workspace fragmentation, compliance evidence, tenant migration, or capability-tier consolidation.

Required data: legal name if known.

Required data: website or domain.

Required data: region and likely primary jurisdiction.

Required data: first known contact and consent basis.

Required data: probable segment, one of SMB, mid-market, enterprise, regulated, public-sector, ISV, partner, or reseller.

Required data: source channel and campaign id.

Required data: preliminary incumbent stack, if discoverable.

Required data: known privacy constraints before any enrichment.

Primary tool: `forms.prospect-intake`.

Primary tool: `crm.account-create`.

Primary tool: `mail.outreach-sequence`.

Primary tool: `audit-chain.gtm-source-seal`.

Primary Cedar policy: `pol_gtm_prospect_intake_v1`.

Primary Cedar action: `gtm.prospect.create`.

Permit note: the policy allows prospect metadata creation only when consent basis and source are recorded.

Permit note: the policy forbids importing personal-contact lists without campaign provenance.

Evidence artifact: `EVT-GTM-PROSPECT-CREATED`.

Evidence artifact: source attribution row in CRM.

Evidence artifact: initial tenant hypothesis field.

Named exit-gate: Gate P1 -- Account Exists With Tenant Hypothesis.

Gate P1 passes when CRM has one account record, at least one contact or partner path, region, source, segment, and first tenant hypothesis.

Gate P1 fails when the account has no named organization, no permitted contact path, or no reason to believe Oyatie solves a problem.

Gate P1 owner: SDR Kofi Asante.

Gate P1 approver: Sales Manager Anthony Costa for any account over enterprise threshold.

Metric target: prospect-source attribution completeness is 99 percent.

Metric target: duplicate-account merge latency is less than 1 business day.

Metric target: consent-basis missing rate is zero.

Failure mode: scraped contact without consent.

Recovery: quarantine the contact, remove the sequence, seal `EVT-GTM-CONSENT-QUARANTINE`, and restart with compliant source.

Failure mode: duplicate account created under regional spelling variants.

Recovery: merge under canonical legal name and keep alias rows.

Failure mode: prospect mapped to wrong segment.

Recovery: reopen Stage 01 and re-run segment classification before MQL.

### Stage 02 -- MQL

Stage name: Marketing Qualified Lead.

Stage owner: Marketing Manager Olu Adeyemi.

Supporting owner: SDR Kofi Asante.

Buyer-facing objective: confirm the account has engaged enough to justify human sales follow-up.

Internal objective: prove the prospect matches a target motion and has a known business trigger.

Entry signal: Prospect passed Gate P1.

Entry signal: prospect requested demo, downloaded security material, attended event, replied to outreach, or appeared through partner referral.

Entry signal: account is in a target region or has explicit expansion interest.

Qualification signal: KR Group enterprise CIO interest in SaaS, Workspace, Cloud, Vertical Corporate, or compliance bundle.

Qualification signal: mid-market vertical buyer wants a pilot for workflow, workspace, HR, payroll, or regulated operations.

Qualification signal: ISV wants plugin authoring, Foundry capability publishing, or marketplace distribution.

Qualification signal: public-sector specialist sees procurement eligibility.

Qualification signal: incumbent migration problem is visible.

Required data: segment confidence.

Required data: trigger event.

Required data: likely buyer committee.

Required data: high-level use case.

Required data: initial compliance pack candidates.

Required data: incumbent stack summary.

Required data: disqualification notes if not ready.

Primary tool: `crm.lead-score`.

Primary tool: `marketing.campaign-attribution`.

Primary tool: `forms.discovery-lite`.

Primary tool: `workflow-engine.mql-routing`.

Primary Cedar policy: `pol_gtm_mql_scoring_v1`.

Primary Cedar action: `gtm.lead.qualify`.

Permit note: the policy allows lead scoring only from permitted marketing events and declared account attributes.

Permit note: the policy forbids scoring on protected-class or personal-tenant content.

Evidence artifact: `EVT-GTM-MQL-SCORED`.

Evidence artifact: MQL scorecard with segment, trigger, use case, and disqualification branch.

Named exit-gate: Gate M1 -- Qualified Trigger And Segment.

Gate M1 passes when the lead has a named trigger, named segment, target motion, and next human owner.

Gate M1 fails when the account is curious but has no buyer trigger, no target segment, or no consented path to continue.

Gate M1 owner: Marketing Manager Olu Adeyemi.

Gate M1 approver: Sales Manager Anthony Costa for enterprise or regulated accounts.

Metric target: MQL-to-SQL conversion is 35 percent for target segments.

Metric target: MQL response SLA is 1 business day for inbound enterprise and regulated signals.

Metric target: partner-referred MQL acceptance SLA is 2 business days.

Failure mode: nurture content overstates current product readiness.

Recovery: send corrected material and attach the exact capability-tier posture.

Failure mode: protected or sensitive inference used in score.

Recovery: invalidate score, delete prohibited attribute, and emit policy-denial evidence.

Failure mode: target account is already owned by a partner.

Recovery: convert to partner-led SQL path with partner tenant named.

### Stage 03 -- SQL

Stage name: Sales Qualified Lead.

Stage owner: Sales AE Maya Lindqvist.

Supporting owner: Sales Manager Anthony Costa.

Buyer-facing objective: run discovery and confirm a real evaluation path.

Internal objective: establish problem, authority, impact, timeline, and tenant feasibility.

Entry signal: MQL passed Gate M1.

Entry signal: buyer accepts a discovery meeting.

Entry signal: a likely decision-maker or champion is present.

Discovery area: business pain.

Discovery area: incumbent stack.

Discovery area: tenant model and legal entity.

Discovery area: jurisdiction and residency.

Discovery area: identity posture, including SSO, SCIM, passkey, and contractor access.

Discovery area: security and compliance path.

Discovery area: data migration scope.

Discovery area: commercial owner and budget cycle.

Discovery area: procurement and legal review steps.

Discovery area: success metrics for demo or POC.

Required data: named champion.

Required data: named economic buyer or route to economic buyer.

Required data: named technical evaluator.

Required data: named procurement contact or procurement unknown flag.

Required data: named primary use case.

Required data: named disqualification reason if no fit.

Primary tool: `crm.opportunity-create`.

Primary tool: `workflow-engine.mutual-action-plan`.

Primary tool: `forms.discovery-deep`.

Primary tool: `drive.customer-data-room`.

Primary Cedar policy: `pol_gtm_sql_discovery_v1`.

Primary Cedar action: `gtm.opportunity.create`.

Permit note: the policy requires the AE to record tenant scope before customer files are accepted.

Permit note: the policy forbids attaching customer data to generic internal workspaces.

Evidence artifact: `EVT-GTM-SQL-ACCEPTED`.

Evidence artifact: mutual action plan version 1.

Named exit-gate: Gate S1 -- Evaluation Plan Accepted.

Gate S1 passes when the account has a champion, buyer problem, target use case, tenant feasibility screen, and agreed demo or workshop date.

Gate S1 fails when there is no access to a buyer, no urgent problem, no viable tenant model, or no next step.

Gate S1 owner: Sales AE Maya Lindqvist.

Gate S1 approver: Sales Manager Anthony Costa.

Metric target: discovery notes completed within 4 business hours.

Metric target: SQL-to-demo conversion is 70 percent for accepted SQLs.

Metric target: no SQL without tenant feasibility field populated.

Failure mode: champion is enthusiastic but not connected to budget.

Recovery: keep SQL open only if a path to CFO Helena Brandt style buyer is scheduled.

Failure mode: technical use case depends on unsupported regulated pack.

Recovery: convert to roadmap-follow-up, not Demo, unless CCO Naveen Iyer accepts a bounded proof.

Failure mode: account asks for migration before tenant model is defined.

Recovery: run Stage 05 pre-POC data map only after tenancy feasibility is recorded.

### Stage 04 -- Demo

Stage name: Demo.

Stage owner: Sales AE Maya Lindqvist.

Supporting owner: Solutions Engineering lead.

Buyer-facing objective: show Oyatie's cohesion thesis in the customer's language.

Internal objective: prove the customer can see the tenant, identity, policy, audit, and capability-tier story.

Entry signal: SQL passed Gate S1.

Entry signal: demo agenda accepted by champion and technical evaluator.

Entry signal: demo tenant or scripted sandbox is prepared with no customer confidential data unless explicitly approved.

Demo storyline: one tenant boundary across workspace, workflow, Foundry, and compliance evidence.

Demo storyline: one passkey-bound identity with active tenant context.

Demo storyline: Cedar default-deny blocks wrong-tenant access.

Demo storyline: audit-chain evidence appears for important decisions.

Demo storyline: capability tier can activate a business surface without creating product sprawl.

Demo storyline: migration path exists for incumbent tools.

Required artifact: demo agenda.

Required artifact: demo data classification.

Required artifact: role list for customer attendees.

Required artifact: explicit do-not-demo list for unsupported claims.

Required artifact: post-demo recap.

Primary tool: `meet.demo-room`.

Primary tool: `workflow-engine.demo-script`.

Primary tool: `tenancy.demo-sandbox`.

Primary tool: `policy-engine.demo-cedar-evaluator`.

Primary tool: `audit-chain.demo-evidence-pack`.

Primary Cedar policy: `pol_gtm_demo_sandbox_v1`.

Primary Cedar action: `demo.sandbox.provision`.

Primary Cedar action: `demo.tenant_context.switch`.

Permit note: the policy grants demo users sandbox-only actions and forbids production tenant mutation.

Permit note: the policy requires every demo file to be labeled synthetic, customer-approved, or public.

Evidence artifact: `EVT-GTM-DEMO-DELIVERED`.

Evidence artifact: demo attendance and questions list.

Named exit-gate: Gate D1 -- Demo Value Confirmed.

Gate D1 passes when the customer confirms the target problem, identifies success metrics, and agrees whether the next step is POC, technical evaluation, commercial negotiation, or nurture.

Gate D1 fails when the customer cannot connect the demo to an operational problem or when the demo reveals a blocker.

Gate D1 owner: Sales AE Maya Lindqvist.

Gate D1 approver: Solutions Engineering lead for technical next steps.

Metric target: demo recap sent within 1 business day.

Metric target: demo action items assigned within 4 business hours.

Metric target: no demo uses production customer data without POC policy.

Failure mode: demo audience asks for unsupported product claim.

Recovery: state current capability tier honestly, record gap, and route to roadmap or POC constraint.

Failure mode: demo sandbox policy denies an intended action.

Recovery: stop demo action, explain default-deny, and decide whether the desired action belongs in POC.

Failure mode: buyer committee missing key persona.

Recovery: schedule role-specific follow-up for CISO, CFO, CHRO, CCO, or procurement.

### Stage 05 -- POC

Stage name: Proof of Concept.

Stage owner: Solutions Engineering lead.

Supporting owner: Customer Success Manager Sofia Rezende.

Buyer-facing objective: prove the workload against agreed success criteria.

Internal objective: exercise tenant, policy, integration, migration, and operations paths without widening scope.

Entry signal: Demo passed Gate D1 and customer agrees to proof criteria.

Entry signal: POC success plan is signed by champion and technical evaluator.

Entry signal: data class and residency posture are approved.

Entry signal: customer data room exists when customer artifacts are used.

POC type: synthetic demo POC.

POC type: integration POC.

POC type: migration dry-run POC.

POC type: regulated evidence POC.

POC type: executive dashboard POC.

POC type: workflow automation POC.

POC type: identity and SCIM POC.

POC type: Foundry capability POC.

Required artifact: POC charter.

Required artifact: customer data inventory.

Required artifact: success metrics.

Required artifact: named technical owners.

Required artifact: timeline.

Required artifact: out-of-scope list.

Required artifact: rollback plan.

Required artifact: final results report.

Primary tool: `workflow-engine.poc-orchestrator`.

Primary tool: `tenancy.trial-provisioning`.

Primary tool: `identity.saml-scim-test`.

Primary tool: `policy-engine.poc-scope-evaluate`.

Primary tool: `foundry.capability-invoke`.

Primary tool: `observability.poc-slo-dashboard`.

Primary Cedar policy: `pol_gtm_poc_data_room_v1`.

Primary Cedar action: `poc.dataset.import`.

Primary Cedar action: `poc.workload.execute`.

Primary Cedar action: `poc.evidence.read`.

Permit note: the policy requires tenant id, data class, purpose, retention, and named customer approver.

Permit note: the policy forbids POC data from being reused for sales collateral without explicit customer permission.

Evidence artifact: `EVT-GTM-POC-STARTED`.

Evidence artifact: `EVT-GTM-POC-RESULTS-SEALED`.

Named exit-gate: Gate C1 -- POC Success Criteria Met.

Gate C1 passes when success criteria are met, exceptions are named, customer confirms value, and the next commercial step is known.

Gate C1 fails when proof metrics miss target, integration cannot proceed, data boundary is unacceptable, or the customer will not commit to next step.

Gate C1 owner: Solutions Engineering lead.

Gate C1 approver: Sales AE Maya Lindqvist and CSM Sofia Rezende.

Metric target: POC charter approved before any customer data import.

Metric target: POC production-data retention is zero unless contractually approved.

Metric target: POC results report delivered within 2 business days of close.

Failure mode: POC becomes unpaid implementation.

Recovery: freeze new scope, re-issue charter, and move additional work to Negotiate or Onboard.

Failure mode: customer data exceeds agreed class.

Recovery: halt import, quarantine batch, and issue revised data map.

Failure mode: POC metrics are met but buyer value remains unclear.

Recovery: run executive readout with CFO Helena Brandt or CEO Aoki Tanaka style buyer.

### Stage 06 -- Negotiate

Stage name: Negotiate.

Stage owner: Sales AE Maya Lindqvist.

Supporting owners: Finance Director Mei-Ling Wu, CCO Naveen Iyer, Procurement Manager Wei Liu.

Buyer-facing objective: align scope, price, legal terms, procurement path, and success commitments.

Internal objective: convert POC value into a contractable tenant activation package.

Entry signal: POC passed Gate C1 or customer moved from Demo directly to purchase.

Entry signal: package, tier, and support plan are named.

Entry signal: legal entity and buyer authority are identified.

Negotiation item: capability tier.

Negotiation item: seat count.

Negotiation item: per-resource or per-capability commit.

Negotiation item: tenant cell posture.

Negotiation item: support response target.

Negotiation item: migration scope.

Negotiation item: data residency.

Negotiation item: regulatory pack.

Negotiation item: security terms.

Negotiation item: service credits.

Negotiation item: renewal and expansion triggers.

Required artifact: commercial proposal.

Required artifact: security response packet.

Required artifact: legal redline log.

Required artifact: procurement checklist.

Required artifact: implementation assumptions.

Required artifact: risk register for exceptions.

Primary tool: `workflow-engine.mutual-close-plan`.

Primary tool: `drive.deal-room`.

Primary tool: `policy-engine.contract-risk-review`.

Primary tool: `finops-portal.pricing-scenario`.

Primary Cedar policy: `pol_gtm_negotiation_data_room_v1`.

Primary Cedar action: `dealroom.document.share`.

Primary Cedar action: `contract.redline.review`.

Permit note: the policy scopes deal-room documents to the buyer committee and Oyatie deal team.

Permit note: the policy requires additional approval for regulated evidence, insurance, or indemnity exceptions.

Evidence artifact: `EVT-GTM-NEGOTIATION-OPENED`.

Evidence artifact: proposal version sealed.

Named exit-gate: Gate N1 -- Commercial And Legal Path Clear.

Gate N1 passes when scope, price, legal blockers, procurement steps, implementation assumptions, and target signature date are accepted.

Gate N1 fails when price is not fundable, legal risk is unacceptable, procurement path is unknown, or scope is not implementable.

Gate N1 owner: Sales AE Maya Lindqvist.

Gate N1 approver: Sales Manager Anthony Costa and Finance Director Mei-Ling Wu.

Metric target: redline response SLA is 3 business days for standard contract.

Metric target: security questionnaire response SLA is 5 business days for standard enterprise packet.

Metric target: no discount approved without expansion or term rationale.

Failure mode: buyer asks for unlimited liability.

Recovery: route to legal exception review and attach risk register.

Failure mode: procurement demands unsupported data residency.

Recovery: propose supported residency class or defer until pack roadmap lands.

Failure mode: success scope is larger than paid scope.

Recovery: move excess work to paid professional-services or phased expansion.

### Stage 07 -- Contract

Stage name: Contract.

Stage owner: Sales AE Maya Lindqvist.

Supporting owners: CCO Naveen Iyer, Finance Director Mei-Ling Wu, Procurement Manager Wei Liu.

Buyer-facing objective: execute the contract and make tenant provisioning authorized.

Internal objective: capture authoritative commercial, legal, finance, and onboarding inputs.

Entry signal: Negotiate passed Gate N1.

Entry signal: signer identity and authority are confirmed.

Entry signal: purchase order, payment method, or invoice terms are accepted.

Entry signal: tenant provisioning package is complete.

Contract artifact: master subscription agreement.

Contract artifact: data processing addendum.

Contract artifact: security addendum when required.

Contract artifact: professional-services statement of work when required.

Contract artifact: support schedule.

Contract artifact: order form.

Contract artifact: regulatory pack annex when required.

Contract artifact: migration scope annex when required.

Provisioning input: tenant legal name.

Provisioning input: tenant slug.

Provisioning input: home region.

Provisioning input: residency class.

Provisioning input: regulatory packs.

Provisioning input: initial admin identities.

Provisioning input: billing account.

Provisioning input: support contacts.

Primary tool: `workflow-engine.contract-generation`.

Primary tool: `workplace-integration.e-signature`.

Primary tool: `payments.first-payment`.

Primary tool: `drive.contract-archive`.

Primary tool: `audit-chain.contract-dual-seal`.

Primary Cedar policy: `pol_gtm_contract_signing_v1`.

Primary Cedar action: `contract.generate`.

Primary Cedar action: `contract.sign`.

Primary Cedar action: `billing.account.activate`.

Permit note: the policy requires signer authority and immutable contract archive.

Permit note: the policy forbids onboarding a tenant from unsigned commercial terms.

Evidence artifact: `EVT-GTM-CONTRACT-SIGNED`.

Evidence artifact: `EVT-GTM-BILLING-AUTHORIZED`.

Named exit-gate: Gate K1 -- Signed And Provisionable.

Gate K1 passes when signed contract, billing authority, tenant slug, region, residency, packs, admins, and support contacts are all complete.

Gate K1 fails when any provisioning input is missing or legal/finance authority is not sealed.

Gate K1 owner: Sales AE Maya Lindqvist.

Gate K1 approver: Finance Director Mei-Ling Wu and CCO Naveen Iyer.

Metric target: contract archive completeness is 100 percent.

Metric target: provisioning package handoff occurs within 1 business day of signature.

Metric target: zero tenants created without Gate K1 evidence.

Failure mode: signature completed but PO missing.

Recovery: allow Onboard only if payment terms permit invoice; otherwise hold at Contract.

Failure mode: tenant slug collides or violates reserved namespace.

Recovery: select compliant slug before onboarding.

Failure mode: DPA redline contradicts residency class.

Recovery: reopen Negotiate and resolve legal before provisioning.

### Stage 08 -- Onboard

Stage name: Onboard.

Stage owner: Customer Success Manager Sofia Rezende.

Supporting owners: Solutions Engineering lead and platform tenancy owner.

Buyer-facing objective: stand up the tenant and make initial users productive.

Internal objective: run the `onboard_saga` from Pending to Active with evidence.

Entry signal: Contract passed Gate K1.

Entry signal: kickoff meeting scheduled.

Entry signal: initial tenant admin has accepted security requirements.

Entry signal: data migration or integration plan is approved if in scope.

Onboard step: kickoff.

Onboard step: tenant create.

Onboard step: cell reserve.

Onboard step: identity bootstrap.

Onboard step: SSO and SCIM setup.

Onboard step: Cedar baseline permits.

Onboard step: regulatory pack binding.

Onboard step: workspace or workflow activation.

Onboard step: migration dry run if in scope.

Onboard step: admin training.

Onboard step: support channel setup.

Onboard step: Day 30 success-plan commit.

Primary tool: `tenancy.createPlatformTenant`.

Primary tool: `workflow-engine.onboard_saga`.

Primary tool: `identity.passkey-bootstrap`.

Primary tool: `identity.saml-scim-activation`.

Primary tool: `policy-engine.publishCedarPolicy`.

Primary tool: `observability.tenant-slo-baseline`.

Primary tool: `ops-dashboard-control-center.tenant-health`.

Primary Cedar policy: `pol_tenant_onboard_saga_v1`.

Primary Cedar action: `tenant.create`.

Primary Cedar action: `tenant.lifecycle.activate`.

Primary Cedar action: `identity.admin.bootstrap`.

Primary Cedar action: `policy.baseline.publish`.

Permit note: the policy requires signed contract evidence, operator tenant, principal id, idempotency key, and residency class.

Permit note: the policy forbids activation when a required microservice acknowledgment is missing.

Evidence artifact: `EVT-TENANT-ONBOARD-STARTED`.

Evidence artifact: `EVT-TENANT-LIFECYCLE-ACTIVE`.

Named exit-gate: Gate O1 -- Tenant Active And Admin Ready.

Gate O1 passes when tenant state is Active, admin can sign in, baseline policy evaluates, support channel exists, and Day 30 plan is accepted.

Gate O1 fails when onboard saga halts, identity is not usable, policy baseline is missing, or no customer admin is trained.

Gate O1 owner: CSM Sofia Rezende.

Gate O1 approver: platform tenancy owner.

Metric target: standard Onboard completion is within 10 business days for mid-market.

Metric target: enterprise Onboard completion is within 30 calendar days unless migration scope says otherwise.

Metric target: onboard saga acknowledgment timeout follows tenant lifecycle standard.

Failure mode: SSO metadata mismatch.

Recovery: fall back to passkey admin bootstrap, correct SSO metadata, and rerun identity smoke test.

Failure mode: microservice acknowledgment missing.

Recovery: pause activation, run compensation chain if needed, and page service owner.

Failure mode: customer admin unavailable.

Recovery: hold at Onboard and schedule replacement admin authorization.

### Stage 09 -- Active

Stage name: Active.

Stage owner: Customer Success Manager Sofia Rezende.

Supporting owner: CS-IC Lin Chen.

Buyer-facing objective: confirm recurring value and operational reliability.

Internal objective: move from project delivery to steady-state adoption and support.

Entry signal: Onboard passed Gate O1.

Entry signal: first users completed the agreed first workflow.

Entry signal: support path is tested.

Entry signal: baseline SLO and adoption metrics are visible.

Active milestone: first admin task completed.

Active milestone: first ordinary user task completed.

Active milestone: first integration or workflow event completed.

Active milestone: first audit evidence packet generated.

Active milestone: first support contact path tested.

Active milestone: first billing or FinOps report delivered if applicable.

Required artifact: success plan.

Required artifact: adoption dashboard.

Required artifact: support escalation path.

Required artifact: risk register.

Required artifact: QBR schedule.

Primary tool: `ops-dashboard-control-center.tenant-health`.

Primary tool: `observability.tenant-slo-dashboard`.

Primary tool: `finops-portal.tenant-cost-center`.

Primary tool: `audit-chain.tenant-evidence-query`.

Primary tool: `workflow-engine.success-plan`.

Primary Cedar policy: `pol_tenant_active_admin_v1`.

Primary Cedar action: `tenant.metrics.read`.

Primary Cedar action: `support.case.create`.

Primary Cedar action: `audit.evidence.read`.

Permit note: the policy grants customer admins tenant-scoped read access to adoption, support, and audit-evidence views.

Permit note: the policy forbids Oyatie support from cross-tenant inspection without JIT or break-glass evidence.

Evidence artifact: `EVT-GTM-TENANT-ACTIVE-CONFIRMED`.

Evidence artifact: Day 30 adoption snapshot.

Named exit-gate: Gate A1 -- Value In Production.

Gate A1 passes when the tenant has active users, reliable support, visible metrics, and an accepted QBR cadence.

Gate A1 fails when users are provisioned but not productive or when support cannot prove response path.

Gate A1 owner: CSM Sofia Rezende.

Gate A1 approver: Customer Success leader.

Metric target: activation-to-first-value is less than 14 days after Onboard for standard tenants.

Metric target: P1 support response matches contracted tier.

Metric target: adoption dashboard latency is under 15 minutes for daily-use metrics.

Failure mode: active users exist but champion disengages.

Recovery: escalate to executive sponsor and reset success plan.

Failure mode: support route untested.

Recovery: schedule support drill and update runbook.

Failure mode: billing surprise appears in first invoice.

Recovery: run FinOps review with Finance Director Mei-Ling Wu style owner.

### Stage 10 -- Expansion

Stage name: Expansion.

Stage owner: Customer Success Manager Sofia Rezende.

Supporting owners: Sales AE Maya Lindqvist and Customer Champion Akemi Sato.

Buyer-facing objective: expand value through new capability tiers, seats, regions, or workloads.

Internal objective: activate expansion only when usage, outcomes, and support posture justify it.

Entry signal: Active passed Gate A1.

Entry signal: customer is achieving success-plan outcomes or explicitly asks for new scope.

Entry signal: risk register does not contain unresolved blocker for new scope.

Expansion type: seat growth.

Expansion type: capability-tier upgrade.

Expansion type: regional pack activation.

Expansion type: additional microservice surface.

Expansion type: migration of another incumbent workload.

Expansion type: partner or reseller sub-tenant enablement.

Expansion type: regulated evidence uplift.

Expansion type: dedicated cell or sovereign posture.

Required artifact: expansion hypothesis.

Required artifact: usage signal.

Required artifact: customer outcome evidence.

Required artifact: commercial amendment or order form.

Required artifact: Cedar tier grant.

Primary tool: `finops-portal.usage-patterns`.

Primary tool: `analytics.expansion-signal-detect`.

Primary tool: `policy-engine.capability-tier-grant`.

Primary tool: `workflow-engine.expansion-plan`.

Primary tool: `audit-chain.expansion-evidence-seal`.

Primary Cedar policy: `pol_expansion_tier_grant_v1`.

Primary Cedar action: `capability_tier.grant`.

Primary Cedar action: `tenant.region.add`.

Primary Cedar action: `tenant.subscope.create`.

Permit note: the policy requires customer admin authority and Oyatie commercial authorization.

Permit note: the policy forbids tier grants that bypass residency, data class, or autonomy ceiling.

Evidence artifact: `EVT-GTM-EXPANSION-APPROVED`.

Evidence artifact: expansion order form or amendment.

Named exit-gate: Gate X1 -- Expansion Approved And Safe To Activate.

Gate X1 passes when new scope has usage rationale, business owner, technical owner, contract authority, and policy grant.

Gate X1 fails when expansion is merely vendor enthusiasm without customer usage, authority, or safety evidence.

Gate X1 owner: CSM Sofia Rezende.

Gate X1 approver: Sales AE Maya Lindqvist and policy owner for regulated scope.

Metric target: expansion proposal is tied to at least one named usage signal.

Metric target: no expansion grant without customer approval evidence.

Metric target: tier upgrade rollback plan is documented before activation.

Failure mode: expansion pushes tenant beyond current tier limits.

Recovery: run capability-tier sizing review before quote.

Failure mode: expansion conflicts with regional pack.

Recovery: require CCO Naveen Iyer style compliance review before activation.

Failure mode: expansion champion lacks budget.

Recovery: move to nurture until CFO or procurement path is named.

## Named Tools and Cedar Permits (specific µservices + actions + Cedar policies)

Tool: `forms.prospect-intake` captures first permitted account signal.

Tool: `crm.account-create` stores canonical account and tenant hypothesis.

Tool: `crm.lead-score` scores MQL readiness without protected-class inference.

Tool: `workflow-engine.mql-routing` routes MQLs to SDR, AE, partner, or nurture.

Tool: `workflow-engine.mutual-action-plan` holds discovery, demo, POC, legal, and onboarding milestones.

Tool: `meet.demo-room` hosts live demo with attendance and recording policy.

Tool: `tenancy.demo-sandbox` provisions synthetic demo tenant only.

Tool: `policy-engine.demo-cedar-evaluator` shows allow and deny outcomes during demo.

Tool: `audit-chain.demo-evidence-pack` seals demo evidence and post-demo recap.

Tool: `workflow-engine.poc-orchestrator` executes bounded proof plans.

Tool: `tenancy.trial-provisioning` creates trial or POC tenant under agreed scope.

Tool: `identity.saml-scim-test` validates identity integration before production.

Tool: `foundry.capability-invoke` invokes tenant-granted capabilities inside POC.

Tool: `observability.poc-slo-dashboard` exposes POC SLO and workload evidence.

Tool: `drive.deal-room` stores commercial, legal, and security documents.

Tool: `finops-portal.pricing-scenario` models tier, usage, and commit economics.

Tool: `workflow-engine.contract-generation` generates MSA, DPA, order form, and annexes.

Tool: `workplace-integration.e-signature` captures binding signatures.

Tool: `payments.first-payment` records payment or invoice authority.

Tool: `drive.contract-archive` stores immutable contract package.

Tool: `tenancy.createPlatformTenant` creates the globally unique tenant.

Tool: `workflow-engine.onboard_saga` runs Pending-to-Active sequence.

Tool: `identity.passkey-bootstrap` creates first admin authentication path.

Tool: `identity.saml-scim-activation` activates enterprise identity.

Tool: `policy-engine.publishCedarPolicy` publishes tenant baseline policy.

Tool: `observability.tenant-slo-baseline` establishes steady-state metrics.

Tool: `ops-dashboard-control-center.tenant-health` shows tenant health and adoption.

Tool: `finops-portal.tenant-cost-center` binds cost center and billing evidence.

Tool: `analytics.expansion-signal-detect` detects upgrade triggers.

Permit: `pol_gtm_prospect_intake_v1` grants `gtm.prospect.create`.

Permit: `pol_gtm_mql_scoring_v1` grants `gtm.lead.qualify`.

Permit: `pol_gtm_sql_discovery_v1` grants `gtm.opportunity.create`.

Permit: `pol_gtm_demo_sandbox_v1` grants `demo.sandbox.provision`.

Permit: `pol_gtm_demo_sandbox_v1` grants `demo.tenant_context.switch`.

Permit: `pol_gtm_poc_data_room_v1` grants `poc.dataset.import`.

Permit: `pol_gtm_poc_data_room_v1` grants `poc.workload.execute`.

Permit: `pol_gtm_poc_data_room_v1` grants `poc.evidence.read`.

Permit: `pol_gtm_negotiation_data_room_v1` grants `dealroom.document.share`.

Permit: `pol_gtm_negotiation_data_room_v1` grants `contract.redline.review`.

Permit: `pol_gtm_contract_signing_v1` grants `contract.generate`.

Permit: `pol_gtm_contract_signing_v1` grants `contract.sign`.

Permit: `pol_gtm_contract_signing_v1` grants `billing.account.activate`.

Permit: `pol_tenant_onboard_saga_v1` grants `tenant.create`.

Permit: `pol_tenant_onboard_saga_v1` grants `tenant.lifecycle.activate`.

Permit: `pol_tenant_onboard_saga_v1` grants `identity.admin.bootstrap`.

Permit: `pol_tenant_onboard_saga_v1` grants `policy.baseline.publish`.

Permit: `pol_tenant_active_admin_v1` grants `tenant.metrics.read`.

Permit: `pol_tenant_active_admin_v1` grants `support.case.create`.

Permit: `pol_tenant_active_admin_v1` grants `audit.evidence.read`.

Permit: `pol_expansion_tier_grant_v1` grants `capability_tier.grant`.

Permit: `pol_expansion_tier_grant_v1` grants `tenant.region.add`.

Permit: `pol_expansion_tier_grant_v1` grants `tenant.subscope.create`.

Forbid: every stage-level policy includes cross-tenant access refusal.

Forbid: every stage-level policy includes expired-token refusal.

Forbid: every data-bearing stage-level policy checks data class and residency.

Forbid: every production mutation requires contract or customer authorization evidence.

Forbid: every support or SE inspection of customer tenant evidence requires JIT or scoped customer approval.

## Specific Metrics + Named SLA Targets

Metric: prospect-source attribution completeness.

Target: 99 percent completeness before MQL.

Metric: consent-basis missing rate.

Target: zero missing rows at Prospect and MQL.

Metric: inbound enterprise response time.

Target: first human response within 1 business day.

Metric: partner-referred response time.

Target: first human response within 2 business days.

Metric: MQL-to-SQL conversion.

Target: 35 percent for target segments.

Metric: SQL discovery-note completion.

Target: notes complete within 4 business hours.

Metric: SQL-to-demo conversion.

Target: 70 percent for accepted SQLs.

Metric: demo recap SLA.

Target: recap delivered within 1 business day.

Metric: demo action assignment SLA.

Target: all action items assigned within 4 business hours.

Metric: POC charter compliance.

Target: 100 percent of POCs have signed charter before customer data import.

Metric: POC result-report SLA.

Target: results report delivered within 2 business days of POC close.

Metric: POC production-data retention.

Target: zero retained production-data copies unless contractually approved.

Metric: redline response SLA.

Target: 3 business days for standard contract redlines.

Metric: security questionnaire SLA.

Target: 5 business days for standard enterprise security packet.

Metric: contract archive completeness.

Target: 100 percent signed package with order form, DPA, annexes, and support schedule when applicable.

Metric: provisioning package handoff.

Target: within 1 business day of signature.

Metric: standard onboarding completion.

Target: within 10 business days for mid-market.

Metric: enterprise onboarding completion.

Target: within 30 calendar days unless migration scope states otherwise.

Metric: activation-to-first-value.

Target: less than 14 days after Onboard for standard tenants.

Metric: support response.

Target: matches contracted tier; P1 Bronze 240 minutes, Silver 120 minutes, Gold 30 minutes, Platinum 15 minutes.

Metric: tenant dashboard latency.

Target: daily-use adoption metrics visible within 15 minutes.

Metric: expansion proposal evidence.

Target: every expansion proposal tied to at least one named usage signal and one business outcome.

Metric: tier-grant safety.

Target: zero capability-tier grants without customer approval and policy evaluation.

Metric: forecast hygiene.

Target: no opportunity in Commit forecast without Gate N1 or K1 evidence.

Metric: stage aging.

Target: Prospect less than 30 days, MQL less than 14 days, SQL less than 21 days, Demo less than 14 days, POC less than 45 days unless mutual plan says otherwise.

Metric: disqualification quality.

Target: 100 percent closed-lost or nurture opportunities have one named reason.

Metric: regulated-deal evidence readiness.

Target: auditor-request packet ready within 5 business days after SQL.

## Named Failure Modes + Recovery

Failure: account lacks lawful contact basis.

Recovery: quarantine contact, remove sequence, record `EVT-GTM-CONSENT-QUARANTINE`, and restart through permitted source.

Failure: duplicate account exists under another spelling.

Recovery: merge records, preserve aliases, and reseal source attribution.

Failure: MQL score uses sensitive or protected inference.

Recovery: invalidate score, delete prohibited attribute, and rerun scoring with permitted fields only.

Failure: SQL exists without economic buyer path.

Recovery: keep in SQL only if a meeting with budget authority is scheduled; otherwise move to nurture.

Failure: demo uses unsupported claim.

Recovery: correct in meeting, send written clarification, and log capability gap.

Failure: demo sandbox accidentally includes customer confidential data.

Recovery: revoke sandbox, delete data according to policy, and restart under POC data-room permit.

Failure: POC charter lacks success metric.

Recovery: do not start workload; run charter reset with customer champion.

Failure: POC imports data beyond approved class.

Recovery: halt import, quarantine batch, notify customer approver, and update data map.

Failure: POC turns into implementation.

Recovery: freeze new asks, split paid implementation scope, and update mutual action plan.

Failure: contract legal terms contradict residency class.

Recovery: reopen negotiation and require CCO review before signature.

Failure: procurement path unknown after commercial agreement.

Recovery: keep stage at Negotiate, assign Procurement Manager Wei Liu style owner, and document steps.

Failure: tenant slug violates reserved namespace.

Recovery: reject slug, choose compliant tenant id, and update contract package if needed.

Failure: billing authority missing after signature.

Recovery: hold Onboard until invoice terms, PO, or payment method is accepted.

Failure: onboard saga fails at reserve_cell.

Recovery: select alternate supported cell or reopen Contract if residency was impossible.

Failure: onboard saga fails at create_identity_entities.

Recovery: fall back to passkey bootstrap, fix identity inputs, and rerun identity step idempotently.

Failure: downstream microservice acknowledgment times out.

Recovery: pause activation, page service owner, run compensation when required, and keep tenant Pending.

Failure: Active tenant has no business owner attending success review.

Recovery: escalate to executive sponsor and reset QBR date.

Failure: adoption metric is below plan but customer asks for expansion.

Recovery: treat as remediation, not upsell, until value is stabilized.

Failure: expansion tier exceeds current cell capacity.

Recovery: run tier-sizing review and migration plan before order form.

Failure: expansion activates regulated data without pack.

Recovery: refuse grant, open compliance pack review, and seal denial evidence.

Failure: partner-led deal hides end-customer tenant.

Recovery: reject progression until partner tenant, customer tenant, and assume-role path are explicit.

Failure: reseller tries to collapse downstream tenant records.

Recovery: enforce ADR-0313-style sovereign child tenant posture and separate audit roots.

## Sample Dialogue / Email Templates

### Template 1 -- Prospect-to-MQL outreach

Subject: Reducing integration tax across your workspace and operations stack

Hi {{first_name}},

I saw {{trigger_event}} and mapped it to a common Oyatie pattern: teams carrying separate workspace, workflow, compliance, and audit tools that do not share one tenant boundary.

Oyatie's thesis is one tenant, one identity, one audit chain, and capability tiers over shared substrate.

The first useful step is not a generic product tour.

It is a 20-minute tenant-fit screen covering your incumbent stack, jurisdiction, identity posture, and the operational problem you want to remove.

If that fit is not real, we will say so and stop.

If it is real, we will bring a solutions engineer and show the tenant-boundary and Cedar-policy path directly.

Would {{date_option_1}} or {{date_option_2}} work?

Regards,

Kofi Asante

Sales Development

### Template 2 -- SQL discovery recap

Subject: Discovery recap and proposed Oyatie evaluation path

Hi {{champion_name}},

Thank you for walking us through {{current_stack}}, {{primary_pain}}, and the {{business_deadline}} timeline.

My notes say the first tenant hypothesis is {{tenant_slug_candidate}}, primary jurisdiction {{jurisdiction}}, likely regulatory packs {{packs}}, and initial buyer committee {{buyer_committee}}.

The problem worth evaluating is {{problem_statement}}.

The success metric we heard is {{success_metric}}.

The next useful step is a demo focused on {{demo_theme}}, not a broad feature survey.

We will show tenant context, identity, Cedar default-deny, audit evidence, and the relevant capability-tier path.

We will not claim support for {{unsupported_scope}} in the demo.

Please reply with corrections before {{correction_deadline}} so the demo agenda is accurate.

Regards,

Maya Lindqvist

Account Executive

### Template 3 -- Demo follow-up and POC fork

Subject: Demo outcomes and POC decision

Hi {{champion_name}},

Today we showed {{demo_scope}} inside a synthetic tenant with no production customer data.

The strongest value signals were {{value_signal_1}}, {{value_signal_2}}, and {{value_signal_3}}.

The open technical questions are {{technical_question_1}}, {{technical_question_2}}, and {{technical_question_3}}.

Our recommendation is {{next_step}}.

If we move to POC, the POC must have a signed charter, named success metrics, data-class map, retention rule, and rollback plan before any customer data is imported.

If you prefer commercial negotiation without POC, we should confirm scope, tier, support posture, and migration assumptions first.

I attached the evidence recap and the action list.

Regards,

Maya Lindqvist

### Template 4 -- POC result readout

Subject: POC results sealed and decision path

Hi {{customer_team}},

The POC closed on {{date}} and the results package is sealed as {{evidence_id}}.

Success metric 1, {{metric_1}}, reached {{metric_1_result}} against target {{metric_1_target}}.

Success metric 2, {{metric_2}}, reached {{metric_2_result}} against target {{metric_2_target}}.

Success metric 3, {{metric_3}}, reached {{metric_3_result}} against target {{metric_3_target}}.

The exceptions are {{exceptions}}.

The recommended commercial scope is {{scope}} at tier {{tier}}, with onboarding path {{onboarding_path}} and migration path {{migration_path}}.

The work we should not include in initial contract is {{out_of_scope}}.

Please confirm whether we should schedule the executive readout with {{economic_buyer}} and {{technical_owner}}.

Regards,

Solutions Engineering

### Template 5 -- Onboard-to-Active handoff

Subject: Tenant active confirmation and Day 30 success plan

Hi {{tenant_admin}},

Your Oyatie tenant {{tenant_id}} has reached Active state.

The first admin sign-in, baseline Cedar policy evaluation, support channel, and initial SLO dashboard are complete.

The Day 30 success plan has three milestones: {{milestone_1}}, {{milestone_2}}, and {{milestone_3}}.

Your support path is {{support_path}} and your first QBR is scheduled for {{qbr_date}}.

Please reply with any admin changes before {{admin_change_deadline}}.

Regards,

Sofia Rezende

Customer Success Manager

## Cross-References

- `docs/GTM-PLAN.md` for GTM thesis, persona/channel summary, KR launch strategy, and customer success operating model.

- `docs/personas/MASTER-ROSTER-2026-05-21.md` for named personas and audience-type context.

- `specs/tenant-lifecycle.json` for Pending, Active, Suspended, Migrating, Offboarded, DeletionConfirmed, and Cancelled lifecycle semantics.

- `specs/tenant-model.json` for tenant id, parent tenant, audience type, jurisdiction, capabilities, merchant status, audit streams, KYC status, compliance packs, and DR tier.

- `docs/standards/tenant-lifecycle.md` for onboard, migrate, offboard, and delete sagas.

- `docs/standards/capability-tier-matrix.md` for Bronze, Silver, Gold, and Platinum tier responsibilities.

- `registry/capability-tiers/index.json` for tier registry authority and source counts.

- `contracts/openapi/platform/platform-tenant-v1.yaml` for `createPlatformTenant`.

- `contracts/openapi/platform/platform-policy-cedar-v1.yaml` for `publishCedarPolicy`.

- `contracts/openapi/foundry/capability-v1.yaml` for `invokeCapability`.

- `contracts/openapi/foundry/policy-v1.yaml` for Foundry autonomy-ceiling policy publication.

- `docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md` for quote-to-contract-to-payment and trial provisioning.

- `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md` for cross-tenant incident and service-credit evidence.

- `docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md` for QBR-style executive metrics and incident debrief evidence.

- `docs/standards/migration-playbook.md` for incumbent migration phases and vendor source surfaces.

- `docs/standards/cedar-policy-authoring.md` for tenant-scoped permit, cross-tenant forbid, capability-tier permit, and auditor JIT posture.

- `docs/standards/per-tenant-resource-quotas-canonical.md` for quota and backpressure concerns during Active and Expansion.

- `docs/runbooks/tenant-onboarding.md` remains a stub and should not be treated as the operational replacement for this GTM playbook.

- Stage owner checkpoint: Prospect through Demo is sales-led, POC is SE-led, Negotiate through Contract is commercial/legal-led, Onboard through Active is CS-led, and Expansion is CS plus AE co-owned.
