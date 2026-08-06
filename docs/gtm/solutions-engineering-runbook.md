---
doc_class: GoToMarketPlaybook
title: Solutions Engineering Runbook
status: Draft
date: 2026-05-20
owner: GoToMarket / Solutions Engineering
related_oyatie_adrs:
  - docs/decisions/ADR-0009-tenant-isolation-and-cell-placement.md
  - docs/decisions/ADR-0010-tenant-data-residency-and-jurisdiction.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0242-capability-tier-registry.md
  - docs/decisions/ADR-0244-business-continuity-dr-tier-registry.md
  - docs/decisions/ADR-0251-trust-score-and-eligibility-gates.md
related_personas:
  - SDR Kofi Asante
  - Sales AE Maya Lindqvist
  - Sales Manager Anthony Costa
  - Marketing Specialist Riya Sharma
  - Marketing Manager Olu Adeyemi
  - Customer Success Manager Sofia Rezende
  - CS-IC Lin Chen
  - Customer Champion Akemi Sato
  - CEO Aoki Tanaka
  - CFO Helena Brandt
  - COO Akira Watanabe
  - CTO Diego Vargas
  - CHRO Linda Foster
  - CMO Felix Ng
  - CCO Naveen Iyer
  - CISO Yuki Park
  - Procurement Manager Wei Liu
  - Finance Director Mei-Ling Wu
  - Compliance Officer Tunde Bello
  - IT Manager Jamie O'Connor
  - Business Analyst Aditya Verma
  - Data Analyst Felipe Andrade
  - External Auditor Hyo-Jin Lee
  - External Regulator Inspector Sergei Petrov
related_journeys:
  - docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md
  - docs/user-journeys/j117-api-customer-tenant-incident-response/README.md
  - docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Solutions Engineering Runbook

## Purpose

This runbook gives Solutions Engineering a complete, repeatable motion for Oyatie tenant demos, proofs of concept, and technical evaluations.

The runbook starts when Sales AE Maya Lindqvist requests technical help for a qualified opportunity.

The runbook ends when the buyer has either approved the technical path, entered commercial negotiation, or exited with a documented non-fit reason.

The runbook is designed for prospects that may become tenants under the tenant lifecycle model.

The runbook uses live product truth, not generic demo theater.

Every demo must map to a named buyer pain, a named tenant capability, a named service, a named Cedar permit, and a named validation artifact.

Every POC must have a written success plan before any environment is provisioned.

Every technical evaluation must have an owner, a risk register, an exit date, and an evidence folder.

The core SE promise is to make Oyatie easy to trust before it is easy to buy.

The SE team must demonstrate tenant isolation, governed capability activation, audit evidence, and operational supportability.

The SE team must avoid implying that a prospect has capabilities, residency, DR posture, or compliance packs before the relevant gates are approved.

The SE team must keep demos narrow enough to finish, but realistic enough for buyers to map them to their operating model.

The SE team must identify when a buyer is evaluating platform maturity, workflow value, procurement risk, or migration feasibility.

The SE team must separate discovery, demonstration, POC execution, and technical evaluation.

The SE team must never let a POC become unbounded implementation.

The SE team must never activate production-like permissions without a documented policy plan.

The SE team must protect prospect trust by answering unknowns with evidence follow-up, not improvised claims.

The SE team must hand Customer Success a complete technical context pack before contract signature when onboarding risk is material.

This document is a GTM operating artifact and not a product specification.

It references product, policy, lifecycle, and journey documents so that GTM motion stays aligned with platform architecture.

## Personas Involved (named — from MASTER-ROSTER)

- SDR Kofi Asante owns early discovery hygiene, meeting context, qualification notes, and pre-demo persona mapping.
- Sales AE Maya Lindqvist owns commercial fit, mutual action plan, buying committee alignment, and technical-resource request timing.
- Sales Manager Anthony Costa owns stage discipline, executive escalation, and deal review quality.
- Marketing Specialist Riya Sharma owns campaign-to-demo source context and persona-specific collateral matching.
- Marketing Manager Olu Adeyemi owns segment positioning, launch narrative, and campaign learning loops.
- Customer Success Manager Sofia Rezende owns onboarding-risk pre-read and post-contract continuity.
- CS-IC Lin Chen owns implementation readiness notes, technical runbook handoff, and adoption instrumentation.
- Customer Champion Akemi Sato represents a live reference voice for pragmatic trust, adoption, and internal enablement.
- CEO Aoki Tanaka evaluates strategic trust, vendor maturity, and board-visible platform direction.
- CFO Helena Brandt evaluates total cost, risk exposure, service credits, and migration economics.
- COO Akira Watanabe evaluates operational continuity, quarterly review evidence, and incident-management readiness.
- CTO Diego Vargas evaluates architecture, integration quality, scalability, reliability, and developer experience.
- CHRO Linda Foster evaluates workforce-change impact, identity governance, and rollout pacing.
- CMO Felix Ng evaluates revenue-operations workflows, campaign integration, and customer-data controls.
- CCO Naveen Iyer evaluates customer-impact controls, support experience, and escalation predictability.
- CISO Yuki Park evaluates tenant isolation, Cedar policy enforcement, auditability, encryption posture, and evidence retention.
- Procurement Manager Wei Liu evaluates vendor risk, commercial package, contract dependencies, and procurement evidence.
- Finance Director Mei-Ling Wu evaluates invoicing, cost-center assignment, FinOps reporting, and budget governance.
- Compliance Officer Tunde Bello evaluates regulatory packs, audit streams, evidence traceability, and jurisdiction controls.
- IT Manager Jamie O'Connor evaluates SSO, SCIM, network access, admin operations, migration mechanics, and support model.
- Office Manager Priya Ramanathan evaluates lightweight rollout logistics and day-to-day usability for smaller organizations.
- Business Analyst Aditya Verma evaluates workflow reports, process fit, and measurable operational outcomes.
- Data Analyst Felipe Andrade evaluates data exports, data quality, analytics access, and metric lineage.
- External Auditor Hyo-Jin Lee evaluates evidence completeness, control traceability, and audit-window retrieval.
- External Regulator Inspector Sergei Petrov evaluates jurisdiction, regulated-data controls, and pack-specific proof.

## Stages / Steps (named, sequenced)

### SE Stage 1: Intake and Technical Qualification

- Step 1.01: AE Maya Lindqvist opens the SE Assist Request with opportunity name, segment, tenant audience type, and requested demo date.
- Step 1.02: SDR Kofi Asante attaches discovery notes, primary pain, current incumbent, buying committee, and known disqualifiers.
- Step 1.03: SE confirms whether the customer profile is Fortune 500, mid-market, SMB, regulated, or mixed.
- Step 1.04: SE records the expected tenant model: standalone tenant, parent-child tenant, subsidiary tenant, sandbox tenant, or regulated tenant.
- Step 1.06: SE records the buyer's likely DR tier from the business-continuity registry.
- Step 1.07: SE records likely residency class: strict_kr, kr_with_us_failover, or global.
- Step 1.08: SE records required compliance packs, including KR privacy, financial-services, health, or internal audit if present.
- Step 1.09: SE records identity requirements such as SAML, SCIM, local admin, just-in-time provisioning, or break-glass.
- Step 1.10: SE records integration requirements such as CRM, ERP, HRIS, finance, workflow, mail, drive, or data warehouse.
- Step 1.11: SE records the prospect's incumbent stack and migration pressure.
- Step 1.12: SE records the prospect's target launch window and procurement deadline.
- Step 1.13: SE records the value hypothesis in one sentence.
- Step 1.14: SE records the technical proof hypothesis in one sentence.
- Step 1.15: SE rejects the request if no business pain is identified.
- Step 1.16: SE rejects the request if no buyer persona is identified.
- Step 1.17: SE rejects the request if the requested demo would imply unsupported regulated claims.
- Step 1.18: SE rejects the request if the POC ask lacks a timebox.
- Step 1.19: SE routes the opportunity to a Segment Playbook.
- Step 1.20: SE schedules the demo design review with AE and CSM if onboarding risk exists.

### SE Stage 2: Demo Design

- Step 2.01: SE selects one primary story and one backup story.
- Step 2.02: SE maps the primary story to a buyer persona from MASTER-ROSTER.
- Step 2.03: SE maps the primary story to one user journey.
- Step 2.04: SE maps each screen shown to one business outcome.
- Step 2.05: SE maps each permission shown to one Cedar permit.
- Step 2.06: SE maps each data boundary shown to one tenant lifecycle or residency control.
- Step 2.07: SE prepares the opening framing for the executive sponsor.
- Step 2.08: SE prepares the architecture framing for CTO Diego Vargas.
- Step 2.09: SE prepares the security framing for CISO Yuki Park.
- Step 2.10: SE prepares the compliance framing for Compliance Officer Tunde Bello.
- Step 2.11: SE prepares the finance framing for CFO Helena Brandt.
- Step 2.12: SE prepares the operations framing for COO Akira Watanabe.
- Step 2.13: SE prepares the procurement framing for Procurement Manager Wei Liu.
- Step 2.14: SE marks every demo artifact as sample, sandbox, generated, or customer-provided.
- Step 2.15: SE confirms no restricted customer data is loaded without written POC authorization.
- Step 2.16: SE confirms that every shown service has a documented API or journey anchor.
- Step 2.17: SE confirms the demo covers how the tenant becomes Active, not just how screens behave.
- Step 2.18: SE confirms the demo can be completed in the allotted meeting time.
- Step 2.19: SE confirms the demo has a recovery path if a live service is unavailable.
- Step 2.20: SE sends the agenda to AE for buyer alignment.

### SE Stage 3: Demo Execution

- Step 3.01: AE opens with the business objective and confirms the agenda.
- Step 3.02: SE states the tenant scenario, persona, service boundary, and evidence objective.
- Step 3.03: SE shows the tenant record only after naming tenant_id, home_cell, jurisdiction, and capability tier.
- Step 3.04: SE shows identity only after naming the principal role and tenant-scope check.
- Step 3.05: SE shows workflow only after naming the service action and policy guard.
- Step 3.06: SE shows audit evidence only after naming the audit stream and retrieval policy.
- Step 3.07: SE shows capability activation only after naming eligibility, trust score, tier, and approval state.
- Step 3.08: SE shows migration only after naming extract, map, import, validate, cutover, and rollback.
- Step 3.09: SE shows incident posture only after naming SLO, escalation owner, and evidence trail.
- Step 3.10: SE asks the buyer to confirm whether the demonstrated workflow matches their operating reality.
- Step 3.11: SE logs each objection as architecture, security, compliance, integration, migration, pricing, or change-management.
- Step 3.12: SE distinguishes product capability from service commitment.
- Step 3.13: SE distinguishes demo capability from purchased entitlement.
- Step 3.14: SE distinguishes contractible SLA from internal engineering objective.
- Step 3.15: SE records every follow-up owner during the call.
- Step 3.16: SE records every claim requiring evidence during the call.
- Step 3.17: SE closes with a proposed technical next step.
- Step 3.18: AE closes with commercial next step.
- Step 3.19: SE updates the opportunity record within one business day.
- Step 3.20: SE sends buyer-facing recap within one business day.

### SE Stage 4: POC Scoping

- Step 4.01: SE opens a POC Success Plan before any POC environment is created.
- Step 4.02: SE names the POC sponsor.
- Step 4.03: SE names the POC technical owner.
- Step 4.04: SE names the Oyatie SE owner.
- Step 4.05: SE names the Oyatie CSM observer if onboarding risk exists.
- Step 4.06: SE names the start date.
- Step 4.07: SE names the end date.
- Step 4.08: SE names the renewal or buying decision date tied to the POC.
- Step 4.09: SE names the maximum number of use cases.
- Step 4.10: SE names the maximum number of users.
- Step 4.11: SE names the data classification limit.
- Step 4.12: SE names the integration limit.
- Step 4.13: SE names the migration-data limit.
- Step 4.14: SE names the security-review artifacts required.
- Step 4.15: SE names the performance benchmark required.
- Step 4.16: SE names the support-response expectation.
- Step 4.17: SE names the acceptance metrics.
- Step 4.18: SE names the disqualifying conditions.
- Step 4.19: SE names the rollback and data-deletion plan.
- Step 4.20: SE obtains written agreement from AE and buyer.

### SE Stage 5: POC Provisioning

- Step 5.01: SE requests a sandbox tenant through the platform tenant service.
- Step 5.02: SE uses a tenant_id that cannot be confused with production.
- Step 5.03: SE sets legal_name to the approved POC entity name.
- Step 5.04: SE sets home_region according to residency expectations.
- Step 5.05: SE sets residency_class according to the POC success plan.
- Step 5.06: SE attaches only approved regulatory_packs.
- Step 5.07: SE records parent_tenant_id if the POC models a subsidiary structure.
- Step 5.08: SE records audience_type according to prospect segment.
- Step 5.09: SE records home_cell and dr_cell.
- Step 5.10: SE records finops_cost_center for internal cost tracking.
- Step 5.11: SE activates demo-scoped identity principals only.
- Step 5.12: SE activates demo-scoped service capabilities only.
- Step 5.13: SE publishes POC Cedar policies through the policy publishing surface.
- Step 5.14: SE validates tenant-scope enforcement.
- Step 5.15: SE validates CI-scope automation enforcement.
- Step 5.16: SE validates auditor-scope read-only enforcement.
- Step 5.17: SE validates no wildcard action or resource exists.
- Step 5.18: SE validates no tenant-admin superuser bypass exists.
- Step 5.19: SE validates POC audit streams are active.
- Step 5.20: SE records provisioning evidence in the POC evidence folder.

### SE Stage 6: POC Execution

- Step 6.01: SE holds a kickoff with buyer sponsor and technical owner.
- Step 6.02: SE reviews the POC success plan line by line.
- Step 6.03: SE confirms the no-production-data rule unless an approved exception exists.
- Step 6.04: SE confirms the support channel.
- Step 6.05: SE confirms the daily issue triage owner.
- Step 6.06: SE confirms the weekly checkpoint cadence.
- Step 6.07: SE tracks use-case completion by named milestone.
- Step 6.08: SE tracks capability usage by tenant.
- Step 6.09: SE tracks policy-deny events by policy id.
- Step 6.10: SE tracks migration import errors by source object.
- Step 6.11: SE tracks data-quality exceptions by mapped field.
- Step 6.12: SE tracks latency and throughput by workflow.
- Step 6.13: SE tracks buyer enablement completion.
- Step 6.14: SE tracks stakeholder attendance.
- Step 6.15: SE records unresolved technical risks.
- Step 6.16: SE records commercial blockers discovered during POC.
- Step 6.17: SE records compliance blockers discovered during POC.
- Step 6.18: SE escalates P1 blockers the same business day.
- Step 6.19: SE escalates scope expansion requests to AE.
- Step 6.20: SE updates evidence before each checkpoint.

### SE Stage 7: Technical Evaluation

- Step 7.01: SE opens the technical evaluation matrix.
- Step 7.02: SE records architecture criteria.
- Step 7.03: SE records security criteria.
- Step 7.04: SE records compliance criteria.
- Step 7.05: SE records identity criteria.
- Step 7.06: SE records integration criteria.
- Step 7.07: SE records migration criteria.
- Step 7.08: SE records reliability criteria.
- Step 7.09: SE records support criteria.
- Step 7.10: SE records procurement criteria.
- Step 7.11: SE attaches the tenant-isolation evidence packet.
- Step 7.12: SE attaches the Cedar policy evidence packet.
- Step 7.13: SE attaches the audit evidence packet.
- Step 7.14: SE attaches the capability-tier mapping.
- Step 7.15: SE attaches the incident-response mapping.
- Step 7.16: SE attaches the migration mapping if applicable.
- Step 7.17: SE marks each criterion pass, conditional pass, fail, or not evaluated.
- Step 7.18: SE documents every conditional pass with contract, onboarding, product, or migration dependency.
- Step 7.19: SE obtains buyer technical-owner signoff.
- Step 7.20: SE sends technical signoff to AE and CSM.

### SE Stage 8: Closeout and Handoff

- Step 8.01: SE writes the Demo Recap or POC Closeout Summary.
- Step 8.02: SE names the buyer pain validated.
- Step 8.03: SE names the product fit validated.
- Step 8.04: SE names the product gaps found.
- Step 8.05: SE names the policy gaps found.
- Step 8.06: SE names the migration gaps found.
- Step 8.07: SE names the security conditions remaining.
- Step 8.08: SE names the commercial conditions remaining.
- Step 8.09: SE names the onboarding assumptions.
- Step 8.10: SE names the buyer success metrics.
- Step 8.11: SE names the tenant lifecycle state expected at contract.
- Step 8.12: SE names the capability tier expected at launch.
- Step 8.13: SE names the compliance packs expected at launch.
- Step 8.14: SE names the support model expected at launch.
- Step 8.15: SE attaches all evidence links.
- Step 8.16: SE flags any claims that must not be repeated without caveat.
- Step 8.17: SE briefs CSM Sofia Rezende before signature if onboarding risk is medium or high.
- Step 8.18: SE briefs CS-IC Lin Chen before kickoff if implementation sequencing is unusual.
- Step 8.19: SE archives the POC environment according to the data-deletion plan.
- Step 8.20: SE marks the technical evaluation complete.

### Fortune 500 Playbook: Executive Trust Lab

- Fortune 500 Step F5-01: Lead with board-visible risk, multi-tenant governance, and executive operating continuity.
- Fortune 500 Step F5-02: Invite CEO Aoki Tanaka, CFO Helena Brandt, COO Akira Watanabe, CTO Diego Vargas, and CISO Yuki Park to the right sections rather than one overloaded session.
- Fortune 500 Step F5-03: Use a multi-session demo arc: executive value, architecture, security, migration, and operating review.
- Fortune 500 Step F5-04: Show parent-child tenant hierarchy when subsidiaries or regions matter.
- Fortune 500 Step F5-05: Show strict tenant-scoped access decisions.
- Fortune 500 Step F5-06: Show capability-tier entitlement checks.
- Fortune 500 Step F5-08: Show incident debrief flow using the COO quarterly ops review journey.
- Fortune 500 Step F5-09: Show FinOps cost-center reporting for Finance Director Mei-Ling Wu.
- Fortune 500 Step F5-10: Show procurement evidence for Procurement Manager Wei Liu.
- Fortune 500 Step F5-11: Keep POC scope to one division, one workflow, one identity model, and one reporting outcome.
- Fortune 500 Step F5-12: Require named executive sponsor before any POC beyond 30 days.
- Fortune 500 Step F5-13: Require security review artifacts before regulated data enters any test environment.
- Fortune 500 Step F5-14: Require architecture signoff before integration buildout.
- Fortune 500 Step F5-15: Require migration source-owner availability before migration dry run.
- Fortune 500 Step F5-17: Use legal and procurement timeline in mutual action plan.
- Fortune 500 Step F5-18: Treat unknown subsidiary data residency as a gating issue.
- Fortune 500 Step F5-19: Treat unsupported custom entitlement models as product-gap risk.
- Fortune 500 Step F5-20: Close with technical-signoff criteria and executive next meeting.

### Mid-Market Playbook: Controlled Business Value Sprint

- Mid-Market Step MM-01: Lead with speed to value, controlled integration, and measurable operating improvement.
- Mid-Market Step MM-02: Invite COO Akira Watanabe, CTO Diego Vargas, CISO Yuki Park, Finance Director Mei-Ling Wu, and IT Manager Jamie O'Connor as needed.
- Mid-Market Step MM-03: Use one demo session and one technical follow-up unless risk demands more.
- Mid-Market Step MM-04: Show quote-to-contract-to-payment journey when revenue operations matter.
- Mid-Market Step MM-05: Show tenant lifecycle states with emphasis on Pending to Active.
- Mid-Market Step MM-06: Show SAML or SCIM only if buyer actually needs it.
- Mid-Market Step MM-08: Show migration dry-run results for the top incumbent source.
- Mid-Market Step MM-09: Show admin operations for IT Manager Jamie O'Connor.
- Mid-Market Step MM-10: Show adoption dashboards for Business Analyst Aditya Verma.
- Mid-Market Step MM-11: Keep POC scope to two workflows, one source system, and one reporting surface.
- Mid-Market Step MM-12: Require technical owner and business sponsor before POC provisioning.
- Mid-Market Step MM-13: Require target launch date and decision date.
- Mid-Market Step MM-14: Require support model clarity before close.
- Mid-Market Step MM-15: Use weekly POC checkpoints.
- Mid-Market Step MM-16: Escalate scope creep to AE within one business day.
- Mid-Market Step MM-17: Tie every success metric to a business owner.
- Mid-Market Step MM-18: Treat missing data owner as a POC blocker.
- Mid-Market Step MM-19: Treat no-show technical owner as risk signal.
- Mid-Market Step MM-20: Close with implementation readiness, not abstract interest.

### SMB Playbook: Fast Fit and Safe Launch

- SMB Step SMB-01: Lead with simple setup, clear pricing, and low operational drag.
- SMB Step SMB-02: Invite CEO Aoki Tanaka, Office Manager Priya Ramanathan, IT Manager Jamie O'Connor, and Finance Director Mei-Ling Wu when relevant.
- SMB Step SMB-03: Use one tightly scripted demo.
- SMB Step SMB-04: Avoid deep architecture unless buyer asks or risk requires it.
- SMB Step SMB-05: Show core workflow in fewer than fifteen minutes.
- SMB Step SMB-06: Show admin setup in fewer than ten minutes.
- SMB Step SMB-07: Show support path in fewer than five minutes.
- SMB Step SMB-09: Avoid promising custom integrations during demo.
- SMB Step SMB-10: Avoid POC unless deal risk justifies it.
- SMB Step SMB-11: Use trial provisioning when available and appropriate.
- SMB Step SMB-12: Keep evaluation to a launch-readiness checklist.
- SMB Step SMB-13: Require one buyer owner and one admin owner.
- SMB Step SMB-14: Require payment and contract path clarity before onboarding.
- SMB Step SMB-15: Require migration source export before scheduling cutover.
- SMB Step SMB-16: Offer guided launch rather than custom build.
- SMB Step SMB-17: Treat lack of owner as non-fit until resolved.
- SMB Step SMB-18: Treat custom compliance needs as escalation to regulated playbook.
- SMB Step SMB-19: Close with launch date, package, and support channel.
- SMB Step SMB-20: Handoff to CSM with minimal but complete tenant context.

### Regulated Playbook: Evidence-First Technical Evaluation

- Regulated Step REG-01: Lead with jurisdiction, data classification, audit streams, and policy enforcement.
- Regulated Step REG-02: Invite CISO Yuki Park, Compliance Officer Tunde Bello, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, CTO Diego Vargas, and Procurement Manager Wei Liu.
- Regulated Step REG-03: Do not demo regulated claims without the relevant pack named.
- Regulated Step REG-04: Do not load regulated data without written authorization and policy approval.
- Regulated Step REG-05: Show tenant model and residency class before workflow value.
- Regulated Step REG-06: Show Cedar policy enforcement before convenience features.
- Regulated Step REG-07: Show audit evidence retrieval.
- Regulated Step REG-08: Show denied access cases, not only allowed cases.
- Regulated Step REG-09: Show destructive-action authority checks.
- Regulated Step REG-10: Show data-class required attributes.
- Regulated Step REG-12: Require compliance-owner signoff before POC provisioning.
- Regulated Step REG-13: Require security-owner signoff before integration.
- Regulated Step REG-14: Require migration chain-of-custody plan before dry run.
- Regulated Step REG-15: Require evidence folder structure before POC kickoff.
- Regulated Step REG-16: Require incident response and notification path review.
- Regulated Step REG-17: Treat missing residency answer as a blocker.
- Regulated Step REG-18: Treat missing data classification as a blocker.
- Regulated Step REG-19: Treat unsupported audit retention as a commercial and product risk.
- Regulated Step REG-20: Close with evidence review and conditional technical approval.

### Demo Track A: Quote-to-Contract-to-Payment

- Demo A.01: Use when buyer pain is revenue operations, contract delay, payment control, or customer onboarding.
- Demo A.02: Anchor to journey j54.
- Demo A.03: Name the prospect buying Marcus's SaaS under a new tenant.
- Demo A.04: Show quote request through forms.
- Demo A.05: Show quote delivery through mail.
- Demo A.06: Show contract generation through workflow-engine.
- Demo A.07: Show signature through workplace integration.
- Demo A.08: Show first payment through payments.
- Demo A.09: Show contract archive through drive.
- Demo A.10: Show trial provisioning through tenancy.
- Demo A.11: Show buyer principal through identity.
- Demo A.12: Name Cedar policy `cedar.tenant.quote_contract_payment.actor`.
- Demo A.13: Name action `forms:SubmitQuoteRequest`.
- Demo A.14: Name action `workflow-engine:GenerateContract`.
- Demo A.15: Name action `payments:CollectFirstPayment`.
- Demo A.16: Name action `tenancy:ProvisionTrialTenant`.
- Demo A.17: Use metric quote-to-contract cycle time.
- Demo A.18: Use metric first-payment completion rate.
- Demo A.19: Use failure mode payment authorization mismatch.
- Demo A.20: Close by asking whether this journey matches the buyer's revenue handoff.

### Demo Track B: Incident Response and SLO Credit

- Demo B.01: Use when buyer pain is reliability, escalation, customer trust, or support predictability.
- Demo B.02: Anchor to journey j117.
- Demo B.03: Show observability event detection.
- Demo B.04: Show workflow-engine incident case creation.
- Demo B.05: Show messenger customer notification.
- Demo B.06: Show mail executive summary.
- Demo B.07: Show finops-portal credit estimate.
- Demo B.08: Show payments credit workflow when applicable.
- Demo B.09: Show audit chain evidence.
- Demo B.10: Show runbook owner assignment.
- Demo B.11: Show support handoff to CCO Naveen Iyer.
- Demo B.12: Name Cedar policy `cedar.tenant.incident_response.read_write`.
- Demo B.13: Name action `observability:ReadIncident`.
- Demo B.14: Name action `workflow-engine:OpenIncidentCase`.
- Demo B.15: Name action `finops-portal:EstimateServiceCredit`.
- Demo B.16: Name action `messenger:NotifyTenantContact`.
- Demo B.17: Use metric P1 acknowledgement time.
- Demo B.18: Use metric customer notification latency.
- Demo B.19: Use failure mode unverified incident scope.
- Demo B.20: Close by asking whether the buyer's escalation matrix can map to this flow.

### Demo Track C: Quarterly Operations Review

- Demo C.01: Use when buyer pain is operational control, leadership reporting, incident learning, or capacity planning.
- Demo C.02: Anchor to journey j168.
- Demo C.03: Show COO quarterly ops review dashboard.
- Demo C.04: Show latency p99 trend.
- Demo C.05: Show throughput trend.
- Demo C.06: Show error budget burn.
- Demo C.07: Show capacity utilization.
- Demo C.08: Show customer NPS.
- Demo C.09: Show on-call burnout signal.
- Demo C.10: Show incident debrief timeline.
- Demo C.11: Show service-credit summary.
- Demo C.12: Name Cedar policy `cedar.tenant.qbr.ops_review.viewer`.
- Demo C.13: Name action `ops-dashboard-control-center:ViewQuarterlyReview`.
- Demo C.14: Name action `incident-management:ReadDebrief`.
- Demo C.15: Name action `slo-budgets:ReadBudget`.
- Demo C.16: Name action `audit-chain:ReadEvidence`.
- Demo C.17: Use metric QBR evidence completeness.
- Demo C.18: Use metric incident corrective-action closure.
- Demo C.19: Use failure mode executive dashboard metric mismatch.
- Demo C.20: Close by asking which operational review pack would replace the buyer's current meeting.

### POC Success Plan Requirements

- POC Requirement 01: Opportunity name is recorded.
- POC Requirement 02: Prospect legal name is recorded.
- POC Requirement 03: Segment is recorded.
- POC Requirement 04: Primary persona is recorded.
- POC Requirement 05: Sponsor is recorded.
- POC Requirement 06: Technical owner is recorded.
- POC Requirement 07: Commercial owner is recorded.
- POC Requirement 08: Compliance owner is recorded if regulated.
- POC Requirement 09: Security owner is recorded if security review is active.
- POC Requirement 10: POC tenant_id is recorded.
- POC Requirement 11: home_cell is recorded.
- POC Requirement 12: dr_cell is recorded if DR is in scope.
- POC Requirement 13: jurisdiction is recorded.
- POC Requirement 14: residency_class is recorded.
- POC Requirement 15: capability tier is recorded.
- POC Requirement 16: compliance packs are recorded.
- POC Requirement 17: data classification is recorded.
- POC Requirement 18: identity model is recorded.
- POC Requirement 19: integration list is recorded.
- POC Requirement 20: migration source is recorded.
- POC Requirement 21: use-case list is recorded.
- POC Requirement 22: out-of-scope list is recorded.
- POC Requirement 23: acceptance metrics are recorded.
- POC Requirement 24: acceptance threshold is recorded.
- POC Requirement 25: success owner is recorded.
- POC Requirement 26: daily issue owner is recorded.
- POC Requirement 27: support channel is recorded.
- POC Requirement 28: weekly checkpoint calendar is recorded.
- POC Requirement 29: evidence folder is recorded.
- POC Requirement 30: rollback plan is recorded.
- POC Requirement 31: data deletion date is recorded.
- POC Requirement 32: commercial decision date is recorded.
- POC Requirement 33: legal review dependency is recorded.
- POC Requirement 34: procurement review dependency is recorded.
- POC Requirement 35: onboarding dependency is recorded.
- POC Requirement 36: product dependency is recorded.
- POC Requirement 37: known blocker is recorded.
- POC Requirement 38: non-fit condition is recorded.
- POC Requirement 39: executive readout date is recorded.
- POC Requirement 40: closeout owner is recorded.

### Technical Evaluation Matrix

- Evaluation Criterion 01: Tenant isolation can be explained with tenant_id, parent_tenant_id, home_cell, and jurisdiction.
- Evaluation Criterion 02: Tenant lifecycle can be explained from Pending to Active.
- Evaluation Criterion 03: Sandbox tenant creation can be traced to request id and idempotency key.
- Evaluation Criterion 04: Capability activation can be traced to tier grant and eligibility gate.
- Evaluation Criterion 05: Trust score dependency can be explained.
- Evaluation Criterion 06: KYC status dependency can be explained.
- Evaluation Criterion 07: Compliance pack dependency can be explained.
- Evaluation Criterion 08: Cedar tenant-scope permit can be shown.
- Evaluation Criterion 09: Cedar auditor-scope permit can be shown.
- Evaluation Criterion 10: Cedar CI-scope permit can be shown.
- Evaluation Criterion 11: Regulated data permit can be shown when relevant.
- Evaluation Criterion 12: Destructive action authority check can be shown when relevant.
- Evaluation Criterion 13: No wildcard policy rule is present.
- Evaluation Criterion 14: No superuser bypass is present.
- Evaluation Criterion 15: SAML or SCIM path is explained when relevant.
- Evaluation Criterion 16: Admin role boundary is explained.
- Evaluation Criterion 17: Audit stream retrieval is explained.
- Evaluation Criterion 18: Evidence retention by tier is explained.
- Evaluation Criterion 19: Migration dry run can be explained.
- Evaluation Criterion 20: Cutover rollback can be explained.
- Evaluation Criterion 21: P1 support response target is explained by tier.
- Evaluation Criterion 22: Availability target is explained by tier.
- Evaluation Criterion 23: RPO target is explained by tier.
- Evaluation Criterion 24: RTO target is explained by tier.
- Evaluation Criterion 25: Rate limit target is explained by tier.
- Evaluation Criterion 26: Seat limit target is explained by tier.
- Evaluation Criterion 27: Cost-center reporting is explained when relevant.
- Evaluation Criterion 28: Incident-response path is explained.
- Evaluation Criterion 29: QBR evidence path is explained.
- Evaluation Criterion 30: Contract handoff dependencies are explained.

## Named Tools and Cedar Permits (specific µservices + actions + Cedar policies)

### Named Tools

- Tool: `platform-tenant-service`; Purpose: create and inspect sandbox or contract-bound tenant records; Primary action: `platform-tenant:createPlatformTenant`.
- Tool: `platform-policy-cedar-service`; Purpose: publish and review Cedar policy versions; Primary action: `platform-policy:publishCedarPolicy`.
- Tool: `foundry-capability-service`; Purpose: invoke tenant-granted capability during POC; Primary action: `foundry-capability:invokeCapability`.
- Tool: `foundry-policy-service`; Purpose: publish autonomy ceiling for Foundry workflows; Primary action: `foundry-policy:publishFoundryPolicyAutonomyCeiling`.
- Tool: `identity-service`; Purpose: create buyer principals, admin principals, and demo role bindings; Primary action: `identity:BindTenantPrincipal`.
- Tool: `tenancy-service`; Purpose: provision trial or sandbox tenant context; Primary action: `tenancy:ProvisionTrialTenant`.
- Tool: `workflow-engine`; Purpose: execute contract, incident, onboarding, and approval workflows; Primary action: `workflow-engine:RunWorkflow`.
- Tool: `payments-service`; Purpose: demonstrate first payment, subscription, credit, and invoice flows; Primary action: `payments:CollectFirstPayment`.
- Tool: `mail-service`; Purpose: demonstrate quote delivery, recap delivery, and executive notification; Primary action: `mail:SendTenantMessage`.
- Tool: `messenger-service`; Purpose: demonstrate incident and onboarding notifications; Primary action: `messenger:NotifyTenantContact`.
- Tool: `drive-service`; Purpose: archive contracts and evidence; Primary action: `drive:ArchiveTenantArtifact`.
- Tool: `audit-chain-service`; Purpose: retrieve demo and POC evidence; Primary action: `audit-chain:ReadEvidence`.
- Tool: `observability-service`; Purpose: show service health, latency, and incident evidence; Primary action: `observability:ReadIncident`.
- Tool: `ops-dashboard-control-center`; Purpose: show operational review and health posture; Primary action: `ops-dashboard-control-center:ViewQuarterlyReview`.
- Tool: `finops-portal`; Purpose: show cost-center, service-credit, and usage-cost evidence; Primary action: `finops-portal:ReadTenantCostCenter`.
- Tool: `slo-budgets-service`; Purpose: show error budget, SLA, and credit context; Primary action: `slo-budgets:ReadBudget`.
- Tool: `migration-discovery-service`; Purpose: discover source systems during POC; Primary action: `oya.migration.discover`.
- Tool: `migration-export-service`; Purpose: extract approved sample data; Primary action: `oya.migration.export`.
- Tool: `migration-transform-service`; Purpose: map source objects to Oyatie objects; Primary action: `oya.migration.transform`.
- Tool: `migration-import-service`; Purpose: import scoped data into sandbox; Primary action: `oya.migration.import`.
- Tool: `migration-validate-service`; Purpose: verify imported data completeness; Primary action: `oya.migration.validate`.
- Tool: `migration-cutover-service`; Purpose: model or execute cutover; Primary action: `oya.migration.cutover`.

### Named Cedar Permits

- Cedar Permit: `cedar.demo.tenant_scope.viewer`; Allows `Action::"ViewTenantDemo"` when principal tenant equals resource tenant.
- Cedar Permit: `cedar.demo.tenant_admin.limited`; Allows `Action::"ManageDemoTenant"` only for approved demo tenant resources.
- Cedar Permit: `cedar.demo.quote_contract_payment.actor`; Allows quote, contract, and first-payment demo actions in journey j54.
- Cedar Permit: `cedar.demo.incident_response.operator`; Allows incident case read/write for journey j117 sandbox resources.
- Cedar Permit: `cedar.demo.qbr.ops_review.viewer`; Allows QBR dashboard viewing for journey j168 sandbox resources.
- Cedar Permit: `cedar.poc.capability_tier.grant_viewer`; Allows viewing capability tier grants during POC evaluation.
- Cedar Permit: `cedar.poc.capability_tier.activator`; Allows `Action::"GrantCapabilityTier"` only when eligibility and trust-score requirements pass.
- Cedar Permit: `cedar.poc.audit_evidence.reader`; Allows `Action::"ReadAuditEvidence"` for assigned auditor and compliance principals.
- Cedar Permit: `cedar.poc.regulated_data.reader`; Allows regulated-data reads only when data_class and regulatory_pack attributes match.
- Cedar Permit: `cedar.poc.migration.operator`; Allows migration discover, export, transform, import, validate, and cutover actions inside the POC scope.
- Cedar Permit: `cedar.poc.identity.admin`; Allows identity role binding only for approved POC principals.
- Cedar Permit: `cedar.poc.destructive_action.guard`; Allows destructive cleanup only when authority context and approval id are present.
- Cedar Permit: `cedar.poc.ci_scope.deployer`; Allows CI automation to deploy POC configuration for the specific tenant.
- Cedar Permit: `cedar.poc.support.viewer`; Allows support read-only diagnostics during POC.
- Cedar Permit: `cedar.poc.finops.viewer`; Allows finance principals to view tenant cost-center evidence.
- Cedar Permit: `cedar.poc.policy.publisher`; Allows policy publication only through approved platform-policy path.
- Cedar Permit: `cedar.poc.external_auditor.reader`; Allows external auditor evidence retrieval without operational write access.
- Cedar Permit: `cedar.poc.procurement.viewer`; Allows procurement principals to view vendor-risk evidence pack.
- Cedar Permit: `cedar.poc.security_reviewer.reader`; Allows security reviewer to inspect policy and audit artifacts.
- Cedar Permit: `cedar.poc.closeout.archiver`; Allows SE to archive closeout artifacts and trigger sandbox data deletion.

## Specific Metrics + Named SLA Targets

### Demo Metrics

- Metric: Demo Preparation Completion; Target: 100 percent of scheduled demos have agenda, persona map, policy map, and recovery plan by one business day before demo.
- Metric: Demo Business-Pain Fit; Target: 90 percent of demos start with a validated business pain recorded by SDR or AE.
- Metric: Demo Evidence Follow-Up Latency; Target: all evidence follow-ups assigned before call end and sent within one business day.
- Metric: Demo No-Unsupported-Claim Rate; Target: zero unsupported regulated, SLA, residency, or tier claims.
- Metric: Demo Technical Next-Step Rate; Target: 70 percent of qualified demos exit with technical next step or documented non-fit.
- Metric: Demo Persona Coverage; Target: every demo names at least one buyer persona and one technical persona.
- Metric: Demo Policy Traceability; Target: every permission shown maps to a named Cedar permit.
- Metric: Demo Service Traceability; Target: every workflow shown maps to a named service.
- Metric: Demo Recovery Readiness; Target: every live demo has backup artifact or recorded path.
- Metric: Demo Recap Timeliness; Target: buyer-facing recap sent within one business day.

### POC Metrics

- Metric: POC Success Plan Completeness; Target: 100 percent before provisioning.
- Metric: POC Provisioning Lead Time; Target: sandbox tenant available within two business days after approved request for standard scope.
- Metric: POC Kickoff Timeliness; Target: kickoff held within three business days of provisioning.
- Metric: POC Weekly Checkpoint Completion; Target: 100 percent of agreed checkpoints held or rescheduled with notes.
- Metric: POC Acceptance Metric Coverage; Target: 100 percent of success metrics measured before closeout.
- Metric: POC Scope Control; Target: no more than one approved scope change per POC without executive review.
- Metric: POC Policy-Deny Review; Target: all unexpected denies triaged within one business day.
- Metric: POC Migration Error Triage; Target: top-severity import errors triaged within one business day.
- Metric: POC Closeout Timeliness; Target: closeout summary sent within two business days after final checkpoint.
- Metric: POC Sandbox Deletion Evidence; Target: deletion or retention exception recorded within five business days after closeout.

### Technical Evaluation Metrics

- Metric: Security Questionnaire First Response; Target: initial complete response within five business days for standard package.
- Metric: Architecture Review Completion; Target: architecture criteria resolved before commercial signature when CTO signoff is a condition.
- Metric: Compliance Evidence Completeness; Target: all requested available evidence mapped to regulatory pack and audit stream.
- Metric: Procurement Evidence Completeness; Target: vendor-risk packet complete before procurement review meeting.
- Metric: Residual Risk Count; Target: zero high residual risks without named mitigation before contract.
- Metric: Conditional Pass Clarity; Target: every conditional pass has owner, due date, and contract or onboarding dependency.
- Metric: Security Owner Signoff; Target: written signoff or written objection before negotiation exits.
- Metric: Technical Owner Signoff; Target: written signoff before POC success is claimed.
- Metric: Handoff Completeness; Target: CSM receives technical context pack within one business day after technical closeout.
- Metric: Evidence Retrieval SLA; Target: SE can retrieve demo, POC, and evaluation evidence within one business day.

### Tier-Specific Targets Used in SE Conversations


## Named Failure Modes + Recovery

### Demo Failure Modes

- Failure Mode: `DEMO-UNQUALIFIED-PAIN`; Signal: buyer cannot name the workflow or risk being evaluated; Recovery: pause demo, return to discovery, and reschedule if no pain emerges.
- Failure Mode: `DEMO-WRONG-PERSONA`; Signal: content targets CTO while actual buyer is CFO or COO; Recovery: switch to relevant persona track and log missing stakeholder.
- Failure Mode: `DEMO-UNSUPPORTED-CLAIM`; Signal: SE states a residency, SLA, or regulated capability not tied to evidence; Recovery: correct in-call, send written clarification, and notify AE.
- Failure Mode: `DEMO-POLICY-BLUR`; Signal: permissions are shown as generic admin power; Recovery: name the Cedar permit and show allow/deny boundary.
- Failure Mode: `DEMO-SERVICE-OUTAGE`; Signal: live service unavailable; Recovery: use approved backup artifact and schedule evidence follow-up.
- Failure Mode: `DEMO-DATA-CONFUSION`; Signal: buyer mistakes sample data for real customer data; Recovery: restate sandbox status and artifact provenance.
- Failure Mode: `DEMO-NO-NEXT-STEP`; Signal: call ends without technical owner or next checkpoint; Recovery: AE schedules technical fit call within two business days.
- Failure Mode: `DEMO-SECURITY-DERAIL`; Signal: security review consumes value demo; Recovery: split security deep dive with CISO Yuki Park and keep business agenda intact.
- Failure Mode: `DEMO-MIGRATION-OVERPROMISE`; Signal: SE implies unsupported source can be migrated with no analysis; Recovery: route to migration discovery and define dry-run scope.

### POC Failure Modes

- Failure Mode: `POC-NO-SUCCESS-PLAN`; Signal: provisioning requested without written criteria; Recovery: block provisioning until plan is approved.
- Failure Mode: `POC-SCOPE-CREEP`; Signal: buyer adds workflows, users, or sources beyond plan; Recovery: log change request and require AE plus sponsor approval.
- Failure Mode: `POC-PRODUCTION-DATA-RISK`; Signal: restricted data appears without authorization; Recovery: suspend ingestion, quarantine data, and review policy authorization.
- Failure Mode: `POC-MISSING-SPONSOR`; Signal: business sponsor misses checkpoints; Recovery: escalate to AE and pause expansion of technical work.
- Failure Mode: `POC-MISSING-TECH-OWNER`; Signal: no buyer owner can validate results; Recovery: freeze new work until owner is named.
- Failure Mode: `POC-POLICY-DENY-NOISE`; Signal: deny events obscure real workflow progress; Recovery: triage policy, separate expected from unexpected denies, and update evidence.
- Failure Mode: `POC-MIGRATION-MAP-DRIFT`; Signal: source fields change after mapping; Recovery: re-run discovery, version mapping, and reset acceptance baseline.
- Failure Mode: `POC-INTEGRATION-BLOCKED`; Signal: source API credentials or network access unavailable; Recovery: move to mocked integration only if buyer signs off on reduced proof.
- Failure Mode: `POC-UNMEASURED-SUCCESS`; Signal: final readout lacks acceptance metrics; Recovery: extend only to measure agreed metrics or mark POC inconclusive.
- Failure Mode: `POC-CLOSEOUT-ORPHANED`; Signal: no summary or deletion evidence after final meeting; Recovery: SE manager enforces closeout within two business days.

### Technical Evaluation Failure Modes

- Failure Mode: `TECH-EVAL-QUESTIONNAIRE-DRIFT`; Signal: buyer sends new questionnaire after signoff target; Recovery: separate must-answer from nice-to-have and update timeline.
- Failure Mode: `TECH-EVAL-RESIDENCY-UNKNOWN`; Signal: buyer cannot choose residency class; Recovery: schedule jurisdiction workshop with compliance and legal stakeholders.
- Failure Mode: `TECH-EVAL-AUDIT-GAP`; Signal: requested audit evidence does not map to current pack; Recovery: mark conditional, identify pack or product dependency, and avoid pass claim.
- Failure Mode: `TECH-EVAL-SECURITY-NO-OWNER`; Signal: security asks questions through AE only; Recovery: require named CISO delegate for technical closure.
- Failure Mode: `TECH-EVAL-ARCHITECTURE-MISMATCH`; Signal: buyer expects single-tenant deployment where offer is multi-tenant cell placement; Recovery: explain tenant isolation and escalate package fit.
- Failure Mode: `TECH-EVAL-CUSTOM-POLICY-DEMAND`; Signal: buyer demands bespoke policy model in POC; Recovery: map to Cedar extensibility or mark product gap.
- Failure Mode: `TECH-EVAL-UNOWNED-CONDITION`; Signal: conditional pass has no owner; Recovery: block technical closeout until owner and date are named.
- Failure Mode: `TECH-EVAL-REFERENCE-NEEDED`; Signal: buyer requires reference before signoff; Recovery: coordinate with AE and Customer Champion Akemi Sato if approved.
- Failure Mode: `TECH-EVAL-HANDOFF-GAP`; Signal: CSM receives no technical context; Recovery: SE writes handoff before contract or flags onboarding risk.

## Sample Dialogue / Email Templates

### Template 1: SE Demo Confirmation Email

Subject: Oyatie technical demo agenda and proof points

Hi {{buyer_name}},

Ahead of our session on {{date}}, we will focus the demo on {{business_pain}} for {{persona_or_team}}.

The planned flow is:

1. Tenant scenario and operating context.
2. Workflow walkthrough for {{journey_name}}.
3. Policy and access evidence for the actions shown.
4. Technical questions and next-step alignment.

We will keep the session grounded in sandbox data and will call out where an item is a package entitlement, a policy requirement, or an onboarding dependency.

The target outcome is to decide whether a scoped POC or technical evaluation is the right next step.

Regards,

{{se_name}}

### Template 2: POC Success Plan Request

Subject: POC success plan inputs needed before provisioning

Hi {{buyer_owner}},

Before we provision the Oyatie POC tenant, we need to lock the success plan so the evaluation remains bounded and measurable.

Please confirm:

1. Business sponsor.
2. Technical owner.
3. Start and end dates.
4. Use cases in scope.
5. Data classification and source systems.
6. Acceptance metrics.
7. Security or compliance artifacts required for signoff.

Once these are confirmed, we can create the sandbox tenant, publish POC-scoped policies, and schedule kickoff.

Regards,

{{se_name}}

### Template 3: POC Weekly Checkpoint Recap

Subject: Oyatie POC checkpoint recap - week {{week_number}}

Hi {{buyer_team}},

Here is the checkpoint summary for week {{week_number}}.

Completed:

- {{completed_item_1}}
- {{completed_item_2}}
- {{completed_item_3}}

Measured results:

- {{metric_1}}: {{value_1}}
- {{metric_2}}: {{value_2}}
- {{metric_3}}: {{value_3}}

Open risks:

- {{risk_1}} - owner: {{owner_1}} - target date: {{date_1}}
- {{risk_2}} - owner: {{owner_2}} - target date: {{date_2}}

Next checkpoint focus:

- {{next_focus}}

Regards,

{{se_name}}

### Template 4: Technical Evaluation Conditional Pass

Subject: Technical evaluation condition - {{criterion_name}}

Hi {{buyer_owner}},

We are marking {{criterion_name}} as a conditional pass.

Evidence reviewed:

- {{evidence_link_1}}
- {{evidence_link_2}}

Condition:

- {{condition_description}}

Owner:

- {{owner_name}}

Target resolution:

- {{target_date}}

We will not represent this criterion as fully passed until the condition is resolved or accepted as a contract/onboarding dependency.

Regards,

{{se_name}}

### Template 5: POC Closeout and Handoff

Subject: Oyatie POC closeout and recommended next step

Hi {{buyer_team}},

The POC for {{tenant_or_project_name}} is complete.

Outcome:

- {{passed_or_conditional_or_failed}}

Validated:

- {{validated_outcome_1}}
- {{validated_outcome_2}}
- {{validated_outcome_3}}

Remaining conditions:

- {{condition_1}}
- {{condition_2}}

Recommended next step:

- {{recommended_next_step}}

We will archive the evidence pack and complete the sandbox data-retention or deletion step according to the agreed plan.

Regards,

{{se_name}}

## Cross-References

- `docs/GTM-PLAN.md` for GTM thesis, packaging, launch, and customer-success motion.
- `docs/personas/MASTER-ROSTER-2026-05-21.md` for named buyer, seller, success, security, compliance, and operations personas.
- `specs/tenant-model.json` for tenant fields used during qualification and POC scoping.
- `specs/tenant-lifecycle.json` for tenant lifecycle states and onboarding saga references.
- `docs/standards/tenant-lifecycle.md` for lifecycle governance and state-transition language.
- `docs/standards/cedar-policy-authoring.md` for tenant-scope, auditor-scope, CI-scope, regulated-data, and destructive-action policy rules.
- `contracts/openapi/platform/platform-tenant-v1.yaml` for tenant creation contract and residency-class fields.
- `contracts/openapi/platform/platform-policy-cedar-v1.yaml` for policy publishing contract.
- `contracts/openapi/foundry/capability-v1.yaml` for tenant-granted capability invocation.
- `contracts/openapi/foundry/policy-v1.yaml` for Foundry policy autonomy ceiling publication.
- `docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md` for quote-to-contract-to-payment demo track.
- `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md` for incident-response and SLO-credit demo track.
- `docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md` for QBR and incident-debrief demo track.
- `docs/standards/migration-playbook.md` for migration discovery, export, transform, import, validate, and cutover references.
- `docs/gtm/tenant-prospect-to-active-stages.md` for funnel-stage exit gates that precede SE engagement.
