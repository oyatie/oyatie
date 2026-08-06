---
doc_class: GoToMarketPlaybook
title: Migration From Incumbent Playbook
status: Draft
date: 2026-05-20
owner: GoToMarket / Solutions Engineering / Customer Success
related_oyatie_adrs:
  - docs/adr-archive/ADR-0009-cell-architecture-per-tenant-per-region.md
  - docs/adr-archive/ADR-0010-regional-pack-architecture.md
  - docs/adr-archive/ADR-0242-oyatie-is-a-tenant-doctrine.md
  - docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
related_personas:
  - Customer Success Manager Sofia Rezende
  - CS-IC Lin Chen
  - Sales AE Maya Lindqvist
  - Solutions Engineering
  - CTO Diego Vargas
  - COO Akira Watanabe
  - CFO Helena Brandt
  - CISO Yuki Park
  - Compliance Officer Tunde Bello
  - IT Manager Jamie O'Connor
  - Procurement Manager Wei Liu
  - Finance Director Mei-Ling Wu
  - Business Analyst Aditya Verma
  - Data Analyst Felipe Andrade
  - External Auditor Hyo-Jin Lee
related_journeys:
  - docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md
  - docs/user-journeys/j117-api-customer-tenant-incident-response/README.md
  - docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Migration From Incumbent Playbook

## Purpose

This playbook defines the GTM and delivery motion for moving a tenant from an incumbent system into Oyatie.

The generic sequence is extract, map, cut over, verify, and decommission.

The operational sequence also includes discovery, pre-flight, dry run, rollback, and post-cutover evidence.

The playbook begins when a prospect or active tenant identifies an incumbent source that must be migrated before value can be realized.

The playbook ends when the migrated scope is verified, users are live, the source system is decommissioned or retained by exception, and the QBR baseline reflects the migration outcome.

Migration is a trust event.

Migration must not be treated as a background technical chore.

Migration can affect data integrity, user trust, regulatory evidence, billing, workflow continuity, and executive confidence.

Every migration must have a named business owner, technical owner, data owner, security owner when needed, compliance owner when needed, and rollback owner.

Every migration must name source system, object types, data classes, mapping approach, validation method, cutover window, rollback window, and decommission plan.

Every migration must use approved tenant-scoped policies and explicit migration Cedar permits.

Every migration must preserve tenant isolation, jurisdiction, regulatory-pack boundaries, and audit evidence.

Every migration must distinguish sample migration, dry run, production migration, and cutover.

Every migration must avoid the anti-pattern of migrating first and regulating later.

Every migration must avoid skipping per-record class annotation when regulated or sensitive data exists.

Every migration must avoid single-shot migration without a dry run.

Every migration must avoid cutover without a read-only source window when source consistency matters.

Every migration must produce a customer-facing migration closeout.

This document supports presales POCs, onboarding migrations, expansion migration waves, and customer-success recovery migrations.

## Personas Involved (named — from MASTER-ROSTER)

- Customer Success Manager Sofia Rezende owns migration plan, customer coordination, risk register, and closeout.
- CS-IC Lin Chen owns migration task execution, evidence, dry-run coordination, and validation tracking.
- Sales AE Maya Lindqvist owns commercial scope, source-system assumptions, amendment needs, and expectation control.
- Solutions Engineering owns presales migration feasibility, POC migration scoping, and technical-evaluation evidence.
- CTO Diego Vargas owns architecture, integration, source access, target model, and technical signoff.
- COO Akira Watanabe owns operational continuity, cutover window, user-impact tolerance, and rollback acceptance.
- CFO Helena Brandt owns budget, incumbent cost overlap, service-credit exposure, and financial-data risk.
- CISO Yuki Park owns migration access, data-class controls, credentials, encryption, and policy approval.
- Compliance Officer Tunde Bello owns regulatory-pack mapping, jurisdiction, evidence retention, and audit readiness.
- IT Manager Jamie O'Connor owns source credentials, admin access, identity alignment, and user cutover logistics.
- Procurement Manager Wei Liu owns incumbent contract dependencies, termination rights, and vendor-risk updates.
- Finance Director Mei-Ling Wu owns invoice, payment, cost-center, ERP, or financial object migration risk.
- Business Analyst Aditya Verma owns process mapping and workflow validation.
- Data Analyst Felipe Andrade owns data profiling, field mapping, row-count validation, and metric lineage.
- External Auditor Hyo-Jin Lee owns audit evidence review when migration is regulated or audit-relevant.

## Stages / Steps (named, sequenced)

### Migration Stage 1: Discovery

- Stage 1 Exit Gate M1-G1: source, scope, owner, data class, and feasibility are documented.
- Step 1.01: CSM records migration reason.
- Step 1.02: CSM records source system.
- Step 1.03: CSM records target tenant_id.
- Step 1.04: CSM records tenant lifecycle state.
- Step 1.05: CSM records target capability tier.
- Step 1.06: CSM records target compliance packs.
- Step 1.07: CSM records jurisdiction and residency class.
- Step 1.08: CSM records business owner.
- Step 1.09: CSM records technical owner.
- Step 1.10: CSM records data owner.
- Step 1.11: CSM records security owner if needed.
- Step 1.12: CSM records compliance owner if needed.
- Step 1.13: CSM records rollback owner.
- Step 1.14: CS-IC records object types.
- Step 1.15: CS-IC records estimated object counts.
- Step 1.16: CS-IC records source export method.
- Step 1.17: CS-IC records source API availability.
- Step 1.18: CS-IC records source rate limits.
- Step 1.19: CS-IC records source data classes.
- Step 1.20: CS-IC records source retention limits.
- Step 1.21: CS-IC records source permissions needed.
- Step 1.22: CS-IC records source identity model.
- Step 1.23: CS-IC records source attachment model.
- Step 1.24: CS-IC records source workflow model.
- Step 1.25: CS-IC records source audit export model.
- Step 1.26: Data Analyst Felipe Andrade profiles source sample.
- Step 1.27: Business Analyst Aditya Verma maps business process.
- Step 1.28: CISO Yuki Park reviews credential and data-class risk.
- Step 1.29: Compliance Officer Tunde Bello reviews regulatory implications.
- Step 1.30: CSM sends discovery summary.

### Migration Stage 2: Pre-Flight

- Stage 2 Exit Gate M2-G1: migration plan, dry-run plan, policy plan, validation plan, and rollback plan are approved.
- Step 2.01: CSM opens migration plan.
- Step 2.02: CSM opens migration risk register.
- Step 2.03: CSM opens migration evidence folder.
- Step 2.04: CSM schedules migration workshop.
- Step 2.05: CSM confirms cutover target date.
- Step 2.06: CSM confirms read-only source window.
- Step 2.07: CSM confirms rollback window.
- Step 2.08: CSM confirms user communication plan.
- Step 2.09: CSM confirms support coverage plan.
- Step 2.10: CSM confirms executive notification path.
- Step 2.11: CS-IC confirms target tenant readiness.
- Step 2.12: CS-IC confirms target identity readiness.
- Step 2.13: CS-IC confirms target policy readiness.
- Step 2.14: CS-IC confirms target workflow readiness.
- Step 2.15: CS-IC confirms target audit stream readiness.
- Step 2.16: CS-IC confirms target storage or object readiness.
- Step 2.17: CS-IC confirms migration tools available.
- Step 2.18: CS-IC confirms source credentials.
- Step 2.19: CS-IC confirms source API quotas.
- Step 2.20: CS-IC confirms source export format.
- Step 2.21: Data Analyst confirms field mapping draft.
- Step 2.22: Business Analyst confirms process mapping draft.
- Step 2.23: CISO approves migration access policy.
- Step 2.24: Compliance approves regulated-data approach.
- Step 2.25: Finance reviews financial-object risk if in scope.
- Step 2.26: Procurement reviews incumbent contract constraints.
- Step 2.27: CSM records dry-run acceptance criteria.
- Step 2.28: CSM records cutover acceptance criteria.
- Step 2.29: CSM records decommission acceptance criteria.
- Step 2.30: CSM sends pre-flight approval request.

### Migration Stage 3: Extract

- Stage 3 Exit Gate M3-G1: source extract is complete, versioned, classified, and protected.
- Step 3.01: CS-IC executes `oya.migration.discover`.
- Step 3.02: CS-IC records discovery run id.
- Step 3.03: CS-IC reviews object inventory.
- Step 3.04: CS-IC reviews source warnings.
- Step 3.05: CS-IC reviews rate-limit warnings.
- Step 3.06: CS-IC reviews permission warnings.
- Step 3.07: CS-IC executes `oya.migration.export`.
- Step 3.08: CS-IC records export run id.
- Step 3.09: CS-IC records object counts.
- Step 3.10: CS-IC records skipped objects.
- Step 3.11: CS-IC records failed objects.
- Step 3.12: CS-IC records source timestamps.
- Step 3.13: CS-IC records extract checksum.
- Step 3.14: CS-IC records data class annotations.
- Step 3.15: CS-IC records storage location.
- Step 3.16: CS-IC confirms tenant-scoped access.
- Step 3.17: CS-IC confirms no unauthorized principals have access.
- Step 3.18: CS-IC confirms encryption expectation.
- Step 3.19: Data Analyst reviews sample records.
- Step 3.20: Data Analyst reviews null rates.
- Step 3.21: Data Analyst reviews duplicate rates.
- Step 3.22: Data Analyst reviews required-field presence.
- Step 3.23: Business Analyst reviews business object completeness.
- Step 3.24: CISO reviews extract access evidence.
- Step 3.25: Compliance reviews regulated-data evidence.
- Step 3.26: CSM updates risk register.
- Step 3.27: CSM reports extraction result.
- Step 3.28: CSM blocks mapping if extract is incomplete and not accepted.
- Step 3.29: CSM records accepted extract exceptions.
- Step 3.30: CSM moves to mapping.

### Migration Stage 4: Map

- Stage 4 Exit Gate M4-G1: source objects are mapped to target objects with validation criteria and exception handling.
- Step 4.01: CS-IC executes `oya.migration.transform` in dry-run mode.
- Step 4.02: CS-IC records transform run id.
- Step 4.03: Data Analyst reviews field mapping.
- Step 4.04: Data Analyst reviews enum mapping.
- Step 4.05: Data Analyst reviews identity mapping.
- Step 4.06: Data Analyst reviews attachment mapping.
- Step 4.07: Data Analyst reviews date and timezone mapping.
- Step 4.08: Data Analyst reviews currency mapping.
- Step 4.09: Data Analyst reviews permission mapping.
- Step 4.10: Data Analyst reviews data-class mapping.
- Step 4.11: Business Analyst reviews process mapping.
- Step 4.12: Business Analyst reviews workflow mapping.
- Step 4.13: Business Analyst reviews owner mapping.
- Step 4.14: Business Analyst reviews status mapping.
- Step 4.15: Business Analyst reviews exception path.
- Step 4.16: CS-IC records unmapped fields.
- Step 4.17: CS-IC records ambiguous fields.
- Step 4.18: CS-IC records lossy transforms.
- Step 4.19: CS-IC records derived fields.
- Step 4.20: CS-IC records target object counts.
- Step 4.21: CISO reviews permission mapping.
- Step 4.22: Compliance reviews regulated mapping.
- Step 4.23: Finance reviews financial object mapping if in scope.
- Step 4.24: Customer owner approves mapping.
- Step 4.25: CSM records accepted mapping exceptions.
- Step 4.26: CSM records rejected mapping exceptions.
- Step 4.27: CSM blocks import if critical mapping remains unresolved.
- Step 4.28: CSM sends mapping approval summary.
- Step 4.29: CSM schedules dry-run import.
- Step 4.30: CSM moves to dry run.

### Migration Stage 5: Dry Run and Import

- Stage 5 Exit Gate M5-G1: dry-run import passes validation or exceptions are explicitly accepted.
- Step 5.01: CS-IC executes `oya.migration.import` in dry-run target.
- Step 5.02: CS-IC records import run id.
- Step 5.03: CS-IC records imported object counts.
- Step 5.04: CS-IC records failed object counts.
- Step 5.05: CS-IC records skipped object counts.
- Step 5.06: CS-IC records duplicate object counts.
- Step 5.07: CS-IC records attachment import result.
- Step 5.08: CS-IC records identity import result.
- Step 5.09: CS-IC records workflow import result.
- Step 5.10: CS-IC records audit import result.
- Step 5.11: CS-IC executes `oya.migration.validate`.
- Step 5.12: CS-IC records validation run id.
- Step 5.13: Data Analyst reviews count validation.
- Step 5.14: Data Analyst reviews field validation.
- Step 5.15: Data Analyst reviews sample record validation.
- Step 5.16: Data Analyst reviews metric validation.
- Step 5.17: Business Analyst reviews process validation.
- Step 5.18: Business Analyst reviews workflow validation.
- Step 5.19: IT Manager reviews access validation.
- Step 5.20: CISO reviews permission validation.
- Step 5.21: Compliance reviews evidence validation.
- Step 5.22: CSM records validation pass items.
- Step 5.23: CSM records validation fail items.
- Step 5.24: CSM records accepted exceptions.
- Step 5.25: CSM records remediation tasks.
- Step 5.26: CSM blocks cutover if critical validation fails.
- Step 5.27: CSM approves cutover readiness if validation passes.
- Step 5.28: CSM sends dry-run readout.
- Step 5.29: CSM confirms cutover window.
- Step 5.30: CSM moves to cutover.

### Migration Stage 6: Cut Over

- Stage 6 Exit Gate M6-G1: source is frozen or read-only, final import runs, target validation passes, and users are routed to Oyatie.
- Step 6.01: CSM confirms cutover go/no-go.
- Step 6.02: COO confirms operational window.
- Step 6.03: IT Manager confirms source read-only.
- Step 6.04: CSM sends user cutover notice.
- Step 6.05: Support team enters migration watch.
- Step 6.06: CS-IC executes final discovery delta.
- Step 6.07: CS-IC executes final export delta.
- Step 6.08: CS-IC executes final transform delta.
- Step 6.09: CS-IC executes final import.
- Step 6.10: CS-IC records final import id.
- Step 6.11: CS-IC executes final validation.
- Step 6.12: CS-IC records validation id.
- Step 6.13: Data Analyst verifies final counts.
- Step 6.14: Business Analyst verifies top workflows.
- Step 6.15: IT Manager verifies user access.
- Step 6.16: CISO verifies access policy.
- Step 6.17: Compliance verifies evidence posture.
- Step 6.18: Finance verifies financial objects if in scope.
- Step 6.19: CSM confirms rollback decision point.
- Step 6.20: CSM records go-live approval.
- Step 6.21: CSM sends go-live notice.
- Step 6.22: CS-IC monitors errors.
- Step 6.23: CS-IC monitors support tickets.
- Step 6.24: CS-IC monitors policy denies.
- Step 6.25: CS-IC monitors workflow completion.
- Step 6.26: CSM updates risk register.
- Step 6.27: CSM updates executive sponsor.
- Step 6.28: CSM decides whether rollback window remains open.
- Step 6.29: CSM closes cutover window when stable.
- Step 6.30: CSM moves to verify.

### Migration Stage 7: Verify

- Stage 7 Exit Gate M7-G1: customer accepts migrated data, workflows, access, evidence, and operational readiness.
- Step 7.01: Data Analyst verifies record counts.
- Step 7.02: Data Analyst verifies required fields.
- Step 7.03: Data Analyst verifies sample records.
- Step 7.04: Data Analyst verifies metric lineage.
- Step 7.05: Business Analyst verifies business workflow.
- Step 7.06: Business Analyst verifies process status.
- Step 7.07: Business Analyst verifies owner assignment.
- Step 7.08: IT Manager verifies user login.
- Step 7.09: IT Manager verifies admin access.
- Step 7.10: IT Manager verifies source read-only remains in effect.
- Step 7.11: CISO verifies policy allow tests.
- Step 7.12: CISO verifies policy deny tests.
- Step 7.13: CISO verifies no unauthorized principal.
- Step 7.14: Compliance verifies regulated evidence.
- Step 7.15: Compliance verifies retention expectation.
- Step 7.16: Finance verifies financial objects if in scope.
- Step 7.17: COO verifies operational continuity.
- Step 7.18: CTO verifies technical stability.
- Step 7.19: CSM verifies support watch results.
- Step 7.20: CSM verifies customer sentiment.
- Step 7.21: CSM records validation acceptance.
- Step 7.22: CSM records validation exceptions.
- Step 7.23: CSM records remediation owners.
- Step 7.24: CSM records rollback close decision.
- Step 7.25: CSM records decommission recommendation.
- Step 7.26: CSM sends verification readout.
- Step 7.27: Customer accepts or rejects migration.
- Step 7.28: CSM escalates rejected migration.
- Step 7.29: CSM moves accepted migration to decommission.
- Step 7.30: CSM updates QBR baseline.

### Migration Stage 8: Decommission

- Stage 8 Exit Gate M8-G1: incumbent source is decommissioned or retained by explicit exception with owner and review date.
- Step 8.01: Procurement reviews incumbent termination terms.
- Step 8.02: Finance reviews duplicate-cost period.
- Step 8.03: IT Manager confirms source access removal plan.
- Step 8.04: IT Manager confirms source archival plan.
- Step 8.05: Compliance reviews retention requirement.
- Step 8.06: CISO reviews credential revocation.
- Step 8.07: External Auditor reviews evidence if required.
- Step 8.08: CSM confirms decommission owner.
- Step 8.09: CSM confirms decommission date.
- Step 8.10: CSM confirms decommission exception if any.
- Step 8.11: CS-IC archives migration evidence.
- Step 8.12: CS-IC archives mapping version.
- Step 8.13: CS-IC archives validation run ids.
- Step 8.14: CS-IC archives cutover decision.
- Step 8.15: CS-IC archives support watch results.
- Step 8.16: IT Manager revokes source credentials.
- Step 8.17: IT Manager disables source user access where approved.
- Step 8.18: Procurement closes incumbent vendor task where approved.
- Step 8.19: Finance confirms cost stop or overlap plan.
- Step 8.20: CSM sends migration closeout.
- Step 8.21: CSM updates QBR migration section.
- Step 8.22: CSM updates expansion signals.
- Step 8.23: CSM updates renewal risk.
- Step 8.24: CSM updates account path.
- Step 8.25: CSM closes migration plan.

### Per-Vendor Specialization References

- Vendor Reference VR-001: Google Workspace migration specializes mail, drive, calendar, identity, and document permission mapping.
- Vendor Reference VR-002: Microsoft 365 migration specializes Exchange, SharePoint, Teams, OneDrive, Entra identity, and retention labels.
- Vendor Reference VR-003: Naver Works migration specializes Korean business collaboration, mail, calendar, contacts, and local admin patterns.
- Vendor Reference VR-004: Kakao Work migration specializes chat, workplace collaboration, local identity, and message retention.
- Vendor Reference VR-005: Notion migration specializes pages, databases, relations, comments, and workspace permissions.
- Vendor Reference VR-006: Slack migration specializes channels, messages, files, apps, user identity, and retention.
- Vendor Reference VR-007: Zoom migration specializes meetings, recordings, transcripts, webinar objects, and user identity.
- Vendor Reference VR-008: Asana migration specializes projects, tasks, subtasks, custom fields, attachments, and assignees.
- Vendor Reference VR-009: Trello migration specializes boards, lists, cards, labels, attachments, and comments.
- Vendor Reference VR-010: Jira migration specializes projects, issues, workflows, statuses, fields, boards, and permissions.
- Vendor Reference VR-011: Linear migration specializes teams, issues, cycles, projects, labels, and workflow states.
- Vendor Reference VR-012: Salesforce migration specializes accounts, contacts, opportunities, cases, activities, ownership, and field history.
- Vendor Reference VR-013: HubSpot migration specializes companies, contacts, deals, tickets, marketing lists, and lifecycle stages.
- Vendor Reference VR-014: Workday migration specializes worker profiles, organizations, roles, HR objects, and sensitive HR controls.
- Vendor Reference VR-015: Douzone migration specializes Korean ERP, accounting, tax, payroll, and financial evidence.
- Vendor Reference VR-016: Younglimwon migration specializes Korean ERP, manufacturing, accounting, and operational data.
- Vendor Reference VR-017: SAP migration specializes finance, procurement, inventory, customer, vendor, and master-data mapping.
- Vendor Reference VR-018: Epic migration specializes healthcare records, encounter data, clinical attachments, and regulated audit trail.
- Vendor Reference VR-019: Cerner migration specializes healthcare records, clinical workflows, identity, and regulated data.
- Vendor Reference VR-020: Korean EMR migration specializes local healthcare records, consent, retention, and regulator-sensitive evidence.
- Vendor Reference VR-021: Toss Payments migration specializes payment records, settlement, refunds, customer identifiers, and audit evidence.
- Vendor Reference VR-022: KakaoPay migration specializes wallet payments, transaction records, refund events, and settlement data.
- Vendor Reference VR-023: NaverPay migration specializes payment records, merchant settlement, refunds, and customer identifiers.
- Vendor Reference VR-024: Adyen migration specializes payment transactions, disputes, settlement, tokenization, and reconciliation.
- Vendor Reference VR-025: Stripe migration specializes customers, subscriptions, invoices, charges, refunds, and payment method constraints.
- Vendor Reference VR-026: Braintree migration specializes payment method tokens, transactions, customers, and subscription mapping.
- Vendor Reference VR-027: Manhattan WMS migration specializes warehouse locations, inventory, orders, waves, and fulfillment events.
- Vendor Reference VR-028: Blue Yonder migration specializes supply-chain planning, inventory, demand, and fulfillment objects.
- Vendor Reference VR-029: Oracle WMS migration specializes warehouse, inventory, order, and fulfillment master data.
- Vendor Reference VR-030: Procore migration specializes construction projects, documents, RFIs, submittals, and field permissions.
- Vendor Reference VR-031: Autodesk Construction migration specializes project files, models, issues, approvals, and field data.
- Vendor Reference VR-032: Canvas migration specializes courses, enrollments, assignments, grades, and learning artifacts.
- Vendor Reference VR-033: Blackboard migration specializes courses, users, assignments, grades, and institutional records.
- Vendor Reference VR-034: Google Classroom migration specializes classes, coursework, submissions, grades, and student identity.

## Named Tools and Cedar Permits (specific µservices + actions + Cedar policies)

### Migration Tools

- Tool: `migration-discovery-service`; Action: `oya.migration.discover`; Use: enumerate source systems, objects, counts, and constraints.
- Tool: `migration-export-service`; Action: `oya.migration.export`; Use: extract approved source data.
- Tool: `migration-transform-service`; Action: `oya.migration.transform`; Use: map source objects to Oyatie target model.
- Tool: `migration-import-service`; Action: `oya.migration.import`; Use: import mapped data into target tenant.
- Tool: `migration-validate-service`; Action: `oya.migration.validate`; Use: validate counts, fields, samples, permissions, and workflow fit.
- Tool: `migration-cutover-service`; Action: `oya.migration.cutover`; Use: manage cutover, delta import, and rollback window.
- Tool: `platform-tenant-service`; Action: `platform-tenant:GetTenant`; Use: verify target tenant state, cell, jurisdiction, and regulatory packs.
- Tool: `identity-service`; Action: `identity:BindTenantPrincipal`; Use: bind migration operators, customer validators, and auditors.
- Tool: `platform-policy-cedar-service`; Action: `platform-policy:publishCedarPolicy`; Use: publish migration permit set.
- Tool: `audit-chain-service`; Action: `audit-chain:ReadEvidence`; Use: retrieve migration evidence.
- Tool: `workflow-engine`; Action: `workflow-engine:RunWorkflow`; Use: validate migrated workflow behavior.
- Tool: `support-service`; Action: `support:OpenMigrationWatch`; Use: monitor cutover support.
- Tool: `finops-portal`; Action: `finops-portal:ReadTenantCostCenter`; Use: monitor duplicate-cost and incumbent overlap.
- Tool: `messenger-service`; Action: `messenger:NotifyTenantContact`; Use: cutover and rollback notices.
- Tool: `mail-service`; Action: `mail:SendTenantMessage`; Use: migration recap and user communication.

### Migration Cedar Permits

- Cedar Permit: `cedar.migration.discovery.operator`; Allows `oya.migration.discover` for named source and tenant.
- Cedar Permit: `cedar.migration.export.operator`; Allows `oya.migration.export` only for approved source objects.
- Cedar Permit: `cedar.migration.transform.operator`; Allows `oya.migration.transform` for approved mapping version.
- Cedar Permit: `cedar.migration.import.dry_run_operator`; Allows dry-run import to non-production target.
- Cedar Permit: `cedar.migration.import.final_operator`; Allows final import only after pre-flight and dry-run gates pass.
- Cedar Permit: `cedar.migration.validate.operator`; Allows validation reads on source extract and target tenant.
- Cedar Permit: `cedar.migration.cutover.operator`; Allows cutover only during approved window.
- Cedar Permit: `cedar.migration.rollback.operator`; Allows rollback only before rollback window closes.
- Cedar Permit: `cedar.migration.decommission.operator`; Allows decommission actions only after acceptance.
- Cedar Permit: `cedar.migration.source_credential.reader`; Allows source credential retrieval for named operator.
- Cedar Permit: `cedar.migration.source_credential.revoker`; Allows source credential revocation during decommission.
- Cedar Permit: `cedar.migration.customer_validator.viewer`; Allows customer validators to inspect migrated records.
- Cedar Permit: `cedar.migration.data_analyst.validator`; Allows Data Analyst Felipe Andrade to validate mapped data.
- Cedar Permit: `cedar.migration.business_analyst.validator`; Allows Business Analyst Aditya Verma to validate workflow fit.
- Cedar Permit: `cedar.migration.security_reviewer.reader`; Allows CISO Yuki Park to inspect access and policy evidence.
- Cedar Permit: `cedar.migration.compliance_reviewer.reader`; Allows Tunde Bello to inspect regulated evidence.
- Cedar Permit: `cedar.migration.external_auditor.reader`; Allows External Auditor Hyo-Jin Lee read-only evidence review.
- Cedar Permit: `cedar.migration.no_regulated_data_without_pack`; Denies regulated-data movement without active pack and data class.
- Cedar Permit: `cedar.migration.no_cutover_without_validation`; Denies cutover until validation gate passes.
- Cedar Permit: `cedar.migration.no_decommission_without_acceptance`; Denies decommission until customer acceptance exists.

## Specific Metrics + Named SLA Targets

- Metric: Migration Discovery Completion; Target: source inventory complete before pre-flight approval.
- Metric: Migration Pre-Flight Completion; Target: plan, policy, validation, rollback, and communication complete before extract.
- Metric: Extract Completeness; Target: 100 percent of in-scope objects extracted or exception accepted.
- Metric: Extract Checksum Coverage; Target: checksum or equivalent integrity evidence for every extract batch.
- Metric: Data-Class Annotation Coverage; Target: 100 percent for regulated or sensitive object classes.
- Metric: Mapping Completeness; Target: 100 percent required fields mapped or exception accepted.
- Metric: Lossy Transform Count; Target: zero unapproved lossy transforms.
- Metric: Dry-Run Import Completion; Target: dry run completed before production cutover.
- Metric: Dry-Run Validation Pass Rate; Target: 99 percent for required records unless exception accepted.
- Metric: Cutover Go/No-Go Timeliness; Target: decision recorded before cutover window opens.
- Metric: Final Import Completion; Target: final import completes within approved cutover window.
- Metric: Final Validation Completion; Target: validation completes before user go-live notice.
- Metric: Rollback Decision Timeliness; Target: rollback decision point recorded during cutover window.
- Metric: User Access Verification; Target: top user cohorts verified before go-live.
- Metric: Workflow Verification; Target: top workflows pass before cutover close.
- Metric: Policy Allow-Deny Verification; Target: allow and deny evidence recorded before acceptance.
- Metric: Migration Support Watch Duration; Target: at least one agreed watch period after go-live.
- Metric: Decommission Decision Timeliness; Target: decommission or retention exception recorded within thirty days after acceptance.
- Metric: Evidence Archive Completeness; Target: migration plan, mapping, validation, cutover, and acceptance evidence archived.
- Metric: QBR Baseline Update; Target: migration outcome included in next QBR pack.
- SLA Target: Migration recap sent within two business days after cutover close.

## Named Failure Modes + Recovery

- Failure Mode: `MIGRATION-NO-OWNER`; Signal: no business, technical, data, or rollback owner; Recovery: block pre-flight approval.
- Failure Mode: `MIGRATION-SOURCE-UNKNOWN`; Signal: source system scope is vague; Recovery: run discovery workshop before estimating.
- Failure Mode: `MIGRATION-DATA-CLASS-MISSING`; Signal: records lack data class; Recovery: annotate before export or block regulated movement.
- Failure Mode: `MIGRATION-CREDENTIAL-RISK`; Signal: source credentials are shared, stale, or overbroad; Recovery: issue scoped credentials and log access.
- Failure Mode: `MIGRATION-RATE-LIMIT-BLOCK`; Signal: source API quotas prevent extract; Recovery: adjust schedule, batch plan, or vendor request.
- Failure Mode: `MIGRATION-EXTRACT-INCOMPLETE`; Signal: object counts do not match source; Recovery: rerun export or accept named exception.
- Failure Mode: `MIGRATION-MAPPING-LOSSY`; Signal: target cannot preserve source field; Recovery: approve transform, create custom mapping, or exclude field.
- Failure Mode: `MIGRATION-PERMISSION-DRIFT`; Signal: migrated permissions exceed intended target access; Recovery: block import and revise mapping.
- Failure Mode: `MIGRATION-DRY-RUN-SKIPPED`; Signal: team attempts cutover without dry run; Recovery: stop cutover and run dry-run gate.
- Failure Mode: `MIGRATION-VALIDATION-FAIL`; Signal: counts, fields, samples, or workflows fail; Recovery: remediate and rerun validation.
- Failure Mode: `MIGRATION-CUTOVER-NO-READONLY`; Signal: source remains writable during final export; Recovery: pause cutover or accept delta-risk explicitly.
- Failure Mode: `MIGRATION-ROLLBACK-UNCLEAR`; Signal: no rollback owner or decision point; Recovery: block go/no-go.
- Failure Mode: `MIGRATION-USER-ACCESS-FAIL`; Signal: users cannot access migrated target; Recovery: run identity fix and extend support watch.
- Failure Mode: `MIGRATION-WORKFLOW-BREAK`; Signal: migrated objects do not support target workflow; Recovery: process mapping remediation.
- Failure Mode: `MIGRATION-AUDIT-GAP`; Signal: audit evidence missing for regulated migration; Recovery: collect evidence before acceptance.
- Failure Mode: `MIGRATION-DECOMMISSION-PREMATURE`; Signal: source shutdown before verification; Recovery: restore from rollback or backup if possible.
- Failure Mode: `MIGRATION-DECOMMISSION-ORPHAN`; Signal: source remains live with no owner; Recovery: assign owner or retention exception.
- Failure Mode: `MIGRATION-COST-OVERLAP`; Signal: incumbent and Oyatie costs overlap beyond plan; Recovery: Finance and Procurement review.
- Failure Mode: `MIGRATION-CUSTOMER-REJECTS`; Signal: customer rejects migrated state; Recovery: reopen validation, triage defects, and decide rollback.
- Failure Mode: `MIGRATION-QBR-MISSING`; Signal: migration outcome absent from QBR; Recovery: update QBR pack and baseline.

## Sample Dialogue / Email Templates

### Template 1: Migration Discovery Request

Subject: Migration discovery inputs for {{tenant_name}}

Hi {{customer_owner}},

To scope the migration from {{source_system}} into Oyatie, we need the following inputs:

1. Source system owner.
2. Object types in scope.
3. Estimated record counts.
4. Data classes and regulated data flags.
5. Export or API access method.
6. Target cutover window.
7. Rollback owner.

We will use these inputs to produce the migration plan, policy plan, validation plan, and dry-run schedule.

Regards,

{{csm_name}}

### Template 2: Dry-Run Readout

Subject: Migration dry-run readout for {{tenant_name}}

Hi {{customer_team}},

The migration dry run from {{source_system}} is complete.

Results:

- Extracted objects: {{extracted_count}}
- Imported objects: {{imported_count}}
- Validation pass rate: {{validation_pass_rate}}
- Exceptions: {{exception_count}}

Open remediation:

- {{remediation_1}} - owner: {{owner_1}} - due: {{date_1}}
- {{remediation_2}} - owner: {{owner_2}} - due: {{date_2}}

Cutover remains blocked until critical validation failures are resolved or explicitly accepted.

Regards,

{{csic_name}}

### Template 3: Cutover Go/No-Go

Subject: Migration cutover go/no-go for {{tenant_name}}

Hi {{customer_team}},

We are at the cutover decision point for {{source_system}}.

Go criteria:

- Source read-only window confirmed.
- Final export access confirmed.
- Rollback owner confirmed.
- Support watch confirmed.
- Validation criteria accepted.

Current recommendation:

- {{go_or_no_go}}

Decision owner:

- {{decision_owner}}

Please confirm the cutover decision by {{deadline}}.

Regards,

{{csm_name}}

### Template 4: Migration Acceptance

Subject: Migration verification acceptance for {{tenant_name}}

Hi {{customer_owner}},

The final migration validation is complete.

Accepted:

- {{accepted_item_1}}
- {{accepted_item_2}}
- {{accepted_item_3}}

Exceptions:

- {{exception_1}} - owner: {{owner_1}} - due: {{date_1}}

Please confirm whether Oyatie should proceed to decommission planning or keep the incumbent source under a retention exception.

Regards,

{{csm_name}}

### Template 5: Decommission Closeout

Subject: Incumbent decommission closeout for {{tenant_name}}

Hi {{customer_team}},

The migration from {{source_system}} to Oyatie is closed.

Decommission status:

- {{decommission_status}}

Evidence archived:

- Migration plan.
- Mapping version.
- Validation results.
- Cutover decision.
- Customer acceptance.

Next QBR update:

- Migration outcome and any remaining exceptions will be included in the next QBR pack.

Regards,

{{csm_name}}

## Migration Control Appendix

### Source Access Controls

- Source Access Control SAC-001: Source owner is named before credentials are requested.
- Source Access Control SAC-002: Credential scope is limited to approved object classes.
- Source Access Control SAC-003: Credential expiry is recorded.
- Source Access Control SAC-004: Credential storage location is recorded.
- Source Access Control SAC-005: Credential reader policy is published before use.
- Source Access Control SAC-006: Credential revoker is named before cutover.
- Source Access Control SAC-007: Source admin access is not shared through personal accounts.
- Source Access Control SAC-008: Source API quota is recorded before extraction.
- Source Access Control SAC-009: Source export format is recorded before extraction.
- Source Access Control SAC-010: Source rate-limit mitigation is recorded before extraction.
- Source Access Control SAC-011: Source read-only window is recorded before cutover.
- Source Access Control SAC-012: Source audit export capability is recorded when regulated.
- Source Access Control SAC-013: Source backup status is recorded before cutover.
- Source Access Control SAC-014: Source retention requirement is recorded before decommission.
- Source Access Control SAC-015: Source access revocation evidence is archived.

### Data Classification Controls

- Data Classification Control DCC-001: Every object class has data-class owner.
- Data Classification Control DCC-002: Regulated object classes are marked before export.
- Data Classification Control DCC-003: Sensitive object classes are marked before export.
- Data Classification Control DCC-004: Public object classes are still tenant-scoped.
- Data Classification Control DCC-005: Unknown data class blocks production migration.
- Data Classification Control DCC-006: Data-class mapping is reviewed by CISO when sensitive.
- Data Classification Control DCC-007: Data-class mapping is reviewed by Compliance when regulated.
- Data Classification Control DCC-008: Data-class mapping is included in validation evidence.
- Data Classification Control DCC-009: Data-class mapping is included in QBR when audit-relevant.
- Data Classification Control DCC-010: Data-class changes after dry run trigger remapping.
- Data Classification Control DCC-011: Data-class exceptions require named approver.
- Data Classification Control DCC-012: Data-class exceptions require expiry or review date.
- Data Classification Control DCC-013: Data-class exceptions are archived with migration evidence.
- Data Classification Control DCC-014: Data-class gaps block decommission.
- Data Classification Control DCC-015: Data-class summary is included in closeout.

### Mapping Controls

- Mapping Control MAP-001: Field map has version id.
- Mapping Control MAP-002: Field map has owner.
- Mapping Control MAP-003: Field map has approval date.
- Mapping Control MAP-004: Required fields are marked.
- Mapping Control MAP-005: Optional fields are marked.
- Mapping Control MAP-006: Dropped fields are listed.
- Mapping Control MAP-007: Derived fields are explained.
- Mapping Control MAP-008: Enum mappings are listed.
- Mapping Control MAP-009: Date mappings include timezone.
- Mapping Control MAP-010: Currency mappings include currency code.
- Mapping Control MAP-011: Identity mappings include principal source.
- Mapping Control MAP-012: Permission mappings include allow and deny expectations.
- Mapping Control MAP-013: Attachment mappings include storage target.
- Mapping Control MAP-014: Workflow mappings include status transitions.
- Mapping Control MAP-015: Audit mappings include evidence target.
- Mapping Control MAP-016: Lossy transforms require approval.
- Mapping Control MAP-017: Ambiguous fields require owner decision.
- Mapping Control MAP-018: Mapping changes after dry run require new validation.
- Mapping Control MAP-019: Mapping version used for cutover is archived.
- Mapping Control MAP-020: Mapping summary is included in migration closeout.

### Cutover Controls

- Cutover Control COC-001: Cutover window is approved by COO owner.
- Cutover Control COC-002: Cutover window is communicated to users.
- Cutover Control COC-003: Cutover go/no-go owner is named.
- Cutover Control COC-004: Rollback owner is named.
- Cutover Control COC-005: Rollback decision point is scheduled.
- Cutover Control COC-006: Source read-only window is active before final export.
- Cutover Control COC-007: Support watch is staffed before cutover begins.
- Cutover Control COC-008: Executive notification path is ready before cutover.
- Cutover Control COC-009: Final export run id is recorded.
- Cutover Control COC-010: Final import run id is recorded.
- Cutover Control COC-011: Final validation run id is recorded.
- Cutover Control COC-012: Go-live notice is sent only after validation.
- Cutover Control COC-013: Rollback remains available until decision point closes.
- Cutover Control COC-014: Cutover close is recorded.
- Cutover Control COC-015: Cutover exception is assigned owner.
- Cutover Control COC-016: User-impact issues are triaged during support watch.
- Cutover Control COC-017: Policy-deny issues are triaged during support watch.
- Cutover Control COC-018: Workflow issues are triaged during support watch.
- Cutover Control COC-019: Data issues are triaged during support watch.
- Cutover Control COC-020: Cutover lessons are captured.

### Validation Controls

- Validation Control VAC-001: Count validation compares source and target object counts.
- Validation Control VAC-002: Field validation checks required target fields.
- Validation Control VAC-003: Sample validation checks customer-selected records.
- Validation Control VAC-004: Workflow validation checks top customer workflow.
- Validation Control VAC-005: Access validation checks allowed users.
- Validation Control VAC-006: Deny validation checks blocked users.
- Validation Control VAC-007: Attachment validation checks file presence.
- Validation Control VAC-008: Identity validation checks principal mapping.
- Validation Control VAC-009: Audit validation checks evidence retrieval.
- Validation Control VAC-010: Financial validation checks financial totals when in scope.
- Validation Control VAC-011: Regulated validation checks pack and data-class rules when in scope.
- Validation Control VAC-012: Metric validation checks dashboard lineage.
- Validation Control VAC-013: Exception validation checks accepted exceptions.
- Validation Control VAC-014: Customer validation records acceptance.
- Validation Control VAC-015: Validation failure records remediation owner.
- Validation Control VAC-016: Validation failure records due date.
- Validation Control VAC-017: Validation failure blocks decommission unless accepted.
- Validation Control VAC-018: Validation pass evidence is archived.
- Validation Control VAC-019: Validation summary is included in customer closeout.
- Validation Control VAC-020: Validation summary is included in QBR baseline.

### Vendor Lane Controls

- Vendor Lane Control VLC-001: Collaboration vendors require permission and attachment mapping review.
- Vendor Lane Control VLC-002: Messaging vendors require retention and channel-membership review.
- Vendor Lane Control VLC-003: CRM vendors require owner, lifecycle, and activity history review.
- Vendor Lane Control VLC-004: ERP vendors require financial control and audit review.
- Vendor Lane Control VLC-005: HRIS vendors require sensitive HR data review.
- Vendor Lane Control VLC-006: Healthcare vendors require regulated clinical-data review.
- Vendor Lane Control VLC-007: Payment vendors require token, settlement, refund, and reconciliation review.
- Vendor Lane Control VLC-008: WMS vendors require inventory, location, and fulfillment review.
- Vendor Lane Control VLC-009: Construction vendors require project, document, approval, and field-permission review.
- Vendor Lane Control VLC-010: Education vendors require student identity, grades, and institutional record review.
- Vendor Lane Control VLC-011: Salesforce lane requires field-history decision.
- Vendor Lane Control VLC-012: HubSpot lane requires lifecycle-stage mapping.
- Vendor Lane Control VLC-013: SAP lane requires master-data ownership.
- Vendor Lane Control VLC-014: Workday lane requires HR security owner approval.
- Vendor Lane Control VLC-015: Google Workspace lane requires drive permission inheritance check.
- Vendor Lane Control VLC-016: Microsoft 365 lane requires SharePoint and Teams retention check.
- Vendor Lane Control VLC-017: Slack lane requires private-channel export decision.
- Vendor Lane Control VLC-018: Notion lane requires relation and database schema mapping.
- Vendor Lane Control VLC-019: Jira lane requires workflow status mapping.
- Vendor Lane Control VLC-020: Stripe lane requires payment method migration limits.

### Decommission Controls

- Decommission Control DEC-001: Customer acceptance exists before decommission.
- Decommission Control DEC-002: Source retention requirement is reviewed before decommission.
- Decommission Control DEC-003: Incumbent contract terms are reviewed before decommission.
- Decommission Control DEC-004: Duplicate-cost impact is reviewed before decommission.
- Decommission Control DEC-005: User access removal is approved before decommission.
- Decommission Control DEC-006: Source credentials are revoked after decommission.
- Decommission Control DEC-007: Source admin accounts are reviewed after decommission.
- Decommission Control DEC-008: Archive location is recorded after decommission.
- Decommission Control DEC-009: Retention exception has owner and review date.
- Decommission Control DEC-010: Decommission exception appears in QBR risk register.
- Decommission Control DEC-011: Procurement confirms vendor close task.
- Decommission Control DEC-012: Finance confirms cost-stop or overlap plan.
- Decommission Control DEC-013: IT confirms access removal.
- Decommission Control DEC-014: Compliance confirms retention evidence.
- Decommission Control DEC-015: CISO confirms credential revocation.
- Decommission Control DEC-016: External Auditor confirms evidence when required.
- Decommission Control DEC-017: Decommission closeout is sent to customer.
- Decommission Control DEC-018: Decommission evidence is archived.
- Decommission Control DEC-019: Decommission status updates QBR.
- Decommission Control DEC-020: Decommission closes migration plan.

### Migration Closeout Decision Rules

- Closeout Rule MCR-001: Close as Complete only when validation is accepted and decommission status is resolved.
- Closeout Rule MCR-002: Close as Complete With Exceptions only when exceptions have owners and review dates.
- Closeout Rule MCR-003: Close as Blocked when customer acceptance, validation, or source access remains unresolved.
- Closeout Rule MCR-004: Close as Rolled Back when target activation is reversed inside the rollback window.
- Closeout Rule MCR-005: Closeout status is included in the next QBR and renewal-risk review.

## Cross-References

- `docs/standards/migration-playbook.md` for migration phases, vendor families, importer capabilities, and anti-patterns.
- `docs/GTM-PLAN.md` for GTM, implementation, customer success, and packaging context.
- `docs/personas/MASTER-ROSTER-2026-05-21.md` for named migration stakeholders.
- `specs/tenant-model.json` for tenant fields affecting migration readiness.
- `specs/tenant-lifecycle.json` for Migrating, Active, Offboarded, and DeletionConfirmed states.
- `docs/standards/tenant-lifecycle.md` for tenant lifecycle transitions and migration governance.
- `docs/standards/cedar-policy-authoring.md` for migration permits, regulated-data guards, and destructive-action rules.
- `contracts/openapi/platform/platform-tenant-v1.yaml` for target tenant inspection and residency fields.
- `contracts/openapi/platform/platform-policy-cedar-v1.yaml` for migration policy publication.
- `docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md` for revenue workflow migration examples.
- `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md` for cutover support and incident readiness.
- `docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md` for QBR migration outcome review.
- `docs/gtm/solutions-engineering-runbook.md` for presales POC migration scoping.
- `docs/gtm/tenant-onboarding-90-day-program.md` for onboarding migration milestones.
- `docs/gtm/customer-success-quarterly-business-review-template.md` for post-migration QBR baseline.
