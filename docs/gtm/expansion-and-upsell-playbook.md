---
doc_class: GoToMarketPlaybook
title: Expansion and Upsell Playbook
status: Draft
date: 2026-05-20
owner: GoToMarket / Customer Success / Sales
related_oyatie_adrs:
  - docs/adr-archive/ADR-0242-oyatie-is-a-tenant-doctrine.md
  - docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
  - docs/adr-archive/ADR-0009-cell-architecture-per-tenant-per-region.md
  - docs/adr-archive/ADR-0010-regional-pack-architecture.md
related_personas:
  - Customer Success Manager Sofia Rezende
  - CS-IC Lin Chen
  - Sales AE Maya Lindqvist
  - Sales Manager Anthony Costa
  - Customer Champion Akemi Sato
  - CEO Aoki Tanaka
  - CFO Helena Brandt
  - COO Akira Watanabe
  - CTO Diego Vargas
  - CISO Yuki Park
  - Procurement Manager Wei Liu
  - Finance Director Mei-Ling Wu
  - Business Analyst Aditya Verma
  - Data Analyst Felipe Andrade
  - Compliance Officer Tunde Bello
related_journeys:
  - docs/user-journeys/j117-api-customer-tenant-incident-response/README.md
  - docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md
  - docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Expansion and Upsell Playbook

## Purpose

This playbook defines how Oyatie identifies, qualifies, governs, and executes tenant expansion after a tenant is Active.

Expansion is not a license to push more product.

Expansion is the controlled response to observed usage, operational maturity, risk posture, and business outcome evidence.

The motion begins when Customer Success observes a named expansion signal, a buyer requests a new capability, or a QBR produces an approved next-step path.

The motion ends when the tenant either remains on the current tier, upgrades tier, adds a capability, expands user or business-unit scope, or enters a recovery path before commercial expansion.

This playbook protects trust by separating value expansion from unresolved onboarding, support, security, or compliance problems.

This playbook protects package integrity by tying upsell conversations to capability-tier registry targets and Cedar permit activation.

This playbook protects Finance by connecting upgrade recommendations to value, usage, SLA exposure, retention, or risk reduction.

This playbook protects Security and Compliance by ensuring that new entitlements, users, data classes, and regulatory packs are approved before activation.

This playbook protects Sales by giving AE Maya Lindqvist evidence-backed conversation entry points.

This playbook protects Customer Success by preventing premature commercial motion when adoption is weak.

Expansion can be a tier upgrade, a capability upgrade, a seat expansion, a business-unit expansion, a compliance-pack expansion, a DR-tier upgrade, a retention upgrade, a support upgrade, or a migration wave expansion.

The account path from the Day 90 onboarding closeout or QBR determines whether expansion is allowed.

Accounts classified Recover do not receive upsell asks until the recovery plan is accepted.

Accounts classified Stabilize may receive education but not pressure.

Accounts classified Optimize may receive value-expansion workshops.

Accounts classified Expand may receive commercial upgrade proposals.

Every expansion recommendation must name the buyer, pain, signal, metric, package gap, policy action, and success target.

Every expansion activation must be reflected in tenant entitlements, Cedar permits, support model, audit evidence, and QBR baseline.

This document is a GTM playbook and not a pricing authority.

## Personas Involved (named — from MASTER-ROSTER)

- Customer Success Manager Sofia Rezende owns account health, expansion signal review, QBR framing, and readiness recommendation.
- CS-IC Lin Chen owns usage evidence, entitlement-change readiness, adoption analysis, and post-upgrade instrumentation.
- Sales AE Maya Lindqvist owns commercial conversation, proposal, negotiation, and contract amendment path.
- Sales Manager Anthony Costa owns deal governance, discount discipline, and escalation when expansion risk conflicts with revenue pressure.
- Customer Champion Akemi Sato provides internal success narrative and peer reference language when approved.
- CEO Aoki Tanaka evaluates strategic expansion, executive value, and board-visible risk.
- CFO Helena Brandt evaluates budget, cost, ROI, service credits, and upgrade economics.
- COO Akira Watanabe evaluates operational scale, workflow throughput, incident posture, and quarterly operating goals.
- CTO Diego Vargas evaluates technical scaling, integration depth, reliability, and architecture fit.
- CISO Yuki Park evaluates new policy surface, identity expansion, regulated data, and audit evidence.
- Procurement Manager Wei Liu evaluates amendment process, vendor risk changes, and package terms.
- Finance Director Mei-Ling Wu evaluates invoice routing, cost-center changes, and usage-cost attribution.
- Business Analyst Aditya Verma evaluates workflow maturity, process value, and team-level adoption.
- Data Analyst Felipe Andrade evaluates reporting depth, metric lineage, and export expansion.
- Compliance Officer Tunde Bello evaluates regulatory-pack expansion, jurisdiction, and evidence-retention implications.

## Stages / Steps (named, sequenced)

### Expansion Stage 1: Signal Discovery

- Stage 1 Exit Gate X1-G1: At least one named usage, risk, value, or buyer-request signal is documented with supporting evidence.
- Step 1.01: CSM reviews account path from Day 90 or latest QBR.
- Step 1.02: CSM blocks upsell if account path is Recover and recovery plan is not accepted.
- Step 1.03: CSM reviews active user growth.
- Step 1.04: CSM reviews workflow completion growth.
- Step 1.05: CSM reviews API or request-rate growth.
- Step 1.06: CSM reviews policy-deny trend.
- Step 1.07: CSM reviews support ticket volume.
- Step 1.08: CSM reviews incident count.
- Step 1.09: CSM reviews SLA exposure.
- Step 1.10: CSM reviews audit evidence requests.
- Step 1.11: CSM reviews compliance-pack questions.
- Step 1.12: CSM reviews integration requests.
- Step 1.13: CSM reviews migration wave requests.
- Step 1.14: CSM reviews cost-center expansion.
- Step 1.15: CSM reviews executive sponsor goals.
- Step 1.16: CSM reviews buyer-initiated feature requests.
- Step 1.17: CSM records one primary expansion hypothesis.
- Step 1.18: CSM records one disqualifying risk.
- Step 1.19: CSM records proposed commercial owner.
- Step 1.20: CSM records proposed technical owner.

### Expansion Stage 2: Fit and Readiness

- Stage 2 Exit Gate X2-G1: Expansion candidate is classified by path, tier, capability, risk, owner, and target metric.
- Step 2.01: CSM verifies tenant is Active.
- Step 2.02: CSM verifies tenant is not locked.
- Step 2.03: CSM verifies unresolved P1 incidents are closed or accepted.
- Step 2.04: CSM verifies onboarding evidence is archived.
- Step 2.05: CSM verifies current package tier.
- Step 2.06: CSM verifies current support tier.
- Step 2.07: CSM verifies current DR tier.
- Step 2.08: CSM verifies current compliance packs.
- Step 2.09: CSM verifies current active seat count.
- Step 2.10: CSM verifies current request-rate pattern.
- Step 2.11: CSM verifies current audit retention usage.
- Step 2.12: CSM verifies current RPO or RTO exposure.
- Step 2.13: CSM verifies current evidence retrieval pattern.
- Step 2.14: CSM verifies buyer owner for expansion.
- Step 2.15: CSM verifies technical owner for expansion.
- Step 2.16: CSM verifies security owner when policy surface changes.
- Step 2.17: CSM verifies compliance owner when regulatory pack changes.
- Step 2.18: CSM verifies finance owner when cost increases.
- Step 2.19: CSM classifies expansion type.
- Step 2.20: CSM creates expansion readiness summary.

### Expansion Stage 3: Value Narrative

- Stage 3 Exit Gate X3-G1: Conversation narrative is approved by CSM and AE with evidence, buyer pain, and package recommendation.
- Step 3.01: CSM names observed signal.
- Step 3.02: CSM names business consequence of the signal.
- Step 3.03: CSM names current package constraint.
- Step 3.04: CSM names recommended upgrade or add-on.
- Step 3.05: CSM names expected outcome.
- Step 3.06: CSM names success metric.
- Step 3.07: CSM names activation dependency.
- Step 3.08: CSM names risk if no action.
- Step 3.09: CSM names buyer persona for conversation.
- Step 3.10: CSM names executive sponsor if needed.
- Step 3.11: AE reviews commercial timing.
- Step 3.12: AE reviews procurement path.
- Step 3.13: AE reviews amendment requirements.
- Step 3.14: AE reviews pricing or packaging assumptions.
- Step 3.15: Sales Manager reviews large or discounted expansion.
- Step 3.16: CSM prepares QBR insert.
- Step 3.17: CS-IC prepares usage evidence appendix.
- Step 3.18: AE prepares commercial next-step ask.
- Step 3.19: CSM prepares technical caveats.
- Step 3.20: CSM and AE align before buyer conversation.

### Expansion Stage 4: Buyer Conversation

- Stage 4 Exit Gate X4-G1: Buyer accepts next evaluation step, rejects expansion, or requests revised recommendation.
- Step 4.01: CSM opens with observed customer outcome.
- Step 4.02: CSM states the signal without exaggeration.
- Step 4.03: CSM states current constraint.
- Step 4.04: CSM states recommendation.
- Step 4.05: CSM states what would change operationally.
- Step 4.06: CSM states what would change technically.
- Step 4.07: CSM states what would change commercially.
- Step 4.08: CSM states what would not change.
- Step 4.09: AE states amendment path.
- Step 4.10: AE states decision owner needed.
- Step 4.11: Buyer validates or rejects the pain.
- Step 4.12: Buyer validates or rejects the target outcome.
- Step 4.13: Buyer validates or rejects the timing.
- Step 4.14: Buyer names security concerns.
- Step 4.15: Buyer names compliance concerns.
- Step 4.16: Buyer names finance concerns.
- Step 4.17: CSM records objections.
- Step 4.18: AE records commercial actions.
- Step 4.19: CS-IC records technical follow-up.
- Step 4.20: CSM sends recap within one business day.

### Expansion Stage 5: Technical and Policy Readiness

- Stage 5 Exit Gate X5-G1: New entitlement can be safely activated under tenant model, policy, support, and evidence controls.
- Step 5.01: CS-IC validates entitlement target.
- Step 5.02: CS-IC validates tenant eligibility.
- Step 5.03: CS-IC validates trust-score requirement.
- Step 5.04: CS-IC validates KYC requirement.
- Step 5.05: CS-IC validates capability-tier dependency.
- Step 5.06: CS-IC validates support-tier dependency.
- Step 5.07: CS-IC validates DR-tier dependency.
- Step 5.08: CS-IC validates compliance-pack dependency.
- Step 5.09: CS-IC validates residency impact.
- Step 5.10: CS-IC validates audit stream impact.
- Step 5.11: CISO Yuki Park reviews policy delta.
- Step 5.12: Compliance Officer Tunde Bello reviews regulated-data delta.
- Step 5.13: Finance Director Mei-Ling Wu reviews cost-center delta.
- Step 5.14: CTO Diego Vargas reviews integration delta.
- Step 5.15: COO Akira Watanabe reviews operational delta.
- Step 5.16: CS-IC drafts Cedar permit changes.
- Step 5.17: CS-IC drafts enablement plan.
- Step 5.18: CS-IC drafts rollback plan.
- Step 5.19: CSM updates risk register.
- Step 5.20: CSM signs readiness or marks blocked.

### Expansion Stage 6: Commercial Commitment

- Stage 6 Exit Gate X6-G1: Commercial amendment, order form, or documented no-decision is complete.
- Step 6.01: AE confirms package recommendation.
- Step 6.02: AE confirms SKU or tier path.
- Step 6.03: AE confirms pricing assumptions.
- Step 6.04: AE confirms start date.
- Step 6.05: AE confirms billing impact.
- Step 6.06: AE confirms support impact.
- Step 6.07: AE confirms SLA impact.
- Step 6.08: AE confirms procurement owner.
- Step 6.09: AE confirms legal review path.
- Step 6.10: AE confirms security addendum path if needed.
- Step 6.11: AE confirms data-processing impact if needed.
- Step 6.12: AE confirms amendment language.
- Step 6.13: AE confirms buyer signature authority.
- Step 6.14: Sales Manager approves discount if applicable.
- Step 6.15: Procurement Manager Wei Liu confirms process.
- Step 6.16: Finance Director Mei-Ling Wu confirms invoice routing.
- Step 6.17: CFO Helena Brandt confirms budget approval if needed.
- Step 6.18: CSM confirms activation date.
- Step 6.19: CS-IC confirms activation runbook.
- Step 6.20: AE closes commercial commitment.

### Expansion Stage 7: Activation and Adoption

- Stage 7 Exit Gate X7-G1: New capability or tier is activated, adopted, instrumented, and included in next QBR baseline.
- Step 7.01: CS-IC verifies signed amendment or entitlement approval.
- Step 7.02: CS-IC updates capability entitlement.
- Step 7.03: CS-IC publishes Cedar policy changes.
- Step 7.04: CS-IC verifies allow tests.
- Step 7.05: CS-IC verifies deny tests.
- Step 7.06: CS-IC updates audit stream.
- Step 7.07: CS-IC updates observability dashboard.
- Step 7.08: CS-IC updates support routing.
- Step 7.09: CS-IC updates cost-center dashboard.
- Step 7.10: CS-IC updates adoption instrumentation.
- Step 7.11: CSM sends activation notice.
- Step 7.12: CSM runs enablement session.
- Step 7.13: CSM records first expanded use.
- Step 7.14: CSM records first expanded value metric.
- Step 7.15: CSM records user feedback.
- Step 7.16: CSM records open risks.
- Step 7.17: AE records expansion close details.
- Step 7.18: CSM updates health score.
- Step 7.19: CSM updates next QBR agenda.
- Step 7.20: CSM closes expansion activation.

### Capability-Tier Upgrade Triggers


### Named Usage-Pattern Signals

- Usage Signal UPS-001: `seat_cap_pressure` means active seats exceed 80 percent of purchased tier cap.
- Usage Signal UPS-002: `request_rate_pressure` means sustained RPS exceeds 70 percent of tier target.
- Usage Signal UPS-003: `audit_retention_pressure` means evidence retrieval requests exceed current hot-retention window.
- Usage Signal UPS-004: `p1_response_pressure` means customer escalation expectation is faster than current package target.
- Usage Signal UPS-005: `rto_gap_pressure` means customer continuity goal is tighter than current RTO.
- Usage Signal UPS-006: `rpo_gap_pressure` means customer recovery-point expectation is tighter than current RPO.
- Usage Signal UPS-007: `multi_bu_pull` means additional business units request access.
- Usage Signal UPS-008: `subsidiary_rollout_pull` means parent tenant wants child tenant expansion.
- Usage Signal UPS-009: `workflow_density_growth` means completed workflows grow for three consecutive periods.
- Usage Signal UPS-010: `admin_workload_growth` means customer admin actions grow faster than active-user growth.
- Usage Signal UPS-011: `integration_depth_growth` means additional source systems are requested.
- Usage Signal UPS-012: `migration_wave_growth` means second or third migration wave is requested.
- Usage Signal UPS-013: `security_review_depth` means CISO asks for additional policy or evidence surfaces.
- Usage Signal UPS-014: `compliance_pack_pull` means compliance owner asks about an unpurchased pack.
- Usage Signal UPS-015: `incident_review_maturity` means customer asks for formal debrief and corrective-action tracking.
- Usage Signal UPS-016: `qbr_executive_dependency` means executive team relies on Oyatie metrics for operating review.
- Usage Signal UPS-017: `cost_center_split` means Finance requests departmental or regional allocation.
- Usage Signal UPS-018: `data_export_growth` means analyst export usage grows and needs lineage controls.
- Usage Signal UPS-019: `policy_deny_pattern` means denied actions are legitimate work blocked by current entitlement.
- Usage Signal UPS-020: `champion_reference_ready` means champion can describe measurable success internally.
- Usage Signal UPS-021: `support_case_repeat` means repeated tickets signal need for higher support tier or enablement.
- Usage Signal UPS-022: `regulatory_exam_signal` means customer prepares for external audit or regulator review.
- Usage Signal UPS-023: `business_criticality_shift` means workflow moved from nice-to-have to critical operation.
- Usage Signal UPS-024: `manual_workaround_decay` means team has retired prior manual process.
- Usage Signal UPS-025: `time_to_value_repeatability` means initial value pattern can repeat in another department.
- Usage Signal UPS-026: `executive_goal_alignment` means expansion maps to named quarterly executive goal.
- Usage Signal UPS-027: `budget_window_open` means customer has budget cycle suitable for expansion.
- Usage Signal UPS-028: `procurement_preapproved_path` means Procurement has no new vendor-risk blocker.
- Usage Signal UPS-029: `contractual_entitlement_request` means customer requests amendment for unpurchased right.
- Usage Signal UPS-030: `data_class_expansion` means new regulated or sensitive data class is requested.

### Conversation Playbooks

- Conversation Playbook CP-01: Seat-Cap Expansion with Finance.
- CP-01 Opening: "Your active-user trend is approaching the current package boundary; let's review whether the next tier prevents avoidable rollout throttling."
- CP-01 Evidence: active seats, invited seats, department growth, admin workload, support trend.
- CP-01 Buyer: CFO Helena Brandt with IT Manager Jamie O'Connor.
- CP-01 Recommendation: upgrade package tier or contract additional seat capacity.
- CP-01 Policy Note: activate `cedar.tenant.expansion.seat_entitlement_admin` only after amendment.
- CP-01 Success Metric: active-seat growth without support-ticket spike.
- CP-01 Risk if deferred: rollout stalls, shadow accounts emerge, admin workload grows.
- Conversation Playbook CP-02: Reliability Upgrade with Operations.
- CP-02 Opening: "The workflow is now operationally important enough that the current RTO and support targets deserve review."
- CP-02 Evidence: incident trend, RTO expectation, workflow criticality, error-budget burn.
- CP-02 Buyer: COO Akira Watanabe with CTO Diego Vargas.
- CP-02 Policy Note: activate `cedar.tenant.slo_budget.executive_viewer` and support escalation changes.
- CP-02 Success Metric: tabletop P1 acknowledgement and recovery target alignment.
- CP-02 Risk if deferred: business-critical workflow relies on package below operational expectation.
- Conversation Playbook CP-03: Compliance-Pack Expansion.
- CP-03 Opening: "Your team is asking for evidence outside the current pack; we should decide whether that is a formal compliance expansion."
- CP-03 Evidence: audit requests, data class, jurisdiction, regulator or auditor deadline.
- CP-03 Buyer: Compliance Officer Tunde Bello with CISO Yuki Park.
- CP-03 Recommendation: add compliance pack or move to package with required retention and evidence posture.
- CP-03 Policy Note: activate regulated-data and auditor-scope permits after pack approval.
- CP-03 Success Metric: evidence retrieval within target and no unauthorized regulated-data access.
- CP-03 Risk if deferred: compliance expectations outpace contractual and policy coverage.
- Conversation Playbook CP-04: Business-Unit Rollout.
- CP-04 Opening: "The first team has a repeatable value pattern; let's test whether the next business unit has the same operating conditions."
- CP-04 Evidence: first value metric, champion quote, workflow repeatability, support load.
- CP-04 Buyer: COO Akira Watanabe with Business Analyst Aditya Verma.
- CP-04 Recommendation: expand cohort, add child tenant, or run new business-unit onboarding wave.
- CP-04 Policy Note: activate business-unit scoped principals and workflow access only after owner approval.
- CP-04 Success Metric: second-unit time-to-value and adoption rate.
- CP-04 Risk if deferred: local success remains isolated and executive value case weakens.
- Conversation Playbook CP-05: Data and Analytics Expansion.
- CP-05 Opening: "Your reporting use has matured; the next question is whether analytics access should become governed and repeatable."
- CP-05 Evidence: export frequency, dashboard usage, metric lineage gaps, analyst requests.
- CP-05 Buyer: Data Analyst Felipe Andrade with CFO Helena Brandt.
- CP-05 Recommendation: enable analytics capability, lineage view, or higher retention package.
- CP-05 Policy Note: activate `cedar.tenant.data_analyst.lineage_reader` and export controls.
- CP-05 Success Metric: trusted recurring metric pack with lineage evidence.
- CP-05 Risk if deferred: spreadsheet drift and metric disputes return.

## Named Tools and Cedar Permits (specific µservices + actions + Cedar policies)

### Expansion Tools

- Tool: `foundry-capability-service`; Action: `foundry-capability:invokeCapability`; Expansion use: verify tenant-granted capability is ready for new use.
- Tool: `platform-policy-cedar-service`; Action: `platform-policy:publishCedarPolicy`; Expansion use: publish new entitlement, viewer, admin, or regulated-data policies.
- Tool: `platform-tenant-service`; Action: `platform-tenant:GetTenant`; Expansion use: inspect tier, lifecycle, home cell, jurisdiction, and lock state.
- Tool: `identity-service`; Action: `identity:BindTenantPrincipal`; Expansion use: add new cohort, admin, auditor, or business-unit principals.
- Tool: `workflow-engine`; Action: `workflow-engine:ConfigureWorkflow`; Expansion use: configure expanded workflow.
- Tool: `audit-chain-service`; Action: `audit-chain:ReadEvidence`; Expansion use: retrieve evidence for readiness and QBR.
- Tool: `observability-service`; Action: `observability:ReadServiceHealth`; Expansion use: validate usage, latency, and incident trend.
- Tool: `slo-budgets-service`; Action: `slo-budgets:ReadBudget`; Expansion use: support reliability upgrade conversation.
- Tool: `finops-portal`; Action: `finops-portal:ReadTenantCostCenter`; Expansion use: support cost-center and ROI review.
- Tool: `ops-dashboard-control-center`; Action: `ops-dashboard-control-center:ViewQuarterlyReview`; Expansion use: support executive operating review.
- Tool: `migration-discovery-service`; Action: `oyatie.migration.discover`; Expansion use: scope next migration wave.
- Tool: `migration-cutover-service`; Action: `oyatie.migration.cutover`; Expansion use: activate approved next wave.

### Expansion Cedar Permits

- Cedar Permit: `cedar.tenant.expansion.signal_reader`; Allows CSM and AE to read expansion signals without changing entitlements.
- Cedar Permit: `cedar.tenant.expansion.readiness_viewer`; Allows account team to view readiness evidence.
- Cedar Permit: `cedar.tenant.expansion.seat_entitlement_admin`; Allows seat entitlement change only after commercial approval.
- Cedar Permit: `cedar.tenant.expansion.capability_requester`; Allows customer owner to request capability review.
- Cedar Permit: `cedar.tenant.expansion.capability_activator`; Allows capability activation when eligibility, trust, KYC, and contract pass.
- Cedar Permit: `cedar.tenant.expansion.tier_grant_admin`; Allows tier upgrade grant after approved amendment.
- Cedar Permit: `cedar.tenant.expansion.business_unit_admin`; Allows business-unit onboarding setup for approved unit.
- Cedar Permit: `cedar.tenant.expansion.child_tenant_creator`; Allows child tenant request after parent approval.
- Cedar Permit: `cedar.tenant.expansion.compliance_pack_admin`; Allows compliance-pack activation after compliance approval.
- Cedar Permit: `cedar.tenant.expansion.regulated_data_guard`; Denies regulated-data access until policy and pack attributes match.
- Cedar Permit: `cedar.tenant.expansion.support_tier_admin`; Allows support tier update after amendment.
- Cedar Permit: `cedar.tenant.expansion.dr_tier_admin`; Allows DR tier update after technical readiness.
- Cedar Permit: `cedar.tenant.expansion.audit_retention_admin`; Allows retention target update after package approval.
- Cedar Permit: `cedar.tenant.expansion.finops_viewer`; Allows finance review of expansion cost and usage.
- Cedar Permit: `cedar.tenant.expansion.procurement_viewer`; Allows Procurement to review amendment evidence.
- Cedar Permit: `cedar.tenant.expansion.qbr_insert_editor`; Allows CSM to add expansion section to QBR.
- Cedar Permit: `cedar.tenant.expansion.rollback_operator`; Allows rollback of newly activated capability within approved window.
- Cedar Permit: `cedar.tenant.expansion.no_premature_activation`; Denies activation when commercial approval or readiness evidence is missing.

## Specific Metrics + Named SLA Targets

- Metric: Expansion Signal Precision; Target: 90 percent of proposed expansions cite a named usage-pattern signal.
- Metric: Expansion Readiness Completeness; Target: 100 percent of expansion proposals include tier, capability, owner, risk, and success metric.
- Metric: Recover Account Suppression; Target: zero upsell asks to accounts in Recover without accepted recovery plan.
- Metric: QBR Expansion Evidence Rate; Target: 100 percent of QBR expansion recommendations include usage and value evidence.
- Metric: Tier Trigger Accuracy; Target: 100 percent of tier-upgrade proposals map to capability-tier trigger.
- Metric: Commercial Handoff Timeliness; Target: AE receives expansion brief within two business days after readiness approval.
- Metric: Buyer Recap Timeliness; Target: expansion conversation recap sent within one business day.
- Metric: Policy Activation Timeliness; Target: approved expansion policies published by activation date.
- Metric: Allow-Deny Test Completion; Target: 100 percent of new permits have allow and deny evidence.
- Metric: Expansion Adoption Baseline; Target: baseline metric captured before activation.
- Metric: Expansion First-Use Completion; Target: first expanded use captured within fourteen days after activation.
- Metric: Expansion QBR Inclusion; Target: next QBR includes post-expansion metric and risk status.
- Metric: Seat Expansion Health; Target: active-seat growth does not increase support tickets by more than 20 percent without intervention.
- Metric: Business-Unit Expansion Health; Target: second unit reaches 60 percent target workflow completion within 45 days.
- Metric: Capability Expansion Health; Target: new capability has named owner and first-value metric within 30 days.
- Metric: Compliance Expansion Health; Target: evidence retrieval passes before regulated workflow activation.
- Metric: Reliability Upgrade Health; Target: tabletop confirms P1 response and RTO/RPO targets before relying on upgraded tier.

## Named Failure Modes + Recovery

- Failure Mode: `EXPANSION-PREMATURE-ASK`; Signal: upsell initiated while launch blockers or recovery plan remain open; Recovery: stop commercial ask and run stabilization plan.
- Failure Mode: `EXPANSION-NO-SIGNAL`; Signal: recommendation has no usage, risk, value, or buyer-request evidence; Recovery: return to QBR discovery and do not propose.
- Failure Mode: `EXPANSION-WRONG-BUYER`; Signal: finance expansion pitched to technical owner without budget authority; Recovery: remap buyer persona and reframe.
- Failure Mode: `EXPANSION-TIER-BLUR`; Signal: customer is told upgrade benefit without clear tier boundary; Recovery: send capability-tier explanation and corrected recommendation.
- Failure Mode: `EXPANSION-POLICY-GAP`; Signal: entitlement activated without matching Cedar permit; Recovery: suspend new access, publish policy, and verify allow-deny tests.
- Failure Mode: `EXPANSION-SECURITY-BYPASS`; Signal: new users or data classes added before CISO approval; Recovery: revoke access and reopen readiness review.
- Failure Mode: `EXPANSION-COMPLIANCE-BYPASS`; Signal: compliance pack assumed but not activated; Recovery: block regulated workflow and route to compliance-pack approval.
- Failure Mode: `EXPANSION-FINOPS-SURPRISE`; Signal: invoice impact not reviewed with Finance; Recovery: notify Finance Director and AE, adjust activation date if needed.
- Failure Mode: `EXPANSION-SUPPORT-SPIKE`; Signal: support tickets rise after activation; Recovery: run enablement session and classify account Optimize.
- Failure Mode: `EXPANSION-NO-FIRST-VALUE`; Signal: new capability has no first-use metric within target window; Recovery: run adoption rescue or reverse entitlement if unused.
- Failure Mode: `EXPANSION-OVER-CUSTOMIZATION`; Signal: expansion requires bespoke unsupported workflow; Recovery: classify product gap and avoid commercial commitment.
- Failure Mode: `EXPANSION-UNOWNED-ROLLOUT`; Signal: business-unit expansion lacks local owner; Recovery: pause rollout until owner is named.
- Failure Mode: `EXPANSION-AUDIT-DRIFT`; Signal: evidence retention expectation exceeds package; Recovery: present tier path or contract exception.
- Failure Mode: `EXPANSION-DR-MISMATCH`; Signal: customer expects RTO or RPO beyond current DR tier; Recovery: run reliability upgrade playbook.
- Failure Mode: `EXPANSION-RENEWAL-CONFLICT`; Signal: expansion ask conflicts with renewal risk or open complaint; Recovery: prioritize renewal stabilization.

## Sample Dialogue / Email Templates

### Template 1: CSM Expansion Signal Note to AE

Subject: Expansion signal for {{tenant_name}} - {{signal_name}}

Hi {{ae_name}},

We have a qualified expansion signal for {{tenant_name}}.

Signal:

- {{signal_name}}: {{signal_value}}

Customer outcome:

- {{outcome}}

Likely path:

- {{tier_or_capability_path}}

Risks:

- {{risk_1}}
- {{risk_2}}

Recommended next step:

- Align on buyer conversation before the next QBR.

Regards,

{{csm_name}}

### Template 2: Buyer Conversation Follow-Up

Subject: Follow-up on {{tenant_name}} expansion discussion

Hi {{buyer_name}},

Thank you for reviewing the expansion signal with us.

We discussed:

- Current usage or risk signal: {{signal}}
- Current package constraint: {{constraint}}
- Recommended path: {{recommendation}}
- Expected outcome: {{expected_outcome}}

Open questions:

- {{question_1}}
- {{question_2}}

Next step:

- {{next_step}} by {{date}}

Regards,

{{csm_or_ae_name}}

### Template 3: Security Readiness Request

Subject: Security review for proposed Oyatie expansion

Hi {{security_owner}},

The account team is preparing a proposed expansion for {{tenant_name}}.

The expansion changes:

- Capability or tier: {{change}}
- New principals: {{principals}}
- New data classes: {{data_classes}}
- New policy permits: {{permits}}

Please review whether the proposed policy set is acceptable before activation.

We will not activate the expanded entitlement until the readiness gate is approved.

Regards,

{{csic_name}}

### Template 4: Commercial Amendment Handoff

Subject: Commercial amendment readiness for {{tenant_name}}

Hi {{ae_name}},

The expansion readiness review is complete.

Approved for commercial path:

- {{yes_or_no}}

Recommended package or capability:

- {{recommendation}}

Activation dependencies:

- {{dependency_1}}
- {{dependency_2}}

Success metric:

- {{metric}}

Please use the attached evidence pack for the buyer conversation and amendment path.

Regards,

{{csm_name}}

### Template 5: Expansion Activation Notice

Subject: {{tenant_name}} expansion activated - {{capability_or_tier}}

Hi {{customer_team}},

The approved expansion for {{tenant_name}} is now active.

Activated:

- {{capability_or_tier}}

Policy controls:

- {{policy_1}}
- {{policy_2}}

First success metric:

- {{metric_name}} by {{target_date}}

We will review first use and risk status in the next checkpoint.

Regards,

{{csm_name}}

## Expansion Review Register

### Signal Review Controls

- Signal Review Control SRC-001: Every expansion candidate names exactly one primary signal.
- Signal Review Control SRC-002: Secondary signals are recorded separately from the primary signal.
- Signal Review Control SRC-003: Usage signal must include source metric and observation window.
- Signal Review Control SRC-004: Buyer-request signal must include requesting persona and business reason.
- Signal Review Control SRC-005: Compliance signal must include data class, pack, jurisdiction, or audit request.
- Signal Review Control SRC-006: Reliability signal must include incident, support, SLO, RPO, or RTO evidence.
- Signal Review Control SRC-007: Finance signal must include cost center, budget window, or ROI evidence.
- Signal Review Control SRC-008: Migration signal must include source system and migration wave.
- Signal Review Control SRC-009: Seat signal must include active seats, invited seats, and tier cap.
- Signal Review Control SRC-010: Request-rate signal must include sustained peak and package target.
- Signal Review Control SRC-011: Audit-retention signal must include requested retention window.
- Signal Review Control SRC-012: Data-export signal must include export use case and lineage requirement.
- Signal Review Control SRC-013: Business-unit signal must include local owner and target group.
- Signal Review Control SRC-014: Subsidiary signal must include parent tenant owner and proposed child tenant.
- Signal Review Control SRC-015: Champion signal must include approved quote or measurable success story.
- Signal Review Control SRC-016: Weak signals remain education-only until evidence matures.
- Signal Review Control SRC-017: Conflicting signals are reviewed by CSM and AE before customer conversation.
- Signal Review Control SRC-018: Recover accounts require recovery review before any expansion signal moves forward.
- Signal Review Control SRC-019: Expansion signal history is attached to QBR evidence pack.
- Signal Review Control SRC-020: Signal review outcome is Advance, Monitor, Defer, or Recover.

### Tier Review Controls

- Tier Review Control TRC-004: Tier recommendation must include current availability target.
- Tier Review Control TRC-005: Tier recommendation must include proposed availability target.
- Tier Review Control TRC-006: Tier recommendation must include current P1 response target.
- Tier Review Control TRC-007: Tier recommendation must include proposed P1 response target.
- Tier Review Control TRC-008: Tier recommendation must include current RPO target.
- Tier Review Control TRC-009: Tier recommendation must include proposed RPO target.
- Tier Review Control TRC-010: Tier recommendation must include current RTO target.
- Tier Review Control TRC-011: Tier recommendation must include proposed RTO target.
- Tier Review Control TRC-012: Tier recommendation must include current audit hot retention.
- Tier Review Control TRC-013: Tier recommendation must include proposed audit hot retention.
- Tier Review Control TRC-014: Tier recommendation must include current request capacity.
- Tier Review Control TRC-015: Tier recommendation must include proposed request capacity.
- Tier Review Control TRC-016: Tier recommendation must include current seat cap.
- Tier Review Control TRC-017: Tier recommendation must include proposed seat cap.
- Tier Review Control TRC-018: Tier recommendation must state whether support model changes.
- Tier Review Control TRC-019: Tier recommendation must state whether DR posture changes.
- Tier Review Control TRC-020: Tier recommendation must state whether audit evidence posture changes.

### Capability Review Controls

- Capability Review Control CRC-001: Capability request must name current tenant state.
- Capability Review Control CRC-002: Capability request must name capability id.
- Capability Review Control CRC-003: Capability request must name requesting persona.
- Capability Review Control CRC-004: Capability request must name business outcome.
- Capability Review Control CRC-005: Capability request must name data classes touched.
- Capability Review Control CRC-006: Capability request must name user cohort.
- Capability Review Control CRC-007: Capability request must name admin owner.
- Capability Review Control CRC-008: Capability request must name support implication.
- Capability Review Control CRC-009: Capability request must name evidence requirement.
- Capability Review Control CRC-010: Capability request must name policy delta.
- Capability Review Control CRC-011: Capability request must name enablement need.
- Capability Review Control CRC-012: Capability request must name first-use target.
- Capability Review Control CRC-013: Capability request must name rollback path.
- Capability Review Control CRC-014: Capability request must pass eligibility gate.
- Capability Review Control CRC-015: Capability request must pass trust-score gate.
- Capability Review Control CRC-016: Capability request must pass KYC gate when relevant.
- Capability Review Control CRC-017: Capability request must pass contract gate.
- Capability Review Control CRC-018: Capability request must pass security gate when data class changes.
- Capability Review Control CRC-019: Capability request must pass compliance gate when pack changes.
- Capability Review Control CRC-020: Capability request must pass customer-owner approval before activation.

### Finance and Procurement Controls

- Finance Control FPC-001: Expansion proposal includes current monthly or annual package cost.
- Finance Control FPC-002: Expansion proposal includes proposed cost impact.
- Finance Control FPC-003: Expansion proposal includes expected value metric.
- Finance Control FPC-004: Expansion proposal includes invoice routing impact.
- Finance Control FPC-005: Expansion proposal includes cost-center impact.
- Finance Control FPC-006: Expansion proposal includes budget timing.
- Finance Control FPC-007: Expansion proposal includes amendment requirement.
- Finance Control FPC-008: Expansion proposal includes procurement owner.
- Finance Control FPC-009: Expansion proposal includes legal review dependency.
- Finance Control FPC-010: Expansion proposal includes discount approval if discount exists.
- Finance Control FPC-011: Finance owner reviews cost before activation.
- Finance Control FPC-012: Procurement owner reviews amendment before signature.
- Finance Control FPC-013: AE records commercial approval before entitlement change.
- Finance Control FPC-014: CSM records value baseline before activation.
- Finance Control FPC-015: CSM records expected payback or risk-reduction rationale when used.
- Finance Control FPC-016: Finance surprise triggers retroactive buyer apology and billing correction plan.
- Finance Control FPC-017: Procurement delay triggers revised activation date.
- Finance Control FPC-018: Unapproved discount triggers Sales Manager review.
- Finance Control FPC-019: Contract ambiguity triggers legal clarification before activation.
- Finance Control FPC-020: Budget risk is included in QBR open-risk section.

### Security and Compliance Controls

- Security Control SCC-001: New principals are reviewed before access binding.
- Security Control SCC-002: New admin permissions are reviewed before activation.
- Security Control SCC-003: New auditor permissions are reviewed before activation.
- Security Control SCC-004: New support permissions are reviewed before activation.
- Security Control SCC-005: New regulated data classes are reviewed before activation.
- Security Control SCC-006: New residency impact is reviewed before activation.
- Security Control SCC-007: New audit retention target is reviewed before activation.
- Security Control SCC-008: New compliance pack is reviewed before activation.
- Security Control SCC-009: New destructive action path is guarded before activation.
- Security Control SCC-010: New migration wave is reviewed before export.
- Security Control SCC-011: CISO approval is required for elevated security surface.
- Security Control SCC-012: Compliance approval is required for pack expansion.
- Security Control SCC-013: Auditor access remains read-only.
- Security Control SCC-014: Support access remains diagnostic and time-bounded.
- Security Control SCC-015: No wildcard actions are allowed in expansion policies.
- Security Control SCC-016: No wildcard resources are allowed in expansion policies.
- Security Control SCC-017: No superuser bypass is allowed in expansion policies.
- Security Control SCC-018: Allow tests are recorded before launch.
- Security Control SCC-019: Deny tests are recorded before launch.
- Security Control SCC-020: Exceptions are attached to expansion evidence pack.

### Activation Controls

- Activation Control AC-001: Signed amendment exists before commercial entitlement change.
- Activation Control AC-002: Readiness summary exists before activation.
- Activation Control AC-003: Policy delta exists before activation.
- Activation Control AC-004: Rollback plan exists before activation.
- Activation Control AC-005: Enablement plan exists before activation.
- Activation Control AC-006: Support routing update exists before activation.
- Activation Control AC-007: Observability dashboard update exists before activation.
- Activation Control AC-008: FinOps dashboard update exists before activation.
- Activation Control AC-009: Audit evidence update exists before activation.
- Activation Control AC-010: Customer admin notification exists before activation.
- Activation Control AC-011: First-use owner exists before activation.
- Activation Control AC-012: First-use metric exists before activation.
- Activation Control AC-013: Activation date is communicated before activation.
- Activation Control AC-014: Activation evidence is attached after activation.
- Activation Control AC-015: Customer notice is sent after activation.
- Activation Control AC-016: AE close record is updated after activation.
- Activation Control AC-017: CSM health score is updated after activation.
- Activation Control AC-018: QBR baseline is updated after activation.
- Activation Control AC-019: First-use checkpoint is scheduled after activation.
- Activation Control AC-020: Expansion is closed only after adoption evidence exists or risk is documented.

### Post-Activation Controls

- Post-Activation Control PAC-001: First-use is reviewed within fourteen days.
- Post-Activation Control PAC-002: Adoption trend is reviewed within thirty days.
- Post-Activation Control PAC-003: Support trend is reviewed within thirty days.
- Post-Activation Control PAC-004: Policy-deny trend is reviewed within thirty days.
- Post-Activation Control PAC-005: Cost-center trend is reviewed within thirty days.
- Post-Activation Control PAC-006: Incident trend is reviewed within thirty days.
- Post-Activation Control PAC-007: Buyer satisfaction is reviewed within thirty days.
- Post-Activation Control PAC-008: Champion feedback is recorded within thirty days.
- Post-Activation Control PAC-009: Security exceptions are reviewed within thirty days.
- Post-Activation Control PAC-010: Compliance exceptions are reviewed within thirty days.
- Post-Activation Control PAC-011: Missing first value triggers Optimize path.
- Post-Activation Control PAC-012: Support spike triggers enablement path.
- Post-Activation Control PAC-013: Policy failure triggers immediate access review.
- Post-Activation Control PAC-014: Budget complaint triggers AE and Finance review.
- Post-Activation Control PAC-015: Unused entitlement triggers adoption rescue.
- Post-Activation Control PAC-016: Successful expansion feeds next QBR story.
- Post-Activation Control PAC-017: Failed expansion feeds recovery plan.
- Post-Activation Control PAC-018: Expansion lessons update playbook notes.
- Post-Activation Control PAC-019: Customer quote is requested only after value is confirmed.
- Post-Activation Control PAC-020: Next expansion is blocked until current expansion is stable.

### Executive Review Controls

- Executive Review Control ERC-001: Executive expansion review names the business outcome before naming package.
- Executive Review Control ERC-002: CEO review focuses on strategic risk, market speed, and vendor maturity.
- Executive Review Control ERC-003: CFO review focuses on value, budget timing, and service-credit exposure.
- Executive Review Control ERC-004: COO review focuses on throughput, continuity, and operating cadence.
- Executive Review Control ERC-005: CTO review focuses on integration depth, reliability, and architecture fit.
- Executive Review Control ERC-006: CISO review focuses on policy scope, evidence, and data class expansion.
- Executive Review Control ERC-007: Compliance review focuses on pack fit, retention, and jurisdiction.
- Executive Review Control ERC-008: Procurement review focuses on amendment path and vendor-risk deltas.
- Executive Review Control ERC-009: Executive deck includes current state, signal, recommendation, risk, cost, and next action.
- Executive Review Control ERC-010: Executive deck excludes speculative features not approved for the tenant.
- Executive Review Control ERC-011: Executive sponsor must hear the risk of doing nothing.
- Executive Review Control ERC-012: Executive sponsor must hear the operational change required.
- Executive Review Control ERC-013: Executive sponsor must hear the commercial change required.
- Executive Review Control ERC-014: Executive sponsor must hear the policy or compliance change required.
- Executive Review Control ERC-015: Executive sponsor must hear the first-value target after activation.
- Executive Review Control ERC-016: Executive review outcome is Approve, Revise, Defer, or Reject.
- Executive Review Control ERC-017: Approve moves to commercial commitment.
- Executive Review Control ERC-018: Revise returns to readiness analysis.
- Executive Review Control ERC-019: Defer returns to monitoring with a named revisit date.
- Executive Review Control ERC-020: Reject closes the expansion candidate with a documented reason.

## Cross-References

- `docs/GTM-PLAN.md` for packaging, customer success, and GTM expansion narrative.
- `docs/personas/MASTER-ROSTER-2026-05-21.md` for named sales, customer success, executive, finance, security, compliance, data, and operations personas.
- `docs/standards/capability-tier-matrix.md` for tier boundaries and package expectations.
- `specs/tenant-model.json` for tenant fields used in entitlement and expansion readiness.
- `docs/standards/tenant-lifecycle.md` for Active, Suspended, Migrating, and Offboarded lifecycle constraints.
- `docs/standards/cedar-policy-authoring.md` for policy rules governing entitlement changes.
- `contracts/openapi/foundry/capability-v1.yaml` for invoking tenant-granted capabilities.
- `contracts/openapi/platform/platform-policy-cedar-v1.yaml` for publishing expansion policy changes.
- `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md` for support and reliability expansion examples.
- `docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md` for QBR expansion framing.
- `docs/gtm/tenant-onboarding-90-day-program.md` for Day 90 account path and expansion readiness inputs.
