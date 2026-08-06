---
doc_class: GoToMarketPlaybook
title: Customer Success Quarterly Business Review Template
status: Draft
date: 2026-05-20
owner: GoToMarket / Customer Success
related_oyatie_adrs:
  - docs/adr-archive/ADR-0242-oyatie-is-a-tenant-doctrine.md
  - docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
  - docs/adr-archive/ADR-0009-cell-architecture-per-tenant-per-region.md
  - docs/adr-archive/ADR-0010-regional-pack-architecture.md
related_personas:
  - Customer Success Manager Sofia Rezende
  - CS-IC Lin Chen
  - Customer Champion Akemi Sato
  - Sales AE Maya Lindqvist
  - COO Akira Watanabe
  - CFO Helena Brandt
  - CTO Diego Vargas
  - CISO Yuki Park
  - CCO Naveen Iyer
  - Procurement Manager Wei Liu
  - Finance Director Mei-Ling Wu
  - Business Analyst Aditya Verma
  - Data Analyst Felipe Andrade
  - Compliance Officer Tunde Bello
  - External Auditor Hyo-Jin Lee
related_journeys:
  - docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md
  - docs/user-journeys/j117-api-customer-tenant-incident-response/README.md
  - docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Customer Success Quarterly Business Review Template

## Purpose

This playbook defines the Oyatie Quarterly Business Review format for Active tenants.

The QBR is the operating review where value, adoption, reliability, policy posture, support experience, risk, and expansion readiness are reviewed against named evidence.

The QBR is not a slide ritual.

The QBR is a customer governance checkpoint that connects the customer's business outcomes to tenant state, capability usage, SLA posture, evidence quality, and next-quarter actions.

The QBR begins when CSM Sofia Rezende opens the quarterly review pack.

The QBR ends when the customer accepts the business review, owners are assigned to next actions, risks are updated, and the account path is classified.

The QBR must include an executive narrative for COO Akira Watanabe, CFO Helena Brandt, CTO Diego Vargas, CISO Yuki Park, and other relevant stakeholders.

The QBR must include metric lineage for Data Analyst Felipe Andrade when data is used for executive decisions.

The QBR must include operational evidence for Business Analyst Aditya Verma when workflow change is under review.

The QBR must include policy and audit evidence for CISO Yuki Park, Compliance Officer Tunde Bello, and External Auditor Hyo-Jin Lee when security or compliance is in scope.

The QBR must include support and customer-impact evidence for CCO Naveen Iyer when incidents or customer commitments matter.

The QBR must include commercial and procurement context when expansion, renewal, package, or amendment decisions are discussed.

The QBR must not hide known risks behind success metrics.

The QBR must not use aggregate adoption numbers without naming blocked cohorts.

The QBR must not use SLA language that exceeds the purchased package.

The QBR must not pitch expansion before unresolved launch, support, security, or compliance blockers are named.

The QBR must tie every recommendation to an owner, due date, and success metric.

The QBR must update the account path as Stabilize, Optimize, Expand, or Recover.

This document provides the repeatable QBR agenda, metric list, risk-signal catalog, evidence checklist, and communication templates.

## Personas Involved (named — from MASTER-ROSTER)

- Customer Success Manager Sofia Rezende owns QBR preparation, facilitation, risk narrative, and account path.
- CS-IC Lin Chen owns metric collection, evidence links, implementation notes, and action tracking.
- Customer Champion Akemi Sato owns customer-side success narrative and internal advocacy when approved.
- Sales AE Maya Lindqvist owns commercial context, expansion path, renewal risk, and amendment follow-up.
- COO Akira Watanabe owns operating review, incident learning, process performance, and quarterly goals.
- CFO Helena Brandt owns ROI, budget, service-credit exposure, cost-center attribution, and renewal economics.
- CTO Diego Vargas owns architecture, integration, performance, reliability, and technical roadmap alignment.
- CISO Yuki Park owns security posture, policy enforcement, exception review, and audit evidence trust.
- CCO Naveen Iyer owns customer-impact review, support escalation, and external communication quality.
- Procurement Manager Wei Liu owns vendor-risk posture, contract changes, and procurement next steps.
- Finance Director Mei-Ling Wu owns invoice, usage cost, cost-center, and budget tracking.
- Business Analyst Aditya Verma owns workflow results, process adoption, and operational insights.
- Data Analyst Felipe Andrade owns metric lineage, dashboard quality, and export reliability.
- Compliance Officer Tunde Bello owns regulatory-pack evidence, jurisdiction, and audit preparedness.
- External Auditor Hyo-Jin Lee owns evidence completeness review when audit readiness is in scope.

## Stages / Steps (named, sequenced)

### QBR Stage 1: Prepare the Evidence Pack

- Stage 1 Exit Gate Q1-G1: QBR evidence pack has metrics, risks, incidents, support, policy, adoption, value, and next-action draft.
- Step 1.01: CSM opens QBR record for tenant.
- Step 1.02: CSM records quarter start and end dates.
- Step 1.03: CSM records tenant_id.
- Step 1.04: CSM records package tier.
- Step 1.05: CSM records support tier.
- Step 1.06: CSM records DR tier.
- Step 1.07: CSM records compliance packs.
- Step 1.08: CSM records account path from prior review.
- Step 1.09: CSM records renewal date.
- Step 1.10: CSM records executive sponsor.
- Step 1.11: CSM records QBR attendees.
- Step 1.12: CS-IC pulls adoption metrics.
- Step 1.13: CS-IC pulls workflow metrics.
- Step 1.14: CS-IC pulls active-user metrics.
- Step 1.15: CS-IC pulls request-rate metrics.
- Step 1.16: CS-IC pulls support metrics.
- Step 1.17: CS-IC pulls incident metrics.
- Step 1.18: CS-IC pulls SLA metrics.
- Step 1.19: CS-IC pulls policy-deny metrics.
- Step 1.20: CS-IC pulls audit evidence retrieval metrics.
- Step 1.21: CS-IC pulls migration metrics if applicable.
- Step 1.22: CS-IC pulls cost-center metrics.
- Step 1.23: CS-IC pulls dashboard usage metrics.
- Step 1.24: CS-IC pulls data-export metrics.
- Step 1.25: CS-IC pulls enablement completion metrics.
- Step 1.26: CSM reviews metric lineage.
- Step 1.27: CSM reviews open risks.
- Step 1.28: CSM reviews closed risks.
- Step 1.29: CSM reviews customer commitments.
- Step 1.30: CSM reviews Oyatie commitments.
- Step 1.31: CSM reviews expansion signals.
- Step 1.32: CSM reviews renewal risks.
- Step 1.33: CSM reviews unresolved security exceptions.
- Step 1.34: CSM reviews unresolved compliance exceptions.
- Step 1.35: CSM reviews procurement dependencies.
- Step 1.36: CSM reviews customer sentiment.
- Step 1.37: CSM drafts executive summary.
- Step 1.38: CSM drafts risk summary.
- Step 1.39: CSM drafts recommendation list.
- Step 1.40: CSM assigns internal pre-read reviewers.

### QBR Stage 2: Internal Review

- Stage 2 Exit Gate Q2-G1: CSM, CS-IC, AE, and escalation owners agree on claims, risks, and ask.
- Step 2.01: CS-IC verifies every metric has a source.
- Step 2.02: CS-IC verifies every metric has a period.
- Step 2.03: CS-IC verifies every metric has an owner.
- Step 2.04: CS-IC verifies every risk has owner and due date.
- Step 2.05: CS-IC verifies incident data matches support record.
- Step 2.06: CS-IC verifies support data matches customer-facing commitments.
- Step 2.07: CS-IC verifies policy-deny data does not expose sensitive details.
- Step 2.08: CS-IC verifies audit evidence links are accessible to approved roles.
- Step 2.09: CS-IC verifies expansion signals are evidence-backed.
- Step 2.10: CS-IC verifies SLA targets match package tier.
- Step 2.11: CSM verifies value narrative is defensible.
- Step 2.12: CSM verifies risks are not minimized.
- Step 2.13: CSM verifies account path recommendation.
- Step 2.14: CSM verifies next actions are realistic.
- Step 2.15: AE verifies commercial context.
- Step 2.16: AE verifies renewal or expansion sensitivity.
- Step 2.17: Sales Manager reviews if account is strategic or escalated.
- Step 2.18: Support owner reviews if incidents occurred.
- Step 2.19: Security owner reviews if policy or exception content appears.
- Step 2.20: Compliance owner reviews if audit or regulatory content appears.
- Step 2.21: CSM updates language after review.
- Step 2.22: CSM removes unsupported claims.
- Step 2.23: CSM marks evidence gaps.
- Step 2.24: CSM sends customer pre-read if appropriate.
- Step 2.25: CSM confirms final agenda.

### QBR Stage 3: Customer Meeting

- Stage 3 Exit Gate Q3-G1: Customer agrees to metric interpretation, risk status, and next-quarter action plan or records objections.
- Step 3.01: CSM opens with quarter outcome and account path.
- Step 3.02: CSM confirms agenda.
- Step 3.03: CSM confirms attendees and roles.
- Step 3.04: CSM reviews business objectives from prior quarter.
- Step 3.05: CSM reviews value delivered.
- Step 3.06: CSM reviews adoption.
- Step 3.07: CSM reviews workflow performance.
- Step 3.08: CSM reviews reliability and SLA posture.
- Step 3.09: CSM reviews incidents and corrective actions.
- Step 3.10: CSM reviews support experience.
- Step 3.11: CSM reviews security and policy posture.
- Step 3.12: CSM reviews compliance and audit evidence.
- Step 3.13: CSM reviews migration outcomes if applicable.
- Step 3.14: CSM reviews FinOps and cost-center data.
- Step 3.15: CSM reviews open risks.
- Step 3.16: CSM reviews expansion signals if appropriate.
- Step 3.17: CSM reviews next-quarter recommendations.
- Step 3.18: Customer validates or challenges metrics.
- Step 3.19: Customer validates or challenges risks.
- Step 3.20: Customer validates or challenges recommendations.
- Step 3.21: AE handles commercial discussion if expansion or renewal is in scope.
- Step 3.22: CS-IC records technical follow-ups.
- Step 3.23: CSM records owner and due date for every action.
- Step 3.24: CSM confirms next review cadence.
- Step 3.25: CSM closes with accepted account path.

### QBR Stage 4: Closeout and Action Tracking

- Stage 4 Exit Gate Q4-G1: QBR recap is sent, actions are tracked, and account health is updated.
- Step 4.01: CSM sends recap within two business days.
- Step 4.02: CSM attaches reviewed metric pack.
- Step 4.03: CSM attaches evidence appendix.
- Step 4.04: CSM records accepted actions.
- Step 4.05: CSM records disputed metrics.
- Step 4.06: CSM records disputed risks.
- Step 4.07: CSM records customer commitments.
- Step 4.08: CSM records Oyatie commitments.
- Step 4.09: CSM records expansion next steps.
- Step 4.10: CSM records renewal risks.
- Step 4.11: CSM updates account path.
- Step 4.12: CSM updates health score.
- Step 4.13: CSM updates risk register.
- Step 4.14: CSM updates success plan.
- Step 4.15: CSM updates support watchlist.
- Step 4.16: CSM updates expansion signal register.
- Step 4.17: CSM updates next QBR date.
- Step 4.18: CS-IC creates implementation tasks.
- Step 4.19: AE creates commercial tasks.
- Step 4.20: CSM archives QBR evidence.

### Standard QBR Agenda

- Agenda Block A01: Opening and meeting objective.
- Agenda Block A02: Attendee roles and decision context.
- Agenda Block A03: Prior-quarter commitments.
- Agenda Block A04: Executive outcome summary.
- Agenda Block A05: Tenant state and package snapshot.
- Agenda Block A06: Adoption and active-user review.
- Agenda Block A07: Workflow performance review.
- Agenda Block A08: Business value and ROI review.
- Agenda Block A09: Reliability and SLA posture.
- Agenda Block A10: Incident review and corrective actions.
- Agenda Block A11: Support experience.
- Agenda Block A12: Security and policy posture.
- Agenda Block A13: Compliance and audit evidence.
- Agenda Block A14: Migration and data-quality status.
- Agenda Block A15: FinOps and cost-center review.
- Agenda Block A16: Risk-signal review.
- Agenda Block A17: Expansion or optimization signals.
- Agenda Block A18: Next-quarter plan.
- Agenda Block A19: Owner and due-date confirmation.
- Agenda Block A20: Account path and close.

### Persona-Specific Agenda Notes

- COO Agenda Note COO-01: Lead with throughput, latency, capacity utilization, incident learning, and action closure.
- COO Agenda Note COO-02: Show whether operations improved or whether work moved from one bottleneck to another.
- COO Agenda Note COO-03: Ask which next-quarter operating goal should map to Oyatie usage.
- CFO Agenda Note CFO-01: Lead with cost-center, ROI, service-credit exposure, renewal economics, and budget timing.
- CFO Agenda Note CFO-02: Separate realized value from projected expansion value.
- CFO Agenda Note CFO-03: Ask which financial measure should anchor the next review.
- CTO Agenda Note CTO-01: Lead with architecture stability, integration health, request rate, latency, and incident trend.
- CTO Agenda Note CTO-02: Separate platform limits from customer configuration issues.
- CTO Agenda Note CTO-03: Ask whether technical risk is acceptable for next-quarter plan.
- CISO Agenda Note CISO-01: Lead with policy enforcement, exceptions, audit evidence, access lifecycle, and regulated-data posture.
- CISO Agenda Note CISO-02: Show allow and deny evidence when policy trust is under review.
- CISO Agenda Note CISO-03: Ask whether any new data class or principal type needs review.
- Compliance Agenda Note COM-01: Lead with regulatory-pack coverage, evidence retention, audit stream access, and jurisdiction.
- Compliance Agenda Note COM-02: Separate available evidence from non-contracted evidence.
- Compliance Agenda Note COM-03: Ask whether external audit or regulator timeline changed.

## Named Tools and Cedar Permits (specific µservices + actions + Cedar policies)

### QBR Tools

- Tool: `ops-dashboard-control-center`; Action: `ops-dashboard-control-center:ViewQuarterlyReview`; QBR use: operational review dashboard.
- Tool: `observability-service`; Action: `observability:ReadServiceHealth`; QBR use: latency, throughput, error, and incident trend.
- Tool: `incident-management-service`; Action: `incident-management:ReadDebrief`; QBR use: incident timeline and corrective action review.
- Tool: `slo-budgets-service`; Action: `slo-budgets:ReadBudget`; QBR use: SLA posture and error budget.
- Tool: `audit-chain-service`; Action: `audit-chain:ReadEvidence`; QBR use: audit evidence retrieval.
- Tool: `workflow-engine`; Action: `workflow-engine:ReadWorkflowMetrics`; QBR use: workflow completion, failure, and cycle-time metrics.
- Tool: `identity-service`; Action: `identity:ReadTenantPrincipals`; QBR use: active users, admin roles, and access lifecycle.
- Tool: `finops-portal`; Action: `finops-portal:ReadTenantCostCenter`; QBR use: cost-center and usage-cost review.
- Tool: `support-service`; Action: `support:ReadTenantTickets`; QBR use: support case trend and response quality.
- Tool: `migration-validate-service`; Action: `oya.migration.validate`; QBR use: migration completeness and exception status.
- Tool: `foundry-capability-service`; Action: `foundry-capability:ReadCapabilityUsage`; QBR use: capability usage and expansion signal.
- Tool: `platform-tenant-service`; Action: `platform-tenant:GetTenant`; QBR use: tenant state, package, cell, and compliance snapshot.

### QBR Cedar Permits

- Cedar Permit: `cedar.tenant.qbr.executive_viewer`; Allows executive stakeholders to view QBR pack.
- Cedar Permit: `cedar.tenant.qbr.csm_editor`; Allows CSM to prepare and update QBR record.
- Cedar Permit: `cedar.tenant.qbr.csic_evidence_writer`; Allows CS-IC to attach evidence artifacts.
- Cedar Permit: `cedar.tenant.qbr.finance_viewer`; Allows finance stakeholders to view cost and value metrics.
- Cedar Permit: `cedar.tenant.qbr.operations_viewer`; Allows operations stakeholders to view workflow and reliability metrics.
- Cedar Permit: `cedar.tenant.qbr.security_viewer`; Allows security stakeholders to view policy, access, and exception posture.
- Cedar Permit: `cedar.tenant.qbr.compliance_viewer`; Allows compliance stakeholders to view regulatory-pack and audit evidence.
- Cedar Permit: `cedar.tenant.qbr.procurement_viewer`; Allows Procurement to view vendor-risk and contract context.
- Cedar Permit: `cedar.tenant.qbr.support_viewer`; Allows CCO or support stakeholders to view support experience.
- Cedar Permit: `cedar.tenant.qbr.expansion_signal_reader`; Allows AE and CSM to view expansion signals.
- Cedar Permit: `cedar.tenant.qbr.audit_reader_external`; Allows approved external auditor read-only evidence access.
- Cedar Permit: `cedar.tenant.qbr.no_sensitive_export`; Denies export of sensitive evidence to unauthorized principals.
- Cedar Permit: `cedar.tenant.qbr.action_owner_editor`; Allows named owners to update assigned QBR actions.
- Cedar Permit: `cedar.tenant.qbr.archive_writer`; Allows QBR closeout archive.
- Cedar Permit: `cedar.tenant.qbr.closeout_guard`; Blocks closeout if owner, due date, account path, or evidence links are missing.

## Specific Metrics + Named SLA Targets

### Named QBR Metrics

- Metric QBR-M001: Active Users; Definition: unique active tenant users in quarter.
- Metric QBR-M002: Invited Users; Definition: users invited to tenant.
- Metric QBR-M003: Activation Rate; Definition: active users divided by invited users.
- Metric QBR-M004: Admin Action Count; Definition: tenant-admin configuration actions.
- Metric QBR-M005: Workflow Completion Count; Definition: completed target workflows.
- Metric QBR-M006: Workflow Completion Rate; Definition: completed workflows divided by started workflows.
- Metric QBR-M007: Workflow Failure Rate; Definition: failed workflows divided by started workflows.
- Metric QBR-M008: Quote-to-Contract Cycle Time; Definition: elapsed time for j54 workflow if used.
- Metric QBR-M009: First-Payment Completion Rate; Definition: completed first payments divided by attempted first payments.
- Metric QBR-M010: Incident Count; Definition: tenant-impacting incidents.
- Metric QBR-M011: P1 Acknowledgement Time; Definition: time from P1 declaration to acknowledgement.
- Metric QBR-M012: P1 Resolution Time; Definition: time from P1 declaration to resolution.
- Metric QBR-M013: Customer Notification Latency; Definition: time from incident confirmation to customer notification.
- Metric QBR-M014: Corrective Action Closure Rate; Definition: closed corrective actions divided by opened corrective actions.
- Metric QBR-M015: Latency P50; Definition: median response latency for target workflow.
- Metric QBR-M016: Latency P95; Definition: p95 response latency for target workflow.
- Metric QBR-M017: Latency P99; Definition: p99 response latency for target workflow.
- Metric QBR-M018: Throughput; Definition: completed operations per period.
- Metric QBR-M019: Request Rate Peak; Definition: highest sustained request rate.
- Metric QBR-M020: Error Budget Burn; Definition: consumed error budget for period.
- Metric QBR-M021: Availability Actual; Definition: measured availability for purchased package.
- Metric QBR-M022: SLA Credit Exposure; Definition: estimated credit exposure from SLA misses.
- Metric QBR-M023: Support Ticket Count; Definition: support tickets opened by tenant.
- Metric QBR-M024: Support First Response Time; Definition: time to first response by severity.
- Metric QBR-M025: Support Reopen Rate; Definition: reopened tickets divided by closed tickets.
- Metric QBR-M026: Support Escalation Rate; Definition: escalated tickets divided by total tickets.
- Metric QBR-M027: Policy Deny Count; Definition: denied actions in tenant scope.
- Metric QBR-M028: Unexpected Deny Count; Definition: deny events classified as unexpected.
- Metric QBR-M029: Access Exception Count; Definition: approved security exceptions.
- Metric QBR-M030: Auditor Evidence Retrieval Time; Definition: time to retrieve requested audit evidence.
- Metric QBR-M031: Audit Evidence Completeness; Definition: requested evidence delivered divided by requested evidence.
- Metric QBR-M032: Compliance Pack Coverage; Definition: active packs divided by required packs.
- Metric QBR-M033: Data Export Count; Definition: governed exports by analysts.
- Metric QBR-M034: Metric Lineage Coverage; Definition: dashboard metrics with lineage.
- Metric QBR-M035: Migration Object Count; Definition: migrated source objects.
- Metric QBR-M036: Migration Validation Pass Rate; Definition: validated records divided by imported records.
- Metric QBR-M037: Migration Exception Count; Definition: unresolved migration exceptions.
- Metric QBR-M038: Cost Center Spend; Definition: usage cost assigned to tenant cost center.
- Metric QBR-M039: Cost Per Active User; Definition: spend divided by active users.
- Metric QBR-M040: Cost Per Completed Workflow; Definition: spend divided by completed workflows.
- Metric QBR-M041: Enablement Completion Rate; Definition: completed enablement tasks divided by assigned tasks.
- Metric QBR-M042: Champion Engagement; Definition: champion attendance and action completion.
- Metric QBR-M043: Executive Attendance; Definition: executive stakeholder attendance at review.
- Metric QBR-M044: Open Risk Count; Definition: unresolved QBR risks.
- Metric QBR-M045: High Risk Count; Definition: unresolved high-severity risks.
- Metric QBR-M046: Action Closure Rate; Definition: prior QBR actions closed by due date.
- Metric QBR-M047: Expansion Signal Count; Definition: qualified expansion signals.
- Metric QBR-M048: Renewal Risk Count; Definition: named renewal risks.
- Metric QBR-M049: Health Score; Definition: Customer Success health classification.
- Metric QBR-M050: Account Path; Definition: Stabilize, Optimize, Expand, or Recover.

### Named SLA Targets

- SLA Target QBR-SLA-017: QBR prep target is evidence pack complete five business days before meeting.
- SLA Target QBR-SLA-018: QBR recap target is sent within two business days after meeting.
- SLA Target QBR-SLA-019: QBR action-owner target is every action has owner and due date before close.
- SLA Target QBR-SLA-020: QBR evidence retrieval target is all referenced evidence accessible to approved viewers before meeting.

## Named Failure Modes + Recovery

### Risk Signals

- Risk Signal RS-001: `active_user_decline`; Meaning: active users decline for two consecutive periods; Recovery: adoption diagnosis.
- Risk Signal RS-002: `invited_but_inactive`; Meaning: invited users do not activate; Recovery: enablement and admin review.
- Risk Signal RS-003: `workflow_failure_spike`; Meaning: workflow failure rate rises above baseline; Recovery: technical diagnosis.
- Risk Signal RS-004: `policy_deny_spike`; Meaning: deny events increase unexpectedly; Recovery: policy review.
- Risk Signal RS-005: `unexpected_deny_persistent`; Meaning: legitimate work is blocked repeatedly; Recovery: entitlement and policy review.
- Risk Signal RS-006: `support_ticket_growth`; Meaning: support tickets grow faster than usage; Recovery: enablement or support-tier review.
- Risk Signal RS-007: `p1_repeat`; Meaning: repeated P1 incidents; Recovery: reliability review.
- Risk Signal RS-008: `notification_delay`; Meaning: incident notifications miss target; Recovery: incident runbook update.
- Risk Signal RS-009: `evidence_retrieval_delay`; Meaning: audit evidence takes too long to retrieve; Recovery: evidence pipeline review.
- Risk Signal RS-010: `metric_lineage_gap`; Meaning: executive metric lacks source lineage; Recovery: data analyst correction.
- Risk Signal RS-011: `cost_surprise`; Meaning: Finance disputes cost or allocation; Recovery: FinOps review.
- Risk Signal RS-012: `service_credit_exposure`; Meaning: SLA credit risk rises; Recovery: support and reliability review.
- Risk Signal RS-013: `migration_exception_backlog`; Meaning: unresolved migration exceptions persist; Recovery: migration remediation.
- Risk Signal RS-014: `security_exception_backlog`; Meaning: unresolved security exceptions persist; Recovery: CISO review.
- Risk Signal RS-015: `compliance_pack_gap`; Meaning: regulatory ask exceeds active pack; Recovery: compliance-pack review.
- Risk Signal RS-016: `procurement_blocker`; Meaning: vendor-risk or amendment process blocks action; Recovery: Procurement plan.
- Risk Signal RS-017: `champion_absent`; Meaning: champion misses QBR or stops engaging; Recovery: stakeholder remap.
- Risk Signal RS-018: `executive_no_show`; Meaning: executive sponsor absent from repeated reviews; Recovery: executive escalation.
- Risk Signal RS-019: `expansion_before_value`; Meaning: expansion ask appears before value proof; Recovery: hold expansion and prove value.
- Risk Signal RS-020: `renewal_risk_unowned`; Meaning: renewal risk has no owner; Recovery: Sales Manager review.
- Risk Signal RS-021: `package_mismatch`; Meaning: customer expects features above tier; Recovery: package boundary review.
- Risk Signal RS-022: `dr_expectation_gap`; Meaning: RTO/RPO expectation exceeds package; Recovery: DR-tier review.
- Risk Signal RS-023: `audit_window_gap`; Meaning: evidence retention expectation exceeds package; Recovery: retention or tier review.
- Risk Signal RS-024: `admin_overload`; Meaning: admin actions or support requests show operational strain; Recovery: admin enablement.
- Risk Signal RS-025: `manual_workaround_return`; Meaning: users return to old process; Recovery: workflow redesign.

### QBR Failure Modes

- Failure Mode: `QBR-NO-EVIDENCE`; Signal: slides contain claims without evidence links; Recovery: delay QBR or mark claims as unsupported.
- Failure Mode: `QBR-METRIC-MISMATCH`; Signal: customer disputes metric definition; Recovery: metric lineage review with Data Analyst Felipe Andrade.
- Failure Mode: `QBR-SLA-OVERCLAIM`; Signal: QBR cites SLA above purchased tier; Recovery: correct record and clarify package target.
- Failure Mode: `QBR-RISK-HIDDEN`; Signal: known risks omitted from customer review; Recovery: send corrected risk register and apologize.
- Failure Mode: `QBR-NO-ACTIONS`; Signal: meeting ends without owners and due dates; Recovery: schedule action closeout within two business days.
- Failure Mode: `QBR-EXPANSION-PREMATURE`; Signal: upsell presented while account path is Recover; Recovery: withdraw ask and present recovery plan.
- Failure Mode: `QBR-SECURITY-LEAK`; Signal: sensitive evidence shown to unauthorized viewer; Recovery: stop meeting, revoke access, and investigate.
- Failure Mode: `QBR-COMPLIANCE-CONFUSION`; Signal: compliance evidence is conflated with non-contracted pack; Recovery: clarify active pack and optional path.
- Failure Mode: `QBR-EXECUTIVE-MISALIGN`; Signal: executive sponsor rejects value narrative; Recovery: revise outcome model and schedule sponsor follow-up.
- Failure Mode: `QBR-CLOSEOUT-MISSING`; Signal: no recap or action register after review; Recovery: CSM manager escalation.

## Sample Dialogue / Email Templates

### Template 1: QBR Invite

Subject: Quarterly Business Review for {{tenant_name}}

Hi {{executive_sponsor}},

We are ready to review the last quarter of Oyatie usage for {{tenant_name}}.

The proposed agenda covers value delivered, adoption, operational health, incidents, support, security, compliance evidence, open risks, and next-quarter actions.

Recommended attendees:

- Executive sponsor
- Operations owner
- Technical owner
- Security or compliance owner if in scope
- Finance owner if cost-center or expansion topics are in scope

Please confirm whether {{date}} works for the QBR.

Regards,

{{csm_name}}

### Template 2: QBR Pre-Read

Subject: Pre-read for {{tenant_name}} QBR

Hi {{customer_team}},

Attached is the pre-read for the upcoming QBR.

Please review these sections before the meeting:

1. Executive outcome summary.
2. Adoption and workflow metrics.
3. Reliability, support, and incident review.
4. Security, compliance, and audit evidence.
5. Open risks.
6. Proposed next-quarter actions.

Any metric marked "lineage review needed" will be discussed live rather than treated as final.

Regards,

{{csm_name}}

### Template 3: QBR Risk Escalation

Subject: QBR risk requiring owner - {{risk_name}}

Hi {{owner_name}},

The QBR identified a risk that needs an accountable owner.

Risk:

- {{risk_name}}

Signal:

- {{risk_signal}}

Impact:

- {{impact}}

Recommended recovery:

- {{recovery}}

Please confirm ownership and target date by {{date}}.

Regards,

{{csm_name}}

### Template 4: QBR Recap

Subject: QBR recap and next actions for {{tenant_name}}

Hi {{customer_team}},

Thank you for the QBR discussion.

Accepted account path:

- {{account_path}}

Confirmed outcomes:

- {{outcome_1}}
- {{outcome_2}}

Open risks:

- {{risk_1}} - owner: {{owner_1}} - due: {{date_1}}
- {{risk_2}} - owner: {{owner_2}} - due: {{date_2}}

Next-quarter actions:

- {{action_1}}
- {{action_2}}
- {{action_3}}

We will track these items in the customer success plan and review progress at the next checkpoint.

Regards,

{{csm_name}}

### Template 5: Expansion Readiness From QBR

Subject: Expansion readiness follow-up from QBR

Hi {{buyer_name}},

The QBR surfaced a qualified expansion signal:

- {{signal_name}}: {{signal_value}}

The current package constraint is:

- {{constraint}}

Recommended next step:

- {{recommended_step}}

We will separate this review from the current risk register and will not activate any entitlement changes without commercial and policy approval.

Regards,

{{csm_or_ae_name}}

## QBR Evidence Workbook

### Metric Workbook Controls

- Metric Workbook Control MWC-001: Active Users includes quarter value, prior-quarter value, and trend.
- Metric Workbook Control MWC-002: Invited Users includes total invited and pending invite count.
- Metric Workbook Control MWC-003: Activation Rate includes denominator definition.
- Metric Workbook Control MWC-004: Admin Action Count separates customer admin from Oyatie support action.
- Metric Workbook Control MWC-005: Workflow Completion Count names workflow id.
- Metric Workbook Control MWC-006: Workflow Completion Rate states started-workflow definition.
- Metric Workbook Control MWC-007: Workflow Failure Rate includes top three failure categories.
- Metric Workbook Control MWC-008: Quote-to-Contract Cycle Time names journey j54 when used.
- Metric Workbook Control MWC-009: First-Payment Completion Rate separates payment authorization from contract completion.
- Metric Workbook Control MWC-010: Incident Count separates P1, P2, P3, and informational events.
- Metric Workbook Control MWC-011: P1 Acknowledgement Time compares actual to tier target.
- Metric Workbook Control MWC-012: P1 Resolution Time names incident owner.
- Metric Workbook Control MWC-013: Customer Notification Latency compares actual to notification target.
- Metric Workbook Control MWC-014: Corrective Action Closure Rate includes overdue items.
- Metric Workbook Control MWC-015: Latency P50 names service and workflow.
- Metric Workbook Control MWC-016: Latency P95 names service and workflow.
- Metric Workbook Control MWC-017: Latency P99 names service and workflow.
- Metric Workbook Control MWC-018: Throughput names measurement period.
- Metric Workbook Control MWC-019: Request Rate Peak compares actual to tier target.
- Metric Workbook Control MWC-020: Error Budget Burn includes remaining budget.
- Metric Workbook Control MWC-021: Availability Actual compares actual to purchased package target.
- Metric Workbook Control MWC-022: SLA Credit Exposure states whether credit is actual, estimated, or not applicable.
- Metric Workbook Control MWC-023: Support Ticket Count separates how-to, defect, incident, billing, and access tickets.
- Metric Workbook Control MWC-024: Support First Response Time compares actual to support target.
- Metric Workbook Control MWC-025: Support Reopen Rate includes root cause category.
- Metric Workbook Control MWC-026: Support Escalation Rate names escalation owner.
- Metric Workbook Control MWC-027: Policy Deny Count separates expected and unexpected denies.
- Metric Workbook Control MWC-028: Unexpected Deny Count includes remediation status.
- Metric Workbook Control MWC-029: Access Exception Count includes exception expiry dates.
- Metric Workbook Control MWC-030: Auditor Evidence Retrieval Time includes requested evidence type.
- Metric Workbook Control MWC-031: Audit Evidence Completeness names missing evidence if incomplete.
- Metric Workbook Control MWC-032: Compliance Pack Coverage names active and requested packs.
- Metric Workbook Control MWC-033: Data Export Count names export purpose.
- Metric Workbook Control MWC-034: Metric Lineage Coverage includes owner for missing lineage.
- Metric Workbook Control MWC-035: Migration Object Count names source system.
- Metric Workbook Control MWC-036: Migration Validation Pass Rate includes failed object class.
- Metric Workbook Control MWC-037: Migration Exception Count includes oldest exception age.
- Metric Workbook Control MWC-038: Cost Center Spend includes cost-center id.
- Metric Workbook Control MWC-039: Cost Per Active User includes active-user denominator.
- Metric Workbook Control MWC-040: Cost Per Completed Workflow includes workflow denominator.
- Metric Workbook Control MWC-041: Enablement Completion Rate includes assigned cohort.
- Metric Workbook Control MWC-042: Champion Engagement includes customer champion name.
- Metric Workbook Control MWC-043: Executive Attendance includes missing executive stakeholders.
- Metric Workbook Control MWC-044: Open Risk Count separates customer-owned and Oyatie-owned risks.
- Metric Workbook Control MWC-045: High Risk Count includes executive escalation status.
- Metric Workbook Control MWC-046: Action Closure Rate includes overdue actions.
- Metric Workbook Control MWC-047: Expansion Signal Count names qualified and unqualified signals.
- Metric Workbook Control MWC-048: Renewal Risk Count includes owner and mitigation.
- Metric Workbook Control MWC-049: Health Score includes reason code.
- Metric Workbook Control MWC-050: Account Path includes rationale.

### Agenda Facilitation Controls

- Agenda Control AGC-001: Opening states whether review is strategic, operational, risk-focused, or expansion-focused.
- Agenda Control AGC-002: Attendee confirmation maps each attendee to a persona and decision role.
- Agenda Control AGC-003: Prior-quarter commitments are reviewed before new recommendations.
- Agenda Control AGC-004: Executive summary is limited to evidence-backed outcomes.
- Agenda Control AGC-005: Tenant snapshot states tier, support target, DR target, and compliance packs.
- Agenda Control AGC-006: Adoption review names active, passive, blocked, and not-invited cohorts.
- Agenda Control AGC-007: Workflow review names the top workflow and the weakest workflow.
- Agenda Control AGC-008: Value review names realized value and unresolved value assumptions.
- Agenda Control AGC-009: Reliability review compares actuals to purchased tier.
- Agenda Control AGC-010: Incident review names corrective actions and closure status.
- Agenda Control AGC-011: Support review separates product issues from enablement issues.
- Agenda Control AGC-012: Security review names exceptions and expiry dates.
- Agenda Control AGC-013: Compliance review names active pack and requested pack gaps.
- Agenda Control AGC-014: Migration review names completed, blocked, and deferred waves.
- Agenda Control AGC-015: FinOps review names cost center and disputed costs.
- Agenda Control AGC-016: Risk review names owner, due date, and severity.
- Agenda Control AGC-017: Expansion review is skipped when account path is Recover.
- Agenda Control AGC-018: Next-quarter plan uses no ownerless action items.
- Agenda Control AGC-019: Account path is stated before close.
- Agenda Control AGC-020: Close confirms recap timing.

### Persona Review Cards

- Persona Review Card PRC-001: COO Akira Watanabe card includes throughput trend.
- Persona Review Card PRC-002: COO Akira Watanabe card includes latency p99 trend.
- Persona Review Card PRC-003: COO Akira Watanabe card includes incident debrief status.
- Persona Review Card PRC-004: COO Akira Watanabe card includes capacity utilization.
- Persona Review Card PRC-005: COO Akira Watanabe card includes next-quarter operational goal.
- Persona Review Card PRC-006: CFO Helena Brandt card includes spend trend.
- Persona Review Card PRC-007: CFO Helena Brandt card includes service-credit exposure.
- Persona Review Card PRC-008: CFO Helena Brandt card includes ROI or value proxy.
- Persona Review Card PRC-009: CFO Helena Brandt card includes budget timing.
- Persona Review Card PRC-010: CFO Helena Brandt card includes cost-center allocation.
- Persona Review Card PRC-011: CTO Diego Vargas card includes integration health.
- Persona Review Card PRC-012: CTO Diego Vargas card includes request-rate trend.
- Persona Review Card PRC-013: CTO Diego Vargas card includes architecture risk.
- Persona Review Card PRC-014: CTO Diego Vargas card includes reliability constraints.
- Persona Review Card PRC-015: CTO Diego Vargas card includes technical roadmap dependency.
- Persona Review Card PRC-016: CISO Yuki Park card includes policy-deny pattern.
- Persona Review Card PRC-017: CISO Yuki Park card includes access exceptions.
- Persona Review Card PRC-018: CISO Yuki Park card includes regulated-data posture.
- Persona Review Card PRC-019: CISO Yuki Park card includes audit evidence retrieval.
- Persona Review Card PRC-020: CISO Yuki Park card includes security next actions.
- Persona Review Card PRC-021: Compliance Officer Tunde Bello card includes regulatory-pack coverage.
- Persona Review Card PRC-022: Compliance Officer Tunde Bello card includes jurisdiction status.
- Persona Review Card PRC-023: Compliance Officer Tunde Bello card includes retention target.
- Persona Review Card PRC-024: Compliance Officer Tunde Bello card includes evidence gaps.
- Persona Review Card PRC-025: Compliance Officer Tunde Bello card includes external audit timing.
- Persona Review Card PRC-026: Finance Director Mei-Ling Wu card includes invoice routing.
- Persona Review Card PRC-027: Finance Director Mei-Ling Wu card includes cost-per-user.
- Persona Review Card PRC-028: Finance Director Mei-Ling Wu card includes disputed cost items.
- Persona Review Card PRC-029: Finance Director Mei-Ling Wu card includes cost-center owner.
- Persona Review Card PRC-030: Finance Director Mei-Ling Wu card includes next-quarter budget action.
- Persona Review Card PRC-031: Business Analyst Aditya Verma card includes process adoption.
- Persona Review Card PRC-032: Business Analyst Aditya Verma card includes workflow completion.
- Persona Review Card PRC-033: Business Analyst Aditya Verma card includes manual workaround evidence.
- Persona Review Card PRC-034: Business Analyst Aditya Verma card includes business-user feedback.
- Persona Review Card PRC-035: Business Analyst Aditya Verma card includes workflow improvement action.
- Persona Review Card PRC-036: Data Analyst Felipe Andrade card includes metric lineage.
- Persona Review Card PRC-037: Data Analyst Felipe Andrade card includes dashboard usage.
- Persona Review Card PRC-038: Data Analyst Felipe Andrade card includes export usage.
- Persona Review Card PRC-039: Data Analyst Felipe Andrade card includes data-quality issue.
- Persona Review Card PRC-040: Data Analyst Felipe Andrade card includes analytics next action.

### Risk Register Controls

- Risk Register Control RRC-001: Every risk has one owner.
- Risk Register Control RRC-002: Every risk has one due date.
- Risk Register Control RRC-003: Every risk has severity.
- Risk Register Control RRC-004: Every risk has signal source.
- Risk Register Control RRC-005: Every risk has customer impact.
- Risk Register Control RRC-006: Every risk has recovery path.
- Risk Register Control RRC-007: Every risk has status.
- Risk Register Control RRC-008: Every risk has next review date.
- Risk Register Control RRC-009: Security risks name CISO or delegate.
- Risk Register Control RRC-010: Compliance risks name compliance owner.
- Risk Register Control RRC-011: Finance risks name finance owner.
- Risk Register Control RRC-012: Operational risks name operations owner.
- Risk Register Control RRC-013: Technical risks name technical owner.
- Risk Register Control RRC-014: Support risks name support owner.
- Risk Register Control RRC-015: Migration risks name migration owner.
- Risk Register Control RRC-016: Procurement risks name procurement owner.
- Risk Register Control RRC-017: Expansion risks name CSM and AE owner.
- Risk Register Control RRC-018: Renewal risks name AE and Sales Manager owner.
- Risk Register Control RRC-019: High risks are included in executive summary.
- Risk Register Control RRC-020: Closed risks retain closure evidence.
- Risk Register Control RRC-021: Recurring risks become root-cause actions.
- Risk Register Control RRC-022: Unowned risks block QBR closeout.
- Risk Register Control RRC-023: Overdue high risks trigger escalation.
- Risk Register Control RRC-024: Risk acceptance requires named approver.
- Risk Register Control RRC-025: Risk deletion requires closure rationale.

### Action Register Controls

- Action Register Control ARC-001: Action item has owner.
- Action Register Control ARC-002: Action item has due date.
- Action Register Control ARC-003: Action item has source agenda block.
- Action Register Control ARC-004: Action item has success condition.
- Action Register Control ARC-005: Action item has customer or Oyatie ownership label.
- Action Register Control ARC-006: Action item has escalation path.
- Action Register Control ARC-007: Action item has next checkpoint.
- Action Register Control ARC-008: Customer action is confirmed in recap.
- Action Register Control ARC-009: Oyatie action is entered into internal tracker.
- Action Register Control ARC-010: Commercial action is owned by AE.
- Action Register Control ARC-011: Technical action is owned by CS-IC or technical delegate.
- Action Register Control ARC-012: Security action is owned by CISO or delegate.
- Action Register Control ARC-013: Compliance action is owned by compliance owner.
- Action Register Control ARC-014: Finance action is owned by finance owner.
- Action Register Control ARC-015: Support action is owned by support owner.
- Action Register Control ARC-016: Migration action is owned by migration owner.
- Action Register Control ARC-017: Expansion action is tracked separately from recovery action.
- Action Register Control ARC-018: Recovery action blocks upsell until accepted.
- Action Register Control ARC-019: Completed action includes evidence link.
- Action Register Control ARC-020: Overdue action appears in next customer checkpoint.
- Action Register Control ARC-021: Action closure rate is reported next QBR.
- Action Register Control ARC-022: Unclear action is rewritten before closeout.
- Action Register Control ARC-023: Duplicate actions are consolidated.
- Action Register Control ARC-024: Cancelled action includes reason.
- Action Register Control ARC-025: Action register is archived with QBR pack.

### Account Path Decision Rules

- Account Path Rule APR-001: Stabilize requires value evidence and controlled risk.
- Account Path Rule APR-002: Stabilize requires no material adoption decline.
- Account Path Rule APR-003: Stabilize requires no unowned high risk.
- Account Path Rule APR-004: Optimize requires value evidence plus improvement opportunity.
- Account Path Rule APR-005: Optimize requires owner for improvement action.
- Account Path Rule APR-006: Optimize may include enablement, workflow redesign, or policy cleanup.
- Account Path Rule APR-007: Expand requires value evidence, controlled risk, and qualified expansion signal.
- Account Path Rule APR-008: Expand requires AE alignment.
- Account Path Rule APR-009: Expand requires no unresolved blocker that would make upsell irresponsible.
- Account Path Rule APR-010: Recover is required when value is unproven and risk is material.
- Account Path Rule APR-011: Recover is required when executive sponsor rejects value narrative.
- Account Path Rule APR-012: Recover is required when critical support or trust issues remain unowned.
- Account Path Rule APR-013: Account path is reviewed with customer.
- Account Path Rule APR-014: Account path rationale is included in QBR recap.
- Account Path Rule APR-015: Account path drives next-quarter cadence.
- Account Path Rule APR-016: Account path drives AE involvement level.
- Account Path Rule APR-017: Account path drives escalation requirements.
- Account Path Rule APR-018: Account path can change mid-quarter only with documented reason.
- Account Path Rule APR-019: Prior account path is retained for trend.
- Account Path Rule APR-020: Missing account path blocks QBR closeout.

### QBR Archive Controls

- Archive Control QAC-001: Final QBR deck is archived after recap.
- Archive Control QAC-002: Metric workbook is archived with source date.
- Archive Control QAC-003: Risk register snapshot is archived with owner list.
- Archive Control QAC-004: Action register snapshot is archived with due dates.
- Archive Control QAC-005: Evidence appendix is archived with access policy id.
- Archive Control QAC-006: Attendance list is archived with persona mapping.
- Archive Control QAC-007: Customer objections are archived verbatim when material.
- Archive Control QAC-008: Corrected metrics are archived with correction reason.
- Archive Control QAC-009: SLA references are archived with package tier.
- Archive Control QAC-010: Incident references are archived with incident ids.
- Archive Control QAC-011: Support references are archived with ticket ids.
- Archive Control QAC-012: Policy references are archived with policy ids.
- Archive Control QAC-013: Audit references are archived with audit stream ids.
- Archive Control QAC-014: Migration references are archived with source system.
- Archive Control QAC-015: FinOps references are archived with cost-center id.
- Archive Control QAC-016: Expansion references are archived with signal id.
- Archive Control QAC-017: Renewal references are archived with renewal date.
- Archive Control QAC-018: Account path is archived with rationale.
- Archive Control QAC-019: Customer recap is archived with sent timestamp.
- Archive Control QAC-020: Internal debrief is archived with lessons learned.
- Archive Control QAC-021: Sensitive evidence is not copied into unrestricted folders.
- Archive Control QAC-022: External auditor evidence uses read-only access.
- Archive Control QAC-023: Archive access follows tenant-scope policy.
- Archive Control QAC-024: Archive access follows auditor-scope policy.
- Archive Control QAC-025: Archive write access is limited to CSM and CS-IC roles.
- Archive Control QAC-026: Archive deletion follows retention policy.
- Archive Control QAC-027: Archive correction creates a new version.
- Archive Control QAC-028: Archive version history is retained.
- Archive Control QAC-029: Archive missing-field check runs before closeout.
- Archive Control QAC-030: Archive completion is recorded in customer success plan.
- Archive Control QAC-031: Archived QBR seeds next-quarter baseline.
- Archive Control QAC-032: Archived QBR informs renewal review.
- Archive Control QAC-033: Archived QBR informs expansion readiness.
- Archive Control QAC-034: Archived QBR informs support trend analysis.
- Archive Control QAC-035: Archived QBR is discoverable for approved tenant viewers.

## Cross-References

- `docs/GTM-PLAN.md` for customer success and GTM operating model.
- `docs/personas/MASTER-ROSTER-2026-05-21.md` for named QBR personas.
- `docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md` for QBR agenda and operating-review metrics.
- `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md` for incident response, SLO credit, and support evidence.
- `docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md` for revenue workflow metrics.
- `docs/standards/capability-tier-matrix.md` for package-specific SLA, support, RPO, RTO, retention, rate, and seat targets.
- `docs/standards/tenant-lifecycle.md` for Active, Suspended, Migrating, and Offboarded tenant states.
- `docs/standards/cedar-policy-authoring.md` for QBR evidence access policy design.
- `contracts/openapi/platform/platform-policy-cedar-v1.yaml` for publishing QBR access policies.
- `docs/gtm/tenant-onboarding-90-day-program.md` for Day 90 closeout and first QBR seed.
- `docs/gtm/expansion-and-upsell-playbook.md` for expansion signals and post-QBR commercial motion.
